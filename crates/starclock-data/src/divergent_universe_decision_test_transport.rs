//! Test-only table overrides over the actual current binary bundle.

use crate::divergent_universe_decisions_generated::{
    SCHEMA_FINGERPRINT,
    runtime::{SoraBundle, SoraDecode, SoraReadError, SoraTableSource},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) struct EditedRows(pub(super) BTreeMap<&'static str, Value>);

impl SoraTableSource for EditedRows {
    fn schema_fingerprint(&self) -> Result<&str, SoraReadError> {
        Ok(SCHEMA_FINGERPRINT)
    }
    fn decode_table<T: SoraDecode + DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Vec<T>, SoraReadError> {
        if let Some(values) = self.0.get(name) {
            serde_json::from_value(values.clone())
                .map_err(|error| SoraReadError::new(error.to_string()))
        } else {
            let bundle = SoraBundle::parse(include_bytes!(
                "../../../config/divergent-universe-decisions-generated/config.sora"
            ))?;
            bundle.decode_table(name)
        }
    }
}
