use crate::{gold_gears_catalog::GoldAndGearsBundleSummary, gold_gears_generated::SoraConfig};

use super::{
    GoldAndGearsUniqueCatalog, GoldAndGearsUniqueError, cognition, dice, progression, validate,
};

pub(super) fn lower(
    bundle: GoldAndGearsBundleSummary,
    source: &SoraConfig,
) -> Result<GoldAndGearsUniqueCatalog, GoldAndGearsUniqueError> {
    let catalog = GoldAndGearsUniqueCatalog {
        bundle,
        cognition_ranges: collect(
            source
                .gold_gears_cognition_range()
                .ordered_rows()
                .map(cognition::cognition_range),
        )?,
        secrets: collect(
            source
                .gold_gears_secret()
                .ordered_rows()
                .map(cognition::secret),
        )?,
        constants: collect(
            source
                .gold_gears_mode_constant()
                .ordered_rows()
                .map(cognition::mode_constant),
        )?,
        dice: collect(
            source
                .gold_gears_dice_definition()
                .ordered_rows()
                .map(dice::definition),
        )?,
        dice_categories: collect(
            source
                .gold_gears_dice_category()
                .ordered_rows()
                .map(dice::category),
        )?,
        dice_path_values: collect(
            source
                .gold_gears_dice_path_value()
                .ordered_rows()
                .map(dice::path_value),
        )?,
        dice_slots: collect(source.gold_gears_dice_slot().ordered_rows().map(dice::slot))?,
        dice_faces: collect(source.gold_gears_dice_face().ordered_rows().map(dice::face))?,
        dice_face_tags: collect(
            source
                .gold_gears_dice_face_tag()
                .ordered_rows()
                .map(dice::face_tag),
        )?,
        knowledge_rules: collect(
            source
                .gold_gears_knowledge_rule()
                .ordered_rows()
                .map(dice::knowledge),
        )?,
        neural_nodes: collect(
            source
                .gold_gears_neural_network()
                .ordered_rows()
                .map(progression::neural),
        )?,
        conundrum_levels: collect(
            source
                .gold_gears_conundrum_level()
                .ordered_rows()
                .map(progression::conundrum),
        )?,
        trailblaze_bonuses: collect(
            source
                .gold_gears_trailblaze_bonus()
                .ordered_rows()
                .map(progression::trailblaze_bonus),
        )?,
        paths: collect(
            source
                .gold_gears_path()
                .ordered_rows()
                .map(progression::path),
        )?,
        path_boosts: collect(
            source
                .gold_gears_path_boost()
                .ordered_rows()
                .map(progression::path_boost),
        )?,
        resonances: collect(
            source
                .gold_gears_resonance()
                .ordered_rows()
                .map(progression::resonance),
        )?,
        extrapolations: collect(
            source
                .gold_gears_resonance_extrapolation()
                .ordered_rows()
                .map(progression::extrapolation),
        )?,
        interplays: collect(
            source
                .gold_gears_resonance_interplay()
                .ordered_rows()
                .map(progression::interplay),
        )?,
    };
    validate::validate(&catalog)?;
    Ok(catalog)
}

fn collect<T>(
    values: impl Iterator<Item = Result<T, GoldAndGearsUniqueError>>,
) -> Result<Box<[T]>, GoldAndGearsUniqueError> {
    values
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
