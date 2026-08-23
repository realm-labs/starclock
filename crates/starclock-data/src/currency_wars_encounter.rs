//! Line-limit exception: exact-once encounter and battle-program lowering stays adjacent to its validation pass.
mod affix;
mod enemy;

use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_combat::Scalar;
use starclock_mode_currency_wars::{
    CurrencyWarsAvatarBattleBehaviorArchetype, CurrencyWarsAvatarBattleBehaviorBindingPolicy,
    CurrencyWarsAvatarBattleBehaviorPolicy, CurrencyWarsBattleBehaviorArchetype,
    CurrencyWarsBattleBehaviorFallbackRank, CurrencyWarsBattleBehaviorPolicy,
    CurrencyWarsBattleConfigurationArchetype, CurrencyWarsBattleConfigurationPolicy,
    CurrencyWarsBattleProgramBinding, CurrencyWarsBattleProgramBindingArchetype,
    CurrencyWarsBattleProgramBindingPolicy, CurrencyWarsBondBattleBehaviorArchetype,
    CurrencyWarsBondBattleBehaviorPolicy, CurrencyWarsBondId, CurrencyWarsBossPool,
    CurrencyWarsCharacterOverrideBinding, CurrencyWarsCharacterOverrideProgram,
    CurrencyWarsComplexAiCombineOperator, CurrencyWarsComplexAiFactor,
    CurrencyWarsComplexAiFactorGroup, CurrencyWarsComplexAiFactorSource,
    CurrencyWarsComplexAiGlobalFactors, CurrencyWarsComplexAiRange, CurrencyWarsEncounterCatalog,
    CurrencyWarsEncounterCatalogParts, CurrencyWarsEncounterGroup,
    CurrencyWarsEncounterRandomization, CurrencyWarsEncounterSourceObligation,
    CurrencyWarsEncounterWave, CurrencyWarsEnemyAiConfiguration,
    CurrencyWarsEnemyAiConfigurationBinding, CurrencyWarsEnemyCharacterConfiguration,
    CurrencyWarsEnemyCharacterConfigurationBinding, CurrencyWarsEquipmentId,
    CurrencyWarsGlobalModifierTemplate, CurrencyWarsGlobalTaskFormationOrder,
    CurrencyWarsGlobalTaskMaximumTargets, CurrencyWarsGlobalTaskNodeCount,
    CurrencyWarsGlobalTaskPredicate, CurrencyWarsGlobalTaskPresentationReason,
    CurrencyWarsGlobalTaskTargetPopulation, CurrencyWarsGlobalTaskTemplate,
    CurrencyWarsGlobalTaskTemplateDefinition, CurrencyWarsGlobalTaskTemplateLibrary,
    CurrencyWarsGlobalTaskWave, CurrencyWarsMechanicActivityProgram,
    CurrencyWarsMechanicEmptyConfigurationAudit, CurrencyWarsMechanicLayoutAudit,
    CurrencyWarsMechanicMetadataAudit, CurrencyWarsMechanicPresentationAudit,
    CurrencyWarsMechanicPresentationKind, CurrencyWarsMechanicProgram,
    CurrencyWarsMechanicProgramDisposition, CurrencyWarsMechanicRolePresentationAudit,
    CurrencyWarsMechanicScope, CurrencyWarsMechanicShapeCount,
    CurrencyWarsMechanicStructuredPresentationAudit,
    CurrencyWarsMechanicUnreachableBattleConfigurationAudit,
    CurrencyWarsMechanicUnreachableCharacterOverrideAudit, CurrencyWarsModuleRoleBan,
    CurrencyWarsOverrideConfigurationKind, CurrencyWarsOverrideDynamicSource,
    CurrencyWarsOverrideSkillAbilityBinding, CurrencyWarsOverrideSkillBinding,
    CurrencyWarsPositionKind, CurrencyWarsProgressionProgram, CurrencyWarsReleasedStage,
    CurrencyWarsReleasedStageEnemy, CurrencyWarsReleasedStageWave,
    CurrencyWarsRoleCostAvailability, CurrencyWarsRoleId, CurrencyWarsRoleReferenceScore,
    CurrencyWarsRunPosition, CurrencyWarsSeasonProgressionRule, CurrencyWarsSeasonRolePool,
    CurrencyWarsSeasonTraitRolePool,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_build::canonical_json,
    currency_wars_flow::{parse_boxed_strings, parse_decimal, parse_json, required},
    currency_wars_generated::SoraConfig,
};

use self::{affix::lower_enemy_affix, enemy::lower_enemy_slot};

pub(super) fn lower_currency_wars_encounters(
    config: &SoraConfig,
) -> Result<CurrencyWarsEncounterCatalog, CurrencyWarsDataError> {
    CurrencyWarsEncounterCatalog::new(CurrencyWarsEncounterCatalogParts {
        groups: config
            .currency_wars_encounter_groups()
            .ordered_rows()
            .map(|row| {
                let randomization: EncounterRandomizationRow = parse_json(required(
                    &row.randomization,
                    "encounter-group randomization",
                )?)?;
                Ok(CurrencyWarsEncounterGroup {
                    stable_key: row.stable_key.clone().into(),
                    source_id: stable_tail(&row.stable_key)?.parse().map_err(debug_error)?,
                    plane_id: required(&row.plane_id, "encounter-group plane")?.into(),
                    difficulty_id: required(&row.difficulty_id, "encounter-group difficulty")?
                        .into(),
                    rank: required(&row.rank, "encounter-group rank")?.into(),
                    candidate_stage_ids: parse_boxed_strings(row.candidate_stage_ids.as_ref())?,
                    monster_ids: parse_number_strings(row.monster_ids.as_ref())?,
                    battle_area_ids: parse_number_strings(row.battle_area_ids.as_ref())?,
                    boss_battle_area_id: parse_optional_number(row.boss_battle_area_id.as_ref())?,
                    randomization: CurrencyWarsEncounterRandomization {
                        initial_code: randomization.initial_code.parse().map_err(debug_error)?,
                        enabled: parse_binary_bool(&randomization.enabled)?,
                    },
                })
            })
            .collect::<Result<_, CurrencyWarsDataError>>()?,
        source_obligations: config
            .currency_wars_encounter_source_obligations()
            .ordered_rows()
            .map(|row| {
                let stage = row
                    .stage_id
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .map(|stage_id| lower_released_stage(stage_id, row.stage_snapshot.as_ref()))
                    .transpose()?;
                Ok(CurrencyWarsEncounterSourceObligation {
                    stable_key: row.stable_key.clone().into(),
                    parent_kind: required(&row.parent_kind, "encounter-source parent kind")?.into(),
                    parent_id: required(&row.parent_id, "encounter-source parent ID")?.into(),
                    resolution_state: required(
                        &row.resolution_state,
                        "encounter-source resolution",
                    )?
                    .into(),
                    camp_ids: parse_number_strings(row.camp_ids.as_ref())?,
                    stage,
                    replacement_condition: optional_boxed_text(row.replacement_condition.as_ref()),
                })
            })
            .collect::<Result<_, CurrencyWarsDataError>>()?,
        waves: config
            .currency_wars_encounter_waves()
            .ordered_rows()
            .map(|row| {
                let trigger: FormationWaveTriggerRow =
                    parse_json(required(&row.trigger, "encounter-wave trigger")?)?;
                let ability = (!trigger.ability.is_empty()).then(|| trigger.ability.into());
                Ok(CurrencyWarsEncounterWave {
                    stable_key: row.stable_key.clone().into(),
                    wave_index: parse_required(&row.wave_index, "encounter-wave index")?,
                    maximum_teammates: trigger.maximum_teammates.parse().map_err(debug_error)?,
                    ability,
                    parameters: trigger.parameters.into_boxed_slice(),
                })
            })
            .collect::<Result<_, CurrencyWarsDataError>>()?,
        enemy_slots: config
            .currency_wars_enemy_slots()
            .ordered_rows()
            .map(lower_enemy_slot)
            .collect::<Result<_, CurrencyWarsDataError>>()?,
        enemy_affixes: config
            .currency_wars_enemy_affixes()
            .ordered_rows()
            .map(lower_enemy_affix)
            .collect::<Result<_, CurrencyWarsDataError>>()?,
        boss_pools: config
            .currency_wars_boss_pools()
            .ordered_rows()
            .map(|row| {
                Ok(CurrencyWarsBossPool {
                    stable_key: row.stable_key.clone().into(),
                    source_id: stable_tail(&row.stable_key)?.parse().map_err(debug_error)?,
                    plane_id: required(&row.plane_id, "boss-pool plane")?.into(),
                    difficulty_id: required(&row.difficulty_id, "boss-pool difficulty")?.into(),
                    candidate_monster_ids: parse_boxed_strings(row.candidate_monster_ids.as_ref())?,
                    selection_policy: required(
                        &row.selection_policy,
                        "boss-pool selection policy",
                    )?
                    .into(),
                    boss_battle_area_id: parse_required(
                        &row.boss_battle_area_id,
                        "boss-pool battle area",
                    )?,
                    candidate_stage_ids: parse_boxed_strings(row.candidate_stage_ids.as_ref())?,
                })
            })
            .collect::<Result<_, CurrencyWarsDataError>>()?,
        mechanic_programs: lower_mechanics(config)?,
    })
    .map_err(debug_error)
}

#[derive(Deserialize)]
struct EncounterRandomizationRow {
    initial_code: String,
    enabled: String,
}

#[derive(Deserialize)]
struct FormationWaveTriggerRow {
    maximum_teammates: String,
    ability: String,
    parameters: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct ReleasedStageSnapshotRow {
    stage_type: String,
    level: String,
    elite_group: String,
    stage_abilities: serde_json::Value,
    resolved_enemy_waves: Vec<Vec<ReleasedStageEnemyRow>>,
}

#[derive(Deserialize)]
struct ReleasedStageEnemyRow {
    formation: String,
    source_monster_id: String,
    shared_enemy_key: String,
}

fn lower_released_stage(
    stage_id: &str,
    snapshot: Option<&String>,
) -> Result<CurrencyWarsReleasedStage, CurrencyWarsDataError> {
    let row: ReleasedStageSnapshotRow = parse_json(
        snapshot
            .filter(|value| !value.is_empty())
            .ok_or_else(|| error("released stage snapshot is missing"))?,
    )?;
    let waves = row
        .resolved_enemy_waves
        .into_iter()
        .map(|wave| {
            Ok(CurrencyWarsReleasedStageWave {
                enemies: wave
                    .into_iter()
                    .map(|enemy| {
                        let formation = enemy
                            .formation
                            .strip_prefix("Monster")
                            .ok_or_else(|| error("released stage formation is invalid"))?
                            .parse()
                            .map_err(debug_error)?;
                        Ok(CurrencyWarsReleasedStageEnemy {
                            formation,
                            source_monster_id: enemy
                                .source_monster_id
                                .parse()
                                .map_err(debug_error)?,
                            shared_enemy_key: enemy.shared_enemy_key.into(),
                        })
                    })
                    .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?
                    .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    Ok(CurrencyWarsReleasedStage {
        stage_id: stage_id.parse().map_err(debug_error)?,
        stage_type: row.stage_type.into(),
        level: row.level.parse().map_err(debug_error)?,
        elite_group: (!row.elite_group.is_empty())
            .then(|| row.elite_group.parse().map_err(debug_error))
            .transpose()?,
        stage_abilities_json: serde_json::to_string(&row.stage_abilities)
            .map_err(debug_error)?
            .into(),
        waves: waves.into_boxed_slice(),
    })
}

fn parse_number_strings<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Box<[T]>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    let values: Vec<String> = value
        .filter(|value| !value.is_empty())
        .map(|value| parse_json(value))
        .transpose()?
        .unwrap_or_default();
    values
        .into_iter()
        .map(|value| value.parse().map_err(debug_error))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn parse_optional_number<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.parse().map_err(debug_error))
        .transpose()
}

fn optional_boxed_text(value: Option<&String>) -> Option<Box<str>> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.as_str().into())
}

fn parse_binary_bool(value: &str) -> Result<bool, CurrencyWarsDataError> {
    match value {
        "0" | "undefined" => Ok(false),
        "1" => Ok(true),
        _ => Err(error("Currency Wars binary boolean is invalid")),
    }
}

fn lower_mechanics(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsMechanicProgram>, CurrencyWarsDataError> {
    let mut sources = BTreeMap::new();
    for row in config.currency_wars_mechanic_source_files().ordered_rows() {
        let key = stable_tail(&row.stable_key)?;
        if sources.insert(key, row).is_some() {
            return Err(error("Currency Wars mechanic source is duplicated"));
        }
    }
    let programs = config
        .currency_wars_mechanic_rules()
        .ordered_rows()
        .map(|row| {
            let key = stable_tail(&row.stable_key)?;
            let source = sources
                .remove(key)
                .ok_or_else(|| error("Currency Wars mechanic rule has no source"))?;
            let digest = required(&source.source_sha256, "mechanic source digest")?;
            let source_disposition = required(&source.disposition, "mechanic source disposition")?;
            if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(error("Currency Wars mechanic source policy is invalid"));
            }
            let operations_json =
                canonical_json(required(&row.ordered_operations, "mechanic operations")?)?;
            let mut operations: Vec<MechanicOperationRow> =
                serde_json::from_str(&operations_json).map_err(debug_error)?;
            if operations.len() != 1 {
                return Err(error(
                    "mechanic program must contain exactly one root operation",
                ));
            }
            let operation = operations
                .pop()
                .ok_or_else(|| error("validated mechanic root operation is missing"))?;
            let state_lifecycle = required(&row.state_lifecycle, "mechanic lifecycle")?;
            let disposition = lower_mechanic_disposition(
                operation,
                MechanicLoweringContext {
                    source_disposition,
                    source_sha256: digest,
                    source_path: required(&source.source_path, "mechanic source path")?,
                    state_lifecycle,
                    runtime_lowered: required(
                        &row.runtime_lowered,
                        "mechanic runtime-lowered state",
                    )?,
                    stable_key: &row.stable_key,
                    operations_json,
                },
            )?;
            Ok(CurrencyWarsMechanicProgram {
                stable_key: row.stable_key.clone().into(),
                source_path: required(&source.source_path, "mechanic source path")?.into(),
                source_sha256: digest.into(),
                mechanic_family: required(&source.mechanic_family, "mechanic family")?.into(),
                scope: match required(&row.scope, "mechanic scope")? {
                    "CrossBattleActivity" => CurrencyWarsMechanicScope::CrossBattleActivity,
                    "BattleVisibleOrBattleBoundary" => {
                        CurrencyWarsMechanicScope::BattleVisibleOrBattleBoundary
                    }
                    _ => return Err(error("Currency Wars mechanic scope is unknown")),
                },
                trigger: required(&row.trigger, "mechanic trigger")?.into(),
                state_lifecycle: state_lifecycle.into(),
                disposition,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    if !sources.is_empty() {
        return Err(error("Currency Wars mechanic source has no rule"));
    }
    Ok(programs)
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum MechanicOperationRow {
    PreserveExactSourceContribution {
        source_id: String,
        interpretation: String,
    },
    AuditPresentationOnly {
        reason: String,
        source_id: String,
        source_sha256: String,
        configuration_type_counts: Vec<ConfigurationTypeCountRow>,
        operation_type_counts: Vec<OperationTypeCountRow>,
        tutorial_keys: Vec<String>,
        custom_time_types: Vec<String>,
        player_action_types: Vec<String>,
        authoritative_operation_count: u32,
        ordered_shape_sha256: String,
    },
    AuditLayoutDescriptor {
        reason: String,
        source_id: String,
        source_sha256: String,
        root_keys: Vec<String>,
        descriptor_entry_count: u32,
        authoritative_operation_count: u32,
        ordered_shape_sha256: String,
    },
    AuditRolePresentationMetadata {
        reason: String,
        source_id: String,
        source_sha256: String,
        record_key: String,
        text_hash: String,
        authoritative_operation_count: u32,
        ordered_shape_sha256: String,
    },
    AuditStructuredPresentationMetadata {
        reason: String,
        source_id: String,
        source_sha256: String,
        record_key: String,
        root_keys: Vec<String>,
        configuration_type_counts: Vec<ConfigurationTypeCountRow>,
        descriptor_entry_count: u32,
        authoritative_operation_count: u32,
        ordered_shape_sha256: String,
    },
    LowerBattleBehaviorPolicy(BattleBehaviorPolicyOperationRow),
    LowerAvatarBattleBehaviorPolicy(AvatarBattleBehaviorPolicyOperationRow),
    LowerBattleConfigurationPolicy(BattleConfigurationPolicyOperationRow),
    LowerBondBattleBehaviorPolicy(BondBattleBehaviorPolicyOperationRow),
    LowerBattleProgramBindingPolicy(BattleProgramBindingPolicyOperationRow),
    LowerEnemyCharacterConfiguration(EnemyCharacterConfigurationOperationRow),
    LowerEnemyAiConfiguration(EnemyAiConfigurationOperationRow),
    LowerGlobalComplexAiFactors(GlobalComplexAiFactorOperationRow),
    LowerGlobalTaskTemplates(GlobalTaskTemplateOperationRow),
    BindCharacterOverride(CharacterOverrideOperationRow),
    AuditUnreachableCharacterOverride(UnreachableCharacterOverrideOperationRow),
    AuditUnreachableBattleConfiguration {
        reason: String,
        source_id: String,
        source_sha256: String,
        ability_names: Vec<String>,
        global_modifier_names: Vec<String>,
        callback_event_counts: Vec<EventTypeCountRow>,
        configuration_type_counts: Vec<ConfigurationTypeCountRow>,
        reachable_binding_count: u32,
        ordered_shape_sha256: String,
    },
    AuditEmptyConfigurationProgram {
        reason: String,
        source_id: String,
        source_sha256: String,
        authoritative_operation_count: u32,
        ordered_shape_sha256: String,
    },
    ApplyRoleCostAvailability {
        source_id: String,
        source_sha256: String,
        cost: u8,
        standard_chapter: u8,
        standard_section: u8,
        overclock_chapter: u8,
        overclock_section: u8,
    },
    ProjectSeasonScoreAndExperience {
        source_id: String,
        source_sha256: String,
        division_id: u8,
        score_rule_id: u16,
        chapter: u8,
        section: u8,
        weekly_score: Option<u32>,
        experience: Option<u32>,
    },
    ApplyModuleRoleBan {
        source_id: String,
        source_sha256: String,
        module_id: u32,
        role_id: u32,
    },
    BindSeasonRolePool {
        source_id: String,
        source_sha256: String,
        season_id: u16,
        role_ids: Vec<u32>,
    },
    BindSeasonTraitRolePool {
        source_id: String,
        source_sha256: String,
        season_id: u16,
        trait_id: u32,
        role_ids: Vec<u32>,
    },
    ScoreSeasonRole {
        source_id: String,
        source_sha256: String,
        season_id: u16,
        role_id: u32,
        reference_score: u16,
    },
}

#[derive(Deserialize)]
struct ConfigurationTypeCountRow {
    #[serde(rename = "type")]
    shape: String,
    count: u32,
}

#[derive(Deserialize)]
struct OperationTypeCountRow {
    operation: String,
    count: u32,
}

#[derive(Deserialize)]
struct EventTypeCountRow {
    event: String,
    count: u32,
}

#[derive(Deserialize)]
struct BattleBehaviorPolicyOperationRow {
    source_id: String,
    source_sha256: String,
    policy_id: String,
    archetype: String,
    family_key: String,
    fallback_rank: String,
    ability_names: Vec<String>,
    global_modifier_names: Vec<String>,
    callback_event_counts: Vec<EventTypeCountRow>,
    configuration_type_counts: Vec<ConfigurationTypeCountRow>,
    selected_behavior: String,
    unresolved_field: String,
    confidence: String,
    replacement_condition: String,
    ordered_shape_sha256: String,
}

#[derive(Deserialize)]
struct AvatarBattleBehaviorPolicyOperationRow {
    source_id: String,
    source_sha256: String,
    policy_id: String,
    archetype: String,
    binding_policy: String,
    role_ids: Vec<u32>,
    avatar_ids: Vec<u32>,
    battle_event_ids: Vec<u32>,
    ability_names: Vec<String>,
    global_modifier_names: Vec<String>,
    callback_event_counts: Vec<EventTypeCountRow>,
    configuration_type_counts: Vec<ConfigurationTypeCountRow>,
    selected_behavior: String,
    unresolved_field: String,
    confidence: String,
    replacement_condition: String,
    ordered_shape_sha256: String,
}

#[derive(Deserialize)]
struct BattleConfigurationPolicyOperationRow {
    source_id: String,
    source_sha256: String,
    policy_id: String,
    archetype: String,
    ability_names: Vec<String>,
    global_modifier_names: Vec<String>,
    callback_event_counts: Vec<EventTypeCountRow>,
    configuration_type_counts: Vec<ConfigurationTypeCountRow>,
    selected_behavior: String,
    unresolved_field: String,
    confidence: String,
    replacement_condition: String,
    ordered_shape_sha256: String,
}

#[derive(Deserialize)]
struct BondBattleBehaviorPolicyOperationRow {
    source_id: String,
    source_sha256: String,
    policy_id: String,
    archetype: String,
    bond_ids: Vec<u32>,
    ability_names: Vec<String>,
    global_modifier_names: Vec<String>,
    callback_event_counts: Vec<EventTypeCountRow>,
    configuration_type_counts: Vec<ConfigurationTypeCountRow>,
    selected_behavior: String,
    unresolved_field: String,
    confidence: String,
    replacement_condition: String,
    ordered_shape_sha256: String,
}

#[derive(Deserialize)]
struct BattleProgramBindingPolicyOperationRow {
    source_id: String,
    source_sha256: String,
    policy_id: String,
    archetype: String,
    role_ids: Vec<u32>,
    avatar_ids: Vec<u32>,
    servant_ids: Vec<u32>,
    battle_event_ids: Vec<u32>,
    bond_ids: Vec<u32>,
    maze_buff_ids: Vec<u32>,
    enemy_affix_maze_buff_ids: Vec<u32>,
    equipment_ids: Vec<u32>,
    ability_names: Vec<String>,
    global_modifier_names: Vec<String>,
    callback_event_counts: Vec<EventTypeCountRow>,
    configuration_type_counts: Vec<ConfigurationTypeCountRow>,
    selected_behavior: String,
    unresolved_field: String,
    confidence: String,
    replacement_condition: String,
    ordered_shape_sha256: String,
}

#[derive(Deserialize)]
struct EnemyCharacterConfigurationOperationRow {
    source_id: String,
    source_sha256: String,
    bindings: Vec<EnemyCharacterConfigurationBindingRow>,
    ability_names: Vec<String>,
    skill_names: Vec<String>,
    skill_ability_count: u32,
    dynamic_source_count: u32,
    mechanical_shape_sha256: String,
}

#[derive(Deserialize)]
struct EnemyCharacterConfigurationBindingRow {
    shared_enemy_key: String,
    source_template_id: String,
}

#[derive(Deserialize)]
struct EnemyAiConfigurationOperationRow {
    source_id: String,
    source_sha256: String,
    ai_name: String,
    bindings: Vec<EnemyCharacterConfigurationBindingRow>,
    variable_names: Vec<String>,
    decision_names: Vec<String>,
    skill_names: Vec<String>,
    node_type_counts: Vec<ConfigurationTypeCountRow>,
    mechanical_shape_sha256: String,
}

#[derive(Deserialize)]
struct GlobalComplexAiFactorOperationRow {
    source_id: String,
    source_sha256: String,
    groups: Vec<ComplexAiFactorGroupRow>,
    mapper_policy_id: String,
    selected_behavior: String,
    unresolved_field: String,
    confidence: String,
    replacement_condition: String,
    mechanical_shape_sha256: String,
}

#[derive(Deserialize)]
struct GlobalTaskTemplateOperationRow {
    source_id: String,
    source_sha256: String,
    templates: Vec<GlobalTaskTemplateRow>,
    mechanical_shape_sha256: String,
}

#[derive(Deserialize)]
struct GlobalTaskTemplateRow {
    stable_key: String,
    kind: String,
    presentation_reason: Option<String>,
    wave: Option<String>,
    target_population: Option<String>,
    predicate: Option<String>,
    formation_order: Option<String>,
    maximum_targets: Option<String>,
    modifier_parameter: Option<String>,
    predicate_parameter: Option<String>,
    node_type_counts: Vec<ConfigurationTypeCountRow>,
    typed_node_count: u32,
    add_modifier_node_count: u32,
    ordered_shape_sha256: String,
}

#[derive(Deserialize)]
struct ComplexAiFactorGroupRow {
    stable_key: String,
    factors: Vec<ComplexAiFactorRow>,
}

#[derive(Deserialize)]
struct ComplexAiFactorRow {
    combine_operator: String,
    source_type: String,
    property_type_a: String,
    property_type_b: String,
    dynamic_value_key: String,
    modifier_name: String,
    is_target: Option<bool>,
    data_type: String,
    team_type: String,
    evaluator_type: String,
    evaluator_dynamic_value_key: String,
    list_combine_type: String,
    ai_tag_key: String,
    default_ai_tag_value: Option<String>,
    power_of_combat_power: Option<String>,
    power_of_damage_carry: Option<String>,
    sum_up_servant_damage_carry: Option<bool>,
    value_type: String,
    ranges: Vec<ComplexAiRangeRow>,
}

#[derive(Deserialize)]
struct ComplexAiRangeRow {
    xmin: Option<String>,
    ymin: Option<String>,
    xmax: Option<String>,
    ymax: Option<String>,
}

#[derive(Deserialize)]
struct CharacterOverrideOperationRow {
    source_id: String,
    source_sha256: String,
    configuration_kind: String,
    parent_config_path: String,
    bindings: Vec<CharacterOverrideBindingRow>,
    ability_names: Vec<String>,
    skill_ability_bindings: Vec<SkillAbilityBindingRow>,
    replaced_skills: Vec<String>,
    skill_bindings: Vec<SkillBindingRow>,
    dynamic_sources: Vec<DynamicSourceRow>,
    mechanical_shape_sha256: String,
}

#[derive(Deserialize)]
struct UnreachableCharacterOverrideOperationRow {
    reason: String,
    #[serde(flatten)]
    override_: CharacterOverrideOperationRow,
}

#[derive(Deserialize)]
#[serde(tag = "kind")]
enum CharacterOverrideBindingRow {
    RoleStar {
        role_id: u32,
        star_levels: Vec<u8>,
    },
    ServantStar {
        role_id: u32,
        servant_id: u32,
        star_levels: Vec<u8>,
    },
    SummonBattleEvent {
        season_id: u16,
        unit_id: u32,
        position: String,
    },
}

#[derive(Deserialize)]
struct SkillAbilityBindingRow {
    skill: String,
    ability_names: Vec<String>,
}

#[derive(Deserialize)]
struct SkillBindingRow {
    name: String,
    skill_type: String,
    use_type: String,
    target_type: String,
    entry_ability: String,
    prepare_ability: String,
    actual_attacker: String,
    child_skills: Vec<String>,
    insertable: bool,
    insert_priority: String,
}

#[derive(Deserialize)]
struct DynamicSourceRow {
    additive: bool,
    value_kind: String,
    key: String,
    source_kind: String,
    trigger_key: String,
    index: u16,
}

struct MechanicLoweringContext<'a> {
    source_disposition: &'a str,
    source_sha256: &'a str,
    source_path: &'a str,
    state_lifecycle: &'a str,
    runtime_lowered: &'a str,
    stable_key: &'a str,
    operations_json: Box<str>,
}

fn lower_mechanic_disposition(
    operation: MechanicOperationRow,
    context: MechanicLoweringContext<'_>,
) -> Result<CurrencyWarsMechanicProgramDisposition, CurrencyWarsDataError> {
    let MechanicLoweringContext {
        source_disposition,
        source_sha256,
        source_path,
        state_lifecycle,
        runtime_lowered,
        stable_key,
        operations_json,
    } = context;
    match operation {
        MechanicOperationRow::PreserveExactSourceContribution {
            source_id,
            interpretation,
        } => {
            if source_disposition != "ExactSourceProgramPreservedNoRuntimeLowering"
                || state_lifecycle != "ReferenceOnlyExactSourceBoundary"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || interpretation != "DeferredToLaterRuntimeGoal"
            {
                return Err(error("pending mechanic source boundary is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::PendingExactSource {
                ordered_operations_json: operations_json,
            })
        }
        MechanicOperationRow::AuditPresentationOnly {
            reason,
            source_id,
            source_sha256: audit_source_sha256,
            configuration_type_counts,
            operation_type_counts,
            tutorial_keys,
            custom_time_types,
            player_action_types,
            authoritative_operation_count,
            ordered_shape_sha256,
        } => {
            let reason = match reason.as_str() {
                "TutorialPresentationAndInputGuidance" => {
                    CurrencyWarsMechanicPresentationKind::TutorialAndInputGuidance
                }
                "WorldPropPresentationAndUiEntry" => {
                    CurrencyWarsMechanicPresentationKind::WorldPropAndUiEntry
                }
                _ => return Err(error("presentation-only mechanic reason is unknown")),
            };
            if source_disposition != "PresentationOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "PresentationOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
                || authoritative_operation_count != 0
            {
                return Err(error("presentation-only mechanic audit is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::Presentation(
                    CurrencyWarsMechanicPresentationAudit {
                        reason,
                        configuration_type_counts: configuration_type_counts
                            .into_iter()
                            .map(|row| CurrencyWarsMechanicShapeCount {
                                shape: row.shape.into(),
                                count: row.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        operation_type_counts: operation_type_counts
                            .into_iter()
                            .map(|row| CurrencyWarsMechanicShapeCount {
                                shape: row.operation.into(),
                                count: row.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        tutorial_keys: boxed_strings(tutorial_keys),
                        custom_time_types: boxed_strings(custom_time_types),
                        player_action_types: boxed_strings(player_action_types),
                        ordered_shape_sha256: ordered_shape_sha256.into(),
                    },
                ),
            ))
        }
        MechanicOperationRow::AuditLayoutDescriptor {
            reason,
            source_id,
            source_sha256: audit_source_sha256,
            root_keys,
            descriptor_entry_count,
            authoritative_operation_count,
            ordered_shape_sha256,
        } => {
            if reason != "DecoderLayoutDescriptor"
                || source_disposition != "MetadataOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "MetadataOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
                || descriptor_entry_count == 0
                || authoritative_operation_count != 0
                || root_keys.is_empty()
            {
                return Err(error("layout-descriptor mechanic audit is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::LayoutDescriptor(
                    CurrencyWarsMechanicLayoutAudit {
                        root_keys: boxed_strings(root_keys),
                        descriptor_entry_count,
                        ordered_shape_sha256: ordered_shape_sha256.into(),
                    },
                ),
            ))
        }
        MechanicOperationRow::AuditRolePresentationMetadata {
            reason,
            source_id,
            source_sha256: audit_source_sha256,
            record_key,
            text_hash,
            authoritative_operation_count,
            ordered_shape_sha256,
        } => {
            if !matches!(
                reason.as_str(),
                "LocalizedRoleRemark" | "LocalizedRoleTagDescription"
            ) || source_disposition != "MetadataOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "MetadataOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
                || record_key.is_empty()
                || text_hash.is_empty()
                || authoritative_operation_count != 0
            {
                return Err(error("role-presentation mechanic audit is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::RolePresentation(
                    CurrencyWarsMechanicRolePresentationAudit {
                        reason: reason.into(),
                        record_key: record_key.into(),
                        text_hash: text_hash.into(),
                        ordered_shape_sha256: ordered_shape_sha256.into(),
                    },
                ),
            ))
        }
        MechanicOperationRow::AuditStructuredPresentationMetadata {
            reason,
            source_id,
            source_sha256: audit_source_sha256,
            record_key,
            root_keys,
            configuration_type_counts,
            descriptor_entry_count,
            authoritative_operation_count,
            ordered_shape_sha256,
        } => {
            if !matches!(
                reason.as_str(),
                "NpcNameDescriptionAndIcon"
                    | "AnimationAudioAndEffectPresentation"
                    | "CameraAndAnimationTimingPresentation"
                    | "EmptyAbilityProgram"
                    | "WorldEntityModelAndLodPresentation"
                    | "WorldPropInteractionPresentation"
            ) || source_disposition != "MetadataOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "MetadataOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
                || record_key.is_empty()
                || root_keys.is_empty()
                || descriptor_entry_count == 0
                || authoritative_operation_count != 0
            {
                return Err(error("structured-presentation mechanic audit is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::StructuredPresentation(
                    CurrencyWarsMechanicStructuredPresentationAudit {
                        reason: reason.into(),
                        record_key: record_key.into(),
                        root_keys: boxed_strings(root_keys),
                        configuration_type_counts: configuration_type_counts
                            .into_iter()
                            .map(|row| CurrencyWarsMechanicShapeCount {
                                shape: row.shape.into(),
                                count: row.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        descriptor_entry_count,
                        ordered_shape_sha256: ordered_shape_sha256.into(),
                    },
                ),
            ))
        }
        MechanicOperationRow::LowerBattleBehaviorPolicy(row) => {
            if source_disposition != "PolicyBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedEnemyBehaviorPolicy"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.policy_id != "mechanic.configuration_program"
                || row.ability_names.is_empty()
                || row.configuration_type_counts.is_empty()
                || row.selected_behavior.is_empty()
                || row.unresolved_field.is_empty()
                || row.confidence != "PolicyOnlyNotObservedParity"
                || row.replacement_condition.is_empty()
            {
                return Err(error("battle-behavior mechanic policy is invalid"));
            }
            let archetype = match row.archetype.as_str() {
                "BossPhaseController" => CurrencyWarsBattleBehaviorArchetype::BossPhaseController,
                "MultiPhaseEnemy" => CurrencyWarsBattleBehaviorArchetype::MultiPhaseEnemy,
                "PartnerAssist" => CurrencyWarsBattleBehaviorArchetype::PartnerAssist,
                "MechanicalTrait" => CurrencyWarsBattleBehaviorArchetype::MechanicalTrait,
                "ShieldAndResourceTrait" => {
                    CurrencyWarsBattleBehaviorArchetype::ShieldAndResourceTrait
                }
                _ => return Err(error("battle-behavior mechanic archetype is unknown")),
            };
            let fallback_rank = match row.fallback_rank.as_str() {
                "Minion" => CurrencyWarsBattleBehaviorFallbackRank::Minion,
                "Elite" => CurrencyWarsBattleBehaviorFallbackRank::Elite,
                "Boss" => CurrencyWarsBattleBehaviorFallbackRank::Boss,
                _ => return Err(error("battle-behavior fallback rank is unknown")),
            };
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedBattlePolicy(
                    CurrencyWarsBattleBehaviorPolicy {
                        policy_id: row.policy_id.into(),
                        archetype,
                        family_key: (!row.family_key.is_empty()).then(|| row.family_key.into()),
                        fallback_rank,
                        ability_names: boxed_strings(row.ability_names),
                        global_modifier_names: boxed_strings(row.global_modifier_names),
                        callback_event_counts: row
                            .callback_event_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.event.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        configuration_type_counts: row
                            .configuration_type_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.shape.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        selected_behavior: row.selected_behavior.into(),
                        unresolved_field: row.unresolved_field.into(),
                        confidence: row.confidence.into(),
                        replacement_condition: row.replacement_condition.into(),
                        ordered_shape_sha256: row.ordered_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerAvatarBattleBehaviorPolicy(row) => {
            if source_disposition != "PolicyBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedAvatarBehaviorPolicy"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.policy_id != "mechanic.configuration_program"
                || row.ability_names.is_empty()
                || row.configuration_type_counts.is_empty()
                || row.selected_behavior.is_empty()
                || row.unresolved_field.is_empty()
                || row.confidence != "PolicyOnlyNotObservedParity"
                || row.replacement_condition.is_empty()
            {
                return Err(error("avatar battle-behavior mechanic policy is invalid"));
            }
            let archetype = match row.archetype.as_str() {
                "RoleBattleEvent" => CurrencyWarsAvatarBattleBehaviorArchetype::RoleBattleEvent,
                "AugmentBattleEvent" => {
                    CurrencyWarsAvatarBattleBehaviorArchetype::AugmentBattleEvent
                }
                _ => {
                    return Err(error(
                        "avatar battle-behavior mechanic archetype is unknown",
                    ));
                }
            };
            let binding_policy = match row.binding_policy.as_str() {
                "ExactBattleEvent" => {
                    CurrencyWarsAvatarBattleBehaviorBindingPolicy::ExactBattleEvent
                }
                "SameFamilyBattleEventFallback" => {
                    CurrencyWarsAvatarBattleBehaviorBindingPolicy::SameFamilyBattleEventFallback
                }
                "TypedAugmentController" => {
                    CurrencyWarsAvatarBattleBehaviorBindingPolicy::TypedAugmentController
                }
                _ => {
                    return Err(error("avatar battle-behavior binding policy is unknown"));
                }
            };
            let role_ids = row
                .role_ids
                .into_iter()
                .map(|role| {
                    CurrencyWarsRoleId::new(role)
                        .ok_or_else(|| error("avatar battle-behavior role ID is zero"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice();
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedAvatarBattlePolicy(
                    CurrencyWarsAvatarBattleBehaviorPolicy {
                        policy_id: row.policy_id.into(),
                        archetype,
                        binding_policy,
                        role_ids,
                        avatar_ids: row.avatar_ids.into_boxed_slice(),
                        battle_event_ids: row.battle_event_ids.into_boxed_slice(),
                        ability_names: boxed_strings(row.ability_names),
                        global_modifier_names: boxed_strings(row.global_modifier_names),
                        callback_event_counts: row
                            .callback_event_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.event.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        configuration_type_counts: row
                            .configuration_type_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.shape.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        selected_behavior: row.selected_behavior.into(),
                        unresolved_field: row.unresolved_field.into(),
                        confidence: row.confidence.into(),
                        replacement_condition: row.replacement_condition.into(),
                        ordered_shape_sha256: row.ordered_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerBattleConfigurationPolicy(row) => {
            if source_disposition != "PolicyBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedConfigurationFamilyPolicy"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.policy_id != "mechanic.configuration_program"
                || row.ability_names.is_empty() && row.global_modifier_names.is_empty()
                || row.configuration_type_counts.is_empty()
                || row.selected_behavior.is_empty()
                || row.unresolved_field.is_empty()
                || row.confidence != "PolicyOnlyNotObservedParity"
                || row.replacement_condition.is_empty()
            {
                return Err(error("battle configuration mechanic policy is invalid"));
            }
            let archetype = match row.archetype.as_str() {
                "CommonBattleKernel" => {
                    CurrencyWarsBattleConfigurationArchetype::CommonBattleKernel
                }
                "SharedModifierDefinitions" => {
                    CurrencyWarsBattleConfigurationArchetype::SharedModifierDefinitions
                }
                "MonsterTagController" => {
                    CurrencyWarsBattleConfigurationArchetype::MonsterTagController
                }
                "CharacterController" => {
                    CurrencyWarsBattleConfigurationArchetype::CharacterController
                }
                "MonsterController" => CurrencyWarsBattleConfigurationArchetype::MonsterController,
                "StageController" => CurrencyWarsBattleConfigurationArchetype::StageController,
                "SeasonController" => CurrencyWarsBattleConfigurationArchetype::SeasonController,
                "CurrentEquipmentController" => {
                    CurrencyWarsBattleConfigurationArchetype::CurrentEquipmentController
                }
                _ => return Err(error("battle configuration archetype is unknown")),
            };
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedBattleConfigurationPolicy(
                    CurrencyWarsBattleConfigurationPolicy {
                        policy_id: row.policy_id.into(),
                        archetype,
                        ability_names: boxed_strings(row.ability_names),
                        global_modifier_names: boxed_strings(row.global_modifier_names),
                        callback_event_counts: row
                            .callback_event_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.event.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        configuration_type_counts: row
                            .configuration_type_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.shape.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        selected_behavior: row.selected_behavior.into(),
                        unresolved_field: row.unresolved_field.into(),
                        confidence: row.confidence.into(),
                        replacement_condition: row.replacement_condition.into(),
                        ordered_shape_sha256: row.ordered_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerBondBattleBehaviorPolicy(row) => {
            if source_disposition != "PolicyBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedBondBehaviorPolicy"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.policy_id != "mechanic.configuration_program"
                || row.bond_ids.is_empty()
                || row.ability_names.is_empty()
                || row.selected_behavior.is_empty()
                || row.unresolved_field.is_empty()
                || row.confidence != "PolicyOnlyNotObservedParity"
                || row.replacement_condition.is_empty()
            {
                return Err(error("Bond battle-behavior mechanic policy is invalid"));
            }
            let archetype = match row.archetype.as_str() {
                "BondStageAbilityController" => {
                    CurrencyWarsBondBattleBehaviorArchetype::BondStageAbilityController
                }
                "MultiBondStageAbilityController" => {
                    CurrencyWarsBondBattleBehaviorArchetype::MultiBondStageAbilityController
                }
                "WolfHuntSummonController" => {
                    CurrencyWarsBondBattleBehaviorArchetype::WolfHuntSummonController
                }
                _ => return Err(error("Bond battle-behavior archetype is unknown")),
            };
            let bond_ids = row
                .bond_ids
                .into_iter()
                .map(|bond| {
                    CurrencyWarsBondId::new(bond)
                        .ok_or_else(|| error("Bond battle-behavior ID is zero"))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice();
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedBondBattlePolicy(
                    CurrencyWarsBondBattleBehaviorPolicy {
                        policy_id: row.policy_id.into(),
                        archetype,
                        bond_ids,
                        ability_names: boxed_strings(row.ability_names),
                        global_modifier_names: boxed_strings(row.global_modifier_names),
                        callback_event_counts: row
                            .callback_event_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.event.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        configuration_type_counts: row
                            .configuration_type_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.shape.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        selected_behavior: row.selected_behavior.into(),
                        unresolved_field: row.unresolved_field.into(),
                        confidence: row.confidence.into(),
                        replacement_condition: row.replacement_condition.into(),
                        ordered_shape_sha256: row.ordered_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerBattleProgramBindingPolicy(row) => {
            if source_disposition != "PolicyBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedProgramBindingPolicy"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.policy_id != "mechanic.configuration_program"
                || row.selected_behavior.is_empty()
                || row.unresolved_field.is_empty()
                || row.confidence != "PolicyOnlyNotObservedParity"
                || row.replacement_condition.is_empty()
            {
                return Err(error("battle-program binding policy is invalid"));
            }
            let archetype = match row.archetype.as_str() {
                "CoreAvatarAbility" => CurrencyWarsBattleProgramBindingArchetype::CoreAvatarAbility,
                "ServantAbility" => CurrencyWarsBattleProgramBindingArchetype::ServantAbility,
                "RoleBattleEvent" => CurrencyWarsBattleProgramBindingArchetype::RoleBattleEvent,
                "BondStageAbility" => CurrencyWarsBattleProgramBindingArchetype::BondStageAbility,
                "AugmentStageAbility" => {
                    CurrencyWarsBattleProgramBindingArchetype::AugmentStageAbility
                }
                "MonsterTagController" => {
                    CurrencyWarsBattleProgramBindingArchetype::MonsterTagController
                }
                "EquipmentController" => {
                    CurrencyWarsBattleProgramBindingArchetype::EquipmentController
                }
                _ => return Err(error("battle-program binding archetype is unknown")),
            };
            if row.ability_names.is_empty()
                && archetype != CurrencyWarsBattleProgramBindingArchetype::BondStageAbility
            {
                return Err(error("battle-program binding has no Ability names"));
            }
            let mut bindings = Vec::new();
            for raw in row.role_ids {
                bindings.push(CurrencyWarsBattleProgramBinding::Role(
                    CurrencyWarsRoleId::new(raw)
                        .ok_or_else(|| error("battle-program role ID is zero"))?,
                ));
            }
            for raw in row.avatar_ids {
                if raw == 0 {
                    return Err(error("battle-program avatar ID is zero"));
                }
                bindings.push(CurrencyWarsBattleProgramBinding::Avatar(raw));
            }
            for raw in row.servant_ids {
                if raw == 0 {
                    return Err(error("battle-program servant ID is zero"));
                }
                bindings.push(CurrencyWarsBattleProgramBinding::Servant(raw));
            }
            for raw in row.battle_event_ids {
                if raw == 0 {
                    return Err(error("battle-program BattleEvent ID is zero"));
                }
                bindings.push(CurrencyWarsBattleProgramBinding::BattleEvent(raw));
            }
            for raw in row.bond_ids {
                bindings.push(CurrencyWarsBattleProgramBinding::Bond(
                    CurrencyWarsBondId::new(raw)
                        .ok_or_else(|| error("battle-program Bond ID is zero"))?,
                ));
            }
            for raw in row.maze_buff_ids {
                if raw == 0 {
                    return Err(error("battle-program Augment MazeBuff ID is zero"));
                }
                bindings.push(CurrencyWarsBattleProgramBinding::AugmentMazeBuff(raw));
            }
            for raw in row.enemy_affix_maze_buff_ids {
                if raw == 0 {
                    return Err(error("battle-program enemy-Affix MazeBuff ID is zero"));
                }
                bindings.push(CurrencyWarsBattleProgramBinding::EnemyAffixMazeBuff(raw));
            }
            for raw in row.equipment_ids {
                bindings.push(CurrencyWarsBattleProgramBinding::Equipment(
                    CurrencyWarsEquipmentId::new(raw)
                        .ok_or_else(|| error("battle-program Equipment ID is zero"))?,
                ));
            }
            bindings.sort_unstable();
            if bindings.windows(2).any(|pair| pair[0] == pair[1]) {
                return Err(error("battle-program binding IDs are duplicated"));
            }
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedBattleProgramBindingPolicy(
                    CurrencyWarsBattleProgramBindingPolicy {
                        policy_id: row.policy_id.into(),
                        archetype,
                        bindings: bindings.into_boxed_slice(),
                        ability_names: boxed_strings(row.ability_names),
                        global_modifier_names: boxed_strings(row.global_modifier_names),
                        callback_event_counts: row
                            .callback_event_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.event.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        configuration_type_counts: row
                            .configuration_type_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.shape.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        selected_behavior: row.selected_behavior.into(),
                        unresolved_field: row.unresolved_field.into(),
                        confidence: row.confidence.into(),
                        replacement_condition: row.replacement_condition.into(),
                        ordered_shape_sha256: row.ordered_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::BindCharacterOverride(row) => {
            if source_disposition != "ExactActivityProgramLowered"
                || state_lifecycle != "ContributionSnapshotCharacterOverrideSelection"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.bindings.is_empty()
            {
                return Err(error("character-override selection program is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
                CurrencyWarsMechanicActivityProgram::CharacterOverride(lower_character_override(
                    row,
                    stable_key,
                    source_path,
                )?),
            ))
        }
        MechanicOperationRow::LowerEnemyCharacterConfiguration(row) => {
            if source_disposition != "ExactBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedEnemyCharacterConfiguration"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.bindings.is_empty()
            {
                return Err(error("enemy character-configuration program is invalid"));
            }
            let bindings = row
                .bindings
                .into_iter()
                .map(|binding| {
                    Ok(CurrencyWarsEnemyCharacterConfigurationBinding {
                        shared_enemy_key: binding.shared_enemy_key.into(),
                        source_template_id: binding
                            .source_template_id
                            .parse()
                            .map_err(debug_error)?,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedEnemyCharacterConfiguration(
                    CurrencyWarsEnemyCharacterConfiguration {
                        bindings: bindings.into_boxed_slice(),
                        ability_names: boxed_strings(row.ability_names),
                        skill_names: boxed_strings(row.skill_names),
                        skill_ability_count: row.skill_ability_count,
                        dynamic_source_count: row.dynamic_source_count,
                        mechanical_shape_sha256: row.mechanical_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerEnemyAiConfiguration(row) => {
            if source_disposition != "ExactBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedEnemyAiConfiguration"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.ai_name.is_empty()
                || row.bindings.is_empty()
                || row.decision_names.is_empty()
                || row.skill_names.is_empty()
            {
                return Err(error("enemy AI configuration program is invalid"));
            }
            let bindings = row
                .bindings
                .into_iter()
                .map(|binding| {
                    Ok(CurrencyWarsEnemyAiConfigurationBinding {
                        shared_enemy_key: binding.shared_enemy_key.into(),
                        source_template_id: binding
                            .source_template_id
                            .parse()
                            .map_err(debug_error)?,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedEnemyAiConfiguration(
                    CurrencyWarsEnemyAiConfiguration {
                        ai_name: row.ai_name.into(),
                        bindings: bindings.into_boxed_slice(),
                        variable_names: boxed_strings(row.variable_names),
                        decision_names: boxed_strings(row.decision_names),
                        skill_names: boxed_strings(row.skill_names),
                        node_type_counts: lower_configuration_type_counts(row.node_type_counts),
                        mechanical_shape_sha256: row.mechanical_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerGlobalComplexAiFactors(row) => {
            if source_disposition != "ExactBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedComplexAiFactorPolicy"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.groups.is_empty()
            {
                return Err(error("global Complex AI factor program is invalid"));
            }
            let groups = row
                .groups
                .into_iter()
                .map(lower_complex_ai_factor_group)
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(
                CurrencyWarsMechanicProgramDisposition::ExecutedComplexAiGlobalFactors(
                    CurrencyWarsComplexAiGlobalFactors {
                        groups: groups.into_boxed_slice(),
                        mapper_policy_id: row.mapper_policy_id.into(),
                        selected_behavior: row.selected_behavior.into(),
                        unresolved_field: row.unresolved_field.into(),
                        confidence: row.confidence.into(),
                        replacement_condition: row.replacement_condition.into(),
                        mechanical_shape_sha256: row.mechanical_shape_sha256.into(),
                    },
                ),
            )
        }
        MechanicOperationRow::LowerGlobalTaskTemplates(row) => {
            if source_disposition != "ExactBattleProgramLowered"
                || state_lifecycle != "BattleOwnedTypedGlobalTaskTemplateLibrary"
                || runtime_lowered != "true"
                || row.source_id.is_empty()
                || row.source_sha256 != source_sha256
                || row.templates.len() != 13
            {
                return Err(error("global task-template library is invalid"));
            }
            let templates = row
                .templates
                .into_iter()
                .map(lower_global_task_template)
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            let library = CurrencyWarsGlobalTaskTemplateLibrary::new(
                templates,
                row.mechanical_shape_sha256.into(),
            )
            .map_err(debug_error)?;
            Ok(CurrencyWarsMechanicProgramDisposition::ExecutedGlobalTaskTemplates(library))
        }
        MechanicOperationRow::AuditUnreachableCharacterOverride(row) => {
            if row.reason != "NoVersion44RoleServantOrSummonBinding"
                || source_disposition != "MetadataOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "MetadataOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || row.override_.source_id.is_empty()
                || row.override_.source_sha256 != source_sha256
                || !row.override_.bindings.is_empty()
            {
                return Err(error("unreachable character-override audit is invalid"));
            }
            let configuration_kind = row.override_.configuration_kind.into_boxed_str();
            let parent_config_path = row.override_.parent_config_path.into_boxed_str();
            let ability_count =
                u32::try_from(row.override_.ability_names.len()).map_err(debug_error)?;
            let skill_count =
                u32::try_from(row.override_.skill_bindings.len()).map_err(debug_error)?;
            let dynamic_source_count =
                u32::try_from(row.override_.dynamic_sources.len()).map_err(debug_error)?;
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::UnreachableCharacterOverride(
                    CurrencyWarsMechanicUnreachableCharacterOverrideAudit {
                        configuration_kind,
                        parent_config_path,
                        ability_count,
                        skill_count,
                        dynamic_source_count,
                        mechanical_shape_sha256: row
                            .override_
                            .mechanical_shape_sha256
                            .into_boxed_str(),
                    },
                ),
            ))
        }
        MechanicOperationRow::AuditUnreachableBattleConfiguration {
            reason,
            source_id,
            source_sha256: audit_source_sha256,
            ability_names,
            global_modifier_names,
            callback_event_counts,
            configuration_type_counts,
            reachable_binding_count,
            ordered_shape_sha256,
        } => {
            if reason != "NoVersion44EquipmentAbilityBinding"
                || source_disposition != "MetadataOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "MetadataOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
                || ability_names.is_empty()
                || reachable_binding_count != 0
                || configuration_type_counts.is_empty()
            {
                return Err(error("unreachable battle configuration audit is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::UnreachableBattleConfiguration(
                    CurrencyWarsMechanicUnreachableBattleConfigurationAudit {
                        reason: reason.into(),
                        ability_names: boxed_strings(ability_names),
                        global_modifier_names: boxed_strings(global_modifier_names),
                        callback_event_counts: callback_event_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.event.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        configuration_type_counts: configuration_type_counts
                            .into_iter()
                            .map(|entry| CurrencyWarsMechanicShapeCount {
                                shape: entry.shape.into(),
                                count: entry.count,
                            })
                            .collect::<Vec<_>>()
                            .into_boxed_slice(),
                        ordered_shape_sha256: ordered_shape_sha256.into(),
                    },
                ),
            ))
        }
        MechanicOperationRow::AuditEmptyConfigurationProgram {
            reason,
            source_id,
            source_sha256: audit_source_sha256,
            authoritative_operation_count,
            ordered_shape_sha256,
        } => {
            if reason != "NoAbilityModifierCallbackOrConfigurationNode"
                || source_disposition != "MetadataOnlyAuditedNoRuntimeLowering"
                || state_lifecycle != "MetadataOnlyNoAuthoritativeState"
                || runtime_lowered != "false"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
                || authoritative_operation_count != 0
                || ordered_shape_sha256.is_empty()
            {
                return Err(error("empty configuration-program audit is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::MetadataOnly(
                CurrencyWarsMechanicMetadataAudit::EmptyConfiguration(
                    CurrencyWarsMechanicEmptyConfigurationAudit {
                        reason: reason.into(),
                        ordered_shape_sha256: ordered_shape_sha256.into(),
                    },
                ),
            ))
        }
        MechanicOperationRow::ApplyRoleCostAvailability {
            source_id,
            source_sha256: audit_source_sha256,
            cost,
            standard_chapter,
            standard_section,
            overclock_chapter,
            overclock_section,
        } => {
            if source_disposition != "ExactActivityProgramLowered"
                || state_lifecycle != "ShopCandidateEligibilityByRunPosition"
                || runtime_lowered != "true"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
            {
                return Err(error("role-cost availability program is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
                CurrencyWarsMechanicActivityProgram::Progression(
                    CurrencyWarsProgressionProgram::RoleCostAvailability(
                        CurrencyWarsRoleCostAvailability {
                            stable_key: stable_key.into(),
                            cost,
                            standard: CurrencyWarsRunPosition::new(
                                standard_chapter,
                                standard_section,
                            )
                            .map_err(debug_error)?,
                            overclock: CurrencyWarsRunPosition::new(
                                overclock_chapter,
                                overclock_section,
                            )
                            .map_err(debug_error)?,
                        },
                    ),
                ),
            ))
        }
        MechanicOperationRow::ProjectSeasonScoreAndExperience {
            source_id,
            source_sha256: audit_source_sha256,
            division_id,
            score_rule_id,
            chapter,
            section,
            weekly_score,
            experience,
        } => {
            if source_disposition != "ExactActivityProgramLowered"
                || state_lifecycle != "SettlementProjectionNoRunMutation"
                || runtime_lowered != "true"
                || source_id.is_empty()
                || audit_source_sha256 != source_sha256
            {
                return Err(error("season-progression program is invalid"));
            }
            Ok(CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
                CurrencyWarsMechanicActivityProgram::Progression(
                    CurrencyWarsProgressionProgram::SeasonScoreAndExperience(
                        CurrencyWarsSeasonProgressionRule {
                            stable_key: stable_key.into(),
                            division: division_id,
                            score_rule: score_rule_id,
                            position: CurrencyWarsRunPosition::new(chapter, section)
                                .map_err(debug_error)?,
                            weekly_score,
                            experience,
                        },
                    ),
                ),
            ))
        }
        MechanicOperationRow::ApplyModuleRoleBan {
            source_id,
            source_sha256: operation_source_sha256,
            module_id,
            role_id: raw_role,
        } => {
            validate_activity_operation(
                source_disposition,
                state_lifecycle,
                "ShopAndRosterRoleEligibilityByModule",
                runtime_lowered,
                &source_id,
                source_sha256,
                &operation_source_sha256,
            )?;
            Ok(executed_progression(
                CurrencyWarsProgressionProgram::ModuleRoleBan(CurrencyWarsModuleRoleBan {
                    stable_key: stable_key.into(),
                    module: module_id,
                    role: role_id(raw_role)?,
                }),
            ))
        }
        MechanicOperationRow::BindSeasonRolePool {
            source_id,
            source_sha256: operation_source_sha256,
            season_id,
            role_ids: raw_roles,
        } => {
            validate_activity_operation(
                source_disposition,
                state_lifecycle,
                "ShopAndRosterRoleEligibilityBySeason",
                runtime_lowered,
                &source_id,
                source_sha256,
                &operation_source_sha256,
            )?;
            Ok(executed_progression(
                CurrencyWarsProgressionProgram::SeasonRolePool(CurrencyWarsSeasonRolePool {
                    stable_key: stable_key.into(),
                    season: season_id,
                    roles: role_ids(raw_roles)?,
                }),
            ))
        }
        MechanicOperationRow::BindSeasonTraitRolePool {
            source_id,
            source_sha256: operation_source_sha256,
            season_id,
            trait_id,
            role_ids: raw_roles,
        } => {
            validate_activity_operation(
                source_disposition,
                state_lifecycle,
                "ControllerRoleTraitIndex",
                runtime_lowered,
                &source_id,
                source_sha256,
                &operation_source_sha256,
            )?;
            Ok(executed_progression(
                CurrencyWarsProgressionProgram::SeasonTraitRolePool(
                    CurrencyWarsSeasonTraitRolePool {
                        stable_key: stable_key.into(),
                        season: season_id,
                        trait_id,
                        roles: role_ids(raw_roles)?,
                    },
                ),
            ))
        }
        MechanicOperationRow::ScoreSeasonRole {
            source_id,
            source_sha256: operation_source_sha256,
            season_id,
            role_id: raw_role,
            reference_score,
        } => {
            validate_activity_operation(
                source_disposition,
                state_lifecycle,
                "ControllerRoleReferenceRanking",
                runtime_lowered,
                &source_id,
                source_sha256,
                &operation_source_sha256,
            )?;
            Ok(executed_progression(
                CurrencyWarsProgressionProgram::RoleReferenceScore(
                    CurrencyWarsRoleReferenceScore {
                        stable_key: stable_key.into(),
                        season: season_id,
                        role: role_id(raw_role)?,
                        score: reference_score,
                    },
                ),
            ))
        }
    }
}

fn validate_activity_operation(
    source_disposition: &str,
    state_lifecycle: &str,
    expected_lifecycle: &str,
    runtime_lowered: &str,
    source_id: &str,
    source_sha256: &str,
    operation_source_sha256: &str,
) -> Result<(), CurrencyWarsDataError> {
    if source_disposition != "ExactActivityProgramLowered"
        || state_lifecycle != expected_lifecycle
        || runtime_lowered != "true"
        || source_id.is_empty()
        || operation_source_sha256 != source_sha256
    {
        return Err(error("role progression Activity program is invalid"));
    }
    Ok(())
}

fn executed_progression(
    program: CurrencyWarsProgressionProgram,
) -> CurrencyWarsMechanicProgramDisposition {
    CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
        CurrencyWarsMechanicActivityProgram::Progression(program),
    )
}

fn lower_global_task_template(
    row: GlobalTaskTemplateRow,
) -> Result<CurrencyWarsGlobalTaskTemplate, CurrencyWarsDataError> {
    let definition = match row.kind.as_str() {
        "PresentationOnly" => CurrencyWarsGlobalTaskTemplateDefinition::PresentationOnly(
            match required_option(row.presentation_reason.as_deref(), "presentation reason")? {
                "EnergyBarPresentation" => {
                    CurrencyWarsGlobalTaskPresentationReason::EnergyBarPresentation
                }
                "CameraPresentation" => {
                    CurrencyWarsGlobalTaskPresentationReason::CameraPresentation
                }
                "PursuedDamagePresentationTiming" => {
                    CurrencyWarsGlobalTaskPresentationReason::PursuedDamagePresentationTiming
                }
                "MonsterDropPresentationEffect" => {
                    CurrencyWarsGlobalTaskPresentationReason::MonsterDropPresentationEffect
                }
                _ => return Err(error("global task-template presentation reason is unknown")),
            },
        ),
        "ApplyModifier" => CurrencyWarsGlobalTaskTemplateDefinition::ApplyModifier(
            CurrencyWarsGlobalModifierTemplate {
                wave: match required_option(row.wave.as_deref(), "template wave")? {
                    "Any" => CurrencyWarsGlobalTaskWave::Any,
                    "First" => CurrencyWarsGlobalTaskWave::First,
                    _ => return Err(error("global task-template wave is unknown")),
                },
                target_population: match required_option(
                    row.target_population.as_deref(),
                    "template target population",
                )? {
                    "AllAlliesIncludingUnselectable" => {
                        CurrencyWarsGlobalTaskTargetPopulation::AllAlliesIncludingUnselectable
                    }
                    "InvocationSelected" => {
                        CurrencyWarsGlobalTaskTargetPopulation::InvocationSelected
                    }
                    _ => return Err(error("global task-template target population is unknown")),
                },
                predicate: match required_option(row.predicate.as_deref(), "template predicate")? {
                    "Any" => CurrencyWarsGlobalTaskPredicate::Any,
                    "InvocationTrait" => CurrencyWarsGlobalTaskPredicate::InvocationTrait,
                    "InvocationTraitWhenEnabled" => {
                        CurrencyWarsGlobalTaskPredicate::InvocationTraitWhenEnabled
                    }
                    "InvocationModifier" => CurrencyWarsGlobalTaskPredicate::InvocationModifier,
                    _ => return Err(error("global task-template predicate is unknown")),
                },
                formation_order: match required_option(
                    row.formation_order.as_deref(),
                    "template formation order",
                )? {
                    "Authored" => CurrencyWarsGlobalTaskFormationOrder::Authored,
                    "Ascending" => CurrencyWarsGlobalTaskFormationOrder::Ascending,
                    "Descending" => CurrencyWarsGlobalTaskFormationOrder::Descending,
                    _ => return Err(error("global task-template formation order is unknown")),
                },
                maximum_targets: match required_option(
                    row.maximum_targets.as_deref(),
                    "template maximum targets",
                )? {
                    "All" => CurrencyWarsGlobalTaskMaximumTargets::All,
                    "Invocation" => CurrencyWarsGlobalTaskMaximumTargets::Invocation,
                    _ => return Err(error("global task-template maximum targets is unknown")),
                },
                modifier_parameter: required_option(
                    row.modifier_parameter.as_deref(),
                    "template modifier parameter",
                )?
                .into(),
                predicate_parameter: row
                    .predicate_parameter
                    .filter(|value| !value.is_empty())
                    .map(Into::into),
            },
        ),
        _ => return Err(error("global task-template kind is unknown")),
    };
    Ok(CurrencyWarsGlobalTaskTemplate {
        stable_key: row.stable_key.into(),
        definition,
        node_type_counts: row
            .node_type_counts
            .into_iter()
            .map(|count| CurrencyWarsGlobalTaskNodeCount {
                node_type: count.shape.into(),
                count: count.count,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        typed_node_count: row.typed_node_count,
        add_modifier_node_count: row.add_modifier_node_count,
        ordered_shape_sha256: row.ordered_shape_sha256.into(),
    })
}

fn required_option<'a>(
    value: Option<&'a str>,
    field: &str,
) -> Result<&'a str, CurrencyWarsDataError> {
    value
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(field))
}

fn lower_complex_ai_factor_group(
    row: ComplexAiFactorGroupRow,
) -> Result<CurrencyWarsComplexAiFactorGroup, CurrencyWarsDataError> {
    let factors = row
        .factors
        .into_iter()
        .map(|factor| {
            let combine_operator = match factor.combine_operator.as_str() {
                "Add" => CurrencyWarsComplexAiCombineOperator::Add,
                "Mul" => CurrencyWarsComplexAiCombineOperator::Multiply,
                _ => return Err(error("Complex AI combine operator is unknown")),
            };
            let source = match factor.source_type.as_str() {
                "RPG.GameCore.ComplexSkillAISourcePropertyCompareRatio"
                    if factor.property_type_a == "CurrentHP"
                        && factor.property_type_b == "MaxHP"
                        && factor.dynamic_value_key.is_empty()
                        && factor.modifier_name.is_empty() =>
                {
                    CurrencyWarsComplexAiFactorSource::CurrentHpRatio
                }
                "RPG.GameCore.ComplexSkillAISourceAITag"
                    if !factor.dynamic_value_key.is_empty()
                        && factor.property_type_a.is_empty()
                        && factor.property_type_b.is_empty()
                        && factor.modifier_name.is_empty() =>
                {
                    if factor.is_target == Some(false) {
                        CurrencyWarsComplexAiFactorSource::CasterAiTag(
                            factor.dynamic_value_key.into(),
                        )
                    } else {
                        CurrencyWarsComplexAiFactorSource::AiTag(factor.dynamic_value_key.into())
                    }
                }
                "RPG.GameCore.ComplexSkillAIContainModifier"
                    if !factor.modifier_name.is_empty()
                        && factor.property_type_a.is_empty()
                        && factor.property_type_b.is_empty()
                        && factor.dynamic_value_key.is_empty() =>
                {
                    if factor.is_target == Some(false) {
                        CurrencyWarsComplexAiFactorSource::CasterContainsModifier(
                            factor.modifier_name.into(),
                        )
                    } else {
                        CurrencyWarsComplexAiFactorSource::ContainsModifier(
                            factor.modifier_name.into(),
                        )
                    }
                }
                "RPG.GameCore.ComplexSkillAIBattleGlobalData" if !factor.data_type.is_empty() => {
                    CurrencyWarsComplexAiFactorSource::BattleGlobalData(factor.data_type.into())
                }
                "RPG.GameCore.ComplexSkillAIAllTeamMemberCombine"
                    if !factor.team_type.is_empty()
                        && factor.evaluator_type == "RPG.GameCore.ComplexSkillAISourceAITag"
                        && !factor.evaluator_dynamic_value_key.is_empty()
                        && factor.list_combine_type == "Max" =>
                {
                    CurrencyWarsComplexAiFactorSource::TeamAiTagMaximum {
                        team_type: factor.team_type.into(),
                        key: factor.evaluator_dynamic_value_key.into(),
                    }
                }
                "RPG.GameCore.ComplexSkillAISourceIsCombatPowerWeightedRandomTarget"
                    if !factor.ai_tag_key.is_empty() =>
                {
                    CurrencyWarsComplexAiFactorSource::CombatPowerWeightedTarget {
                        ai_tag_key: factor.ai_tag_key.into(),
                        default_ai_tag_value: required_scalar(factor.default_ai_tag_value)?,
                        power_of_combat_power: required_scalar(factor.power_of_combat_power)?,
                        power_of_damage_carry: required_scalar(factor.power_of_damage_carry)?,
                        sum_up_servant_damage_carry: factor
                            .sum_up_servant_damage_carry
                            .ok_or_else(|| error("weighted-target servant policy is missing"))?,
                    }
                }
                "RPG.GameCore.ComplexSkillAISourceValueInTeamRatio"
                    if !factor.value_type.is_empty() =>
                {
                    CurrencyWarsComplexAiFactorSource::ValueInTeamRatio(factor.value_type.into())
                }
                _ => return Err(error("Complex AI factor source is unknown")),
            };
            let ranges = factor
                .ranges
                .into_iter()
                .map(|range| {
                    Ok(CurrencyWarsComplexAiRange {
                        minimum_input: optional_scalar(range.xmin)?,
                        minimum_output: optional_scalar(range.ymin)?,
                        maximum_input: optional_scalar(range.xmax)?,
                        maximum_output: optional_scalar(range.ymax)?,
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(CurrencyWarsComplexAiFactor {
                combine_operator,
                source,
                ranges: ranges.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    Ok(CurrencyWarsComplexAiFactorGroup {
        stable_key: row.stable_key.into(),
        factors: factors.into_boxed_slice(),
    })
}

fn optional_scalar(value: Option<String>) -> Result<Option<Scalar>, CurrencyWarsDataError> {
    value
        .map(|value| parse_decimal(&value).map(Scalar::from_scaled))
        .transpose()
}

fn required_scalar(value: Option<String>) -> Result<Scalar, CurrencyWarsDataError> {
    optional_scalar(value)?.ok_or_else(|| error("Complex AI scalar source field is missing"))
}

fn lower_configuration_type_counts(
    rows: Vec<ConfigurationTypeCountRow>,
) -> Box<[CurrencyWarsMechanicShapeCount]> {
    rows.into_iter()
        .map(|row| CurrencyWarsMechanicShapeCount {
            shape: row.shape.into(),
            count: row.count,
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn lower_character_override(
    row: CharacterOverrideOperationRow,
    stable_key: &str,
    source_path: &str,
) -> Result<CurrencyWarsCharacterOverrideProgram, CurrencyWarsDataError> {
    let configuration_kind = match row.configuration_kind.as_str() {
        "Character" => CurrencyWarsOverrideConfigurationKind::Character,
        "Servant" => CurrencyWarsOverrideConfigurationKind::Servant,
        "SummonBattleEvent" => CurrencyWarsOverrideConfigurationKind::SummonBattleEvent,
        _ => return Err(error("character-override configuration kind is unknown")),
    };
    let bindings = row
        .bindings
        .into_iter()
        .map(|binding| match binding {
            CharacterOverrideBindingRow::RoleStar {
                role_id: raw,
                star_levels,
            } => Ok(CurrencyWarsCharacterOverrideBinding::RoleStar {
                role: role_id(raw)?,
                star_levels: star_levels.into_boxed_slice(),
            }),
            CharacterOverrideBindingRow::ServantStar {
                role_id: raw,
                servant_id,
                star_levels,
            } => Ok(CurrencyWarsCharacterOverrideBinding::ServantStar {
                role: role_id(raw)?,
                servant_id,
                star_levels: star_levels.into_boxed_slice(),
            }),
            CharacterOverrideBindingRow::SummonBattleEvent {
                season_id,
                unit_id,
                position,
            } => Ok(CurrencyWarsCharacterOverrideBinding::SummonBattleEvent {
                season_id,
                unit_id,
                position: match position.as_str() {
                    "Front" => CurrencyWarsPositionKind::Front,
                    "Back" => CurrencyWarsPositionKind::Back,
                    _ => return Err(error("summon override position is unknown")),
                },
            }),
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    Ok(CurrencyWarsCharacterOverrideProgram {
        stable_key: stable_key.into(),
        source_path: source_path.into(),
        source_sha256: row.source_sha256.into(),
        configuration_kind,
        parent_config_path: row.parent_config_path.into(),
        bindings: bindings.into_boxed_slice(),
        ability_names: boxed_strings(row.ability_names),
        skill_ability_bindings: row
            .skill_ability_bindings
            .into_iter()
            .map(|binding| CurrencyWarsOverrideSkillAbilityBinding {
                skill: binding.skill.into(),
                ability_names: boxed_strings(binding.ability_names),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        replaced_skills: boxed_strings(row.replaced_skills),
        skill_bindings: row
            .skill_bindings
            .into_iter()
            .map(|binding| CurrencyWarsOverrideSkillBinding {
                name: binding.name.into(),
                skill_type: binding.skill_type.into(),
                use_type: binding.use_type.into(),
                target_type: binding.target_type.into(),
                entry_ability: binding.entry_ability.into(),
                prepare_ability: binding.prepare_ability.into(),
                actual_attacker: binding.actual_attacker.into(),
                child_skills: boxed_strings(binding.child_skills),
                insertable: binding.insertable,
                insert_priority: binding.insert_priority.into(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        dynamic_sources: row
            .dynamic_sources
            .into_iter()
            .map(|source| CurrencyWarsOverrideDynamicSource {
                additive: source.additive,
                value_kind: source.value_kind.into(),
                key: source.key.into(),
                source_kind: source.source_kind.into(),
                trigger_key: source.trigger_key.into(),
                index: source.index,
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        mechanical_shape_sha256: row.mechanical_shape_sha256.into(),
    })
}

fn role_id(raw: u32) -> Result<CurrencyWarsRoleId, CurrencyWarsDataError> {
    CurrencyWarsRoleId::new(raw).ok_or_else(|| error("Currency Wars role ID is zero"))
}

fn role_ids(raw: Vec<u32>) -> Result<Box<[CurrencyWarsRoleId]>, CurrencyWarsDataError> {
    let mut roles = raw
        .into_iter()
        .map(role_id)
        .collect::<Result<Vec<_>, _>>()?;
    let count = roles.len();
    roles.sort_unstable();
    roles.dedup();
    if roles.len() != count {
        return Err(error("Currency Wars role pool contains a duplicate role"));
    }
    Ok(roles.into_boxed_slice())
}

fn boxed_strings(values: Vec<String>) -> Box<[Box<str>]> {
    values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn stable_tail(value: &str) -> Result<&str, CurrencyWarsDataError> {
    value
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error("Currency Wars mechanic stable key has no tail"))
}

fn parse_required<T: std::str::FromStr>(
    value: &Option<String>,
    name: &str,
) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    required(value, name)?.parse().map_err(debug_error)
}
