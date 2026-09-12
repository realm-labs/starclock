//! Real consecutive battles preserve cross-battle resources and bind each scope.

use super::{instance, reward_draws, slot_value};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBattleAssemblyPolicy, state::LAYER_SEQUENCE_SLOT,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityTerminalOutcome, ActivityValue, AttemptId,
    BattleOutcome, BattleResult, BattleSequence, ProjectedValue,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;

#[test]
fn layer_battles_bind_each_destination_and_carry_prior_hp_energy_without_reset() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(23_901))
            .unwrap()
            .into_activity();
        for _ in 0..2 {
            runner
                .advance(factory, &flow, &mut activity, fixture.core(), &policy)
                .unwrap();
        }
        let mut old_result = None;
        let mut battle_nodes = Vec::new();
        for ordinal in 1..=3 {
            if ordinal > 1 {
                assert!(
                    flow.offered_battle_domain_choices(&activity)
                        .unwrap()
                        .is_some()
                );
                runner
                    .advance(factory, &flow, &mut activity, fixture.core(), &policy)
                    .unwrap();
            }
            let view = activity.player_view();
            assert_eq!(
                view.decision().unwrap().kind(),
                ActivityDecisionKind::Encounter
            );
            assert_eq!(
                slot_value(&view, LAYER_SEQUENCE_SLOT),
                &ActivityValue::BoundedInteger(i64::from(ordinal))
            );
            let previous_carry = view.participant_carry().to_vec();
            let destination = flow.encounter_destination(activity.current_node()).unwrap();
            let contribution = factory
                .contribution_snapshot_runtime()
                .unwrap()
                .snapshot(&flow, &activity)
                .unwrap();
            let (group, stage) = flow.offered_encounter(&activity).unwrap().unwrap();
            let encounter = factory
                .encounter_reachability_runtime()
                .unwrap()
                .select_stage_candidate(&activity, view.state_hash(), group, stage)
                .unwrap();
            let assembled = factory.battle_assembly_runtime().materialize_current_battle(
                &flow, &activity, fixture.core(), &contribution, &encounter,
                DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
            ).unwrap();
            let runtime = factory.battle_settlement_runtime();
            let start = runtime
                .start_current_battle(
                    &flow,
                    &mut activity,
                    view.state_hash(),
                    AttemptId::new(ordinal).unwrap(),
                    BattleSequence::new(ordinal).unwrap(),
                    &assembled,
                )
                .unwrap();
            assert_eq!(activity.current_node(), destination.0);
            assert_eq!(destination.1.get(), ordinal);
            assert!(!battle_nodes.contains(&destination.0));
            battle_nodes.push(destination.0);
            if ordinal > 1 {
                assert_eq!(previous_carry.len(), 4);
                for current in start.handoff().participant_carry() {
                    let previous = previous_carry
                        .iter()
                        .find(|entry| entry.participant() == current.participant())
                        .unwrap();
                    assert_eq!(
                        current.current_hp(),
                        previous.current_hp().min(current.maximum_hp())
                    );
                    assert_eq!(
                        current.current_energy(),
                        previous.current_energy().min(current.maximum_energy())
                    );
                    assert_eq!(current.life(), previous.life());
                    assert_eq!(current.presence(), previous.presence());
                }
            }
            if let Some(result) = old_result.take() {
                let before = activity.canonical_state_bytes();
                let hash = activity.state_hash();
                assert!(
                    runtime
                        .settle_started_result(&flow, &mut activity, hash, result, None)
                        .is_err()
                );
                assert_eq!(activity.canonical_state_bytes(), before);
            }
            let (result, report) = start.execute().unwrap();
            old_result = Some(result.clone());
            let hash = activity.state_hash();
            let settled = runtime
                .settle_started_result(&flow, &mut activity, hash, result, Some(report))
                .unwrap();
            assert_eq!(settled.settlement().outcome(), BattleOutcome::Won);
            assert_eq!(activity.player_view().completed_battle_count(), ordinal);
            assert_eq!(
                activity.player_view().decision().unwrap().kind(),
                ActivityDecisionKind::Reward
            );
            runner
                .advance(factory, &flow, &mut activity, fixture.core(), &policy)
                .unwrap();
        }
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }
}

#[test]
fn loss_in_a_later_layer_terminates_without_sampling_victory_rewards() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runner = DivergentUniverseBaselineRunner::default();
    let policy = fixture.policy().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let mut activity = flow
            .start(instance(1), ActivityMasterSeed::from_u64(23_902))
            .unwrap()
            .into_activity();
        for _ in 0..5 {
            runner
                .advance(factory, &flow, &mut activity, fixture.core(), &policy)
                .unwrap();
        }
        assert_eq!(activity.player_view().completed_battle_count(), 1);
        assert_eq!(
            slot_value(&activity.player_view(), LAYER_SEQUENCE_SLOT),
            &ActivityValue::BoundedInteger(2)
        );
        let hash = activity.state_hash();
        let contribution = factory
            .contribution_snapshot_runtime()
            .unwrap()
            .snapshot(&flow, &activity)
            .unwrap();
        let (group, stage) = flow.offered_encounter(&activity).unwrap().unwrap();
        let encounter = factory
            .encounter_reachability_runtime()
            .unwrap()
            .select_stage_candidate(&activity, hash, group, stage)
            .unwrap();
        let assembled = factory.battle_assembly_runtime().materialize_current_battle(
            &flow, &activity, fixture.core(), &contribution, &encounter,
            DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy,
        ).unwrap();
        let runtime = factory.battle_settlement_runtime();
        let start = runtime
            .start_current_battle(
                &flow,
                &mut activity,
                hash,
                AttemptId::new(2).unwrap(),
                BattleSequence::new(2).unwrap(),
                &assembled,
            )
            .unwrap();
        let (result, _) = start.execute().unwrap();
        // A counterfactual validated result probes the graph's loss boundary;
        // this does not claim these calibrated enemies caused a natural defeat.
        let lost = BattleResult::seal(
            result.identity(),
            result
                .values()
                .iter()
                .map(|value| match value {
                    ProjectedValue::Outcome(_) => ProjectedValue::Outcome(BattleOutcome::Lost),
                    value => value.clone(),
                })
                .collect(),
        );
        let hash = activity.state_hash();
        let draws = reward_draws(&activity);
        runtime
            .settle_started_result(&flow, &mut activity, hash, lost.clone(), None)
            .unwrap();
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Failed)
        );
        assert!(activity.player_view().decision().is_none());
        assert_eq!(reward_draws(&activity), draws);
        let before = activity.canonical_state_bytes();
        let hash = activity.state_hash();
        assert!(
            runtime
                .settle_started_result(&flow, &mut activity, hash, lost, None)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}
