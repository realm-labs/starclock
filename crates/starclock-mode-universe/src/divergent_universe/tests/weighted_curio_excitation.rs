//! Actual Sora-lowered gains, consumption, damage and ordinary control lifecycle.
use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    tests::{
        curio_battle_grants::ready,
        curio_battle_stats::assemble,
        weighted_curio_excitation_fixture::{
            BASIC, CAP, CLEANSE, ENTANGLE, GAIN, KILL_ALLY, KILL_ENEMY, LINKED_ACTION, Probe,
            SEED_STACKS, SKILL, SPEND, STACKS, ULTIMATE, accept, cast, id, idle, scenario,
        },
    },
    weighted_curio::WeightedCurioSlotLimit,
};
use starclock_combat::{
    ActionEventData, ActionGaugeChangeKind, Battle, BattleEvent, BattleEventKind, BattleSeed,
    Command, DecisionId, EffectEventData, LifeState, LinkedEntityKind, PresenceState,
    ResourceEventData, Rounding, Scalar, TeamSide, TurnEventData, UnitId,
    catalog::action::AbilityKind, formula::model::DamageClass,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_replay::battle_event::encode_battle_event_payload;
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
const PARTY: [u32; 4] = [1102, 1201, 1001, 1002];

fn equipped(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
) -> DivergentUniverseAssembledBattle {
    let (flow, mut activity) = ready(fixture, family);
    let runtime = fixture.factory().weighted_curio_runtime().unwrap();
    let curio = fixture
        .factory()
        .decision_catalog()
        .weighted_curio_excitations()[0]
        .weighted_curio
        .clone();
    let hash = activity.state_hash();
    runtime
        .replace_accepted_loadout(
            &flow,
            &mut activity,
            hash,
            WeightedCurioSlotLimit::new(1).unwrap(),
            &[curio],
        )
        .unwrap();
    let bytes = activity.canonical_state_bytes();
    let source = assemble(fixture, &flow, &activity);
    assert_eq!(bytes, activity.canonical_state_bytes());
    source
}
fn stacks(battle: &Battle) -> Vec<(u8, u16)> {
    battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition().get() == STACKS)
        .map(|effect| {
            (
                battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == effect.target())
                    .unwrap()
                    .formation()
                    .get(),
                effect.stacks(),
            )
        })
        .collect()
}
fn target(battle: &Battle) -> UnitId {
    battle
        .view()
        .units_by_id()
        .find(|unit| {
            unit.side() == TeamSide::Enemy
                && unit.presence() == PresenceState::Present
                && unit.life() == LifeState::Alive
        })
        .unwrap()
        .id()
}

#[test]
fn weighted_curio_excitation_effective_gain_excludes_overflow_cap_spend_and_nonquantum() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for (initial_sp, expected) in [(0, 2), (4, 2), (5, 0)] {
            let mut battle = scenario(
                &source,
                Probe {
                    initial_sp,
                    gain: 3,
                    ..Probe::default()
                },
            );
            let events = cast(&mut battle, 0, GAIN, None);
            assert!(events.iter().any(|event| matches!(
                event.kind(),
                BattleEventKind::Resource(ResourceEventData::SkillPoints { attempted: 3, .. })
            )));
            assert_eq!(
                stacks(&battle),
                if expected == 0 {
                    vec![]
                } else {
                    vec![(0, expected), (1, expected), (2, expected), (3, expected)]
                }
            );
            let before = stacks(&battle);
            cast(&mut battle, 0, CAP, None);
            cast(&mut battle, 0, SPEND, None);
            assert_eq!(
                stacks(&battle),
                before,
                "cap-only and negative balance are not gains"
            );
        }
        let mut battle = scenario(&source, Probe::default());
        cast(&mut battle, 2, GAIN, None);
        assert!(stacks(&battle).is_empty(), "Ice gain is not Quantum gain");
        let events = cast(&mut battle, 0, GAIN, None);
        assert!(
            additional(&events).is_empty(),
            "a nonattack Skill only grants stacks"
        );
        assert_eq!(stacks(&battle), [(0, 2), (1, 2), (2, 2), (3, 2)]);
        let victim = target(&battle);
        let events = cast(&mut battle, 2, ULTIMATE, Some(victim));
        assert!(
            additional(&events).is_empty(),
            "Ultimate Attack does not consume"
        );
        assert_eq!(stacks(&battle), [(0, 2), (1, 2), (2, 2), (3, 2)]);
    }
}

#[test]
fn weighted_curio_excitation_partial_roster_and_cap_use_recipient_local_counts() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(
            &source,
            Probe {
                defeated: Some(2),
                absent: Some(3),
                ..Probe::default()
            },
        );
        cast(&mut battle, 0, GAIN, None);
        assert_eq!(stacks(&battle), [(0, 2), (1, 2)]);
        let victim = target(&battle);
        cast(&mut battle, 1, SKILL, Some(victim));
        assert_eq!(stacks(&battle), [(0, 1), (1, 1)]);

        let mut battle = scenario(&source, Probe::default());
        cast(&mut battle, 0, SEED_STACKS, Some(UnitId::new(1).unwrap()));
        let victim = target(&battle);
        let events = cast(&mut battle, 1, SKILL, Some(victim));
        assert!(
            additional(&events).is_empty(),
            "actor must own a positive count"
        );
        assert_eq!(stacks(&battle), [(0, 65534)]);
        cast(&mut battle, 1, GAIN, None);
        assert_eq!(stacks(&battle), [(0, 65535), (1, 2), (2, 2), (3, 2)]);
        cast(&mut battle, 0, SKILL, Some(victim));
        assert_eq!(stacks(&battle), [(0, 65534), (1, 1), (2, 1), (3, 1)]);
        cast(&mut battle, 0, SKILL, Some(victim));
        assert_eq!(stacks(&battle), [(0, 65533)]);
        cast(&mut battle, 0, SKILL, Some(victim));
        assert_eq!(
            stacks(&battle),
            [(0, 65532)],
            "zero recipients never underflow"
        );
    }
}

#[test]
fn weighted_curio_excitation_multitarget_additional_crit_is_independent_of_nevercrit_hits() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for (critical_bonus, amount) in [(-50_000, 500), (950_000, 750)] {
            let mut battle = scenario(
                &source,
                Probe {
                    all: true,
                    critical_bonus,
                    ..Probe::default()
                },
            );
            let events = cast(&mut battle, 0, BASIC, None);
            assert_eq!(additional(&events), [amount; 3]);
            assert_eq!(stacks(&battle), [(0, 1), (1, 1), (2, 1), (3, 1)]);
            let targets = events
                .iter()
                .filter_map(|event| match event.kind() {
                    BattleEventKind::Damage(data) if data.class == DamageClass::Additional => {
                        Some(data.target.get())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(
                targets,
                [5, 6, 7],
                "one packet per surviving attacked target in formation order"
            );
            assert_eq!(
                battle
                    .view()
                    .effects_by_id()
                    .filter(|effect| effect.definition().get() == ENTANGLE)
                    .count(),
                3
            );
            assert!(
                events
                    .iter()
                    .filter_map(|event| match event.kind() {
                        BattleEventKind::Damage(data) if data.class == DamageClass::Direct =>
                            Some(data.calculated.get()),
                        _ => None,
                    })
                    .all(|damage| damage == 10)
            );
            assert_eq!(
                battle.view().rng_draw_count(),
                0,
                "certain outcomes do not draw"
            );
        }
    }
}

#[test]
fn weighted_curio_excitation_resistance_and_lethal_additional_do_not_apply_a_marker() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(
            &source,
            Probe {
                resistance: 1_000_000,
                ..Probe::default()
            },
        );
        let victim = target(&battle);
        let events = cast(&mut battle, 0, BASIC, Some(victim));
        assert_eq!(additional(&events), [500]);
        assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Effect(
            EffectEventData::Resisted { pre_clamp_chance, .. }) if *pre_clamp_chance == Scalar::ZERO)));
        assert!(
            !battle
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == ENTANGLE)
        );

        let mut battle = scenario(
            &source,
            Probe {
                control_resistance: 1_000_000,
                ..Probe::default()
            },
        );
        let victim = target(&battle);
        let events = cast(&mut battle, 0, BASIC, Some(victim));
        assert_eq!(additional(&events), [500]);
        assert!(events.iter().any(|event| matches!(event.kind(), BattleEventKind::Effect(
            EffectEventData::Resisted { pre_clamp_chance, .. }) if *pre_clamp_chance == Scalar::ZERO)));
        assert!(
            !battle
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == ENTANGLE)
        );

        let mut battle = scenario(
            &source,
            Probe {
                enemy_hp: 100,
                ..Probe::default()
            },
        );
        let victim = target(&battle);
        let events = cast(&mut battle, 0, BASIC, Some(victim));
        assert_eq!(additional(&events), [500]);
        assert!(
            !battle
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == ENTANGLE)
        );
    }
}

#[test]
fn weighted_curio_excitation_uncertain_crit_and_effect_chance_use_independent_reproducible_draws() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut crit_outcomes = [false; 2];
        let mut control_outcomes = [false; 2];
        for seed in 1..=16 {
            let probe = Probe {
                critical_bonus: 450_000,
                hit_rate: 0,
                seed,
                ..Probe::default()
            };
            let mut first = scenario(&source, probe);
            let mut replay = scenario(&source, probe);
            let victim = target(&first);
            let events = cast(&mut first, 0, BASIC, Some(victim));
            assert_eq!(events, cast(&mut replay, 0, BASIC, Some(victim)));
            assert_eq!(first.state_hash(), replay.state_hash());
            let damage = additional(&events);
            assert!(damage == [500] || damage == [750]);
            crit_outcomes[usize::from(damage == [750])] = true;
            let applied = first
                .view()
                .effects_by_id()
                .any(|effect| effect.definition().get() == ENTANGLE);
            control_outcomes[usize::from(applied)] = true;
            assert_eq!(
                first.view().rng_draw_count(),
                2,
                "one Crit draw and one independent effect-chance draw"
            );
        }
        assert_eq!(crit_outcomes, [true; 2]);
        assert_eq!(control_outcomes, [true; 2]);
    }
}

#[test]
fn weighted_curio_excitation_empty_quantum_roster_and_unequip_have_no_contribution() {
    let fixture =
        DivergentUniverseBaselineFixture::production_for_source_party([1001, 1002, 1005, 1202])
            .unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(&source, Probe::default());
        let victim = target(&battle);
        assert!(additional(&cast(&mut battle, 0, BASIC, Some(victim))).is_empty());
        assert!(stacks(&battle).is_empty());
    }
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let runtime = fixture.factory().weighted_curio_runtime().unwrap();
        let curio = fixture
            .factory()
            .decision_catalog()
            .weighted_curio_excitations()[0]
            .weighted_curio
            .clone();
        for loadout in [vec![curio], vec![]] {
            let hash = activity.state_hash();
            runtime
                .replace_accepted_loadout(
                    &flow,
                    &mut activity,
                    hash,
                    WeightedCurioSlotLimit::new(1).unwrap(),
                    &loadout,
                )
                .unwrap();
        }
        let bytes = activity.canonical_state_bytes();
        let source = assemble(&fixture, &flow, &activity);
        assert!(source.combat_catalog().effect(id(STACKS)).is_none());
        assert!(source.combat_catalog().effect(id(ENTANGLE)).is_none());
        let mut battle = Battle::create(
            Arc::clone(source.combat_catalog()),
            source.battle_spec().clone(),
            BattleSeed::new([0xe7; 32]),
        )
        .unwrap();
        let start = Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        };
        accept(&mut battle, start);
        let mut executed = false;
        for _ in 0..64 {
            let chosen = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command|
                matches!(command, Command::UseAbility {actor, ability, ..} if actor.get() == 1
                    && source.combat_catalog().ability(*ability).and_then(|a| a.action()).is_some_and(|a| a.kind() == AbilityKind::Basic))).cloned());
            let selected = chosen.is_some();
            let command = chosen.unwrap_or_else(|| battle.advance_command().unwrap());
            let events = accept(&mut battle, command);
            assert!(additional(&events).is_empty());
            if selected {
                executed = true;
                break;
            }
        }
        assert!(executed, "unequipped real Quantum Basic must execute");
        assert!(stacks(&battle).is_empty());
        assert_eq!(
            activity.canonical_state_bytes(),
            bytes,
            "live battle cannot mutate Activity or carry stacks into it"
        );
    }
}

#[test]
fn weighted_curio_excitation_entanglement_refresh_hits_cleanse_and_expiry_are_native() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for cleanse in [false, true] {
            let mut battle = scenario(&source, Probe::default());
            let victim = target(&battle);
            cast(&mut battle, 0, BASIC, Some(victim));
            let original = battle
                .view()
                .effects_by_id()
                .find(|effect| effect.definition().get() == ENTANGLE)
                .unwrap();
            assert_eq!(original.entanglement_hits(), Some(0));
            assert_eq!(original.entanglement_delay().unwrap().scaled(), 200_000);
            let base = original.entanglement_base();
            let events = cast(&mut battle, 1, SKILL, Some(victim));
            let refreshed = battle
                .view()
                .effects_by_id()
                .find(|effect| effect.definition().get() == ENTANGLE)
                .unwrap();
            assert_eq!(refreshed.applier().get(), 1);
            assert_eq!(refreshed.entanglement_base(), base);
            assert_eq!(refreshed.entanglement_hits(), Some(3));
            assert!(!events.iter().any(|event| matches!(
                event.kind(),
                BattleEventKind::Turn(TurnEventData::ActionGaugeChanged {
                    kind: ActionGaugeChangeKind::Delay,
                    ..
                })
            )));
            assert!(
                !events
                    .iter()
                    .any(|event| matches!(event.kind(), BattleEventKind::Toughness(_)))
            );
            assert!(
                !battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == victim)
                    .unwrap()
                    .weakness_broken()
            );
            assert_eq!(
                battle
                    .view()
                    .units_by_id()
                    .find(|unit| unit.id() == victim)
                    .unwrap()
                    .toughness_layers()
                    .next()
                    .unwrap()
                    .current()
                    .get(),
                60
            );
            if cleanse {
                let events = cast(&mut battle, 2, CLEANSE, Some(victim));
                assert!(
                    !events
                        .iter()
                        .any(|event| matches!(event.kind(), BattleEventKind::BreakDamage(_)))
                );
                assert!(
                    !battle
                        .view()
                        .effects_by_id()
                        .any(|effect| effect.definition().get() == ENTANGLE)
                );
            } else {
                cast(&mut battle, 2, SKILL, Some(victim));
                assert_eq!(
                    battle
                        .view()
                        .effects_by_id()
                        .find(|effect| effect.definition().get() == ENTANGLE)
                        .unwrap()
                        .entanglement_hits(),
                    Some(5)
                );
                let mut burst = false;
                for _ in 0..128 {
                    let events = idle(&mut battle);
                    for event in &events {
                        if let BattleEventKind::BreakDamage(data) = event.kind() {
                            assert_eq!(event.cause().applier().unwrap().get(), 1);
                            let expected = base
                                .unwrap()
                                .checked_mul(
                                    Scalar::checked_from_integer(5).unwrap(),
                                    Rounding::NearestTiesEven,
                                )
                                .unwrap()
                                .checked_mul(
                                    Scalar::from_scaled(900_000),
                                    Rounding::NearestTiesEven,
                                )
                                .unwrap()
                                .rounded_integer(Rounding::Floor)
                                .unwrap();
                            assert_eq!(data.calculated.get(), expected);
                        }
                    }
                    burst |= events
                        .iter()
                        .any(|event| matches!(event.kind(), BattleEventKind::BreakDamage(_)));
                    if !battle
                        .view()
                        .effects_by_id()
                        .any(|effect| effect.definition().get() == ENTANGLE)
                    {
                        break;
                    }
                }
                assert!(
                    burst,
                    "expiry must deal delayed damage rather than act as a marker"
                );
            }
        }
    }
}

#[test]
fn weighted_curio_excitation_stacks_survive_provider_defeat_and_wave_but_not_fresh_battle() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(&source, Probe::default());
        cast(&mut battle, 0, GAIN, None);
        cast(&mut battle, 1, KILL_ALLY, Some(UnitId::new(1).unwrap()));
        assert!(
            stacks(&battle)
                .iter()
                .filter(|entry| entry.0 != 0)
                .all(|entry| entry.1 == 2)
        );
        for _ in 0..3 {
            let victim = target(&battle);
            cast(&mut battle, 1, KILL_ENEMY, Some(victim));
        }
        assert_eq!(battle.view().encounter().number(), 2);
        assert!(
            stacks(&battle)
                .iter()
                .filter(|entry| entry.0 != 0)
                .all(|entry| entry.1 == 2)
        );
        assert!(stacks(&scenario(&source, Probe::default())).is_empty());
    }
}

#[test]
fn weighted_curio_excitation_inherited_linked_and_shared_actors_cannot_gain_or_consume() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        for kind in [
            LinkedEntityKind::Summon,
            LinkedEntityKind::Memosprite,
            LinkedEntityKind::SharedActor,
        ] {
            for presence in [PresenceState::Linked, PresenceState::Present] {
                let mut battle = scenario(
                    &source,
                    Probe {
                        linked_kind: Some(kind),
                        linked_presence: presence,
                        ..Probe::default()
                    },
                );
                cast(&mut battle, 0, GAIN, None);
                let before = stacks(&battle);
                let events = cast(&mut battle, 0, ULTIMATE, None);
                let resolved = |events: &[BattleEvent]| {
                    events.iter().any(|event| matches!(event.kind(),
                    BattleEventKind::Action(ActionEventData::Resolved { ability, .. }) if ability.get() == LINKED_ACTION))
                };
                let mut executed = resolved(&events);
                assert!(additional(&events).is_empty());
                for _ in 0..64 {
                    if executed {
                        break;
                    }
                    let events = idle(&mut battle);
                    assert!(additional(&events).is_empty());
                    executed = resolved(&events);
                }
                assert!(
                    executed,
                    "excluded actor must execute: {kind:?}/{presence:?}"
                );
                assert_eq!(stacks(&battle), before);
            }
        }
    }
}

#[test]
fn weighted_curio_excitation_fresh_reconstruction_and_rejections_preserve_payloads_rng_and_state() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let fresh = equipped(
            &DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap(),
            family,
        );
        assert_eq!(source.battle_spec(), fresh.battle_spec());
        let mut first = scenario(&source, Probe::default());
        let mut second = scenario(&fresh, Probe::default());
        let victim = target(&first);
        let first_events = cast(&mut first, 0, BASIC, Some(victim));
        let second_events = cast(&mut second, 0, BASIC, Some(victim));
        assert_eq!(
            first_events
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>(),
            second_events
                .iter()
                .map(|event| encode_battle_event_payload(event).unwrap())
                .collect::<Vec<_>>()
        );
        assert_eq!(first.state_hash(), second.state_hash());
        assert_eq!(
            first.view().rng_draw_count(),
            second.view().rng_draw_count()
        );
        let hash = first.state_hash();
        let draws = first.view().rng_draw_count();
        assert!(
            first
                .apply(Command::UseAbility {
                    decision: DecisionId::new(u64::MAX).unwrap(),
                    actor: UnitId::new(1).unwrap(),
                    ability: id(BASIC),
                    primary_target: Some(victim)
                })
                .is_err()
        );
        assert_eq!(first.state_hash(), hash);
        assert_eq!(first.view().rng_draw_count(), draws);
    }
}

#[test]
fn weighted_curio_excitation_unmodified_production_quantum_basic_executes_and_reconstructs() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = Battle::create(
            Arc::clone(source.combat_catalog()),
            source.battle_spec().clone(),
            BattleSeed::new([0xe7; 32]),
        )
        .unwrap();
        let mut replay = Battle::create(
            Arc::clone(source.combat_catalog()),
            source.battle_spec().clone(),
            BattleSeed::new([0xe7; 32]),
        )
        .unwrap();
        let start = Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        };
        assert_eq!(
            accept(&mut battle, start.clone()),
            accept(&mut replay, start)
        );
        let mut executed = false;
        for _ in 0..128 {
            let chosen = battle.decision().and_then(|decision| decision.legal_commands().iter().find(|command|
                matches!(command, Command::UseAbility {actor, ability, ..} if actor.get() == 1
                    && source.combat_catalog().ability(*ability).and_then(|a| a.action()).is_some_and(|a| a.kind() == AbilityKind::Basic))).cloned());
            let selected = chosen.is_some();
            let command = chosen.unwrap_or_else(|| battle.advance_command().unwrap());
            let events = accept(&mut battle, command.clone());
            assert_eq!(events, accept(&mut replay, command));
            assert_eq!(battle.state_hash(), replay.state_hash());
            if selected {
                assert_eq!(stacks(&battle), [(0, 1), (1, 1), (2, 1), (3, 1)]);
                assert!(!additional(&events).is_empty());
                executed = true;
                break;
            }
        }
        assert!(executed, "real Sora-mapped Quantum Basic must execute");
    }
}
fn additional(events: &[BattleEvent]) -> Vec<i64> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if data.class == DamageClass::Additional
                    && event
                        .cause()
                        .source_definition()
                        .is_some_and(|source| source.get() == 0x7ed4_0001) =>
            {
                Some(data.calculated.get())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn weighted_curio_excitation_gain_is_per_effective_event_and_consumes_once_per_multihit_action() {
    let fixture = DivergentUniverseBaselineFixture::production_for_source_party(PARTY).unwrap();
    for family in FAMILIES {
        let source = equipped(&fixture, family);
        let mut battle = scenario(&source, Probe::default());
        assert!(stacks(&battle).is_empty());
        let victim = target(&battle);
        let events = cast(&mut battle, 0, BASIC, Some(victim));
        assert_eq!(stacks(&battle), [(0, 1), (1, 1), (2, 1), (3, 1)]);
        assert_eq!(additional(&events), [500]);
        assert_eq!(
            battle
                .view()
                .effects_by_id()
                .filter(|effect| effect.definition().get() == ENTANGLE)
                .count(),
            1
        );
        let events = cast(&mut battle, 2, SKILL, Some(victim));
        assert!(stacks(&battle).is_empty());
        assert_eq!(additional(&events), [500]);
        let events = cast(&mut battle, 2, SKILL, Some(victim));
        assert!(additional(&events).is_empty());
        let events = cast(&mut battle, 0, GAIN, None);
        assert!(additional(&events).is_empty());
        assert_eq!(stacks(&battle), [(0, 2), (1, 2), (2, 2), (3, 2)]);
    }
}
