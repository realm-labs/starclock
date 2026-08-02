//! Production binding for the Swarm Curio lifecycle rule.

use serde::Deserialize;

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{SwarmDisasterRuntimeInstance, content_runtime::ContentRuntimeCatalog};

#[derive(Clone, Debug)]
pub(super) struct CurioRuleRuntimeCatalog {
    digest: [u8; 32],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleSlot {
    id: Box<str>,
    owner: Box<str>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleStep {
    sequence: u16,
    operation: Box<str>,
    source_fact: Box<str>,
    unresolved_behavior: Box<str>,
}

const SLOTS: [&str; 2] = ["curio-inventory", "curio-state"];
const STEPS: [(&str, &str); 3] = [
    ("Review1000SeriesModeCopy", "1000-series mode copy"),
    ("ReviewChargesOrState", "charges or state"),
    ("ReviewRepairOrReplacement", "repair or replacement"),
];

impl CurioRuleRuntimeCatalog {
    pub(super) fn compile(
        input: MechanicRuleRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        validate_rule(&input)?;
        Ok(Self {
            digest: rule_digest(&input),
        })
    }

    pub(super) const fn content<'a>(
        &self,
        content: &'a ContentRuntimeCatalog,
    ) -> &'a ContentRuntimeCatalog {
        content
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the exact Sora Curio-lifecycle rule binding.
    #[must_use]
    pub fn curio_rule_runtime_digest(&self) -> [u8; 32] {
        self.curio_rules.digest
    }
}

fn validate_rule(input: &MechanicRuleRuntimeInput) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid Curio mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid Curio mechanic-rule program"))?;
    let valid = input.key.as_ref() == "swarm-disaster.mechanic-rule.curio-lifecycle"
        && input.family.as_ref() == "curio-lifecycle"
        && input.domain.as_ref() == "CrossBattle"
        && input.triggers.iter().map(Box::as_ref).eq([
            "CurioGranted",
            "BattleCompleted",
            "CurioRepairRequested",
        ])
        && input
            .fixtures
            .iter()
            .map(Box::as_ref)
            .eq(["swarm-disaster.fixture.curio-lifecycle"])
        && input.source_disposition.as_ref() == "ReferenceOnly"
        && slots.len() == SLOTS.len()
        && slots.iter().zip(SLOTS).all(|(slot, expected)| {
            slot.id.as_ref() == expected && slot.owner.as_ref() == "Activity"
        })
        && steps.len() == STEPS.len()
        && steps
            .iter()
            .zip(STEPS)
            .enumerate()
            .all(|(index, (step, expected))| {
                usize::from(step.sequence) == index + 1
                    && step.operation.as_ref() == expected.0
                    && step.source_fact.as_ref() == expected.1
                    && step.unresolved_behavior.as_ref() == "FailClosed"
            });
    if valid {
        Ok(())
    } else {
        Err(reference("Curio mechanic-rule contract drift"))
    }
}

fn rule_digest(input: &MechanicRuleRuntimeInput) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.curio-rule-runtime");
    encoder.u32(input.id);
    encoder.text(&input.key);
    encoder.text(&input.family);
    encoder.text(&input.domain);
    for trigger in &input.triggers {
        encoder.text(trigger);
    }
    encoder.text(&input.slots);
    encoder.text(&input.program);
    for fixture in &input.fixtures {
        encoder.text(fixture);
    }
    encoder.text(&input.source_disposition);
    encoder.finish()
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "curio_rule_runtime_tests.rs"]
mod tests;
