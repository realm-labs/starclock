//! Immutable effective-level ability-parameter access.

use std::collections::BTreeMap;

use super::{CombatCatalog, definition::ProgramDefinition};
use crate::{AbilityId, ProgramId, rule::model::RuleValue};

pub(super) type Table = BTreeMap<AbilityId, BTreeMap<Box<str>, RuleValue>>;

pub(super) fn count(table: &Table) -> usize {
    table.values().map(BTreeMap::len).sum()
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

impl crate::rule::evaluate::AbilityParameterReader for CombatCatalog {
    fn ability_parameter(&self, ability: AbilityId, key: &str) -> Option<RuleValue> {
        self.ability_parameter(ability, key).cloned()
    }
}

impl crate::rule::evaluate::ProgramLookup for CombatCatalog {
    fn program_steps(&self, id: ProgramId) -> Option<&[crate::rule::model::ProgramStep]> {
        self.program(id).map(ProgramDefinition::steps)
    }
}
