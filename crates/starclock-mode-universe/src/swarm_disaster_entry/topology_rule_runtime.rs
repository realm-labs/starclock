//! Production bindings for topology, event, domain, and beacon mechanic rules.

use serde::Deserialize;
use starclock_activity::{ActivityProgramDefinition, ActivityRngStreams, NodeId};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
    swarm_disaster_structural::entry_access::SwarmDisasterTopologyInput,
};

use super::{
    SwarmDisasterRuntimeInstance,
    map_overlay::MapRuntimeCatalog,
    topology::{self, CompiledTopology},
};

#[derive(Clone, Debug)]
pub(super) struct TopologyRuleRuntimeCatalog {
    digest: [u8; 32],
}

pub(super) struct PlaneMapContext<'a> {
    pub(super) board: &'a str,
    pub(super) nodes: &'a [NodeId],
    pub(super) terminal: NodeId,
    pub(super) terminal_domain: &'a str,
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
    steps: &'static [(&'static str, &'static str, &'static str)],
}

const CONTRACTS: [RuleContract; 4] = [
    RuleContract {
        family: "beacon-copy-and-blanking",
        trigger: "DiceFaceAccepted",
        slots: &["plane-graph", "beacon-set"],
        steps: &[
            ("ReviewBeaconPlacement", "beacon placement", "FailClosed"),
            ("ReviewCopyTarget", "copy target", "FailClosed"),
            ("ReviewBlanking", "blanking", "FailClosed"),
            (
                "ReviewSimultaneousOrdering",
                "simultaneous ordering",
                "FailClosed",
            ),
        ],
    },
    RuleContract {
        family: "domain-replacement",
        trigger: "DiceFaceAccepted",
        slots: &["plane-graph", "domain-kind"],
        steps: &[
            (
                "ReviewEligibleDomainSet",
                "eligible domain set",
                "FailClosed",
            ),
            ("ReviewReplacementOrder", "replacement order", "FailClosed"),
            (
                "ReviewNoLegalReplacement",
                "no legal replacement",
                "FailClosed",
            ),
        ],
    },
    RuleContract {
        family: "topology-event-order",
        trigger: "TopologyEventMatched",
        slots: &["plane-graph", "topology-event-queue"],
        steps: &[
            ("ReviewTriggerMatching", "trigger matching", "NotApplicable"),
            ("ReviewWeightOrdering", "weight ordering", "NotApplicable"),
            (
                "ReviewReplacementOrShuffleOrder",
                "replacement or shuffle order",
                "NotApplicable",
            ),
        ],
    },
    RuleContract {
        family: "topology-generation",
        trigger: "PlaneGraphRequested",
        slots: &["plane-graph", "current-node"],
        steps: &[
            ("ReviewColumnOrder", "column order", "FailClosed"),
            (
                "ReviewProjectpolicyEdgeDerivation",
                "ProjectPolicy edge derivation",
                "FailClosed",
            ),
            ("ReviewStartEndBehavior", "start/end behavior", "FailClosed"),
            (
                "ReviewUnspecifiedDomainFallback",
                "Unspecified domain fallback",
                "FailClosed",
            ),
        ],
    },
];

impl TopologyRuleRuntimeCatalog {
    pub(super) fn compile(
        mut inputs: [MechanicRuleRuntimeInput; 4],
    ) -> Result<Self, UniverseCatalogLoadError> {
        inputs.sort_unstable_by(|left, right| left.family.cmp(&right.family));
        for (input, contract) in inputs.iter().zip(CONTRACTS.iter()) {
            validate_rule(input, contract)?;
        }
        Ok(Self {
            digest: rule_digest(&inputs),
        })
    }

    pub(super) fn compile_topology(
        &self,
        input: SwarmDisasterTopologyInput,
    ) -> Result<CompiledTopology, UniverseCatalogLoadError> {
        topology::compile(input)
    }

    pub(super) fn compile_creation(
        &self,
        map: &MapRuntimeCatalog,
        context: PlaneMapContext<'_>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        map.compile_creation(
            context.board,
            context.nodes,
            context.terminal,
            context.terminal_domain,
            rng,
        )
    }

    pub(super) fn compile_event_then_creation(
        &self,
        map: &MapRuntimeCatalog,
        context: PlaneMapContext<'_>,
        trigger: &str,
        parameter: u32,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        map.compile_event_then_creation(
            context.board,
            trigger,
            parameter,
            context.nodes,
            context.terminal,
            context.terminal_domain,
            rng,
        )
    }

    pub(super) fn compile_replacement(
        &self,
        map: &MapRuntimeCatalog,
        target: NodeId,
        domain: &str,
        beacon: Option<&str>,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        map.compile_replacement(target, domain, beacon)
    }

    pub(super) fn compile_copy(
        &self,
        map: &MapRuntimeCatalog,
        source: NodeId,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        map.compile_copy(source, target)
    }

    pub(super) fn compile_blank(
        &self,
        map: &MapRuntimeCatalog,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        map.compile_blank(target)
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the four exact Sora topology-rule bindings.
    #[must_use]
    pub fn topology_rule_runtime_digest(&self) -> [u8; 32] {
        self.topology_rules.digest
    }
}

fn validate_rule(
    input: &MechanicRuleRuntimeInput,
    contract: &RuleContract,
) -> Result<(), UniverseCatalogLoadError> {
    let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
        .map_err(|_| reference("invalid topology mechanic-rule slots"))?;
    let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
        .map_err(|_| reference("invalid topology mechanic-rule program"))?;
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
                    && step.unresolved_behavior.as_ref() == expected.2
            });
    if valid {
        Ok(())
    } else {
        Err(reference("topology mechanic-rule contract drift"))
    }
}

fn rule_digest(inputs: &[MechanicRuleRuntimeInput]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.topology-rule-runtime");
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
#[path = "topology_rule_runtime_tests.rs"]
mod tests;
