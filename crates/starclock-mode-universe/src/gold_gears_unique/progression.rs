use crate::gold_gears_generated::{
    gold_gears_conundrum_level::GoldGearsConundrumLevel,
    gold_gears_neural_network::GoldGearsNeuralNetwork, gold_gears_path::GoldGearsPath,
    gold_gears_path_boost::GoldGearsPathBoost, gold_gears_resonance::GoldGearsResonance,
    gold_gears_resonance_extrapolation::GoldGearsResonanceExtrapolation,
    gold_gears_resonance_interplay::GoldGearsResonanceInterplay,
    gold_gears_trailblaze_bonus::GoldGearsTrailblazeBonus,
};

use super::{
    GoldAndGearsUniqueError,
    support::{
        identity, json_text, nonnegative_u16, optional_texts, positive_u8, positive_u16,
        positive_u32, row, scalar, scalars, text, texts,
    },
    types::{
        ConundrumLevel, ConundrumLevelId, Extrapolation, ExtrapolationId, Interplay, InterplayId,
        NeuralNode, NeuralNodeId, PathBoost, PathBoostId, PathDefinition, PathId, Resonance,
        ResonanceId, TrailblazeBonus, TrailblazeBonusId,
    },
};

pub(super) fn neural(
    source: &GoldGearsNeuralNetwork,
) -> Result<NeuralNode, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "NeuralNetworkNode",
    )?;
    json_text(&source.quality_overrides_json, &source.stable_key)?;
    Ok(NeuralNode {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            NeuralNodeId,
        )?,
        topological_index: nonnegative_u16(source.topological_index, &source.stable_key)?,
        prerequisites: optional_texts(source.prerequisite_ids.as_deref(), &source.stable_key)?,
        next: optional_texts(source.next_ids.as_deref(), &source.stable_key)?,
        external_unlocks: optional_texts(
            source.external_unlock_ids.as_deref(),
            &source.stable_key,
        )?,
        costs_json: json_text(&source.costs_json, &source.stable_key)?,
        important: source.important,
        disposition: text(&source.disposition, &source.stable_key)?,
        effect_domain: text(&source.effect_domain, &source.stable_key)?,
        source_parameters_json: json_text(&source.source_parameters_json, &source.stable_key)?,
        effect_contributions_json: json_text(
            &source.effect_contributions_json,
            &source.stable_key,
        )?,
        rule_contribution: text(&source.rule_contribution_id, &source.stable_key)?,
    })
}

pub(super) fn conundrum(
    source: &GoldGearsConundrumLevel,
) -> Result<ConundrumLevel, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "ConundrumLevel",
    )?;
    json_text(&source.quality_overrides_json, &source.stable_key)?;
    Ok(ConundrumLevel {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            ConundrumLevelId,
        )?,
        source_type: text(&source.source_type, &source.stable_key)?,
        track: text(&source.track, &source.stable_key)?,
        level: positive_u8(source.level, &source.stable_key)?,
        track_cap: positive_u8(source.track_cap, &source.stable_key)?,
        total_cap: positive_u8(source.total_conundrum_cap, &source.stable_key)?,
        total_formula: text(&source.total_level_formula, &source.stable_key)?,
        unlock_requirement_json: json_text(&source.unlock_requirement_json, &source.stable_key)?,
        composition_mode: text(&source.composition_mode, &source.stable_key)?,
        active_contributions: texts(&source.active_contribution_ids, &source.stable_key)?,
        replaces_levels: optional_texts(source.replaces_level_ids.as_deref(), &source.stable_key)?,
        source_tag: nonnegative_u16(source.source_tag, &source.stable_key)?,
        source_sort: nonnegative_u16(source.source_sort, &source.stable_key)?,
        source_parameters_json: json_text(&source.source_parameters_json, &source.stable_key)?,
        effect_contributions_json: json_text(
            &source.effect_contributions_json,
            &source.stable_key,
        )?,
        rule_contribution: text(&source.rule_contribution_id, &source.stable_key)?,
    })
}

pub(super) fn trailblaze_bonus(
    source: &GoldGearsTrailblazeBonus,
) -> Result<TrailblazeBonus, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "TrailblazeBonus",
    )?;
    Ok(TrailblazeBonus {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            TrailblazeBonusId,
        )?,
        bonus_event: text(&source.bonus_event_id, &source.stable_key)?,
        effect_contributions_json: json_text(
            &source.effect_contributions_json,
            &source.stable_key,
        )?,
        rule_contribution: text(&source.rule_contribution_id, &source.stable_key)?,
    })
}

pub(super) fn path(source: &GoldGearsPath) -> Result<PathDefinition, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "Path",
    )?;
    Ok(PathDefinition {
        identity: identity(source.id, &source.stable_key, &source.source_id, PathId)?,
        sort: positive_u16(source.sort, &source.stable_key)?,
        buff_type: positive_u16(source.buff_type, &source.stable_key)?,
        shared_resonance_id: positive_u32(source.shared_resonance_id, &source.stable_key)?,
        shared_formation_ids: texts(&source.shared_formation_ids, &source.stable_key)?,
        path_boost: PathBoostId(positive_u32(source.path_boost_id, &source.stable_key)?),
        normal_event_group: text(&source.normal_battle_event_group, &source.stable_key)?,
        enhanced_event_group: text(&source.enhanced_battle_event_group, &source.stable_key)?,
    })
}

pub(super) fn path_boost(
    source: &GoldGearsPathBoost,
) -> Result<PathBoost, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "PathBoost",
    )?;
    Ok(PathBoost {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            PathBoostId,
        )?,
        path: PathId(positive_u32(source.path_id, &source.stable_key)?),
        aeon_source: text(&source.aeon_source_id, &source.stable_key)?,
        effect_type: text(&source.effect_type, &source.stable_key)?,
        ability_name: text(&source.ability_name, &source.stable_key)?,
        target_team: text(&source.target_team, &source.stable_key)?,
        target_property: text(&source.target_property, &source.stable_key)?,
        boost_stat: text(&source.boost_stat, &source.stable_key)?,
        stacking: text(&source.stacking, &source.stable_key)?,
        value_conversion: text(&source.source_value_conversion, &source.stable_key)?,
        dice_path_value_keys: texts(&source.dice_path_value_ids, &source.stable_key)?,
        allowed_increments: scalars(&source.allowed_increment_values, &source.stable_key)?,
        rule_contribution: text(&source.rule_contribution_id, &source.stable_key)?,
    })
}

pub(super) fn resonance(source: &GoldGearsResonance) -> Result<Resonance, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "Resonance",
    )?;
    Ok(Resonance {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            ResonanceId,
        )?,
        path: PathId(positive_u32(source.path_id, &source.stable_key)?),
        resonance_kind: text(&source.resonance_kind, &source.stable_key)?,
        threshold: nonnegative_u16(source.threshold, &source.stable_key)?,
        energy_max: scalar(&source.energy_max, &source.stable_key)?,
        initial_energy: scalar(&source.initial_energy, &source.stable_key)?,
        parameter_values_json: json_text(&source.parameter_values_json, &source.stable_key)?,
        mechanic_tags: optional_texts(source.mechanic_tags.as_deref(), &source.stable_key)?,
        source_modifier: text(&source.source_modifier_name, &source.stable_key)?,
        source_binding_type: text(&source.source_binding_type, &source.stable_key)?,
        source_binding_key: text(&source.source_binding_key, &source.stable_key)?,
        inherited_rule_ids: optional_texts(
            source.inherited_rule_ids.as_deref(),
            &source.stable_key,
        )?,
    })
}

pub(super) fn extrapolation(
    source: &GoldGearsResonanceExtrapolation,
) -> Result<Extrapolation, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "ResonanceExtrapolation",
    )?;
    json_text(&source.quality_overrides_json, &source.stable_key)?;
    Ok(Extrapolation {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            ExtrapolationId,
        )?,
        path: PathId(positive_u32(source.path_id, &source.stable_key)?),
        aeon_source: text(&source.aeon_source_id, &source.stable_key)?,
        buff_group: text(&source.buff_group_id, &source.stable_key)?,
        enhanced: source.enhanced,
        shared_resonance_id: positive_u32(source.shared_resonance_id, &source.stable_key)?,
        shared_resonance_kind: text(&source.shared_resonance_kind, &source.stable_key)?,
        battle_event_type: text(&source.source_battle_event_type, &source.stable_key)?,
        source_modifier: text(&source.source_modifier_name, &source.stable_key)?,
        source_binding_type: text(&source.source_binding_type, &source.stable_key)?,
        source_binding_key: text(&source.source_binding_key, &source.stable_key)?,
        source_parameters_json: json_text(&source.source_parameters_json, &source.stable_key)?,
        battle_scope: text(&source.battle_scope, &source.stable_key)?,
        controller_policy_json: json_text(&source.controller_policy_json, &source.stable_key)?,
        rule_contribution: text(&source.rule_contribution_id, &source.stable_key)?,
    })
}

pub(super) fn interplay(
    source: &GoldGearsResonanceInterplay,
) -> Result<Interplay, GoldAndGearsUniqueError> {
    row(
        &source.stable_key,
        &source.schema_revision,
        &source.kind,
        "ResonanceInterplay",
    )?;
    Ok(Interplay {
        identity: identity(
            source.id,
            &source.stable_key,
            &source.source_id,
            InterplayId,
        )?,
        main_path: PathId(positive_u32(source.main_path_id, &source.stable_key)?),
        sub_path: PathId(positive_u32(source.sub_path_id, &source.stable_key)?),
        main_threshold: positive_u16(source.main_blessing_threshold, &source.stable_key)?,
        sub_threshold: positive_u16(source.sub_blessing_threshold, &source.stable_key)?,
        buff_group: text(&source.buff_group_id, &source.stable_key)?,
        shared_maze_buff: text(&source.shared_maze_buff_id, &source.stable_key)?,
        source_modifier: text(&source.source_modifier_name, &source.stable_key)?,
        source_binding_type: text(&source.source_binding_type, &source.stable_key)?,
        source_binding_key: text(&source.source_binding_key, &source.stable_key)?,
        source_parameters_json: json_text(&source.source_parameters_json, &source.stable_key)?,
        rule_contribution: text(&source.rule_contribution_id, &source.stable_key)?,
    })
}
