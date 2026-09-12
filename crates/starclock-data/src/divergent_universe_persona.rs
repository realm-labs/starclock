//! Admission guard for reference-only Persona source obligations.

use std::collections::BTreeSet;

use crate::divergent_universe::{DivergentUniverseDataError, error};
use crate::divergent_universe_generated::{
    SoraConfig,
    divergent_universe_persona_source_obligations::DivergentUniversePersonaSourceObligations,
    du_coverage_state::DuCoverageState, du_ownership::DuOwnership,
};

pub(super) fn validate_persona_obligations(
    config: &SoraConfig,
) -> Result<(), DivergentUniverseDataError> {
    validate_rows(
        config
            .divergent_universe_persona_source_obligations()
            .ordered_rows(),
    )
}

fn validate_rows<'a>(
    rows: impl Iterator<Item = &'a DivergentUniversePersonaSourceObligations>,
) -> Result<(), DivergentUniverseDataError> {
    let mut locators = BTreeSet::new();
    let mut current = 0_u32;
    let mut pending = 0_u32;
    for row in rows {
        let Some(locator) = row.source_locator.as_deref() else {
            return Err(error("Persona source locator is absent"));
        };
        if !locator.starts_with("ExcelOutput/RoguePersona")
            || !locators.insert(locator)
            || row.runtime_disposition.as_deref() != Some("Unimplemented")
            || !row
                .interpretation
                .as_deref()
                .is_some_and(|value| !value.is_empty())
        {
            return Err(error(
                "Persona source identity or unimplemented boundary drift",
            ));
        }
        let parents: Vec<String> =
            serde_json::from_str(row.parent_sources.as_deref().unwrap_or(""))
                .map_err(|_| error("Persona parent source references are invalid"))?;
        match row.selector_proof.as_deref() {
            Some("CurrentLayerReference")
                if matches!(row.ownership, DuOwnership::DivergentUniverse)
                    && matches!(row.coverage_state, DuCoverageState::Researched)
                    && !parents.is_empty() =>
            {
                current = current
                    .checked_add(1)
                    .ok_or_else(|| error("Persona current row count overflow"))?;
            }
            Some("PendingSelectorProof")
                if matches!(row.ownership, DuOwnership::SharedCandidate)
                    && matches!(row.coverage_state, DuCoverageState::Cataloged)
                    && parents.is_empty() =>
            {
                pending = pending
                    .checked_add(1)
                    .ok_or_else(|| error("Persona pending row count overflow"))?;
            }
            _ => {
                return Err(error(
                    "Persona source row was silently promoted or excluded",
                ));
            }
        }
    }
    if (current, pending) != (78, 469) {
        return Err(error("Persona source obligation denominator drift"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_rows;
    use crate::divergent_universe_generated::{
        SoraConfig, du_coverage_state::DuCoverageState, du_ownership::DuOwnership,
        runtime::SoraBundle,
    };

    #[test]
    fn persona_source_only_bundle_rejects_silent_promotion_and_lost_membership() {
        let bundle = SoraBundle::parse(include_bytes!(
            "../../../config/divergent-universe-generated/config.sora"
        ))
        .unwrap();
        let config = SoraConfig::from_source(&bundle).unwrap();
        let baseline = config
            .divergent_universe_persona_source_obligations()
            .ordered_rows()
            .cloned()
            .collect::<Vec<_>>();
        validate_rows(baseline.iter()).unwrap();
        for mutation in 0..6 {
            let mut rows = baseline.clone();
            let pending = rows
                .iter_mut()
                .find(|row| row.selector_proof.as_deref() == Some("PendingSelectorProof"))
                .unwrap();
            match mutation {
                0 => pending.coverage_state = DuCoverageState::DataReady,
                1 => pending.ownership = DuOwnership::DivergentUniverse,
                2 => pending.runtime_disposition = Some("ExactIntegrated".into()),
                3 => pending.selector_proof = Some("PrefixMatch".into()),
                4 => pending.parent_sources = Some("[\"invented\"]".into()),
                _ => {
                    rows.pop();
                }
            }
            assert!(
                validate_rows(rows.iter()).is_err(),
                "accepted mutation {mutation}"
            );
        }
    }
}
