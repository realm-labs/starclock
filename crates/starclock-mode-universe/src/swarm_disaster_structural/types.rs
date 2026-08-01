use crate::definition::RecommendedElement;

macro_rules! structural_id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub(super) struct $name(pub(super) u32);
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
pub(super) enum ProfileRowKind {
    ResidentActivity,
    DlcEntrance,
    ModeTitle,
    RuntimeProfile,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ProfileDefinition {
    pub(super) id: ProfileId,
    pub(super) stable_key: Box<str>,
    pub(super) kind: ProfileRowKind,
    pub(super) sub_mode: Box<str>,
    pub(super) runtime_enabled: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AreaKind {
    Formal,
    Guide,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct AreaDefinition {
    pub(super) id: AreaId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) kind: AreaKind,
    pub(super) difficulty: u8,
    pub(super) difficulty_segment_keys: Box<[Box<str>]>,
    pub(super) plane_keys: Box<[Box<str>]>,
    pub(super) recommended_level: u16,
    pub(super) recommended_elements: Box<[RecommendedElement]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DifficultySegmentDefinition {
    pub(super) id: DifficultySegmentId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) cut_positions: Box<[u16]>,
    pub(super) levels: Box<[u16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct PlaneDefinition {
    pub(super) id: PlaneId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) plane_number: u8,
    pub(super) chessboard_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ChessboardDefinition {
    pub(super) id: ChessboardId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) width: u16,
    pub(super) height: u16,
    pub(super) start: MapNodeId,
    pub(super) end: MapNodeId,
    pub(super) block_create_group: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MapColumnDefinition {
    pub(super) id: MapColumnId,
    pub(super) stable_key: Box<str>,
    pub(super) chessboard: ChessboardId,
    pub(super) index: u16,
    pub(super) position_x: i32,
    pub(super) node_keys: Box<[Box<str>]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum DomainResolution {
    AuthoredCandidates,
    Unspecified,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MapNodeDefinition {
    pub(super) id: MapNodeId,
    pub(super) stable_key: Box<str>,
    pub(super) chessboard: ChessboardId,
    pub(super) column: MapColumnId,
    pub(super) position_x: i32,
    pub(super) domain_keys: Box<[Box<str>]>,
    pub(super) domain_resolution: DomainResolution,
    pub(super) is_start: bool,
    pub(super) is_end: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct MapEdgeDefinition {
    pub(super) id: MapEdgeId,
    pub(super) stable_key: Box<str>,
    pub(super) chessboard: ChessboardId,
    pub(super) source: MapNodeId,
    pub(super) target: MapNodeId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RoomDefinition {
    pub(super) id: RoomId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) sub_mode: Box<str>,
    pub(super) sections: Box<[u16]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct DomainDefinition {
    pub(super) id: DomainId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BeaconDefinition {
    pub(super) id: BeaconId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) block_intro_id: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct BossChoiceDefinition {
    pub(super) id: BossChoiceId,
    pub(super) stable_key: Box<str>,
    pub(super) source_id: Box<str>,
    pub(super) display_level: u16,
    pub(super) enemy_variant_id: Box<str>,
    pub(super) weakness_elements: Box<[RecommendedElement]>,
}
