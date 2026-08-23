use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_build::{
    ability::AbilityInvestment,
    light_cone::{LightConeLevel, Superimposition},
    spec::{CombatantBuildSpec, EidolonLevel, LightConeLoadout, PromotionStage},
};
use starclock_combat::{Scalar, UnitLevel};
use starclock_mode_currency_wars::{
    CurrencyWarsBuildCatalog, CurrencyWarsBuildCatalogParts, CurrencyWarsBuildMapping,
    CurrencyWarsBuildMinimum, CurrencyWarsBuildReference, CurrencyWarsBuildSource,
    CurrencyWarsBuildSourceDisposition, CurrencyWarsBuildSourceRole,
    CurrencyWarsBuildSubstitutionRule, CurrencyWarsEquipmentCategory,
    CurrencyWarsEquipmentCategoryLimit, CurrencyWarsEquipmentDefinition,
    CurrencyWarsEquipmentDressRule, CurrencyWarsEquipmentId, CurrencyWarsEquipmentRecommendation,
    CurrencyWarsOffFieldConversion, CurrencyWarsOffFieldDestination,
    CurrencyWarsOffFieldEligibility, CurrencyWarsOffFieldPayload, CurrencyWarsOffFieldSourceKind,
    CurrencyWarsPropertyContribution, CurrencyWarsRelicSetThreshold, CurrencyWarsRoleId,
    CurrencyWarsRuntimeEquipment, CurrencyWarsSourceAbilityBinding, CurrencyWarsTrialBuild,
};

use crate::{
    catalog::{self as core_catalog, SimulationCatalog},
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::{
        SoraConfig, currency_wars_trial_builds::CurrencyWarsTrialBuilds as TrialBuildRow,
    },
};

pub(super) fn lower_currency_wars_build(
    config: &SoraConfig,
) -> Result<CurrencyWarsBuildCatalog, CurrencyWarsDataError> {
    CurrencyWarsBuildCatalog::new(CurrencyWarsBuildCatalogParts {
        mappings: lower_mappings(config)?,
        references: lower_references(config)?,
        trial_builds: lower_trial_builds(config)?,
        sources: lower_sources(config)?,
        substitution_rules: lower_substitution_rules(config)?,
        equipment: lower_equipment(config)?,
        recommendations: lower_recommendations(config)?,
        off_field_conversions: lower_off_field_conversions(config)?,
    })
    .map_err(debug_error)
}

fn lower_trial_builds(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsTrialBuild>, CurrencyWarsDataError> {
    let core = core_catalog::load(include_bytes!("../../../config/generated/config.sora"))
        .map_err(debug_error)?;
    config
        .currency_wars_trial_builds()
        .ordered_rows()
        .map(|row| {
            let role = role_id(required(&row.role_id, "trial Build role ID")?)?;
            let avatar_id = parse_required(&row.avatar_id, "trial Build avatar ID")?;
            let form = core
                .character_form_for_source_avatar(avatar_id)
                .ok_or_else(|| error("trial Build avatar has no shared character form"))?;
            let character = core
                .character(form)
                .ok_or_else(|| error("trial Build character is missing"))?;
            let build_character = core
                .build_catalog()
                .character(form)
                .ok_or_else(|| error("trial Build character Build definition is missing"))?;
            let technique_ability = character
                .technique_ability()
                .ok_or_else(|| error("trial Build character has no unique Technique ability"))?;
            let mut spec = CombatantBuildSpec::new(
                form,
                UnitLevel::new(parse_required(&row.level, "trial Build level")?)
                    .ok_or_else(|| error("trial Build level is invalid"))?,
                PromotionStage::new(parse_required(&row.promotion, "trial Build promotion")?)
                    .ok_or_else(|| error("trial Build promotion is invalid"))?,
            )
            .with_ability_levels(
                build_character
                    .ability_levels()
                    .iter()
                    .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
                    .collect(),
            )
            .map_err(debug_error)?
            .with_traces(
                build_character
                    .trace_graph()
                    .map_or_else(Vec::new, |graph| graph.canonical_order().to_vec()),
            )
            .map_err(debug_error)?
            .with_eidolon(
                EidolonLevel::new(parse_required(&row.eidolon, "trial Build Eidolon")?)
                    .ok_or_else(|| error("trial Build Eidolon is invalid"))?,
            );
            let source_equipment_id =
                parse_required::<u32>(&row.equipment_id, "trial Build Light Cone source ID")?;
            let cone_id = core
                .light_cone_for_source_equipment(source_equipment_id)
                .ok_or_else(|| error("trial Build Light Cone has no shared definition"))?;
            spec = spec.with_light_cone(LightConeLoadout::new(
                cone_id,
                LightConeLevel::new(parse_required(
                    &row.equipment_level,
                    "trial Build Light Cone level",
                )?)
                .ok_or_else(|| error("trial Build Light Cone level is invalid"))?,
                PromotionStage::new(parse_required(
                    &row.equipment_promotion,
                    "trial Build Light Cone promotion",
                )?)
                .ok_or_else(|| error("trial Build Light Cone promotion is invalid"))?,
                Superimposition::new(parse_required(
                    &row.equipment_rank,
                    "trial Build Light Cone rank",
                )?)
                .ok_or_else(|| error("trial Build Light Cone rank is invalid"))?,
            ));
            let relics = relic_properties(row)?;
            spec = spec.with_relic_stats(relics);
            let compiled = starclock_build::compiler::LoadoutCompiler
                .compile(core.build_catalog(), core.combat_catalog(), &spec)
                .map_err(debug_error)?;
            Ok(CurrencyWarsTrialBuild {
                stable_key: row.stable_key.clone().into(),
                role,
                avatar_id,
                special_avatar_id: parse_required(
                    &row.special_avatar_id,
                    "special trial avatar ID",
                )?,
                world_level: parse_required(&row.world_level, "trial Build world level")?,
                skill_tree_key: required(&row.skill_tree_key, "trial Build skill tree key")?.into(),
                relic_property_type: parse_required(
                    &row.relic_property_type,
                    "trial Build relic property type",
                )?,
                relic_property_type_extra: parse_required(
                    &row.relic_property_type_extra,
                    "trial Build extra relic property type",
                )?,
                relic_main_value: parse_required(
                    &row.relic_main_value,
                    "trial Build relic main value",
                )?,
                relic_sub_value: parse_required(
                    &row.relic_sub_value,
                    "trial Build relic sub value",
                )?,
                relic_sets: relic_set_thresholds(row)?.into_boxed_slice(),
                source_ability_bindings: source_ability_bindings(row, &core)?.into_boxed_slice(),
                effective_ability_levels: compiled.effective_ability_levels().into(),
                technique_ability,
                spec,
                combatant: compiled.combatant().clone(),
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct SourceAbilityBinding {
    source_skill_id: Box<str>,
    shared_ability_stable_key: Box<str>,
}

fn source_ability_bindings(
    row: &TrialBuildRow,
    core: &SimulationCatalog,
) -> Result<Vec<CurrencyWarsSourceAbilityBinding>, CurrencyWarsDataError> {
    let mut bindings = parse_json::<Vec<SourceAbilityBinding>>(
        row.source_ability_bindings.as_deref().unwrap_or("[]"),
    )?
    .into_iter()
    .map(|binding| {
        let source_skill_id = binding.source_skill_id.parse().map_err(debug_error)?;
        let shared_ability = core
            .ability_by_stable_key(&binding.shared_ability_stable_key)
            .ok_or_else(|| error("trial Build source Ability binding is not shared content"))?;
        Ok(CurrencyWarsSourceAbilityBinding {
            source_skill_id,
            shared_ability,
        })
    })
    .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    bindings.sort_unstable_by_key(|binding| binding.source_skill_id);
    if bindings
        .windows(2)
        .any(|pair| pair[0].source_skill_id == pair[1].source_skill_id)
    {
        return Err(error("trial Build source Ability binding is duplicated"));
    }
    Ok(bindings)
}

#[derive(Deserialize)]
struct RelicProperty {
    #[serde(rename = "FODBMMCKAEN")]
    kind: Box<str>,
    #[serde(rename = "MNDFOPKBHKP")]
    value: Option<Box<str>>,
}

#[derive(Deserialize)]
struct RelicSet {
    property_type: Box<str>,
    set_id: Box<str>,
    piece_count: Box<str>,
    ability_name: Box<str>,
    static_properties: Vec<RelicProperty>,
    ability_parameters: Vec<Box<str>>,
}

fn relic_set_thresholds(
    row: &TrialBuildRow,
) -> Result<Vec<CurrencyWarsRelicSetThreshold>, CurrencyWarsDataError> {
    parse_json::<Vec<RelicSet>>(required(&row.relic_sets, "trial Build relic set closure")?)?
        .into_iter()
        .map(|set| {
            Ok(CurrencyWarsRelicSetThreshold {
                property_type: set.property_type.parse().map_err(debug_error)?,
                set_id: set.set_id.parse().map_err(debug_error)?,
                piece_count: set.piece_count.parse().map_err(debug_error)?,
                ability_name: set.ability_name,
                ability_parameters: set.ability_parameters.into_boxed_slice(),
            })
        })
        .collect()
}

fn relic_properties(
    row: &TrialBuildRow,
) -> Result<starclock_build::spec::RelicStatContribution, CurrencyWarsDataError> {
    let mut values = BTreeMap::<Box<str>, Scalar>::new();
    let main = parse_json::<Vec<RelicProperty>>(required(
        &row.relic_main_properties,
        "trial Build relic main properties",
    )?)?;
    let sub = parse_json::<Vec<RelicProperty>>(required(
        &row.relic_sub_properties,
        "trial Build relic sub properties",
    )?)?;
    let sets =
        parse_json::<Vec<RelicSet>>(required(&row.relic_sets, "trial Build relic set closure")?)?;
    for property in main
        .into_iter()
        .chain(sub)
        .chain(sets.into_iter().flat_map(|set| set.static_properties))
    {
        let Some(raw) = property.value else {
            continue;
        };
        let value = parse_decimal_scalar(&raw)?;
        values
            .entry(property.kind)
            .and_modify(|current| {
                *current = current
                    .checked_add(value)
                    .expect("released relic aggregate remains in Scalar range");
            })
            .or_insert(value);
    }
    Ok(starclock_build::spec::RelicStatContribution::new(
        scalar(&values, "HPDelta"),
        scalar(&values, "AttackDelta"),
        scalar(&values, "DefenceDelta"),
        scalar(&values, "SpeedDelta"),
        scalar(&values, "HPAddedRatio"),
        scalar(&values, "AttackAddedRatio"),
        scalar(&values, "DefenceAddedRatio"),
        scalar(&values, "SpeedAddedRatio"),
        scalar(&values, "CriticalChanceBase"),
        scalar(&values, "CriticalDamageBase"),
        scalar(&values, "StatusProbabilityBase"),
        scalar(&values, "StatusResistanceBase"),
        scalar(&values, "BreakDamageAddedRatioBase"),
        scalar(&values, "SPRatioBase"),
        scalar(&values, "HealRatioBase"),
        [
            scalar(&values, "PhysicalAddedRatio"),
            scalar(&values, "FireAddedRatio"),
            scalar(&values, "IceAddedRatio"),
            scalar(&values, "ThunderAddedRatio"),
            scalar(&values, "WindAddedRatio"),
            scalar(&values, "QuantumAddedRatio"),
            scalar(&values, "ImaginaryAddedRatio"),
        ],
    ))
}

fn scalar(values: &BTreeMap<Box<str>, Scalar>, key: &str) -> Scalar {
    values.get(key).copied().unwrap_or(Scalar::ZERO)
}

fn parse_decimal_scalar(value: &str) -> Result<Scalar, CurrencyWarsDataError> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |unsigned| (true, unsigned));
    let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || fractional.len() > 6
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(error("trial Build relic decimal is not canonical"));
    }
    let whole = whole.parse::<i64>().map_err(debug_error)?;
    let fraction = if fractional.is_empty() {
        0
    } else {
        fractional.parse::<i64>().map_err(debug_error)?
            * 10_i64.pow(u32::try_from(6 - fractional.len()).expect("at most six digits"))
    };
    let scaled = whole
        .checked_mul(1_000_000)
        .and_then(|whole| whole.checked_add(fraction))
        .ok_or_else(|| error("trial Build relic decimal overflows Scalar"))?;
    Ok(Scalar::from_scaled(if negative { -scaled } else { scaled }))
}

fn lower_mappings(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsBuildMapping>, CurrencyWarsDataError> {
    config
        .currency_wars_build_mappings()
        .ordered_rows()
        .map(|row| {
            let role = role_id(required(&row.source_id, "Build role ID")?)?;
            let level = minimum(required(&row.level, "Build level policy")?)?;
            let trace_state = minimum(required(&row.trace_state, "Build Trace policy")?)?;
            let light_cone = minimum(required(&row.light_cone, "Build Light Cone policy")?)?;
            let relics = minimum(required(&row.relics, "Build relic policy")?)?;
            if level != CurrencyWarsBuildMinimum::AccountOrModeMinimum
                || trace_state != CurrencyWarsBuildMinimum::AccountOrModeMinimum
                || light_cone != CurrencyWarsBuildMinimum::AccountOrMappedMinimum
                || relics != CurrencyWarsBuildMinimum::AccountOrMappedMinimum
                || required(&row.account_mutation, "Build account mutation")? != "false"
            {
                return Err(error("Currency Wars Build policy is unknown"));
            }
            Ok(CurrencyWarsBuildMapping {
                stable_key: row.stable_key.clone().into(),
                role,
                avatar_id: parse_required(&row.avatar_id, "Build avatar ID")?,
                special_avatar_id: parse_required(
                    &row.special_avatar_id,
                    "special Build avatar ID",
                )?,
                level,
                trace_state,
                light_cone,
                relics,
                mutates_account: false,
            })
        })
        .collect()
}

fn lower_references(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsBuildReference>, CurrencyWarsDataError> {
    config
        .currency_wars_build_reference_avatars()
        .ordered_rows()
        .map(|row| {
            let source_role = role_id(stable_tail(&row.stable_key)?)?;
            let eligibility: BuildEligibility =
                parse_json(required(&row.eligibility, "Build eligibility")?)?;
            let role = role_id(&eligibility.role_id)?;
            let avatar_id = parse_required(&row.avatar_id, "Build-reference avatar ID")?;
            if role != source_role
                || !eligibility.in_pool
                || required(&row.owned_build_id, "owned Build ID")?
                    != format!("account-avatar:{avatar_id}")
                || required(&row.trial_build_id, "trial Build ID")?
                    != format!("gridfight-special-avatar:{}", eligibility.special_avatar_id)
            {
                return Err(error("Currency Wars Build reference is invalid"));
            }
            Ok(CurrencyWarsBuildReference {
                stable_key: row.stable_key.clone().into(),
                role,
                avatar_id,
                owned_build_id: required(&row.owned_build_id, "owned Build ID")?.into(),
                trial_build_id: required(&row.trial_build_id, "trial Build ID")?.into(),
                in_pool: true,
            })
        })
        .collect()
}

fn lower_sources(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsBuildSource>, CurrencyWarsDataError> {
    config
        .currency_wars_build_source_files()
        .ordered_rows()
        .map(|row| {
            if required(&row.mapping_role, "Build source role")? != "SharedBuildCandidate" {
                return Err(error("Currency Wars Build source policy is unknown"));
            }
            let disposition = match required(&row.disposition, "Build source disposition")? {
                "PendingExplicitRoleRowJoin" => {
                    CurrencyWarsBuildSourceDisposition::PendingExplicitRoleRowJoin
                }
                "ExplicitRoleRowJoin" => CurrencyWarsBuildSourceDisposition::ExplicitRoleRowJoin,
                _ => return Err(error("Currency Wars Build source policy is unknown")),
            };
            let sha256 = required(&row.source_sha256, "Build source digest")?;
            if sha256.len() != 64 || !sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(error("Currency Wars Build source digest is invalid"));
            }
            Ok(CurrencyWarsBuildSource {
                stable_key: row.stable_key.clone().into(),
                path: required(&row.source_path, "Build source path")?.into(),
                sha256: sha256.into(),
                role: CurrencyWarsBuildSourceRole::SharedBuildCandidate,
                disposition,
            })
        })
        .collect()
}

fn lower_substitution_rules(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsBuildSubstitutionRule>, CurrencyWarsDataError> {
    config
        .currency_wars_build_substitution_rules()
        .ordered_rows()
        .map(|row| {
            Ok(CurrencyWarsBuildSubstitutionRule {
                stable_key: row.stable_key.clone().into(),
                selection_timing: required(&row.selection_timing, "Build selection timing")?.into(),
                owned_trial_policy: required(&row.owned_trial_policy, "owned/trial policy")?.into(),
                refresh_timing: required(&row.refresh_timing, "Build refresh timing")?.into(),
                teardown: required(&row.teardown, "Build teardown")?.into(),
            })
        })
        .collect()
}

fn lower_equipment(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsEquipmentDefinition>, CurrencyWarsDataError> {
    config
        .currency_wars_equipment()
        .ordered_rows()
        .map(|row| {
            let stable_key = row.stable_key.as_str();
            let source_id = required(&row.source_id, "equipment source ID")?;
            let slot = required(&row.slot, "equipment slot")?;
            let (runtime, category_limit, character_slot_limit, character_implant_limit) =
                if stable_key.contains(".equipment.equipment.") {
                    let eligibility: Vec<String> =
                        parse_json(required(&row.eligibility, "equipment eligibility")?)?;
                    let parameters: EquipmentParameters =
                        parse_json(required(&row.parameters, "equipment parameters")?)?;
                    let id = CurrencyWarsEquipmentId::new(source_id.parse().map_err(debug_error)?)
                        .ok_or_else(|| error("Currency Wars equipment ID is zero"))?;
                    let category = equipment_category(slot)?;
                    let tags = parameters
                        .equipment_tag_list
                        .iter()
                        .map(|value| value.parse().map_err(debug_error))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice();
                    let rule = equipment_dress_rule(parameters.dress_rule.as_deref(), eligibility)?;
                    let properties = lower_properties(parameters.general_property_list)?;
                    let ability_name = parameters
                        .ability_name
                        .filter(|value| !value.is_empty())
                        .map(String::into_boxed_str);
                    let parameters = parameters
                        .param_list
                        .into_iter()
                        .map(|value| parse_decimal_scalar(&value))
                        .collect::<Result<Vec<_>, _>>()?
                        .into_boxed_slice();
                    (
                        Some(CurrencyWarsRuntimeEquipment {
                            id,
                            category,
                            tags,
                            dress_rule: rule,
                            properties,
                            ability_name,
                            parameters,
                        }),
                        None,
                        None,
                        None,
                    )
                } else if stable_key.contains(".equipment.equipmentcategory.") {
                    let eligibility: EquipmentMaximum = parse_json(required(
                        &row.eligibility,
                        "equipment category eligibility",
                    )?)?;
                    (
                        None,
                        Some(CurrencyWarsEquipmentCategoryLimit {
                            category: equipment_category(slot)?,
                            maximum: optional_maximum(&eligibility.maximum_count)?,
                        }),
                        None,
                        None,
                    )
                } else if stable_key.ends_with(".equipment.slot-cap.three-per-character") {
                    let eligibility: EquipmentMaximum =
                        parse_json(required(&row.eligibility, "equipment slot eligibility")?)?;
                    (
                        None,
                        None,
                        optional_maximum(&eligibility.maximum_count)?,
                        None,
                    )
                } else if stable_key.ends_with(".equipment.slot-cap.one-implant-per-character") {
                    let eligibility: EquipmentMaximum =
                        parse_json(required(&row.eligibility, "equipment implant eligibility")?)?;
                    (
                        None,
                        None,
                        None,
                        optional_maximum(&eligibility.maximum_count)?,
                    )
                } else {
                    (None, None, None, None)
                };
            Ok(CurrencyWarsEquipmentDefinition {
                stable_key: row.stable_key.clone().into(),
                source_id: source_id.into(),
                slot: slot.into(),
                eligibility_json: canonical_json(required(
                    &row.eligibility,
                    "equipment eligibility",
                )?)?,
                effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
                parameters_json: canonical_json(required(
                    &row.parameters,
                    "equipment parameters",
                )?)?,
                replacement_rule: required(&row.replacement_rule, "equipment replacement rule")?
                    .into(),
                runtime,
                category_limit,
                character_slot_limit,
                character_implant_limit,
            })
        })
        .collect()
}

fn lower_off_field_conversions(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsOffFieldConversion>, CurrencyWarsDataError> {
    let core = core_catalog::load(include_bytes!("../../../config/generated/config.sora"))
        .map_err(debug_error)?;
    let rank_roles = config
        .currency_wars_roster_avatars()
        .ordered_rows()
        .flat_map(|row| {
            let role = row.role_id.as_deref().and_then(|value| role_id(value).ok());
            row.backend_rank_ids
                .as_deref()
                .and_then(|value| parse_json::<Vec<String>>(value).ok())
                .into_iter()
                .flatten()
                .filter_map(move |rank| Some((rank.parse::<u32>().ok()?, role?)))
        })
        .collect::<BTreeMap<_, _>>();
    config
        .currency_wars_off_field_conversions()
        .ordered_rows()
        .map(|row| {
            let (source_kind, expected_destination) =
                match required(&row.source_kind, "conversion source kind")? {
                    "BackEquipment" => (
                        CurrencyWarsOffFieldSourceKind::BackEquipment,
                        CurrencyWarsOffFieldDestination::BackEquipmentContribution,
                    ),
                    "BackRoleRank" => (
                        CurrencyWarsOffFieldSourceKind::BackRoleRank,
                        CurrencyWarsOffFieldDestination::BackPositionContribution,
                    ),
                    _ => return Err(error("Currency Wars off-field source kind is unknown")),
                };
            let destination = match required(&row.destination_state, "conversion destination")? {
                "CurrencyWarsBackEquipmentContribution" => {
                    CurrencyWarsOffFieldDestination::BackEquipmentContribution
                }
                "CurrencyWarsBackPositionContribution" => {
                    CurrencyWarsOffFieldDestination::BackPositionContribution
                }
                _ => return Err(error("Currency Wars off-field destination is unknown")),
            };
            if destination != expected_destination {
                return Err(error(
                    "Currency Wars off-field source/destination pair is invalid",
                ));
            }
            let eligibility = match source_kind {
                CurrencyWarsOffFieldSourceKind::BackRoleRank => {
                    let value: BackRankEligibility =
                        parse_json(required(&row.eligibility, "rank conversion eligibility")?)?;
                    let rank_id = value.rank_id.parse().map_err(debug_error)?;
                    CurrencyWarsOffFieldEligibility::Eidolon {
                        role: rank_roles
                            .get(&rank_id)
                            .copied()
                            .ok_or_else(|| error("Currency Wars backend rank has no role join"))?,
                        rank_id,
                        rank: value.rank.parse().map_err(debug_error)?,
                    }
                }
                CurrencyWarsOffFieldSourceKind::BackEquipment => {
                    let value: BackEquipmentEligibility = parse_json(required(
                        &row.eligibility,
                        "equipment conversion eligibility",
                    )?)?;
                    let source_equipment = value.equipment_id.parse().map_err(debug_error)?;
                    CurrencyWarsOffFieldEligibility::SignatureLightCone {
                        role: role_id(&value.role_id)?,
                        light_cone: core
                            .light_cone_for_source_equipment(source_equipment)
                            .ok_or_else(|| {
                                error("Currency Wars back equipment has no shared Light Cone")
                            })?,
                        superimposition: value.level.parse().map_err(debug_error)?,
                    }
                }
            };
            let payload =
                lower_off_field_payload(required(&row.conversion, "conversion payload")?)?;
            Ok(CurrencyWarsOffFieldConversion {
                stable_key: row.stable_key.clone().into(),
                source_id: required(&row.source_id, "conversion source ID")?.into(),
                source_kind,
                eligibility,
                payload,
                destination,
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct RecommendationEligibility {
    role_ids: Vec<String>,
}

fn lower_recommendations(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsEquipmentRecommendation>, CurrencyWarsDataError> {
    config
        .currency_wars_equipment()
        .ordered_rows()
        .filter(|row| row.stable_key.contains(".equipmentrecommendation."))
        .map(|row| {
            let eligibility: RecommendationEligibility = parse_json(required(
                &row.eligibility,
                "equipment recommendation eligibility",
            )?)?;
            Ok(CurrencyWarsEquipmentRecommendation {
                equipment: CurrencyWarsEquipmentId::new(
                    required(&row.source_id, "recommended equipment source ID")?
                        .parse()
                        .map_err(debug_error)?,
                )
                .ok_or_else(|| error("recommended equipment ID is zero"))?,
                roles: eligibility
                    .role_ids
                    .iter()
                    .map(|value| role_id(value))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
            })
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct EquipmentParameters {
    #[serde(default)]
    dress_rule: Option<String>,
    #[serde(default)]
    ability_name: Option<String>,
    #[serde(default)]
    param_list: Vec<String>,
    #[serde(default)]
    general_property_list: Vec<PropertyValue>,
    #[serde(default)]
    equipment_tag_list: Vec<String>,
}

#[derive(Deserialize)]
struct EquipmentMaximum {
    maximum_count: String,
}

#[derive(Deserialize)]
struct PropertyValue {
    #[serde(rename = "PropertyType")]
    property_type: String,
    #[serde(rename = "Value")]
    value: String,
}

#[derive(Deserialize)]
struct BackRankEligibility {
    rank_id: String,
    rank: String,
}

#[derive(Deserialize)]
struct BackEquipmentEligibility {
    role_id: String,
    equipment_id: String,
    level: String,
}

#[derive(Deserialize)]
struct OffFieldPayload {
    #[serde(default)]
    owner_properties: Vec<PropertyValue>,
    #[serde(default)]
    all_member_properties: Vec<PropertyValue>,
    #[serde(default)]
    modified_skills: Vec<String>,
    #[serde(default)]
    rank_abilities: Vec<String>,
    #[serde(default)]
    parameters: Vec<String>,
}

pub(super) fn equipment_category(
    value: &str,
) -> Result<CurrencyWarsEquipmentCategory, CurrencyWarsDataError> {
    match value {
        "Artifacts" => Ok(CurrencyWarsEquipmentCategory::Artifacts),
        "Basic" => Ok(CurrencyWarsEquipmentCategory::Basic),
        "Craftable" => Ok(CurrencyWarsEquipmentCategory::Craftable),
        "Crown" => Ok(CurrencyWarsEquipmentCategory::Crown),
        "Emblem" => Ok(CurrencyWarsEquipmentCategory::Emblem),
        "FateEquip" => Ok(CurrencyWarsEquipmentCategory::FateEquip),
        "GoldTrash" => Ok(CurrencyWarsEquipmentCategory::GoldTrash),
        "Hack" => Ok(CurrencyWarsEquipmentCategory::Hack),
        "Material" => Ok(CurrencyWarsEquipmentCategory::Material),
        "Other" => Ok(CurrencyWarsEquipmentCategory::Other),
        "Radiant" => Ok(CurrencyWarsEquipmentCategory::Radiant),
        "Support" => Ok(CurrencyWarsEquipmentCategory::Support),
        "TraitSpecial" => Ok(CurrencyWarsEquipmentCategory::TraitSpecial),
        "Trash" => Ok(CurrencyWarsEquipmentCategory::Trash),
        _ => Err(error("Currency Wars equipment category is unknown")),
    }
}

fn equipment_dress_rule(
    rule: Option<&str>,
    values: Vec<String>,
) -> Result<CurrencyWarsEquipmentDressRule, CurrencyWarsDataError> {
    let roles = || {
        values
            .iter()
            .map(|value| role_id(value))
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    };
    let traits = || {
        values
            .iter()
            .map(|value| value.parse().map_err(debug_error))
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    };
    match rule {
        None if values.is_empty() => Ok(CurrencyWarsEquipmentDressRule::Any),
        Some("DressRuleAllSlotEmpty") if values.is_empty() => {
            Ok(CurrencyWarsEquipmentDressRule::AllSlotsEmpty)
        }
        Some("DressRuleUnique") if values.is_empty() => Ok(CurrencyWarsEquipmentDressRule::Unique),
        Some("DressRuleRoleOnly") => Ok(CurrencyWarsEquipmentDressRule::RoleOnly(roles()?)),
        Some("DressRuleTraitOnly") => Ok(CurrencyWarsEquipmentDressRule::TraitOnly(traits()?)),
        Some("DressRuleUniqueAndExclusiveTrait") => Ok(
            CurrencyWarsEquipmentDressRule::UniqueAndExclusiveTrait(traits()?),
        ),
        _ => Err(error("Currency Wars equipment dress rule is unknown")),
    }
}

fn lower_properties(
    values: Vec<PropertyValue>,
) -> Result<Box<[CurrencyWarsPropertyContribution]>, CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|value| {
            Ok(CurrencyWarsPropertyContribution {
                property: value.property_type.into_boxed_str(),
                value: parse_decimal_scalar(&value.value)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn lower_off_field_payload(
    value: &str,
) -> Result<CurrencyWarsOffFieldPayload, CurrencyWarsDataError> {
    let value: OffFieldPayload = parse_json(value)?;
    Ok(CurrencyWarsOffFieldPayload {
        owner_properties: lower_properties(value.owner_properties)?,
        all_member_properties: lower_properties(value.all_member_properties)?,
        modified_skills: value
            .modified_skills
            .into_iter()
            .map(|value| value.parse().map_err(debug_error))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        rank_abilities: value
            .rank_abilities
            .into_iter()
            .map(String::into_boxed_str)
            .collect(),
        parameters: value
            .parameters
            .into_iter()
            .map(|value| parse_decimal_scalar(&value))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    })
}

fn optional_maximum(value: &str) -> Result<Option<u8>, CurrencyWarsDataError> {
    if value == "undefined" {
        Ok(None)
    } else {
        Ok(Some(value.parse().map_err(debug_error)?))
    }
}

#[derive(Deserialize)]
struct BuildEligibility {
    role_id: String,
    special_avatar_id: String,
    in_pool: bool,
}

fn minimum(value: &str) -> Result<CurrencyWarsBuildMinimum, CurrencyWarsDataError> {
    match value {
        "AccountOrModeMinimum" => Ok(CurrencyWarsBuildMinimum::AccountOrModeMinimum),
        "AccountOrMappedMinimum" => Ok(CurrencyWarsBuildMinimum::AccountOrMappedMinimum),
        _ => Err(error("Currency Wars Build minimum policy is unknown")),
    }
}

fn role_id(value: &str) -> Result<CurrencyWarsRoleId, CurrencyWarsDataError> {
    CurrencyWarsRoleId::new(value.parse().map_err(debug_error)?)
        .ok_or_else(|| error("Currency Wars Build role ID is zero"))
}

fn stable_tail(value: &str) -> Result<&str, CurrencyWarsDataError> {
    value
        .rsplit('.')
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error("Currency Wars stable key has no tail"))
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

pub(super) fn canonical_json(value: &str) -> Result<Box<str>, CurrencyWarsDataError> {
    let parsed: serde_json::Value = parse_json(value)?;
    let canonical = serde_json::to_string(&parsed).map_err(debug_error)?;
    if canonical != value {
        return Err(error("Currency Wars authored JSON is not canonical"));
    }
    Ok(value.into())
}
