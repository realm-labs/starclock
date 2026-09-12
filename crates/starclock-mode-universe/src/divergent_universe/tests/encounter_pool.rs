//! Production offers, assembly authentication and independent per-layer draws.

use super::{instance, reward_draws};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseBaselineStep, DivergentUniverseBattleAssemblyError,
    DivergentUniverseBattleAssemblyPolicy, DivergentUniverseOfferedSelection,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    verify_divergent_universe_replay,
};
use starclock_activity::{
    ActivityDecisionKind, ActivityMasterSeed, ActivityOptionId, ActivityRngLabel,
    ActivityTerminalOutcome, GraphActivity,
};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use std::collections::BTreeSet;

fn encounter_draws(activity: &GraphActivity) -> u64 {
    activity
        .debug_view()
        .rng()
        .iter()
        .filter(|snapshot| snapshot.label() == ActivityRngLabel::Encounter)
        .map(|snapshot| snapshot.draw_count())
        .sum()
}

#[test]
fn encounter_pool_draws_each_later_layer_and_replays_actual_selected_stages() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh = DivergentUniverseBaselineFixture::production().unwrap();
    let policy = fixture.policy().unwrap();
    let runner = DivergentUniverseBaselineRunner::default();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let flow = fixture.flow(family).unwrap();
        let pool = flow.encounter_pool_policy().unwrap();
        let mut seen = BTreeSet::new();
        let mut repeated_across_layers = false;
        for seed in 0..32 {
            let mut activity = flow
                .start(instance(1), ActivityMasterSeed::from_u64(seed))
                .unwrap()
                .into_activity();
            assert!(flow.offered_encounter(&activity).is_err());
            let mut steps = Vec::new();
            let mut stages = Vec::new();
            while activity.player_view().terminal().is_none() {
                assert!(steps.len() < 10);
                let view = activity.player_view();
                let decision = view.decision().unwrap();
                let before_draws = encounter_draws(&activity);
                let before_reward = reward_draws(&activity);
                let selected_stage = if decision.kind() == ActivityDecisionKind::Encounter {
                    let (group, stage) = flow.offered_encounter(&activity).unwrap().unwrap();
                    assert_eq!(group, &pool.encounter_group);
                    assert!(
                        pool.candidate_stages
                            .iter()
                            .any(|candidate| candidate.as_ref() == stage)
                    );
                    if stages.is_empty() {
                        assert_eq!(stage, pool.first_stage.as_ref());
                        assert_eq!(before_draws, 0);
                    } else {
                        assert!(before_draws > 0);
                        seen.insert(stage.to_owned());
                    }
                    let hidden = (1..=5)
                        .find(|id| *id != decision.options()[0].id().get())
                        .unwrap();
                    let bytes = activity.canonical_state_bytes();
                    assert!(
                        runner
                            .advance_selected(
                                fixture.factory(),
                                &flow,
                                &mut activity,
                                fixture.core(),
                                &policy,
                                DivergentUniverseOfferedSelection::new(
                                    decision.id(),
                                    ActivityOptionId::new(hidden).unwrap()
                                )
                            )
                            .is_err()
                    );
                    assert_eq!(activity.canonical_state_bytes(), bytes);
                    Some(stage.to_owned())
                } else {
                    None
                };
                let step = runner
                    .advance(
                        fixture.factory(),
                        &flow,
                        &mut activity,
                        fixture.core(),
                        &policy,
                    )
                    .unwrap();
                if let Some(stage) = selected_stage {
                    let DivergentUniverseBaselineStep::Battle {
                        encounter_group,
                        encounter_stage,
                        ..
                    } = &step
                    else {
                        panic!("encounter must execute battle");
                    };
                    assert_eq!(encounter_group, &pool.encounter_group);
                    assert_eq!(encounter_stage.as_ref(), stage);
                    stages.push(stage);
                    assert_eq!(encounter_draws(&activity), before_draws);
                } else if decision.kind() == ActivityDecisionKind::Reward {
                    assert_eq!(reward_draws(&activity), before_reward);
                    assert_eq!(encounter_draws(&activity), before_draws);
                    // Reusing the previous reward command cannot resample the encounter.
                    let bytes = activity.canonical_state_bytes();
                    assert!(
                        flow.choose_battle_blessing(
                            &mut activity,
                            view.state_hash(),
                            decision.id(),
                            decision.options()[0].id()
                        )
                        .is_err()
                    );
                    assert_eq!(activity.canonical_state_bytes(), bytes);
                } else if decision.kind() == ActivityDecisionKind::Route {
                    assert!(encounter_draws(&activity) > before_draws);
                    assert_eq!(reward_draws(&activity), before_reward);
                } else {
                    assert_eq!(encounter_draws(&activity), before_draws);
                }
                steps.push(step);
            }
            assert_eq!(stages.len(), 3);
            repeated_across_layers |= stages[1] == stages[2];
            if seed == 0 {
                let recorded =
                    record_divergent_universe_transcript(&fixture, &flow, &activity, seed, steps)
                        .unwrap();
                let bytes = encode_divergent_universe_replay(&recorded).unwrap();
                let replay = verify_divergent_universe_replay(&bytes, &fresh).unwrap();
                assert_eq!(replay.action_count(), 10);
                assert_eq!(replay.terminal(), ActivityTerminalOutcome::Completed);
            }
        }
        assert_eq!(
            seen,
            pool.candidate_stages
                .iter()
                .map(ToString::to_string)
                .collect()
        );
        assert!(
            repeated_across_layers,
            "the seeded corpus exercises replacement"
        );
    }
}

#[test]
fn encounter_pool_rejects_other_candidate_before_assembly_cache_or_state_mutation() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let flow = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let policy = fixture.policy().unwrap();
    let factory = fixture.factory();
    let mut activity = flow
        .start(instance(1), ActivityMasterSeed::from_u64(23_911))
        .unwrap()
        .into_activity();
    for _ in 0..5 {
        DivergentUniverseBaselineRunner::default()
            .advance(factory, &flow, &mut activity, fixture.core(), &policy)
            .unwrap();
    }
    let (group, stage) = flow.offered_encounter(&activity).unwrap().unwrap();
    let wrong_stage = flow
        .encounter_pool_policy()
        .unwrap()
        .candidate_stages
        .iter()
        .find(|candidate| candidate.as_ref() != stage)
        .unwrap();
    let reachability = factory.encounter_reachability_runtime().unwrap();
    let selected = reachability
        .select_stage_candidate(&activity, activity.state_hash(), group, stage)
        .unwrap();
    let wrong = reachability
        .select_stage_candidate(&activity, activity.state_hash(), group, wrong_stage)
        .unwrap();
    let contribution = factory
        .contribution_snapshot_runtime()
        .unwrap()
        .snapshot(&flow, &activity)
        .unwrap();
    let runtime = factory.battle_assembly_runtime();
    let assembly_policy = DivergentUniverseBattleAssemblyPolicy::ExplicitWeeklyDisplayCandidateWithCalibratedSharedMinionProxy;
    let accepted = runtime
        .resolve_current_battle(
            &flow,
            &activity,
            fixture.core(),
            &contribution,
            &selected,
            assembly_policy,
        )
        .unwrap();
    let before = activity.canonical_state_bytes();
    let metrics = runtime.cache_metrics().unwrap();
    assert!(matches!(
        runtime.resolve_current_battle(
            &flow,
            &activity,
            fixture.core(),
            &contribution,
            &wrong,
            assembly_policy
        ),
        Err(DivergentUniverseBattleAssemblyError::InvalidEncounter)
    ));
    assert!(matches!(
        runtime.materialize_current_battle(
            &flow,
            &activity,
            fixture.core(),
            &contribution,
            &wrong,
            assembly_policy
        ),
        Err(DivergentUniverseBattleAssemblyError::InvalidEncounter)
    ));
    assert_eq!(runtime.cache_metrics().unwrap(), metrics);
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(accepted.encounter_binding(), (group, stage));
}
