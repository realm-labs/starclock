//! All three operands use real Sora-backed resources and accepted commands.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
        weighted_curio_attack_debuff::{Probe, action, scenario, step},
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_activity::ProjectedValue;
use starclock_combat::{BattleEvent, BattleEventKind, Command, ShieldEventData, TeamSide, UnitId};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1009, 1001, 1103, 1005];
const EFFECT: u32 = 0x7e85_0001;

fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_prayers()[0]
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
    assemble(fixture, &flow, &activity)
}
fn shields(events: &[BattleEvent], actor: UnitId) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Shield(ShieldEventData::Applied { target, amount, .. })
                if *target == actor =>
            {
                Some(amount.get())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_prayer_changes_real_capacity_only_for_both_released_paths() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, activity) = ready(&fixture, family);
        let before = activity.canonical_state_bytes();
        let plain = assemble(&fixture, &flow, &activity);
        let added = equipped(&fixture, family);
        for (old, new) in plain
            .battle_spec()
            .participants()
            .iter()
            .zip(added.battle_spec().participants())
        {
            let applies = old.side() == TeamSide::Player && old.formation().get() >= 2;
            let base = old.combatant().maximum_hp().get();
            assert_eq!(
                new.combatant().maximum_hp().get(),
                base + if applies { base * 3 / 5 } else { 0 }
            );
            assert_eq!(new.locked_combatant_digest(), old.locked_combatant_digest());
            assert_eq!(new.combatant().base_attack(), old.combatant().base_attack());
            assert_eq!(
                new.combatant().base_defense(),
                old.combatant().base_defense()
            );
            assert_eq!(new.combatant().modifiers(), old.combatant().modifiers());
            assert_eq!(
                new.combatant().modifier_bindings(),
                old.combatant().modifier_bindings()
            );
            if !applies {
                assert_eq!(new.combatant(), old.combatant());
            }
        }
        for formation in 0..4 {
            let mut test = scenario(
                &added,
                Probe {
                    formation,
                    use_resolved_hp: true,
                    enemy_hp: 10_000_000,
                    ..Probe::default()
                },
            );
            let unit = test
                .battle
                .view()
                .units_by_id()
                .find(|unit| unit.id() == test.actor)
                .unwrap();
            let maximum = unit.maximum_hp().get();
            assert_eq!(unit.current_hp().get(), maximum);
            let selected = test.selected;
            let events = step(&mut test, selected);
            assert_eq!(
                shields(&events, test.actor),
                if formation >= 2 {
                    vec![maximum / 4]
                } else {
                    vec![]
                }
            );
            let unit = test
                .battle
                .view()
                .units_by_id()
                .find(|unit| unit.id() == test.actor)
                .unwrap();
            assert_eq!(
                unit.current_hp().get(),
                maximum
                    - if formation >= 2 {
                        maximum * 15 / 100
                    } else {
                        0
                    }
            );
            assert_eq!(unit.maximum_hp().get(), maximum);
        }
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn weighted_curio_prayer_consumes_current_hp_before_maximum_shield_and_preserves_one_hp() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for formation in [2, 3] {
            for hp in [1, 7, 107, 999] {
                let mut test = scenario(
                    &assembled,
                    Probe {
                        formation,
                        initial_hp: Some(hp),
                        use_resolved_hp: true,
                        enemy_hp: 10_000_000,
                        ..Probe::default()
                    },
                );
                let selected = test.selected;
                let events = step(&mut test, selected);
                let capacity = test
                    .battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == test.actor)
                    .unwrap()
                    .maximum_hp()
                    .get();
                let hp_after = test
                    .battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == test.actor)
                    .unwrap()
                    .current_hp()
                    .get();
                assert_eq!(hp_after, (hp - hp * 15 / 100).max(1));
                assert_eq!(shields(&events, test.actor), vec![capacity / 4]);
                let shield_position = events
                    .iter()
                    .position(|event| {
                        matches!(
                            event.kind(),
                            BattleEventKind::Shield(ShieldEventData::Applied { .. })
                        )
                    })
                    .unwrap();
                if let Some(drain_position) = events
                    .iter()
                    .position(|event| matches!(event.kind(), BattleEventKind::HpConsumption(_)))
                {
                    assert!(drain_position < shield_position);
                }
            }
        }
    }
}

#[test]
fn weighted_curio_prayer_refreshes_one_battle_lifetime_shield_once_per_owner_turn_not_action() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut test = scenario(
            &assembled,
            Probe {
                formation: 3,
                use_resolved_hp: true,
                enemy_hp: 10_000_000,
                ..Probe::default()
            },
        );
        let selected = test.selected;
        let entry = step(&mut test, selected);
        assert_eq!(shields(&entry, test.actor).len(), 1);
        let ultimate = test.ultimate;
        assert!(shields(&action(&mut test, ultimate), test.actor).is_empty());
        let selected = test.selected;
        let mut events = action(&mut test, selected);
        for _ in 0..32 {
            if !shields(&events, test.actor).is_empty() {
                break;
            }
            events.extend(step(&mut test, selected));
        }
        assert_eq!(shields(&events, test.actor).len(), 1);
        assert_eq!(
            test.battle
                .view()
                .shields_by_id()
                .filter(|shield| shield.owner() == test.actor)
                .count(),
            1
        );
        let effects = test
            .battle
            .view()
            .effects_by_id()
            .filter(|effect| effect.definition().get() == EFFECT)
            .collect::<Vec<_>>();
        assert_eq!(effects.len(), 1);
        assert!(effects[0].remaining().is_none());
    }
}

#[test]
fn weighted_curio_prayer_preserves_fresh_battles_rejected_commands_and_unequip() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    let fresh = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_prayers()[0]
            .weighted_curio
            .clone();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[selected],
            )
            .unwrap();
        let bytes = activity.canonical_state_bytes();
        let assembled = assemble(&fixture, &flow, &activity);
        let rebuilt = equipped(&fresh, family);
        assert_eq!(assembled.assembly_digest(), rebuilt.assembly_digest());
        let input = Probe {
            formation: 3,
            use_resolved_hp: true,
            enemy_hp: 10_000_000,
            ..Probe::default()
        };
        let mut first = scenario(&assembled, input);
        let initial_hash = first.battle.state_hash();
        assert!(
            first
                .battle
                .apply(Command::Concede {
                    decision: first.battle.decision().unwrap().id()
                })
                .is_err()
        );
        assert_eq!(first.battle.state_hash(), initial_hash);
        let mut second = scenario(&rebuilt, input);
        let ability = first.selected;
        assert_eq!(action(&mut first, ability), action(&mut second, ability));
        assert_eq!(first.battle.state_hash(), second.battle.state_hash());
        assert_eq!(activity.canonical_state_bytes(), bytes);
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                &[],
            )
            .unwrap();
        let removed = assemble(&fixture, &flow, &activity);
        let mut test = scenario(&removed, input);
        let ability = test.selected;
        assert!(shields(&action(&mut test, ability), test.actor).is_empty());
    }
}

#[test]
fn weighted_curio_prayer_real_battle_handoff_exposes_the_new_maxima_in_verified_result() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_prayers()[0]
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
        let prepared = assemble(&fixture, &flow, &activity);
        let actual = result(&fixture, &flow, &mut activity);
        let participants = actual
            .values()
            .iter()
            .filter_map(|value| match value {
                ProjectedValue::ParticipantState(state) => Some(state),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (participant, spec) in participants.iter().zip(
            prepared
                .battle_spec()
                .participants()
                .iter()
                .filter(|p| p.side() == TeamSide::Player),
        ) {
            assert_eq!(participant.maximum_hp(), spec.combatant().maximum_hp());
            assert!(participant.current_hp() <= participant.maximum_hp());
        }
        assert_eq!(participants.len(), 4);
    }
}

#[test]
fn weighted_curio_prayer_uses_live_resource_capacity_not_hp_stat_or_entry_capacity() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let capacity = assembled
            .battle_spec()
            .participants()
            .iter()
            .find(|p| p.side() == TeamSide::Player && p.formation().get() == 3)
            .unwrap()
            .combatant()
            .maximum_hp()
            .get();
        let reduction = capacity / 2;
        let mut test = scenario(
            &assembled,
            Probe {
                formation: 3,
                use_resolved_hp: true,
                reduce_hp: Some(reduction),
                extra_hp_stat: Some(10_000),
                enemy_hp: 10_000_000,
                ..Probe::default()
            },
        );
        let ability = test.selected;
        let entry = step(&mut test, ability);
        assert_eq!(shields(&entry, test.actor), vec![capacity / 4]);
        let mut events = action(&mut test, ability);
        let current = test
            .battle
            .view()
            .units_by_id()
            .find(|u| u.id() == test.actor)
            .unwrap();
        assert_eq!(current.maximum_hp().get(), capacity - reduction);
        assert_eq!(current.initial_maximum_hp().get(), capacity);
        for _ in 0..32 {
            if !shields(&events, test.actor).is_empty() {
                break;
            }
            events.extend(step(&mut test, ability));
        }
        assert_eq!(
            shields(&events, test.actor),
            vec![(capacity - reduction) / 4]
        );
    }
}
