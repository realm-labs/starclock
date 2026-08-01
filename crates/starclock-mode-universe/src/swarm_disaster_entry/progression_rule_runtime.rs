//! Production bindings for Communing Trail and Pathstrider progression rules.

use serde::Deserialize;
use starclock_activity::{ActivityProgramDefinition, ActivityTransactionState};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{
    SwarmDisasterRuntimeInstance,
    trail::{CompiledTrailRuntime, TrailRuntimeCatalog},
};

const REVISION: &str = "swarm-disaster-progression-rule-runtime-v1";

#[derive(Clone, Debug)]
pub(super) struct ProgressionRuleRuntimeCatalog {
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
    slots: &'static [(&'static str, &'static str)],
    steps: &'static [(&'static str, &'static str)],
    unresolved: &'static str,
}

const CONTRACTS: [RuleContract; 2] = [
    RuleContract {
        family: "communing-trail-effect",
        domain: "CrossBattle",
        triggers: &["CommuningTrailNodeUnlocked", "BattleSpecRequested"],
        slots: &[
            ("communing-trail-unlocks", "Activity"),
            ("battle-contributions", "Battle"),
        ],
        steps: &[
            ("ReviewPrerequisite", "prerequisite"),
            ("ReviewThreshold", "threshold"),
            ("ReviewRunContribution", "run contribution"),
            ("ReviewBattleProjection", "battle projection"),
        ],
        unresolved: "NotApplicable",
    },
    RuleContract {
        family: "pathstrider-progress",
        domain: "Activity",
        triggers: &["ActivityOperationCommitted"],
        slots: &[
            ("pathstrider-progress", "Activity"),
            ("pathstrider-unlocks", "Activity"),
        ],
        steps: &[
            ("ReviewFinishCondition", "finish condition"),
            ("ReviewProgressUpdate", "progress update"),
            ("ReviewComparison", "comparison"),
            ("ReviewUnlockConsequence", "unlock consequence"),
        ],
        unresolved: "FailClosed",
    },
];

impl ProgressionRuleRuntimeCatalog {
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

    pub(super) fn select_trail(
        &self,
        catalog: &TrailRuntimeCatalog,
        progression: &[Box<str>],
        communing: &[(u32, u16)],
    ) -> Result<CompiledTrailRuntime, UniverseCatalogLoadError> {
        catalog.select(progression, communing)
    }

    pub(super) fn compile_trail_run_start(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        instance.trail.compile_run_start(state)
    }

    pub(super) fn trail_nodes<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> impl ExactSizeIterator<Item = (&'a str, u16)> {
        instance.trail.nodes()
    }

    pub(super) fn trail_prerequisites<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        node: &str,
    ) -> Option<impl ExactSizeIterator<Item = &'a str>> {
        instance.trail.prerequisites(node)
    }

    pub(super) fn trail_battle_effects<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> impl ExactSizeIterator<Item = (&'a str, &'a str, &'a str)> {
        instance.trail.battle()
    }

    pub(super) fn trail_battle_effect_parameters<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        effect_ref: &str,
    ) -> Option<impl ExactSizeIterator<Item = &'a str>> {
        instance.trail.battle_parameters(effect_ref)
    }

    pub(super) const fn trail_abandon_reward(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
    ) -> i64 {
        instance.trail.abandon_reward()
    }

    pub(super) const fn trail_next_plane_rerolls(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
    ) -> i64 {
        instance.trail.next_plane_rerolls()
    }

    pub(super) fn compile_trail_battle_entry_accounting(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        plane_layer: u8,
        boss: bool,
        previous_first_plane_completed: bool,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        instance.trail.compile_battle_entry_accounting(
            state,
            plane_layer,
            boss,
            previous_first_plane_completed,
        )
    }

    pub(super) fn compile_objective_completion(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        external_condition: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let cabinet = instance.pathstrider.objective_cabinet(external_condition)?;
        let objective = external_condition
            .strip_prefix("swarm-disaster.external-quest-condition.")
            .ok_or_else(|| reference("invalid Pathstrider external quest condition"))?;
        instance
            .communing_rules
            .compile_cabinet_completion(instance, state, cabinet, objective)
    }

    pub(super) fn finish_conditions<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> impl ExactSizeIterator<Item = (&'a str, &'a str, &'a str, u32)> {
        instance.pathstrider.finish_conditions()
    }

    pub(super) fn finish_parameters<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
        condition: &str,
    ) -> Option<impl ExactSizeIterator<Item = &'a str>> {
        instance.pathstrider.finish_parameters(condition)
    }

    pub(super) fn compile_progress(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        condition: &str,
        observed_progress: u32,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        instance
            .pathstrider
            .compile_progress(state, condition, observed_progress)
    }

    pub(super) fn unlock_applied(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        unlock: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance.pathstrider.unlock_applied(state, unlock)
    }

    pub(super) fn chapters<'a>(
        &self,
        instance: &'a SwarmDisasterRuntimeInstance,
    ) -> impl ExactSizeIterator<Item = (&'a str, u8, Option<(u32, u16)>, bool)> {
        instance.pathstrider.chapters()
    }

    pub(super) fn compile_chapter_availability(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        instance
            .pathstrider
            .compile_chapter_availability(instance, state)
    }

    pub(super) fn chapter_available(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
        chapter: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        instance.pathstrider.chapter_available(state, chapter)
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the two exact Sora progression-rule bindings.
    #[must_use]
    pub fn progression_rule_runtime_digest(&self) -> [u8; 32] {
        self.progression_rules.digest
    }
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid progression mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid progression mechanic-rule program"))?;
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
            slot.id.as_ref() == expected.0 && slot.owner.as_ref() == expected.1
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
                    && step.unresolved_behavior.as_ref() == contract.unresolved
            });
    if valid {
        Ok(())
    } else {
        Err(reference("progression mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.progression-rule-runtime.v1");
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
#[path = "progression_rule_runtime_tests.rs"]
mod tests;
