//! Registered external results include automatic graph work in their transaction.

use std::sync::Arc;

use super::{
    SUCCESS_OUTCOME, definition, edge, external, node, option, program, section, slot, start,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityFault, ActivityGraphDefinition, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityProgramDefinition,
    ActivityRandomCheckpoint, ActivityRandomPolicies, ActivityRngLabel, ActivityTerminalOutcome,
    ActivityTransactionEventKind, ActivityTransactionRejection, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
    GraphActivityRuntimeError,
};

#[derive(Clone, Copy)]
enum Tail {
    Rejected,
    AmbiguousCheckpoint,
    Complete,
    Faulted,
}

#[test]
fn external_result_automatic_rejection_restores_handler_checkpoint_and_both_rng_streams() {
    for (tail, error) in [
        (
            Tail::Rejected,
            GraphActivityRuntimeError::Rejected(
                ActivityTransactionRejection::ConditionNotSatisfied,
            ),
        ),
        (
            Tail::AmbiguousCheckpoint,
            GraphActivityRuntimeError::InvalidRandomCheckpoint,
        ),
    ] {
        let mut activity = start(with_tail(tail), 101);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let hash = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        for _ in 0..3 {
            assert_eq!(
                activity.submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME)),
                Err(GraphActivityCommandError::Runtime(error))
            );
            assert_eq!(activity.state_hash(), hash, "automatic work must roll back");
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.debug_view(), debug);
            assert_eq!(activity.player_view().decision().unwrap().id(), decision);
        }
    }
}

#[test]
fn external_result_success_reconstructs_handler_checkpoint_and_terminal_events() {
    let mut left = start(with_tail(Tail::Complete), 102);
    let mut right = start(with_tail(Tail::Complete), 102);
    let before = left.canonical_state_bytes();
    let hash = left.state_hash();
    let decision = left.player_view().decision().unwrap().id();
    let events = left
        .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
        .unwrap();
    assert_eq!(
        events,
        right
            .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
            .unwrap()
    );
    assert_ne!(left.canonical_state_bytes(), before);
    assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_eq!(
        left.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_eq!(draws(&left, ActivityRngLabel::Occurrence), 1);
    assert_eq!(draws(&left, ActivityRngLabel::Graph), 1);
    let changed = events
        .iter()
        .filter(|event| matches!(event.kind(), ActivityTransactionEventKind::SlotChanged(_)))
        .map(|event| event.cause())
        .collect::<Vec<_>>();
    assert_eq!(changed.len(), 3);
    assert_eq!(changed[0].node(), node(1));
    assert_eq!(changed[0].option(), Some(option(SUCCESS_OUTCOME)));
    assert_eq!(changed[1].node(), node(2));
    assert!(matches!(changed[1].option().unwrap().get(), 101 | 102));
    assert_eq!(changed[2].node(), node(3));
    assert_eq!(changed[2].option(), None);
    assert_eq!(left.current_node(), node(4));
    assert_eq!(
        events.last().unwrap().kind(),
        &ActivityTransactionEventKind::EdgeTraversed(edge(3))
    );
}

#[test]
fn external_result_accepted_downstream_fault_commits_its_documented_fault_state() {
    let mut left = start(with_tail(Tail::Faulted), 103);
    let mut right = start(with_tail(Tail::Faulted), 103);
    let hash = left.state_hash();
    let decision = left.player_view().decision().unwrap().id();
    let events = left
        .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
        .unwrap();
    assert_eq!(
        events,
        right
            .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
            .unwrap()
    );
    assert_ne!(left.state_hash(), hash);
    assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_eq!(left.current_node(), node(3));
    assert_eq!(
        left.player_view().terminal(),
        Some(ActivityTerminalOutcome::Faulted)
    );
    assert_eq!(draws(&left, ActivityRngLabel::Occurrence), 1);
    assert_eq!(draws(&left, ActivityRngLabel::Graph), 1);
    assert!(matches!(
        events.last().unwrap().kind(),
        ActivityTransactionEventKind::Faulted(ActivityFault::SlotBounds(id)) if *id == slot(1)
    ));
    let before = left.canonical_state_bytes();
    assert_eq!(
        left.submit_external_outcome(left.state_hash(), decision, external(SUCCESS_OUTCOME)),
        Err(GraphActivityCommandError::DecisionNotOffered)
    );
    assert_eq!(left.canonical_state_bytes(), before);
}

fn with_tail(tail: Tail) -> Arc<GraphActivityDefinition> {
    let base = definition();
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            *base.graph().node(node(1)).unwrap(),
            ActivityNodeDefinition::new(node(2), section(1), ActivityNodeKind::Checkpoint, 1)
                .unwrap(),
            ActivityNodeDefinition::new(
                node(3),
                section(2),
                if matches!(tail, Tail::AmbiguousCheckpoint) {
                    ActivityNodeKind::Checkpoint
                } else {
                    ActivityNodeKind::Choice
                },
                1,
            )
            .unwrap(),
            ActivityNodeDefinition::new(
                node(4),
                section(2),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            base.graph().edges()[0],
            ActivityEdgeDefinition::new(
                edge(2),
                node(2),
                node(3),
                ActivityEdgeCondition::OptionSelected,
                0,
                1,
            )
            .unwrap(),
            ActivityEdgeDefinition::new(
                edge(3),
                node(3),
                node(4),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        4,
    )
    .unwrap();
    let checkpoint_options = [101, 102]
        .into_iter()
        .map(|id| {
            ActivityOptionDefinition::new(
                option(id),
                0,
                boolean(true),
                vec![
                    add(i64::try_from(id - 100).unwrap()),
                    ActivityOperation::Traverse(edge(2)),
                ],
            )
        })
        .collect::<Vec<_>>();
    let follow_up = match tail {
        Tail::Rejected => vec![
            add(1),
            ActivityOperation::Require(boolean(false)),
            ActivityOperation::Traverse(edge(3)),
        ],
        Tail::Complete => vec![add(1), ActivityOperation::Traverse(edge(3))],
        Tail::Faulted => vec![add(100), ActivityOperation::Traverse(edge(3))],
        Tail::AmbiguousCheckpoint => vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Checkpoint,
            options: [101, 102]
                .into_iter()
                .map(|id| {
                    ActivityOptionDefinition::new(
                        option(id),
                        0,
                        boolean(true),
                        vec![ActivityOperation::Traverse(edge(3))],
                    )
                })
                .collect(),
        }],
    };
    let mut programs = base.programs().to_vec();
    programs.extend([
        GraphActivityNodeProgram::new(
            node(2),
            ActivityProgramDefinition::new(
                program(2),
                vec![ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Checkpoint,
                    options: checkpoint_options.into_boxed_slice(),
                }],
            )
            .unwrap(),
        ),
        GraphActivityNodeProgram::new(
            node(3),
            ActivityProgramDefinition::new(program(3), follow_up).unwrap(),
        ),
    ]);
    let interactions = base.interactions().unwrap();
    Arc::new(
        GraphActivityDefinition::new(
            base.identity(),
            graph,
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            programs,
            None,
            ActivityRandomPolicies::new(
                vec![
                    ActivityRandomCheckpoint::new(
                        node(2),
                        ActivityRngLabel::Graph,
                        901,
                        vec![(option(101), 1), (option(102), 1)],
                    )
                    .unwrap(),
                ],
                vec![],
            ),
        )
        .unwrap()
        .with_interactions(
            (**interactions.registry()).clone(),
            interactions.bindings().to_vec(),
        )
        .unwrap(),
    )
}

fn boolean(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(value)))
}

fn add(value: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot: slot(1),
        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(value)),
    }
}

fn draws(activity: &GraphActivity, label: ActivityRngLabel) -> u64 {
    activity
        .debug_view()
        .rng()
        .iter()
        .find(|stream| stream.label() == label)
        .unwrap()
        .draw_count()
}
