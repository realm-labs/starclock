//! Production-backed transfer, owner-turn decay and actual-loss healing in both families.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_transfer_fixture::{
            ABSORB, GRANT, ORDINARY, Probe, REMOVE, SHRINK_HP, SPECIAL, SUMMON, accept, cast, id,
            idle, scenario, unit,
        },
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    Battle, BattleEvent, BattleEventKind, BattleSeed, Command, EffectDefinitionId,
    LinkedEntityKind, ShieldEventData, TurnEventData, UnitId,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1009, 1001, 1103, 1005];

fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_transfers()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    fixture
        .factory()
        .weighted_curio_runtime()
        .unwrap()
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &[selected],
        )
        .unwrap();
    let before = activity.canonical_state_bytes();
    let assembled = assemble(fixture, &flow, &activity);
    assert_eq!(activity.canonical_state_bytes(), before);
    assembled
}
fn capacity(battle: &Battle, recipient: UnitId, effect: u32) -> i64 {
    battle
        .view()
        .shields_by_id()
        .filter(|s| {
            s.owner() == recipient && s.source_effect() == Some(id::<EffectDefinitionId>(effect))
        })
        .map(|s| s.remaining().get())
        .sum()
}
fn changes(events: &[BattleEvent], recipient: UnitId) -> Vec<(i64, i64, i64)> {
    events
        .iter()
        .filter_map(|e| match e.kind() {
            BattleEventKind::Shield(ShieldEventData::Adjusted {
                target,
                effect,
                requested,
                before,
                after,
                ..
            }) if *target == recipient && effect.get() == SPECIAL => {
                Some((requested.get(), before.get(), after.get()))
            }
            _ => None,
        })
        .collect()
}
fn hp(battle: &Battle, recipient: UnitId) -> i64 {
    battle
        .view()
        .units_by_id()
        .find(|u| u.id() == recipient)
        .unwrap()
        .current_hp()
        .get()
}
fn next_turn(battle: &mut Battle, recipient: UnitId) -> Vec<BattleEvent> {
    let mut events = Vec::new();
    for _ in 0..256 {
        let applied = idle(battle);
        let started = applied.iter().any(|e| matches!(e.kind(), BattleEventKind::Turn(TurnEventData::Started {owner,..}) if *owner == recipient));
        events.extend(applied);
        if started {
            return events;
        }
    }
    panic!("recipient turn not reached");
}

#[test]
fn weighted_curio_transfer_binds_all_original_recipients_only_with_preservation_party() {
    for (party, admitted) in [(PARTY, true), ([1009, 1004, 1103, 1005], false)] {
        let fixture = DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap();
        for family in FAMILIES {
            let assembled = equipped(&fixture, family);
            assert_eq!(
                assembled.combat_catalog().effect(id(SPECIAL)).is_some(),
                admitted
            );
            let mut battle = scenario(&assembled, Probe::default());
            idle(&mut battle);
            let markers = battle
                .view()
                .effects_by_id()
                .filter(|e| e.definition().get() == SPECIAL)
                .count();
            assert_eq!(markers, if admitted { 4 } else { 0 });
            assert_eq!(battle.view().shields_by_id().count(), 0);
            let recipient = unit(&battle, 2);
            let events = cast(&mut battle, 0, GRANT, recipient);
            assert_eq!(
                capacity(&battle, recipient, SPECIAL),
                if admitted { 300 } else { 0 }
            );
            assert_eq!(
                changes(&events, recipient),
                if admitted {
                    vec![(300, 0, 300)]
                } else {
                    vec![]
                }
            );
            assert_eq!(hp(&battle, recipient), 1);
        }
    }
}

#[test]
fn weighted_curio_transfer_preserves_threshold_precision_and_heals_only_actual_decay() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        // The 101-HP case distinguishes 30.3 from prematurely floored 30.
        for (maximum, grant, repetitions, after, loss, healing) in [
            (1000, 0, 1, 0, 0, 0),
            (1000, 399, 1, 299, 0, 0),
            (1000, 400, 1, 300, 0, 0),
            (1000, 800, 1, 330, 270, 27),
            (1000, 400, 2, 330, 270, 27),
            (101, 55, 1, 32, 9, 0),
            (203, 200, 1, 70, 80, 8),
        ] {
            let mut battle = scenario(
                &assembled,
                Probe {
                    hp: [maximum; 4],
                    grant,
                    repetitions,
                    ..Probe::default()
                },
            );
            let recipient = unit(&battle, 0);
            let events = cast(&mut battle, 0, GRANT, recipient);
            let gained = grant * 3 / 4;
            assert_eq!(
                capacity(&battle, recipient, SPECIAL),
                gained * i64::from(repetitions)
            );
            assert_eq!(
                changes(&events, recipient).len(),
                if grant == 0 {
                    0
                } else {
                    usize::from(repetitions)
                }
            );
            assert_eq!(hp(&battle, recipient), 1);
            let events = next_turn(&mut battle, recipient);
            assert_eq!(
                capacity(&battle, recipient, SPECIAL),
                after,
                "{maximum}/{grant}/{repetitions}"
            );
            assert_eq!(hp(&battle, recipient), 1 + healing);
            assert_eq!(
                changes(&events, recipient),
                if loss == 0 {
                    vec![]
                } else {
                    vec![(loss, gained * i64::from(repetitions), after)]
                }
            );
        }
    }
}

#[test]
fn weighted_curio_transfer_uses_recipient_turn_and_live_maximum_hp_not_other_turns() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut battle = scenario(
            &assembled,
            Probe {
                grant: 800,
                ..Probe::default()
            },
        );
        let recipient = unit(&battle, 3);
        cast(&mut battle, 0, GRANT, recipient);
        assert_eq!(capacity(&battle, recipient, SPECIAL), 600);
        // Formation 1 acts before formation 3: mutate the real resource capacity.
        let events = cast(&mut battle, 1, SHRINK_HP, recipient);
        assert!(changes(&events, recipient).is_empty());
        assert_eq!(hp(&battle, recipient), 1);
        let events = next_turn(&mut battle, recipient);
        assert_eq!(changes(&events, recipient), vec![(297, 600, 303)]);
        assert_eq!(hp(&battle, recipient), 30);
        assert_eq!(capacity(&battle, recipient, ORDINARY), 800);
    }
}

#[test]
fn weighted_curio_transfer_absorption_and_explicit_removal_do_not_heal_or_recurse() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut battle = scenario(
            &assembled,
            Probe {
                grant: 400,
                ..Probe::default()
            },
        );
        let recipient = unit(&battle, 3);
        cast(&mut battle, 0, GRANT, recipient);
        let events = cast(&mut battle, 1, ABSORB, recipient);
        assert!(changes(&events, recipient).is_empty());
        assert_eq!(hp(&battle, recipient), 1);
        assert_eq!(capacity(&battle, recipient, SPECIAL), 200);
        assert_eq!(capacity(&battle, recipient, ORDINARY), 300);
        let events = cast(&mut battle, 2, REMOVE, recipient);
        assert!(changes(&events, recipient).is_empty());
        assert_eq!(capacity(&battle, recipient, SPECIAL), 0);
        assert_eq!(capacity(&battle, recipient, ORDINARY), 300);
        assert_eq!(hp(&battle, recipient), 1);
        assert!(
            !battle
                .view()
                .effects_by_id()
                .any(|e| e.definition().get() == SPECIAL && e.target() == recipient)
        );
        // A later ordinary grant reinstalls the zero marker and creates new capacity.
        cast(&mut battle, 3, GRANT, recipient);
        assert_eq!(capacity(&battle, recipient, SPECIAL), 300);
    }
}

#[test]
fn weighted_curio_transfer_fresh_commands_are_deterministic_and_stale_commands_inert() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let mut left = scenario(
            &equipped(&fixture, family),
            Probe {
                grant: 800,
                ..Probe::default()
            },
        );
        let mut right = scenario(
            &equipped(&fresh, family),
            Probe {
                grant: 800,
                ..Probe::default()
            },
        );
        let stale = Command::StartBattle {
            decision: left.decision().unwrap().id(),
        };
        let recipient = unit(&left, 0);
        assert_eq!(recipient, unit(&right, 0));
        let encode = |events: Vec<BattleEvent>| {
            events
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(
            encode(cast(&mut left, 0, GRANT, recipient)),
            encode(cast(&mut right, 0, GRANT, recipient))
        );
        assert_eq!(
            next_turn(&mut left, recipient),
            next_turn(&mut right, recipient)
        );
        assert_eq!(left.view().rng_draw_count(), right.view().rng_draw_count());
        assert_eq!(left.state_hash(), right.state_hash());
        let before = (left.state_hash(), left.view().rng_draw_count());
        assert!(left.apply(stale).is_err());
        assert_eq!((left.state_hash(), left.view().rng_draw_count()), before);
        let command = Command::Concede {
            decision: left.decision().unwrap().id(),
        };
        assert_eq!(
            accept(&mut left, command.clone()),
            accept(&mut right, command)
        );
        assert_eq!(left.state_hash(), right.state_hash());
    }
}

#[test]
fn weighted_curio_transfer_provider_defeat_absence_and_inherited_linked_bundles_do_not_change_scope()
 {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for input in [
            Probe {
                defeated: Some(1),
                ..Probe::default()
            },
            Probe {
                absent: Some(1),
                ..Probe::default()
            },
        ] {
            let mut battle = scenario(&assembled, input);
            idle(&mut battle);
            assert_eq!(
                battle
                    .view()
                    .effects_by_id()
                    .filter(|e| e.definition().get() == SPECIAL)
                    .count(),
                3
            );
            let recipient = unit(&battle, 2);
            cast(&mut battle, 0, GRANT, recipient);
            assert_eq!(capacity(&battle, recipient, SPECIAL), 300);
        }
        for kind in [
            LinkedEntityKind::Summon,
            LinkedEntityKind::Memosprite,
            LinkedEntityKind::SharedActor,
        ] {
            let mut battle = scenario(
                &assembled,
                Probe {
                    linked_kind: Some(kind),
                    ..Probe::default()
                },
            );
            let recipient = unit(&battle, 0);
            cast(&mut battle, 0, SUMMON, recipient);
            let linked = unit(&battle, 4);
            assert!(
                !battle
                    .view()
                    .effects_by_id()
                    .any(|e| e.definition().get() == SPECIAL && e.target() == linked)
            );
            cast(&mut battle, 1, GRANT, linked);
            assert_eq!(capacity(&battle, linked, ORDINARY), 400);
            assert_eq!(capacity(&battle, linked, SPECIAL), 0);
        }
    }
}

#[test]
fn weighted_curio_transfer_unmodified_production_battle_starts_with_four_zero_capacity_markers() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut battle = Battle::create(
            Arc::clone(assembled.combat_catalog()),
            assembled.battle_spec().clone(),
            BattleSeed::new([0x9a; 32]),
        )
        .unwrap();
        let command = Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        };
        accept(&mut battle, command);
        assert_eq!(
            battle
                .view()
                .effects_by_id()
                .filter(|e| e.definition().get() == SPECIAL)
                .count(),
            4
        );
        assert!(
            !battle
                .view()
                .shields_by_id()
                .any(|s| s.source_effect() == Some(id(SPECIAL)))
        );
        let command = battle
            .decision()
            .unwrap()
            .legal_commands()
            .iter()
            .find(|c| matches!(c, Command::UseAbility { .. }))
            .unwrap()
            .clone();
        accept(&mut battle, command);
    }
}
