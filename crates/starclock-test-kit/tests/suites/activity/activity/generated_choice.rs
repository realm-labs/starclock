//! Shared option transaction fixtures, not released occurrence parity claims.

#[path = "generated_choice_advance.rs"]
mod advance;

use std::{cell::Cell, sync::Arc};

use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId,
    ActivityExpression, ActivityOperation, ActivityOptionDefinition, ActivityProgramDefinition,
    ActivityRandomPolicies, ActivityRngLabel, ActivityRngStreams, ActivityStateHash,
    ActivityTerminalOutcome, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityDefinition, GraphActivityNodeProgram, MAX_ACTIVITY_PROGRAM_OPERATIONS,
};

use super::{always, definition, integer, node, option, program, slot, start, visible_counters};

fn reward(streams: &mut ActivityRngStreams) -> Result<u32, GraphActivityCommandError> {
    Ok(streams
        .choose_index(ActivityRngLabel::Reward, 301, 5)
        .map_err(GraphActivityCommandError::Rng)?
        .expect("nonempty reward pool")
        .value()
        .try_into()
        .expect("five candidates fit u32"))
}

fn credit(amount: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(1),
        key: 77,
        delta: integer(amount),
    }
}

fn decision(activity: &GraphActivity) -> ActivityDecisionId {
    activity
        .player_view()
        .decision()
        .expect("offered choice")
        .id()
}

#[test]
fn generated_choice_commits_reward_and_consumes_the_offered_option_once() {
    let mut left = start(definition(), 11);
    let mut right = start(definition(), 11);
    let initial = left.state_hash();
    let offered = decision(&left);
    let execute = |activity: &mut GraphActivity| {
        activity
            .choose_option_with_generated_prefix(initial, offered, option(100), |view, streams| {
                assert_eq!(view.decision().unwrap().id(), offered);
                let selected = reward(streams)?;
                Ok((vec![credit(i64::from(selected) + 1)], selected))
            })
            .expect("atomic generated choice")
    };
    let first = execute(&mut left);
    let replayed = execute(&mut right);
    assert_eq!(first, replayed);
    assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_eq!(
        visible_counters(&left),
        vec![(77, i64::from(*first.value()) + 1)]
    );
    assert!(left.player_view().decision().is_none());
    assert_ne!(initial, left.state_hash());
    let after = left.canonical_state_bytes();
    let hash = left.state_hash();
    assert!(
        left.choose_option_with_generated_prefix::<()>(hash, offered, option(100), |_, _| {
            panic!("consumed option cannot invoke generator")
        })
        .is_err()
    );
    assert_eq!(after, left.canonical_state_bytes());
}

#[test]
fn empty_generated_prefix_matches_ordinary_successful_choice() {
    let mut ordinary = start(definition(), 12);
    let mut generated = start(definition(), 12);
    let hash = ordinary.state_hash();
    let offered = decision(&ordinary);
    let events = ordinary.choose_option(hash, offered, option(100)).unwrap();
    let result = generated
        .choose_option_with_generated_prefix(hash, offered, option(100), |_, _| Ok((vec![], ())))
        .unwrap();
    assert_eq!(events.as_ref(), result.events());
    assert_eq!(
        ordinary.canonical_state_bytes(),
        generated.canonical_state_bytes()
    );
}

#[test]
fn invalid_choice_identity_rejects_before_generation() {
    let mut activity = start(definition(), 13);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let offered = decision(&activity);
    for (expected, selected_decision, selected_option) in [
        (
            ActivityStateHash::new([0xaa; 32]).unwrap(),
            offered,
            option(100),
        ),
        (
            hash,
            ActivityDecisionId::new(offered.get() + 1).unwrap(),
            option(100),
        ),
        (hash, offered, option(99)),
    ] {
        let called = Cell::new(false);
        let result = activity.choose_option_with_generated_prefix(
            expected,
            selected_decision,
            selected_option,
            |_, _| {
                called.set(true);
                Ok((vec![], ()))
            },
        );
        assert!(result.is_err());
        assert!(!called.get());
        assert_eq!(before, activity.canonical_state_bytes());
    }
}

#[test]
fn generator_errors_and_invalid_programs_restore_draws_and_pending_option() {
    let mut activity = start(definition(), 14);
    let hash = activity.state_hash();
    let before = activity.canonical_state_bytes();
    let offered = decision(&activity);
    let result = activity.choose_option_with_generated_prefix::<()>(
        hash,
        offered,
        option(100),
        |_, streams| {
            reward(streams)?;
            Err(GraphActivityCommandError::DecisionNotOffered)
        },
    );
    assert!(result.is_err());
    assert_eq!(before, activity.canonical_state_bytes());
    for operations in [
        vec![ActivityOperation::SetOrderedIdSet {
            slot: slot(1),
            values: Box::new([1]),
        }],
        vec![ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap())],
        vec![credit(1); MAX_ACTIVITY_PROGRAM_OPERATIONS - 1],
        vec![credit(1), credit(1_000)],
    ] {
        let result = activity.choose_option_with_generated_prefix(
            hash,
            offered,
            option(100),
            |_, streams| {
                reward(streams)?;
                Ok((operations, ()))
            },
        );
        assert!(result.is_err());
        assert_eq!(before, activity.canonical_state_bytes());
        assert_eq!(decision(&activity), offered);
    }
}

#[test]
fn late_authored_option_rejection_restores_generated_rewards_and_rng() {
    let base = definition();
    let rejected = ActivityCondition::Boolean(boolean(false));
    let definition = with_choice(
        &base,
        ActivityDecisionKind::Choice,
        vec![
            credit(-1),
            ActivityOperation::Require(rejected),
            ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap()),
        ],
    );
    let mut activity = start(definition, 15);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let offered = decision(&activity);
    let result =
        activity.choose_option_with_generated_prefix(hash, offered, option(100), |_, streams| {
            reward(streams)?;
            Ok((vec![credit(200)], ()))
        });
    assert!(result.is_err());
    assert_eq!(before, activity.canonical_state_bytes());
}

#[test]
fn external_outcome_cannot_use_a_generated_player_choice() {
    let base = definition();
    let definition = with_choice(
        &base,
        ActivityDecisionKind::ExternalOutcome,
        vec![ActivityOperation::Terminal(
            ActivityTerminalOutcome::Completed,
        )],
    );
    let mut activity = start(definition, 16);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let offered = decision(&activity);
    let result =
        activity.choose_option_with_generated_prefix::<()>(hash, offered, option(100), |_, _| {
            panic!("external outcome must use its registered handler")
        });
    assert_eq!(result, Err(GraphActivityCommandError::InteractionNotBound));
    assert_eq!(before, activity.canonical_state_bytes());
}

fn boolean(value: bool) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::Boolean(value))
}

fn with_choice(
    base: &GraphActivityDefinition,
    kind: ActivityDecisionKind,
    operations: Vec<ActivityOperation>,
) -> Arc<GraphActivityDefinition> {
    let choice = ActivityOptionDefinition::new(option(100), 0, always(), operations);
    let program = ActivityProgramDefinition::new(
        program(1),
        vec![ActivityOperation::Offer {
            kind,
            options: Box::new([choice]),
        }],
    )
    .unwrap();
    Arc::new(
        GraphActivityDefinition::new(
            base.identity(),
            base.graph().clone(),
            base.state_definition().clone(),
            Arc::clone(base.participants()),
            vec![GraphActivityNodeProgram::new(node(1), program)],
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    )
}
