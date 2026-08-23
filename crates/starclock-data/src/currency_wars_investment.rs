use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_mode_currency_wars::{
    CurrencyWarsAugmentCatalog, CurrencyWarsAugmentDefinition, CurrencyWarsAugmentLifecycle,
    CurrencyWarsAugmentMonsterRule, CurrencyWarsAugmentQuality, CurrencyWarsAugmentRemark,
    CurrencyWarsDecimal, CurrencyWarsEnhancement, CurrencyWarsEnhancementSelectCondition,
    CurrencyWarsInvestmentId, CurrencyWarsInvestmentMazeBuff, CurrencyWarsSelectedEnhancement,
    CurrencyWarsSelectedEnhancementId,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

#[derive(Deserialize)]
struct LifecycleRow {
    saved_values: Vec<Box<str>>,
    overclock_effective: Box<str>,
    effect_parameters: Vec<Box<str>>,
    description_parameters: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct RemarkRow {
    en: Box<str>,
    zh_cn: Box<str>,
}

#[derive(Deserialize)]
struct SelectedEnhancementParameters {
    select_condition: Box<str>,
    parameters: Vec<Box<str>>,
    effects: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct EnhancementParameters {
    select_condition: Box<str>,
    effects: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct MazeBuffLevel {
    current: Box<str>,
    maximum: Box<str>,
}

#[derive(Deserialize)]
struct MazeBuffBinding {
    #[serde(rename = "type")]
    kind: Box<str>,
    key: Box<str>,
    maze_buff_type: Box<str>,
}

#[derive(Deserialize)]
struct MonsterParameters {
    division_level: Box<str>,
    enemy_difficulty_level_add: Box<str>,
}

pub(super) fn lower_currency_wars_augments(
    config: &SoraConfig,
) -> Result<CurrencyWarsAugmentCatalog, CurrencyWarsDataError> {
    let mut seasons = BTreeMap::<u32, Vec<u16>>::new();
    for row in config
        .currency_wars_season_augment_memberships()
        .ordered_rows()
    {
        seasons
            .entry(parsed(required(&row.augment_id, "season Augment ID")?)?)
            .or_default()
            .push(parsed(required(&row.season_id, "season ID")?)?);
    }
    let mut remarks = BTreeMap::<u32, CurrencyWarsAugmentRemark>::new();
    for row in config.currency_wars_augment_remarks().ordered_rows() {
        let parsed: RemarkRow = parse_json(required(&row.remark, "Augment remark")?)?;
        let augment = parsed_id(required(&row.augment_id, "remark Augment ID")?)?;
        if remarks
            .insert(
                augment,
                CurrencyWarsAugmentRemark {
                    en: parsed.en,
                    zh_cn: parsed.zh_cn,
                },
            )
            .is_some()
        {
            return Err(error("duplicate Currency Wars Augment remark"));
        }
    }
    let mut bans = BTreeMap::<u32, Vec<u32>>::new();
    for row in config.currency_wars_module_ban_rules().ordered_rows() {
        if row.subject_kind.as_deref() == Some("Augment") {
            bans.entry(parsed(required(&row.subject_id, "banned Augment ID")?)?)
                .or_default()
                .push(parsed(required(&row.module_id, "banning module ID")?)?);
        }
    }
    let augments = config
        .currency_wars_augment_definitions()
        .ordered_rows()
        .map(|row| {
            let source_id = stable_source_id(&row.stable_key)?;
            let lifecycle: LifecycleRow =
                parse_json(required(&row.lifecycle, "Augment lifecycle")?)?;
            let mut season_ids = seasons
                .remove(&source_id)
                .ok_or_else(|| error("Currency Wars Augment season membership is missing"))?;
            season_ids.sort_unstable();
            let mut banned_module_ids = bans.remove(&source_id).unwrap_or_default();
            banned_module_ids.sort_unstable();
            Ok(CurrencyWarsAugmentDefinition {
                investment: augment_investment_id(row.id)?,
                source_id,
                stable_key: row.stable_key.clone().into(),
                category_id: parsed(required(&row.category_id, "Augment category")?)?,
                quality: quality(required(&row.quality, "Augment quality")?)?,
                chapter_limits: parsed_array(row.chapter_limits.as_ref())?,
                season_ids: season_ids.into_boxed_slice(),
                effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
                config_path: required(&row.config_path, "Augment config path")?.into(),
                lifecycle: CurrencyWarsAugmentLifecycle {
                    saved_values: lifecycle.saved_values.into_boxed_slice(),
                    overclock_effective: match lifecycle.overclock_effective.as_ref() {
                        "" | "0" => false,
                        "1" => true,
                        _ => return Err(error("Currency Wars Augment overclock flag is invalid")),
                    },
                    effect_parameters: decimals(lifecycle.effect_parameters)?,
                    description_parameters: decimals(lifecycle.description_parameters)?,
                },
                remark: remarks.remove(&source_id),
                banned_module_ids: banned_module_ids.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    if !seasons.is_empty() || !remarks.is_empty() || !bans.is_empty() {
        return Err(error("Currency Wars Augment join contains an orphan row"));
    }
    let selected_enhancements = config
        .currency_wars_selected_enhancements()
        .ordered_rows()
        .map(|row| {
            let parameters: SelectedEnhancementParameters = parse_json(required(
                &row.parameters,
                "selected Enhancement parameters",
            )?)?;
            Ok(CurrencyWarsSelectedEnhancement {
                id: CurrencyWarsSelectedEnhancementId::new(stable_source_id(&row.stable_key)?)
                    .ok_or_else(|| error("Currency Wars selected Enhancement ID is zero"))?,
                stable_key: row.stable_key.clone().into(),
                trait_effect_id: parsed(required(
                    &row.trait_effect_id,
                    "selected Enhancement trait effect",
                )?)?,
                gold_cost: row
                    .cost
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .map(parsed)
                    .transpose()?,
                condition: match parameters.select_condition.as_ref() {
                    "" => CurrencyWarsEnhancementSelectCondition::Always,
                    "MaxStar" => CurrencyWarsEnhancementSelectCondition::MaximumStar,
                    _ => {
                        return Err(error(
                            "Currency Wars selected Enhancement condition is invalid",
                        ));
                    }
                },
                parameters: decimals(parameters.parameters)?,
                effects: decimals(parameters.effects)?,
                effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let enhancements = config
        .currency_wars_enhancements()
        .ordered_rows()
        .map(|row| {
            let source_id = stable_source_id(&row.stable_key)?;
            let parameters: EnhancementParameters =
                parse_json(required(&row.parameters, "Enhancement parameters")?)?;
            Ok(CurrencyWarsEnhancement {
                investment: enhancement_investment_id(row.id)?,
                id: CurrencyWarsSelectedEnhancementId::new(source_id)
                    .ok_or_else(|| error("Currency Wars Enhancement ID is zero"))?,
                stable_key: row.stable_key.clone().into(),
                trait_effect_id: parsed(required(&row.group_id, "Enhancement group")?)?,
                gold_cost: row
                    .cost
                    .as_deref()
                    .filter(|value| !value.is_empty())
                    .map(parsed)
                    .transpose()?,
                condition: enhancement_condition(&parameters.select_condition)?,
                effects: decimals(parameters.effects)?,
                effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let maze_buffs = config
        .currency_wars_augment_maze_buffs()
        .ordered_rows()
        .map(|row| {
            let level: MazeBuffLevel = parse_json(required(&row.level, "Augment maze level")?)?;
            let binding: MazeBuffBinding =
                parse_json(required(&row.binding, "Augment maze binding")?)?;
            Ok(CurrencyWarsInvestmentMazeBuff {
                source_id: stable_component(&row.stable_key, 2)?,
                stable_key: row.stable_key.clone().into(),
                series: parsed(required(&row.buff_series, "Augment maze series")?)?,
                level: parsed(&level.current)?,
                maximum_level: parsed(&level.maximum)?,
                binding_type: binding.kind,
                binding_key: binding.key,
                maze_buff_type: binding.maze_buff_type,
                parameters: decimals(parse_json(required(
                    &row.parameters,
                    "Augment maze parameters",
                )?)?)?,
                modifier: required(&row.modifier, "Augment maze modifier")?.into(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    let monster_rules = config
        .currency_wars_augment_monster_rules()
        .ordered_rows()
        .map(|row| {
            let parameters: MonsterParameters =
                parse_json(required(&row.parameters, "Augment monster parameters")?)?;
            Ok(CurrencyWarsAugmentMonsterRule {
                quality: quality(required(&row.quality, "Augment monster quality")?)?,
                division_level: optional_parsed(&parameters.division_level)?,
                enemy_difficulty_level_add: optional_parsed(
                    &parameters.enemy_difficulty_level_add,
                )?,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    CurrencyWarsAugmentCatalog::new(
        augments,
        selected_enhancements,
        enhancements,
        maze_buffs,
        monster_rules,
    )
    .map_err(debug_error)
}

pub(super) fn augment_investment_id(
    row_id: i32,
) -> Result<CurrencyWarsInvestmentId, CurrencyWarsDataError> {
    let raw = 1_000_000_u64
        .checked_add(u64::try_from(row_id).map_err(debug_error)?)
        .ok_or_else(|| error("Currency Wars Augment investment ID overflow"))?;
    CurrencyWarsInvestmentId::new(raw)
        .ok_or_else(|| error("Currency Wars Augment investment ID is zero"))
}

fn enhancement_investment_id(
    row_id: i32,
) -> Result<CurrencyWarsInvestmentId, CurrencyWarsDataError> {
    let raw = 2_000_000_u64
        .checked_add(u64::try_from(row_id).map_err(debug_error)?)
        .ok_or_else(|| error("Currency Wars Enhancement investment ID overflow"))?;
    CurrencyWarsInvestmentId::new(raw)
        .ok_or_else(|| error("Currency Wars Enhancement investment ID is zero"))
}

fn stable_source_id(stable_key: &str) -> Result<u32, CurrencyWarsDataError> {
    stable_key
        .rsplit('.')
        .next()
        .ok_or_else(|| error("Currency Wars stable key has no source ID"))
        .and_then(parsed)
}

fn quality(value: &str) -> Result<CurrencyWarsAugmentQuality, CurrencyWarsDataError> {
    match value {
        "Silver" => Ok(CurrencyWarsAugmentQuality::Silver),
        "Gold" => Ok(CurrencyWarsAugmentQuality::Gold),
        "Prismatic" => Ok(CurrencyWarsAugmentQuality::Prismatic),
        _ => Err(error("Currency Wars Augment quality is invalid")),
    }
}

fn enhancement_condition(
    value: &str,
) -> Result<CurrencyWarsEnhancementSelectCondition, CurrencyWarsDataError> {
    match value {
        "" => Ok(CurrencyWarsEnhancementSelectCondition::Always),
        "Permanent" => Ok(CurrencyWarsEnhancementSelectCondition::Permanent),
        "MaxStar" => Ok(CurrencyWarsEnhancementSelectCondition::MaximumStar),
        _ => Err(error("Currency Wars Enhancement condition is invalid")),
    }
}

fn stable_component<T: std::str::FromStr>(
    stable_key: &str,
    index: usize,
) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    stable_key
        .split('.')
        .nth(index)
        .ok_or_else(|| error("Currency Wars stable key component is missing"))
        .and_then(parsed)
}

fn optional_parsed<T: std::str::FromStr>(value: &str) -> Result<Option<T>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    (!value.is_empty()).then(|| parsed(value)).transpose()
}

fn parsed<T: std::str::FromStr>(value: &str) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value.parse().map_err(debug_error)
}

fn parsed_id(value: &str) -> Result<u32, CurrencyWarsDataError> {
    parsed(value)
}

fn parsed_array<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Box<[T]>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    parse_json::<Vec<Box<str>>>(value.map_or("[]", String::as_str))?
        .into_iter()
        .map(|value| parsed(&value))
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
        return Err(error("Currency Wars authored decimal is invalid"));
    }
    let significand = format!("{whole}{fractional}")
        .parse::<i64>()
        .map_err(debug_error)?;
    CurrencyWarsDecimal::new(
        if negative { -significand } else { significand },
        u8::try_from(fractional.len()).expect("at most 18 decimal places"),
    )
    .ok_or_else(|| error("Currency Wars authored decimal scale is invalid"))
}
