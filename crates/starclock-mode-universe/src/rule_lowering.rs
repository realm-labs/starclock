//! Strict lowering for Universe mechanic-rule contribution metadata.

use serde::Deserialize;

use crate::error::UniverseCatalogLoadError;
use crate::generated::{SoraConfig, universe_mechanic_rule_kind::UniverseMechanicRuleKind};
use crate::id::MechanicRuleId;
use crate::lowering::{checked_key, checked_source, invalid, localized, reference};
use crate::path_lowering::tags;
use crate::rule::{MechanicParameter, MechanicRuleDefinition, MechanicRuleKind};

pub(crate) fn lower(
    config: &SoraConfig,
) -> Result<Box<[MechanicRuleDefinition]>, UniverseCatalogLoadError> {
    let mut definitions = Vec::with_capacity(config.universe_mechanic_rule().len());
    for row in config.universe_mechanic_rule().ordered_rows() {
        if config
            .universe_content_audit()
            .get_by_content_stable_key(&row.source_record_stable_key)
            .is_none()
        {
            return Err(reference(
                "Mechanic rule content-record reference is unresolved",
            ));
        }
        definitions.push(MechanicRuleDefinition::new(
            rule_id(row.id)?,
            checked_key(&row.stable_key, "Mechanic rule stable key")?,
            checked_key(
                &row.source_record_stable_key,
                "Mechanic rule source-record key",
            )?,
            checked_source(&row.source_file, "Mechanic rule source file")?,
            match row.rule_kind {
                UniverseMechanicRuleKind::PathResonance => MechanicRuleKind::PathResonance,
                UniverseMechanicRuleKind::BlessingDefinition => {
                    MechanicRuleKind::BlessingDefinition
                }
                UniverseMechanicRuleKind::BlessingLevel => MechanicRuleKind::BlessingLevel,
                UniverseMechanicRuleKind::CurioDefinition => MechanicRuleKind::CurioDefinition,
                UniverseMechanicRuleKind::CurioState => MechanicRuleKind::CurioState,
                UniverseMechanicRuleKind::RunService => MechanicRuleKind::RunService,
                UniverseMechanicRuleKind::AbilityTreeContribution => {
                    MechanicRuleKind::AbilityTreeContribution
                }
            },
            optional_key(
                row.native_handler_stable_key.as_deref(),
                "Mechanic native-handler key",
            )?,
            optional_key(
                row.source_binding_key.as_deref(),
                "Mechanic source-binding key",
            )?,
            parse_parameters(&row.parameter_values_json)?,
            tags(row.mechanic_tags.as_deref(), "Mechanic rule tag")?,
            row.approximation_replacement_condition
                .as_deref()
                .map(|value| bounded(value, "Mechanic approximation condition").map(Into::into))
                .transpose()?,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Mechanic rule",
            )?,
        ));
    }
    definitions.sort_by_key(MechanicRuleDefinition::id);
    if definitions.len() != 786 {
        return Err(reference("Mechanic-rule denominator differs"));
    }
    Ok(definitions.into_boxed_slice())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(untagged)]
enum ParameterRow {
    Indexed { index: u32, value: String },
    Named { key: String, value: String },
}
fn parse_parameters(value: &str) -> Result<Box<[MechanicParameter]>, UniverseCatalogLoadError> {
    let rows: Vec<ParameterRow> = serde_json::from_str(value)
        .map_err(|_| invalid("Mechanic-rule parameter JSON is malformed"))?;
    let indexed = rows
        .iter()
        .filter_map(|row| match row {
            ParameterRow::Indexed { index, .. } => Some(*index),
            ParameterRow::Named { .. } => None,
        })
        .collect::<Vec<_>>();
    if !indexed.is_empty()
        && (indexed.len() != rows.len() || indexed.into_iter().ne(1..=rows.len() as u32))
    {
        return Err(invalid(
            "Mechanic-rule parameter indexes are not contiguous",
        ));
    }
    let mut named = std::collections::BTreeSet::new();
    rows.into_iter()
        .map(|row| match row {
            ParameterRow::Indexed { index, value } => {
                checked_source(&value, "Mechanic parameter value")?;
                Ok(MechanicParameter::indexed(index, &value))
            }
            ParameterRow::Named { key, value } => {
                checked_key(&key, "Mechanic parameter key")?;
                checked_source(&value, "Mechanic parameter value")?;
                if !named.insert(key.clone()) {
                    return Err(invalid("Mechanic parameter key is duplicated"));
                }
                Ok(MechanicParameter::named(&key, &value))
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}
fn rule_id(raw: i32) -> Result<MechanicRuleId, UniverseCatalogLoadError> {
    u32::try_from(raw)
        .ok()
        .and_then(MechanicRuleId::new)
        .ok_or_else(|| invalid("Mechanic rule ID must be a positive u32"))
}
fn optional_key(
    value: Option<&str>,
    label: &str,
) -> Result<Option<Box<str>>, UniverseCatalogLoadError> {
    value
        .map(|value| checked_key(value, label).map(Into::into))
        .transpose()
}
fn bounded<'a>(value: &'a str, label: &str) -> Result<&'a str, UniverseCatalogLoadError> {
    if !value.trim().is_empty() && value.len() <= 2_048 {
        Ok(value)
    } else {
        Err(invalid(format!("{label} is empty or too long")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parameter_json_is_typed_and_contiguous() {
        assert_eq!(
            parse_parameters(r#"[{"index":1,"value":"0.5"}]"#)
                .expect("parameter")
                .len(),
            1
        );
        assert!(parse_parameters(r#"[{"index":2,"value":"0.5"}]"#).is_err());
    }
}
