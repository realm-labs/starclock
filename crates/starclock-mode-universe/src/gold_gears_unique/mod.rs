//! Immutable catalogs for Gold and Gears mode-unique mechanics.

mod cognition;
mod dice;
mod lower;
mod progression;
mod support;
mod types;
mod validate;

use crate::{
    gold_gears_catalog::{
        GoldAndGearsBundleLoadError, GoldAndGearsBundleSummary, load_gold_and_gears_bundle,
    },
    gold_gears_unique::types::{
        CognitionRange, ConundrumLevel, DiceCategory, DiceDefinition, DiceFace, DiceFaceTag,
        DicePathValue, DiceSlot, Extrapolation, Interplay, KnowledgeRule, ModeConstant, NeuralNode,
        PathBoost, PathDefinition, Resonance, Secret, TrailblazeBonus,
    },
};

pub(crate) const EXPECTED_UNIQUE_ROWS: usize = 462;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GoldAndGearsUniqueErrorKind {
    Bundle,
    Metadata,
    Identifier,
    Denominator,
    Duplicate,
    Reference,
    Ordering,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GoldAndGearsUniqueError {
    pub(crate) kind: GoldAndGearsUniqueErrorKind,
    pub(crate) key: Box<str>,
}

impl core::fmt::Display for GoldAndGearsUniqueError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Gold and Gears unique catalog error {:?}: {}",
            self.kind, self.key
        )
    }
}

impl std::error::Error for GoldAndGearsUniqueError {}

impl From<GoldAndGearsBundleLoadError> for GoldAndGearsUniqueError {
    fn from(value: GoldAndGearsBundleLoadError) -> Self {
        Self {
            kind: GoldAndGearsUniqueErrorKind::Bundle,
            key: value.to_string().into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct GoldAndGearsUniqueCatalog {
    pub(crate) bundle: GoldAndGearsBundleSummary,
    pub(crate) cognition_ranges: Box<[CognitionRange]>,
    pub(crate) secrets: Box<[Secret]>,
    pub(crate) constants: Box<[ModeConstant]>,
    pub(crate) dice: Box<[DiceDefinition]>,
    pub(crate) dice_categories: Box<[DiceCategory]>,
    pub(crate) dice_path_values: Box<[DicePathValue]>,
    pub(crate) dice_slots: Box<[DiceSlot]>,
    pub(crate) dice_faces: Box<[DiceFace]>,
    pub(crate) dice_face_tags: Box<[DiceFaceTag]>,
    pub(crate) knowledge_rules: Box<[KnowledgeRule]>,
    pub(crate) neural_nodes: Box<[NeuralNode]>,
    pub(crate) conundrum_levels: Box<[ConundrumLevel]>,
    pub(crate) trailblaze_bonuses: Box<[TrailblazeBonus]>,
    pub(crate) paths: Box<[PathDefinition]>,
    pub(crate) path_boosts: Box<[PathBoost]>,
    pub(crate) resonances: Box<[Resonance]>,
    pub(crate) extrapolations: Box<[Extrapolation]>,
    pub(crate) interplays: Box<[Interplay]>,
}

impl GoldAndGearsUniqueCatalog {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, GoldAndGearsUniqueError> {
        let (bundle, transport) = load_gold_and_gears_bundle(bytes)?;
        lower::lower(bundle, &transport)
    }

    pub(crate) fn row_count(&self) -> usize {
        self.cognition_ranges.len()
            + self.secrets.len()
            + self.constants.len()
            + self.dice.len()
            + self.dice_categories.len()
            + self.dice_path_values.len()
            + self.dice_slots.len()
            + self.dice_faces.len()
            + self.dice_face_tags.len()
            + self.knowledge_rules.len()
            + self.neural_nodes.len()
            + self.conundrum_levels.len()
            + self.trailblaze_bonuses.len()
            + self.paths.len()
            + self.path_boosts.len()
            + self.resonances.len()
            + self.extrapolations.len()
            + self.interplays.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

    #[test]
    fn lowers_all_unique_catalog_rows_with_closed_references() {
        let catalog = GoldAndGearsUniqueCatalog::load(BUNDLE).unwrap();
        assert_eq!(catalog.cognition_ranges.len(), 13);
        assert_eq!(catalog.secrets.len(), 20);
        assert_eq!(catalog.constants.len(), 22);
        assert_eq!(catalog.dice.len(), 12);
        assert_eq!(catalog.dice_categories.len(), 4);
        assert_eq!(catalog.dice_path_values.len(), 108);
        assert_eq!(catalog.dice_slots.len(), 6);
        assert_eq!(catalog.dice_faces.len(), 80);
        assert_eq!(catalog.dice_face_tags.len(), 10);
        assert_eq!(catalog.knowledge_rules.len(), 22);
        assert_eq!(catalog.neural_nodes.len(), 40);
        assert_eq!(catalog.conundrum_levels.len(), 12);
        assert_eq!(catalog.trailblaze_bonuses.len(), 5);
        assert_eq!(catalog.paths.len(), 9);
        assert_eq!(catalog.path_boosts.len(), 9);
        assert_eq!(catalog.resonances.len(), 36);
        assert_eq!(catalog.extrapolations.len(), 36);
        assert_eq!(catalog.interplays.len(), 18);
        assert_eq!(catalog.row_count(), EXPECTED_UNIQUE_ROWS);
    }

    #[test]
    fn canonical_scalars_reject_float_and_noncanonical_text() {
        assert!(support::scalar("-40", "test").is_ok());
        assert!(support::scalar("0.02", "test").is_ok());
        assert!(support::scalar("2e-2", "test").is_err());
        assert!(support::scalar("0.020", "test").is_err());
    }
}
