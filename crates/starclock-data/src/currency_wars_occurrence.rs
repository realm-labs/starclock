use serde::Deserialize;
use starclock_mode_currency_wars::{
    CurrencyWarsOccurrence, CurrencyWarsOccurrenceCatalog, CurrencyWarsOccurrenceChoice,
    CurrencyWarsOccurrenceCondition, CurrencyWarsOccurrenceCost, CurrencyWarsOccurrenceKind,
    CurrencyWarsOccurrenceOutcome, CurrencyWarsOccurrenceOutcomeKind,
    CurrencyWarsOccurrenceVariant,
};

use crate::{
    currency_wars::{CurrencyWarsDataError, debug_error, error},
    currency_wars_build::canonical_json,
    currency_wars_flow::{parse_boxed_strings, parse_json, required},
    currency_wars_generated::SoraConfig,
};

pub(super) fn lower_currency_wars_occurrences(
    config: &SoraConfig,
) -> Result<CurrencyWarsOccurrenceCatalog, CurrencyWarsDataError> {
    CurrencyWarsOccurrenceCatalog::new(
        config
            .currency_wars_occurrences()
            .ordered_rows()
            .map(|row| {
                Ok(CurrencyWarsOccurrence {
                    stable_key: row.stable_key.clone().into(),
                    source_id: stable_tail(&row.stable_key)?,
                    kind: occurrence_kind(&row.stable_key)?,
                    unlock_rules_json: canonical_json(required(
                        &row.unlock_rules,
                        "occurrence unlock rules",
                    )?)?,
                    variant_keys: parse_boxed_strings(row.variant_ids.as_ref())?,
                    choice_keys: parse_boxed_strings(row.choice_ids.as_ref())?,
                })
            })
            .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?,
        config
            .currency_wars_occurrence_variants()
            .ordered_rows()
            .map(|row| {
                let graph_path = optional_text(row.graph_path.as_ref());
                Ok(CurrencyWarsOccurrenceVariant {
                    stable_key: row.stable_key.clone().into(),
                    source_id: stable_tail(&row.stable_key)?,
                    occurrence_key: required(&row.occurrence_id, "occurrence-variant parent")?
                        .into(),
                    condition: condition(
                        required(&row.entry_conditions, "occurrence entry condition")?,
                        graph_path.is_some(),
                    )?,
                    graph_path,
                    choice_keys: parse_boxed_strings(row.choice_ids.as_ref())?,
                })
            })
            .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?,
        config
            .currency_wars_occurrence_choices()
            .ordered_rows()
            .map(|row| {
                Ok(CurrencyWarsOccurrenceChoice {
                    stable_key: row.stable_key.clone().into(),
                    source_id: stable_tail(&row.stable_key)?,
                    variant_key: required(&row.variant_id, "occurrence-choice parent")?.into(),
                    ordinal: required(&row.ordinal, "occurrence-choice ordinal")?
                        .parse()
                        .map_err(debug_error)?,
                    conditions_json: canonical_json(required(
                        &row.conditions,
                        "occurrence-choice conditions",
                    )?)?,
                    costs: parse_json::<Vec<CostRow>>(required(
                        &row.costs,
                        "occurrence-choice costs",
                    )?)?
                    .into_iter()
                    .map(|row| {
                        if row.kind != "AcceptBonus" {
                            return Err(error("Currency Wars occurrence cost kind is unknown"));
                        }
                        Ok(CurrencyWarsOccurrenceCost {
                            bonus_id: row.id.parse().map_err(debug_error)?,
                        })
                    })
                    .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?
                    .into_boxed_slice(),
                    ordered_outcomes: parse_json::<Vec<OutcomeRow>>(required(
                        &row.ordered_outcomes,
                        "occurrence-choice outcomes",
                    )?)?
                    .into_iter()
                    .map(|row| {
                        Ok(CurrencyWarsOccurrenceOutcome {
                            kind: outcome_kind(&row.operation)?,
                            bonus_id: row.bonus_id.parse().map_err(debug_error)?,
                        })
                    })
                    .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?
                    .into_boxed_slice(),
                })
            })
            .collect::<Result<Vec<_>, CurrencyWarsDataError>>()?,
    )
    .map_err(debug_error)
}

#[derive(Deserialize)]
struct PrayConditionRow {
    finish_type: String,
    parameter_type: String,
    integer_1: String,
    string_1: String,
    integer_list: Vec<String>,
    item_list: serde_json::Value,
    progress: String,
    #[serde(default)]
    backtracks: bool,
}

#[derive(Deserialize)]
struct TutorialConditionRow {
    task_id: String,
}

#[derive(Deserialize)]
struct CostRow {
    kind: String,
    id: String,
}

#[derive(Deserialize)]
struct OutcomeRow {
    operation: String,
    bonus_id: String,
}

fn condition(
    source: &str,
    tutorial: bool,
) -> Result<CurrencyWarsOccurrenceCondition, CurrencyWarsDataError> {
    if tutorial {
        let row = parse_json::<TutorialConditionRow>(source)?;
        return Ok(CurrencyWarsOccurrenceCondition::TutorialTask {
            task_id: row.task_id.parse().map_err(debug_error)?,
        });
    }
    let row = parse_json::<PrayConditionRow>(source)?;
    Ok(CurrencyWarsOccurrenceCondition::PrayFinish {
        finish_type: row.finish_type.into(),
        parameter_type: row.parameter_type.into(),
        integer_1: parse_undefined(&row.integer_1)?,
        string_1: (!row.string_1.is_empty()).then(|| row.string_1.into()),
        integer_list: row
            .integer_list
            .into_iter()
            .map(|value| value.parse().map_err(debug_error))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        item_list_json: serde_json::to_string(&row.item_list)
            .map_err(debug_error)?
            .into(),
        required_progress: row.progress.parse().map_err(debug_error)?,
        backtracks: row.backtracks,
    })
}

fn occurrence_kind(stable_key: &str) -> Result<CurrencyWarsOccurrenceKind, CurrencyWarsDataError> {
    if stable_key.contains(".occurrence.pray.") {
        Ok(CurrencyWarsOccurrenceKind::Pray)
    } else if stable_key.contains(".occurrence.present.") {
        Ok(CurrencyWarsOccurrenceKind::Present)
    } else if stable_key.contains(".occurrence.tutorial-task.") {
        Ok(CurrencyWarsOccurrenceKind::TutorialTask)
    } else {
        Err(error("Currency Wars occurrence kind is unknown"))
    }
}

fn outcome_kind(value: &str) -> Result<CurrencyWarsOccurrenceOutcomeKind, CurrencyWarsDataError> {
    match value {
        "ApplyAcceptBonus" => Ok(CurrencyWarsOccurrenceOutcomeKind::ApplyAcceptBonus),
        "ApplyBonus" => Ok(CurrencyWarsOccurrenceOutcomeKind::ApplyBonus),
        "ApplyFinishBonus" => Ok(CurrencyWarsOccurrenceOutcomeKind::ApplyFinishBonus),
        _ => Err(error("Currency Wars occurrence outcome kind is unknown")),
    }
}

fn stable_tail(value: &str) -> Result<u32, CurrencyWarsDataError> {
    value
        .rsplit('.')
        .next()
        .ok_or_else(|| error("Currency Wars occurrence stable key has no tail"))?
        .parse()
        .map_err(debug_error)
}

fn parse_undefined(value: &str) -> Result<Option<u32>, CurrencyWarsDataError> {
    (value != "undefined")
        .then(|| value.parse().map_err(debug_error))
        .transpose()
}

fn optional_text(value: Option<&String>) -> Option<Box<str>> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.as_str().into())
}
