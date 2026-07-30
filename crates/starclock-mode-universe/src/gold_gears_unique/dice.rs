use crate::gold_gears_generated::{
    gold_gears_dice_category::GoldGearsDiceCategory,
    gold_gears_dice_definition::GoldGearsDiceDefinition, gold_gears_dice_face::GoldGearsDiceFace,
    gold_gears_dice_face_tag::GoldGearsDiceFaceTag,
    gold_gears_dice_path_value::GoldGearsDicePathValue, gold_gears_dice_slot::GoldGearsDiceSlot,
    gold_gears_knowledge_rule::GoldGearsKnowledgeRule,
};

use super::{
    GoldAndGearsUniqueError,
    support::{
        identity, json_text, nonnegative_u8, nonnegative_u16, optional_text, optional_texts,
        optional_u8, positive_u8, positive_u16, positive_u32, row, scalar, text, texts,
    },
    types::{
        DiceCategory, DiceCategoryId, DiceDefinition, DiceFace, DiceFaceId, DiceFaceTag,
        DiceFaceTagId, DiceId, DicePathValue, DicePathValueId, DiceSlot, DiceSlotId, KnowledgeRule,
        KnowledgeRuleId,
    },
};

pub(super) fn category(
    source: &GoldGearsDiceCategory,
) -> Result<DiceCategory, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "DiceCategory",
    )?;
    Ok(DiceCategory {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            DiceCategoryId,
        )?,
        sort: positive_u16(source.sort, &source.stable_key)?,
    })
}

pub(super) fn definition(
    source: &GoldGearsDiceDefinition,
) -> Result<DiceDefinition, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "CustomDice",
    )?;
    json_text(&source.effect_parts_json, &source.stable_key)?;
    text(&source.dice_icon_path, &source.stable_key)?;
    Ok(DiceDefinition {
        identity: identity(source.id, &source.stable_key, &source.source_id, DiceId)?,
        sort: positive_u16(source.sort, &source.stable_key)?,
        category: DiceCategoryId(positive_u32(source.category_id, &source.stable_key)?),
        category_source: text(&source.category_source_id, &source.stable_key)?,
        initial_effects: optional_texts(
            source.initial_effect_extra_ids.as_deref(),
            &source.stable_key,
        )?,
        passive_effects: optional_texts(
            source.passive_effect_extra_ids.as_deref(),
            &source.stable_key,
        )?,
        available_by_default: source.available_by_default,
        unlock_id: optional_text(source.unlock_id.as_deref(), &source.stable_key)?,
        ultra_face_source: text(&source.default_ultra_surface_id, &source.stable_key)?,
        common_face_sources: texts(&source.default_common_surface_ids, &source.stable_key)?,
        default_face_sources: texts(&source.default_surface_ids, &source.stable_key)?,
        suggestive_face_sources: optional_texts(
            source.suggestive_surface_ids.as_deref(),
            &source.stable_key,
        )?,
        recommended_face_sources: optional_texts(
            source.recommended_surface_ids.as_deref(),
            &source.stable_key,
        )?,
    })
}

pub(super) fn path_value(
    source: &GoldGearsDicePathValue,
) -> Result<DicePathValue, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "DicePathValue",
    )?;
    Ok(DicePathValue {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            DicePathValueId,
        )?,
        dice: DiceId(positive_u32(source.dice_id, &source.stable_key)?),
        dice_source: text(&source.dice_source_id, &source.stable_key)?,
        path_key: text(&source.path_stable_key, &source.stable_key)?,
        path_source: text(&source.path_source_id, &source.stable_key)?,
        boost_stat: text(&source.boost_stat, &source.stable_key)?,
        trigger_interval: text(&source.trigger_interval, &source.stable_key)?,
        boost_value: scalar(&source.boost_value, &source.stable_key)?,
        boost_unit: text(&source.boost_value_unit, &source.stable_key)?,
        parameters: optional_texts(source.parameters.as_deref(), &source.stable_key)?,
    })
}

pub(super) fn slot(source: &GoldGearsDiceSlot) -> Result<DiceSlot, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "DiceSlot",
    )?;
    Ok(DiceSlot {
        identity: identity(source.id, &source.stable_key, &source.source_id, DiceSlotId)?,
        index: positive_u8(source.slot_index, &source.stable_key)?,
        base_max_rarity: positive_u8(source.base_max_rarity, &source.stable_key)?,
        extra_max_rarity: optional_u8(source.extra_max_rarity, &source.stable_key)?,
        upgraded_max_rarity: positive_u8(source.upgraded_max_rarity, &source.stable_key)?,
    })
}

pub(super) fn face(source: &GoldGearsDiceFace) -> Result<DiceFace, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "DiceFace",
    )?;
    text(&source.icon_path, &source.stable_key)?;
    text(&source.unlock_display_id, &source.stable_key)?;
    Ok(DiceFace {
        identity: identity(source.id, &source.stable_key, &source.source_id, DiceFaceId)?,
        sort: nonnegative_u16(source.sort, &source.stable_key)?,
        item_id: text(&source.item_id, &source.stable_key)?,
        rarity: positive_u8(source.rarity, &source.stable_key)?,
        activation_stage: nonnegative_u8(source.activation_stage, &source.stable_key)?,
        parameters: optional_texts(source.parameters.as_deref(), &source.stable_key)?,
        allowed_slot_keys: texts(&source.allowed_slot_ids, &source.stable_key)?,
        allowed_slot_sources: texts(&source.allowed_slot_source_ids, &source.stable_key)?,
        mechanical_codes: optional_texts(
            source.mechanical_tag_codes.as_deref(),
            &source.stable_key,
        )?,
        filter_tag_sources: optional_texts(source.filter_tag_ids.as_deref(), &source.stable_key)?,
        allowed_dice_keys: optional_texts(source.allowed_dice_ids.as_deref(), &source.stable_key)?,
        allowed_dice_sources: optional_texts(
            source.allowed_dice_source_ids.as_deref(),
            &source.stable_key,
        )?,
        universal_dice_eligibility: source.universal_dice_eligibility,
        no_target_behavior: text(&source.no_legal_target_behavior, &source.stable_key)?,
        target_policy_json: json_text(&source.target_resolution_policy_json, &source.stable_key)?,
    })
}

pub(super) fn face_tag(
    source: &GoldGearsDiceFaceTag,
) -> Result<DiceFaceTag, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "DiceFaceTag",
    )?;
    Ok(DiceFaceTag {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            DiceFaceTagId,
        )?,
        sort: positive_u16(source.sort, &source.stable_key)?,
        mechanical_code: text(&source.mechanical_code, &source.stable_key)?,
        replacement_condition: text(&source.mapping_replacement_condition, &source.stable_key)?,
    })
}

pub(super) fn knowledge(
    source: &GoldGearsKnowledgeRule,
) -> Result<KnowledgeRule, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "KnowledgeBinding",
    )?;
    Ok(KnowledgeRule {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            KnowledgeRuleId,
        )?,
        dice_face: DiceFaceId(positive_u32(source.dice_face_id, &source.stable_key)?),
        operation: text(&source.operation, &source.stable_key)?,
        trigger_boundary: text(&source.trigger_boundary, &source.stable_key)?,
        target_scope: text(&source.target_scope, &source.stable_key)?,
        selection_mode: text(&source.selection_mode, &source.stable_key)?,
        knowledge_access: text(&source.knowledge_access, &source.stable_key)?,
        parameters: optional_texts(source.parameters.as_deref(), &source.stable_key)?,
        activation_stage: nonnegative_u8(source.activation_stage, &source.stable_key)?,
        target_policy_json: json_text(&source.target_policy_json, &source.stable_key)?,
        simultaneous_policy_json: json_text(
            &source.simultaneous_resolution_policy_json,
            &source.stable_key,
        )?,
        dice_interactions_json: json_text(
            &source.custom_dice_interactions_json,
            &source.stable_key,
        )?,
    })
}
