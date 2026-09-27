//! Authored external option IR needs no native registration or handler payload.

use super::{SUCCESS_OUTCOME, definition, external, node, option, program, slot, start};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityExpression, ActivityHandlerRegistry,
    ActivityInteractionBinding, ActivityInteractionBindingError, ActivityInteractionRandomPolicy,
    ActivityOperation, ActivityOptionDefinition, ActivityProgramDefinition, ActivityRandomPolicies,
    ActivityRngLabel, ActivityTerminalOutcome, ActivityValue, GraphActivityDefinition,
    GraphActivityDefinitionError, GraphActivityNodeProgram, core_activity_handler_bundle,
};
use std::sync::Arc;

fn authored(operations: Vec<ActivityOperation>) -> GraphActivityDefinition {
    let base = definition();
    GraphActivityDefinition::new(
        base.identity(),
        base.graph().clone(),
        base.state_definition().clone(),
        Arc::clone(base.participants()),
        vec![GraphActivityNodeProgram::new(
            node(1),
            ActivityProgramDefinition::new(
                program(1),
                vec![ActivityOperation::Offer {
                    kind: ActivityDecisionKind::ExternalOutcome,
                    options: vec![ActivityOptionDefinition::new(
                        option(SUCCESS_OUTCOME),
                        0,
                        ActivityCondition::Boolean(ActivityExpression::Literal(
                            ActivityValue::Boolean(true),
                        )),
                        operations,
                    )]
                    .into_boxed_slice(),
                }],
            )
            .unwrap(),
        )],
        None,
        ActivityRandomPolicies::default(),
    )
    .unwrap()
}
fn binding() -> ActivityInteractionBinding {
    ActivityInteractionBinding::authored_option(
        node(1),
        external(SUCCESS_OUTCOME),
        "test.authored-result",
    )
    .unwrap()
}
fn registry() -> ActivityHandlerRegistry {
    ActivityHandlerRegistry::compose(vec![core_activity_handler_bundle()]).unwrap()
}

#[test]
fn authored_external_ir_changes_state_with_zero_native_handlers_and_fresh_definition_parity() {
    let make = || {
        Arc::new(
            authored(vec![
                ActivityOperation::SetSlot {
                    slot: slot(1),
                    value: ActivityExpression::Literal(ActivityValue::BoundedInteger(7)),
                },
                ActivityOperation::Traverse(super::edge(1)),
            ])
            .with_interactions(registry(), vec![binding()])
            .unwrap(),
        )
    };
    let left_definition = make();
    let bindings = left_definition.interactions().unwrap();
    assert!(
        bindings
            .registry()
            .bundles()
            .iter()
            .all(|bundle| bundle.registrations().is_empty())
    );
    assert!(bindings.bindings()[0].handler().is_none());
    assert!(bindings.bindings()[0].payload().is_empty());
    let mut left = start(left_definition, 9);
    let mut right = start(make(), 9);
    let before = left.canonical_state_bytes();
    let hash = left.state_hash();
    let decision = left.player_view().decision().unwrap().id();
    assert!(
        left.choose_option(hash, decision, option(SUCCESS_OUTCOME))
            .is_err()
    );
    assert_eq!(left.canonical_state_bytes(), before);
    let events = left
        .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
        .unwrap();
    assert_eq!(
        right
            .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
            .unwrap(),
        events
    );
    assert_eq!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_eq!(left.debug_view(), right.debug_view());
    assert_eq!(
        left.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
    assert_ne!(left.canonical_state_bytes(), before);
}

#[test]
fn authored_external_ir_rejection_preserves_offer_state_and_rng() {
    let definition = Arc::new(
        authored(vec![
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(7)),
            },
            ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
                ActivityValue::Boolean(false),
            ))),
        ])
        .with_interactions(registry(), vec![binding()])
        .unwrap(),
    );
    let mut activity = start(definition, 9);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let debug = activity.debug_view();
    let decision = activity.player_view().decision().unwrap().id();
    for _ in 0..3 {
        assert!(
            activity
                .submit_external_outcome(hash, decision, external(SUCCESS_OUTCOME))
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.state_hash(), hash);
        assert_eq!(activity.debug_view(), debug);
    }
}

#[test]
fn authored_external_bindings_reject_rng_undeclared_results_and_duplicates() {
    let policy =
        ActivityInteractionRandomPolicy::new(ActivityRngLabel::Occurrence, 901, 7).unwrap();
    for (bindings, expected) in [
        (
            vec![binding().with_random_policy(policy)],
            ActivityInteractionBindingError::InvalidRandomPolicy,
        ),
        (
            vec![
                ActivityInteractionBinding::authored_option(
                    node(1),
                    external(999),
                    "test.authored-result",
                )
                .unwrap(),
            ],
            ActivityInteractionBindingError::OutcomeNotOffered,
        ),
        (
            vec![binding(), binding()],
            ActivityInteractionBindingError::DuplicateBinding,
        ),
    ] {
        let error = authored(Vec::new())
            .with_interactions(registry(), bindings)
            .unwrap_err();
        assert_eq!(
            error,
            GraphActivityDefinitionError::InvalidInteractionBindings(expected)
        );
    }
}
