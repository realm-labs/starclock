use crate::gold_gears_generated::{
    gold_gears_cognition_range::GoldGearsCognitionRange,
    gold_gears_mode_constant::GoldGearsModeConstant, gold_gears_secret::GoldGearsSecret,
};

use super::{
    GoldAndGearsUniqueError,
    support::{identity, json_text, optional_texts, positive_u8, row, scalar, text, texts},
    types::{CognitionRange, CognitionRangeId, ModeConstant, ModeConstantId, Secret, SecretId},
};

pub(super) fn cognition_range(
    source: &GoldGearsCognitionRange,
) -> Result<CognitionRange, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "CognitionRange",
    )?;
    Ok(CognitionRange {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            CognitionRangeId,
        )?,
        area_key: text(&source.area_stable_key, &source.stable_key)?,
        minimum: scalar(&source.minimum_cognition, &source.stable_key)?,
        maximum: scalar(&source.maximum_cognition, &source.stable_key)?,
        global_minimum: scalar(&source.global_minimum_cognition, &source.stable_key)?,
        global_maximum: scalar(&source.global_maximum_cognition, &source.stable_key)?,
        inclusive: source.bounds_inclusive,
        lifecycle_json: json_text(&source.lifecycle_json, &source.stable_key)?,
    })
}

pub(super) fn secret(source: &GoldGearsSecret) -> Result<Secret, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "SecretCondition",
    )?;
    Ok(Secret {
        identity: identity(source.id, &source.stable_key, &source.source_id, SecretId)?,
        area_key: text(&source.required_area_stable_key, &source.stable_key)?,
        area_source: text(&source.required_area_source_id, &source.stable_key)?,
        plane_layer: positive_u8(source.plane_layer, &source.stable_key)?,
        cognition_minimum: scalar(&source.minimum_cognition, &source.stable_key)?,
        cognition_maximum: scalar(&source.maximum_cognition, &source.stable_key)?,
        origin_minimum: text(&source.minimum_origin, &source.stable_key)?,
        origin_maximum: text(&source.maximum_origin, &source.stable_key)?,
        inclusive: source.bounds_inclusive,
        predecessors: optional_texts(source.predecessor_secret_ids.as_deref(), &source.stable_key)?,
        next: optional_texts(source.next_secret_ids.as_deref(), &source.stable_key)?,
        evaluation_boundary: text(&source.evaluation_boundary, &source.stable_key)?,
        condition_hash: text(&source.trigger_condition_hash, &source.stable_key)?,
        condition_digest: text(&source.trigger_condition_digest, &source.stable_key)?,
        terminal: source.terminal,
        lifecycle_policy: text(&source.lifecycle_policy_id, &source.stable_key)?,
    })
}

pub(super) fn mode_constant(
    source: &GoldGearsModeConstant,
) -> Result<ModeConstant, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "ModeConstant",
    )?;
    Ok(ModeConstant {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            ModeConstantId,
        )?,
        mechanical_role: text(&source.mechanical_role, &source.stable_key)?,
        value_kind: text(&source.value_kind, &source.stable_key)?,
        values: texts(&source.values, &source.stable_key)?,
    })
}
