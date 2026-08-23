use serde::Deserialize;
use starclock_combat::Ratio;
use starclock_mode_currency_wars::{
    CurrencyWarsEnemyScaling, CurrencyWarsEnemySlot, CurrencyWarsEnemySlotDefinition,
    CurrencyWarsEnemyStatRatios,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_decimal, parse_json, required},
    currency_wars_generated::currency_wars_enemy_slots::CurrencyWarsEnemySlots,
};

use super::{parse_number_strings, parse_required};

#[derive(Deserialize)]
struct EnemyScalingRankRow {
    chapter_id: String,
}

#[derive(Deserialize)]
struct EnemyScalingContributionRow {
    hp_ratio: String,
    attack_ratio: String,
    defence_ratio: String,
    speed_ratio: String,
    stance_ratio: String,
}

#[derive(Deserialize)]
struct EliteScalingSlotRow {
    elite_group: String,
}

#[derive(Deserialize)]
struct MonsterSlotRow {
    monster_tier: String,
}

pub(super) fn lower_enemy_slot(
    row: &CurrencyWarsEnemySlots,
) -> Result<CurrencyWarsEnemySlot, CurrencyWarsDataError> {
    let wave = required(&row.wave_id, "enemy-slot wave")?;
    let slot_index = parse_required::<u32>(&row.slot_index, "enemy-slot index")?;
    let monster = required(&row.monster_id, "enemy-slot monster")?;
    let level = required(&row.level, "enemy-slot level")?;
    let references = parse_boxed_strings(row.ability_refs.as_ref())?;
    let definition = if monster == "none:elite-scaling-group" {
        let level: EliteScalingSlotRow = parse_json(level)?;
        let group = level.elite_group.parse::<u16>().map_err(debug_error)?;
        if wave != "GridFightEliteScalingCatalog"
            || slot_index != u32::from(group)
            || row.shared_enemy_key.is_some()
            || references.len() != 5
        {
            return Err(error("enemy elite-scaling slot identity is invalid"));
        }
        CurrencyWarsEnemySlotDefinition::EliteScaling {
            group,
            ratios: CurrencyWarsEnemyStatRatios {
                attack: labeled_ratio(&references, "attack-ratio")?,
                defense: labeled_ratio(&references, "defence-ratio")?,
                hp: labeled_ratio(&references, "hp-ratio")?,
                speed: labeled_ratio(&references, "speed-ratio")?,
                stance: labeled_ratio(&references, "stance-ratio")?,
            },
        }
    } else {
        let source_monster_id = monster.parse::<u32>().map_err(debug_error)?;
        let level: MonsterSlotRow = parse_json(level)?;
        if wave != "GridFightCampMonsterPool"
            || slot_index != source_monster_id
            || references.len() != 4
        {
            return Err(error("enemy monster-slot identity is invalid"));
        }
        CurrencyWarsEnemySlotDefinition::Monster {
            source_monster_id,
            tier: match level.monster_tier.as_str() {
                "undefined" => None,
                value => Some(value.parse().map_err(debug_error)?),
            },
            star_scaling_groups: [
                labeled_integer(&references, "Star1EliteGroup3")?,
                labeled_integer(&references, "Star2EliteGroup3")?,
                labeled_integer(&references, "Star3EliteGroup3")?,
                labeled_integer(&references, "Star4EliteGroup3")?,
            ],
            shared_enemy_key: required(&row.shared_enemy_key, "enemy shared key")?.into(),
        }
    };
    Ok(CurrencyWarsEnemySlot {
        stable_key: row.stable_key.clone().into(),
        definition,
    })
}

pub(super) fn lower_enemy_scaling(
    stable_key: &str,
    rank_bounds: Option<&String>,
    difficulty_ids: Option<&String>,
    contributions: Option<&String>,
) -> Result<Option<CurrencyWarsEnemyScaling>, CurrencyWarsDataError> {
    if !stable_key.starts_with("currency-wars.enemy-difficulty.") {
        return Ok(None);
    }
    let rank: EnemyScalingRankRow = parse_json(
        rank_bounds
            .filter(|value| !value.is_empty())
            .ok_or_else(|| error("enemy scaling rank is missing"))?,
    )?;
    let difficulty = parse_number_strings::<u16>(difficulty_ids)?;
    let [difficulty_level] = difficulty.as_ref() else {
        return Err(error("enemy scaling difficulty is not singular"));
    };
    let values: EnemyScalingContributionRow = parse_json(
        contributions
            .filter(|value| !value.is_empty())
            .ok_or_else(|| error("enemy scaling contribution is missing"))?,
    )?;
    Ok(Some(CurrencyWarsEnemyScaling {
        chapter: rank.chapter_id.parse().map_err(debug_error)?,
        difficulty_level: *difficulty_level,
        hp_ratio: parse_source_ratio(&values.hp_ratio)?,
        attack_ratio: parse_source_ratio(&values.attack_ratio)?,
        defense_ratio: parse_source_ratio(&values.defence_ratio)?,
        speed_ratio: parse_source_ratio(&values.speed_ratio)?,
        stance_ratio: parse_source_ratio(&values.stance_ratio)?,
    }))
}

fn labeled_ratio(values: &[Box<str>], label: &str) -> Result<Ratio, CurrencyWarsDataError> {
    let value = labeled_value(values, label)?;
    parse_source_ratio(value)
}

fn labeled_integer(values: &[Box<str>], label: &str) -> Result<u16, CurrencyWarsDataError> {
    labeled_value(values, label)?.parse().map_err(debug_error)
}

fn labeled_value<'a>(
    values: &'a [Box<str>],
    label: &str,
) -> Result<&'a str, CurrencyWarsDataError> {
    let prefix = format!("{label}:");
    let mut matching = values
        .iter()
        .filter_map(|value| value.strip_prefix(&prefix));
    let value = matching
        .next()
        .ok_or_else(|| error("enemy-slot labeled value is missing"))?;
    if matching.next().is_some() {
        return Err(error("enemy-slot labeled value is duplicated"));
    }
    Ok(value)
}

fn parse_source_ratio(value: &str) -> Result<Ratio, CurrencyWarsDataError> {
    if let Ok(scaled) = parse_decimal(value) {
        return Ok(Ratio::from_scaled(scaled));
    }
    let (whole, fraction) = value
        .split_once('.')
        .ok_or_else(|| error("enemy scaling decimal is invalid"))?;
    if whole.is_empty()
        || fraction.len() <= 6
        || fraction.len() > 18
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(error("enemy scaling decimal is invalid"));
    }
    let whole = whole.parse::<i128>().map_err(debug_error)?;
    let retained = fraction[..6].parse::<i128>().map_err(debug_error)?;
    let discarded = fraction[6..].parse::<i128>().map_err(debug_error)?;
    let half = 5_i128
        .checked_mul(10_i128.pow(u32::try_from(fraction.len() - 7).map_err(debug_error)?))
        .ok_or_else(|| error("enemy scaling decimal overflows"))?;
    let rounded = retained + i128::from(discarded >= half);
    let scaled = whole
        .checked_mul(1_000_000)
        .and_then(|value| value.checked_add(rounded))
        .and_then(|value| i64::try_from(value).ok())
        .ok_or_else(|| error("enemy scaling decimal overflows"))?;
    Ok(Ratio::from_scaled(scaled))
}
