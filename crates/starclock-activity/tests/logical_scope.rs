use starclock_activity::{
    ActivityCause, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityGraphDefinition, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityProgramDefinition, ActivityProgramId, ActivityStateDefinition, ActivityTerminalOutcome,
    ActivityTransactionOutcome, ActivityTransactionState, LogicalScopeAddress,
    LogicalScopeClassDefinition, LogicalScopeClassId, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, SectionId,
};

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
        activity_node(2, ActivityNodeKind::Reward, 2),
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
