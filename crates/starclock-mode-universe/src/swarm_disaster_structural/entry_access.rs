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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterEncounterStructuralInput {
    pub(crate) area_id: u32,
    pub(crate) area_key: Box<str>,
    pub(crate) difficulty: u8,
    pub(crate) bands: Box<[SwarmDisasterDifficultyBandInput]>,
    pub(crate) selected_band_keys: Box<[Box<str>]>,
    pub(crate) nodes: Box<[SwarmDisasterEncounterNodeInput]>,
    pub(crate) room_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterDifficultyBandInput {
    pub(crate) key: Box<str>,
    pub(crate) cuts: Box<[u16]>,
    pub(crate) levels: Box<[u16]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterEncounterNodeInput {
    pub(crate) id: u32,
    pub(crate) plane: u8,
    pub(crate) position: u16,
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

    pub(crate) fn encounter_runtime_input(
        &self,
        area_id: u32,
    ) -> Option<SwarmDisasterEncounterStructuralInput> {
        let area = self
            .areas
            .iter()
            .find(|area| area.id.0 == area_id && area.kind == AreaKind::Formal)?;
        if area.difficulty_segment_keys.len() != area.plane_keys.len() {
            return None;
        }
        let bands = self
            .difficulty_segments
            .iter()
            .map(|segment| SwarmDisasterDifficultyBandInput {
                key: segment.stable_key.clone(),
                cuts: segment.cut_positions.clone(),
                levels: segment.levels.clone(),
            })
            .collect::<Vec<_>>();
        let mut nodes = Vec::new();
        for (plane_index, plane_key) in area.plane_keys.iter().enumerate() {
            let plane = self
                .planes
                .iter()
                .find(|plane| plane.stable_key.as_ref() == plane_key.as_ref())?;
            let root_source = format!("{}1", plane.source_id);
            let board = self.chessboards.iter().find(|board| {
                board.source_id.as_ref() == root_source
                    && plane
                        .chessboard_keys
                        .iter()
                        .any(|key| key.as_ref() == board.stable_key.as_ref())
            })?;
            let plane = u8::try_from(plane_index + 1).ok()?;
            for node in self.nodes.iter().filter(|node| node.chessboard == board.id) {
                let position = self
                    .columns
                    .iter()
                    .find(|column| column.id == node.column)?
                    .index;
                nodes.push(SwarmDisasterEncounterNodeInput {
                    id: node.id.0,
                    plane,
                    position,
                });
            }
        }
        nodes.sort_unstable_by_key(|node| node.id);
        Some(SwarmDisasterEncounterStructuralInput {
            area_id: area.id.0,
            area_key: area.stable_key.clone(),
            difficulty: area.difficulty,
            bands: bands.into_boxed_slice(),
            selected_band_keys: area.difficulty_segment_keys.clone(),
            nodes: nodes.into_boxed_slice(),
            room_count: self.rooms.len(),
        })
    }
}
