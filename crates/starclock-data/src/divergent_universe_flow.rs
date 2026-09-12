use crate::divergent_universe_catalog::{
    DivergentUniverseAreaDefinition, DivergentUniverseAreaId, DivergentUniverseAreaKind,
    DivergentUniverseCatalogError, DivergentUniverseCyclicalChallengeDefinition,
    DivergentUniverseCyclicalChallengeId, DivergentUniverseDifficultyDefinition,
    DivergentUniverseDifficultyId, DivergentUniverseEntryDefinition, DivergentUniverseEntryId,
    DivergentUniverseEntryKind, DivergentUniverseFinishConditionDefinition,
    DivergentUniverseFinishConditionId, DivergentUniverseFlowCatalog,
    DivergentUniverseFlowCatalogParts, DivergentUniverseFlowCondition,
    DivergentUniverseFlowDefinition, DivergentUniverseFlowId, DivergentUniverseFlowOperation,
    DivergentUniverseLayerDefinition, DivergentUniverseLayerId, DivergentUniverseModuleDefinition,
    DivergentUniverseModuleId, DivergentUniverseProfileDefinition, DivergentUniverseProfileId,
    DivergentUniverseRoomCandidateDefinition, DivergentUniverseRoomCandidateId,
    DivergentUniverseRoomPositionResolution, DivergentUniverseRoomReachability,
};
use serde::Deserialize;

use crate::divergent_universe::{DivergentUniverseDataError, debug_error, error};
use crate::divergent_universe_generated::SoraConfig;

pub(super) fn lower_divergent_universe_flow(
    config: &SoraConfig,
) -> Result<DivergentUniverseFlowCatalog, DivergentUniverseDataError> {
    let profile_row = one(
        config.divergent_universe_profiles().ordered_rows(),
        "profile",
    )?;
    let profile: ProfilePayload = payload(&profile_row.payload_json)?;
    let module_row = one(config.divergent_universe_modules().ordered_rows(), "module")?;
    let module: ModulePayload = payload(&module_row.payload_json)?;
    let parts = DivergentUniverseFlowCatalogParts {
        profile: DivergentUniverseProfileDefinition {
            id: profile_id(&profile_row.stable_key)?,
            module: module_id(&profile.module_id)?,
            entries: map_ids(profile.entry_refs, entry_id)?,
            finish_conditions: map_ids(profile.finish_condition_ids, finish_id)?,
            game_version: profile.game_version.into_boxed_str(),
            reference_runtime_enabled: profile.runtime_enabled,
        },
        module: DivergentUniverseModuleDefinition {
            id: module_id(&module_row.stable_key)?,
            source_id: required(module_row.source_id.as_deref(), "module source ID")?.into(),
            main_tournament: module.main_tourn_id,
            sub_tournament: module.sub_tourn_id,
        },
        entries: config
            .divergent_universe_entries()
            .ordered_rows()
            .map(|row| {
                let value: EntryPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseEntryDefinition {
                    id: entry_id(&row.stable_key)?,
                    kind: entry_kind(&value.entry_kind)?,
                    source_id: required(row.source_id.as_deref(), "entry source ID")?.into(),
                    module: module_id(&value.module_id)?,
                    related_panel_id: optional_text(value.related_panel_id),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        finish_conditions: config
            .divergent_universe_finish_conditions()
            .ordered_rows()
            .map(|row| {
                let value: FinishPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseFinishConditionDefinition {
                    id: finish_id(&row.stable_key)?,
                    source_id: required(row.source_id.as_deref(), "finish source ID")?.into(),
                    kind: value.condition_kind.into_boxed_str(),
                    parameter_type: value.parameter_type.into_boxed_str(),
                    string_parameter: value.string_parameter.into_boxed_str(),
                    integer_parameters: boxed_text(value.integer_parameters),
                    item_parameters: boxed_text(value.item_parameters),
                    progress: value.progress.into_boxed_str(),
                    source_only: value.terminal_disposition == "SourceConditionOnly",
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        areas: config
            .divergent_universe_areas()
            .ordered_rows()
            .map(|row| {
                let value: AreaPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseAreaDefinition {
                    id: area_id(&row.stable_key)?,
                    source_id: required(row.source_id.as_deref(), "area source ID")?.into(),
                    kind: area_kind(&value.area_type)?,
                    group: optional_text(value.area_group),
                    difficulties: map_ids(value.difficulty_ids, difficulty_id)?,
                    layers: map_ids(value.layer_ids, layer_id)?,
                    map_entry_id: value.map_entry_id.into_boxed_str(),
                    initial_room_type: value.initial_room_type.into_boxed_str(),
                    unlock_finish_source_id: value.unlock_finish_condition_id.into_boxed_str(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        difficulties: config
            .divergent_universe_difficulties()
            .ordered_rows()
            .map(|row| {
                let value: DifficultyPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseDifficultyDefinition {
                    id: difficulty_id(&row.stable_key)?,
                    source_id: required(row.source_id.as_deref(), "difficulty source ID")?.into(),
                    levels: value.level_list.into_boxed_slice(),
                    protocol_id: optional_text(value.protocol_id),
                    enemy_scaling_refs: boxed_text(value.enemy_scaling_refs),
                    unresolved_fields: boxed_text(value.unresolved_fields),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        layers: config
            .divergent_universe_layers()
            .ordered_rows()
            .map(|row| {
                let value: LayerPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseLayerDefinition {
                    id: layer_id(&row.stable_key)?,
                    source_id: required(row.source_id.as_deref(), "layer source ID")?.into(),
                    number: value.layer_number,
                    ordered_room_positions: boxed_text(value.ordered_room_position_ids),
                    room_position_resolution: room_position_resolution(
                        &value.room_position_resolution,
                    )?,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        room_candidates: config
            .divergent_universe_rooms()
            .ordered_rows()
            .map(|row| {
                let value: RoomPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseRoomCandidateDefinition {
                    id: room_id(&row.stable_key)?,
                    source_id: required(row.source_id.as_deref(), "room source ID")?.into(),
                    room_type: value.room_type.into_boxed_str(),
                    reachability: room_reachability(&value.reachability_disposition)?,
                    stage_refs: boxed_text(value.stage_refs),
                    offered_pool_ids: boxed_text(value.offered_pool_ids),
                    replacement_condition: value.replacement_condition.into_boxed_str(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        flows: config
            .divergent_universe_stage_flow()
            .ordered_rows()
            .map(|row| {
                let value: FlowPayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseFlowDefinition {
                    id: flow_id(&row.stable_key)?,
                    area: optional_id(value.area_id, area_id)?,
                    from_state: value.from_state.into_boxed_str(),
                    condition: flow_condition(&value.condition)?,
                    to_state: value.to_state.into_boxed_str(),
                    operations: value
                        .ordered_operations
                        .into_iter()
                        .map(|operation| flow_operation(&operation))
                        .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?
                        .into_boxed_slice(),
                    policy_id: value.policy_id.into_boxed_str(),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        cyclical_challenges: config
            .divergent_universe_cyclical_challenges()
            .ordered_rows()
            .map(|row| {
                let value: CyclicalPayload = payload(&row.payload_json)?;
                if value.challenge_kind != "WeekChallenge" {
                    return Err(error("unknown cyclical challenge kind"));
                }
                Ok(DivergentUniverseCyclicalChallengeDefinition {
                    id: cyclical_id(&row.stable_key)?,
                    source_id: required(row.source_id.as_deref(), "cyclical source ID")?.into(),
                    area: area_id(&value.area_id)?,
                    modifier_ids: boxed_text(value.modifier_ids),
                    modifier_resolution: value.modifier_resolution.into_boxed_str(),
                    enemy_display_refs: boxed_text(value.enemy_display_refs),
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
    };
    if !config.divergent_universe_layer_rooms().is_empty() {
        return Err(error(
            "Divergent Universe layer-room table must remain explicitly empty",
        ));
    }
    DivergentUniverseFlowCatalog::new(parts).map_err(catalog_error)
}

fn profile_id(value: &str) -> Result<DivergentUniverseProfileId, DivergentUniverseDataError> {
    DivergentUniverseProfileId::new(value).map_err(catalog_error)
}
fn module_id(value: &str) -> Result<DivergentUniverseModuleId, DivergentUniverseDataError> {
    DivergentUniverseModuleId::new(value).map_err(catalog_error)
}
fn entry_id(value: &str) -> Result<DivergentUniverseEntryId, DivergentUniverseDataError> {
    DivergentUniverseEntryId::new(value).map_err(catalog_error)
}
fn finish_id(
    value: &str,
) -> Result<DivergentUniverseFinishConditionId, DivergentUniverseDataError> {
    DivergentUniverseFinishConditionId::new(value).map_err(catalog_error)
}
fn area_id(value: &str) -> Result<DivergentUniverseAreaId, DivergentUniverseDataError> {
    DivergentUniverseAreaId::new(value).map_err(catalog_error)
}
fn difficulty_id(value: &str) -> Result<DivergentUniverseDifficultyId, DivergentUniverseDataError> {
    DivergentUniverseDifficultyId::new(value).map_err(catalog_error)
}
fn layer_id(value: &str) -> Result<DivergentUniverseLayerId, DivergentUniverseDataError> {
    DivergentUniverseLayerId::new(value).map_err(catalog_error)
}
fn room_id(value: &str) -> Result<DivergentUniverseRoomCandidateId, DivergentUniverseDataError> {
    DivergentUniverseRoomCandidateId::new(value).map_err(catalog_error)
}
fn flow_id(value: &str) -> Result<DivergentUniverseFlowId, DivergentUniverseDataError> {
    DivergentUniverseFlowId::new(value).map_err(catalog_error)
}
fn cyclical_id(
    value: &str,
) -> Result<DivergentUniverseCyclicalChallengeId, DivergentUniverseDataError> {
    DivergentUniverseCyclicalChallengeId::new(value).map_err(catalog_error)
}

fn area_kind(value: &str) -> Result<DivergentUniverseAreaKind, DivergentUniverseDataError> {
    match value {
        "Guide" => Ok(DivergentUniverseAreaKind::Guide),
        "Formal" => Ok(DivergentUniverseAreaKind::Formal),
        "WeekChallenge" => Ok(DivergentUniverseAreaKind::WeekChallenge),
        _ => Err(error("unknown Divergent Universe area kind")),
    }
}

fn entry_kind(value: &str) -> Result<DivergentUniverseEntryKind, DivergentUniverseDataError> {
    match value {
        "ModeTitle" => Ok(DivergentUniverseEntryKind::ModeTitle),
        "ResidentActivity" => Ok(DivergentUniverseEntryKind::ResidentActivity),
        _ => Err(error("unknown Divergent Universe entry kind")),
    }
}

fn room_position_resolution(
    value: &str,
) -> Result<DivergentUniverseRoomPositionResolution, DivergentUniverseDataError> {
    match value {
        "NoMatchingReleasedRow" => {
            Ok(DivergentUniverseRoomPositionResolution::NoMatchingReleasedRow)
        }
        _ => Err(error("unknown room-position resolution")),
    }
}

fn room_reachability(
    value: &str,
) -> Result<DivergentUniverseRoomReachability, DivergentUniverseDataError> {
    match value {
        "UnprovenSharedCandidate" => Ok(DivergentUniverseRoomReachability::UnprovenSharedCandidate),
        _ => Err(error("unknown room reachability")),
    }
}

fn flow_condition(
    value: &str,
) -> Result<DivergentUniverseFlowCondition, DivergentUniverseDataError> {
    match value {
        "AcceptedEntry" => Ok(DivergentUniverseFlowCondition::AcceptedEntry),
        "CurrentLayerCompleted" => Ok(DivergentUniverseFlowCondition::CurrentLayerCompleted),
        "NextLayerSelected" => Ok(DivergentUniverseFlowCondition::NextLayerSelected),
        "NextRoomSelected" => Ok(DivergentUniverseFlowCondition::NextRoomSelected),
        "RunFinalized" => Ok(DivergentUniverseFlowCondition::RunFinalized),
        _ => Err(error("unknown flow condition")),
    }
}

fn flow_operation(
    value: &str,
) -> Result<DivergentUniverseFlowOperation, DivergentUniverseDataError> {
    match value {
        "InitializeArea" => Ok(DivergentUniverseFlowOperation::InitializeArea),
        "EnterLayer" => Ok(DivergentUniverseFlowOperation::EnterLayer),
        "ExitLayer" => Ok(DivergentUniverseFlowOperation::ExitLayer),
        "EvaluateFinish" => Ok(DivergentUniverseFlowOperation::EvaluateFinish),
        "FinalizeArea" => Ok(DivergentUniverseFlowOperation::FinalizeArea),
        "CarryRunState" => Ok(DivergentUniverseFlowOperation::CarryRunState),
        "CarryRunInventory" => Ok(DivergentUniverseFlowOperation::CarryRunInventory),
        "CarryEquationProgress" => Ok(DivergentUniverseFlowOperation::CarryEquationProgress),
        "CarryTemporaryBuilds" => Ok(DivergentUniverseFlowOperation::CarryTemporaryBuilds),
        "ClearRunState" => Ok(DivergentUniverseFlowOperation::ClearRunState),
        "RemoveTemporaryBuilds" => Ok(DivergentUniverseFlowOperation::RemoveTemporaryBuilds),
        "PreservePermanentUnlocks" => Ok(DivergentUniverseFlowOperation::PreservePermanentUnlocks),
        _ => Err(error("unknown flow operation")),
    }
}

fn optional_id<T>(
    value: String,
    parser: fn(&str) -> Result<T, DivergentUniverseDataError>,
) -> Result<Option<T>, DivergentUniverseDataError> {
    if value.is_empty() {
        Ok(None)
    } else {
        parser(&value).map(Some)
    }
}

fn map_ids<T>(
    values: Vec<String>,
    parser: fn(&str) -> Result<T, DivergentUniverseDataError>,
) -> Result<Box<[T]>, DivergentUniverseDataError> {
    values
        .into_iter()
        .map(|value| parser(&value))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn boxed_text(values: Vec<String>) -> Box<[Box<str>]> {
    values
        .into_iter()
        .map(String::into_boxed_str)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn optional_text(value: String) -> Option<Box<str>> {
    (!value.is_empty()).then(|| value.into_boxed_str())
}

fn required<'a>(
    value: Option<&'a str>,
    label: &str,
) -> Result<&'a str, DivergentUniverseDataError> {
    value
        .filter(|text| !text.is_empty())
        .ok_or_else(|| error(&format!("missing {label}")))
}

fn payload<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(value).map_err(debug_error)
}

fn one<T>(
    mut values: impl Iterator<Item = T>,
    label: &str,
) -> Result<T, DivergentUniverseDataError> {
    let value = values
        .next()
        .ok_or_else(|| error(&format!("missing {label}")))?;
    if values.next().is_some() {
        return Err(error(&format!("duplicate {label}")));
    }
    Ok(value)
}

fn catalog_error(value: DivergentUniverseCatalogError) -> DivergentUniverseDataError {
    debug_error(value)
}

#[derive(Deserialize)]
struct ProfilePayload {
    game_version: String,
    runtime_enabled: bool,
    entry_refs: Vec<String>,
    module_id: String,
    finish_condition_ids: Vec<String>,
}
#[derive(Deserialize)]
struct ModulePayload {
    main_tourn_id: u16,
    sub_tourn_id: u16,
}
#[derive(Deserialize)]
struct EntryPayload {
    entry_kind: String,
    module_id: String,
    related_panel_id: String,
}
#[derive(Deserialize)]
struct FinishPayload {
    condition_kind: String,
    parameter_type: String,
    string_parameter: String,
    integer_parameters: Vec<String>,
    item_parameters: Vec<String>,
    progress: String,
    terminal_disposition: String,
}
#[derive(Deserialize)]
struct AreaPayload {
    area_type: String,
    area_group: String,
    difficulty_ids: Vec<String>,
    layer_ids: Vec<String>,
    map_entry_id: String,
    initial_room_type: String,
    unlock_finish_condition_id: String,
}
#[derive(Deserialize)]
struct DifficultyPayload {
    level_list: Vec<u16>,
    protocol_id: String,
    enemy_scaling_refs: Vec<String>,
    unresolved_fields: Vec<String>,
}
#[derive(Deserialize)]
struct LayerPayload {
    layer_number: u16,
    ordered_room_position_ids: Vec<String>,
    room_position_resolution: String,
}
#[derive(Deserialize)]
struct RoomPayload {
    room_type: String,
    reachability_disposition: String,
    stage_refs: Vec<String>,
    offered_pool_ids: Vec<String>,
    replacement_condition: String,
}
#[derive(Deserialize)]
struct FlowPayload {
    area_id: String,
    from_state: String,
    condition: String,
    to_state: String,
    ordered_operations: Vec<String>,
    policy_id: String,
}
#[derive(Deserialize)]
struct CyclicalPayload {
    challenge_kind: String,
    area_id: String,
    modifier_ids: Vec<String>,
    modifier_resolution: String,
    enemy_display_refs: Vec<String>,
}
