//! Immutable content, encounter, rule and coverage catalogs for Swarm Disaster.

pub(crate) mod coverage_access;
pub(crate) mod encounter_access;
pub(crate) mod interaction_access;
pub(crate) mod inventory_access;
mod lower;
pub(super) mod map_access;
pub(crate) mod mechanic_access;
pub(crate) mod semantic_access;
pub(crate) mod types;
mod validate;

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_catalog::{SwarmDisasterBundleSummary, load_validated_swarm_disaster_bundle},
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};
use types::*;

pub(crate) const EXPECTED_CONTENT_ROWS: usize = 25_892;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SwarmDisasterContentErrorKind {
    Metadata,
    Identifier,
    Denominator,
    Duplicate,
    Reference,
    Ordering,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterContentError {
    pub(crate) kind: SwarmDisasterContentErrorKind,
    pub(crate) key: Box<str>,
}

impl core::fmt::Display for SwarmDisasterContentError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Swarm Disaster content catalog error {:?}: {}",
            self.kind, self.key
        )
    }
}

impl std::error::Error for SwarmDisasterContentError {}

#[derive(Debug)]
pub(crate) struct SwarmDisasterContentCatalog {
    bundle: SwarmDisasterBundleSummary,
    map_events: Box<[MapEventDefinition]>,
    block_rules: Box<[BlockRuleDefinition]>,
    topology_consequences: Box<[TopologyConsequenceDefinition]>,
    blessings: Box<[BlessingDefinition]>,
    blessing_levels: Box<[BlessingLevelDefinition]>,
    pool_memberships: Box<[PoolMembershipDefinition]>,
    curios: Box<[CurioDefinition]>,
    curio_states: Box<[CurioStateDefinition]>,
    curio_rules: Box<[CurioRuleDefinition]>,
    occurrences: Box<[OccurrenceDefinition]>,
    occurrence_variants: Box<[OccurrenceVariantDefinition]>,
    occurrence_choices: Box<[OccurrenceChoiceDefinition]>,
    services: Box<[ServiceDefinition]>,
    adventure_outcomes: Box<[AdventureOutcomeDefinition]>,
    currencies: Box<[CurrencyDefinition]>,
    service_rules: Box<[ServiceRuleDefinition]>,
    encounter_groups: Box<[EncounterGroupDefinition]>,
    encounter_waves: Box<[EncounterWaveDefinition]>,
    enemy_slots: Box<[EnemySlotDefinition]>,
    boss_pools: Box<[BossPoolDefinition]>,
    mechanic_rules: Box<[MechanicRuleDefinition]>,
    review_fixtures: Box<[ReviewFixtureDefinition]>,
    audit: AuditCatalogSummary,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn load(
        bytes: &[u8],
        structural: &SwarmDisasterStructuralCatalog,
        unique: &SwarmDisasterUniqueCatalog,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let (bundle, transport) = load_validated_swarm_disaster_bundle(bytes)?;
        lower::lower(bundle, &transport, structural, unique).map_err(|error| {
            UniverseCatalogLoadError::new(
                match error.kind {
                    SwarmDisasterContentErrorKind::Reference
                    | SwarmDisasterContentErrorKind::Ordering => {
                        UniverseCatalogLoadErrorKind::InvalidReference
                    }
                    _ => UniverseCatalogLoadErrorKind::InvalidDefinition,
                },
                error.to_string(),
            )
        })
    }

    pub(crate) const fn bundle_summary(&self) -> SwarmDisasterBundleSummary {
        self.bundle
    }

    fn row_count(&self) -> usize {
        self.map_events.len()
            + self.block_rules.len()
            + self.topology_consequences.len()
            + self.blessings.len()
            + self.blessing_levels.len()
            + self.pool_memberships.len()
            + self.curios.len()
            + self.curio_states.len()
            + self.curio_rules.len()
            + self.occurrences.len()
            + self.occurrence_variants.len()
            + self.occurrence_choices.len()
            + self.services.len()
            + self.adventure_outcomes.len()
            + self.currencies.len()
            + self.service_rules.len()
            + self.encounter_groups.len()
            + self.encounter_waves.len()
            + self.enemy_slots.len()
            + self.boss_pools.len()
            + self.mechanic_rules.len()
            + self.audit.source_records
            + self.audit.coverage_rows
            + self.audit.research_gaps
            + self.audit.affected_rows
            + self.audit.fixtures
            + self.audit.receipts
            + self.audit.manifest_rows
            + self.audit.pack_rows
    }

    pub(super) fn initial_currency(&self) -> Option<i64> {
        self.currencies.first()?.initial_value.parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../../config/swarm-disaster-generated/config.sora");

    #[test]
    fn lowers_remaining_tables_with_cross_catalog_closure() {
        let structural = SwarmDisasterStructuralCatalog::load(BUNDLE).unwrap();
        let unique = SwarmDisasterUniqueCatalog::load(BUNDLE).unwrap();
        let catalog = SwarmDisasterContentCatalog::load(BUNDLE, &structural, &unique).unwrap();
        assert_eq!(catalog.row_count(), EXPECTED_CONTENT_ROWS);
        assert_eq!(catalog.mechanic_rules.len(), 23);
        assert_eq!(catalog.audit.frozen_obligations, 6_963);
        assert_eq!(catalog.audit.fixture_families, 23);
    }

    #[test]
    fn stable_keys_and_canonical_scalars_reject_invalid_text() {
        assert!(lower::stable("swarm-disaster.test", "test").is_ok());
        assert!(lower::stable("contains space", "test").is_err());
        assert!(lower::scalar("0.04", "test").is_ok());
        assert!(lower::scalar("4e-2", "test").is_err());
    }
}
