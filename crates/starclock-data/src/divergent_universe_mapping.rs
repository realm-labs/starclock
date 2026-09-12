use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::divergent_universe::{DivergentUniverseDataError, debug_error, error};
use crate::divergent_universe_generated::SoraConfig;
use crate::divergent_universe_mapping_catalog::{
    DivergentUniverseAccountComparisonPolicy, DivergentUniverseAvatarLocator,
    DivergentUniverseExactTemporaryLoadout, DivergentUniverseMappedLevelPatch,
    DivergentUniverseMappedLightConePatch, DivergentUniverseMappedRelicPatch,
    DivergentUniverseMappedTracePatch, DivergentUniverseMappingBuildDefinition,
    DivergentUniverseMappingBuildId, DivergentUniverseMappingCatalog,
    DivergentUniverseMappingCatalogParts, DivergentUniverseMappingCondition,
    DivergentUniverseMappingDecimal, DivergentUniverseMappingEligibilityDefinition,
    DivergentUniverseMappingEligibilityId, DivergentUniverseMappingEligibilityKind,
    DivergentUniverseMappingError, DivergentUniverseMappingOperation,
    DivergentUniverseMappingRuleDefinition, DivergentUniverseMappingRuleId,
    DivergentUniverseMappingRuleKind, DivergentUniverseMappingTiming,
    DivergentUniversePublicIdentityResolution, DivergentUniverseStrongerBuildRule,
    DivergentUniverseTemporaryBuildPatch,
};

const TARGET_SOURCE_PATHS: [&str; 3] = [
    "ExcelOutput/RogueTournBuildRefAvatar.json",
    "ExcelOutput/RogueTournAvatar.json",
    "ExcelOutput/RogueTournRole.json",
];

pub(super) fn lower_divergent_universe_mapping(
    config: &SoraConfig,
) -> Result<DivergentUniverseMappingCatalog, DivergentUniverseDataError> {
    let parts = DivergentUniverseMappingCatalogParts {
        eligibility: config
            .divergent_universe_arithmetic_mapping_eligibility()
            .ordered_rows()
            .map(|row| {
                let value: EligibilityPayload = payload(&row.payload_json)?;
                validate_row_avatar(row.avatar_id.as_deref(), &value.avatar_id)?;
                Ok(DivergentUniverseMappingEligibilityDefinition {
                    id: eligibility_id(&row.stable_key)?,
                    avatar: avatar(&value.avatar_id)?,
                    sort_weight: value.sort_weight,
                    eligibility: eligibility_kind(&value.eligibility)?,
                    has_special_avatar_mapping: value.has_special_avatar_mapping,
                    has_role_buff: value.has_role_buff,
                    account_comparison: account_comparison(&value.account_comparison_policy)?,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        builds: config
            .divergent_universe_arithmetic_mapping_builds()
            .ordered_rows()
            .map(|row| {
                let value: BuildPayload = payload(&row.payload_json)?;
                validate_row_avatar(row.avatar_id.as_deref(), &value.avatar_id)?;
                Ok(DivergentUniverseMappingBuildDefinition {
                    id: build_id(&row.stable_key)?,
                    avatar: avatar(&value.avatar_id)?,
                    public_identity: public_identity(&value.public_identity_resolution)?,
                    eligible_catalog_entry: value.eligible_catalog_entry,
                    special_avatar: optional_text(value.special_avatar_id),
                    role_buff_id: required_text(value.role_buff_id, "role buff ID")?,
                    role_buff_binding_key: required_text(
                        value.role_buff_binding_key,
                        "role buff binding key",
                    )?,
                    role_buff_modifier_name: required_text(
                        value.role_buff_modifier_name,
                        "role buff modifier name",
                    )?,
                    role_buff_parameters: value
                        .role_buff_parameters
                        .into_iter()
                        .map(mapping_decimal)
                        .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?
                        .into_boxed_slice(),
                    patch: DivergentUniverseTemporaryBuildPatch {
                        level: level_patch(&value.level)?,
                        traces: trace_patch(&value.trace_state)?,
                        light_cone: light_cone_patch(&value.light_cone)?,
                        relics: relic_patch(&value.relics)?,
                        exact_loadout: exact_loadout(&value.exact_temporary_loadout)?,
                    },
                    runtime_lowered: value.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        rules: config
            .divergent_universe_arithmetic_mapping_rules()
            .ordered_rows()
            .map(|row| {
                let value: RulePayload = payload(&row.payload_json)?;
                Ok(DivergentUniverseMappingRuleDefinition {
                    id: rule_id(&row.stable_key)?,
                    kind: rule_kind(&row.stable_key)?,
                    timing: timing(&value.selection_timing)?,
                    condition: condition(&value.condition)?,
                    operations: value
                        .ordered_operations
                        .into_iter()
                        .map(|value| operation(&value))
                        .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?
                        .into_boxed_slice(),
                    stronger_build_rule: stronger_build_rule(&value.stronger_build_rule)?,
                    account_mutation: value.account_mutation,
                    runtime_lowered: value.runtime_lowered,
                })
            })
            .collect::<Result<Vec<_>, DivergentUniverseDataError>>()?,
        source_obligations: mapping_source_obligations(config)?,
    };
    DivergentUniverseMappingCatalog::new(parts).map_err(mapping_error)
}

fn mapping_source_obligations(config: &SoraConfig) -> Result<usize, DivergentUniverseDataError> {
    let target_sources = config
        .divergent_universe_sources()
        .ordered_rows()
        .filter_map(|row| {
            row.path
                .as_deref()
                .is_some_and(|path| TARGET_SOURCE_PATHS.contains(&path))
                .then_some(row.id)
        })
        .collect::<BTreeSet<_>>();
    let referenced_sources = config
        .divergent_universe_arithmetic_mapping_eligibility()
        .ordered_rows()
        .flat_map(|row| row.source_refs.iter().flatten().copied())
        .chain(
            config
                .divergent_universe_arithmetic_mapping_builds()
                .ordered_rows()
                .flat_map(|row| row.source_refs.iter().flatten().copied()),
        )
        .collect::<BTreeSet<_>>();
    if target_sources.len() != 258 || !target_sources.is_subset(&referenced_sources) {
        return Err(error("Arithmetic Mapping source receipt closure drift"));
    }
    let counts = config
        .divergent_universe_sources()
        .ordered_rows()
        .filter_map(|row| row.path.as_deref())
        .filter(|path| TARGET_SOURCE_PATHS.contains(path))
        .fold(BTreeMap::new(), |mut counts, path| {
            *counts.entry(path).or_insert(0usize) += 1;
            counts
        });
    if counts
        != BTreeMap::from([
            (TARGET_SOURCE_PATHS[0], 84),
            (TARGET_SOURCE_PATHS[1], 79),
            (TARGET_SOURCE_PATHS[2], 95),
        ])
    {
        return Err(error(
            "Arithmetic Mapping source category denominator drift",
        ));
    }
    Ok(target_sources.len())
}

fn validate_row_avatar(
    avatar_id: Option<&str>,
    payload_avatar: &str,
) -> Result<(), DivergentUniverseDataError> {
    if avatar_id != Some(payload_avatar) {
        return Err(error("Arithmetic Mapping row/avatar projection drift"));
    }
    Ok(())
}

fn eligibility_id(
    value: &str,
) -> Result<DivergentUniverseMappingEligibilityId, DivergentUniverseDataError> {
    DivergentUniverseMappingEligibilityId::new(value).map_err(mapping_error)
}

fn build_id(value: &str) -> Result<DivergentUniverseMappingBuildId, DivergentUniverseDataError> {
    DivergentUniverseMappingBuildId::new(value).map_err(mapping_error)
}

fn rule_id(value: &str) -> Result<DivergentUniverseMappingRuleId, DivergentUniverseDataError> {
    DivergentUniverseMappingRuleId::new(value).map_err(mapping_error)
}

fn avatar(value: &str) -> Result<DivergentUniverseAvatarLocator, DivergentUniverseDataError> {
    DivergentUniverseAvatarLocator::new(value).map_err(mapping_error)
}

fn mapping_decimal(
    value: String,
) -> Result<DivergentUniverseMappingDecimal, DivergentUniverseDataError> {
    DivergentUniverseMappingDecimal::new(value).map_err(mapping_error)
}

fn eligibility_kind(
    value: &str,
) -> Result<DivergentUniverseMappingEligibilityKind, DivergentUniverseDataError> {
    match value {
        "ExplicitBuildReferenceCatalog" => {
            Ok(DivergentUniverseMappingEligibilityKind::ExplicitBuildReferenceCatalog)
        }
        _ => Err(error("unknown Arithmetic Mapping eligibility kind")),
    }
}

fn account_comparison(
    value: &str,
) -> Result<DivergentUniverseAccountComparisonPolicy, DivergentUniverseDataError> {
    match value {
        "Apply only when the corresponding released below-threshold condition is true." => Ok(
            DivergentUniverseAccountComparisonPolicy::ApplyOnlyWhenCorrespondingBelowThresholdConditionIsTrue,
        ),
        _ => Err(error("unknown Arithmetic Mapping account comparison policy")),
    }
}

fn public_identity(
    value: &str,
) -> Result<DivergentUniversePublicIdentityResolution, DivergentUniverseDataError> {
    match value {
        "ResolvedAvatarConfig" => {
            Ok(DivergentUniversePublicIdentityResolution::ResolvedAvatarConfig)
        }
        "MissingReleasedAvatarConfig" => {
            Ok(DivergentUniversePublicIdentityResolution::MissingReleasedAvatarConfig)
        }
        _ => Err(error(
            "unknown Arithmetic Mapping public identity resolution",
        )),
    }
}

fn level_patch(
    value: &str,
) -> Result<DivergentUniverseMappedLevelPatch, DivergentUniverseDataError> {
    match value {
        "EquilibriumLevelCapWhenBelow" => {
            Ok(DivergentUniverseMappedLevelPatch::EquilibriumLevelCapWhenBelow)
        }
        _ => Err(error("unknown Arithmetic Mapping level patch")),
    }
}

fn trace_patch(
    value: &str,
) -> Result<DivergentUniverseMappedTracePatch, DivergentUniverseDataError> {
    match value {
        "ActivateOrRaiseWhenInactiveOrBelowRequirement" => {
            Ok(DivergentUniverseMappedTracePatch::ActivateOrRaiseWhenInactiveOrBelowRequirement)
        }
        _ => Err(error("unknown Arithmetic Mapping Trace patch")),
    }
}

fn light_cone_patch(
    value: &str,
) -> Result<DivergentUniverseMappedLightConePatch, DivergentUniverseDataError> {
    match value {
        "UnspecifiedConditionAndTemporaryIdentity" => {
            Ok(DivergentUniverseMappedLightConePatch::UnspecifiedConditionAndTemporaryIdentity)
        }
        _ => Err(error("unknown Arithmetic Mapping Light Cone patch")),
    }
}

fn relic_patch(
    value: &str,
) -> Result<DivergentUniverseMappedRelicPatch, DivergentUniverseDataError> {
    match value {
        "ReplaceWhenTotalEnhancementBelowRequirement" => {
            Ok(DivergentUniverseMappedRelicPatch::ReplaceWhenTotalEnhancementBelowRequirement)
        }
        _ => Err(error("unknown Arithmetic Mapping Relic patch")),
    }
}

fn exact_loadout(
    value: &str,
) -> Result<DivergentUniverseExactTemporaryLoadout, DivergentUniverseDataError> {
    match value {
        "Unspecified" => Ok(DivergentUniverseExactTemporaryLoadout::Unspecified),
        _ => Err(error("unknown exact Arithmetic Mapping loadout")),
    }
}

fn rule_kind(value: &str) -> Result<DivergentUniverseMappingRuleKind, DivergentUniverseDataError> {
    match value.strip_prefix("divergent-universe.mapping-rule.") {
        Some("scope") => Ok(DivergentUniverseMappingRuleKind::Scope),
        Some("character-level") => Ok(DivergentUniverseMappingRuleKind::CharacterLevel),
        Some("traces") => Ok(DivergentUniverseMappingRuleKind::Traces),
        Some("relics") => Ok(DivergentUniverseMappingRuleKind::Relics),
        Some("light-cone") => Ok(DivergentUniverseMappingRuleKind::LightCone),
        Some("refresh") => Ok(DivergentUniverseMappingRuleKind::Refresh),
        Some("teardown") => Ok(DivergentUniverseMappingRuleKind::Teardown),
        _ => Err(error("unknown Arithmetic Mapping rule identity")),
    }
}

fn timing(value: &str) -> Result<DivergentUniverseMappingTiming, DivergentUniverseDataError> {
    match value {
        "ModeBoundary" => Ok(DivergentUniverseMappingTiming::ModeBoundary),
        "RunEntryOrRefresh" => Ok(DivergentUniverseMappingTiming::RunEntryOrRefresh),
        "Unspecified" => Ok(DivergentUniverseMappingTiming::Unspecified),
        "RunEntryAndAcceptedPartyChange" => {
            Ok(DivergentUniverseMappingTiming::RunEntryAndAcceptedPartyChange)
        }
        "RunFinalization" => Ok(DivergentUniverseMappingTiming::RunFinalization),
        _ => Err(error("unknown Arithmetic Mapping timing")),
    }
}

fn condition(value: &str) -> Result<DivergentUniverseMappingCondition, DivergentUniverseDataError> {
    match value {
        "InsideDivergentUniverse" => Ok(DivergentUniverseMappingCondition::InsideDivergentUniverse),
        "CharacterLevelBelowEquilibriumCap" => {
            Ok(DivergentUniverseMappingCondition::CharacterLevelBelowEquilibriumCap)
        }
        "UnlockedTraceInactiveOrBelowRequirement" => {
            Ok(DivergentUniverseMappingCondition::UnlockedTraceInactiveOrBelowRequirement)
        }
        "RelicTotalEnhancementBelowRequirement" => {
            Ok(DivergentUniverseMappingCondition::RelicTotalEnhancementBelowRequirement)
        }
        "Unspecified" => Ok(DivergentUniverseMappingCondition::Unspecified),
        "MappingInputChanged" => Ok(DivergentUniverseMappingCondition::MappingInputChanged),
        "LeavingDivergentUniverse" => {
            Ok(DivergentUniverseMappingCondition::LeavingDivergentUniverse)
        }
        _ => Err(error("unknown Arithmetic Mapping condition")),
    }
}

fn operation(value: &str) -> Result<DivergentUniverseMappingOperation, DivergentUniverseDataError> {
    match value {
        "ApplyTemporaryMappingState" => {
            Ok(DivergentUniverseMappingOperation::ApplyTemporaryMappingState)
        }
        "RaiseCharacterToEquilibriumCap" => {
            Ok(DivergentUniverseMappingOperation::RaiseCharacterToEquilibriumCap)
        }
        "ActivateOrRaiseTrace" => Ok(DivergentUniverseMappingOperation::ActivateOrRaiseTrace),
        "ReplaceWithCompatibleTemporaryRelics" => {
            Ok(DivergentUniverseMappingOperation::ReplaceWithCompatibleTemporaryRelics)
        }
        "Unspecified" => Ok(DivergentUniverseMappingOperation::Unspecified),
        "ReevaluateOnlyBelowThresholdFields" => {
            Ok(DivergentUniverseMappingOperation::ReevaluateOnlyBelowThresholdFields)
        }
        "RemoveTemporaryMappingState" => {
            Ok(DivergentUniverseMappingOperation::RemoveTemporaryMappingState)
        }
        _ => Err(error("unknown Arithmetic Mapping operation")),
    }
}

fn stronger_build_rule(
    value: &str,
) -> Result<DivergentUniverseStrongerBuildRule, DivergentUniverseDataError> {
    match value {
        "PreserveWhenConditionIsFalse" => {
            Ok(DivergentUniverseStrongerBuildRule::PreserveWhenConditionIsFalse)
        }
        _ => Err(error("unknown stronger-build rule")),
    }
}

fn optional_text(value: String) -> Option<Box<str>> {
    (!value.is_empty()).then(|| value.into_boxed_str())
}

fn required_text(value: String, label: &str) -> Result<Box<str>, DivergentUniverseDataError> {
    if value.is_empty() {
        Err(error(&format!("missing {label}")))
    } else {
        Ok(value.into_boxed_str())
    }
}

fn payload<T: for<'de> Deserialize<'de>>(value: &str) -> Result<T, DivergentUniverseDataError> {
    serde_json::from_str(value).map_err(debug_error)
}

fn mapping_error(value: DivergentUniverseMappingError) -> DivergentUniverseDataError {
    debug_error(value)
}

#[derive(Deserialize)]
struct EligibilityPayload {
    account_comparison_policy: String,
    avatar_id: String,
    eligibility: String,
    has_role_buff: bool,
    has_special_avatar_mapping: bool,
    sort_weight: u32,
}

#[derive(Deserialize)]
struct BuildPayload {
    avatar_id: String,
    eligible_catalog_entry: bool,
    exact_temporary_loadout: String,
    level: String,
    light_cone: String,
    public_identity_resolution: String,
    relics: String,
    role_buff_binding_key: String,
    role_buff_id: String,
    role_buff_modifier_name: String,
    role_buff_parameters: Vec<String>,
    runtime_lowered: bool,
    special_avatar_id: String,
    trace_state: String,
}

#[derive(Deserialize)]
struct RulePayload {
    account_mutation: bool,
    condition: String,
    ordered_operations: Vec<String>,
    runtime_lowered: bool,
    selection_timing: String,
    stronger_build_rule: String,
}
