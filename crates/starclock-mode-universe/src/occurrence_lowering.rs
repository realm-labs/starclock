//! Strict lowering for spatial-free Occurrence choice graphs.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use crate::error::UniverseCatalogLoadError;
use crate::generated::SoraConfig;
use crate::id::{OccurrenceChoiceId, OccurrenceId, OccurrenceVariantId};
use crate::lowering::{checked_key, checked_source, invalid, localized, parse_digest, reference};
use crate::occurrence::{
    AuthoredScalar, AuthoredScalarUnit, OccurrenceChoiceDefinition, OccurrenceCost,
    OccurrenceDefinition, OccurrenceOperation, OccurrenceOutcome, OccurrenceParameterVector,
    OccurrenceTarget, OccurrenceVariantDefinition, RandomOutcomePolicy,
};
use crate::path_lowering::{parse_decimal, tags};

pub(crate) struct OccurrenceDefinitions {
    pub(crate) occurrences: Box<[OccurrenceDefinition]>,
    pub(crate) variants: Box<[OccurrenceVariantDefinition]>,
    pub(crate) choices: Box<[OccurrenceChoiceDefinition]>,
}

pub(crate) fn lower(
    config: &SoraConfig,
) -> Result<OccurrenceDefinitions, UniverseCatalogLoadError> {
    let choices = lower_choices(config)?;
    let variants = lower_variants(config, &choices)?;
    let occurrences = lower_occurrences(config, &variants)?;
    Ok(OccurrenceDefinitions {
        occurrences,
        variants,
        choices,
    })
}

fn lower_choices(
    config: &SoraConfig,
) -> Result<Box<[OccurrenceChoiceDefinition]>, UniverseCatalogLoadError> {
    let costs = group_costs(config)?;
    let outcomes = group_outcomes(config)?;
    let mut definitions = Vec::with_capacity(config.universe_occurrence_choice().len());
    for row in config.universe_occurrence_choice().ordered_rows() {
        if config
            .universe_occurrence_variant()
            .get(&row.variant_id)
            .is_none()
        {
            return Err(reference("Occurrence choice references an unknown variant"));
        }
        let vectors = parse_vectors(&row.parameter_vectors_json)?;
        definitions.push(OccurrenceChoiceDefinition::new(
            choice_id(row.id, "Occurrence choice")?,
            checked_key(&row.stable_key, "Occurrence choice stable key")?,
            variant_id(row.variant_id, "Occurrence choice variant")?,
            keys(row.condition_ids.as_deref(), "Occurrence choice condition")?,
            row.next_node_id
                .as_deref()
                .map(|value| checked_key(value, "Occurrence next node").map(Into::into))
                .transpose()?,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Occurrence choice",
            )?,
            [
                parse_digest(
                    &row.choice_label_sha256_en,
                    "Occurrence English choice digest",
                )?,
                parse_digest(
                    &row.choice_label_sha256_zh_cn,
                    "Occurrence Chinese choice digest",
                )?,
            ],
            [
                parse_digest(&row.result_sha256_en, "Occurrence English result digest")?,
                parse_digest(&row.result_sha256_zh_cn, "Occurrence Chinese result digest")?,
            ],
            vectors,
            costs
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
            outcomes
                .get(&row.id)
                .cloned()
                .unwrap_or_default()
                .into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(OccurrenceChoiceDefinition::id);
    if definitions.len() != 321
        || config.universe_occurrence_cost().len() != 70
        || config.universe_occurrence_outcome().len() != 321
        || definitions
            .iter()
            .any(|choice| choice.outcomes().len() != 1)
    {
        return Err(reference(
            "Occurrence choice/cost/outcome denominator differs",
        ));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_variants(
    config: &SoraConfig,
    choices: &[OccurrenceChoiceDefinition],
) -> Result<Box<[OccurrenceVariantDefinition]>, UniverseCatalogLoadError> {
    let mut definitions = Vec::with_capacity(config.universe_occurrence_variant().len());
    for row in config.universe_occurrence_variant().ordered_rows() {
        let id = variant_id(row.id, "Occurrence variant")?;
        if config
            .universe_occurrence()
            .get(&row.occurrence_id)
            .is_none()
        {
            return Err(reference(
                "Occurrence variant references an unknown Occurrence",
            ));
        }
        let authored_choices = choices
            .iter()
            .filter(|choice| choice.variant() == id)
            .map(OccurrenceChoiceDefinition::id)
            .collect::<Vec<_>>();
        if authored_choices.is_empty() {
            return Err(reference("Occurrence variant has no choices"));
        }
        definitions.push(OccurrenceVariantDefinition::new(
            id,
            checked_key(&row.stable_key, "Occurrence variant stable key")?,
            occurrence_id(row.occurrence_id, "Occurrence variant parent")?,
            checked_key(&row.entry_node_id, "Occurrence entry node")?,
            keys(row.condition_ids.as_deref(), "Occurrence variant condition")?,
            checked_source(&row.source_dialogue_type, "Occurrence dialogue type")?,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Occurrence variant",
            )?,
            authored_choices.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(OccurrenceVariantDefinition::id);
    if definitions.len() != 67 {
        return Err(reference("Occurrence variant denominator differs"));
    }
    Ok(definitions.into_boxed_slice())
}

fn lower_occurrences(
    config: &SoraConfig,
    variants: &[OccurrenceVariantDefinition],
) -> Result<Box<[OccurrenceDefinition]>, UniverseCatalogLoadError> {
    let mut definitions = Vec::with_capacity(config.universe_occurrence().len());
    for row in config.universe_occurrence().ordered_rows() {
        let id = occurrence_id(row.id, "Occurrence")?;
        let authored_variants = variants
            .iter()
            .filter(|variant| variant.occurrence() == id)
            .map(OccurrenceVariantDefinition::id)
            .collect::<Vec<_>>();
        if authored_variants.is_empty() {
            return Err(reference("Occurrence has no variants"));
        }
        definitions.push(OccurrenceDefinition::new(
            id,
            checked_key(&row.stable_key, "Occurrence stable key")?,
            checked_key(&row.choice_graph_stable_key, "Occurrence choice graph key")?,
            tags(row.pool_tags.as_deref(), "Occurrence pool tag")?,
            row.index_only,
            localized(
                &row.name_en,
                &row.name_zh_cn,
                &row.summary_en,
                &row.summary_zh_cn,
                "Occurrence",
            )?,
            authored_variants.into_boxed_slice(),
        ));
    }
    definitions.sort_by_key(OccurrenceDefinition::id);
    if definitions.len() != 59 {
        return Err(reference("Occurrence denominator differs"));
    }
    Ok(definitions.into_boxed_slice())
}

fn group_costs(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<OccurrenceCost>>, UniverseCatalogLoadError> {
    grouped(
        config.universe_occurrence_cost().iter().map(|row| {
            let operation = match row.kind.as_str() {
                "Consume" => Ok(OccurrenceOperation::Consume),
                "Discard" => Ok(OccurrenceOperation::Discard),
                "Lose" => Ok(OccurrenceOperation::Lose),
                _ => Err(invalid("Occurrence cost operation is unknown")),
            }?;
            Ok((
                row.choice_id,
                row.sequence,
                OccurrenceCost::new(operation, targets(row.targets.as_deref())?),
            ))
        }),
        config,
        "Occurrence cost",
    )
}

fn group_outcomes(
    config: &SoraConfig,
) -> Result<BTreeMap<i32, Vec<OccurrenceOutcome>>, UniverseCatalogLoadError> {
    grouped(
        config.universe_occurrence_outcome().iter().map(|row| {
            if row.kinds.is_empty() {
                return Err(invalid("Occurrence outcome has no operation"));
            }
            let operations = row
                .kinds
                .iter()
                .map(|value| operation(value))
                .collect::<Result<Vec<_>, _>>()?;
            let numeric_literals = row
                .numeric_literals
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|value| authored_scalar(value))
                .collect::<Result<Vec<_>, _>>()?;
            let chance_percentages = row
                .chance_percentages
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|value| parse_decimal(value))
                .collect::<Result<Vec<_>, _>>()?;
            if chance_percentages.iter().any(|value| {
                value.coefficient() < 0
                    || value.coefficient() > 100_i64 * 10_i64.pow(u32::from(value.scale()))
            }) {
                return Err(invalid(
                    "Occurrence chance percentage is outside 0 through 100",
                ));
            }
            let random_policy = match row.unspecified_random_policy.as_deref() {
                None => None,
                Some("StableUniformOrderedCandidates") => {
                    Some(RandomOutcomePolicy::StableUniformOrderedCandidates)
                }
                Some(_) => return Err(invalid("Occurrence random policy is unknown")),
            };
            Ok((
                row.choice_id,
                row.sequence,
                OccurrenceOutcome::new(
                    operations.into_boxed_slice(),
                    targets(row.targets.as_deref())?,
                    numeric_literals.into_boxed_slice(),
                    tokens(
                        row.parameter_refs.as_deref(),
                        "Occurrence parameter reference",
                    )?,
                    chance_percentages.into_boxed_slice(),
                    random_policy,
                ),
            ))
        }),
        config,
        "Occurrence outcome",
    )
}

fn grouped<T>(
    rows: impl IntoIterator<Item = Result<(i32, i32, T), UniverseCatalogLoadError>>,
    config: &SoraConfig,
    label: &str,
) -> Result<BTreeMap<i32, Vec<T>>, UniverseCatalogLoadError> {
    let mut groups: BTreeMap<i32, Vec<(u32, T)>> = BTreeMap::new();
    for row in rows {
        let (parent, sequence, value) = row?;
        if config.universe_occurrence_choice().get(&parent).is_none() {
            return Err(reference(format!("{label} parent choice is unresolved")));
        }
        let sequence = u32::try_from(sequence)
            .ok()
            .filter(|value| *value != 0)
            .ok_or_else(|| invalid(format!("{label} sequence must be positive")))?;
        groups.entry(parent).or_default().push((sequence, value));
    }
    groups
        .into_iter()
        .map(|(parent, mut values)| {
            values.sort_by_key(|value| value.0);
            if values
                .iter()
                .map(|value| value.0)
                .ne(1..=values.len() as u32)
            {
                return Err(invalid(format!("{label} sequence is not contiguous")));
            }
            Ok((parent, values.into_iter().map(|value| value.1).collect()))
        })
        .collect()
}

fn operation(value: &str) -> Result<OccurrenceOperation, UniverseCatalogLoadError> {
    match value {
        "Battle" => Ok(OccurrenceOperation::Battle),
        "Consume" => Ok(OccurrenceOperation::Consume),
        "Discard" => Ok(OccurrenceOperation::Discard),
        "Enhance" => Ok(OccurrenceOperation::Enhance),
        "Lose" => Ok(OccurrenceOperation::Lose),
        "Obtain" => Ok(OccurrenceOperation::Obtain),
        "Repair" => Ok(OccurrenceOperation::Repair),
        "Restore" => Ok(OccurrenceOperation::Restore),
        "Special" => Ok(OccurrenceOperation::Special),
        _ => Err(invalid("Occurrence outcome operation is unknown")),
    }
}

fn targets(values: Option<&[String]>) -> Result<Box<[OccurrenceTarget]>, UniverseCatalogLoadError> {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| match value.as_str() {
            "Blessing" => Ok(OccurrenceTarget::Blessing),
            "Character" => Ok(OccurrenceTarget::Character),
            "CosmicFragments" => Ok(OccurrenceTarget::CosmicFragments),
            "Curio" => Ok(OccurrenceTarget::Curio),
            "HP" => Ok(OccurrenceTarget::Hp),
            _ => Err(invalid("Occurrence target is unknown")),
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

fn authored_scalar(value: &str) -> Result<AuthoredScalar, UniverseCatalogLoadError> {
    let (value, unit) = value
        .strip_suffix('%')
        .map_or((value, AuthoredScalarUnit::Scalar), |value| {
            (value, AuthoredScalarUnit::Percent)
        });
    Ok(AuthoredScalar::new(parse_decimal(value)?, unit))
}

fn keys(
    values: Option<&[String]>,
    label: &str,
) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
    let mut result = Vec::new();
    let mut seen = BTreeSet::new();
    for value in values.unwrap_or_default() {
        checked_key(value, label)?;
        if !seen.insert(value) {
            return Err(invalid(format!("{label} is duplicated")));
        }
        result.push(value.as_str().into());
    }
    Ok(result.into_boxed_slice())
}

fn tokens(
    values: Option<&[String]>,
    label: &str,
) -> Result<Box<[Box<str>]>, UniverseCatalogLoadError> {
    values
        .unwrap_or_default()
        .iter()
        .map(|value| checked_source(value, label).map(Into::into))
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorRow {
    source_option_id: String,
    values: Vec<VectorValue>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VectorValue {
    index: u32,
    value: String,
}

fn parse_vectors(
    value: &str,
) -> Result<Box<[OccurrenceParameterVector]>, UniverseCatalogLoadError> {
    let rows: Vec<VectorRow> = serde_json::from_str(value)
        .map_err(|_| invalid("Occurrence parameter-vector JSON is malformed"))?;
    rows.into_iter()
        .map(|row| {
            checked_source(&row.source_option_id, "Occurrence source option ID")?;
            if row
                .values
                .iter()
                .map(|value| value.index)
                .ne(1..=row.values.len() as u32)
            {
                return Err(invalid(
                    "Occurrence parameter-vector indexes are not contiguous",
                ));
            }
            let values = row
                .values
                .iter()
                .map(|value| parse_decimal(&value.value))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(OccurrenceParameterVector::new(
                &row.source_option_id,
                values.into_boxed_slice(),
            ))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Vec::into_boxed_slice)
}

macro_rules! lower_id {
    ($name:ident, $ty:ty) => {
        fn $name(raw: i32, label: &str) -> Result<$ty, UniverseCatalogLoadError> {
            u32::try_from(raw)
                .ok()
                .and_then(<$ty>::new)
                .ok_or_else(|| invalid(format!("{label} ID must be a positive u32")))
        }
    };
}
lower_id!(occurrence_id, OccurrenceId);
lower_id!(variant_id, OccurrenceVariantId);
lower_id!(choice_id, OccurrenceChoiceId);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_scalar_preserves_percent_semantics() {
        assert_eq!(
            authored_scalar("50%").expect("percent").unit(),
            AuthoredScalarUnit::Percent
        );
        assert!(authored_scalar("50%%").is_err());
    }
}
