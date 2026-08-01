use super::SwarmDisasterStructuralCatalog;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterMapStructuralInput {
    pub(crate) boards: Box<[SwarmDisasterMapBoardInput]>,
    pub(crate) domains: Box<[(Box<str>, u32)]>,
    pub(crate) beacons: Box<[(Box<str>, u32)]>,
    pub(crate) room_bindings: Box<[SwarmDisasterRoomBindingInput]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterMapBoardInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) nodes: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterRoomBindingInput {
    pub(crate) key: Box<str>,
    pub(crate) sections: Box<[u16]>,
}

impl SwarmDisasterStructuralCatalog {
    pub(crate) fn map_structural_input(&self) -> SwarmDisasterMapStructuralInput {
        let boards = self
            .chessboards
            .iter()
            .map(|board| SwarmDisasterMapBoardInput {
                id: board.id.0,
                key: board.stable_key.clone(),
                nodes: self
                    .nodes
                    .iter()
                    .filter(|node| node.chessboard == board.id)
                    .map(|node| node.id.0)
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        SwarmDisasterMapStructuralInput {
            boards,
            domains: self
                .domains
                .iter()
                .map(|domain| (domain.stable_key.clone(), domain.id.0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            beacons: self
                .beacons
                .iter()
                .map(|beacon| (beacon.stable_key.clone(), beacon.id.0))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            room_bindings: self
                .rooms
                .iter()
                .map(|room| SwarmDisasterRoomBindingInput {
                    key: room.stable_key.clone(),
                    sections: room.sections.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }
}
