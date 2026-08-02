//! Production contract binding for Swarm encounter selection.

use serde::Deserialize;

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::SwarmDisasterRuntimeInstance;

#[derive(Clone, Debug)]
pub(super) struct EncounterRuleRuntimeCatalog {
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

const SLOTS: [&str; 3] = [
    "resolved-domain",
    "difficulty-segment",
    "encounter-selection",
];
const STEPS: [(&str, &str); 4] = [
    ("ReviewRoomParent", "room parent"),
    ("ReviewWeightedGroup", "weighted group"),
    ("ReviewWaveAndEnemySlot", "wave and enemy slot"),
    ("ReviewBossAlternative", "boss alternative"),
];

impl EncounterRuleRuntimeCatalog {
    pub(super) fn compile(
        input: MechanicRuleRuntimeInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        validate_rule(&input)?;
        Ok(Self {
            digest: rule_digest(&input),
        })
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Digest of the encounter-selection contract consumed by Phase 6.
    #[must_use]
    pub fn encounter_rule_runtime_digest(&self) -> [u8; 32] {
        self.encounter_rule.digest
    }
}

fn validate_rule(input: &MechanicRuleRuntimeInput) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid encounter mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid encounter mechanic-rule program"))?;
    let valid = input.key.as_ref() == "swarm-disaster.mechanic-rule.encounter-selection"
        && input.family.as_ref() == "encounter-selection"
        && input.domain.as_ref() == "CrossBattle"
        && input
            .triggers
            .iter()
            .map(Box::as_ref)
            .eq(["BattleSpecRequested"])
        && input
            .fixtures
            .iter()
            .map(Box::as_ref)
            .eq(["swarm-disaster.fixture.encounter-selection"])
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
        Err(reference("encounter mechanic-rule contract drift"))
    }
}

fn rule_digest(input: &MechanicRuleRuntimeInput) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.encounter-rule-runtime");
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
#[path = "encounter_rule_runtime_tests.rs"]
mod tests;
