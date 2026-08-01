use std::collections::{BTreeMap, BTreeSet};

use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityTerminalOutcome, LogicalScopeAddress,
    LogicalScopeClassDefinition, LogicalScopeClassId, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, SectionId,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_structural::entry_access::{
        SwarmDisasterPlaneTopologyInput, SwarmDisasterTopologyEdgeInput,
        SwarmDisasterTopologyInput, SwarmDisasterTopologyNodeInput,
    },
};

pub(super) const PLANE_BOARD_SCOPE_CLASS: u32 = 0x5344_1001;
pub(super) const BOARD_NODE_VISIT_SCOPE_CLASS: u32 = 0x5344_1002;
pub(super) const NODE_INTERACTION_SCOPE_CLASS: u32 = 0x5344_1003;

const EXPECTED_PLANE_COUNT: usize = 3;
const INTERACTION_KIND_ROUTE: u64 = 1;
const MAXIMUM_BOARD_NODE_INSTANCES: u32 = 1_991;
const MAXIMUM_INTERACTION_INSTANCES: u32 = 8_192;

#[derive(Debug)]
pub(super) struct CompiledTopology {
    pub(super) graph: ActivityGraphDefinition,
    pub(super) scopes: LogicalScopeDefinitions,
    pub(super) planes: Box<[CompiledPlane]>,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledPlane {
    pub(super) plane_key: Box<str>,
    pub(super) board_key: Box<str>,
    pub(super) start: NodeId,
    pub(super) end: NodeId,
}

pub(super) fn compile(
    input: SwarmDisasterTopologyInput,
) -> Result<CompiledTopology, UniverseCatalogLoadError> {
    validate_planes(&input.planes)?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut bindings = Vec::new();
    let mut planes = Vec::new();

    for plane in &input.planes {
        let section = section(plane.plane_number)?;
        let route_nodes = legal_route_nodes(plane);
        if !route_nodes.contains(&plane.start) || !route_nodes.contains(&plane.end) {
            return Err(graph_error("Swarm root board has no legal terminal route"));
        }
        let column_by_node = plane
            .nodes
            .iter()
            .map(|node| (node.id, node.column_index))
            .collect::<BTreeMap<_, _>>();
        for node in plane
            .nodes
            .iter()
            .filter(|node| route_nodes.contains(&node.id))
        {
            let node_id = node_id(node.id)?;
            nodes.push(
                ActivityNodeDefinition::new(node_id, section, ActivityNodeKind::Choice, 1)
                    .map_err(|_| graph_error("invalid Swarm node visit budget"))?,
            );
            bindings.push(scope_binding(plane, node)?);
        }
        for edge in plane
            .edges
            .iter()
            .filter(|edge| route_nodes.contains(&edge.source) && route_nodes.contains(&edge.target))
        {
            let source_column = column_by_node
                .get(&edge.source)
                .ok_or_else(|| graph_error("Swarm route source is missing"))?;
            let target_column = column_by_node
                .get(&edge.target)
                .ok_or_else(|| graph_error("Swarm route target is missing"))?;
            if source_column.checked_add(1) != Some(*target_column) {
                return Err(graph_error(
                    "Swarm route does not advance exactly one authored column",
                ));
            }
            edges.push(edge_definition(edge.id, edge.source, edge.target)?);
        }
        planes.push(CompiledPlane {
            plane_key: plane.plane_key.clone(),
            board_key: plane.board_key.clone(),
            start: node_id(plane.start)?,
            end: node_id(plane.end)?,
        });
    }

    for (index, pair) in input.planes.windows(2).enumerate() {
        let offset =
            u32::try_from(index + 1).map_err(|_| graph_error("Swarm transition edge overflow"))?;
        let id = input
            .catalog_edge_count
            .checked_add(offset)
            .ok_or_else(|| graph_error("Swarm transition edge overflow"))?;
        edges.push(edge_definition(id, pair[0].end, pair[1].start)?);
    }

    let terminal = input
        .catalog_node_count
        .checked_add(1)
        .and_then(NodeId::new)
        .ok_or_else(|| graph_error("Swarm terminal node overflow"))?;
    nodes.push(
        ActivityNodeDefinition::new(
            terminal,
            section(3)?,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .map_err(|_| graph_error("invalid Swarm terminal visit budget"))?,
    );
    bindings.push(terminal_scope_binding(
        input
            .planes
            .last()
            .ok_or_else(|| graph_error("Swarm topology has no final plane"))?,
        terminal,
    )?);
    let terminal_edge = input
        .catalog_edge_count
        .checked_add(3)
        .ok_or_else(|| graph_error("Swarm terminal edge overflow"))?;
    edges.push(edge_definition(
        terminal_edge,
        input.planes[2].end,
        terminal.get(),
    )?);

    let maximum_total_visits =
        u32::try_from(nodes.len()).map_err(|_| graph_error("Swarm total visit budget overflow"))?;
    let graph = ActivityGraphDefinition::new(
        node_id(input.planes[0].start)?,
        nodes,
        edges,
        maximum_total_visits,
    )
    .map_err(|error| {
        UniverseCatalogLoadError::new(
            UniverseCatalogLoadErrorKind::InvalidGraph,
            format!("invalid bounded Swarm Activity graph: {error:?}"),
        )
    })?;
    let scopes = logical_scopes(bindings)?;
    Ok(CompiledTopology {
        graph,
        scopes,
        planes: planes.into_boxed_slice(),
    })
}

fn validate_planes(
    planes: &[SwarmDisasterPlaneTopologyInput],
) -> Result<(), UniverseCatalogLoadError> {
    if planes.len() != EXPECTED_PLANE_COUNT
        || planes
            .iter()
            .enumerate()
            .any(|(index, plane)| usize::from(plane.plane_number) != index + 1)
        || planes.iter().any(|plane| {
            plane.nodes.is_empty()
                || plane.edges.is_empty()
                || !plane.nodes.iter().any(|node| node.id == plane.start)
                || !plane.nodes.iter().any(|node| node.id == plane.end)
        })
    {
        return Err(graph_error(
            "Swarm topology must contain ordered planes 1..=3",
        ));
    }
    Ok(())
}

fn legal_route_nodes(plane: &SwarmDisasterPlaneTopologyInput) -> BTreeSet<u32> {
    let from_start = walk([plane.start], &plane.edges, false);
    let reaches_end = walk([plane.end], &plane.edges, true);
    from_start.intersection(&reaches_end).copied().collect()
}

fn walk(
    starts: impl IntoIterator<Item = u32>,
    edges: &[SwarmDisasterTopologyEdgeInput],
    reverse: bool,
) -> BTreeSet<u32> {
    let mut seen = BTreeSet::new();
    let mut pending = starts.into_iter().collect::<Vec<_>>();
    while let Some(current) = pending.pop() {
        if !seen.insert(current) {
            continue;
        }
        for edge in edges {
            let (source, target) = if reverse {
                (edge.target, edge.source)
            } else {
                (edge.source, edge.target)
            };
            if source == current && !seen.contains(&target) {
                pending.push(target);
            }
        }
    }
    seen
}

fn edge_definition(
    id: u32,
    source: u32,
    target: u32,
) -> Result<ActivityEdgeDefinition, UniverseCatalogLoadError> {
    ActivityEdgeDefinition::new(
        ActivityEdgeId::new(id).ok_or_else(|| graph_error("invalid Swarm edge ID"))?,
        node_id(source)?,
        node_id(target)?,
        ActivityEdgeCondition::Always,
        0,
        1,
    )
    .map_err(|_| graph_error("invalid Swarm edge traversal budget"))
}

fn scope_binding(
    plane: &SwarmDisasterPlaneTopologyInput,
    node: &SwarmDisasterTopologyNodeInput,
) -> Result<LogicalScopeNodeBinding, UniverseCatalogLoadError> {
    let plane_key = u64::from(plane.plane_number)
        .checked_shl(32)
        .and_then(|value| value.checked_add(u64::from(plane.board_id)))
        .ok_or_else(|| graph_error("Swarm plane scope key overflow"))?;
    let interaction_key = INTERACTION_KIND_ROUTE
        .checked_shl(32)
        .and_then(|value| value.checked_add(u64::from(node.id)))
        .ok_or_else(|| graph_error("Swarm interaction scope key overflow"))?;
    let path = vec![
        address(PLANE_BOARD_SCOPE_CLASS, plane_key)?,
        address(BOARD_NODE_VISIT_SCOPE_CLASS, u64::from(node.id))?,
        address(NODE_INTERACTION_SCOPE_CLASS, interaction_key)?,
    ];
    LogicalScopeNodeBinding::new(node_id(node.id)?, path)
        .map_err(|_| graph_error("invalid Swarm logical scope binding"))
}

fn terminal_scope_binding(
    plane: &SwarmDisasterPlaneTopologyInput,
    terminal: NodeId,
) -> Result<LogicalScopeNodeBinding, UniverseCatalogLoadError> {
    let plane_key = u64::from(plane.plane_number)
        .checked_shl(32)
        .and_then(|value| value.checked_add(u64::from(plane.board_id)))
        .ok_or_else(|| graph_error("Swarm terminal scope key overflow"))?;
    LogicalScopeNodeBinding::new(terminal, vec![address(PLANE_BOARD_SCOPE_CLASS, plane_key)?])
        .map_err(|_| graph_error("invalid Swarm terminal scope binding"))
}

fn logical_scopes(
    bindings: Vec<LogicalScopeNodeBinding>,
) -> Result<LogicalScopeDefinitions, UniverseCatalogLoadError> {
    let plane = scope_class(PLANE_BOARD_SCOPE_CLASS)?;
    let node = scope_class(BOARD_NODE_VISIT_SCOPE_CLASS)?;
    let interaction = scope_class(NODE_INTERACTION_SCOPE_CLASS)?;
    let classes = vec![
        LogicalScopeClassDefinition::new(plane, None, 3)
            .ok_or_else(|| graph_error("invalid Swarm plane scope class"))?,
        LogicalScopeClassDefinition::new(node, Some(plane), MAXIMUM_BOARD_NODE_INSTANCES)
            .ok_or_else(|| graph_error("invalid Swarm node scope class"))?,
        LogicalScopeClassDefinition::new(interaction, Some(node), MAXIMUM_INTERACTION_INSTANCES)
            .ok_or_else(|| graph_error("invalid Swarm interaction scope class"))?,
    ];
    LogicalScopeDefinitions::new(classes, bindings)
        .map_err(|_| graph_error("invalid Swarm logical scope definitions"))
}

fn address(raw_class: u32, key: u64) -> Result<LogicalScopeAddress, UniverseCatalogLoadError> {
    LogicalScopeAddress::new(scope_class(raw_class)?, key)
        .ok_or_else(|| graph_error("invalid Swarm logical scope address"))
}

fn scope_class(raw: u32) -> Result<LogicalScopeClassId, UniverseCatalogLoadError> {
    LogicalScopeClassId::new(raw).ok_or_else(|| graph_error("invalid Swarm scope class ID"))
}

fn node_id(raw: u32) -> Result<NodeId, UniverseCatalogLoadError> {
    NodeId::new(raw).ok_or_else(|| graph_error("invalid Swarm topology node ID"))
}

fn section(raw: u8) -> Result<SectionId, UniverseCatalogLoadError> {
    SectionId::new(u32::from(raw)).ok_or_else(|| graph_error("invalid Swarm plane section"))
}

fn graph_error(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidGraph, message)
}
