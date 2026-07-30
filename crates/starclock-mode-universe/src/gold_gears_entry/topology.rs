//! Bounded Gold and Gears chessboard compilation over the generic Activity graph.

use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityTerminalOutcome, LogicalScopeAddress,
    LogicalScopeClassDefinition, LogicalScopeClassId, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, SectionId,
};

use crate::gold_gears_structural::{
    AreaDefinition, ChessboardDefinition, GoldAndGearsStructuralCatalog, MapNodeDefinition,
};

use super::GoldAndGearsEntryError;

pub(super) const PLANE_BOARD_SCOPE_CLASS: u32 = 0x4747_1001;
pub(super) const BOARD_NODE_VISIT_SCOPE_CLASS: u32 = 0x4747_1002;
pub(super) const NODE_INTERACTION_SCOPE_CLASS: u32 = 0x4747_1003;

const EXPECTED_PLANE_COUNT: usize = 3;
const ROOT_CHESSBOARD_PREFIX: &str = "211";
const INTERACTION_KIND_ROUTE: u64 = 1;

pub(super) struct CompiledTopology {
    pub(super) graph: ActivityGraphDefinition,
    pub(super) scopes: LogicalScopeDefinitions,
    pub(super) planes: Box<[CompiledPlaneBoard]>,
}

pub(super) struct CompiledPlaneBoard {
    pub(super) plane_key: Box<str>,
    pub(super) chessboard_key: Box<str>,
}

pub(super) fn compile_topology(
    catalog: &GoldAndGearsStructuralCatalog,
    area: &AreaDefinition,
) -> Result<CompiledTopology, GoldAndGearsEntryError> {
    if area.plane_sources.len() != EXPECTED_PLANE_COUNT {
        return Err(GoldAndGearsEntryError::InvalidPlaneCount);
    }

    let selected = area
        .plane_sources
        .iter()
        .map(|source| select_plane_board(catalog, source))
        .collect::<Result<Vec<_>, _>>()?;
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut bindings = Vec::new();
    let mut planes = Vec::new();

    for (index, (plane, board)) in selected.iter().enumerate() {
        let ordinal =
            u32::try_from(index + 1).map_err(|_| GoldAndGearsEntryError::InvalidTopology)?;
        let section = SectionId::new(ordinal).ok_or(GoldAndGearsEntryError::InvalidTopology)?;
        let board_nodes = catalog
            .nodes
            .iter()
            .filter(|node| node.chessboard == board.id)
            .collect::<Vec<_>>();
        for node in board_nodes {
            let node_id = NodeId::new(node.id.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?;
            let kind = if index + 1 == selected.len() && node.id == board.end {
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed)
            } else {
                ActivityNodeKind::Choice
            };
            nodes.push(
                ActivityNodeDefinition::new(node_id, section, kind, 1)
                    .map_err(|_| GoldAndGearsEntryError::InvalidTopology)?,
            );
            bindings.push(scope_binding(ordinal, board, node)?);
        }
        for edge in catalog
            .edges
            .iter()
            .filter(|edge| edge.chessboard == board.id)
        {
            edges.push(
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(edge.id.0)
                        .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
                    NodeId::new(edge.source.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?,
                    NodeId::new(edge.target.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .map_err(|_| GoldAndGearsEntryError::InvalidTopology)?,
            );
        }
        planes.push(CompiledPlaneBoard {
            plane_key: plane.stable_key.clone(),
            chessboard_key: board.stable_key.clone(),
        });
    }

    for (index, pair) in selected.windows(2).enumerate() {
        let edge_offset =
            u32::try_from(index + 1).map_err(|_| GoldAndGearsEntryError::InvalidTopology)?;
        let edge_id = u32::try_from(catalog.edges.len())
            .ok()
            .and_then(|count| count.checked_add(edge_offset))
            .and_then(ActivityEdgeId::new)
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?;
        edges.push(
            ActivityEdgeDefinition::new(
                edge_id,
                NodeId::new(pair[0].1.end.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?,
                NodeId::new(pair[1].1.start.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .map_err(|_| GoldAndGearsEntryError::InvalidTopology)?,
        );
    }

    let entry =
        NodeId::new(selected[0].1.start.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?;
    let maximum_total_visits =
        u32::try_from(nodes.len()).map_err(|_| GoldAndGearsEntryError::InvalidTopology)?;
    let graph = ActivityGraphDefinition::new(entry, nodes, edges, maximum_total_visits)
        .map_err(|_| GoldAndGearsEntryError::InvalidTopology)?;
    let scopes = logical_scopes(bindings)?;

    Ok(CompiledTopology {
        graph,
        scopes,
        planes: planes.into_boxed_slice(),
    })
}

fn select_plane_board<'a>(
    catalog: &'a GoldAndGearsStructuralCatalog,
    plane_source: &str,
) -> Result<
    (
        &'a crate::gold_gears_structural::PlaneDefinition,
        &'a ChessboardDefinition,
    ),
    GoldAndGearsEntryError,
> {
    let plane = catalog
        .planes
        .iter()
        .find(|plane| plane.source_id.as_ref() == plane_source)
        .ok_or_else(|| GoldAndGearsEntryError::MissingPlane(plane_source.into()))?;
    let board_source = format!("{ROOT_CHESSBOARD_PREFIX}{plane_source}");
    let board = catalog
        .chessboards
        .iter()
        .find(|board| board.source_id.as_ref() == board_source)
        .ok_or_else(|| GoldAndGearsEntryError::MissingChessboard(board_source.into()))?;
    Ok((plane, board))
}

fn scope_binding(
    ordinal: u32,
    board: &ChessboardDefinition,
    node: &MapNodeDefinition,
) -> Result<LogicalScopeNodeBinding, GoldAndGearsEntryError> {
    let plane_class = scope_class(PLANE_BOARD_SCOPE_CLASS)?;
    let node_class = scope_class(BOARD_NODE_VISIT_SCOPE_CLASS)?;
    let interaction_class = scope_class(NODE_INTERACTION_SCOPE_CLASS)?;
    let plane_key = u64::from(ordinal)
        .checked_shl(32)
        .and_then(|value| value.checked_add(u64::from(board.id.0)))
        .ok_or(GoldAndGearsEntryError::InvalidTopology)?;
    let interaction_key = INTERACTION_KIND_ROUTE
        .checked_shl(32)
        .and_then(|value| value.checked_add(u64::from(node.id.0)))
        .ok_or(GoldAndGearsEntryError::InvalidTopology)?;
    let path = vec![
        LogicalScopeAddress::new(plane_class, plane_key)
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
        LogicalScopeAddress::new(node_class, u64::from(node.id.0))
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
        LogicalScopeAddress::new(interaction_class, interaction_key)
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
    ];
    LogicalScopeNodeBinding::new(
        NodeId::new(node.id.0).ok_or(GoldAndGearsEntryError::InvalidTopology)?,
        path,
    )
    .map_err(|_| GoldAndGearsEntryError::InvalidTopology)
}

fn logical_scopes(
    bindings: Vec<LogicalScopeNodeBinding>,
) -> Result<LogicalScopeDefinitions, GoldAndGearsEntryError> {
    let plane = scope_class(PLANE_BOARD_SCOPE_CLASS)?;
    let node = scope_class(BOARD_NODE_VISIT_SCOPE_CLASS)?;
    let interaction = scope_class(NODE_INTERACTION_SCOPE_CLASS)?;
    let classes = vec![
        LogicalScopeClassDefinition::new(plane, None, 3)
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
        LogicalScopeClassDefinition::new(node, Some(plane), 2_502)
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
        LogicalScopeClassDefinition::new(interaction, Some(node), 8_192)
            .ok_or(GoldAndGearsEntryError::InvalidTopology)?,
    ];
    LogicalScopeDefinitions::new(classes, bindings)
        .map_err(|_| GoldAndGearsEntryError::InvalidTopology)
}

fn scope_class(raw: u32) -> Result<LogicalScopeClassId, GoldAndGearsEntryError> {
    LogicalScopeClassId::new(raw).ok_or(GoldAndGearsEntryError::InvalidTopology)
}
