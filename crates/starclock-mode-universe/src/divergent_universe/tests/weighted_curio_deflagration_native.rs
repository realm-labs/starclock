//! Real command consumers of the constructor; no Activity equipment bypass.
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_burn_fixture::{
            ATTACK, BURN, DETONATE, Probe, RESTORE, Scenario, attack, cast, command, idle_step,
            scenario_with_players, until_source_tick,
        },
    },
    weighted_curio::deflagration::{
        DeflagrationBaseDamagePolicy, native::bind_mapped_deflagration_policy,
    },
};
use starclock_combat::{
    BattleEvent, BattleEventKind, BreakDamageKind, Command, DamageKind, EffectEventData,
    ParticipantSpec, SourceDefinitionId, TeamSide,
    catalog::{CombatCatalog, builder::CombatCatalogBuilder},
    rule::model::RuleValue,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::weighted_curio_deflagrations::WeightedCurioDeflagrationDefinition,
};
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

pub(super) const EFFECT: u32 = 0x7f75_0001;
pub(super) const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const FIRE: [u32; 4] = [1009, 1003, 1112, 1301];
const OTHER: [u32; 4] = [1008, 1105, 1211, 1002];
pub(super) fn fixture(count: usize) -> DivergentUniverseBaselineFixture {
    let party = std::array::from_fn(|index| {
        if index < count {
            FIRE[index]
        } else {
            OTHER[index]
        }
    });
    DivergentUniverseBaselineFixture::production_for_source_party(party).unwrap()
}
pub(super) fn parts(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> (Arc<CombatCatalog>, Vec<ParticipantSpec>) {
    let (flow, activity) = ready(fixture, family);
    let before = activity.canonical_state_bytes();
    let assembled = assemble(fixture, &flow, &activity);
    let snapshot = fixture
        .factory()
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&flow, &activity)
        .unwrap();
    let mut builder =
        CombatCatalogBuilder::from_catalog(assembled.combat_catalog(), assembled.assembly_digest());
    let original = assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|p| p.side() == TeamSide::Player)
        .cloned()
        .collect::<Vec<_>>();
    let players = bind_mapped_deflagration_policy(
        &mut builder,
        &fixture
            .factory()
            .decision_catalog()
            .weighted_curio_deflagrations()[0],
        fixture.core(),
        &original,
        snapshot.difficulty_protocol(),
        assembled.assembly_digest(),
    )
    .unwrap();
    assert_eq!(activity.canonical_state_bytes(), before);
    (builder.build().unwrap(), players)
}
pub(super) fn scenario(
    parts: &(Arc<CombatCatalog>, Vec<ParticipantSpec>),
    input: Probe,
) -> Scenario {
    scenario_with_players(
        &parts.0,
        &parts.1,
        Probe {
            full_party: true,
            second_owner: true,
            ..input
        },
    )
}
pub(super) fn source(formation: u32) -> SourceDefinitionId {
    SourceDefinitionId::new(0x7f74_0000 + (formation + 1) * 32).unwrap()
}
pub(super) fn applications(events: &[BattleEvent]) -> usize {
    events.iter().filter(|event| matches!(event.kind(), BattleEventKind::Effect(EffectEventData::Applied { definition, .. }) if definition.get() == EFFECT)).count()
}
fn values(events: &[BattleEvent], kind: DamageKind) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.kind == kind => Some(data.raw.scaled()),
            _ => None,
        })
        .collect()
}
pub(super) fn payloads(events: &[BattleEvent]) -> Vec<Vec<u8>> {
    events
        .iter()
        .map(|event| encode_battle_event_payload(event).unwrap())
        .collect()
}

#[test]
fn deflagration_native_zero_through_four_original_fire_members_and_complete_actions() {
    for count in 0..=4 {
        let fixture = fixture(count);
        for family in FAMILIES {
            let parts = parts(&fixture, family);
            for (all, hits, tag, expected) in [
                (false, 1, true, 1),
                (false, 5, true, 1),
                (true, 3, true, 3),
                (true, 3, false, 0),
            ] {
                let mut test = scenario(
                    &parts,
                    Probe {
                        all,
                        hits,
                        attack: tag,
                        ..Probe::default()
                    },
                );
                let events = attack(&mut test);
                assert_eq!(
                    applications(&events),
                    if count == 0 { 0 } else { expected },
                    "{events:#?}"
                );
                assert!(values(&events, DamageKind::DotTick).is_empty());
                assert!(values(&events, DamageKind::DotDetonation).is_empty());
                assert_eq!(test.battle.view().rng_draw_count(), 0);
                for effect in test
                    .battle
                    .view()
                    .effects_by_id()
                    .filter(|effect| effect.definition().get() == EFFECT)
                {
                    assert_eq!(
                        (
                            effect.remaining(),
                            effect.stacks(),
                            effect.magnitude().scaled()
                        ),
                        (Some(2), 1, 29_602_204_000)
                    );
                    assert_eq!(effect.applier(), test.actor);
                }
                if count > 0 {
                    let counts = test
                        .started
                        .iter()
                        .filter_map(|event| match event.kind() {
                            BattleEventKind::RuleState(data) => Some(data.after.clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    assert!(counts.contains(&RuleValue::Integer(i64::try_from(count).unwrap())));
                }
            }
        }
    }
}

#[test]
fn deflagration_native_each_entry_count_detonates_only_other_burns_without_recursion() {
    for count in 1..=4 {
        let fixture = fixture(count);
        for family in FAMILIES {
            let parts = parts(&fixture, family);
            let mut test = scenario(
                &parts,
                Probe {
                    burns: true,
                    break_burn: true,
                    ..Probe::default()
                },
            );
            let burn = test
                .battle
                .view()
                .effects_by_id()
                .find(|effect| {
                    effect.definition().get() == BURN && effect.target() == test.targets[0]
                })
                .unwrap();
            let before = (
                burn.id(),
                burn.remaining(),
                burn.stacks(),
                burn.source_operation(),
            );
            attack(&mut test);
            let events = until_source_tick(&mut test, source(0));
            assert_eq!(values(&events, DamageKind::DotTick), [29_602_204_000]);
            assert_eq!(
                values(&events, DamageKind::DotDetonation),
                [i64::try_from(count).unwrap() * 50_000_000]
            );
            let after = test
                .battle
                .view()
                .effects_by_id()
                .find(|effect| effect.id() == before.0)
                .unwrap();
            assert_eq!(
                before,
                (
                    after.id(),
                    after.remaining(),
                    after.stacks(),
                    after.source_operation()
                )
            );
            for event in &events {
                if let BattleEventKind::Damage(data) = event.kind()
                    && data.kind == DamageKind::DotDetonation
                {
                    assert_eq!(event.cause().applier(), Some(test.seeder));
                    assert_eq!(data.source_effect, Some(before.0));
                }
            }
            let break_values = events
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::BreakDamage(data)
                        if data.kind == BreakDamageKind::EffectDetonation =>
                    {
                        Some(data.raw.scaled())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(break_values.len(), 1);
            let natural = events
                .iter()
                .find_map(|event| match event.kind() {
                    BattleEventKind::BreakDamage(data) if data.kind == BreakDamageKind::Effect => {
                        Some(data.raw.scaled())
                    }
                    _ => None,
                })
                .unwrap();
            assert_eq!(
                break_values[0],
                i64::try_from(i128::from(natural) * i128::try_from(count).unwrap() / 2).unwrap()
            );
        }
    }
}

#[test]
fn deflagration_native_external_detonation_never_cascades_into_other_burns() {
    let fixture = fixture(4);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        let mut test = scenario(
            &parts,
            Probe {
                burns: true,
                ..Probe::default()
            },
        );
        attack(&mut test);
        let seeder = test.seeder;
        let events = cast(&mut test, seeder, DETONATE);
        assert_eq!(values(&events, DamageKind::DotDetonation).len(), 4);
        let burn = test
            .battle
            .view()
            .effects_by_id()
            .find(|effect| effect.definition().get() == BURN && effect.target() == test.targets[0])
            .unwrap()
            .id();
        assert_eq!(events.iter().filter(|event| matches!(event.kind(), BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation && data.source_effect == Some(burn))).count(), 1);
        assert_eq!(test.battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn deflagration_native_cross_caster_replacement_keeps_target_base_not_owner_attack() {
    let fixture = fixture(4);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        let mut test = scenario(&parts, Probe::default());
        attack(&mut test);
        let seeder = test.seeder;
        cast(&mut test, seeder, ATTACK);
        let effects = test
            .battle
            .view()
            .effects_by_id()
            .filter(|effect| effect.definition().get() == EFFECT)
            .collect::<Vec<_>>();
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].applier(), seeder);
        assert_eq!(effects[0].magnitude().scaled(), 29_602_204_000);
        let events = until_source_tick(&mut test, source(1));
        assert_eq!(values(&events, DamageKind::DotTick), [29_602_204_000]);
    }
}

#[test]
fn deflagration_native_ticks_twice_expires_and_preserves_fresh_rejections_and_terminal_cleanup() {
    let fixture = fixture(4);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        let mut friendly = scenario(
            &parts,
            Probe {
                allied: true,
                all: true,
                ..Probe::default()
            },
        );
        assert_eq!(applications(&attack(&mut friendly)), 0);
        let mut lethal = scenario(
            &parts,
            Probe {
                enemy_hp: 1,
                all: true,
                hits: 1,
                ..Probe::default()
            },
        );
        assert_eq!(applications(&attack(&mut lethal)), 0);
        let run = || {
            let mut test = scenario(&parts, Probe::default());
            let hash = test.battle.state_hash();
            let invalid = Command::StartBattle {
                decision: test.battle.decision().unwrap().id(),
            };
            assert!(test.battle.apply(invalid).is_err());
            assert_eq!(hash, test.battle.state_hash());
            let mut events = attack(&mut test);
            let mut ticks = 0;
            for _ in 0..192 {
                let next = idle_step(&mut test);
                ticks += next.iter().filter(|event| matches!(event.kind(), BattleEventKind::Damage(data) if data.kind == DamageKind::DotTick && event.cause().source_definition() == Some(source(0)))).count();
                events.extend(next);
                if test
                    .battle
                    .view()
                    .effects_by_id()
                    .all(|effect| effect.definition().get() != EFFECT)
                {
                    break;
                }
            }
            assert_eq!(ticks, 2);
            assert!(
                test.battle
                    .view()
                    .effects_by_id()
                    .all(|effect| effect.definition().get() != EFFECT)
            );
            assert_eq!(test.battle.view().rng_draw_count(), 0);
            (payloads(&events), events, test.battle.state_hash())
        };
        assert_eq!(run(), run());
        let mut test = scenario(&parts, Probe::default());
        attack(&mut test);
        for _ in 0..8 {
            if test.battle.decision().is_some() {
                break;
            }
            idle_step(&mut test);
        }
        let resolution = test
            .battle
            .apply(Command::Concede {
                decision: test.battle.decision().unwrap().id(),
            })
            .unwrap();
        assert!(resolution.fault().is_none());
        assert_eq!(resolution.events().iter().filter(|event| matches!(event.kind(), BattleEventKind::RuleState(data) if data.after == RuleValue::Integer(0))).count(), 4);
        assert!(
            test.battle
                .view()
                .effects_by_id()
                .all(|effect| effect.definition().get() != EFFECT)
        );
    }
}

#[test]
fn deflagration_native_captures_selected_enemy_level_instead_of_owner_level_or_attack() {
    let fixture = fixture(4);
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0];
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        for level in [1, 40, 80, 95] {
            let mut test = scenario(
                &parts,
                Probe {
                    enemy_level: level,
                    ..Probe::default()
                },
            );
            attack(&mut test);
            let expected = definition.hp_ratios_millionths[usize::from(level - 1)] * 200;
            let effect = test
                .battle
                .view()
                .effects_by_id()
                .find(|effect| effect.definition().get() == EFFECT)
                .unwrap();
            assert_eq!(effect.magnitude().scaled(), expected);
            let events = until_source_tick(&mut test, source(0));
            assert_eq!(values(&events, DamageKind::DotTick), [expected]);
            assert_eq!(test.battle.view().rng_draw_count(), 0);
        }
    }
}

#[test]
fn deflagration_native_rejects_invalid_original_rosters_operands_and_twice_base_overflow() {
    let fixture = fixture(4);
    let (flow, activity) = ready(&fixture, FAMILIES[0]);
    let assembled = assemble(&fixture, &flow, &activity);
    let snapshot = fixture
        .factory()
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&flow, &activity)
        .unwrap();
    let original = assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|player| player.side() == TeamSide::Player)
        .cloned()
        .collect::<Vec<_>>();
    let definition = &fixture
        .factory()
        .decision_catalog()
        .weighted_curio_deflagrations()[0];
    let check = |definition: &WeightedCurioDeflagrationDefinition, players: &[ParticipantSpec]| {
        let mut builder = CombatCatalogBuilder::from_catalog(
            assembled.combat_catalog(),
            assembled.assembly_digest(),
        );
        bind_mapped_deflagration_policy(
            &mut builder,
            definition,
            fixture.core(),
            players,
            snapshot.difficulty_protocol(),
            assembled.assembly_digest(),
        )
    };
    assert!(check(definition, &[]).is_err());
    let mut duplicate = original.clone();
    duplicate[1] = original[0].clone();
    assert!(check(definition, &duplicate).is_err());
    let mut excess = original.clone();
    excess.push(original[0].clone());
    assert!(check(definition, &excess).is_err());
    for change in 0..6 {
        let mut altered = definition.clone();
        match change {
            0 => altered.burn_fractions_millionths[0] = 0,
            1 => altered.damage_multiplier_millionths = 0,
            2 => altered.duration_turns = 3,
            3 => altered.runtime_policy_note = "observed parity".into(),
            4 => altered.runtime_replacement_condition = " ".into(),
            5 => altered.hp_ratios_millionths[94] = i64::MAX / 100,
            _ => unreachable!(),
        }
        if change == 5 {
            assert!(
                DeflagrationBaseDamagePolicy::from_authored(
                    &altered,
                    snapshot.difficulty_protocol()
                )
                .is_ok()
            );
        }
        assert!(check(&altered, &original).is_err());
    }
}

#[test]
fn deflagration_native_entry_count_is_not_recaptured_after_presence_restore() {
    let fixture = fixture(4);
    for family in FAMILIES {
        let parts = parts(&fixture, family);
        let mut test = scenario(
            &parts,
            Probe {
                burns: true,
                absent_formation: Some(3),
                ..Probe::default()
            },
        );
        assert!(test.started.iter().any(|event| matches!(event.kind(), BattleEventKind::RuleState(data) if data.after == RuleValue::Integer(3))));
        let seeder = test.seeder;
        let restore = command(&mut test, seeder, RESTORE.try_into().unwrap(), None);
        let restored = test.battle.apply(restore).unwrap();
        assert!(restored.fault().is_none());
        attack(&mut test);
        let events = until_source_tick(&mut test, source(0));
        assert_eq!(values(&events, DamageKind::DotDetonation), [150_000_000]);
    }
}
