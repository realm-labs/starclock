//! Immutable structural catalog lowered from private Gold and Gears rows.

mod lower;
mod types;

use crate::gold_gears_catalog::{
    GoldAndGearsBundleLoadError, GoldAndGearsBundleSummary, load_gold_and_gears_bundle,
};
pub(crate) use types::AreaGroup;
pub(crate) use types::{
    AreaDefinition, BeaconDefinition, BossChoiceDefinition, ChessboardDefinition,
    DifficultySegmentDefinition, DomainDefinition, MapColumnDefinition, MapEdgeDefinition,
    MapNodeDefinition, PlaneDefinition, ProfileDefinition, RoomDefinition,
};

pub(crate) const EXPECTED_STRUCTURAL_ROWS: usize = 8_621;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoldAndGearsStructuralErrorKind {
    Bundle,
    Metadata,
    Identifier,
    Denominator,
    Duplicate,
    Reference,
    Graph,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoldAndGearsStructuralError {
    pub(crate) kind: GoldAndGearsStructuralErrorKind,
    pub(crate) key: Box<str>,
}

impl core::fmt::Display for GoldAndGearsStructuralError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Gold and Gears structural catalog error {:?}: {}",
            self.kind, self.key
        )
    }
}

impl std::error::Error for GoldAndGearsStructuralError {}

impl From<GoldAndGearsBundleLoadError> for GoldAndGearsStructuralError {
    fn from(value: GoldAndGearsBundleLoadError) -> Self {
        Self {
            kind: GoldAndGearsStructuralErrorKind::Bundle,
            key: value.to_string().into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct GoldAndGearsStructuralCatalog {
    pub(crate) bundle: GoldAndGearsBundleSummary,
    pub(crate) profiles: Box<[ProfileDefinition]>,
    pub(crate) areas: Box<[AreaDefinition]>,
    pub(crate) difficulty_segments: Box<[DifficultySegmentDefinition]>,
    pub(crate) planes: Box<[PlaneDefinition]>,
    pub(crate) chessboards: Box<[ChessboardDefinition]>,
    pub(crate) columns: Box<[MapColumnDefinition]>,
    pub(crate) nodes: Box<[MapNodeDefinition]>,
    pub(crate) edges: Box<[MapEdgeDefinition]>,
    pub(crate) rooms: Box<[RoomDefinition]>,
    pub(crate) domains: Box<[DomainDefinition]>,
    pub(crate) beacons: Box<[BeaconDefinition]>,
    pub(crate) boss_choices: Box<[BossChoiceDefinition]>,
}

impl GoldAndGearsStructuralCatalog {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, GoldAndGearsStructuralError> {
        let (bundle, transport) = load_gold_and_gears_bundle(bytes)?;
        lower::lower(bundle, &transport)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.profiles.len()
            + self.areas.len()
            + self.difficulty_segments.len()
            + self.planes.len()
            + self.chessboards.len()
            + self.columns.len()
            + self.nodes.len()
            + self.edges.len()
            + self.rooms.len()
            + self.domains.len()
            + self.beacons.len()
            + self.boss_choices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

    #[test]
    fn lowers_all_structural_rows_and_validates_graph_closure() {
        let catalog = GoldAndGearsStructuralCatalog::load(BUNDLE).unwrap();
        assert_eq!(catalog.bundle.table_count(), 52);
        assert_eq!(catalog.profiles.len(), 4);
        assert_eq!(catalog.areas.len(), 8);
        assert_eq!(catalog.difficulty_segments.len(), 16);
        assert_eq!(catalog.planes.len(), 8);
        assert_eq!(catalog.chessboards.len(), 115);
        assert_eq!(catalog.columns.len(), 1_313);
        assert_eq!(catalog.nodes.len(), 2_502);
        assert_eq!(catalog.edges.len(), 3_407);
        assert_eq!(catalog.rooms.len(), 1_224);
        assert_eq!(catalog.domains.len(), 12);
        assert_eq!(catalog.beacons.len(), 6);
        assert_eq!(catalog.boss_choices.len(), 6);
        assert_eq!(catalog.row_count(), EXPECTED_STRUCTURAL_ROWS);
    }

    #[test]
    fn closed_value_parsers_reject_unknown_runtime_values() {
        assert!(lower::difficulty("Difficulty_5", "test").is_ok());
        assert!(lower::difficulty("Difficulty_6", "test").is_err());
        assert!(lower::element("Thunder", "test").is_ok());
        assert!(lower::element("Void", "test").is_err());
    }
}
