use serde::Deserialize;
use starclock_combat::Ratio;
use starclock_mode_currency_wars::{
    CurrencyWarsBackBattleEvent, CurrencyWarsBattleEventKind, CurrencyWarsBattleEventProperty,
    CurrencyWarsBattleEventPropertyKind, CurrencyWarsBattleEventTeam, CurrencyWarsBattleOverride,
    CurrencyWarsBattleOverrideDefinition, CurrencyWarsCharacterEmpowerment,
    CurrencyWarsCyreneSkillOverride, CurrencyWarsDecimal, CurrencyWarsEmpowermentCatalog,
    CurrencyWarsFrontSpecialResource, CurrencyWarsLethalRescueHpPolicy, CurrencyWarsPositionKind,
    CurrencyWarsRankSkillOverride, CurrencyWarsRoleGlobalModifier, CurrencyWarsRoleId,
    CurrencyWarsSkillParameterEdit, CurrencyWarsSkillParameterOperator,
    CurrencyWarsSpecialResourceKind, CurrencyWarsSummonBattleEventOverride,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

pub(super) fn lower_currency_wars_empowerment(
    config: &SoraConfig,
) -> Result<CurrencyWarsEmpowermentCatalog, CurrencyWarsDataError> {
    CurrencyWarsEmpowermentCatalog::new(
        lower_empowerments(config)?,
        lower_battle_overrides(config)?,
    )
    .map_err(debug_error)
}

fn lower_empowerments(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsCharacterEmpowerment>, CurrencyWarsDataError> {
    config
        .currency_wars_character_empowerments()
        .ordered_rows()
        .map(|row| {
            let avatar_id = parse_optional(row.avatar_id.as_ref(), "Empowerment avatar ID")?;
            let skill_level = parse_optional(row.skill_level.as_ref(), "Empowerment skill level")?;
            if avatar_id.is_some() == skill_level.is_some() {
                return Err(error(
                    "Currency Wars Empowerment must be exactly one of display or skill",
                ));
            }
            Ok(CurrencyWarsCharacterEmpowerment {
                stable_key: row.stable_key.clone().into(),
                source_id: required(&row.source_id, "Empowerment source ID")?.into(),
                avatar_id,
                skill_id: skill_level
                    .map(|_| skill_id(required(&row.source_id, "Empowerment source ID")?))
                    .transpose()?,
                position: position(required(&row.position_id, "Empowerment position")?)?,
                activation: required(&row.activation, "Empowerment activation")?.into(),
                effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
                category_tags: parse_boxed_strings(row.category_tags.as_ref())?,
                skill_level,
                cooldown: parse_optional(row.cooldown.as_ref(), "Empowerment cooldown")?,
                initial_cooldown: parse_optional(
                    row.initial_cooldown.as_ref(),
                    "Empowerment initial cooldown",
                )?,
                sp_multiple_ratio: optional_text(row.sp_multiple_ratio.as_ref()),
                delay_ratio: optional_text(row.delay_ratio.as_ref()),
                parameter_values: decimals(
                    parse_boxed_strings(row.parameter_values.as_ref())?.into_vec(),
                )?,
                teardown: required(&row.teardown, "Empowerment teardown")?.into(),
            })
        })
        .collect()
}

fn skill_id(source_id: &str) -> Result<u32, CurrencyWarsDataError> {
    source_id
        .split_once(':')
        .map(|(skill, _)| skill)
        .ok_or_else(|| error("Currency Wars Empowerment skill source ID is invalid"))?
        .parse()
        .map_err(debug_error)
}

fn lower_battle_overrides(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsBattleOverride>, CurrencyWarsDataError> {
    config
        .currency_wars_battle_overrides()
        .ordered_rows()
        .map(|row| {
            let rule_kind = required(&row.rule_kind, "battle-override rule kind")?;
            let trigger = required(&row.trigger, "battle-override trigger")?;
            let parameters = required(&row.parameters, "battle-override parameters")?;
            let source_id = required(&row.source_id, "battle-override source ID")?;
            Ok(CurrencyWarsBattleOverride {
                stable_key: row.stable_key.clone().into(),
                source_id: source_id.into(),
                definition: battle_override(rule_kind, trigger, parameters, source_id)?,
                teardown: required(&row.teardown, "battle-override teardown")?.into(),
            })
        })
        .collect()
}

#[derive(Deserialize)]
struct AutomaticTechniqueParameters {
    eligible_position: Box<str>,
}

#[derive(Deserialize)]
struct DefeatEnergyParameters {
    regular_energy_ratio: Box<str>,
}

#[derive(Deserialize)]
struct LethalRescueParameters {
    restored_hp: Box<str>,
    countdown_loss: Box<str>,
}

#[derive(Deserialize)]
struct BackBattleEventParameters {
    team: Box<str>,
    abilities: Vec<Box<str>>,
    speed: Box<str>,
    #[serde(default)]
    hard_level: bool,
    values: Vec<Box<str>>,
    override_properties: Vec<BattleEventPropertyParameters>,
}

#[derive(Deserialize)]
struct BattleEventPropertyParameters {
    property_type: Box<str>,
    value: Box<str>,
}

#[derive(Deserialize)]
struct FrontSpecialResourceParameters {
    role_id: Box<str>,
    star: Box<str>,
    special_sp_type: Box<str>,
    maximum: Box<str>,
}

#[derive(Deserialize)]
struct RoleGlobalModifierParameters {
    role_id: Box<str>,
    saved_value: Box<str>,
    values: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct SkillOverrideParameters {
    rank_id: Box<str>,
    skill_id: Box<str>,
    indexes: Vec<Box<str>>,
    operators: Vec<Box<str>>,
    values: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct SummonBattleEventParameters {
    season_id: Box<str>,
    battle_event_id: Box<str>,
    front_json: Box<str>,
    back_json: Box<str>,
}

#[derive(Deserialize)]
struct CyreneSkillOverrideParameters {
    provider_role_id: Box<str>,
    role_id: Box<str>,
    skill_id: Box<str>,
    indexes: Vec<Box<str>>,
    operators: Vec<Box<str>>,
    values: Vec<Box<str>>,
    multiple_value_key: Box<str>,
}

fn battle_override(
    kind: &str,
    trigger: &str,
    parameters: &str,
    source_id: &str,
) -> Result<CurrencyWarsBattleOverrideDefinition, CurrencyWarsDataError> {
    match kind {
        "AutomaticTechnique" if trigger == "BeforeBattleStart" => {
            let value = parse_json::<AutomaticTechniqueParameters>(parameters)?;
            Ok(CurrencyWarsBattleOverrideDefinition::AutomaticTechnique {
                eligible_position: position_name(&value.eligible_position)?,
            })
        }
        "DefeatEnergyScaling" if trigger == "EnemyDefeated" => {
            let value = parse_json::<DefeatEnergyParameters>(parameters)?;
            Ok(CurrencyWarsBattleOverrideDefinition::DefeatEnergyScaling {
                regular_energy_ratio: Ratio::from_scaled(decimal_scaled(
                    &value.regular_energy_ratio,
                )?),
            })
        }
        "LethalDamageRescue" if trigger == "BeforeRoleIncapacitated" => {
            let value = parse_json::<LethalRescueParameters>(parameters)?;
            if value.restored_hp.as_ref() != "FullMaximumHp"
                || value.countdown_loss.as_ref() != "PenaltyRuleAvatarReviveDelayLose"
            {
                return Err(error("Currency Wars lethal-rescue policy is unknown"));
            }
            Ok(CurrencyWarsBattleOverrideDefinition::LethalDamageRescue {
                hp_policy: CurrencyWarsLethalRescueHpPolicy::FullMaximumHp,
            })
        }
        "BackBattleEvent" => {
            let value = parse_json::<BackBattleEventParameters>(parameters)?;
            let event_id = source_id.parse().map_err(debug_error)?;
            Ok(CurrencyWarsBattleOverrideDefinition::BackBattleEvent(
                CurrencyWarsBackBattleEvent {
                    event_id,
                    kind: battle_event_kind(trigger)?,
                    team: battle_event_team(&value.team)?,
                    abilities: value.abilities.into_boxed_slice(),
                    speed: optional_decimal(&value.speed)?,
                    hard_level: value.hard_level,
                    values: decimals(value.values)?,
                    properties: value
                        .override_properties
                        .into_iter()
                        .map(|property| {
                            Ok(CurrencyWarsBattleEventProperty {
                                kind: battle_event_property(&property.property_type)?,
                                value: decimal(&property.value)?,
                            })
                        })
                        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?
                        .into_boxed_slice(),
                },
            ))
        }
        "FrontSpecialSP" if trigger == "BattleEntry" => {
            let value = parse_json::<FrontSpecialResourceParameters>(parameters)?;
            Ok(CurrencyWarsBattleOverrideDefinition::FrontSpecialResource(
                CurrencyWarsFrontSpecialResource {
                    role: role(&value.role_id)?,
                    star: value.star.parse().map_err(debug_error)?,
                    kind: special_resource_kind(&value.special_sp_type)?,
                    maximum: decimal(&value.maximum)?,
                },
            ))
        }
        "RoleGlobalModifier" if trigger == "RoleStateProjection" => {
            let value = parse_json::<RoleGlobalModifierParameters>(parameters)?;
            Ok(CurrencyWarsBattleOverrideDefinition::RoleGlobalModifier(
                CurrencyWarsRoleGlobalModifier {
                    role: role(&value.role_id)?,
                    saved_value: optional_boxed(value.saved_value),
                    values: decimals(value.values)?,
                },
            ))
        }
        "RankSkillModify" if trigger == "RankContribution" => {
            let value = parse_json::<SkillOverrideParameters>(parameters)?;
            Ok(CurrencyWarsBattleOverrideDefinition::RankSkillOverride(
                CurrencyWarsRankSkillOverride {
                    rank_id: value.rank_id.parse().map_err(debug_error)?,
                    skill_id: value.skill_id.parse().map_err(debug_error)?,
                    edits: skill_edits(value.indexes, value.operators, value.values)?,
                },
            ))
        }
        "SummonBattleEventOverride" if trigger == "SummonBattleEvent" => {
            let value = parse_json::<SummonBattleEventParameters>(parameters)?;
            let override_ = CurrencyWarsSummonBattleEventOverride {
                season_id: value.season_id.parse().map_err(debug_error)?,
                battle_event_id: value.battle_event_id.parse().map_err(debug_error)?,
                front_config: optional_boxed(value.front_json),
                back_config: optional_boxed(value.back_json),
            };
            if override_.front_config.is_some() == override_.back_config.is_some() {
                return Err(error("Currency Wars summon override position is ambiguous"));
            }
            Ok(CurrencyWarsBattleOverrideDefinition::SummonBattleEventOverride(override_))
        }
        "CyreneSkillModify" if trigger == "CyreneContribution" => {
            let value = parse_json::<CyreneSkillOverrideParameters>(parameters)?;
            Ok(CurrencyWarsBattleOverrideDefinition::CyreneSkillOverride(
                CurrencyWarsCyreneSkillOverride {
                    provider_role: role(&value.provider_role_id)?,
                    role: role(&value.role_id)?,
                    skill_id: value.skill_id.parse().map_err(debug_error)?,
                    multiple_value_key: value.multiple_value_key,
                    edits: skill_edits(value.indexes, value.operators, value.values)?,
                },
            ))
        }
        _ => Err(error(
            "Currency Wars battle-override kind or trigger is unknown",
        )),
    }
}

fn position_name(value: &str) -> Result<CurrencyWarsPositionKind, CurrencyWarsDataError> {
    match value {
        "Front" => Ok(CurrencyWarsPositionKind::Front),
        "Back" => Ok(CurrencyWarsPositionKind::Back),
        _ => Err(error("Currency Wars battle-override position is unknown")),
    }
}

fn battle_event_kind(value: &str) -> Result<CurrencyWarsBattleEventKind, CurrencyWarsDataError> {
    match value {
        "AssistEvent" => Ok(CurrencyWarsBattleEventKind::Assist),
        "BEServant" => Ok(CurrencyWarsBattleEventKind::Servant),
        "DummyCharacter" => Ok(CurrencyWarsBattleEventKind::DummyCharacter),
        "GridFightCountDownWarningEvent" => Ok(CurrencyWarsBattleEventKind::CountdownWarning),
        "GridFightTraitAssistEvent" => Ok(CurrencyWarsBattleEventKind::TraitAssist),
        _ => Err(error("Currency Wars back battle-event kind is unknown")),
    }
}

fn battle_event_team(value: &str) -> Result<CurrencyWarsBattleEventTeam, CurrencyWarsDataError> {
    match value {
        "TeamLight" => Ok(CurrencyWarsBattleEventTeam::Player),
        "TeamNeutral" => Ok(CurrencyWarsBattleEventTeam::Neutral),
        _ => Err(error("Currency Wars back battle-event team is unknown")),
    }
}

fn battle_event_property(
    value: &str,
) -> Result<CurrencyWarsBattleEventPropertyKind, CurrencyWarsDataError> {
    match value {
        "AllDamageTypeAddedRatio" => {
            Ok(CurrencyWarsBattleEventPropertyKind::AllDamageTypeAddedRatio)
        }
        "AttackAddedRatio" => Ok(CurrencyWarsBattleEventPropertyKind::AttackAddedRatio),
        "AttackDelta" => Ok(CurrencyWarsBattleEventPropertyKind::AttackDelta),
        "BaseAttack" => Ok(CurrencyWarsBattleEventPropertyKind::BaseAttack),
        "BaseDefence" => Ok(CurrencyWarsBattleEventPropertyKind::BaseDefence),
        "BaseHP" => Ok(CurrencyWarsBattleEventPropertyKind::BaseHp),
        "CriticalChance" => Ok(CurrencyWarsBattleEventPropertyKind::CriticalChance),
        "CriticalDamage" => Ok(CurrencyWarsBattleEventPropertyKind::CriticalDamage),
        "FireAddedRatio" => Ok(CurrencyWarsBattleEventPropertyKind::FireAddedRatio),
        "FirePenetrate" => Ok(CurrencyWarsBattleEventPropertyKind::FirePenetration),
        "MaxSP" => Ok(CurrencyWarsBattleEventPropertyKind::MaximumEnergy),
        "StatusProbability" => Ok(CurrencyWarsBattleEventPropertyKind::StatusProbability),
        _ => Err(error("Currency Wars back battle-event property is unknown")),
    }
}

fn special_resource_kind(
    value: &str,
) -> Result<CurrencyWarsSpecialResourceKind, CurrencyWarsDataError> {
    match value {
        "EnergyBar" => Ok(CurrencyWarsSpecialResourceKind::EnergyBar),
        "MaxSP" => Ok(CurrencyWarsSpecialResourceKind::MaximumEnergy),
        _ => Err(error("Currency Wars special resource kind is unknown")),
    }
}

fn skill_edits(
    indexes: Vec<Box<str>>,
    operators: Vec<Box<str>>,
    values: Vec<Box<str>>,
) -> Result<Box<[CurrencyWarsSkillParameterEdit]>, CurrencyWarsDataError> {
    if indexes.is_empty() || indexes.len() != operators.len() || indexes.len() != values.len() {
        return Err(error(
            "Currency Wars skill parameter edit columns are misaligned",
        ));
    }
    indexes
        .into_iter()
        .zip(operators)
        .zip(values)
        .map(|((index, operator), value)| {
            Ok(CurrencyWarsSkillParameterEdit {
                index: index.parse().map_err(debug_error)?,
                operator: match operator.as_ref() {
                    "Add" => CurrencyWarsSkillParameterOperator::Add,
                    "Mul" => CurrencyWarsSkillParameterOperator::Multiply,
                    "Set" => CurrencyWarsSkillParameterOperator::Set,
                    _ => return Err(error("Currency Wars skill parameter operator is unknown")),
                },
                value: decimal(&value)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn decimals(values: Vec<Box<str>>) -> Result<Box<[CurrencyWarsDecimal]>, CurrencyWarsDataError> {
    values
        .into_iter()
        .map(|value| decimal(&value))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn optional_decimal(value: &str) -> Result<Option<CurrencyWarsDecimal>, CurrencyWarsDataError> {
    (!value.is_empty()).then(|| decimal(value)).transpose()
}

fn decimal(value: &str) -> Result<CurrencyWarsDecimal, CurrencyWarsDataError> {
    let (negative, unsigned) = value
        .strip_prefix('-')
        .map_or((false, value), |unsigned| (true, unsigned));
    let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || fractional.len() > 18
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(error("Currency Wars battle-override decimal is invalid"));
    }
    let digits = format!("{whole}{fractional}");
    let significand = digits.parse::<i64>().map_err(debug_error)?;
    CurrencyWarsDecimal::new(
        if negative { -significand } else { significand },
        u8::try_from(fractional.len()).expect("at most 18 decimal places"),
    )
    .ok_or_else(|| error("Currency Wars battle-override decimal scale is invalid"))
}

fn decimal_scaled(value: &str) -> Result<i64, CurrencyWarsDataError> {
    let value = decimal(value)?;
    if value.decimal_places() > 6 {
        return Err(error("Currency Wars Ratio exceeds six decimal places"));
    }
    value
        .significand()
        .checked_mul(10_i64.pow(u32::from(6 - value.decimal_places())))
        .ok_or_else(|| error("Currency Wars Ratio overflows"))
}

fn role(value: &str) -> Result<CurrencyWarsRoleId, CurrencyWarsDataError> {
    value
        .parse::<u32>()
        .ok()
        .and_then(CurrencyWarsRoleId::new)
        .ok_or_else(|| error("Currency Wars battle-override role is invalid"))
}

fn optional_boxed(value: Box<str>) -> Option<Box<str>> {
    (!value.is_empty()).then_some(value)
}

fn position(value: &str) -> Result<CurrencyWarsPositionKind, CurrencyWarsDataError> {
    match value {
        "currency-wars.position.front" => Ok(CurrencyWarsPositionKind::Front),
        "currency-wars.position.back" => Ok(CurrencyWarsPositionKind::Back),
        _ => Err(error("Currency Wars Empowerment position is unknown")),
    }
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
