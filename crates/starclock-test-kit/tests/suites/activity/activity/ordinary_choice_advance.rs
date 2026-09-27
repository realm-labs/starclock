//! Ordinary choices include all automatic work without changing fault semantics.

use super::{
    always, definition, integer, node, option, program, section, slot, start, visible_counters,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityFault,
    ActivityGraphDefinition, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityProgramDefinition, ActivityRandomCheckpoint,
    ActivityRandomOffer, ActivityRandomPolicies, ActivityRngLabel, ActivityStateHash,
    ActivityTerminalOutcome, ActivityTransactionEventKind, ActivityTransactionRejection,
    ActivityValue, GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
    GraphActivityNodeProgram, GraphActivityRuntimeError,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
enum Tail {
    Rejected,
    AmbiguousCheckpoint,
    EmptyRandomOffer,
    Complete,
    Faulted,
    OptionFaulted,
    OptionRejected,
}

#[test]
fn ordinary_choice_returned_automatic_errors_restore_selected_prefix_checkpoint_and_rng() {
    for (tail, expected) in [
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
        (
            Tail::EmptyRandomOffer,
            GraphActivityRuntimeError::InvalidRandomOffer,
        ),
    ] {
        let mut activity = start(with_tail(tail), 301);
        let before = activity.canonical_state_bytes();
        let debug = activity.debug_view();
        let hash = activity.state_hash();
        let decision = activity.player_view().decision().unwrap().id();
        for _ in 0..3 {
            assert_eq!(
                activity.choose_option(hash, decision, option(100)),
                Err(GraphActivityCommandError::Runtime(expected))
            );
            assert_eq!(activity.state_hash(), hash);
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.debug_view(), debug);
            assert_eq!(activity.player_view().decision().unwrap().id(), decision);
        }
    }
}

#[test]
fn ordinary_choice_success_reconstructs_authored_prefix_checkpoint_events_and_terminal() {
    let mut left = start(with_tail(Tail::Complete), 302);
    let mut right = start(with_tail(Tail::Complete), 302);
    let hash = left.state_hash();
    let decision = left.player_view().decision().unwrap().id();
    let events = left.choose_option(hash, decision, option(100)).unwrap();
    assert_eq!(
        events,
        right.choose_option(hash, decision, option(100)).unwrap()
    );
    assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_eq!(left.debug_view(), right.debug_view());
    assert_eq!(
        left.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_eq!(draws(&left, ActivityRngLabel::Occurrence), 1);
    assert_eq!(draws(&left, ActivityRngLabel::Graph), 1);
    let counters = visible_counters(&left);
    assert_eq!(counters[0], (77, 5));
    assert_eq!(counters[1], (78, 7));
    assert!(matches!(counters[2], (79, 1 | 2)));
    assert_eq!(counters[3], (80, 3));
    let changed = events
        .iter()
        .filter(|event| {
            matches!(
                event.kind(),
                ActivityTransactionEventKind::CounterChanged { .. }
            )
        })
        .map(|event| event.cause())
        .collect::<Vec<_>>();
    assert_eq!(changed.len(), 4);
    assert_eq!(changed[0], changed[1]);
    assert_eq!(changed[0].node(), node(1));
    assert_eq!(changed[0].option(), Some(option(100)));
    assert_eq!(changed[2].node(), node(2));
    assert!(matches!(changed[2].option().unwrap().get(), 101 | 102));
    assert_eq!(changed[3].node(), node(3));
    assert_eq!(changed[3].option(), None);
    assert_eq!(
        events.last().unwrap().kind(),
        &ActivityTransactionEventKind::EdgeTraversed(edge(3))
    );
    let bytes = left.canonical_state_bytes();
    let debug = left.debug_view();
    assert_eq!(
        left.choose_option(left.state_hash(), decision, option(100)),
        Err(GraphActivityCommandError::DecisionNotOffered)
    );
    assert_eq!(left.canonical_state_bytes(), bytes);
    assert_eq!(left.debug_view(), debug);
}

#[test]
fn ordinary_choice_accepted_option_or_downstream_fault_commits_existing_fault_semantics() {
    for (tail, fault_node, graph_draws) in [
        (Tail::OptionFaulted, node(1), 0),
        (Tail::Faulted, node(3), 1),
    ] {
        let mut left = start(with_tail(tail), 303);
        let mut right = start(with_tail(tail), 303);
        let hash = left.state_hash();
        let decision = left.player_view().decision().unwrap().id();
        let events = left.choose_option(hash, decision, option(100)).unwrap();
        assert_eq!(
            events,
            right.choose_option(hash, decision, option(100)).unwrap()
        );
        assert_ne!(left.state_hash(), hash);
        assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
        assert_eq!(left.current_node(), fault_node);
        assert_eq!(
            left.player_view().terminal(),
            Some(ActivityTerminalOutcome::Faulted)
        );
        assert_eq!(draws(&left, ActivityRngLabel::Occurrence), 1);
        assert_eq!(draws(&left, ActivityRngLabel::Graph), graph_draws);
        assert_eq!(
            events.last().unwrap().kind(),
            &ActivityTransactionEventKind::Faulted(ActivityFault::SlotBounds(slot(1)))
        );
        let before = left.canonical_state_bytes();
        assert_eq!(
            left.choose_option(left.state_hash(), decision, option(100)),
            Err(GraphActivityCommandError::DecisionNotOffered)
        );
        assert_eq!(left.canonical_state_bytes(), before);
    }
}

#[test]
fn ordinary_choice_invalid_and_rejected_options_preserve_original_offer_prefix_and_rng() {
    let mut activity = start(with_tail(Tail::OptionRejected), 304);
    let before = activity.canonical_state_bytes();
    let debug = activity.debug_view();
    let hash = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    for (expected_hash, selected_decision, selected_option, error) in [
        (
            ActivityStateHash::new([3; 32]).unwrap(),
            decision,
            option(100),
            GraphActivityCommandError::StaleStateHash,
        ),
        (
            hash,
            ActivityDecisionId::new(u64::MAX).unwrap(),
            option(100),
            GraphActivityCommandError::DecisionNotOffered,
        ),
        (
            hash,
            decision,
            option(999),
            GraphActivityCommandError::Rejected(ActivityTransactionRejection::UnknownOption),
        ),
        (
            hash,
            decision,
            option(100),
            GraphActivityCommandError::Rejected(
                ActivityTransactionRejection::ConditionNotSatisfied,
            ),
        ),
    ] {
        for _ in 0..3 {
            assert_eq!(
                activity.choose_option(expected_hash, selected_decision, selected_option),
                Err(error)
            );
            assert_eq!(activity.canonical_state_bytes(), before);
            assert_eq!(activity.state_hash(), hash);
            assert_eq!(activity.debug_view(), debug);
        }
    }
}

fn with_tail(tail: Tail) -> Arc<GraphActivityDefinition> {
    let base = definition();
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 1).unwrap(),
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
        (1..=3)
            .map(|id| {
                ActivityEdgeDefinition::new(
                    edge(id),
                    node(id),
                    node(id + 1),
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap()
            })
            .collect(),
        4,
    )
    .unwrap();
    let mut choice_ops = vec![add(
        78,
        if matches!(tail, Tail::OptionFaulted) {
            2000
        } else {
            7
        },
    )];
    if matches!(tail, Tail::OptionRejected) {
        choice_ops.push(ActivityOperation::Require(boolean(false)));
    }
    choice_ops.push(ActivityOperation::Traverse(edge(1)));
    let first = ActivityOperation::Offer {
        kind: ActivityDecisionKind::Choice,
        options: vec![ActivityOptionDefinition::new(
            option(100),
            0,
            always(),
            choice_ops,
        )]
        .into_boxed_slice(),
    };
    let checkpoint = ActivityOperation::Offer {
        kind: ActivityDecisionKind::Checkpoint,
        options: [101, 102]
            .into_iter()
            .map(|id| {
                ActivityOptionDefinition::new(
                    option(id),
                    0,
                    always(),
                    vec![
                        add(79, i64::try_from(id - 100).unwrap()),
                        ActivityOperation::Traverse(edge(2)),
                    ],
                )
            })
            .collect(),
    };
    let follow_up = match tail {
        Tail::Rejected => vec![
            add(80, 3),
            ActivityOperation::Require(boolean(false)),
            ActivityOperation::Traverse(edge(3)),
        ],
        Tail::Faulted => vec![add(80, 2000), ActivityOperation::Traverse(edge(3))],
        Tail::AmbiguousCheckpoint => vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Checkpoint,
            options: [101, 102]
                .into_iter()
                .map(|id| {
                    ActivityOptionDefinition::new(
                        option(id),
                        0,
                        always(),
                        vec![ActivityOperation::Traverse(edge(3))],
                    )
                })
                .collect(),
        }],
        Tail::Complete | Tail::OptionFaulted | Tail::OptionRejected => {
            vec![add(80, 3), ActivityOperation::Traverse(edge(3))]
        }
        Tail::EmptyRandomOffer => vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Choice,
            options: [201, 202]
                .into_iter()
                .map(|id| {
                    ActivityOptionDefinition::new(
                        option(id),
                        0,
                        boolean(id == 201),
                        vec![ActivityOperation::Traverse(edge(3))],
                    )
                })
                .collect(),
        }],
    };
    let initial_offer = ActivityRandomOffer::new(
        node(1),
        ActivityRngLabel::Occurrence,
        801,
        1,
        vec![(option(100), 1)],
        None,
    )
    .unwrap()
    .with_selection_prefix(vec![add(77, 5)])
    .unwrap();
    let mut offers = vec![initial_offer];
    if matches!(tail, Tail::EmptyRandomOffer) {
        offers.push(
            ActivityRandomOffer::new(
                node(3),
                ActivityRngLabel::Reward,
                803,
                1,
                vec![(option(201), 1), (option(202), 1)],
                None,
            )
            .unwrap()
            .with_conditional_candidate_filter(always(), vec![option(202)])
            .unwrap(),
        );
    }
    Arc::new(
        GraphActivityDefinition::new(
            base.identity(),
            graph,
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            vec![
                record(1, vec![first]),
                record(2, vec![checkpoint]),
                record(3, follow_up),
            ],
            None,
            ActivityRandomPolicies::new(
                vec![
                    ActivityRandomCheckpoint::new(
                        node(2),
                        ActivityRngLabel::Graph,
                        802,
                        vec![(option(101), 1), (option(102), 1)],
                    )
                    .unwrap(),
                ],
                offers,
            ),
        )
        .unwrap(),
    )
}
fn record(id: u32, operations: Vec<ActivityOperation>) -> GraphActivityNodeProgram {
    GraphActivityNodeProgram::new(
        node(id),
        ActivityProgramDefinition::new(program(id), operations).unwrap(),
    )
}
fn edge(id: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(id).unwrap()
}
fn boolean(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(value)))
}
fn add(key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(1),
        key,
        delta: integer(value),
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
