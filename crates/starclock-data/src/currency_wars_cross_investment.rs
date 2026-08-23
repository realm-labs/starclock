use std::collections::BTreeMap;

use serde::Deserialize;
use starclock_mode_currency_wars::{
    CurrencyWarsCrossInvestmentCatalog, CurrencyWarsDecimal, CurrencyWarsInvestmentId,
    CurrencyWarsMazeBuff, CurrencyWarsOrbDefinition, CurrencyWarsOrbDisplay, CurrencyWarsOrbType,
    CurrencyWarsPortalDefinition, CurrencyWarsPortalRemark, CurrencyWarsProjectionDefinition,
    CurrencyWarsRoleId, CurrencyWarsTalentDefinition, CurrencyWarsTalentKind,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

#[derive(Deserialize)]
struct PortalLifecycleRow {
    overclock_effective: Box<str>,
    in_index: Box<str>,
    delayed_bonus: Vec<Box<str>>,
    effect_parameters: Vec<Box<str>>,
    npc_ids: Vec<Box<str>>,
}

#[derive(Deserialize)]
struct RemarkRow {
    en: Box<str>,
    zh_cn: Box<str>,
}
#[derive(Deserialize)]
struct DisplayRow {
    icon_path: Box<str>,
    prefab_path: Box<str>,
}
#[derive(Deserialize)]
struct LevelRow {
    current: Box<str>,
    maximum: Box<str>,
}
#[derive(Deserialize)]
struct BindingRow {
    r#type: Box<str>,
    key: Box<str>,
    maze_buff_type: Box<str>,
}

pub(super) fn lower_currency_wars_cross_investments(
    config: &SoraConfig,
) -> Result<CurrencyWarsCrossInvestmentCatalog, CurrencyWarsDataError> {
    let maze_buffs = lower_maze_buffs(config)?;
    let maze_by_id = maze_buffs
        .iter()
        .map(|value| (value.source_id, value.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut portal_seasons = BTreeMap::<u32, Vec<u16>>::new();
    for row in config
        .currency_wars_season_portal_memberships()
        .ordered_rows()
    {
        portal_seasons
            .entry(parsed(required(&row.portal_id, "season Portal ID")?)?)
            .or_default()
            .push(parsed(required(&row.season_id, "Portal season ID")?)?);
    }
    let mut portal_remarks = BTreeMap::<u32, CurrencyWarsPortalRemark>::new();
    for row in config.currency_wars_portal_remarks().ordered_rows() {
        let value: RemarkRow = parse_json(required(&row.remark, "Portal remark")?)?;
        portal_remarks.insert(
            parsed(required(&row.portal_id, "remark Portal ID")?)?,
            CurrencyWarsPortalRemark {
                en: value.en,
                zh_cn: value.zh_cn,
            },
        );
    }
    let mut portal_bans = BTreeMap::<u32, Vec<u32>>::new();
    for row in config.currency_wars_module_ban_rules().ordered_rows() {
        if row.subject_kind.as_deref() == Some("Portal") {
            portal_bans
                .entry(parsed(required(&row.subject_id, "banned Portal ID")?)?)
                .or_default()
                .push(parsed(required(&row.module_id, "Portal module ID")?)?);
        }
    }
    let portals = config
        .currency_wars_portal_buffs()
        .ordered_rows()
        .map(|row| {
            let source_id = stable_tail(&row.stable_key)?;
            let lifecycle: PortalLifecycleRow =
                parse_json(required(&row.lifecycle, "Portal lifecycle")?)?;
            let mut seasons = portal_seasons.remove(&source_id).unwrap_or_default();
            seasons.sort_unstable();
            let mut bans = portal_bans.remove(&source_id).unwrap_or_default();
            bans.sort_unstable();
            let effect_ids = parse_boxed_strings(row.effect_ids.as_ref())?;
            Ok(CurrencyWarsPortalDefinition {
                investment: investment_id(4_000_000, row.id)?,
                source_id,
                stable_key: row.stable_key.clone().into(),
                season_ids: seasons.into(),
                config_path: required(&row.config_path, "Portal config path")?.into(),
                maze_buffs: referenced_maze_buffs(&effect_ids, &maze_by_id),
                effect_ids,
                bonus_ids: parsed_array(row.bonus_ids.as_ref())?,
                overclock_effective: flag(&lifecycle.overclock_effective)?,
                in_index: lifecycle.in_index.as_ref() == "true",
                delayed_bonus_ids: parsed_values(lifecycle.delayed_bonus)?,
                effect_parameters: decimals(lifecycle.effect_parameters)?,
                npc_ids: parsed_values(lifecycle.npc_ids)?,
                remark: portal_remarks.remove(&source_id),
                banned_module_ids: bans.into(),
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;
    if !portal_seasons.is_empty() || !portal_remarks.is_empty() || !portal_bans.is_empty() {
        return Err(error("Currency Wars Portal join contains an orphan row"));
    }

    let displays = config
        .currency_wars_orb_displays()
        .ordered_rows()
        .map(|row| {
            let orb_type = orb_type(required(&row.orb_type, "Orb display type")?)?;
            let display: DisplayRow = parse_json(required(&row.display_locator, "Orb display")?)?;
            Ok((
                orb_type,
                CurrencyWarsOrbDisplay {
                    orb_type,
                    icon_path: display.icon_path,
                    prefab_path: display.prefab_path,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, CurrencyWarsDataError>>()?;
    let orbs = config
        .currency_wars_orbs()
        .ordered_rows()
        .map(|row| {
            let orb_type = orb_type(required(&row.orb_type, "Orb type")?)?;
            Ok(CurrencyWarsOrbDefinition {
                investment: investment_id(3_000_000, row.id)?,
                source_id: row
                    .stable_key
                    .strip_prefix("currency-wars.orb.")
                    .ok_or_else(|| error("Currency Wars Orb stable key is invalid"))?
                    .into(),
                stable_key: row.stable_key.clone().into(),
                bonus_id: parsed(required(&row.bonus_id, "Orb bonus ID")?)?,
                orb_type,
                effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
                display: displays
                    .get(&orb_type)
                    .cloned()
                    .ok_or_else(|| error("Currency Wars Orb display is missing"))?,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;

    let projections = config
        .currency_wars_projections()
        .ordered_rows()
        .map(|row| {
            let effect_ids = parse_boxed_strings(row.effect_ids.as_ref())?;
            let source_id = stable_tail(&row.stable_key)?;
            Ok(CurrencyWarsProjectionDefinition {
                investment: investment_id(5_000_000, row.id)?,
                source_id,
                stable_key: row.stable_key.clone().into(),
                role: CurrencyWarsRoleId::new(parsed(required(&row.role_id, "Projection role")?)?)
                    .ok_or_else(|| error("Currency Wars Projection role is zero"))?,
                unlock_type: required(&row.unlock_type, "Projection unlock type")?.into(),
                trait_ids: parsed_array(row.trait_ids.as_ref())?,
                maze_buffs: referenced_maze_buffs(&effect_ids, &maze_by_id),
                effect_ids,
            })
        })
        .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?;

    let mut talents = config
        .currency_wars_talents()
        .ordered_rows()
        .map(|row| {
            lower_talent(
                row.id,
                &row.stable_key,
                CurrencyWarsTalentKind::Permanent,
                None,
                row.cost.as_ref(),
                row.prerequisite_ids.as_ref(),
                row.successor_ids.as_ref(),
                row.effect_ids.as_ref(),
                row.config_path.as_ref(),
                &maze_by_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    talents.extend(
        config
            .currency_wars_season_talents()
            .ordered_rows()
            .map(|row| {
                lower_talent(
                    row.id,
                    &row.stable_key,
                    CurrencyWarsTalentKind::Season,
                    Some(parsed(required(&row.season_id, "Talent season")?)?),
                    row.cost.as_ref(),
                    row.prerequisite_ids.as_ref(),
                    row.successor_ids.as_ref(),
                    row.effect_ids.as_ref(),
                    row.config_path.as_ref(),
                    &maze_by_id,
                )
            })
            .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?,
    );
    CurrencyWarsCrossInvestmentCatalog::new(portals, orbs, projections, talents, maze_buffs)
        .map_err(debug_error)
}

#[allow(clippy::too_many_arguments)]
fn lower_talent(
    id: i32,
    stable_key: &str,
    kind: CurrencyWarsTalentKind,
    season_id: Option<u16>,
    cost: Option<&String>,
    prerequisites: Option<&String>,
    successors: Option<&String>,
    effect_ids: Option<&String>,
    config_path: Option<&String>,
    maze_by_id: &BTreeMap<u32, CurrencyWarsMazeBuff>,
) -> Result<CurrencyWarsTalentDefinition, CurrencyWarsDataError> {
    let effects = parse_boxed_strings(effect_ids)?;
    Ok(CurrencyWarsTalentDefinition {
        investment: (kind == CurrencyWarsTalentKind::Permanent)
            .then(|| investment_id(6_000_000, id))
            .transpose()?,
        source_id: stable_tail(stable_key)?,
        stable_key: stable_key.into(),
        kind,
        season_id,
        cost: parsed(required_ref(cost, "Talent cost")?)?,
        prerequisites: parsed_array(prerequisites)?,
        successors: parsed_array(successors)?,
        maze_buffs: referenced_maze_buffs(&effects, maze_by_id),
        effect_ids: effects,
        config_path: required_ref(config_path, "Talent config path")?.into(),
    })
}

fn lower_maze_buffs(
    config: &SoraConfig,
) -> Result<Vec<CurrencyWarsMazeBuff>, CurrencyWarsDataError> {
    let mut values = Vec::new();
    macro_rules! extend {
        ($rows:expr) => {
            for row in $rows.ordered_rows() {
                let level: LevelRow = parse_json(required(&row.level, "maze-buff level")?)?;
                let binding: BindingRow = parse_json(required(&row.binding, "maze-buff binding")?)?;
                values.push(CurrencyWarsMazeBuff {
                    source_id: stable_source(&row.stable_key)?,
                    stable_key: row.stable_key.clone().into(),
                    series: parsed(required(&row.buff_series, "maze-buff series")?)?,
                    level: parsed(&level.current)?,
                    maximum_level: parsed(&level.maximum)?,
                    binding_type: binding.r#type,
                    binding_key: binding.key,
                    maze_buff_type: binding.maze_buff_type,
                    parameters: decimals(parse_json(row.parameters.as_deref().unwrap_or("[]"))?)?,
                    modifier: required(&row.modifier, "maze-buff modifier")?.into(),
                });
            }
        };
    }
    extend!(config.currency_wars_portal_maze_buffs());
    extend!(config.currency_wars_projection_maze_buffs());
    extend!(config.currency_wars_talent_maze_buffs());
    Ok(values)
}

fn referenced_maze_buffs(
    effect_ids: &[Box<str>],
    values: &BTreeMap<u32, CurrencyWarsMazeBuff>,
) -> Box<[CurrencyWarsMazeBuff]> {
    effect_ids
        .iter()
        .filter_map(|effect| effect.strip_prefix("maze-buff:"))
        .filter_map(|raw| raw.parse::<u32>().ok())
        .filter_map(|id| values.get(&id).cloned())
        .collect()
}
fn orb_type(value: &str) -> Result<CurrencyWarsOrbType, CurrencyWarsDataError> {
    match value {
        "White" => Ok(CurrencyWarsOrbType::White),
        "Blue" => Ok(CurrencyWarsOrbType::Blue),
        "Glod" => Ok(CurrencyWarsOrbType::Gold),
        "Colorful" => Ok(CurrencyWarsOrbType::Colorful),
        _ => Err(error("Currency Wars Orb type is invalid")),
    }
}
fn flag(value: &str) -> Result<bool, CurrencyWarsDataError> {
    match value {
        "" | "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(error("Currency Wars flag is invalid")),
    }
}
fn required_ref<'a>(
    value: Option<&'a String>,
    name: &str,
) -> Result<&'a str, CurrencyWarsDataError> {
    value
        .map(String::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| error(&format!("{name} is missing")))
}
fn stable_tail(value: &str) -> Result<u32, CurrencyWarsDataError> {
    value
        .rsplit('.')
        .next()
        .ok_or_else(|| error("Currency Wars stable key is invalid"))
        .and_then(parsed)
}
fn stable_source(value: &str) -> Result<u32, CurrencyWarsDataError> {
    value
        .split('.')
        .nth(2)
        .ok_or_else(|| error("Currency Wars maze-buff key is invalid"))
        .and_then(parsed)
}
fn investment_id(prefix: u64, id: i32) -> Result<CurrencyWarsInvestmentId, CurrencyWarsDataError> {
    CurrencyWarsInvestmentId::new(prefix + u64::try_from(id).map_err(debug_error)?)
        .ok_or_else(|| error("Currency Wars investment ID is zero"))
}
fn parsed<T: std::str::FromStr>(value: &str) -> Result<T, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    value.parse().map_err(debug_error)
}
fn parsed_array<T: std::str::FromStr>(
    value: Option<&String>,
) -> Result<Box<[T]>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    parsed_values(parse_json(value.map_or("[]", String::as_str))?)
}
fn parsed_values<T: std::str::FromStr>(
    values: Vec<Box<str>>,
) -> Result<Box<[T]>, CurrencyWarsDataError>
where
    T::Err: std::fmt::Debug,
{
    values
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
        .map_or((false, value), |value| (true, value));
    let (whole, fractional) = unsigned.split_once('.').unwrap_or((unsigned, ""));
    if whole.is_empty()
        || fractional.len() > 18
        || !whole.bytes().all(|v| v.is_ascii_digit())
        || !fractional.bytes().all(|v| v.is_ascii_digit())
    {
        return Err(error("Currency Wars decimal is invalid"));
    }
    let raw = format!("{whole}{fractional}")
        .parse::<i64>()
        .map_err(debug_error)?;
    CurrencyWarsDecimal::new(
        if negative { -raw } else { raw },
        u8::try_from(fractional.len()).expect("at most 18 decimals"),
    )
    .ok_or_else(|| error("Currency Wars decimal scale is invalid"))
}
