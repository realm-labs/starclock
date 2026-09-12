//! Ordered generators read applied state, with one outer settlement rollback.

use super::{amount, condition, credit, reward, reward_offer, start, with_outcome};
use starclock_activity::{
    ActivityOperation, ActivityProgramId, ActivityTerminalOutcome, ActivityTransactionEventKind,
    ActivityValue, BattleOutcome, GraphActivityCommandError, MAX_ACTIVITY_PROGRAM_OPERATIONS,
};

fn stages() -> [ActivityProgramId; 2] {
    [101, 102].map(|id| ActivityProgramId::new(id).unwrap())
}

#[test]
fn generated_settlement_stages_observe_prior_state_before_single_offer_and_replay() {
    let run = || {
        let (mut activity, result) = start(reward_offer());
        let stages = stages();
        let mut invoked = Vec::new();
        let resolution = activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                result,
                &stages,
                |id, view, settlement, streams| {
                    invoked.push(id);
                    assert!(view.decision().is_none());
                    assert_eq!(view.completed_battle_count(), 1);
                    assert_eq!(settlement.outcome(), BattleOutcome::Won);
                    reward(streams)?;
                    if id == stages[0] {
                        assert_eq!(amount(view), &ActivityValue::BoundedInteger(17));
                        assert_eq!(view.state_hash(), settlement.state_hash());
                        Ok(vec![credit(2)])
                    } else {
                        assert_eq!(amount(view), &ActivityValue::BoundedInteger(19));
                        assert_ne!(view.state_hash(), settlement.state_hash());
                        // The final offer is disabled until this stage runs.
                        Ok(vec![credit(4)])
                    }
                },
            )
            .unwrap();
        assert_eq!(invoked, stages);
        assert_eq!(
            amount(&activity.player_view()),
            &ActivityValue::BoundedInteger(23)
        );
        assert_eq!(
            activity.player_view().decision().unwrap().options().len(),
            1
        );
        assert_eq!(resolution.events().len(), 3);
        for (event, id) in resolution.events().iter().zip(stages) {
            assert_eq!(event.cause().program(), id);
            assert!(matches!(
                event.kind(),
                ActivityTransactionEventKind::SlotChanged(_)
            ));
        }
        assert!(matches!(
            resolution.events()[2].kind(),
            ActivityTransactionEventKind::DecisionOffered(_)
        ));
        (
            activity.canonical_state_bytes(),
            activity.debug_view(),
            resolution.events().to_vec(),
        )
    };
    assert_eq!(run(), run());
}

#[test]
fn generated_settlement_later_stage_failures_restore_prior_stages_and_retry() {
    let (mut activity, result) = start(reward_offer());
    let stages = stages();
    let original = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    for failure in [
        None,
        Some(vec![
            credit(1),
            ActivityOperation::Require(condition(false)),
        ]),
        Some(vec![credit(100)]),
        Some(reward_offer()),
        Some(vec![ActivityOperation::Conditional {
            condition: condition(false),
            if_true: reward_offer().into_boxed_slice(),
            if_false: Vec::new().into_boxed_slice(),
        }]),
        Some(vec![credit(0); MAX_ACTIVITY_PROGRAM_OPERATIONS + 1]),
    ] {
        let mut invoked = Vec::new();
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    activity.state_hash(),
                    result.clone(),
                    &stages,
                    |id, view, _, streams| {
                        invoked.push(id);
                        reward(streams)?;
                        if id == stages[0] {
                            Ok(vec![credit(10)])
                        } else {
                            assert_eq!(amount(view), &ActivityValue::BoundedInteger(27));
                            failure
                                .clone()
                                .ok_or(GraphActivityCommandError::DecisionNotOffered)
                        }
                    },
                )
                .is_err()
        );
        assert_eq!(invoked, stages);
        assert_eq!(activity.canonical_state_bytes(), original);
        assert_eq!(activity.debug_view(), debug);
        assert!(activity.player_view().pending_battle().is_some());
    }
    activity
        .submit_pending_battle_result_with_generated_boundary(
            activity.state_hash(),
            result.clone(),
            &stages,
            |_, _, _, streams| {
                reward(streams)?;
                Ok(vec![credit(3)])
            },
        )
        .unwrap();
    let committed = activity.canonical_state_bytes();
    assert!(
        activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                result,
                &stages,
                |_, _, _, _| panic!("duplicate result cannot invoke a stage"),
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), committed);
}

#[test]
fn generated_settlement_stage_ids_are_nonempty_unique_and_bounded_before_callbacks() {
    let (mut activity, result) = start(reward_offer());
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    for ids in [
        Vec::new(),
        vec![stages()[0], stages()[0]],
        (1..=33)
            .map(|id| ActivityProgramId::new(id).unwrap())
            .collect(),
    ] {
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    activity.state_hash(),
                    result.clone(),
                    &ids,
                    |_, _, _, _| panic!("invalid stage sequence cannot generate"),
                )
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        assert_eq!(debug, activity.debug_view());
    }
    let ids = (100..132)
        .map(|id| ActivityProgramId::new(id).unwrap())
        .collect::<Vec<_>>();
    let mut invoked = Vec::new();
    activity
        .submit_pending_battle_result_with_generated_boundary(
            activity.state_hash(),
            result,
            &ids,
            |id, _, _, _| {
                invoked.push(id);
                Ok(vec![credit(1)])
            },
        )
        .unwrap();
    assert_eq!(invoked, ids);
    assert_eq!(
        amount(&activity.player_view()),
        &ActivityValue::BoundedInteger(49)
    );
}

#[test]
fn generated_settlement_final_pump_failure_restores_every_stage_and_rng() {
    for destination in [
        vec![ActivityOperation::Require(condition(false))],
        vec![credit(100)],
    ] {
        let (mut activity, result) = start(destination);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let mut called = 0;
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    activity.state_hash(),
                    result,
                    &stages(),
                    |_, _, _, streams| {
                        called += 1;
                        reward(streams)?;
                        Ok(vec![credit(3)])
                    },
                )
                .is_err()
        );
        assert_eq!(called, 2);
        assert_eq!(before, activity.canonical_state_bytes());
        assert_eq!(debug, activity.debug_view());
    }
}

#[test]
fn generated_settlement_empty_stages_preserve_verified_terminal_outcomes() {
    for outcome in [BattleOutcome::Lost, BattleOutcome::Faulted] {
        let (mut activity, original) = start(reward_offer());
        let before_rng = activity.debug_view().rng().to_vec();
        let mut called = 0;
        let resolution = activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                with_outcome(&original, outcome),
                &stages(),
                |_, view, settlement, _| {
                    called += 1;
                    assert_eq!(settlement.outcome(), outcome);
                    assert_eq!(view.terminal(), settlement.terminal());
                    Ok(Vec::new())
                },
            )
            .unwrap();
        assert_eq!(called, 2);
        assert_eq!(before_rng, activity.debug_view().rng());
        assert!(resolution.events().is_empty());
        assert_eq!(
            activity.player_view().terminal(),
            Some(if outcome == BattleOutcome::Lost {
                ActivityTerminalOutcome::Failed
            } else {
                ActivityTerminalOutcome::Faulted
            })
        );
    }
}
