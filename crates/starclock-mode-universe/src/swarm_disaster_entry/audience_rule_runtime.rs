//! Production bindings for Audience Die passive, control, and targeting rules.

use serde::Deserialize;
use starclock_activity::{
    ActivityProgramDefinition, ActivityRngStreams, ActivityTransactionState, NodeId,
};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{
    SwarmDisasterRuntimeInstance,
    audience::{AudienceRuntimeCatalog, CompiledAudienceRuntime},
};

const REVISION: &str = "swarm-disaster-audience-rule-runtime-v1";

#[derive(Clone, Debug)]
pub(super) struct AudienceRuleRuntimeCatalog {
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
    triggers: &'static [&'static str],
    slots: &'static [&'static str],
    steps: &'static [(&'static str, &'static str)],
}

const CONTRACTS: [RuleContract; 3] = [
    RuleContract {
        family: "audience-die-passive",
        triggers: &["PathSelected", "RunStarted", "MapMoveAccepted"],
        slots: &["selected-path", "audience-die", "plane-graph"],
        steps: &[
            ("ReviewSelectedPath", "selected Path"),
            ("ReviewInitialEffect", "initial effect"),
            ("ReviewPassiveEffect", "passive effect"),
            ("ReviewGraphMutation", "graph mutation"),
        ],
    },
    RuleContract {
        family: "dice-face-targeting",
        triggers: &["DiceFaceAccepted"],
        slots: &["plane-graph", "dice-result"],
        steps: &[
            ("ReviewRarity", "rarity"),
            ("ReviewActivationStage", "activation stage"),
            ("ReviewOrderedTargets", "ordered targets"),
            ("ReviewNoLegalTarget", "no legal target"),
        ],
    },
    RuleContract {
        family: "dice-roll-reroll-cheat",
        triggers: &["DiceRolled", "DiceRerolled", "DiceCheatRequested"],
        slots: &["dice-result", "reroll-charges", "cheat-charges"],
        steps: &[
            ("ReviewRollResult", "roll result"),
            ("ReviewRerollConsumption", "reroll consumption"),
            ("ReviewCheatReplacement", "cheat replacement"),
            ("ReviewResultOrdering", "result ordering"),
        ],
    },
];

impl AudienceRuleRuntimeCatalog {
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

    pub(super) fn select(
        &self,
        catalog: &AudienceRuntimeCatalog,
        path: &str,
        die: &str,
        unlocks: &[Box<str>],
    ) -> Result<CompiledAudienceRuntime, UniverseCatalogLoadError> {
        catalog.select(path, die, unlocks)
    }

    pub(super) fn compile_initialization(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance.audience.compile_initialization(state)
    }

    pub(super) fn initialization_applied(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance.audience.initialization_applied(state)
    }

    pub(super) fn roll_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .roll_available(state, &faces(instance))
    }

    pub(super) fn reroll_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .reroll_available(state, &faces(instance))
    }

    pub(super) fn cheat_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .cheat_available(state, &faces(instance))
    }

    pub(super) fn abandon_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .abandon_available(state, &faces(instance))
    }

    pub(super) fn compile_roll(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .compile_roll(state, &faces(instance), rng)
    }

    pub(super) fn compile_reroll(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .compile_reroll(state, &faces(instance), rng)
    }

    pub(super) fn compile_cheat(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        selected_face: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance
            .dice_controls
            .compile_cheat(state, &faces(instance), selected_face)
    }

    pub(super) fn compile_abandon(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance.dice_controls.compile_abandon(
            state,
            &faces(instance),
            instance.trail.abandon_reward(),
        )
    }

    pub(super) fn resolution_face<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Option<&'a str> {
        instance
            .dice_controls
            .resolution_face_id(state)
            .and_then(|id| instance.audience.face_key(id))
    }

    pub(super) fn resolution_kind(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<Option<u8>, UniverseCatalogLoadError> {
        instance.dice_controls.resolution_kind(state)
    }

    pub(super) fn compile_face_activation(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance.face_effects.compile_activation(
            &instance.dice_controls,
            &instance.map,
            state,
            explicit_target,
            rng,
        )
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the three exact Sora Audience-rule bindings.
    #[must_use]
    pub fn audience_rule_runtime_digest(&self) -> [u8; 32] {
        self.audience_rules.digest
    }
}

fn faces(instance: &SwarmDisasterRuntimeInstance) -> Vec<(&str, u32)> {
    instance.audience.face_ids().collect()
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid Audience mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid Audience mechanic-rule program"))?;
    let expected_key = format!("swarm-disaster.mechanic-rule.{}", contract.family);
    let expected_fixture = format!("swarm-disaster.fixture.{}", contract.family);
    let valid = input.key.as_ref() == expected_key
        && input.family.as_ref() == contract.family
        && input.domain.as_ref() == "Activity"
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
        Err(reference("Audience mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.audience-rule-runtime.v1");
    encoder.text(REVISION);
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
#[path = "audience_rule_runtime_tests.rs"]
mod tests;
