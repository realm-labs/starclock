//! Generic finite-program counter deletion, distinct from a stored zero.

use crate::{
    ActivityCause, ActivityCondition, ActivityConfigDigest, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityFault,
    ActivityGraphDefinition, ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityProgramBindingError, ActivityProgramDefinition,
    ActivityProgramDefinitionError, ActivityProgramId, ActivityRngContext, ActivityRngLabel,
    ActivityRngStreams, ActivityScope, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityTransactionEventKind, ActivityTransactionOutcome, ActivityTransactionRejection,
    ActivityTransactionState, ActivityValue, NodeId, SectionId, SlotCarryPolicy,
};

fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn node() -> NodeId {
    NodeId::new(1).unwrap()
}
fn integer(raw: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(raw))
}
fn definition() -> ActivityStateDefinition {
    let values = [
        ActivityValue::BoundedCounterMap(vec![(10, i64::MIN), (20, 0), (30, i64::MAX)].into()),
        ActivityValue::BoundedInteger(0),
    ];
    let slots = values
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            let raw = u32::try_from(index + 1).unwrap();
            ActivitySlotDefinition::new_with_policy(
                slot(raw),
                ActivityScope::Activity,
                value,
                Some(if raw == 1 {
                    (i64::MIN, i64::MAX)
                } else {
                    (0, 10)
                }),
                (raw == 1).then_some(3),
                Vec::new(),
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(u64::from(raw)).unwrap(),
            )
            .unwrap()
        })
        .collect();
    ActivityStateDefinition::new(slots, Vec::new(), Vec::new()).unwrap()
}
fn graph() -> ActivityGraphDefinition {
    let terminal = NodeId::new(2).unwrap();
    let section = SectionId::new(1).unwrap();
    ActivityGraphDefinition::new(
        node(),
        vec![
            ActivityNodeDefinition::new(node(), section, ActivityNodeKind::Choice, 1).unwrap(),
            ActivityNodeDefinition::new(
                terminal,
                section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(1).unwrap(),
                node(),
                terminal,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        2,
    )
    .unwrap()
}
fn program(raw: u32, operations: Vec<ActivityOperation>) -> ActivityProgramDefinition {
    ActivityProgramDefinition::new(ActivityProgramId::new(raw).unwrap(), operations).unwrap()
}
fn cause(raw: u32) -> ActivityCause {
    ActivityCause::new(u64::from(raw), ActivityProgramId::new(raw).unwrap(), node()).unwrap()
}
fn remove(key: u64) -> ActivityOperation {
    ActivityOperation::RemoveCounter { slot: slot(1), key }
}

fn encoded(state: &ActivityTransactionState) -> Box<[u8]> {
    let graph = graph();
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([1; 32]).unwrap(),
        ActivityConfigDigest::new([2; 32]).unwrap(),
    );
    let instance = ActivityInstanceId::new(3).unwrap();
    let mut rng = ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(4),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        graph.digest(),
        instance,
        None,
        None,
        None,
        0,
    ));
    rng.choose_index(ActivityRngLabel::Reward, 1, 3).unwrap();
    state.canonical_state_bytes(identity, &graph, instance, &rng)
}

#[test]
fn counter_removal_preserves_order_and_handles_zero_and_integer_extremes() {
    for key in [10, 20, 30, 40] {
        let mut state = ActivityTransactionState::new(definition(), node());
        let operation = program(1, vec![remove(key)]);
        operation.validate_against(&definition(), &graph()).unwrap();
        let ActivityTransactionOutcome::Committed(events) =
            state.apply_program(&operation, cause(1), &graph())
        else {
            panic!("validated counter removal must commit");
        };
        let expected = [(10, i64::MIN), (20, 0), (30, i64::MAX)]
            .into_iter()
            .filter(|(candidate, _)| *candidate != key)
            .collect::<Vec<_>>();
        assert_eq!(
            state.slot(slot(1)),
            Some(&ActivityValue::BoundedCounterMap(expected.into()))
        );
        if key == 40 {
            assert!(events.is_empty());
        } else {
            assert_eq!(events.len(), 1);
            assert_eq!(
                events[0].kind(),
                &ActivityTransactionEventKind::CounterChanged { slot: slot(1), key }
            );
            assert_eq!(events[0].cause(), cause(1));
        }
    }
}

#[test]
fn counter_removal_frees_capacity_and_repeated_absence_is_an_eventless_noop() {
    let mut state = ActivityTransactionState::new(definition(), node());
    let operations = program(
        1,
        vec![
            remove(20),
            ActivityOperation::SetCounter {
                slot: slot(1),
                key: 40,
                value: integer(-1),
            },
        ],
    );
    assert!(
        matches!(state.apply_program(&operations, cause(1), &graph()),
        ActivityTransactionOutcome::Committed(events) if events.len() == 2)
    );
    assert_eq!(
        state.slot(slot(1)),
        Some(&ActivityValue::BoundedCounterMap(
            vec![(10, i64::MIN), (30, i64::MAX), (40, -1)].into()
        ))
    );
    let before = state.slot(slot(1)).cloned();
    assert!(
        matches!(state.apply_program(&program(2, vec![remove(20)]), cause(2), &graph()),
        ActivityTransactionOutcome::Committed(events) if events.is_empty())
    );
    assert_eq!(state.slot(slot(1)).cloned(), before);
    assert_eq!(state.command_sequence(), 2);
}

#[test]
fn counter_removal_does_not_change_zero_value_set_semantics() {
    let mut state = ActivityTransactionState::new(definition(), node());
    assert!(matches!(
        state.apply_program(
            &program(
                1,
                vec![ActivityOperation::SetCounter {
                    slot: slot(1),
                    key: 10,
                    value: integer(0),
                }]
            ),
            cause(1),
            &graph()
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
    assert_eq!(
        state.slot(slot(1)),
        Some(&ActivityValue::BoundedCounterMap(
            vec![(10, 0), (20, 0), (30, i64::MAX)].into()
        ))
    );
    assert!(
        matches!(state.apply_program(&program(2, vec![remove(10), remove(20), remove(30)]),
        cause(2), &graph()), ActivityTransactionOutcome::Committed(events) if events.len() == 3)
    );
    assert_eq!(
        state.slot(slot(1)),
        Some(&ActivityValue::BoundedCounterMap(Box::new([])))
    );
}

#[test]
fn counter_removal_validates_nonzero_keys_and_declared_counter_slots() {
    assert_eq!(
        ActivityProgramDefinition::new(ActivityProgramId::new(1).unwrap(), vec![remove(0)]),
        Err(ActivityProgramDefinitionError::InvalidStableId)
    );
    for (target, expected) in [
        (slot(2), ActivityProgramBindingError::TypeMismatch(slot(2))),
        (slot(3), ActivityProgramBindingError::MissingSlot(slot(3))),
    ] {
        let operations = program(
            1,
            vec![ActivityOperation::RemoveCounter {
                slot: target,
                key: 10,
            }],
        );
        assert_eq!(
            operations.validate_against(&definition(), &graph()),
            Err(expected)
        );
    }
}

#[test]
fn counter_removal_and_later_rejection_restore_canonical_state() {
    let mut state = ActivityTransactionState::new(definition(), node());
    let before = encoded(&state);
    let operations = program(
        1,
        vec![
            remove(10),
            remove(20),
            ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
                ActivityValue::Boolean(false),
            ))),
        ],
    );
    for _ in 0..2 {
        assert_eq!(
            state.apply_program(&operations, cause(1), &graph()),
            ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::ConditionNotSatisfied
            )
        );
        assert_eq!(encoded(&state), before);
    }
}

#[test]
fn counter_removal_and_later_overflow_restore_values_before_a_deterministic_fault() {
    let mut state = ActivityTransactionState::new(definition(), node());
    let before = state.slot(slot(1)).cloned();
    let operations = program(
        1,
        vec![
            remove(10),
            ActivityOperation::AddCounter {
                slot: slot(1),
                key: 30,
                delta: integer(1),
            },
        ],
    );
    let ActivityTransactionOutcome::Faulted(events, fault) =
        state.apply_program(&operations, cause(1), &graph())
    else {
        panic!("overflow must enter the documented fault state");
    };
    assert_eq!(fault, ActivityFault::ArithmeticOverflow);
    assert_eq!(state.slot(slot(1)).cloned(), before);
    assert_eq!(state.terminal(), Some(ActivityTerminalOutcome::Faulted));
    assert_eq!(events.len(), 1);
    assert_eq!(
        events[0].kind(),
        &ActivityTransactionEventKind::Faulted(fault)
    );
}

#[test]
fn counter_removal_executes_only_the_selected_conditional_branch() {
    for selected in [false, true] {
        let mut state = ActivityTransactionState::new(definition(), node());
        let operations = program(
            1,
            vec![ActivityOperation::Conditional {
                condition: ActivityCondition::Boolean(ActivityExpression::Literal(
                    ActivityValue::Boolean(selected),
                )),
                if_true: vec![remove(10)].into(),
                if_false: vec![remove(30)].into(),
            }],
        );
        operations
            .validate_against(&definition(), &graph())
            .unwrap();
        assert!(
            matches!(state.apply_program(&operations, cause(1), &graph()),
            ActivityTransactionOutcome::Committed(events) if events.len() == 1)
        );
        let expected = if selected {
            vec![(20, 0), (30, i64::MAX)]
        } else {
            vec![(10, i64::MIN), (20, 0)]
        };
        assert_eq!(
            state.slot(slot(1)),
            Some(&ActivityValue::BoundedCounterMap(expected.into()))
        );
    }
}
