//! Production bindings for Path unlock and Resonance Interplay rules.

use serde::Deserialize;
use starclock_activity::{ActivityProgramDefinition, ActivityTransactionState};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{
    SwarmDisasterRuntimeInstance,
    path_runtime::{CompiledPathRuntime, PathRuntimeCatalog},
};

#[derive(Clone, Debug)]
pub(super) struct PathRuleRuntimeCatalog {
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
    domain: &'static str,
    triggers: &'static [&'static str],
    slots: &'static [&'static str],
    steps: &'static [(&'static str, &'static str)],
}

const CONTRACTS: [RuleContract; 2] = [
    RuleContract {
        family: "path-and-propagation-unlock",
        domain: "Activity",
        triggers: &["MechanicalChapterRequirementSatisfied"],
        slots: &["available-paths", "mechanical-chapters"],
        steps: &[
            (
                "ReviewMechanicalChapterLocator",
                "mechanical chapter locator",
            ),
            ("ReviewPathUnlock", "Path unlock"),
            ("ReviewPropagationAvailability", "Propagation availability"),
        ],
    },
    RuleContract {
        family: "resonance-interplay",
        domain: "CrossBattle",
        triggers: &["BlessingInventoryMutationCommitted", "BattleSpecRequested"],
        slots: &["blessing-inventory", "active-interplays"],
        steps: &[
            ("ReviewMainPathThreshold", "main Path threshold"),
            ("ReviewSubPathThreshold", "sub Path threshold"),
            ("ReviewEffectBinding", "effect binding"),
            ("ReviewActivationOrder", "activation order"),
        ],
    },
];

impl PathRuleRuntimeCatalog {
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

    pub(super) fn select(
        &self,
        catalog: &PathRuntimeCatalog,
        shared_path: &str,
        audience_unlock: Option<&str>,
        bonus: Option<&str>,
    ) -> Result<CompiledPathRuntime, UniverseCatalogLoadError> {
        catalog.select(shared_path, audience_unlock, bonus)
    }

    pub(super) fn progression_unlock_id<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> Option<&'a str> {
        instance.path_runtime.progression_unlock_id()
    }

    pub(super) fn boost_binding<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> (&'a str, &'a str) {
        instance.path_runtime.boost_binding()
    }

    pub(super) fn resonance_bindings<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> impl ExactSizeIterator<Item = (&'a str, &'a str, u16, &'a str)> {
        instance.path_runtime.resonance_bindings()
    }

    pub(super) fn resonance_parameters<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        key: &str,
    ) -> Option<impl ExactSizeIterator<Item = &'a str>> {
        instance.path_runtime.resonance_parameters(key)
    }

    pub(super) fn compile_interplays(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        blessing_counts: &[(String, u16)],
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        instance
            .path_runtime
            .compile_interplays(state, blessing_counts)
    }

    pub(super) fn active_interplays<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<Vec<(&'a str, &'a str, &'a str)>, UniverseCatalogLoadError> {
        instance.path_runtime.active_interplays(state)
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the two exact Sora Path-rule bindings.
    #[must_use]
    pub fn path_rule_runtime_digest(&self) -> [u8; 32] {
        self.path_rules.digest
    }
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid Path mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid Path mechanic-rule program"))?;
    let expected_key = format!("swarm-disaster.mechanic-rule.{}", contract.family);
    let expected_fixture = format!("swarm-disaster.fixture.{}", contract.family);
    let valid = input.key.as_ref() == expected_key
        && input.family.as_ref() == contract.family
        && input.domain.as_ref() == contract.domain
        && input
            .triggers
            .iter()
            .map(Box::as_ref)
            .eq(contract.triggers.iter().copied())
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
        Err(reference("Path mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.path-rule-runtime");
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
#[path = "path_rule_runtime_tests.rs"]
mod tests;
