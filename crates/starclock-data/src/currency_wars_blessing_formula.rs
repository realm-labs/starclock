use starclock_mode_currency_wars::{
    CurrencyWarsBlessingFormulaCatalog, CurrencyWarsDecimal, CurrencyWarsMazeBuffEnhancement,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

pub(super) fn lower_currency_wars_blessing_formula(
    config: &SoraConfig,
) -> Result<CurrencyWarsBlessingFormulaCatalog, CurrencyWarsDataError> {
    validate_proven_empty_closure(config)?;
    CurrencyWarsBlessingFormulaCatalog::new(
        config
            .currency_wars_blessing_levels()
            .ordered_rows()
            .map(|row| {
                let blessing_id = required(&row.blessing_id, "MazeBuff enhancement parent")?;
                if blessing_id != "none:maze-buff-enhancement" {
                    return Err(error(
                        "Currency Wars Blessing identity appeared in the proven-empty closure",
                    ));
                }
                Ok(CurrencyWarsMazeBuffEnhancement {
                    stable_key: row.stable_key.clone().into(),
                    source_id: required(&row.level, "MazeBuff enhancement source ID")?
                        .parse()
                        .map_err(debug_error)?,
                    parameters: parse_json::<Vec<String>>(required(
                        &row.parameters,
                        "MazeBuff enhancement parameters",
                    )?)?
                    .into_iter()
                    .map(|value| decimal(&value))
                    .collect::<Result<Vec<_>, _>>()?
                    .into_boxed_slice(),
                    effect_ids: parse_boxed_strings(row.effect_ids.as_ref())?,
                })
            })
            .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?,
    )
    .map_err(debug_error)
}

fn validate_proven_empty_closure(config: &SoraConfig) -> Result<(), CurrencyWarsDataError> {
    let paths = config.currency_wars_blessing_paths();
    let formulas = config.currency_wars_formulas();
    if paths.len() != 1
        || formulas.len() != 1
        || paths
            .ordered_rows()
            .next()
            .and_then(|row| row.path_id.as_deref())
            != Some("none")
        || formulas
            .ordered_rows()
            .next()
            .and_then(|row| row.formula_kind.as_deref())
            != Some("ProvenEmptyDirectAndSharedClosure")
        || !config.currency_wars_blessings().is_empty()
        || !config.currency_wars_blessing_groups().is_empty()
        || !config.currency_wars_formula_displays().is_empty()
        || !config.currency_wars_formula_randomizers().is_empty()
        || !config.currency_wars_formula_recipes().is_empty()
        || !config.currency_wars_formula_contributions().is_empty()
    {
        return Err(error(
            "Currency Wars Blessing/formula proven-empty closure drifted",
        ));
    }
    Ok(())
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
        return Err(error(
            "Currency Wars MazeBuff enhancement decimal is invalid",
        ));
    }
    let digits = format!("{whole}{fractional}");
    let significand = digits.parse::<i64>().map_err(debug_error)?;
    CurrencyWarsDecimal::new(
        if negative { -significand } else { significand },
        u8::try_from(fractional.len()).expect("at most 18 decimal places"),
    )
    .ok_or_else(|| error("Currency Wars MazeBuff enhancement decimal scale is invalid"))
}
