use super::{SwarmDisasterStructuralCatalog, types::AreaKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterTopologyInput {
    pub(crate) planes: Box<[SwarmDisasterPlaneTopologyInput]>,
    pub(crate) catalog_node_count: u32,
    pub(crate) catalog_edge_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterPlaneTopologyInput {
    pub(crate) plane_key: Box<str>,
    pub(crate) plane_number: u8,
    pub(crate) board_key: Box<str>,
    pub(crate) board_id: u32,
    pub(crate) start: u32,
    pub(crate) end: u32,
    pub(crate) nodes: Box<[SwarmDisasterTopologyNodeInput]>,
    pub(crate) edges: Box<[SwarmDisasterTopologyEdgeInput]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterTopologyNodeInput {
    pub(crate) id: u32,
    pub(crate) column_index: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterTopologyEdgeInput {
    pub(crate) id: u32,
    pub(crate) source: u32,
    pub(crate) target: u32,
}

impl SwarmDisasterStructuralCatalog {
    pub(crate) fn topology_input(&self, area_id: u32) -> Option<SwarmDisasterTopologyInput> {
        let area = self
            .areas
            .iter()
            .find(|area| area.id.0 == area_id && area.kind == AreaKind::Formal)?;
        let mut planes = area
            .plane_keys
            .iter()
            .map(|key| {
                let plane = self
                    .planes
                    .iter()
                    .find(|plane| plane.stable_key.as_ref() == key.as_ref())?;
                let root_source = format!("{}1", plane.source_id);
                let board = self.chessboards.iter().find(|board| {
                    board.source_id.as_ref() == root_source
                        && plane
                            .chessboard_keys
                            .iter()
                            .any(|key| key.as_ref() == board.stable_key.as_ref())
                })?;
                let nodes = self
                    .nodes
                    .iter()
                    .filter(|node| node.chessboard == board.id)
                    .map(|node| {
                        let column_index = self
                            .columns
                            .iter()
                            .find(|column| column.id == node.column)?
                            .index;
                        Some(SwarmDisasterTopologyNodeInput {
                            id: node.id.0,
                            column_index,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?;
                let edges = self
                    .edges
                    .iter()
                    .filter(|edge| edge.chessboard == board.id)
                    .map(|edge| SwarmDisasterTopologyEdgeInput {
                        id: edge.id.0,
                        source: edge.source.0,
                        target: edge.target.0,
                    })
                    .collect::<Vec<_>>();
                Some(SwarmDisasterPlaneTopologyInput {
                    plane_key: plane.stable_key.clone(),
                    plane_number: plane.plane_number,
                    board_key: board.stable_key.clone(),
                    board_id: board.id.0,
                    start: board.start.0,
                    end: board.end.0,
                    nodes: nodes.into_boxed_slice(),
                    edges: edges.into_boxed_slice(),
                })
            })
            .collect::<Option<Vec<_>>>()?;
        planes.sort_unstable_by_key(|plane| plane.plane_number);
        Some(SwarmDisasterTopologyInput {
            planes: planes.into_boxed_slice(),
            catalog_node_count: u32::try_from(self.nodes.len()).ok()?,
            catalog_edge_count: u32::try_from(self.edges.len()).ok()?,
        })
    }
}
