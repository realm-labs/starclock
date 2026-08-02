//! Production bindings for boss-choice and final-boss consequence rules.

use serde::Deserialize;

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{SwarmDisasterRuntimeInstance, plane_transition::PlaneTransitionRuntimeCatalog};

#[derive(Clone, Debug)]
pub(super) struct BossRuleRuntimeCatalog {
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
        family: "boss-choice-consequence",
        trigger: "PlaneBossChoiceAccepted",
        slots: &["boss-choice-set", "final-boss-contributions"],
        steps: &[
            ("ReviewBossChoice", "boss choice"),
            ("ReviewWeaknessConsequence", "weakness consequence"),
            ("ReviewLaterBossBinding", "later boss binding"),
        ],
    },
    RuleContract {
        family: "final-boss-consequence",
        trigger: "FinalBossBattleSpecRequested",
        slots: &[
            "final-boss-selection",
            "boss-decay-selection-set",
            "active-interplays",
            "planar-disarray-tier",
        ],
        steps: &[
            ("ReviewBossPool", "boss pool"),
            ("ReviewPriorChoiceState", "prior choice state"),
            (
                "ReviewPropagationOrInterplayContribution",
                "Propagation or Interplay contribution",
            ),
            ("ReviewDisarrayContribution", "Disarray contribution"),
        ],
    },
];

impl BossRuleRuntimeCatalog {
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

    pub(super) const fn transitions<'a>(
        &self,
        transitions: &'a PlaneTransitionRuntimeCatalog,
    ) -> &'a PlaneTransitionRuntimeCatalog {
        transitions
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the exact Sora boss-rule partition binding.
    #[must_use]
    pub fn boss_rule_runtime_digest(&self) -> [u8; 32] {
        self.boss_rules.digest
    }
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid boss mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid boss mechanic-rule program"))?;
    let valid = input.key.as_ref() == format!("swarm-disaster.mechanic-rule.{}", contract.family)
        && input.family.as_ref() == contract.family
        && input.domain.as_ref() == "CrossBattle"
        && input
            .triggers
            .iter()
            .map(Box::as_ref)
            .eq([contract.trigger])
        && input
            .fixtures
            .iter()
            .map(Box::as_ref)
            .eq([format!("swarm-disaster.fixture.{}", contract.family)])
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
        Err(reference("boss mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput; 2]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.boss-rule-runtime");
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
#[path = "boss_rule_runtime_tests.rs"]
mod tests;
