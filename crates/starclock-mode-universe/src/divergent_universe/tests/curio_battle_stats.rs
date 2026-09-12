//! Passive contribution changes actual speed; battle lifetime is transactional.

use crate::divergent_universe::{
    DivergentUniverseAssembledBattle, DivergentUniverseBaselineFixture,
    DivergentUniverseBattleAssemblyPolicy, DivergentUniverseFlowInstance,
    tests::{
        curio_battle_grants::{project, ready, result},
        domain_choices::advance,
    },
};
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityValue, BattleOutcome, BattleResult, GraphActivity, ProjectedValue,
};
use starclock_combat::{
    Battle, BattleFault, BattleSeed, Command, FaultBoundary, FaultKind, FaultPolicy, LifeState,
    TeamSide,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
};
use std::sync::Arc;

const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
fn state(raw: &str) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
pub(super) fn assemble(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &GraphActivity,
) -> DivergentUniverseAssembledBattle {
    let factory = fixture.factory();
    let contribution = factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(flow, activity)
        .unwrap();
    let (group, stage) = flow.offered_encounter(activity).unwrap().unwrap();
    let encounter = factory
        .encounter_reachability_runtime()
        .unwrap()
        .select_stage_candidate(activity, activity.state_hash(), group, stage)
        .unwrap();
    factory.battle_assembly_runtime().materialize_current_battle(flow, activity, fixture.core(), &contribution, &encounter,
        DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy).unwrap()
}
fn assert_timeline(assembled: &DivergentUniverseAssembledBattle, player_bonus_percent: i64) {
    let mut battle = Battle::create(
        Arc::clone(assembled.combat_catalog()),
        assembled.battle_spec().clone(),
        BattleSeed::new([24; 32]),
    )
    .unwrap();
    let initial = battle
        .view()
        .timeline_actors()
        .map(|actor| {
            let unit = battle
                .view()
                .units_by_id()
                .find(|unit| Some(unit.id()) == actor.unit())
                .unwrap();
            let speed = actor.speed().scaled()
                + if unit.side() == TeamSide::Player {
                    unit.base_speed().scaled() * player_bonus_percent / 100
                } else {
                    0
                };
            (
                actor.id(),
                actor.action_gauge().scaled(),
                actor.speed().scaled(),
                speed,
                actor.is_active()
                    && unit.life() == LifeState::Alive
                    && unit.presence().is_timeline_eligible(),
            )
        })
        .collect::<Vec<_>>();
    let selected = initial
        .iter()
        .filter(|entry| entry.4)
        .min_by(|left, right| {
            (i128::from(left.1) * i128::from(right.3))
                .cmp(&(i128::from(right.1) * i128::from(left.3)))
        })
        .unwrap();
    let started = battle
        .apply(Command::StartBattle {
            decision: battle.decision().unwrap().id(),
        })
        .unwrap();
    assert!(started.fault().is_none());
    assert_eq!(
        started.timeline_elapsed_scaled(),
        i64::try_from(i128::from(selected.1) * 1_000_000 / i128::from(selected.3)).unwrap()
    );
    let view = battle.view();
    for (id, gauge, raw, effective, eligible) in initial.iter().copied() {
        let actor = view
            .timeline_actors()
            .find(|actor| actor.id() == id)
            .unwrap();
        assert_eq!(
            actor.speed().scaled(),
            raw,
            "resolved speed must not compound into the independent actor clock"
        );
        let expected = if !eligible {
            gauge
        } else if id == selected.0 {
            0
        } else {
            gauge
                - i64::try_from(
                    i128::from(effective) * i128::from(selected.1) / i128::from(selected.3),
                )
                .unwrap()
        };
        assert_eq!(
            actor.action_gauge().scaled(),
            expected,
            "actual action order and gauge distance use resolved SPD"
        );
    }
}
fn charges(fixture: &DivergentUniverseBaselineFixture, activity: &GraphActivity) -> Option<u16> {
    fixture
        .factory()
        .curio_runtime()
        .unwrap()
        .owned(activity)
        .unwrap()
        .iter()
        .find(|held| held.state() == &state("9068"))
        .map(|held| held.charges())
}

pub(super) fn assert_public_speed_passive(
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &GraphActivity,
) {
    let assembled = assemble(fixture, flow, activity);
    for participant in assembled
        .battle_spec()
        .participants()
        .iter()
        .filter(|participant| participant.side() == TeamSide::Player)
    {
        assert!(
            participant
                .combatant()
                .modifiers()
                .iter()
                .any(|id| id.get() == 0x7e28_0001)
        );
    }
    assert_timeline(&assembled, 35);
}

#[test]
fn curio_battle_stats_change_real_speed_preserve_build_and_follow_lifecycle() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        let (flow, mut activity) = ready(&fixture, family);
        let control = assemble(&fixture, &flow, &activity);
        assert_timeline(&control, 0);
        let hash = activity.state_hash();
        curios
            .acquire_accepted_state(&mut activity, hash, &state("9068"))
            .unwrap();
        for transition in 0..4 {
            let hash = activity.state_hash();
            match transition {
                1 => {
                    curios
                        .destroy_accepted(&mut activity, hash, &state("9068"))
                        .unwrap();
                }
                2 => {
                    curios
                        .repair_accepted(&mut activity, hash, &state("9068"))
                        .unwrap();
                }
                3 => {
                    curios
                        .replace_accepted(&mut activity, hash, &state("9068"), &state("9070"))
                        .unwrap();
                }
                _ => {}
            }
            let bytes = activity.canonical_state_bytes();
            if transition == 1 {
                assert!(
                    curios
                        .battle_lifetime_operations(&activity.player_view())
                        .unwrap()
                        .is_empty()
                );
            }
            let assembled = assemble(&fixture, &flow, &activity);
            assert_timeline(&assembled, if transition % 2 == 0 { 35 } else { 0 });
            assert_eq!(
                activity.canonical_state_bytes(),
                bytes,
                "assembly cannot consume charges"
            );
            for (old, new) in control
                .battle_spec()
                .participants()
                .iter()
                .zip(assembled.battle_spec().participants())
            {
                if old.side() == TeamSide::Enemy || transition % 2 == 1 {
                    assert_eq!(old, new);
                    continue;
                }
                assert_eq!(old.locked_combatant_digest(), new.locked_combatant_digest());
                let old = old.combatant();
                let new = new.combatant();
                assert_eq!(old.form(), new.form());
                assert_eq!(old.level(), new.level());
                assert_eq!(old.maximum_hp(), new.maximum_hp());
                assert_eq!(old.speed(), new.speed());
                assert_eq!(old.base_attack(), new.base_attack());
                assert_eq!(old.base_defense(), new.base_defense());
                assert_eq!(old.base_effect_hit_rate(), new.base_effect_hit_rate());
                assert_eq!(old.base_effect_resistance(), new.base_effect_resistance());
                assert_eq!(old.build_bonuses(), new.build_bonuses());
                assert_eq!(old.current_energy(), new.current_energy());
                assert_eq!(old.maximum_energy(), new.maximum_energy());
                assert_eq!(old.rank(), new.rank());
                assert_eq!(old.weaknesses(), new.weaknesses());
                assert_eq!(old.toughness_layers(), new.toughness_layers());
                assert_eq!(old.abilities(), new.abilities());
                assert_eq!(old.rule_bundles(), new.rule_bundles());
                assert_eq!(new.modifiers().len(), old.modifiers().len() + 1);
                assert!(
                    old.modifiers()
                        .iter()
                        .all(|id| new.modifiers().contains(id))
                );
                assert!(
                    old.sources()
                        .iter()
                        .all(|source| new.sources().contains(source))
                );
                assert!(
                    old.modifier_bindings()
                        .iter()
                        .all(|binding| new.modifier_bindings().contains(binding))
                );
                assert!(
                    new.modifier_bindings()
                        .iter()
                        .any(|binding| binding.definition().get() == 0x7e28_0001
                            && binding.applies_to_linked_subjects())
                );
            }
        }
    }
}

#[test]
fn curio_battle_stats_late_blessing_rejection_restores_discard_carry_and_rng() {
    use super::reward_draws;
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let (flow, mut activity) = ready(&fixture, FAMILIES[0]);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9068"))
        .unwrap();
    let hash = activity.state_hash();
    curios
        .set_accepted_charges(&mut activity, hash, &state("9068"), 1)
        .unwrap();
    let actual = result(&fixture, &flow, &mut activity);
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    let hash = activity.state_hash();
    let expiry = ActivityProgramId::new(24110).unwrap();
    let offer = ActivityProgramId::new(23702).unwrap();
    let mut reached_discard = false;
    assert!(
        activity
            .submit_pending_battle_result_with_generated_boundary(
                hash,
                actual.clone(),
                &[expiry, offer],
                |program, view, _, rng| {
                    if program == expiry {
                        return Ok(curios.battle_lifetime_operations(view).unwrap());
                    }
                    reached_discard = curios
                        .owned_from_view(view)
                        .unwrap()
                        .iter()
                        .all(|held| held.state() != &state("9068"));
                    let mut operations = flow
                        .battle_blessings
                        .as_ref()
                        .unwrap()
                        .generate(view, rng)?;
                    operations.push(ActivityOperation::Require(ActivityCondition::Boolean(
                        ActivityExpression::Literal(ActivityValue::Boolean(false)),
                    )));
                    Ok(operations)
                }
            )
            .is_err()
    );
    assert!(reached_discard);
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(reward_draws(&activity), draws);
    assert_eq!(charges(&fixture, &activity), Some(1));
    fixture
        .factory()
        .battle_settlement_runtime()
        .settle_started_result(&flow, &mut activity, hash, actual, None)
        .unwrap();
    assert_eq!(charges(&fixture, &activity), None);
}

#[test]
fn curio_battle_stats_five_battle_allowance_is_not_domain_counted() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    let (_, mut activity) = ready(&fixture, FAMILIES[0]);
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9068"))
        .unwrap();
    assert_eq!(charges(&fixture, &activity), Some(5));
    assert!(
        curios
            .domain_entry_operations(&activity.player_view())
            .unwrap()
            .is_empty()
    );
    for remaining in (1..=5).rev() {
        assert_eq!(charges(&fixture, &activity), Some(remaining));
        // Operation vector, not five public battles in the current short route.
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(24110).unwrap(),
            curios
                .battle_lifetime_operations(&activity.player_view())
                .unwrap(),
        )
        .unwrap();
        activity
            .apply_boundary_program(activity.state_hash(), &program)
            .unwrap();
    }
    assert_eq!(charges(&fixture, &activity), None);
    let hash = activity.state_hash();
    assert!(
        curios
            .repair_accepted(&mut activity, hash, &state("9068"))
            .is_err()
    );
    curios
        .acquire_accepted_state(&mut activity, hash, &state("9068"))
        .unwrap();
    assert_eq!(charges(&fixture, &activity), Some(5));
}

#[test]
fn curio_battle_stats_verified_results_discard_and_faults_do_not_consume() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let curios = fixture.factory().curio_runtime().unwrap();
    for family in FAMILIES {
        for outcome in [
            BattleOutcome::Won,
            BattleOutcome::Lost,
            BattleOutcome::Faulted,
        ] {
            let (flow, mut activity) = ready(&fixture, family);
            let hash = activity.state_hash();
            curios
                .acquire_accepted_state(&mut activity, hash, &state("9068"))
                .unwrap();
            let hash = activity.state_hash();
            curios
                .set_accepted_charges(&mut activity, hash, &state("9068"), 1)
                .unwrap();
            let actual = result(&fixture, &flow, &mut activity);
            let projected = project(&actual, 4, outcome);
            let projected = BattleResult::seal(
                projected.identity(),
                projected
                    .values()
                    .iter()
                    .map(|value| {
                        if outcome == BattleOutcome::Faulted
                            && matches!(value, ProjectedValue::TerminalFault(_))
                        {
                            ProjectedValue::TerminalFault(Some(BattleFault::from_parts(
                                FaultKind::Numeric,
                                FaultBoundary::Command,
                                FaultPolicy::Rollback,
                                1,
                                None,
                            )))
                        } else {
                            value.clone()
                        }
                    })
                    .collect(),
            );
            let hash = activity.state_hash();
            let settled = fixture
                .factory()
                .battle_settlement_runtime()
                .settle_started_result(&flow, &mut activity, hash, projected.clone(), None)
                .unwrap();
            assert_eq!(
                charges(&fixture, &activity),
                if outcome == BattleOutcome::Faulted {
                    Some(1)
                } else {
                    None
                }
            );
            assert_eq!(
                settled
                    .events()
                    .iter()
                    .any(|event| event.cause().program().get() == 24110),
                outcome != BattleOutcome::Faulted
            );
            let bytes = activity.canonical_state_bytes();
            let hash = activity.state_hash();
            assert!(
                fixture
                    .factory()
                    .battle_settlement_runtime()
                    .settle_started_result(&flow, &mut activity, hash, projected, None)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), bytes);
            if outcome == BattleOutcome::Won {
                advance(&fixture, &flow, &mut activity, None); // ordinary Blessing selection
                advance(&fixture, &flow, &mut activity, None); // next domain
                let next = assemble(&fixture, &flow, &activity);
                assert!(
                    !next
                        .battle_spec()
                        .participants()
                        .iter()
                        .any(|participant| participant
                            .combatant()
                            .modifiers()
                            .iter()
                            .any(|id| id.get() == 0x7e28_0001))
                );
            }
        }
    }
}
