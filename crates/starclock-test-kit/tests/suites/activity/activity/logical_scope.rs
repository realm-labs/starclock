use std::sync::Arc;

use starclock_activity::{
    ActivityCause, ActivityCondition, ActivityConfigDigest, ActivityDecisionKind,
    ActivityDefinitionDigest, ActivityDefinitionId, ActivityDefinitionIdentity,
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression,
    ActivityGraphDefinition, ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityTerminalOutcome, ActivityTransactionEvent, ActivityTransactionEventKind,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue, BuildDigest,
    GraphActivity, GraphActivityDefinition, GraphActivityDefinitionError, GraphActivityNodeProgram,
    LoadoutLockScope, LogicalScopeAddress, LogicalScopeClassDefinition, LogicalScopeClassId,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, OpaqueParticipantBuild,
    ParticipantId, ParticipantLock, ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind,
    ParticipantUniquenessScope, SectionId, SlotDefinitionError, SlotResetPoint,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

#[test]
fn physical_nodes_share_one_logical_visit_and_reentry_is_fresh() {
    let class = LogicalScopeClassId::new(1).unwrap();
    let address = LogicalScopeAddress::new(class, 100).unwrap();
    let logical = LogicalScopeDefinitions::new(
        vec![LogicalScopeClassDefinition::new(class, None, 4).unwrap()],
        vec![
            LogicalScopeNodeBinding::new(node(1), vec![address]).unwrap(),
            LogicalScopeNodeBinding::new(node(2), vec![address]).unwrap(),
        ],
    )
    .unwrap();
    let definition = ActivityStateDefinition::new(vec![], vec![], vec![])
        .unwrap()
        .with_logical_scopes(logical);
    let graph = graph();
    let mut state = ActivityTransactionState::new(definition, node(1));

    assert_eq!(state.active_logical_scopes()[0].visit_sequence(), 1);
    apply_traverse(&mut state, &graph, 1, 1);
    assert_eq!(state.active_logical_scopes()[0].visit_sequence(), 1);

    apply_traverse(&mut state, &graph, 2, 2);
    assert!(state.active_logical_scopes().is_empty());

    apply_traverse(&mut state, &graph, 3, 3);
    assert_eq!(state.active_logical_scopes()[0].visit_sequence(), 2);
}

#[test]
fn logical_slots_preserve_internal_traversal_but_reset_on_exit_and_reentry() {
    let class = LogicalScopeClassId::new(1).unwrap();
    let logical = LogicalScopeDefinitions::new(
        vec![LogicalScopeClassDefinition::new(class, None, 4).unwrap()],
        [1, 2]
            .into_iter()
            .map(|raw| LogicalScopeNodeBinding::new(node(raw), vec![address(1, 100)]).unwrap())
            .collect(),
    )
    .unwrap();
    let definition = ActivityStateDefinition::new(
        vec![boolean_slot(1), boolean_slot(2).with_logical_scope(class)],
        vec![],
        vec![],
    )
    .unwrap()
    .with_logical_scopes(logical);
    let mut state = ActivityTransactionState::new(definition, node(1));
    let graph = graph();
    let events = apply(
        &mut state,
        &graph,
        1,
        vec![set_true(1), set_true(2), traverse(1)],
    );
    assert_eq!(state.slot(slot(1)), Some(&ActivityValue::Boolean(false)));
    assert_eq!(state.slot(slot(2)), Some(&ActivityValue::Boolean(true)));
    assert_eq!(resets(&events), vec![(slot(1), SlotResetPoint::NodeStart)]);

    let events = apply(&mut state, &graph, 2, vec![traverse(2)]);
    assert_eq!(state.slot(slot(2)), Some(&ActivityValue::Boolean(false)));
    assert_eq!(
        resets(&events),
        vec![
            (slot(1), SlotResetPoint::NodeStart),
            (slot(2), SlotResetPoint::LogicalScopeChanged),
        ]
    );
    let events = apply(&mut state, &graph, 3, vec![set_true(2), traverse(3)]);
    assert_eq!(state.slot(slot(2)), Some(&ActivityValue::Boolean(false)));
    assert_eq!(state.active_logical_scopes()[0].visit_sequence(), 2);
    assert_eq!(
        resets(&events).last(),
        Some(&(slot(2), SlotResetPoint::LogicalScopeChanged))
    );
}

#[test]
fn child_replacement_and_parent_change_reset_only_the_changed_logical_instances() {
    let plane = LogicalScopeClassId::new(1).unwrap();
    let room = LogicalScopeClassId::new(2).unwrap();
    let scopes = LogicalScopeDefinitions::new(
        vec![
            LogicalScopeClassDefinition::new(plane, None, 2).unwrap(),
            LogicalScopeClassDefinition::new(room, Some(plane), 3).unwrap(),
        ],
        [(1, 1, 1), (2, 1, 1), (3, 1, 2), (4, 2, 2)]
            .into_iter()
            .map(|(raw, parent, child)| {
                LogicalScopeNodeBinding::new(node(raw), vec![address(1, parent), address(2, child)])
                    .unwrap()
            })
            .collect(),
    )
    .unwrap();
    // Reverse authoring order must not change reset-event ordering.
    let definition = ActivityStateDefinition::new(
        vec![
            boolean_slot(3).with_logical_scope(room),
            boolean_slot(2).with_logical_scope(plane),
            ActivitySlotDefinition::new(
                slot(1),
                ActivityScope::Section,
                ActivityValue::Boolean(false),
                None,
                vec![SlotResetPoint::SectionStart],
            )
            .unwrap(),
        ],
        vec![],
        vec![],
    )
    .unwrap()
    .with_logical_scopes(scopes);
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            activity_node(1, ActivityNodeKind::Choice, 1),
            activity_node(2, ActivityNodeKind::Choice, 1),
            activity_node(3, ActivityNodeKind::Choice, 1),
            activity_node(
                4,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            ),
        ],
        vec![edge(1, 1, 2, 1), edge(2, 2, 3, 1), edge(3, 3, 4, 1)],
        4,
    )
    .unwrap();
    let mut state = ActivityTransactionState::new(definition, node(1));
    assert!(
        resets(&apply(
            &mut state,
            &graph,
            1,
            vec![set_true(1), set_true(2), set_true(3), traverse(1)]
        ))
        .is_empty()
    );
    let events = apply(&mut state, &graph, 2, vec![traverse(2)]);
    assert_eq!(state.slot(slot(2)), Some(&ActivityValue::Boolean(true)));
    assert_eq!(state.slot(slot(3)), Some(&ActivityValue::Boolean(false)));
    assert_eq!(
        resets(&events),
        vec![(slot(3), SlotResetPoint::LogicalScopeChanged)]
    );
    let previous_child = state.active_logical_scopes()[1];
    let events = apply(&mut state, &graph, 3, vec![set_true(3), traverse(3)]);
    assert_eq!(state.slot(slot(1)), Some(&ActivityValue::Boolean(true)));
    assert_eq!(state.slot(slot(2)), Some(&ActivityValue::Boolean(false)));
    assert_eq!(state.slot(slot(3)), Some(&ActivityValue::Boolean(false)));
    let next_child = state.active_logical_scopes()[1];
    assert_eq!(previous_child.address(), next_child.address());
    assert_ne!(previous_child.visit_sequence(), next_child.visit_sequence());
    assert_eq!(
        resets(&events),
        vec![
            (slot(2), SlotResetPoint::LogicalScopeChanged),
            (slot(3), SlotResetPoint::LogicalScopeChanged),
        ]
    );
}

#[test]
fn raw_logical_reset_and_missing_class_are_rejected_at_definition_time() {
    assert_eq!(
        ActivitySlotDefinition::new(
            slot(1),
            ActivityScope::Node,
            ActivityValue::Boolean(false),
            None,
            vec![SlotResetPoint::LogicalScopeChanged]
        ),
        Err(SlotDefinitionError::UnboundLogicalScopeReset)
    );
    let state = ActivityStateDefinition::new(
        vec![boolean_slot(1).with_logical_scope(LogicalScopeClassId::new(1).unwrap())],
        vec![],
        vec![],
    )
    .unwrap();
    assert_eq!(
        GraphActivityDefinition::new(
            identity(),
            graph(),
            state,
            participants(),
            (1..=3)
                .map(|raw| GraphActivityNodeProgram::new(
                    node(raw),
                    ActivityProgramDefinition::new(ActivityProgramId::new(raw).unwrap(), vec![])
                        .unwrap()
                ))
                .collect(),
            None,
            ActivityRandomPolicies::default()
        )
        .unwrap_err(),
        GraphActivityDefinitionError::InvalidLogicalSlotScope(slot(1))
    );
}

#[test]
fn logical_slot_binding_enters_state_hash_even_when_current_values_match() {
    let left = start(Arc::new(choice_definition(1, false).unwrap()), 1);
    let right = start(Arc::new(choice_definition(2, false).unwrap()), 1);
    assert_eq!(left.current_node(), right.current_node());
    assert_eq!(
        left.debug_view().all_slots(),
        right.debug_view().all_slots()
    );
    assert_eq!(left.debug_view().rng(), right.debug_view().rng());
    assert_ne!(left.canonical_state_bytes(), right.canonical_state_bytes());
    assert_ne!(left.state_hash(), right.state_hash());
}

#[test]
fn downstream_failure_restores_logical_instances_slot_resets_and_generated_rng() {
    let mut activity = start(Arc::new(choice_definition(2, true).unwrap()), 2);
    assert_eq!(
        activity.debug_view().all_slots()[0].value(),
        &ActivityValue::Boolean(true)
    );
    let before = activity.canonical_state_bytes();
    let rng = activity.debug_view().rng().to_vec();
    let hash = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    assert!(
        activity
            .choose_option_with_generated_prefix(
                hash,
                decision,
                ActivityOptionId::new(1).unwrap(),
                |_, rng| {
                    rng.choose_index(ActivityRngLabel::Reward, 99, 5).unwrap();
                    Ok((vec![set_true(1)], ()))
                }
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view().rng(), rng);
    assert_eq!(activity.player_view().decision().unwrap().id(), decision);
}

fn choice_definition(
    bound_class: u32,
    reject: bool,
) -> Result<GraphActivityDefinition, GraphActivityDefinitionError> {
    let scopes = LogicalScopeDefinitions::new(
        vec![
            LogicalScopeClassDefinition::new(LogicalScopeClassId::new(1).unwrap(), None, 1)
                .unwrap(),
            LogicalScopeClassDefinition::new(
                LogicalScopeClassId::new(2).unwrap(),
                Some(LogicalScopeClassId::new(1).unwrap()),
                2,
            )
            .unwrap(),
        ],
        vec![
            LogicalScopeNodeBinding::new(node(1), vec![address(1, 1), address(2, 1)]).unwrap(),
            LogicalScopeNodeBinding::new(node(2), vec![address(1, 1), address(2, 2)]).unwrap(),
            LogicalScopeNodeBinding::new(node(3), vec![address(1, 1)]).unwrap(),
        ],
    )
    .unwrap();
    let state = ActivityStateDefinition::new(
        vec![boolean_slot(1).with_logical_scope(LogicalScopeClassId::new(bound_class).unwrap())],
        vec![],
        vec![],
    )
    .unwrap()
    .with_logical_scopes(scopes);
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            activity_node(1, ActivityNodeKind::Choice, 1),
            activity_node(2, ActivityNodeKind::Choice, 1),
            activity_node(
                3,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            ),
        ],
        vec![edge(1, 1, 2, 1), edge(2, 2, 3, 1)],
        3,
    )
    .unwrap();
    let first = ActivityProgramDefinition::new(
        ActivityProgramId::new(1).unwrap(),
        vec![
            set_true(1),
            ActivityOperation::Offer {
                kind: ActivityDecisionKind::Choice,
                options: vec![ActivityOptionDefinition::new(
                    ActivityOptionId::new(1).unwrap(),
                    0,
                    ActivityCondition::Boolean(ActivityExpression::Literal(
                        ActivityValue::Boolean(true),
                    )),
                    vec![traverse(1)],
                )]
                .into_boxed_slice(),
            },
        ],
    )
    .unwrap();
    let next = ActivityProgramDefinition::new(
        ActivityProgramId::new(2).unwrap(),
        vec![
            ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
                ActivityValue::Boolean(!reject),
            ))),
            traverse(2),
        ],
    )
    .unwrap();
    GraphActivityDefinition::new(
        identity(),
        graph,
        state,
        participants(),
        vec![
            GraphActivityNodeProgram::new(node(1), first),
            GraphActivityNodeProgram::new(node(2), next),
        ],
        None,
        ActivityRandomPolicies::default(),
    )
}

fn address(class: u32, key: u64) -> LogicalScopeAddress {
    LogicalScopeAddress::new(LogicalScopeClassId::new(class).unwrap(), key).unwrap()
}
fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([1; 32]).unwrap(),
        ActivityConfigDigest::new([2; 32]).unwrap(),
    )
}
fn participants() -> Arc<ParticipantLock> {
    let policy = ParticipantPolicy::new(
        1,
        1,
        1,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let build = OpaqueParticipantBuild::new(
        CombatantSpecDigest::new([3; 32]).unwrap(),
        BuildDigest::new([4; 32]).unwrap(),
        ParticipantSourceKind::Synthetic,
    )
    .unwrap();
    Arc::new(
        ParticipantLock::seal(
            policy,
            vec![
                ParticipantLockEntry::new(
                    ParticipantId::new(1).unwrap(),
                    0,
                    0,
                    UnitDefinitionId::new(1).unwrap(),
                    build,
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    )
}
fn start(definition: Arc<GraphActivityDefinition>, instance: u64) -> GraphActivity {
    GraphActivity::start(
        definition,
        ActivityInstanceId::new(instance).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap()
    .into_activity()
}
fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn boolean_slot(raw: u32) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new(
        slot(raw),
        ActivityScope::Node,
        ActivityValue::Boolean(false),
        None,
        vec![SlotResetPoint::NodeStart],
    )
    .unwrap()
}
fn set_true(raw: u32) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: slot(raw),
        value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
    }
}
fn traverse(raw: u32) -> ActivityOperation {
    ActivityOperation::Traverse(ActivityEdgeId::new(raw).unwrap())
}
fn resets(events: &[ActivityTransactionEvent]) -> Vec<(ActivitySlotId, SlotResetPoint)> {
    events
        .iter()
        .filter_map(|event| match event.kind() {
            ActivityTransactionEventKind::SlotReset { slot, point } => Some((*slot, *point)),
            _ => None,
        })
        .collect()
}
fn apply(
    state: &mut ActivityTransactionState,
    graph: &ActivityGraphDefinition,
    sequence: u32,
    operations: Vec<ActivityOperation>,
) -> Box<[ActivityTransactionEvent]> {
    let program =
        ActivityProgramDefinition::new(ActivityProgramId::new(sequence).unwrap(), operations)
            .unwrap();
    let cause =
        ActivityCause::new(u64::from(sequence), program.id(), state.current_node()).unwrap();
    match state.apply_program(&program, cause, graph) {
        ActivityTransactionOutcome::Committed(events) => events,
        other => panic!("expected committed transaction, got {other:?}"),
    }
}

fn apply_traverse(
    state: &mut ActivityTransactionState,
    graph: &ActivityGraphDefinition,
    sequence: u32,
    edge: u32,
) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(sequence).unwrap(),
        vec![ActivityOperation::Traverse(
            ActivityEdgeId::new(edge).unwrap(),
        )],
    )
    .unwrap();
    let cause = ActivityCause::new(
        u64::from(sequence),
        ActivityProgramId::new(sequence).unwrap(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, graph),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn graph() -> ActivityGraphDefinition {
    let nodes = vec![
        activity_node(1, ActivityNodeKind::Choice, 2),
        ActivityNodeDefinition::new(
            node(2),
            SectionId::new(2).unwrap(),
            ActivityNodeKind::Reward,
            2,
        )
        .unwrap(),
        activity_node(3, ActivityNodeKind::Checkpoint, 2),
        activity_node(
            4,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        ),
    ];
    let edges = vec![
        edge(1, 1, 2, 2),
        edge(2, 2, 3, 2),
        edge(3, 3, 1, 1),
        edge(4, 3, 4, 1),
    ];
    ActivityGraphDefinition::new(node(1), nodes, edges, 6).unwrap()
}

fn activity_node(raw: u32, kind: ActivityNodeKind, visits: u32) -> ActivityNodeDefinition {
    ActivityNodeDefinition::new(node(raw), SectionId::new(1).unwrap(), kind, visits).unwrap()
}

fn edge(raw: u32, from: u32, to: u32, traversals: u32) -> ActivityEdgeDefinition {
    ActivityEdgeDefinition::new(
        ActivityEdgeId::new(raw).unwrap(),
        node(from),
        node(to),
        ActivityEdgeCondition::Always,
        0,
        traversals,
    )
    .unwrap()
}

fn node(raw: u32) -> NodeId {
    NodeId::new(raw).unwrap()
}
