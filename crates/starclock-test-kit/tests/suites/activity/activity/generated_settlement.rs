//! Generic settlement ordering and rollback, not a mode reward parity fixture.

#[path = "generated_settlement_stages.rs"]
mod stages;

use std::sync::Arc;

use starclock_activity::{
    ActivityBattlePreparationRequest, ActivityCondition, ActivityDecisionKind,
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression,
    ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityPlayerView, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel, ActivityRngStreams,
    ActivityRosterLock, ActivityScopePath, ActivityStateHash, ActivityTerminalOutcome,
    ActivityTransactionEventKind, ActivityValue, AttemptId, BattleOutcome, BattleResult,
    BattleSequence, GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, MAX_ACTIVITY_PROGRAM_OPERATIONS, ProjectedValue,
};
use starclock_combat::{
    BattleFault, FaultBoundary, FaultKind, FaultPolicy, LifeState, PresenceState,
};

use super::{
    Setup, edge, graph_node, hp, node, participant_state, result, section, slot, state_definition,
};

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn condition(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(value)))
}

fn credit(value: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot: slot(1),
        delta: integer(value),
    }
}

fn reward(streams: &mut ActivityRngStreams) -> Result<(), GraphActivityCommandError> {
    streams
        .choose_index(ActivityRngLabel::Reward, 301, 5)
        .map_err(GraphActivityCommandError::Rng)?
        .expect("five reward candidates");
    Ok(())
}

fn amount(view: &ActivityPlayerView) -> &ActivityValue {
    view.slots()
        .iter()
        .find(|value| value.id() == slot(1))
        .unwrap()
        .value()
}

fn program_id() -> ActivityProgramId {
    ActivityProgramId::new(99).unwrap()
}

fn reward_offer() -> Vec<ActivityOperation> {
    vec![ActivityOperation::Offer {
        kind: ActivityDecisionKind::Reward,
        options: vec![ActivityOptionDefinition::new(
            ActivityOptionId::new(21).unwrap(),
            0,
            ActivityCondition::LessThan(integer(20), ActivityExpression::Slot(slot(1))),
            vec![ActivityOperation::Traverse(
                ActivityEdgeId::new(21).unwrap(),
            )],
        )]
        .into_boxed_slice(),
    }]
}

fn start(destination: Vec<ActivityOperation>) -> (GraphActivity, BattleResult) {
    let setup = Setup::new(false);
    let graph = ActivityGraphDefinition::new(
        node(10),
        vec![
            graph_node(10, ActivityNodeKind::Choice),
            graph_node(20, ActivityNodeKind::Battle),
            graph_node(21, ActivityNodeKind::Reward),
            graph_node(
                22,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
            ),
            graph_node(
                23,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            ),
            graph_node(
                24,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            ),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(10).unwrap(),
                node(10),
                node(20),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
            edge(1, node(20), node(21), BattleOutcome::Won),
            edge(2, node(20), node(22), BattleOutcome::Lost),
            edge(3, node(20), node(23), BattleOutcome::Faulted),
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(21).unwrap(),
                node(21),
                node(24),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        4,
    )
    .unwrap();
    let programs = [
        (
            10,
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::Encounter,
                options: vec![ActivityOptionDefinition::new(
                    ActivityOptionId::new(10).unwrap(),
                    0,
                    condition(true),
                    vec![ActivityOperation::Traverse(
                        ActivityEdgeId::new(10).unwrap(),
                    )],
                )]
                .into_boxed_slice(),
            }],
        ),
        (20, vec![]),
        (21, destination),
    ]
    .into_iter()
    .map(|(id, operations)| {
        GraphActivityNodeProgram::new(
            node(id),
            ActivityProgramDefinition::new(ActivityProgramId::new(id).unwrap(), operations)
                .unwrap(),
        )
    })
    .collect();
    let definition = GraphActivityDefinition::new(
        setup.identity,
        graph,
        state_definition(),
        Arc::new(setup.roster.clone()),
        programs,
        None,
        ActivityRandomPolicies::default(),
    )
    .unwrap();
    let mut activity = GraphActivity::start(
        Arc::new(definition),
        setup.instance,
        ActivityMasterSeed::from_u64(5),
    )
    .unwrap()
    .into_activity();
    let request = ActivityBattlePreparationRequest::new(
        ActivityScopePath::new(setup.instance)
            .enter_section(section())
            .unwrap()
            .enter_node(node(20))
            .unwrap()
            .enter_attempt(AttemptId::new(1).unwrap())
            .unwrap(),
        ActivityRosterLock::new(ActivityScopePath::new(setup.instance), setup.roster).unwrap(),
        BattleSequence::new(1).unwrap(),
        0,
        setup.preparation,
    );
    activity
        .engage_encounter(
            activity.state_hash(),
            activity.player_view().decision().unwrap().id(),
            ActivityOptionId::new(10).unwrap(),
            request,
        )
        .unwrap();
    activity
        .choose_preparation_option(activity.state_hash(), ActivityOptionId::new(10).unwrap())
        .unwrap();
    let handoff = activity
        .start_pending_battle(activity.state_hash(), setup.contract)
        .unwrap();
    let result = result(
        handoff.identity(),
        BattleOutcome::Won,
        participant_state(700, 60, LifeState::Alive, PresenceState::Present),
        12,
    );
    (activity, result)
}

#[test]
fn generated_settlement_runs_after_verified_carry_and_before_reward_eligibility() {
    let (mut left, result) = start(reward_offer());
    let (mut right, replay_result) = start(reward_offer());
    let initial_rng = left.debug_view().rng().to_vec();
    let run = |activity: &mut GraphActivity, result: BattleResult| {
        activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                result,
                &[program_id()],
                |_, view, settlement, streams| {
                    assert_eq!(settlement.outcome(), BattleOutcome::Won);
                    assert_eq!(settlement.target(), node(21));
                    assert_eq!(view.current_node(), node(21));
                    assert_eq!(view.state_hash(), settlement.state_hash());
                    assert_eq!(amount(view), &ActivityValue::BoundedInteger(17));
                    assert_eq!(view.participant_carry()[0].current_hp(), hp(700));
                    assert_eq!(view.completed_battle_count(), 1);
                    assert!(view.pending_battle().is_none());
                    assert!(view.decision().is_none());
                    reward(streams)?;
                    Ok(vec![credit(10)])
                },
            )
            .unwrap()
    };
    let resolution = run(&mut left, result.clone());
    let replayed = run(&mut right, replay_result);
    assert_eq!(resolution.settlement(), replayed.settlement());
    assert_eq!(resolution.events(), replayed.events());
    assert_eq!(resolution.state_hash(), replayed.state_hash());
    assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_ne!(initial_rng, left.debug_view().rng());
    let view = left.player_view();
    assert_eq!(amount(&view), &ActivityValue::BoundedInteger(27));
    assert_eq!(
        view.decision().unwrap().kind(),
        ActivityDecisionKind::Reward
    );
    assert_eq!(view.decision().unwrap().options().len(), 1);
    let kinds = resolution
        .events()
        .iter()
        .map(|event| event.kind())
        .collect::<Vec<_>>();
    assert!(matches!(
        kinds[0],
        ActivityTransactionEventKind::SlotChanged(_)
    ));
    assert!(matches!(
        kinds[1],
        ActivityTransactionEventKind::DecisionOffered(_)
    ));
    let before = left.canonical_state_bytes();
    assert!(
        left.submit_pending_battle_result_with_generated_boundary(
            left.state_hash(),
            result,
            &[program_id()],
            |_, _, _, _| panic!("duplicate cannot generate"),
        )
        .is_err()
    );
    assert_eq!(before, left.canonical_state_bytes());
}

#[test]
fn post_advance_follow_up_cannot_supply_pre_offer_eligibility() {
    let (mut activity, result) = start(reward_offer());
    let before = activity.canonical_state_bytes();
    assert!(
        activity
            .submit_pending_battle_result_with_generated_follow_up(
                activity.state_hash(),
                result,
                None,
                program_id(),
                |view, streams| {
                    // The old boundary pumps first: the empty offer has already
                    // faulted, so its follow-up cannot repair offer eligibility.
                    assert_eq!(view.terminal(), Some(ActivityTerminalOutcome::Faulted));
                    assert!(view.decision().is_none());
                    reward(streams)?;
                    Ok(vec![credit(10)])
                },
            )
            .is_err()
    );
    assert_eq!(before, activity.canonical_state_bytes());
}

#[test]
fn generated_settlement_rejects_stale_and_unverified_results_before_generation() {
    let (mut activity, result) = start(reward_offer());
    let before = activity.canonical_state_bytes();
    assert!(
        activity
            .submit_pending_battle_result_with_generated_boundary(
                ActivityStateHash::new([0xaa; 32]).unwrap(),
                result.clone(),
                &[program_id()],
                |_, _, _, _| panic!("stale result cannot generate"),
            )
            .is_err()
    );
    let invalid = BattleResult::seal(result.identity(), vec![]);
    assert!(
        activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                invalid,
                &[program_id()],
                |_, _, _, _| panic!("invalid projection cannot generate"),
            )
            .is_err()
    );
    assert_eq!(before, activity.canonical_state_bytes());
}

#[test]
fn generated_settlement_error_restores_result_carry_rng_and_allows_retry() {
    let (mut activity, result) = start(reward_offer());
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    assert!(
        activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                result.clone(),
                &[program_id()],
                |_, _, _, streams| {
                    reward(streams)?;
                    Err(GraphActivityCommandError::DecisionNotOffered)
                },
            )
            .is_err()
    );
    assert_eq!(before, activity.canonical_state_bytes());
    assert_eq!(debug, activity.debug_view());
    assert!(activity.player_view().pending_battle().is_some());
    activity
        .submit_pending_battle_result_with_generated_boundary(
            activity.state_hash(),
            result,
            &[program_id()],
            |_, _, _, streams| {
                reward(streams)?;
                Ok(vec![credit(10)])
            },
        )
        .unwrap();
}

#[test]
fn generated_settlement_invalid_program_rejection_and_fault_restore_everything() {
    let (mut activity, result) = start(reward_offer());
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    for operations in [
        vec![ActivityOperation::SetSlot {
            slot: slot(99),
            value: integer(1),
        }],
        reward_offer(),
        vec![ActivityOperation::Relocate(node(24))],
        vec![ActivityOperation::Conditional {
            condition: condition(false),
            if_true: reward_offer().into_boxed_slice(),
            if_false: vec![credit(10)].into_boxed_slice(),
        }],
        vec![ActivityOperation::Traverse(
            ActivityEdgeId::new(21).unwrap(),
        )],
        vec![ActivityOperation::Terminal(
            ActivityTerminalOutcome::Completed,
        )],
        vec![credit(10), ActivityOperation::Require(condition(false))],
        vec![credit(100)],
        vec![credit(0); MAX_ACTIVITY_PROGRAM_OPERATIONS + 1],
    ] {
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    activity.state_hash(),
                    result.clone(),
                    &[program_id()],
                    |_, _, _, streams| {
                        reward(streams)?;
                        Ok(operations.clone())
                    },
                )
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        assert_eq!(debug, activity.debug_view());
    }
}

#[test]
fn generated_settlement_destination_rejection_and_fault_roll_back_submission() {
    for destination in [
        vec![credit(1), ActivityOperation::Require(condition(false))],
        vec![credit(100)],
    ] {
        let (mut activity, result) = start(destination);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    activity.state_hash(),
                    result,
                    &[program_id()],
                    |_, _, _, streams| {
                        reward(streams)?;
                        Ok(vec![credit(10)])
                    },
                )
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        assert_eq!(debug, activity.debug_view());
    }
}

#[test]
fn empty_generated_settlement_preserves_ordinary_outcomes_without_random_draws() {
    for outcome in [
        BattleOutcome::Won,
        BattleOutcome::Lost,
        BattleOutcome::Faulted,
    ] {
        let mut destination = vec![credit(10)];
        destination.extend(reward_offer());
        let (mut ordinary, original) = start(destination.clone());
        let (mut generated, _) = start(destination);
        let result = with_outcome(&original, outcome);
        let expected = ordinary
            .submit_pending_battle_result(ordinary.state_hash(), result.clone())
            .unwrap();
        let actual = generated
            .submit_pending_battle_result_with_generated_boundary(
                generated.state_hash(),
                result,
                &[program_id()],
                |_, view, settlement, _| {
                    assert_eq!(settlement.outcome(), outcome);
                    assert_eq!(view.terminal(), settlement.terminal());
                    Ok(vec![])
                },
            )
            .unwrap();
        assert_eq!(expected.settlement(), actual.settlement());
        assert_eq!(expected.events(), actual.events());
        assert_eq!(expected.state_hash(), actual.state_hash());
        assert_eq!(
            ordinary.canonical_state_bytes(),
            generated.canonical_state_bytes()
        );
        assert_eq!(ordinary.debug_view().rng(), generated.debug_view().rng());
    }
}

#[test]
fn generated_settlement_terminal_extensions_commit_or_restore_pending_battle() {
    for outcome in [BattleOutcome::Lost, BattleOutcome::Faulted] {
        let (mut activity, original) = start(reward_offer());
        let result = with_outcome(&original, outcome);
        let before = activity.canonical_state_bytes();
        assert!(
            activity
                .submit_pending_battle_result_with_generated_boundary(
                    activity.state_hash(),
                    result.clone(),
                    &[program_id()],
                    |_, _, _, streams| {
                        reward(streams)?;
                        Ok(vec![credit(100)])
                    },
                )
                .is_err()
        );
        assert_eq!(before, activity.canonical_state_bytes());
        let resolution = activity
            .submit_pending_battle_result_with_generated_boundary(
                activity.state_hash(),
                result,
                &[program_id()],
                |_, view, settlement, streams| {
                    assert_eq!(settlement.outcome(), outcome);
                    assert_eq!(view.terminal(), settlement.terminal());
                    reward(streams)?;
                    Ok(vec![credit(10)])
                },
            )
            .unwrap();
        assert_eq!(
            amount(&activity.player_view()),
            &ActivityValue::BoundedInteger(27)
        );
        assert_eq!(
            activity.player_view().terminal(),
            resolution.settlement().terminal()
        );
        assert!(activity.player_view().decision().is_none());
    }
}

fn with_outcome(original: &BattleResult, outcome: BattleOutcome) -> BattleResult {
    let mut values = original.values().to_vec();
    for value in &mut values {
        match value {
            ProjectedValue::Outcome(value) => *value = outcome,
            ProjectedValue::TerminalFault(value) => {
                *value = (outcome == BattleOutcome::Faulted).then_some(BattleFault::from_parts(
                    FaultKind::BudgetExceeded,
                    FaultBoundary::Command,
                    FaultPolicy::Rollback,
                    1,
                    None,
                ));
            }
            _ => {}
        }
    }
    BattleResult::seal(original.identity(), values)
}
