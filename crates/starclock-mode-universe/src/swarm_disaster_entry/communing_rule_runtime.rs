//! Production bindings for Communing choice and dimension-point rules.

use serde::Deserialize;
use starclock_activity::{ActivityProgramDefinition, ActivityTransactionState};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::SwarmDisasterRuntimeInstance;

#[derive(Clone, Debug)]
pub(super) struct CommuningRuleRuntimeCatalog {
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

struct RuleContract {
    family: &'static str,
    trigger: &'static str,
    slots: &'static [&'static str],
    steps: &'static [(&'static str, &'static str)],
}

const CONTRACTS: [RuleContract; 2] = [
    RuleContract {
        family: "communing-choice",
        trigger: "CommuningChoiceAccepted",
        slots: &["aeon-choice-counters", "communing-cabinet"],
        steps: &[
            ("ReviewAeonEligibility", "Aeon eligibility"),
            ("ReviewChoiceOrder", "choice order"),
            ("ReviewPointChanges", "point changes"),
            ("ReviewSimultaneousUnlock", "simultaneous unlock"),
        ],
    },
    RuleContract {
        family: "communing-dimension-points",
        trigger: "CommuningPointAdjustmentAccepted",
        slots: &["communing-dimension-points"],
        steps: &[
            ("ReviewPointAddition", "point addition"),
            ("ReviewCap", "cap"),
            ("ReviewCarry", "carry"),
            ("ReviewDimensionThreshold", "dimension threshold"),
        ],
    },
];

impl CommuningRuleRuntimeCatalog {
    pub(super) fn compile(
        mut inputs: [MechanicRuleRuntimeInput; 2],
    ) -> Result<Self, UniverseCatalogLoadError> {
        inputs.sort_unstable_by(|left, right| left.family.cmp(&right.family));
        for (input, contract) in inputs.iter().zip(CONTRACTS.iter()) {
            validate_rule(input, contract)?;
        }
        Ok(Self {
            digest: rule_digest(&inputs),
        })
    }

    pub(super) fn choices<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        story_stage: u16,
    ) -> impl Iterator<Item = &'a str> {
        instance.communing.choices(story_stage)
    }

    pub(super) fn choice_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        story_stage: u16,
        choice: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance
            .communing
            .choice_available(state, story_stage, choice)
    }

    pub(super) fn compile_choice(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        story_stage: u16,
        choice: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance
            .communing
            .compile_choice(state, story_stage, choice)
    }

    pub(super) fn choice_count(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        path: &str,
    ) -> Result<i64, UniverseCatalogLoadError> {
        instance.communing.choice_count(state, path)
    }

    pub(super) fn dimension_points(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        dimension: &str,
    ) -> Result<i64, UniverseCatalogLoadError> {
        instance.communing.dimension_points(state, dimension)
    }

    pub(super) fn dimension_maximum(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        dimension: &str,
    ) -> Option<i64> {
        instance.communing.dimension_maximum(dimension)
    }

    pub(super) fn available_cabinets<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<Box<[&'a str]>, UniverseCatalogLoadError> {
        instance.communing.available_cabinets(state)
    }

    pub(super) fn cabinet_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        cabinet: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance.communing.cabinet_available(state, cabinet)
    }

    pub(super) fn compile_cabinet_completion(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        cabinet: &str,
        completed_objective: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance
            .communing
            .compile_cabinet_completion(state, cabinet, completed_objective)
    }

    pub(super) fn cabinet_objective<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        cabinet: &str,
    ) -> Option<&'a str> {
        instance.communing.cabinet_objective(cabinet)
    }

    pub(super) fn cabinet_prerequisites<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        cabinet: &str,
    ) -> Option<impl ExactSizeIterator<Item = &'a str>> {
        instance.communing.cabinet_prerequisites(cabinet)
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the two exact Sora Communing-rule bindings.
    #[must_use]
    pub fn communing_rule_runtime_digest(&self) -> [u8; 32] {
        self.communing_rules.digest
    }
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid Communing mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid Communing mechanic-rule program"))?;
    let expected_key = format!("swarm-disaster.mechanic-rule.{}", contract.family);
    let expected_fixture = format!("swarm-disaster.fixture.{}", contract.family);
    let valid = input.key.as_ref() == expected_key
        && input.family.as_ref() == contract.family
        && input.domain.as_ref() == "Activity"
        && input
            .triggers
            .iter()
            .map(Box::as_ref)
            .eq([contract.trigger])
        && input
            .fixtures
            .iter()
            .map(Box::as_ref)
            .eq([expected_fixture])
        && input.source_disposition.as_ref() == "ReferenceOnly"
        && slots.len() == contract.slots.len()
        && slots.iter().zip(contract.slots).all(|(slot, expected)| {
            slot.id.as_ref() == *expected && slot.owner.as_ref() == "Activity"
        })
        && steps.len() == contract.steps.len()
        && steps
            .iter()
            .zip(contract.steps)
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
        Err(reference("Communing mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.communing-rule-runtime");
    for input in inputs {
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
    }
    encoder.finish()
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "communing_rule_runtime_tests.rs"]
mod tests;
