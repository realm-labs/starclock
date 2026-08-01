//! Production binding for the Swarm Service/Adventure rule.

use serde::Deserialize;

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{
    SwarmDisasterRuntimeInstance, service_adventure_runtime::ServiceAdventureRuntimeCatalog,
};

const REVISION: &str = "swarm-disaster-service-rule-runtime-v1";

#[derive(Clone, Debug)]
pub(super) struct ServiceRuleRuntimeCatalog {
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

const SLOTS: [&str; 3] = ["cosmic-fragments", "run-inventory", "external-outcome"];
const STEPS: [(&str, &str); 3] = [
    ("ReviewEligibility", "eligibility"),
    ("ReviewPriceOrAbstractTier", "price or abstract tier"),
    ("ReviewOfferedResultBoundary", "offered result boundary"),
];

impl ServiceRuleRuntimeCatalog {
    pub(super) fn compile(
        input: MechanicRuleRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        validate_rule(&input)?;
        Ok(Self {
            digest: rule_digest(&input),
        })
    }

    pub(super) const fn runtime<'a>(
        &self,
        runtime: &'a ServiceAdventureRuntimeCatalog,
    ) -> &'a ServiceAdventureRuntimeCatalog {
        runtime
    }
}

impl SwarmDisasterRuntimeInstance {
    pub(super) fn service_runtime(&self) -> &ServiceAdventureRuntimeCatalog {
        self.service_rules.runtime(&self.service_adventure)
    }

    /// Deterministic digest of the exact Sora Service/Adventure rule binding.
    #[must_use]
    pub fn service_rule_runtime_digest(&self) -> [u8; 32] {
        self.service_rules.digest
    }
}

fn validate_rule(input: &MechanicRuleRuntimeInput) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid Service mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid Service mechanic-rule program"))?;
    let valid = input.key.as_ref() == "swarm-disaster.mechanic-rule.service-and-adventure"
        && input.family.as_ref() == "service-and-adventure"
        && input.domain.as_ref() == "Activity"
        && input
            .triggers
            .iter()
            .map(Box::as_ref)
            .eq(["ServicePurchaseAccepted", "AdventureOutcomeOffered"])
        && input
            .fixtures
            .iter()
            .map(Box::as_ref)
            .eq(["swarm-disaster.fixture.service-and-adventure"])
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
        Err(reference("Service mechanic-rule contract drift"))
    }
}

fn rule_digest(input: &MechanicRuleRuntimeInput) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.service-rule-runtime.v1");
    encoder.text(REVISION);
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
#[path = "service_rule_runtime_tests.rs"]
mod tests;
