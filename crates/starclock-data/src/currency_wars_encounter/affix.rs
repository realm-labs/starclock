use serde::Deserialize;
use starclock_combat::Scalar;
use starclock_mode_currency_wars::{
    CurrencyWarsEnemyAffix, CurrencyWarsEnemyAffixBindingType, CurrencyWarsEnemyAffixDefinition,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_decimal, parse_json, required},
    currency_wars_generated::currency_wars_enemy_affixes::CurrencyWarsEnemyAffixes,
};

use super::{enemy::lower_enemy_scaling, stable_tail};

const AFFIX_PREFIX: &str = "currency-wars.enemy-affix.definition.";
const MAZE_BUFF_PREFIX: &str = "currency-wars.enemy-affix.maze-buff.";
const SCALING_PREFIX: &str = "currency-wars.enemy-difficulty.";

#[derive(Deserialize)]
struct AffixContributionRow {
    maze_buff_ids: Vec<String>,
    config_path: String,
    parameters: Vec<String>,
}

#[derive(Deserialize)]
struct MazeBuffContributionRow {
    modifier: String,
    binding_type: String,
    binding_key: String,
    level: String,
    maximum_level: String,
    parameters: Vec<String>,
}

pub(super) fn lower_enemy_affix(
    row: &CurrencyWarsEnemyAffixes,
) -> Result<CurrencyWarsEnemyAffix, CurrencyWarsDataError> {
    let definition = if row.stable_key.starts_with(AFFIX_PREFIX) {
        lower_affix_definition(row)?
    } else if row.stable_key.starts_with(MAZE_BUFF_PREFIX) {
        lower_maze_buff(row)?
    } else if row.stable_key.starts_with(SCALING_PREFIX) {
        CurrencyWarsEnemyAffixDefinition::Scaling(
            lower_enemy_scaling(
                &row.stable_key,
                row.rank_bounds.as_ref(),
                row.difficulty_ids.as_ref(),
                row.battle_contributions.as_ref(),
            )?
            .ok_or_else(|| error("enemy scaling row was not recognized"))?,
        )
    } else {
        return Err(error("enemy-affix stable key is not recognized"));
    };
    Ok(CurrencyWarsEnemyAffix {
        stable_key: row.stable_key.clone().into(),
        definition,
    })
}

fn lower_affix_definition(
    row: &CurrencyWarsEnemyAffixes,
) -> Result<CurrencyWarsEnemyAffixDefinition, CurrencyWarsDataError> {
    require_affix_scope(row, "SelectedByDivisionOrStageConfiguration")?;
    let contribution: AffixContributionRow = parse_json(required(
        &row.battle_contributions,
        "enemy-affix contribution",
    )?)?;
    Ok(CurrencyWarsEnemyAffixDefinition::Affix {
        source_id: stable_tail(&row.stable_key)?.parse().map_err(debug_error)?,
        maze_buff_ids: contribution
            .maze_buff_ids
            .iter()
            .map(|id| id.parse().map_err(debug_error))
            .collect::<Result<_, _>>()?,
        config_path: contribution.config_path.into(),
        parameters: parse_parameters(&contribution.parameters)?,
    })
}

fn lower_maze_buff(
    row: &CurrencyWarsEnemyAffixes,
) -> Result<CurrencyWarsEnemyAffixDefinition, CurrencyWarsDataError> {
    require_affix_scope(row, "ReferencedByAffixOrConfiguration")?;
    let identity = row
        .stable_key
        .strip_prefix(MAZE_BUFF_PREFIX)
        .ok_or_else(|| error("enemy-affix MazeBuff identity is invalid"))?;
    let (source_id, locator) = identity
        .split_once('.')
        .ok_or_else(|| error("enemy-affix MazeBuff identity is invalid"))?;
    let _: u16 = locator.parse().map_err(debug_error)?;
    let contribution: MazeBuffContributionRow = parse_json(required(
        &row.battle_contributions,
        "enemy-affix MazeBuff contribution",
    )?)?;
    let binding_type = match contribution.binding_type.as_str() {
        "StageAbilityBeforeCharacterBorn" => CurrencyWarsEnemyAffixBindingType::BeforeCharacterBorn,
        _ => return Err(error("enemy-affix MazeBuff binding type is unsupported")),
    };
    Ok(CurrencyWarsEnemyAffixDefinition::MazeBuff {
        source_id: source_id.parse().map_err(debug_error)?,
        modifier: contribution.modifier.into(),
        binding_type,
        binding_key: contribution.binding_key.into(),
        level: contribution.level.parse().map_err(debug_error)?,
        maximum_level: contribution.maximum_level.parse().map_err(debug_error)?,
        parameters: parse_parameters(&contribution.parameters)?,
    })
}

fn require_affix_scope(
    row: &CurrencyWarsEnemyAffixes,
    expected_rank_bounds: &str,
) -> Result<(), CurrencyWarsDataError> {
    if required(&row.rank_bounds, "enemy-affix rank bounds")? != expected_rank_bounds
        || !parse_boxed_strings(row.difficulty_ids.as_ref())?.is_empty()
    {
        return Err(error("enemy-affix scope is invalid"));
    }
    Ok(())
}

fn parse_parameters(values: &[String]) -> Result<Box<[Scalar]>, CurrencyWarsDataError> {
    values
        .iter()
        .map(|value| parse_decimal(value).map(Scalar::from_scaled))
        .collect()
}
