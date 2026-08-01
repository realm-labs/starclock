//! Immutable source/rule coverage inputs for production closure.

use super::SwarmDisasterContentCatalog;

#[derive(Clone, Debug)]
pub(crate) struct MechanicCoverageInput {
    pub(crate) key: Box<str>,
    pub(crate) family: Box<str>,
    pub(crate) fixture_keys: Box<[Box<str>]>,
}

impl SwarmDisasterContentCatalog {
    pub(crate) fn source_coverage_categories(&self) -> impl ExactSizeIterator<Item = (&str, u32)> {
        self.audit
            .coverage_categories
            .iter()
            .map(|row| (row.key.as_ref(), row.obligations))
    }

    pub(crate) fn mechanic_coverage_inputs(&self) -> Box<[MechanicCoverageInput]> {
        self.mechanic_rules
            .iter()
            .map(|row| MechanicCoverageInput {
                key: row.key.clone(),
                family: row.family_key.clone(),
                fixture_keys: row.fixture_keys.clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}
