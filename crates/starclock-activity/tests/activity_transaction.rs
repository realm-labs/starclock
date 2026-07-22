use starclock_activity::{
    ActivityCause, ActivityCondition, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityInventoryDefinition,
    ActivityInventoryId, ActivityModifierDefinition, ActivityModifierId, ActivityModifierOwner,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramBindingError, ActivityProgramDefinition,
    ActivityProgramDefinitionError, ActivityProgramId, ActivityScope, ActivitySlotDefinition,
    ActivitySlotId, ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility,
    ActivityTerminalOutcome, ActivityTransactionOutcome, ActivityTransactionRejection,
    ActivityTransactionState, ActivityValue, NodeId, SectionId, SlotCarryPolicy,
};

#[test]
fn ordered_program_commits_slots_inventory_modifiers_graph_and_decision_atomically() {
    let mut state = runtime();
    let program = ActivityProgramDefinition::new(
        program_id(1),
        vec![
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: integer(4),
            },
            ActivityOperation::AddCounter {
                slot: slot(2),
                key: 10,
                delta: integer(2),
            },
            ActivityOperation::AddInventory {
                inventory: inventory_id(1),
                content: 100,
                count: integer(1),
            },
            ActivityOperation::AddModifier {
                modifier: modifier_id(1),
                stacks: integer(1),
            },
            ActivityOperation::Traverse(edge_id(1)),
            ActivityOperation::Offer {
                kind: starclock_activity::ActivityDecisionKind::Choice,
                options: vec![ActivityOptionDefinition::new(
                    option_id(1),
                    0,
                    boolean(true),
                    vec![ActivityOperation::Terminal(
                        ActivityTerminalOutcome::Completed,
                    )],
                )]
                .into_boxed_slice(),
            },
        ],
    )
    .unwrap();
    let outcome = state.apply_program(&program, cause(1), &graph());
    assert!(matches!(outcome, ActivityTransactionOutcome::Committed(events) if events.len() == 6));
    assert_eq!(state.slot(slot(1)), Some(&ActivityValue::BoundedInteger(4)));
    assert_eq!(state.current_node(), node(2));
    assert_eq!(state.terminal(), None);
    let selected = state.apply_option(option_id(1), cause_at(2, node(2)), &graph());
    assert!(matches!(selected, ActivityTransactionOutcome::Committed(events) if events.len() == 1));
    assert_eq!(state.terminal(), Some(ActivityTerminalOutcome::Completed));
}

#[test]
fn failed_requirement_rejects_without_changing_any_state() {
    let mut state = runtime();
    let before = state.clone();
    let program = ActivityProgramDefinition::new(
        program_id(2),
        vec![
            ActivityOperation::Require(boolean(false)),
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: integer(5),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        state.apply_program(&program, cause(2), &graph()),
        ActivityTransactionOutcome::Rejected(ActivityTransactionRejection::ConditionNotSatisfied)
    );
    assert_eq!(state, before);
}

#[test]
fn internal_fault_discards_partial_work_and_commits_only_faulted_settlement() {
    let mut state = runtime();
    let program = ActivityProgramDefinition::new(
        program_id(3),
        vec![
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: integer(5),
            },
            ActivityOperation::AddToSlot {
                slot: slot(1),
                delta: integer(20),
            },
        ],
    )
    .unwrap();
    let outcome = state.apply_program(&program, cause(3), &graph());
    assert!(matches!(outcome, ActivityTransactionOutcome::Faulted(events, _) if events.len() == 1));
    assert_eq!(state.slot(slot(1)), Some(&ActivityValue::BoundedInteger(0)));
    assert_eq!(state.terminal(), Some(ActivityTerminalOutcome::Faulted));
}

#[test]
fn program_validation_rejects_unsorted_options_and_operations_after_boundary() {
    let options = vec![
        ActivityOptionDefinition::new(option_id(2), 0, boolean(true), vec![]),
        ActivityOptionDefinition::new(option_id(1), 0, boolean(true), vec![]),
    ]
    .into_boxed_slice();
    let error = ActivityProgramDefinition::new(
        program_id(4),
        vec![ActivityOperation::Offer {
            kind: starclock_activity::ActivityDecisionKind::Choice,
            options,
        }],
    )
    .unwrap_err();
    assert_eq!(error, ActivityProgramDefinitionError::NonCanonicalOptions);

    let error = ActivityProgramDefinition::new(
        program_id(5),
        vec![
            ActivityOperation::Terminal(ActivityTerminalOutcome::Completed),
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: integer(1),
            },
        ],
    )
    .unwrap_err();
    assert_eq!(
        error,
        ActivityProgramDefinitionError::OperationAfterBoundary(1)
    );

    let invalid = ActivityProgramDefinition::new(
        program_id(9),
        vec![ActivityOperation::Traverse(edge_id(99))],
    )
    .unwrap();
    assert_eq!(
        invalid
            .validate_against(&definition(), &graph())
            .unwrap_err(),
        ActivityProgramBindingError::MissingEdge(edge_id(99))
    );
}

#[test]
fn graph_visit_and_edge_budgets_are_authoritative_transaction_limits() {
    let graph = cycle_graph();
    let mut state = runtime();
    for (sequence, from, edge) in [(6, 1, 1), (7, 2, 2)] {
        let program = ActivityProgramDefinition::new(
            program_id(sequence),
            vec![ActivityOperation::Traverse(edge_id(edge))],
        )
        .unwrap();
        assert!(matches!(
            state.apply_program(&program, cause_at(u64::from(sequence), node(from)), &graph),
            ActivityTransactionOutcome::Committed(_)
        ));
    }
    assert_eq!(state.current_node(), node(1));
    assert_eq!(state.edge_traversals(edge_id(1)), 1);
    assert_eq!(state.node_visits(node(1)), 2);

    let before_slot = state.slot(slot(1)).cloned();
    let program = ActivityProgramDefinition::new(
        program_id(8),
        vec![
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: integer(7),
            },
            ActivityOperation::Traverse(edge_id(1)),
        ],
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause_at(8, node(1)), &graph),
        ActivityTransactionOutcome::Faulted(
            _,
            starclock_activity::ActivityFault::VisitLimitExceeded
        )
    ));
    assert_eq!(state.slot(slot(1)).cloned(), before_slot);
    assert_eq!(state.terminal(), Some(ActivityTerminalOutcome::Faulted));
}

fn runtime() -> ActivityTransactionState {
    ActivityTransactionState::new(definition(), node(1))
}

fn definition() -> ActivityStateDefinition {
    let slots = vec![
        ActivitySlotDefinition::new_with_policy(
            slot(1),
            ActivityScope::Activity,
            ActivityValue::BoundedInteger(0),
            Some((0, 10)),
            None,
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Private,
            source(1),
        )
        .unwrap(),
        ActivitySlotDefinition::new_with_policy(
            slot(2),
            ActivityScope::Activity,
            ActivityValue::BoundedCounterMap(Vec::new().into_boxed_slice()),
            Some((0, 10)),
            Some(4),
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Private,
            source(2),
        )
        .unwrap(),
    ];
    let inventory = ActivityInventoryDefinition::new(
        inventory_id(1),
        ActivityScope::Activity,
        4,
        5,
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Private,
        source(3),
    )
    .unwrap();
    let modifier = ActivityModifierDefinition::new(
        modifier_id(1),
        ActivityModifierOwner::Scope(ActivityScope::Activity),
        1,
        3,
        SlotCarryPolicy::CarryExact,
        source(4),
    )
    .unwrap();
    ActivityStateDefinition::new(slots, vec![inventory], vec![modifier]).unwrap()
}

fn graph() -> ActivityGraphDefinition {
    let nodes = vec![
        ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 2).unwrap(),
        ActivityNodeDefinition::new(node(2), section(1), ActivityNodeKind::Choice, 2).unwrap(),
        ActivityNodeDefinition::new(
            node(3),
            section(1),
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .unwrap(),
    ];
    let edges = vec![
        ActivityEdgeDefinition::new(
            edge_id(1),
            node(1),
            node(2),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
        ActivityEdgeDefinition::new(
            edge_id(2),
            node(2),
            node(3),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    ];
    ActivityGraphDefinition::new(node(1), nodes, edges, 5).unwrap()
}

fn cycle_graph() -> ActivityGraphDefinition {
    let nodes = vec![
        ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 2).unwrap(),
        ActivityNodeDefinition::new(node(2), section(1), ActivityNodeKind::Choice, 1).unwrap(),
        ActivityNodeDefinition::new(
            node(3),
            section(1),
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .unwrap(),
    ];
    let edges = vec![
        ActivityEdgeDefinition::new(
            edge_id(1),
            node(1),
            node(2),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
        ActivityEdgeDefinition::new(
            edge_id(2),
            node(2),
            node(1),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
        ActivityEdgeDefinition::new(
            edge_id(3),
            node(1),
            node(3),
            ActivityEdgeCondition::Always,
            1,
            1,
        )
        .unwrap(),
    ];
    ActivityGraphDefinition::new(node(1), nodes, edges, 4).unwrap()
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn boolean(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(value)))
}
fn cause(sequence: u64) -> ActivityCause {
    cause_at(sequence, node(1))
}
fn cause_at(sequence: u64, node: NodeId) -> ActivityCause {
    ActivityCause::new(sequence, program_id(sequence as u32), node).unwrap()
}
fn source(value: u64) -> ActivityStateSource {
    ActivityStateSource::new(value).unwrap()
}
fn slot(value: u32) -> ActivitySlotId {
    ActivitySlotId::new(value).unwrap()
}
fn inventory_id(value: u32) -> ActivityInventoryId {
    ActivityInventoryId::new(value).unwrap()
}
fn modifier_id(value: u32) -> ActivityModifierId {
    ActivityModifierId::new(value).unwrap()
}
fn program_id(value: u32) -> ActivityProgramId {
    ActivityProgramId::new(value).unwrap()
}
fn option_id(value: u64) -> ActivityOptionId {
    ActivityOptionId::new(value).unwrap()
}
fn edge_id(value: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(value).unwrap()
}
fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}
fn section(value: u32) -> SectionId {
    SectionId::new(value).unwrap()
}
