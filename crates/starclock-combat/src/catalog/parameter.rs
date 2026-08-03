//! Immutable effective-level ability-parameter access.

use crate::rule::{
    evaluate::{AbilityParameterReader, ProgramLookup},
    model::ProgramStep,
};
use std::collections::BTreeMap;

use super::definition;
use super::{CombatCatalog, definition::ProgramDefinition};
use crate::{AbilityId, ProgramId, rule::model::RuleValue};

pub(super) type Table = BTreeMap<AbilityId, BTreeMap<Box<str>, RuleValue>>;

pub(super) fn count(table: &Table) -> usize {
    table.values().map(BTreeMap::len).sum()
}

pub(super) fn definitions(
    table: &Table,
) -> impl Iterator<Item = definition::AbilityParameterDefinition> + '_ {
    table.iter().flat_map(|(ability, parameters)| {
        parameters.iter().map(|(stable_key, value)| {
            definition::AbilityParameterDefinition::new(*ability, stable_key.clone(), value.clone())
                .expect("validated catalog parameters remain valid builder inputs")
        })
    })
}

impl CombatCatalog {
    /// Looks up one selected effective-level parameter by exact semantic key.
    #[must_use]
    pub fn ability_parameter(&self, ability: AbilityId, stable_key: &str) -> Option<&RuleValue> {
        self.ability_parameters
            .get(&ability)
            .and_then(|parameters| parameters.get(stable_key))
    }
}

impl AbilityParameterReader for CombatCatalog {
    fn ability_parameter(&self, ability: AbilityId, key: &str) -> Option<RuleValue> {
        self.ability_parameter(ability, key).cloned()
    }
}

impl ProgramLookup for CombatCatalog {
    fn program_steps(&self, id: ProgramId) -> Option<&[ProgramStep]> {
        self.program(id).map(ProgramDefinition::steps)
    }
}
