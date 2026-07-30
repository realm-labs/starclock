use crate::definition::RecommendedElement;

macro_rules! structural_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(crate) struct $name(pub(crate) u32);
    };
}

structural_id!(ProfileId);
structural_id!(AreaId);
structural_id!(DifficultySegmentId);
structural_id!(PlaneId);
structural_id!(ChessboardId);
structural_id!(MapColumnId);
structural_id!(MapNodeId);
structural_id!(MapEdgeId);
structural_id!(RoomId);
structural_id!(DomainId);
structural_id!(BeaconId);
structural_id!(BossChoiceId);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProfileRowKind {
    ResidentActivity,
    DlcEntrance,
    ModeTitle,
    RuntimeProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProfileDefinition {
    pub(crate) id: ProfileId,
    pub(crate) stable_key: Box<str>,
    pub(crate) kind: ProfileRowKind,
    pub(crate) source_id: Option<Box<str>>,
    pub(crate) unlock_id: Option<Box<str>>,
    pub(crate) sub_mode: Box<str>,
    pub(crate) game_version: Option<Box<str>>,
    pub(crate) reference_runtime_enabled: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AreaGroup {
    Formal,
    Guide,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AreaDefinition {
    pub(crate) id: AreaId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) group: AreaGroup,
    pub(crate) difficulty: u8,
    pub(crate) difficulty_segment_sources: Box<[Box<str>]>,
    pub(crate) plane_sources: Box<[Box<str>]>,
    pub(crate) unlock_id: Box<str>,
    pub(crate) recommended_level: u16,
    pub(crate) recommended_elements: Box<[RecommendedElement]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DifficultySegmentDefinition {
    pub(crate) id: DifficultySegmentId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) cut_positions: Box<[u16]>,
    pub(crate) levels: Box<[u16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PlaneDefinition {
    pub(crate) id: PlaneId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChessboardDefinition {
    pub(crate) id: ChessboardId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) start: MapNodeId,
    pub(crate) end: MapNodeId,
    pub(crate) block_create_group: Box<str>,
    pub(crate) event_sources: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MapColumnDefinition {
    pub(crate) id: MapColumnId,
    pub(crate) stable_key: Box<str>,
    pub(crate) chessboard: ChessboardId,
    pub(crate) index: u16,
    pub(crate) position_x: i32,
    pub(crate) node_keys: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DomainResolution {
    AuthoredCandidates,
    Unspecified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MapNodeDefinition {
    pub(crate) id: MapNodeId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) chessboard: ChessboardId,
    pub(crate) column: MapColumnId,
    pub(crate) position_x: i32,
    pub(crate) position_y: i32,
    pub(crate) domains: Box<[Box<str>]>,
    pub(crate) domain_resolution: DomainResolution,
    pub(crate) is_start: bool,
    pub(crate) is_end: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MapEdgeDefinition {
    pub(crate) id: MapEdgeId,
    pub(crate) stable_key: Box<str>,
    pub(crate) chessboard: ChessboardId,
    pub(crate) source: MapNodeId,
    pub(crate) target: MapNodeId,
    pub(crate) policy: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RoomDefinition {
    pub(crate) id: RoomId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) sub_mode: Box<str>,
    pub(crate) sections: Box<[u16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DomainDefinition {
    pub(crate) id: DomainId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BeaconDefinition {
    pub(crate) id: BeaconId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BossChoiceDefinition {
    pub(crate) id: BossChoiceId,
    pub(crate) stable_key: Box<str>,
    pub(crate) source_id: Box<str>,
    pub(crate) display_level: u16,
    pub(crate) weakness_elements: Box<[RecommendedElement]>,
    pub(crate) monster_template_id: Box<str>,
}
