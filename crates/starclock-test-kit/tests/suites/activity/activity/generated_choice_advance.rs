//! Ordered authored/generated prefixes and automatic graph failure rollback.

use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityGraphDefinition, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityProgramDefinition, ActivityRandomOffer,
    ActivityRandomPolicies, ActivityRngLabel, ActivityTerminalOutcome,
    ActivityTransactionEventKind, GraphActivityDefinition, GraphActivityNodeProgram,
};

use super::{boolean, credit, decision, reward, with_choice};
use crate::activity_random_boundary::{
    always, definition, integer, node, option, program, section, slot, start, visible_counters,
};

#[test]
fn authored_prefix_then_generated_reward_then_choice_share_the_option_cause() {
    let base = definition();
    let base = with_choice(
        &base,
        ActivityDecisionKind::Choice,
        vec![
            ActivityOperation::AddCounter {
                slot: slot(1),
                key: 79,
                delta: integer(1),
            },
            ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap()),
        ],
    );
    let policy = ActivityRandomOffer::new(
        node(1),
        ActivityRngLabel::Occurrence,
        302,
        1,
        vec![(option(100), 1)],
        None,
    )
    .unwrap()
    .with_selection_prefix(vec![credit(100)])
    .unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            base.identity(),
            base.graph().clone(),
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            base.programs().to_vec(),
            None,
            ActivityRandomPolicies::new(vec![], vec![policy]),
        )
        .unwrap(),
    );
    let mut activity = start(definition, 21);
    let hash = activity.state_hash();
    let offered = decision(&activity);
    let result = activity
        .choose_option_with_generated_prefix(hash, offered, option(100), |_, streams| {
            reward(streams)?;
            Ok((
                vec![
                    credit(-100),
                    ActivityOperation::AddCounter {
                        slot: slot(1),
                        key: 78,
                        delta: integer(1),
                    },
                ],
                (),
            ))
        })
        .unwrap();
    assert_eq!(visible_counters(&activity), vec![(77, 0), (78, 1), (79, 1)]);
    let changed = result
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            ActivityTransactionEventKind::CounterChanged { key, .. } => Some((*key, event.cause())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        changed.iter().map(|(key, _)| *key).collect::<Vec<_>>(),
        vec![77, 77, 78, 79]
    );
    for (_, cause) in &changed {
        assert_eq!(cause.option(), Some(option(100)));
        assert_eq!(*cause, changed[0].1);
    }
}

#[test]
fn automatic_node_rejection_or_fault_restores_choice_rewards_and_random_streams() {
    for follow_up in [
        vec![
            credit(1),
            ActivityOperation::Require(ActivityCondition::Boolean(boolean(false))),
        ],
        vec![credit(i64::MAX)],
    ] {
        let mut activity = start(with_follow_up(follow_up), 22);
        let hash = activity.state_hash();
        let before = activity.canonical_state_bytes();
        let offered = decision(&activity);
        let result = activity.choose_option_with_generated_prefix(
            hash,
            offered,
            option(100),
            |_, streams| {
                reward(streams)?;
                Ok((vec![credit(200)], ()))
            },
        );
        assert!(result.is_err());
        assert_eq!(before, activity.canonical_state_bytes());
        assert_eq!(decision(&activity), offered);
    }
}

fn with_follow_up(mut follow_up: Vec<ActivityOperation>) -> Arc<GraphActivityDefinition> {
    let base = definition();
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 1).unwrap(),
            ActivityNodeDefinition::new(node(2), section(1), ActivityNodeKind::Choice, 1).unwrap(),
            ActivityNodeDefinition::new(
                node(3),
                section(1),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(1).unwrap(),
                node(1),
                node(2),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(2).unwrap(),
                node(2),
                node(3),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        3,
    )
    .unwrap();
    let choice = ActivityOptionDefinition::new(
        option(100),
        0,
        always(),
        vec![ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap())],
    );
    let offer = ActivityProgramDefinition::new(
        program(1),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Choice,
            options: Box::new([choice]),
        }],
    )
    .unwrap();
    follow_up.push(ActivityOperation::Traverse(ActivityEdgeId::new(2).unwrap()));
    let automatic = ActivityProgramDefinition::new(program(2), follow_up).unwrap();
    Arc::new(
        GraphActivityDefinition::new(
            base.identity(),
            graph,
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            vec![
                GraphActivityNodeProgram::new(node(1), offer),
                GraphActivityNodeProgram::new(node(2), automatic),
            ],
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    )
}
