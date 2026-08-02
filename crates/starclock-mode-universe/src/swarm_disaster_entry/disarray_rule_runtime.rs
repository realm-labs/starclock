//! Production bindings for Countdown, Disarray, and boss-decay mechanic rules.

use serde::Deserialize;
use starclock_activity::{ActivityCondition, ActivityProgramDefinition, ActivityTransactionState};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{SwarmDisasterRuntimeInstance, countdown::CountdownRuntimeCatalog};

#[derive(Clone, Debug)]
pub(super) struct DisarrayRuleRuntimeCatalog {
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

const CONTRACTS: [RuleContract; 3] = [
    RuleContract {
        family: "boss-decay-stack",
        domain: "CrossBattle",
        triggers: &["PlaneBossChoiceAccepted", "FinalBossBattleSpecRequested"],
        slots: &["boss-decay-selection-set"],
        steps: &[
            ("ReviewTierThreshold", "tier threshold"),
            ("ReviewStacking", "stacking"),
            ("ReviewCap", "cap"),
            ("ReviewBossEffectOrder", "boss effect order"),
        ],
    },
    RuleContract {
        family: "countdown-lifecycle",
        domain: "Activity",
        triggers: &["RunStarted", "MapMoveAccepted", "PlaneTransitioned"],
        slots: &["countdown"],
        steps: &[
            ("ReviewInitialValue", "initial value"),
            ("ReviewMovementDelta", "movement delta"),
            ("ReviewClamp", "clamp"),
            ("ReviewPlaneCarry", "plane carry"),
        ],
    },
    RuleContract {
        family: "planar-disarray-transition",
        domain: "CrossBattle",
        triggers: &["MapMoveAccepted", "BattleSpecRequested"],
        slots: &["countdown", "planar-disarray-tier"],
        steps: &[
            ("ReviewEntryBoundary", "entry boundary"),
            ("ReviewTierSelection", "tier selection"),
            ("ReviewSingleTransition", "single transition"),
            ("ReviewBattleProjection", "battle projection"),
        ],
    },
];

impl DisarrayRuleRuntimeCatalog {
    pub(super) fn compile(
        mut inputs: [MechanicRuleRuntimeInput; 3],
    ) -> Result<Self, UniverseCatalogLoadError> {
        inputs.sort_unstable_by(|left, right| left.family.cmp(&right.family));
        for (input, contract) in inputs.iter().zip(CONTRACTS.iter()) {
            validate_rule(input, contract)?;
        }
        Ok(Self {
            digest: rule_digest(&inputs),
        })
    }

    pub(super) fn compile_move(
        &self,
        countdown: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
        adjustments: &[(u32, i64)],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        countdown.compile_move(state, adjustments)
    }

    pub(super) fn compile_adjustments(
        &self,
        countdown: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
        adjustments: &[(u32, i64)],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        countdown.compile_adjustments(state, adjustments)
    }

    pub(super) fn compile_boss_decay_selection(
        &self,
        countdown: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
        keys: &[&str],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        countdown.compile_boss_decay_selection(state, keys)
    }

    pub(super) fn completion_requirements(
        &self,
        countdown: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
        plane_layer: u8,
    ) -> Result<Vec<ActivityCondition>, UniverseCatalogLoadError> {
        countdown.completion_requirements(state, plane_layer)
    }

    pub(super) fn countdown(
        &self,
        catalog: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        catalog.countdown(state)
    }

    pub(super) fn disarray_level(
        &self,
        catalog: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        catalog.disarray_level(state)
    }

    pub(super) fn disarray_modifiers(
        &self,
        catalog: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
    ) -> Result<(i64, i64, i64), UniverseCatalogLoadError> {
        catalog.disarray_modifiers(state)
    }

    pub(super) fn warning_active(
        &self,
        catalog: &CountdownRuntimeCatalog,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        catalog.warning_active(state)
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the three exact Sora Disarray-rule bindings.
    #[must_use]
    pub fn disarray_rule_runtime_digest(&self) -> [u8; 32] {
        self.disarray_rules.digest
    }
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid Disarray mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid Disarray mechanic-rule program"))?;
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
        Err(reference("Disarray mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.disarray-rule-runtime");
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
#[path = "disarray_rule_runtime_tests.rs"]
mod tests;
