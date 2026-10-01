//! Whole Mock Crimson Moon from the production workbook through accepted commands.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::{ready, result},
        curio_battle_stats::assemble,
        weighted_curio_necrosis_fixture::{
            ATTACK, BURN, DETONATE, NECROSIS, Probe, attack, cast, idle_step, scenario, until_tick,
        },
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    BattleEvent, BattleEventKind, BreakDamageKind, Command, DamageKind, EffectEventData,
    ToughnessEventData, catalog::action::AbilityKind, formula::model::DamageClass,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1105, 1211, 1009, 1008];
fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let selected = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_necroses()[0]
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
    let hash = activity.state_hash();
    let assembled = assemble(fixture, &flow, &activity);
    assert_eq!(hash, activity.state_hash());
    assembled
}
fn applications(events: &[BattleEvent]) -> usize {
    events
        .iter()
        .filter(|event| match event.kind() {
            BattleEventKind::Effect(EffectEventData::Applied { definition, .. }) => {
                definition.get() == NECROSIS
            }
            _ => false,
        })
        .count()
}
fn necrosis_damage(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if event
                    .cause()
                    .source_definition()
                    .is_some_and(|id| id.get() == 0x7eb3_0001) =>
            {
                assert_eq!(data.class, DamageClass::Dot);
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_necrosis_executes_abundance_multihit_and_multitarget_once_per_action() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for (formation, all, enabled, expected) in [
            (0, false, true, 1),
            (1, true, true, 3),
            (2, true, true, 0),
            (0, true, false, 0),
        ] {
            let mut test = scenario(
                &assembled,
                Probe {
                    formation,
                    all,
                    attack: enabled,
                    ..Probe::default()
                },
            );
            let events = attack(&mut test);
            assert_eq!(applications(&events), expected);
            assert!(necrosis_damage(&events).is_empty());
            assert_eq!(test.battle.view().rng_draw_count(), 0);
            for effect in test
                .battle
                .view()
                .effects_by_id()
                .filter(|e| e.definition().get() == NECROSIS)
            {
                assert_eq!(
                    (
                        effect.remaining(),
                        effect.stacks(),
                        effect.magnitude().scaled()
                    ),
                    (Some(3), 1, 1_200_000_000)
                );
                assert_eq!(effect.applier(), test.actor);
            }
        }
        let mut allied = scenario(
            &assembled,
            Probe {
                allied: true,
                ..Probe::default()
            },
        );
        assert_eq!(applications(&attack(&mut allied)), 0);
        let mut skill = scenario(
            &assembled,
            Probe {
                kind: AbilityKind::Skill,
                all: true,
                ..Probe::default()
            },
        );
        assert_eq!(applications(&attack(&mut skill)), 3);
        let mut lethal = scenario(
            &assembled,
            Probe {
                all: true,
                enemy_hp: 1,
                ..Probe::default()
            },
        );
        assert_eq!(applications(&attack(&mut lethal)), 0);
    }
}

#[test]
fn weighted_curio_necrosis_chance_uses_current_hit_rate_resistance_and_labeled_draws() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        for (resistance, hit_rate, expected) in [(1_000_000, 0, 0), (500_000, 1_000_000, 1)] {
            let mut test = scenario(
                &assembled,
                Probe {
                    resistance,
                    hit_rate,
                    ..Probe::default()
                },
            );
            assert_eq!(applications(&attack(&mut test)), expected);
            assert_eq!(test.battle.view().rng_draw_count(), 0);
        }
        let mut outcomes = [false; 2];
        for seed in 1..=16 {
            let run = || {
                let mut test = scenario(
                    &assembled,
                    Probe {
                        resistance: 500_000,
                        seed,
                        ..Probe::default()
                    },
                );
                let events = attack(&mut test);
                let applied = applications(&events);
                assert!(applied <= 1);
                assert_eq!(test.battle.view().rng_draw_count(), 1);
                (applied, events, test.battle.state_hash())
            };
            let first = run();
            assert_eq!(first, run());
            outcomes[first.0] = true;
        }
        assert_eq!(outcomes, [true, true]);
    }
}

#[test]
fn weighted_curio_necrosis_ticks_three_turns_then_expires_without_reapplication() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut test = scenario(&assembled, Probe::default());
        attack(&mut test);
        let mut amounts = vec![];
        for remaining in [3, 2, 1] {
            let events = until_tick(&mut test);
            amounts.extend(necrosis_damage(&events));
            assert_eq!(
                test.battle
                    .view()
                    .effects_by_id()
                    .find(|e| e.definition().get() == NECROSIS)
                    .unwrap()
                    .remaining(),
                Some(remaining)
            );
            for _ in 0..32 {
                idle_step(&mut test);
                if test
                    .battle
                    .view()
                    .effects_by_id()
                    .filter(|e| e.definition().get() == NECROSIS)
                    .all(|e| e.remaining() != Some(remaining))
                {
                    break;
                }
            }
        }
        assert_eq!(amounts, vec![1_200_000_000; 3]);
        assert!(
            test.battle
                .view()
                .effects_by_id()
                .all(|e| e.definition().get() != NECROSIS)
        );
        let mut events = vec![];
        for _ in 0..24 {
            events.extend(idle_step(&mut test));
        }
        assert!(necrosis_damage(&events).is_empty());
    }
}

#[test]
fn weighted_curio_necrosis_detonates_other_burns_in_both_stores_without_self_recursion() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut test = scenario(
            &assembled,
            Probe {
                burns: true,
                break_burn: true,
                ..Probe::default()
            },
        );
        let before = test
            .battle
            .view()
            .effects_by_id()
            .find(|e| e.definition().get() == BURN && e.target() == test.targets[0])
            .unwrap();
        let before = (
            before.id(),
            before.remaining(),
            before.stacks(),
            before.source_operation(),
        );
        let applied = attack(&mut test);
        assert_eq!(applications(&applied), 1);
        assert!(applied.iter().all(|e| !matches!(
            e.kind(),
            BattleEventKind::Effect(EffectEventData::Detonated { .. })
        )));
        let events = until_tick(&mut test);
        assert_eq!(necrosis_damage(&events), [1_200_000_000]);
        let ordinary = events
            .iter()
            .filter_map(|event| match event.kind() {
                BattleEventKind::Damage(data) if data.kind == DamageKind::DotDetonation => {
                    assert_eq!(event.cause().applier(), Some(test.seeder));
                    assert_eq!(data.source_effect, Some(before.0));
                    Some(data.raw.scaled())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(ordinary, [200_000_000]);
        let after = test
            .battle
            .view()
            .effects_by_id()
            .find(|e| e.id() == before.0)
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
        let mut natural = None;
        let mut detonated = None;
        for event in &events {
            if let BattleEventKind::BreakDamage(data) = event.kind() {
                assert_eq!(event.cause().applier(), Some(test.seeder));
                match data.kind {
                    BreakDamageKind::Effect => natural = Some(data.raw.scaled()),
                    BreakDamageKind::EffectDetonation => detonated = Some(data.raw.scaled()),
                    _ => {}
                }
            }
        }
        assert_eq!(detonated.unwrap(), natural.unwrap() * 2);
        let ids = events
            .iter()
            .filter_map(|e| match e.kind() {
                BattleEventKind::Effect(EffectEventData::Detonated { effect, .. })
                | BattleEventKind::Toughness(ToughnessEventData::BaseEffectDetonated {
                    effect,
                    ..
                }) => Some(effect.get()),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(ids.len(), 2);
        assert!(ids[0] < ids[1]);
        assert_eq!(test.battle.view().rng_draw_count(), 0);
    }
}

#[test]
fn weighted_curio_necrosis_replaces_cross_caster_source_and_recaptures_attack() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut test = scenario(
            &assembled,
            Probe {
                second_abundance: true,
                ..Probe::default()
            },
        );
        attack(&mut test);
        let old = test
            .battle
            .view()
            .effects_by_id()
            .find(|e| e.definition().get() == NECROSIS)
            .unwrap()
            .id();
        let seeder = test.seeder;
        assert_eq!(applications(&cast(&mut test, seeder, ATTACK)), 1);
        let effects = test
            .battle
            .view()
            .effects_by_id()
            .filter(|e| e.definition().get() == NECROSIS)
            .collect::<Vec<_>>();
        assert_eq!(effects.len(), 1);
        assert_ne!(effects[0].id(), old);
        assert_eq!(
            (
                effects[0].applier(),
                effects[0].magnitude().scaled(),
                effects[0].remaining()
            ),
            (seeder, 600_000_000, Some(3))
        );
    }
}

#[test]
fn weighted_curio_necrosis_external_detonation_produces_one_bounded_other_burn_reaction() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let mut test = scenario(
            &assembled,
            Probe {
                burns: true,
                ..Probe::default()
            },
        );
        attack(&mut test);
        let seeder = test.seeder;
        let events = cast(&mut test, seeder, DETONATE);
        assert_eq!(necrosis_damage(&events), [1_200_000_000]);
        let reactions=events.iter().filter(|e|matches!(e.kind(),BattleEventKind::Effect(EffectEventData::Detonated{fraction,..}) if fraction.scaled()==2_000_000)).count();
        assert_eq!(reactions, 1);
        assert_eq!(
            test.battle
                .view()
                .effects_by_id()
                .filter(|e| e.definition().get() == NECROSIS)
                .count(),
            1
        );
    }
}

#[test]
fn weighted_curio_necrosis_preserves_rejections_fresh_handoffs_and_unequip() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let assembled = equipped(&fixture, family);
        let run = || {
            let mut test = scenario(
                &assembled,
                Probe {
                    burns: true,
                    break_burn: true,
                    ..Probe::default()
                },
            );
            let hash = test.battle.state_hash();
            let rng = test.battle.view().rng_draw_count();
            let invalid = Command::StartBattle {
                decision: test.battle.decision().unwrap().id(),
            };
            assert!(test.battle.apply(invalid).is_err());
            assert_eq!(hash, test.battle.state_hash());
            assert_eq!(rng, test.battle.view().rng_draw_count());
            let mut events = attack(&mut test);
            events.extend(until_tick(&mut test));
            let bytes = events
                .iter()
                .map(|e| encode_battle_event_payload(e).unwrap())
                .collect::<Vec<_>>();
            (events, bytes, test.battle.state_hash())
        };
        assert_eq!(run(), run());
        let (flow, mut activity) = ready(&fixture, family);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let selected = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_necroses()[0]
            .weighted_curio
            .clone();
        let hash = activity.state_hash();
        runtime
            .replace_accepted_loadout(
                &flow,
                &mut activity,
                hash,
                WeightedCurioSlotLimit::new(1).unwrap(),
                std::slice::from_ref(&selected),
            )
            .unwrap();
        let enabled = assemble(&fixture, &flow, &activity);
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
        let disabled = assemble(&fixture, &flow, &activity);
        assert_ne!(enabled.battle_spec(), disabled.battle_spec());
        let mut test = scenario(&disabled, Probe::default());
        assert_eq!(applications(&attack(&mut test)), 0);
        let actual = || {
            let (flow, mut activity) = ready(&fixture, family);
            let hash = activity.state_hash();
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    std::slice::from_ref(&selected),
                )
                .unwrap();
            result(&fixture, &flow, &mut activity)
        };
        assert_eq!(actual(), actual());
    }
}
