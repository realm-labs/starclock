//! Immutable mechanic-rule runtime input boundary.

use super::SwarmDisasterContentCatalog;

#[derive(Clone, Debug)]
pub(crate) struct MechanicRuleRuntimeInput {
    pub(crate) id: u32,
    pub(crate) key: Box<str>,
    pub(crate) family: Box<str>,
    pub(crate) domain: Box<str>,
    pub(crate) triggers: Box<[Box<str>]>,
    pub(crate) slots: Box<str>,
    pub(crate) program: Box<str>,
    pub(crate) fixtures: Box<[Box<str>]>,
    pub(crate) source_disposition: Box<str>,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn mechanic_rule_runtime_input(
        &self,
        family: &str,
    ) -> Option<MechanicRuleRuntimeInput> {
        self.mechanic_rules
            .iter()
            .find(|row| row.family_key.as_ref() == family)
            .map(|row| MechanicRuleRuntimeInput {
                id: row.id.0,
                key: row.key.clone(),
                family: row.family_key.clone(),
                domain: row.domain.clone(),
                triggers: row.triggers.clone(),
                slots: row.slots.clone(),
                program: row.program.clone(),
                fixtures: row.fixture_keys.clone(),
                source_disposition: row.disposition.clone(),
            })
    }
}
