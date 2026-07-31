//! Immutable shared and mode-owned content catalogs for Gold and Gears.

mod lower;
pub(crate) mod types;
mod validate;

use crate::{
    gold_gears_catalog::{
        GoldAndGearsBundleLoadError, GoldAndGearsBundleSummary, load_gold_and_gears_bundle,
    },
    gold_gears_content::types::{
        AdventureOutcome, Blessing, BlessingLevel, CatalogCoverage, Curio, CurioState,
        EncounterGroup, EncounterWave, EnemySlot, MechanicRule, Occurrence, OccurrenceChoice,
        OccurrenceVariant, Service, StableIndexRow,
    },
};
pub(crate) use types::{BlockCreateRule, MapEvent, MapEventEffect, MapEventTrigger};

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
    pub(crate) blessings: Box<[Blessing]>,
    pub(crate) blessing_levels: Box<[BlessingLevel]>,
    pub(crate) curios: Box<[Curio]>,
    pub(crate) curio_states: Box<[CurioState]>,
    pub(crate) occurrences: Box<[Occurrence]>,
    pub(crate) occurrence_variants: Box<[OccurrenceVariant]>,
    pub(crate) occurrence_choices: Box<[OccurrenceChoice]>,
    pub(crate) services: Box<[Service]>,
    pub(crate) adventure_outcomes: Box<[AdventureOutcome]>,
    encounter_groups: Box<[EncounterGroup]>,
    encounter_waves: Box<[EncounterWave]>,
    enemy_slots: Box<[EnemySlot]>,
    pub(crate) map_events: Box<[MapEvent]>,
    pub(crate) block_create_rules: Box<[BlockCreateRule]>,
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

    pub(crate) fn review_fixture_keys(&self) -> impl ExactSizeIterator<Item = &str> {
        self.review_fixtures.iter().map(|row| row.key.as_str())
    }

    pub(crate) fn encounter_fixture_shape(&self, group_key: &str) -> Option<(usize, usize)> {
        let group = self
            .encounter_groups
            .iter()
            .find(|group| group.key.as_str() == group_key)?;
        let waves = self
            .encounter_waves
            .iter()
            .filter(|wave| wave.group_id == group.id)
            .collect::<Vec<_>>();
        let slot_count = waves.iter().map(|wave| wave.slots.len()).sum();
        let boss_choice_count = waves
            .iter()
            .flat_map(|wave| {
                self.enemy_slots
                    .iter()
                    .filter(move |slot| slot.wave_id == wave.id)
            })
            .map(|slot| slot.boss_choices.len())
            .sum();
        Some((slot_count, boss_choice_count))
    }
}

#[cfg(test)]
mod tests;
