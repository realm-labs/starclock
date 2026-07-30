//! Immutable shared and mode-owned content catalogs for Gold and Gears.

mod lower;
mod types;
mod validate;

use crate::{
    gold_gears_catalog::{
        GoldAndGearsBundleLoadError, GoldAndGearsBundleSummary, load_gold_and_gears_bundle,
    },
    gold_gears_content::types::{
        AdventureOutcome, Blessing, BlessingLevel, BlockCreateRule, CatalogCoverage, Curio,
        CurioState, EncounterGroup, EncounterWave, EnemySlot, MapEvent, MechanicRule, Occurrence,
        OccurrenceChoice, OccurrenceVariant, Service, StableIndexRow,
    },
};

pub(crate) const EXPECTED_CONTENT_ROWS: usize = 20_056;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoldAndGearsContentErrorKind {
    Bundle,
    Metadata,
    Denominator,
    Duplicate,
    Reference,
    Json,
    SharedIdentity,
    Coverage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoldAndGearsContentError {
    pub(crate) kind: GoldAndGearsContentErrorKind,
    pub(crate) key: Box<str>,
}

impl core::fmt::Display for GoldAndGearsContentError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Gold and Gears content catalog error {:?}: {}",
            self.kind, self.key
        )
    }
}

impl std::error::Error for GoldAndGearsContentError {}

impl From<GoldAndGearsBundleLoadError> for GoldAndGearsContentError {
    fn from(value: GoldAndGearsBundleLoadError) -> Self {
        Self {
            kind: GoldAndGearsContentErrorKind::Bundle,
            key: value.to_string().into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct GoldAndGearsContentCatalog {
    pub(crate) bundle: GoldAndGearsBundleSummary,
    blessings: Box<[Blessing]>,
    blessing_levels: Box<[BlessingLevel]>,
    curios: Box<[Curio]>,
    curio_states: Box<[CurioState]>,
    occurrences: Box<[Occurrence]>,
    occurrence_variants: Box<[OccurrenceVariant]>,
    occurrence_choices: Box<[OccurrenceChoice]>,
    services: Box<[Service]>,
    adventure_outcomes: Box<[AdventureOutcome]>,
    encounter_groups: Box<[EncounterGroup]>,
    encounter_waves: Box<[EncounterWave]>,
    enemy_slots: Box<[EnemySlot]>,
    map_events: Box<[MapEvent]>,
    block_create_rules: Box<[BlockCreateRule]>,
    mechanic_rules: Box<[MechanicRule]>,
    source_records: Box<[StableIndexRow]>,
    coverage: Box<[CatalogCoverage]>,
    research_gaps: Box<[StableIndexRow]>,
    gap_affected_records: Box<[StableIndexRow]>,
    review_fixtures: Box<[StableIndexRow]>,
    pack_index: Box<[StableIndexRow]>,
}

impl GoldAndGearsContentCatalog {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, GoldAndGearsContentError> {
        let (bundle, transport) = load_gold_and_gears_bundle(bytes)?;
        lower::lower(bundle, &transport)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.blessings.len()
            + self.blessing_levels.len()
            + self.curios.len()
            + self.curio_states.len()
            + self.occurrences.len()
            + self.occurrence_variants.len()
            + self.occurrence_choices.len()
            + self.services.len()
            + self.adventure_outcomes.len()
            + self.encounter_groups.len()
            + self.encounter_waves.len()
            + self.enemy_slots.len()
            + self.map_events.len()
            + self.block_create_rules.len()
            + self.mechanic_rules.len()
            + self.source_records.len()
            + self.coverage.len()
            + self.research_gaps.len()
            + self.gap_affected_records.len()
            + self.review_fixtures.len()
            + self.pack_index.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

    #[test]
    fn lowers_all_remaining_tables_and_closes_cross_catalog_references() {
        let catalog = GoldAndGearsContentCatalog::load(BUNDLE).unwrap();
        let counts = [
            catalog.blessings.len(),
            catalog.blessing_levels.len(),
            catalog.curios.len(),
            catalog.curio_states.len(),
            catalog.occurrences.len(),
            catalog.occurrence_variants.len(),
            catalog.occurrence_choices.len(),
            catalog.services.len(),
            catalog.adventure_outcomes.len(),
            catalog.encounter_groups.len(),
            catalog.encounter_waves.len(),
            catalog.enemy_slots.len(),
            catalog.map_events.len(),
            catalog.block_create_rules.len(),
            catalog.mechanic_rules.len(),
            catalog.source_records.len(),
            catalog.coverage.len(),
            catalog.research_gaps.len(),
            catalog.gap_affected_records.len(),
            catalog.review_fixtures.len(),
            catalog.pack_index.len(),
        ];
        assert_eq!(
            counts,
            [
                162, 324, 80, 80, 62, 65, 257, 15, 8, 181, 478, 1_513, 332, 1_091, 1_224, 9_082,
                42, 16, 5_025, 18, 1
            ]
        );
        assert_eq!(catalog.row_count(), EXPECTED_CONTENT_ROWS);
    }
}
