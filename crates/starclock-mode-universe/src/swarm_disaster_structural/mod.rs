//! Immutable Swarm Disaster structural catalog lowered from private Sora rows.

pub(super) mod entry_access;
mod lower;
pub(super) mod map_access;
mod types;

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_catalog::{SwarmDisasterBundleSummary, load_validated_swarm_disaster_bundle},
};
use types::{
    AreaDefinition, BeaconDefinition, BossChoiceDefinition, ChessboardDefinition,
    DifficultySegmentDefinition, DomainDefinition, MapColumnDefinition, MapEdgeDefinition,
    MapNodeDefinition, PlaneDefinition, ProfileDefinition, RoomDefinition,
};

pub(crate) const EXPECTED_STRUCTURAL_ROWS: usize = 6_716;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SwarmDisasterEntryArea {
    pub(super) id: u32,
    pub(super) difficulty: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SwarmDisasterStructuralErrorKind {
    Metadata,
    Identifier,
    Denominator,
    Duplicate,
    Reference,
    Graph,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterStructuralError {
    pub(crate) kind: SwarmDisasterStructuralErrorKind,
    pub(crate) key: Box<str>,
}

impl core::fmt::Display for SwarmDisasterStructuralError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Swarm Disaster structural catalog error {:?}: {}",
            self.kind, self.key
        )
    }
}

impl std::error::Error for SwarmDisasterStructuralError {}

#[derive(Debug)]
pub(crate) struct SwarmDisasterStructuralCatalog {
    bundle: SwarmDisasterBundleSummary,
    profiles: Box<[ProfileDefinition]>,
    areas: Box<[AreaDefinition]>,
    difficulty_segments: Box<[DifficultySegmentDefinition]>,
    planes: Box<[PlaneDefinition]>,
    chessboards: Box<[ChessboardDefinition]>,
    columns: Box<[MapColumnDefinition]>,
    nodes: Box<[MapNodeDefinition]>,
    edges: Box<[MapEdgeDefinition]>,
    rooms: Box<[RoomDefinition]>,
    domains: Box<[DomainDefinition]>,
    beacons: Box<[BeaconDefinition]>,
    boss_choices: Box<[BossChoiceDefinition]>,
}

impl SwarmDisasterStructuralCatalog {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, UniverseCatalogLoadError> {
        let (bundle, transport) = load_validated_swarm_disaster_bundle(bytes)?;
        lower::lower(bundle, &transport).map_err(|error| {
            UniverseCatalogLoadError::new(
                match error.kind {
                    SwarmDisasterStructuralErrorKind::Reference
                    | SwarmDisasterStructuralErrorKind::Graph => {
                        UniverseCatalogLoadErrorKind::InvalidGraph
                    }
                    _ => UniverseCatalogLoadErrorKind::InvalidDefinition,
                },
                error.to_string(),
            )
        })
    }

    fn row_count(&self) -> usize {
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

    pub(crate) const fn bundle_summary(&self) -> SwarmDisasterBundleSummary {
        self.bundle
    }

    pub(super) fn contains_chessboard_id(&self, id: u32) -> bool {
        self.chessboards.iter().any(|row| row.id.0 == id)
    }

    pub(super) fn contains_domain_id(&self, id: u32) -> bool {
        self.domains.iter().any(|row| row.id.0 == id)
    }

    pub(super) fn contains_area_id(&self, id: u32) -> bool {
        self.areas.iter().any(|row| row.id.0 == id)
    }

    pub(super) fn contains_area_key(&self, key: &str) -> bool {
        self.areas.iter().any(|row| row.stable_key.as_ref() == key)
    }

    pub(super) fn contains_room_key(&self, key: &str) -> bool {
        self.rooms.iter().any(|row| row.stable_key.as_ref() == key)
    }

    pub(super) fn contains_boss_choice_key(&self, key: &str) -> bool {
        self.boss_choices
            .iter()
            .any(|row| row.stable_key.as_ref() == key)
    }

    pub(super) fn contains_beacon_key(&self, key: &str) -> bool {
        self.beacons
            .iter()
            .any(|row| row.stable_key.as_ref() == key)
    }

    pub(super) fn contains_difficulty_key(&self, key: &str) -> bool {
        let Some(value) = key.strip_prefix("Difficulty_") else {
            return false;
        };
        value
            .parse::<u8>()
            .is_ok_and(|difficulty| self.areas.iter().any(|row| row.difficulty == difficulty))
    }

    pub(super) fn entry_area(&self, key: &str) -> Option<SwarmDisasterEntryArea> {
        self.areas
            .iter()
            .find(|row| row.stable_key.as_ref() == key && row.kind == types::AreaKind::Formal)
            .map(|row| SwarmDisasterEntryArea {
                id: row.id.0,
                difficulty: row.difficulty,
            })
    }

    pub(super) fn has_runtime_profile(&self) -> bool {
        self.profiles
            .iter()
            .any(|row| row.stable_key.as_ref() == "swarm-disaster.profile.v1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../../config/swarm-disaster-generated/config.sora");

    #[test]
    fn lowers_all_structural_rows_and_validates_graph_closure() {
        let catalog = SwarmDisasterStructuralCatalog::load(BUNDLE).unwrap();
        assert_eq!(catalog.bundle.table_count(), 65);
        assert_eq!(catalog.profiles.len(), 4);
        assert_eq!(catalog.areas.len(), 8);
        assert_eq!(catalog.difficulty_segments.len(), 20);
        assert_eq!(catalog.planes.len(), 11);
        assert_eq!(catalog.chessboards.len(), 101);
        assert_eq!(catalog.columns.len(), 1_109);
        assert_eq!(catalog.nodes.len(), 1_991);
        assert_eq!(catalog.edges.len(), 2_593);
        assert_eq!(catalog.rooms.len(), 861);
        assert_eq!(catalog.domains.len(), 12);
        assert_eq!(catalog.beacons.len(), 4);
        assert_eq!(catalog.boss_choices.len(), 2);
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
