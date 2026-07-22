use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityGraphDefinitionError, ActivityNodeDefinition, ActivityNodeKind,
    ActivityTerminalOutcome, NodeId, OneBattleFlow, SectionId, TerminalOutcome,
};

#[test]
fn one_battle_profile_compiles_to_the_generic_graph_without_legacy_state_changes() {
    let flow = OneBattleFlow::new(section(20), node(21), node(22), node(23), node(24)).unwrap();
    let graph = flow.into_graph();

    assert_eq!(graph.entry(), node(21));
    assert_eq!(graph.nodes().len(), 4);
    assert_eq!(graph.edges().len(), 3);
    assert_eq!(graph.maximum_total_visits(), 2);
    assert_eq!(
        graph
            .outgoing(node(21))
            .map(|edge| edge.condition())
            .collect::<Vec<_>>(),
        vec![
            ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete),
            ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed),
            ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Faulted),
        ]
    );
    assert!(graph.node(node(24)).is_some());
}

#[test]
fn graph_identity_is_independent_of_input_order() {
    let nodes = bounded_cycle_nodes();
    let edges = bounded_cycle_edges();
    let left = ActivityGraphDefinition::new(node(1), nodes.clone(), edges.clone(), 8).unwrap();
    let right = ActivityGraphDefinition::new(
        node(1),
        nodes.into_iter().rev().collect(),
        edges.into_iter().rev().collect(),
        8,
    )
    .unwrap();

    assert_eq!(left, right);
    assert_eq!(left.digest(), right.digest());
    assert_eq!(
        left.digest().bytes(),
        [
            13, 197, 204, 33, 1, 156, 97, 254, 180, 66, 84, 180, 190, 152, 30, 163, 154, 53, 36,
            128, 225, 194, 91, 0, 156, 51, 5, 186, 246, 111, 106, 247,
        ]
    );
    assert_eq!(
        left.nodes()
            .iter()
            .map(|item| item.id().get())
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn loops_are_accepted_only_with_finite_node_edge_and_activity_budgets() {
    let graph =
        ActivityGraphDefinition::new(node(1), bounded_cycle_nodes(), bounded_cycle_edges(), 8)
            .unwrap();
    assert_eq!(graph.node(node(2)).unwrap().maximum_visits(), 4);

    let error =
        ActivityGraphDefinition::new(node(1), bounded_cycle_nodes(), bounded_cycle_edges(), 3)
            .unwrap_err();
    assert_eq!(error, ActivityGraphDefinitionError::NodeLimitExceedsTotal);
}

#[test]
fn validation_rejects_dangling_unreachable_and_nonterminating_shapes() {
    let dangling = vec![edge(1, 1, 9, ActivityEdgeCondition::Always, 1)];
    let error = ActivityGraphDefinition::new(
        node(1),
        vec![plain_node(1, ActivityNodeKind::Choice, 1)],
        dangling,
        2,
    )
    .unwrap_err();
    assert_eq!(
        error,
        ActivityGraphDefinitionError::MissingEdgeTarget(edge_id(1))
    );

    let error = ActivityGraphDefinition::new(
        node(1),
        vec![
            plain_node(1, ActivityNodeKind::Choice, 1),
            plain_node(
                2,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            ),
            plain_node(
                3,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
                1,
            ),
        ],
        vec![edge(1, 1, 2, ActivityEdgeCondition::Always, 1)],
        3,
    )
    .unwrap_err();
    assert_eq!(
        error,
        ActivityGraphDefinitionError::UnreachableNode(node(3))
    );

    let error = ActivityGraphDefinition::new(
        node(1),
        vec![
            plain_node(1, ActivityNodeKind::Choice, 2),
            plain_node(2, ActivityNodeKind::Choice, 2),
            plain_node(
                3,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            ),
        ],
        vec![
            edge(1, 1, 2, ActivityEdgeCondition::Always, 2),
            edge(2, 2, 1, ActivityEdgeCondition::Always, 2),
            edge(3, 3, 1, ActivityEdgeCondition::Always, 1),
        ],
        6,
    )
    .unwrap_err();
    assert_eq!(
        error,
        ActivityGraphDefinitionError::TerminalHasOutgoingEdge(node(3))
    );

    let error = ActivityGraphDefinition::new(
        node(1),
        vec![
            plain_node(1, ActivityNodeKind::Choice, 2),
            plain_node(2, ActivityNodeKind::Reward, 2),
            plain_node(
                3,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            ),
        ],
        vec![
            edge(1, 1, 2, ActivityEdgeCondition::Always, 1),
            edge(2, 2, 2, ActivityEdgeCondition::Always, 2),
            edge(3, 1, 3, ActivityEdgeCondition::Always, 1),
        ],
        5,
    )
    .unwrap_err();
    assert_eq!(
        error,
        ActivityGraphDefinitionError::CannotReachTerminal(node(2))
    );
}

fn bounded_cycle_nodes() -> Vec<ActivityNodeDefinition> {
    vec![
        plain_node(1, ActivityNodeKind::Choice, 4),
        plain_node(2, ActivityNodeKind::Reward, 4),
        plain_node(
            3,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        ),
    ]
}

fn bounded_cycle_edges() -> Vec<ActivityEdgeDefinition> {
    vec![
        edge(1, 1, 2, ActivityEdgeCondition::OptionSelected, 4),
        edge(2, 2, 1, ActivityEdgeCondition::Always, 3),
        edge(3, 2, 3, ActivityEdgeCondition::Always, 1),
    ]
}

fn plain_node(id: u32, kind: ActivityNodeKind, maximum_visits: u32) -> ActivityNodeDefinition {
    ActivityNodeDefinition::new(node(id), section(1), kind, maximum_visits).unwrap()
}

fn edge(
    id: u32,
    from: u32,
    to: u32,
    condition: ActivityEdgeCondition,
    maximum_traversals: u32,
) -> ActivityEdgeDefinition {
    ActivityEdgeDefinition::new(
        edge_id(id),
        node(from),
        node(to),
        condition,
        id as i32,
        maximum_traversals,
    )
    .unwrap()
}

fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}

fn section(value: u32) -> SectionId {
    SectionId::new(value).unwrap()
}

fn edge_id(value: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(value).unwrap()
}
