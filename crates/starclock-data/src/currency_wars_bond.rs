use std::collections::BTreeSet;

use starclock_combat::Ratio;
use starclock_mode_currency_wars::{
    CurrencyWarsBondActivation, CurrencyWarsBondCatalog, CurrencyWarsBondContribution,
    CurrencyWarsBondDefinition, CurrencyWarsBondId, CurrencyWarsBondLevel, CurrencyWarsBondMember,
    CurrencyWarsBondPropertyContribution, CurrencyWarsBondPropertyKind,
    CurrencyWarsBondPropertyScope, CurrencyWarsBondRecompute, CurrencyWarsBondSelectionRule,
    CurrencyWarsEquipmentId, CurrencyWarsRoleId,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_build::canonical_json,
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

type LoweredBondContribution = (Option<Box<str>>, CurrencyWarsBondContribution);

pub(super) fn lower_currency_wars_bonds(
    config: &SoraConfig,
) -> Result<CurrencyWarsBondCatalog, CurrencyWarsDataError> {
    let roster_roles = config
        .currency_wars_roster_avatars()
        .ordered_rows()
        .map(|row| role_id(required(&row.role_id, "roster role ID")?))
        .collect::<Result<BTreeSet<_>, _>>()?;
    CurrencyWarsBondCatalog::assemble(
        lower_definitions(config, &roster_roles)?,
        lower_levels(config)?,
        lower_contributions(config)?,
    )
    .map_err(debug_error)
}

fn lower_definitions(
    config: &SoraConfig,
    roster_roles: &BTreeSet<CurrencyWarsRoleId>,
) -> Result<Vec<CurrencyWarsBondDefinition>, CurrencyWarsDataError> {
    config
        .currency_wars_bonds()
        .ordered_rows()
        .map(|row| {
            let members = parse_json::<Vec<String>>(required(&row.member_ids, "Bond members")?)?
                .into_iter()
                .map(|stable_key| {
                    let role = role_id(stable_tail(&stable_key)?)?;
                    Ok(if roster_roles.contains(&role) {
                        CurrencyWarsBondMember::RosterRole(role)
                    } else {
                        CurrencyWarsBondMember::ExternalAuthoredRole(role)
                    })
                })
                .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
            Ok(CurrencyWarsBondDefinition {
                id: CurrencyWarsBondId::new(
                    stable_tail(&row.stable_key)?.parse().map_err(debug_error)?,
                )
                .ok_or_else(|| error("Currency Wars Bond ID is zero"))?,
                stable_key: row.stable_key.clone().into(),
                source_id: required(&row.source_id, "Bond source ID")?.into(),
                parent: optional_text(row.parent_bond_id.as_ref())
                    .map(|stable_key| bond_id(stable_tail(&stable_key)?))
                    .transpose()?,
                members: members.into_boxed_slice(),
                selection_rules: lower_selection_rules(row.selection_rules.as_ref())?,
                level_ids: parse_boxed_strings(row.level_ids.as_ref())?,
                activation: activation(required(&row.activation_type, "Bond activation")?)?,
                recompute: recompute(required(&row.recompute_timing, "Bond recompute timing")?)?,
                contribution_ids: parse_boxed_strings(row.contribution_ids.as_ref())?,
                trait_effect_ids: parse_number_array(row.trait_effect_ids.as_ref())?,
                battle_event_ids: parse_number_array(row.battle_event_ids.as_ref())?,
            })
        })
        .collect()
}

#[derive(serde::Deserialize)]
struct SelectionRuleRow {
    kind: String,
    source_id: Option<String>,
}

fn lower_selection_rules(
    value: Option<&String>,
) -> Result<Box<[CurrencyWarsBondSelectionRule]>, CurrencyWarsDataError> {
    parse_json::<Vec<SelectionRuleRow>>(value.map_or("[]", String::as_str))?
        .into_iter()
        .map(|row| match row.kind.as_str() {
            "DeployedRole" => Ok(CurrencyWarsBondSelectionRule::DeployedRole(role_id(
                required(&row.source_id, "sub-Bond role selector")?,
            )?)),
            "EquippedEquipment" => Ok(CurrencyWarsBondSelectionRule::EquippedEquipment(
                CurrencyWarsEquipmentId::new(
                    required(&row.source_id, "sub-Bond equipment selector")?
                        .parse()
                        .map_err(debug_error)?,
                )
                .ok_or_else(|| error("Currency Wars sub-Bond equipment ID is zero"))?,
            )),
            "GrantedFrontTrait" => Ok(CurrencyWarsBondSelectionRule::GrantedFrontTrait(role_id(
                required(&row.source_id, "sub-Bond granted-trait selector")?,
            )?)),
            "DefaultModule" => Ok(CurrencyWarsBondSelectionRule::DefaultModule),
            "Module" => Ok(CurrencyWarsBondSelectionRule::Module(
                required(&row.source_id, "sub-Bond module selector")?
                    .parse()
                    .map_err(debug_error)?,
            )),
            _ => Err(error("Currency Wars sub-Bond selection rule is unknown")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn lower_levels(
    config: &SoraConfig,
) -> Result<Vec<(Box<str>, CurrencyWarsBondLevel)>, CurrencyWarsDataError> {
    config
        .currency_wars_bond_levels()
        .ordered_rows()
        .map(|row| {
            if required(&row.threshold_semantics, "Bond threshold semantics")?
                != "AuthoredTraitLayer"
                || required(&row.property_bind_type, "Bond property bind type")? != "SpecificScope"
            {
                return Err(error("Currency Wars Bond level policy is unknown"));
            }
            Ok((
                required(&row.bond_id, "Bond-level parent")?.into(),
                CurrencyWarsBondLevel {
                    stable_key: row.stable_key.clone().into(),
                    source_id: required(&row.source_id, "Bond-level source ID")?.into(),
                    level: parse_required(&row.level, "Bond level")?,
                    threshold: parse_required(&row.threshold, "Bond threshold")?,
                    threshold_semantics: "AuthoredTraitLayer".into(),
                    property_bind_type: "SpecificScope".into(),
                    property_parameters_json: canonical_json(required(
                        &row.property_parameters,
                        "Bond property parameters",
                    )?)?,
                    properties: lower_properties(
                        required(&row.trait_member_properties, "Bond trait-member properties")?,
                        required(&row.all_member_properties, "Bond all-member properties")?,
                    )?,
                    effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
                    trait_member_properties_json: canonical_json(required(
                        &row.trait_member_properties,
                        "Bond trait-member properties",
                    )?)?,
                    all_member_properties_json: canonical_json(required(
                        &row.all_member_properties,
                        "Bond all-member properties",
                    )?)?,
                    override_battle_event_properties_json: canonical_json(required(
                        &row.override_battle_event_properties,
                        "Bond battle-event properties",
                    )?)?,
                },
            ))
        })
        .collect()
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PropertyRow {
    property_type: String,
    value: String,
}

fn lower_properties(
    member_json: &str,
    all_json: &str,
) -> Result<Box<[CurrencyWarsBondPropertyContribution]>, CurrencyWarsDataError> {
    let mut rows = parse_json::<Vec<PropertyRow>>(member_json)?
        .into_iter()
        .map(|row| (CurrencyWarsBondPropertyScope::BondMembers, row))
        .collect::<Vec<_>>();
    rows.extend(
        parse_json::<Vec<PropertyRow>>(all_json)?
            .into_iter()
            .map(|row| (CurrencyWarsBondPropertyScope::AllDeployed, row)),
    );
    rows.into_iter()
        .map(|(scope, row)| {
            Ok(CurrencyWarsBondPropertyContribution {
                scope,
                kind: property_kind(&row.property_type)?,
                value: Ratio::from_scaled(parse_decimal(&row.value)?),
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn parse_decimal(source: &str) -> Result<i64, CurrencyWarsDataError> {
    let (negative, unsigned) = source
        .strip_prefix('-')
        .map_or((false, source), |rest| (true, rest));
    let mut parts = unsigned.split('.');
    let integer = parts.next().unwrap_or_default();
    let fraction = parts.next();
    if unsigned.is_empty()
        || (negative && unsigned == "0")
        || parts.next().is_some()
        || integer.is_empty()
        || !integer.bytes().all(|byte| byte.is_ascii_digit())
        || (integer.len() > 1 && integer.starts_with('0'))
        || fraction.is_some_and(|value| {
            value.is_empty()
                || value.len() > 6
                || !value.bytes().all(|byte| byte.is_ascii_digit())
                || value.ends_with('0')
        })
    {
        return Err(error("Currency Wars Bond decimal is not canonical"));
    }
    let integer = integer.parse::<i128>().map_err(debug_error)?;
    let fraction = fraction.unwrap_or("");
    let fraction = if fraction.is_empty() {
        0
    } else {
        fraction.parse::<i128>().map_err(debug_error)?
            * 10_i128.pow(6 - u32::try_from(fraction.len()).map_err(debug_error)?)
    };
    let magnitude = integer
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(fraction))
        .ok_or_else(|| error("Currency Wars Bond decimal overflows"))?;
    i64::try_from(if negative { -magnitude } else { magnitude }).map_err(debug_error)
}

fn property_kind(value: &str) -> Result<CurrencyWarsBondPropertyKind, CurrencyWarsDataError> {
    match value {
        "ExtraAllDamageTypeAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::AllDamage),
        "ExtraAllDamageTypeAddedRatio5" => Ok(CurrencyWarsBondPropertyKind::AllDamageSecondary),
        "ExtraBackPowerAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::BackPower),
        "ExtraDOTDamageAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::DamageOverTime),
        "ExtraElementDamageAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::ElementDamage),
        "ExtraFrontPowerAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::FrontPower),
        "ExtraHPAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::Hp),
        "ExtraHealAddedRatio" => Ok(CurrencyWarsBondPropertyKind::Healing),
        "ExtraInsertDamageAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::InsertDamage),
        "ExtraLuckChance" => Ok(CurrencyWarsBondPropertyKind::LuckChance),
        "ExtraLuckDamage" => Ok(CurrencyWarsBondPropertyKind::LuckDamage),
        "ExtraNormalDamageAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::NormalDamage),
        "ExtraShieldAddedRatio" => Ok(CurrencyWarsBondPropertyKind::Shield),
        "ExtraSkillDamageAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::SkillDamage),
        "ExtraSpeedAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::Speed),
        "ExtraUltraDamageAddedRatio1" => Ok(CurrencyWarsBondPropertyKind::UltimateDamage),
        _ => Err(error("Currency Wars Bond property kind is unknown")),
    }
}

fn lower_contributions(
    config: &SoraConfig,
) -> Result<Vec<LoweredBondContribution>, CurrencyWarsDataError> {
    config
        .currency_wars_bond_contributions()
        .ordered_rows()
        .map(|row| {
            Ok((
                optional_text(row.bond_id.as_ref()),
                CurrencyWarsBondContribution {
                    stable_key: row.stable_key.clone().into(),
                    source_id: required(&row.source_id, "Bond-contribution source ID")?.into(),
                    level: parse_optional(row.level.as_ref(), "Bond-contribution level")?,
                    scope: required(&row.scope, "Bond-contribution scope")?.into(),
                    activation: required(&row.activation, "Bond-contribution activation")?.into(),
                    ordered_effects: parse_boxed_strings(row.ordered_effects.as_ref())?,
                    parameters_json: canonical_json(required(
                        &row.parameters,
                        "Bond-contribution parameters",
                    )?)?,
                },
            ))
        })
        .collect()
}

fn activation(value: &str) -> Result<CurrencyWarsBondActivation, CurrencyWarsDataError> {
    match value {
        "GreaterEqualThan" => Ok(CurrencyWarsBondActivation::GreaterEqualThan),
        "ExplicitSubTraitSelection" => Ok(CurrencyWarsBondActivation::ExplicitSubTraitSelection),
        _ => Err(error("Currency Wars Bond activation is unknown")),
    }
}

fn recompute(value: &str) -> Result<CurrencyWarsBondRecompute, CurrencyWarsDataError> {
    match value {
        "Recompute after an ordered roster mutation and before battle contribution projection." => {
            Ok(CurrencyWarsBondRecompute::OrderedRosterMutationBeforeBattleProjection)
        }
        "Recompute after the parent Bond's explicit sub-trait selection changes." => {
            Ok(CurrencyWarsBondRecompute::ExplicitSubTraitSelectionChange)
        }
        _ => Err(error("Currency Wars Bond recompute timing is unknown")),
    }
}

fn parse_number_array(value: Option<&String>) -> Result<Box<[u32]>, CurrencyWarsDataError> {
    parse_json::<Vec<String>>(value.map_or("[]", String::as_str))?
        .into_iter()
        .map(|value| value.parse().map_err(debug_error))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn role_id(value: &str) -> Result<CurrencyWarsRoleId, CurrencyWarsDataError> {
    CurrencyWarsRoleId::new(value.parse().map_err(debug_error)?)
        .ok_or_else(|| error("Currency Wars Bond role ID is zero"))
}

fn bond_id(value: &str) -> Result<CurrencyWarsBondId, CurrencyWarsDataError> {
    CurrencyWarsBondId::new(value.parse().map_err(debug_error)?)
        .ok_or_else(|| error("Currency Wars Bond ID is zero"))
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

fn parse_optional<T: std::str::FromStr>(
    value: Option<&String>,
    name: &str,
) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.parse().map_err(debug_error))
        .transpose()
        .map_err(|error| error.context(name))
}

fn optional_text(value: Option<&String>) -> Option<Box<str>> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.clone().into_boxed_str())
}
