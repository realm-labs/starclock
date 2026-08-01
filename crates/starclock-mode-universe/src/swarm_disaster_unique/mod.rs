//! Immutable catalog for Swarm Disaster mode-unique systems.

pub(super) mod entry_access;
mod lower;
mod types;
mod validate;

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_catalog::{SwarmDisasterBundleSummary, load_validated_swarm_disaster_bundle},
};
use types::*;

pub(crate) const EXPECTED_UNIQUE_ROWS: usize = 772;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SwarmDisasterUniqueErrorKind {
    Metadata,
    Identifier,
    Denominator,
    Duplicate,
    Reference,
    Ordering,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterUniqueError {
    pub(crate) kind: SwarmDisasterUniqueErrorKind,
    pub(crate) key: Box<str>,
}

impl core::fmt::Display for SwarmDisasterUniqueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Swarm Disaster unique catalog error {:?}: {}",
            self.kind, self.key
        )
    }
}

impl std::error::Error for SwarmDisasterUniqueError {}

#[derive(Debug)]
pub(crate) struct SwarmDisasterUniqueCatalog {
    bundle: SwarmDisasterBundleSummary,
    countdown: Box<[CountdownDefinition]>,
    boss_decay_levels: Box<[BossDecayDefinition]>,
    audience_paths: Box<[AudiencePathDefinition]>,
    audience_dice: Box<[AudienceDieDefinition]>,
    dice_rarities: Box<[DiceRarityDefinition]>,
    dice_faces: Box<[DiceFaceDefinition]>,
    dice_targets: Box<[DiceTargetDefinition]>,
    dice_controls: Box<[DiceControlDefinition]>,
    communing_choices: Box<[CommuningChoiceDefinition]>,
    communing_dimensions: Box<[CommuningDimensionDefinition]>,
    point_adjustments: Box<[PointAdjustmentDefinition]>,
    trail_nodes: Box<[TrailNodeDefinition]>,
    trail_prerequisites: Box<[TrailPrerequisiteDefinition]>,
    trail_effects: Box<[TrailEffectDefinition]>,
    cabinets: Box<[CabinetDefinition]>,
    objectives: Box<[ObjectiveDefinition]>,
    finish_conditions: Box<[FinishDefinition]>,
    unlocks: Box<[UnlockDefinition]>,
    chapters: Box<[ChapterDefinition]>,
    bonuses: Box<[BonusDefinition]>,
    paths: Box<[PathDefinition]>,
    path_boosts: Box<[PathBoostDefinition]>,
    resonances: Box<[ResonanceDefinition]>,
    interplays: Box<[InterplayDefinition]>,
}

impl SwarmDisasterUniqueCatalog {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, UniverseCatalogLoadError> {
        let (bundle, transport) = load_validated_swarm_disaster_bundle(bytes)?;
        lower::lower(bundle, &transport).map_err(|error| {
            UniverseCatalogLoadError::new(
                match error.kind {
                    SwarmDisasterUniqueErrorKind::Reference
                    | SwarmDisasterUniqueErrorKind::Ordering => {
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

    pub(super) fn contains_audience_die_id(&self, id: u32) -> bool {
        self.audience_dice.iter().any(|row| row.id.0 == id)
    }

    pub(super) fn contains_shared_path(&self, key: &str) -> bool {
        self.paths.iter().any(|row| row.shared_path.as_ref() == key)
    }

    pub(super) fn contains_shared_resonance(&self, key: &str) -> bool {
        self.resonances
            .iter()
            .any(|row| row.shared_resonance.as_ref() == key)
    }

    fn row_count(&self) -> usize {
        self.countdown.len()
            + self.boss_decay_levels.len()
            + self.audience_paths.len()
            + self.audience_dice.len()
            + self.dice_rarities.len()
            + self.dice_faces.len()
            + self.dice_targets.len()
            + self.dice_controls.len()
            + self.communing_choices.len()
            + self.communing_dimensions.len()
            + self.point_adjustments.len()
            + self.trail_nodes.len()
            + self.trail_prerequisites.len()
            + self.trail_effects.len()
            + self.cabinets.len()
            + self.objectives.len()
            + self.finish_conditions.len()
            + self.unlocks.len()
            + self.chapters.len()
            + self.bonuses.len()
            + self.paths.len()
            + self.path_boosts.len()
            + self.resonances.len()
            + self.interplays.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../../config/swarm-disaster-generated/config.sora");

    #[test]
    fn lowers_all_unique_rows_with_closed_references() {
        let catalog = SwarmDisasterUniqueCatalog::load(BUNDLE).unwrap();
        assert_eq!(catalog.countdown.len(), 1);
        assert_eq!(catalog.boss_decay_levels.len(), 42);
        assert_eq!(catalog.audience_paths.len(), 8);
        assert_eq!(catalog.audience_dice.len(), 8);
        assert_eq!(catalog.dice_rarities.len(), 3);
        assert_eq!(catalog.dice_faces.len(), 42);
        assert_eq!(catalog.dice_targets.len(), 42);
        assert_eq!(catalog.dice_controls.len(), 4);
        assert_eq!(catalog.communing_choices.len(), 21);
        assert_eq!(catalog.communing_dimensions.len(), 7);
        assert_eq!(catalog.point_adjustments.len(), 55);
        assert_eq!(catalog.trail_nodes.len(), 63);
        assert_eq!(catalog.trail_prerequisites.len(), 56);
        assert_eq!(catalog.trail_effects.len(), 63);
        assert_eq!(catalog.cabinets.len(), 31);
        assert_eq!(catalog.objectives.len(), 31);
        assert_eq!(catalog.finish_conditions.len(), 102);
        assert_eq!(catalog.unlocks.len(), 110);
        assert_eq!(catalog.chapters.len(), 13);
        assert_eq!(catalog.bonuses.len(), 6);
        assert_eq!(catalog.paths.len(), 8);
        assert_eq!(catalog.path_boosts.len(), 8);
        assert_eq!(catalog.resonances.len(), 32);
        assert_eq!(catalog.interplays.len(), 16);
        assert_eq!(catalog.row_count(), EXPECTED_UNIQUE_ROWS);
    }

    #[test]
    fn canonical_scalars_reject_float_and_noncanonical_text() {
        assert!(lower::scalar("-1", "test").is_ok());
        assert!(lower::scalar("0.04", "test").is_ok());
        assert!(lower::scalar("4e-2", "test").is_err());
        assert!(lower::scalar("0.040", "test").is_err());
    }
}
