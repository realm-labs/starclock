use crate::{
    swarm_disaster_catalog::SwarmDisasterBundleSummary,
    swarm_disaster_generated::{
        SoraConfig, swarm_disaster_audience_die::SwarmDisasterAudienceDie,
        swarm_disaster_audience_path::SwarmDisasterAudiencePath,
        swarm_disaster_boss_decay_level::SwarmDisasterBossDecayLevel,
        swarm_disaster_communing_choice::SwarmDisasterCommuningChoice,
        swarm_disaster_communing_dimension::SwarmDisasterCommuningDimension,
        swarm_disaster_communing_trail_node::SwarmDisasterCommuningTrailNode,
        swarm_disaster_countdown_disarray::SwarmDisasterCountdownDisarray,
        swarm_disaster_dice_face::SwarmDisasterDiceFace,
        swarm_disaster_dice_rarity::SwarmDisasterDiceRarity,
        swarm_disaster_dice_roll_control::SwarmDisasterDiceRollControl,
        swarm_disaster_dice_target_rule::SwarmDisasterDiceTargetRule,
        swarm_disaster_mechanical_chapter::SwarmDisasterMechanicalChapter,
        swarm_disaster_path::SwarmDisasterPath, swarm_disaster_path_boost::SwarmDisasterPathBoost,
        swarm_disaster_path_objective::SwarmDisasterPathObjective,
        swarm_disaster_pathstrider_cabinet::SwarmDisasterPathstriderCabinet,
        swarm_disaster_pathstrider_finish::SwarmDisasterPathstriderFinish,
        swarm_disaster_pathstrider_unlock::SwarmDisasterPathstriderUnlock,
        swarm_disaster_point_adjustment::SwarmDisasterPointAdjustment,
        swarm_disaster_resonance::SwarmDisasterResonance,
        swarm_disaster_resonance_interplay::SwarmDisasterResonanceInterplay,
        swarm_disaster_trail_effect::SwarmDisasterTrailEffect,
        swarm_disaster_trail_prerequisite::SwarmDisasterTrailPrerequisite,
        swarm_disaster_trailblaze_bonus::SwarmDisasterTrailblazeBonus,
    },
};

use super::{
    SwarmDisasterUniqueCatalog, SwarmDisasterUniqueError, SwarmDisasterUniqueErrorKind, types::*,
    validate,
};

const ROW_REVISION: &str = "starclock.swarm-disaster-row.v1";

pub(super) fn lower(
    bundle: SwarmDisasterBundleSummary,
    source: &SoraConfig,
) -> Result<SwarmDisasterUniqueCatalog, SwarmDisasterUniqueError> {
    let catalog = SwarmDisasterUniqueCatalog {
        bundle,
        countdown: source
            .swarm_disaster_countdown_disarray()
            .ordered_rows()
            .map(lower_countdown)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        boss_decay_levels: source
            .swarm_disaster_boss_decay_level()
            .ordered_rows()
            .map(lower_boss_decay)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        audience_paths: source
            .swarm_disaster_audience_path()
            .ordered_rows()
            .map(lower_audience_path)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        audience_dice: source
            .swarm_disaster_audience_die()
            .ordered_rows()
            .map(lower_audience_die)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        dice_rarities: source
            .swarm_disaster_dice_rarity()
            .ordered_rows()
            .map(lower_dice_rarity)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        dice_faces: source
            .swarm_disaster_dice_face()
            .ordered_rows()
            .map(lower_dice_face)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        dice_targets: source
            .swarm_disaster_dice_target_rule()
            .ordered_rows()
            .map(lower_dice_target)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        dice_controls: source
            .swarm_disaster_dice_roll_control()
            .ordered_rows()
            .map(lower_dice_control)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        communing_choices: source
            .swarm_disaster_communing_choice()
            .ordered_rows()
            .map(lower_communing_choice)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        communing_dimensions: source
            .swarm_disaster_communing_dimension()
            .ordered_rows()
            .map(lower_communing_dimension)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        point_adjustments: source
            .swarm_disaster_point_adjustment()
            .ordered_rows()
            .map(lower_point_adjustment)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        trail_nodes: source
            .swarm_disaster_communing_trail_node()
            .ordered_rows()
            .map(lower_trail_node)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        trail_prerequisites: source
            .swarm_disaster_trail_prerequisite()
            .ordered_rows()
            .map(lower_trail_prerequisite)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        trail_effects: source
            .swarm_disaster_trail_effect()
            .ordered_rows()
            .map(lower_trail_effect)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        cabinets: source
            .swarm_disaster_pathstrider_cabinet()
            .ordered_rows()
            .map(lower_cabinet)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        objectives: source
            .swarm_disaster_path_objective()
            .ordered_rows()
            .map(lower_objective)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        finish_conditions: source
            .swarm_disaster_pathstrider_finish()
            .ordered_rows()
            .map(lower_finish)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        unlocks: source
            .swarm_disaster_pathstrider_unlock()
            .ordered_rows()
            .map(lower_unlock)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        chapters: source
            .swarm_disaster_mechanical_chapter()
            .ordered_rows()
            .map(lower_chapter)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        bonuses: source
            .swarm_disaster_trailblaze_bonus()
            .ordered_rows()
            .map(lower_bonus)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        paths: source
            .swarm_disaster_path()
            .ordered_rows()
            .map(lower_path)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        path_boosts: source
            .swarm_disaster_path_boost()
            .ordered_rows()
            .map(lower_path_boost)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        resonances: source
            .swarm_disaster_resonance()
            .ordered_rows()
            .map(lower_resonance)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        interplays: source
            .swarm_disaster_resonance_interplay()
            .ordered_rows()
            .map(lower_interplay)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    };
    validate::catalog(&catalog)?;
    Ok(catalog)
}

fn lower_countdown(
    row: &SwarmDisasterCountdownDisarray,
) -> Result<CountdownDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    for (actual, expected) in [
        (&row.carry_policy, "CarryAcrossPlaneTransitions"),
        (
            &row.transition_boundary,
            "AcceptedMoveWhenPreMoveCountdownIsZero",
        ),
        (&row.same_boundary_order, "StableOperationId"),
        (&row.cap_policy, "Level21AndAboveRetainsLevel20Modifiers"),
    ] {
        if actual != expected {
            return invalid(&row.stable_key);
        }
    }
    json(&row.transition_result_json, &row.stable_key)?;
    json(&row.source_constant_bindings_json, &row.stable_key)?;
    Ok(CountdownDefinition {
        id: CountdownId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        initial: scalar(&row.initial_value, &row.stable_key)?,
        warning: scalar(&row.warning_threshold, &row.stable_key)?,
        movement_delta: scalar(&row.movement_delta, &row.stable_key)?,
        tiers: json(&row.disarray_tiers_json, &row.stable_key)?,
        source_constants: json(&row.source_constant_bindings_json, &row.stable_key)?,
    })
}

fn lower_boss_decay(
    row: &SwarmDisasterBossDecayLevel,
) -> Result<BossDecayDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.application_boundary != "FinalBossBattleSpecCreation"
        || row.stacking_policy != "SelectedRowsCoexistByStableBossDecayId"
        || !matches!(
            row.swarm_applicability.as_str(),
            "EnabledByReleasedSwarmText" | "DisabledUnprovenSharedDlcRow"
        )
    {
        return invalid(&row.stable_key);
    }
    Ok(BossDecayDefinition {
        id: BossDecayId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        threshold: nonempty(&row.threshold, &row.stable_key)?,
        tier: nonempty(&row.tier, &row.stable_key)?,
        effect_program: json(&row.effect_parameters_json, &row.stable_key)?,
        enabled: row.swarm_applicability == "EnabledByReleasedSwarmText",
    })
}

fn lower_audience_path(
    row: &SwarmDisasterAudiencePath,
) -> Result<AudiencePathDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(AudiencePathDefinition {
        id: AudiencePathId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        audience_die: AudienceDieId(positive(row.audience_die_id, &row.stable_key)?),
        shared_path: stable(&row.path_id)?,
        sort: positive_u16(row.sort, &row.stable_key)?,
        unlock_id: optional_stable(row.unlock_id.as_deref(), &row.stable_key)?,
        unlock_policy: json(&row.unlock_policy_json, &row.stable_key)?,
        initial_program: json(&row.initial_effects_json, &row.stable_key)?,
        passive_program: json(&row.passive_effects_json, &row.stable_key)?,
        description_parameters: scalar_list(
            row.description_parameters.as_deref(),
            &row.stable_key,
        )?,
        rogue_buff_type: nonempty(&row.rogue_buff_type, &row.stable_key)?,
        battle_event_buff_group: nonempty(&row.battle_event_buff_group, &row.stable_key)?,
        battle_event_enhance_buff_group: nonempty(
            &row.battle_event_enhance_buff_group,
            &row.stable_key,
        )?,
        extra_effect_refs: optional_text_list(row.extra_effect_refs.as_deref(), &row.stable_key)?,
    })
}

fn lower_audience_die(
    row: &SwarmDisasterAudienceDie,
) -> Result<AudienceDieDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(AudienceDieDefinition {
        id: AudienceDieId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        audience_path: AudiencePathId(positive(row.audience_path_id, &row.stable_key)?),
        shared_path: stable(&row.path_id)?,
        face_keys: text_list(&row.face_ids, &row.stable_key)?,
        roll_policy: json(&row.roll_policy_json, &row.stable_key)?,
        unlock_id: optional_stable(row.unlock_id.as_deref(), &row.stable_key)?,
        initial_effect_parameters: scalar_list(
            row.initial_effect_parameters.as_deref(),
            &row.stable_key,
        )?,
        passive_description_parameters: scalar_list(
            row.passive_description_parameters.as_deref(),
            &row.stable_key,
        )?,
        extra_effect_refs: optional_text_list(row.extra_effect_refs.as_deref(), &row.stable_key)?,
    })
}

fn lower_dice_rarity(
    row: &SwarmDisasterDiceRarity,
) -> Result<DiceRarityDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.name_color, &row.stable_key)?;
    Ok(DiceRarityDefinition {
        id: DiceRarityId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        rank: positive_u8(row.rank, &row.stable_key)?,
    })
}

fn lower_dice_face(
    row: &SwarmDisasterDiceFace,
) -> Result<DiceFaceDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    positive_u8(row.activation_stage, &row.stable_key)?;
    Ok(DiceFaceDefinition {
        id: DiceFaceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        audience_die: AudienceDieId(positive(row.audience_die_id, &row.stable_key)?),
        rarity: DiceRarityId(positive(row.rarity_id, &row.stable_key)?),
        target: DiceTargetId(positive(row.target_rule_id, &row.stable_key)?),
        sort: positive_u16(row.sort, &row.stable_key)?,
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
    })
}

fn lower_dice_target(
    row: &SwarmDisasterDiceTargetRule,
) -> Result<DiceTargetDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    for value in [&row.cardinality_json, &row.no_legal_target_json] {
        json(value, &row.stable_key)?;
    }
    if row.ordering != "StableDomainThenNodeId" {
        return invalid(&row.stable_key);
    }
    Ok(DiceTargetDefinition {
        id: DiceTargetId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        candidate_filter: json(&row.candidate_filter_json, &row.stable_key)?,
    })
}

fn lower_dice_control(
    row: &SwarmDisasterDiceRollControl,
) -> Result<DiceControlDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    for value in [&row.abandon_reward_json, &row.fallback_policy_json] {
        json(value, &row.stable_key)?;
    }
    if row.result_order != "AuthoredSortThenStableFaceId" {
        return invalid(&row.stable_key);
    }
    Ok(DiceControlDefinition {
        id: DiceControlId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        operation: nonempty(&row.operation, &row.stable_key)?,
        resource_cost: json(&row.resource_cost_json, &row.stable_key)?,
    })
}

fn lower_communing_choice(
    row: &SwarmDisasterCommuningChoice,
) -> Result<CommuningChoiceDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    json(&row.eligibility_json, &row.stable_key)?;
    json(&row.point_deltas_json, &row.stable_key)?;
    Ok(CommuningChoiceDefinition {
        id: CommuningChoiceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        shared_path: stable(&row.path_id)?,
        story_stage: scalar_u16(&row.story_stage, &row.stable_key)?,
        operations: json(&row.ordered_operations_json, &row.stable_key)?,
    })
}

fn lower_communing_dimension(
    row: &SwarmDisasterCommuningDimension,
) -> Result<CommuningDimensionDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.carry_policy != "PersistentAcrossRuns"
        || row.clamp_policy != "ClampAfterEachOrderedIncrement"
    {
        return invalid(&row.stable_key);
    }
    Ok(CommuningDimensionDefinition {
        id: CommuningDimensionId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        shared_path: stable(&row.path_id)?,
        maximum: positive_u16(row.max_points, &row.stable_key)?,
    })
}

fn lower_point_adjustment(
    row: &SwarmDisasterPointAdjustment,
) -> Result<PointAdjustmentDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.clamp_policy != "ClampToDimensionMaximumAfterOperation" {
        return invalid(&row.stable_key);
    }
    scalar(&row.operation_order, &row.stable_key)?;
    Ok(PointAdjustmentDefinition {
        id: PointAdjustmentId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        dimension: CommuningDimensionId(positive(row.dimension_id, &row.stable_key)?),
        source_id: nonempty(&row.source_id, &row.stable_key)?,
        source_kind: nonempty(&row.source_kind, &row.stable_key)?,
        ordinal: nonnegative_u16(row.ordinal, &row.stable_key)?,
        delta: scalar(&row.delta, &row.stable_key)?,
    })
}

fn lower_trail_node(
    row: &SwarmDisasterCommuningTrailNode,
) -> Result<TrailNodeDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(TrailNodeDefinition {
        id: TrailNodeId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        dimension: CommuningDimensionId(positive(row.dimension_id, &row.stable_key)?),
        effect_keys: text_list(&row.effect_ids, &row.stable_key)?,
        prerequisite_keys: optional_text_list(row.prerequisite_ids.as_deref(), &row.stable_key)?,
        threshold: scalar(&row.threshold, &row.stable_key)?,
    })
}

fn lower_trail_prerequisite(
    row: &SwarmDisasterTrailPrerequisite,
) -> Result<TrailPrerequisiteDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(TrailPrerequisiteDefinition {
        id: TrailPrerequisiteId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        node: TrailNodeId(positive(row.node_id, &row.stable_key)?),
        required_node: TrailNodeId(positive(row.required_node_id, &row.stable_key)?),
        ordinal: nonnegative_u16(row.ordinal, &row.stable_key)?,
        required_points: scalar(&row.required_points, &row.stable_key)?,
    })
}

fn lower_trail_effect(
    row: &SwarmDisasterTrailEffect,
) -> Result<TrailEffectDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    json(&row.battle_projection_json, &row.stable_key)?;
    Ok(TrailEffectDefinition {
        id: TrailEffectId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        node: TrailNodeId(positive(row.node_id, &row.stable_key)?),
        ordinal: nonnegative_u16(row.ordinal, &row.stable_key)?,
        operations: json(&row.ordered_operations_json, &row.stable_key)?,
    })
}

fn lower_cabinet(
    row: &SwarmDisasterPathstriderCabinet,
) -> Result<CabinetDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(CabinetDefinition {
        id: CabinetId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        objective_id: nonempty(&row.objective_id, &row.stable_key)?,
        prerequisite_keys: optional_text_list(row.prerequisite_ids.as_deref(), &row.stable_key)?,
        unlock_keys: optional_text_list(row.unlocks_cabinet_ids.as_deref(), &row.stable_key)?,
        point_deltas: json(&row.point_deltas_json, &row.stable_key)?,
    })
}

fn lower_objective(
    row: &SwarmDisasterPathObjective,
) -> Result<ObjectiveDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(ObjectiveDefinition {
        id: ObjectiveId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        cabinet: CabinetId(positive(row.cabinet_id, &row.stable_key)?),
        finish_key: stable(&row.finish_condition_id)?,
        progress_policy: json(&row.progress_policy_json, &row.stable_key)?,
    })
}

fn lower_finish(
    row: &SwarmDisasterPathstriderFinish,
) -> Result<FinishDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    nonempty(&row.finish_type, &row.stable_key)?;
    nonempty(&row.comparison, &row.stable_key)?;
    json(&row.parameters_json, &row.stable_key)?;
    Ok(FinishDefinition {
        id: FinishId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        enabled: row.enabled_for_swarm_compilation,
        target: scalar(&row.target_progress, &row.stable_key)?,
        unlock_keys: optional_text_list(row.unlock_ids.as_deref(), &row.stable_key)?,
    })
}

fn lower_unlock(
    row: &SwarmDisasterPathstriderUnlock,
) -> Result<UnlockDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.evaluation_boundary != "AfterAcceptedActivityOperation" {
        return invalid(&row.stable_key);
    }
    Ok(UnlockDefinition {
        id: UnlockId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        finish: FinishId(positive(row.finish_condition_id, &row.stable_key)?),
        consequence: json(&row.unlock_consequence_json, &row.stable_key)?,
    })
}

fn lower_chapter(
    row: &SwarmDisasterMechanicalChapter,
) -> Result<ChapterDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    json(&row.mechanical_unlock_json, &row.stable_key)?;
    Ok(ChapterDefinition {
        id: ChapterId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        dimension: row
            .dimension_id
            .map(|value| positive(value, &row.stable_key).map(CommuningDimensionId))
            .transpose()?,
        layer: positive_u8(row.layer, &row.stable_key)?,
        threshold: row
            .point_threshold
            .as_deref()
            .map(|value| scalar(value, &row.stable_key))
            .transpose()?,
    })
}

fn lower_bonus(
    row: &SwarmDisasterTrailblazeBonus,
) -> Result<BonusDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.application_boundary != "AfterTrailblazeBonusSelectionAtRunStart" {
        return invalid(&row.stable_key);
    }
    Ok(BonusDefinition {
        id: BonusId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
    })
}

fn lower_path(row: &SwarmDisasterPath) -> Result<PathDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    json(&row.battle_event_groups_json, &row.stable_key)?;
    json(&row.propagation_unlock_json, &row.stable_key)?;
    if !row.selectable {
        return invalid(&row.stable_key);
    }
    Ok(PathDefinition {
        id: PathId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        shared_path: stable(&row.shared_path_id)?,
        audience_die: AudienceDieId(positive(row.audience_die_id, &row.stable_key)?),
        resonance: ResonanceId(positive(row.resonance_id, &row.stable_key)?),
        sort: positive_u16(row.sort, &row.stable_key)?,
    })
}

fn lower_path_boost(
    row: &SwarmDisasterPathBoost,
) -> Result<PathBoostDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.application_boundary != "AfterPathSelectionAtRunStart" {
        return invalid(&row.stable_key);
    }
    Ok(PathBoostDefinition {
        id: PathBoostId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        path: PathId(positive(row.path_id, &row.stable_key)?),
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
    })
}

fn lower_resonance(
    row: &SwarmDisasterResonance,
) -> Result<ResonanceDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    scalar(&row.energy_max, &row.stable_key)?;
    scalar(&row.initial_energy, &row.stable_key)?;
    Ok(ResonanceDefinition {
        id: ResonanceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        path: PathId(positive(row.path_id, &row.stable_key)?),
        shared_resonance: stable(&row.shared_resonance_id)?,
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
    })
}

fn lower_interplay(
    row: &SwarmDisasterResonanceInterplay,
) -> Result<InterplayDefinition, SwarmDisasterUniqueError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    if row.application_boundary != "AfterAcceptedBlessingInventoryMutation"
        || !row.once_scope.starts_with("ResonanceInterplay:")
    {
        return invalid(&row.stable_key);
    }
    Ok(InterplayDefinition {
        id: InterplayId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key)?,
        main_path: PathId(positive(row.main_path_id, &row.stable_key)?),
        sub_path: PathId(positive(row.sub_path_id, &row.stable_key)?),
        thresholds: json(&row.thresholds_json, &row.stable_key)?,
        effect_program: json(&row.effect_program_json, &row.stable_key)?,
    })
}

fn metadata(key: &str, revision: &str, kind: &str) -> Result<(), SwarmDisasterUniqueError> {
    stable(key)?;
    if revision != ROW_REVISION || kind.is_empty() {
        return fail(SwarmDisasterUniqueErrorKind::Metadata, key);
    }
    Ok(())
}

fn stable(value: &str) -> Result<Box<str>, SwarmDisasterUniqueError> {
    if value.is_empty() || value.len() > 256 || !value.bytes().all(|byte| byte.is_ascii_graphic()) {
        return fail(SwarmDisasterUniqueErrorKind::Identifier, value);
    }
    Ok(value.into())
}

fn nonempty(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterUniqueError> {
    if value.trim().is_empty() {
        return invalid(key);
    }
    Ok(value.into())
}

fn json(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterUniqueError> {
    serde_json::from_str::<serde_json::Value>(value)
        .map_err(|_| error(SwarmDisasterUniqueErrorKind::Metadata, key))?;
    Ok(value.into())
}

pub(super) fn scalar(value: &str, key: &str) -> Result<Box<str>, SwarmDisasterUniqueError> {
    let unsigned = value.strip_prefix('-').unwrap_or(value);
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    let valid_integer = integer == "0"
        || (!integer.starts_with('0') && integer.bytes().all(|byte| byte.is_ascii_digit()));
    let valid_fraction = fraction.is_none_or(|fraction| {
        !fraction.is_empty()
            && !fraction.ends_with('0')
            && fraction.bytes().all(|byte| byte.is_ascii_digit())
    });
    if value.is_empty()
        || value == "-0"
        || value.starts_with('+')
        || !valid_integer
        || !valid_fraction
    {
        return fail(SwarmDisasterUniqueErrorKind::Identifier, key);
    }
    Ok(value.into())
}

fn scalar_u16(value: &str, key: &str) -> Result<u16, SwarmDisasterUniqueError> {
    scalar(value, key)?;
    value
        .parse::<u16>()
        .map_err(|_| error(SwarmDisasterUniqueErrorKind::Identifier, key))
}

fn text_list(values: &[String], key: &str) -> Result<Box<[Box<str>]>, SwarmDisasterUniqueError> {
    values
        .iter()
        .map(|value| stable(value))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| error(SwarmDisasterUniqueErrorKind::Identifier, key))
        .map(Vec::into_boxed_slice)
}

fn optional_text_list(
    values: Option<&[String]>,
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterUniqueError> {
    values.map_or_else(
        || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
        |values| text_list(values, key),
    )
}

fn scalar_list(
    values: Option<&[String]>,
    key: &str,
) -> Result<Box<[Box<str>]>, SwarmDisasterUniqueError> {
    values.map_or_else(
        || Ok(Vec::<Box<str>>::new().into_boxed_slice()),
        |values| {
            values
                .iter()
                .map(|value| scalar(value, key))
                .collect::<Result<Vec<_>, _>>()
                .map(Vec::into_boxed_slice)
        },
    )
}

fn optional_stable(
    value: Option<&str>,
    key: &str,
) -> Result<Option<Box<str>>, SwarmDisasterUniqueError> {
    value
        .map(|value| {
            stable(value).map_err(|_| error(SwarmDisasterUniqueErrorKind::Identifier, key))
        })
        .transpose()
}

fn positive(value: i32, key: &str) -> Result<u32, SwarmDisasterUniqueError> {
    u32::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterUniqueErrorKind::Identifier, key))
}

fn positive_u16(value: i32, key: &str) -> Result<u16, SwarmDisasterUniqueError> {
    u16::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterUniqueErrorKind::Identifier, key))
}

fn positive_u8(value: i32, key: &str) -> Result<u8, SwarmDisasterUniqueError> {
    u8::try_from(value)
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| error(SwarmDisasterUniqueErrorKind::Identifier, key))
}

fn nonnegative_u16(value: i32, key: &str) -> Result<u16, SwarmDisasterUniqueError> {
    u16::try_from(value).map_err(|_| error(SwarmDisasterUniqueErrorKind::Identifier, key))
}

fn invalid<T>(key: &str) -> Result<T, SwarmDisasterUniqueError> {
    fail(SwarmDisasterUniqueErrorKind::Metadata, key)
}

fn fail<T>(kind: SwarmDisasterUniqueErrorKind, key: &str) -> Result<T, SwarmDisasterUniqueError> {
    Err(error(kind, key))
}

fn error(kind: SwarmDisasterUniqueErrorKind, key: &str) -> SwarmDisasterUniqueError {
    SwarmDisasterUniqueError {
        kind,
        key: key.into(),
    }
}
