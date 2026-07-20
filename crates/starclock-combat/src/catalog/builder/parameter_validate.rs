//! Effective-level ability-parameter table construction.

use std::collections::BTreeMap;

use super::{CatalogBuildError, CatalogBuildErrorKind, error};
use crate::{AbilityId, catalog::definition::AbilityParameterDefinition, rule::model::RuleValue};

pub(super) fn table(
    definitions: Vec<AbilityParameterDefinition>,
) -> Result<BTreeMap<AbilityId, BTreeMap<Box<str>, RuleValue>>, CatalogBuildError> {
    let mut table = BTreeMap::<_, BTreeMap<_, _>>::new();
    for definition in definitions {
        let parameters = table.entry(definition.ability()).or_default();
        if parameters
            .insert(definition.stable_key().into(), definition.value().clone())
            .is_some()
        {
            return Err(error(
                CatalogBuildErrorKind::DuplicateDefinition,
                format!(
                    "duplicate ability parameter {}:{}",
                    definition.ability().get(),
                    definition.stable_key()
                ),
            ));
        }
    }
    Ok(table)
}
