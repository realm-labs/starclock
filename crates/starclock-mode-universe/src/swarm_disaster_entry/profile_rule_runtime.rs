//! Production execution binding for the Profile-entry mechanic rule.

use serde::Deserialize;
use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::{
    SwarmDisasterRuntimeInstance,
    path_runtime::PathRuntimeCatalog,
    state::{DEFERRED, ENTRY},
};

const PROFILE_RULE_PROGRAM_BASE: u32 = 0x5352_0000;
const PROFILE_RULE_MARKER_BASE: u64 = 0x5344_7300_0000_0000;

#[derive(Clone, Debug)]
pub(super) struct ProfileRuleRuntimeCatalog {
    id: u32,
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

impl ProfileRuleRuntimeCatalog {
    pub(super) fn compile(
        input: MechanicRuleRuntimeInput,
        paths: &PathRuntimeCatalog,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let slots: Vec<RuleSlot> = serde_json::from_str(&input.slots)
            .map_err(|_| reference("invalid Profile-entry rule slots"))?;
        let steps: Vec<RuleStep> = serde_json::from_str(&input.program)
            .map_err(|_| reference("invalid Profile-entry rule program"))?;
        let expected_bonus_keys = [
            "swarm-disaster.trailblaze-bonus.101",
            "swarm-disaster.trailblaze-bonus.102",
            "swarm-disaster.trailblaze-bonus.103",
            "swarm-disaster.trailblaze-bonus.104",
            "swarm-disaster.trailblaze-bonus.105",
            "swarm-disaster.trailblaze-bonus.106",
        ];
        if input.key.as_ref() != "swarm-disaster.mechanic-rule.profile-entry"
            || input.family.as_ref() != "profile-entry"
            || input.domain.as_ref() != "Activity"
            || input
                .triggers
                .iter()
                .map(Box::as_ref)
                .ne(["RunEntryRequested"])
            || input
                .fixtures
                .iter()
                .map(Box::as_ref)
                .ne(["swarm-disaster.fixture.profile-entry"])
            || input.source_disposition.as_ref() != "ReferenceOnly"
            || paths.profile_bonus_keys().ne(expected_bonus_keys)
            || !valid_slots(&slots)
            || !valid_steps(&steps)
        {
            return Err(reference("Profile-entry mechanic rule contract drift"));
        }
        let digest = rule_digest(&input);
        Ok(Self {
            id: input.id,
            digest,
        })
    }

    fn compile_program(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        if state.current_node() != instance.graph_definition().entry() {
            return Err(invalid("Profile-entry rule must execute at the entry node"));
        }
        if !(1..=5).contains(&instance.difficulty) {
            return Err(invalid("Profile-entry difficulty escaped Formal bounds"));
        }
        let expected_entry = initial_entry(instance)?;
        if !matches!(
            state.slot(slot(ENTRY)),
            Some(ActivityValue::BoundedCounterMap(values)) if values.as_ref() == expected_entry
        ) {
            return Err(invalid(
                "Profile-entry state does not match compiled eligibility",
            ));
        }
        let marker = marker(self.id)?;
        if counter(state, DEFERRED, marker)? != 0 {
            return Err(invalid("Profile-entry rule already executed"));
        }
        let mut operations = vec![require_counter(DEFERRED, marker, 0)];
        operations.extend(
            expected_entry
                .iter()
                .map(|(key, value)| require_counter(ENTRY, *key, *value)),
        );
        if let Some(bonus) = instance.compile_trailblaze_bonus_run_start(state)? {
            operations.extend(bonus.operations().iter().cloned());
        }
        operations.push(add_counter(DEFERRED, marker, 1));
        let program_id = PROFILE_RULE_PROGRAM_BASE
            .checked_add(self.id)
            .ok_or_else(|| invalid("Profile-entry program ID overflow"))?;
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(program_id)
                .ok_or_else(|| invalid("zero Profile-entry program ID"))?,
            operations,
        )
        .map_err(|_| invalid("invalid Profile-entry Activity program"))?;
        program
            .validate_against(instance.state_definition(), instance.graph_definition())
            .map_err(|_| invalid("Profile-entry program failed Activity validation"))?;
        Ok(program)
    }

    const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Deterministic digest of the exact Sora Profile-entry rule binding.
    #[must_use]
    pub fn profile_rule_runtime_digest(&self) -> [u8; 32] {
        self.profile_rule.digest()
    }

    /// Compiles the exact Profile-entry rule and selected run-start bonus into
    /// one guarded Activity program. An unaffordable bonus fails before any
    /// mutation. The caller applies the returned program in the same accepted
    /// entry command; reapplication after its once marker commits is rejected.
    pub fn compile_profile_entry_rule(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.profile_rule.compile_program(self, state)
    }
}

fn valid_slots(slots: &[RuleSlot]) -> bool {
    slots.len() == 3
        && slots.iter().all(|slot| slot.owner.as_ref() == "Activity")
        && slots
            .iter()
            .map(|slot| slot.id.as_ref())
            .eq(["profile", "difficulty", "entry-bonus"])
}

fn valid_steps(steps: &[RuleStep]) -> bool {
    const EXPECTED: [(&str, &str); 3] = [
        ("ReviewEntryEligibility", "entry eligibility"),
        ("ReviewFiveFormalDifficulties", "five Formal difficulties"),
        ("ReviewBonus101106Ownership", "bonus 101-106 ownership"),
    ];
    steps.len() == 3
        && steps
            .iter()
            .zip(EXPECTED)
            .enumerate()
            .all(|(index, (step, (operation, fact)))| {
                usize::from(step.sequence) == index + 1
                    && step.operation.as_ref() == operation
                    && step.source_fact.as_ref() == fact
                    && step.unresolved_behavior.as_ref() == "FailClosed"
            })
}

fn initial_entry(
    instance: &SwarmDisasterRuntimeInstance,
) -> Result<&[(u64, i64)], UniverseCatalogLoadError> {
    let definition = instance
        .state_definition()
        .slots()
        .iter()
        .find(|definition| definition.id() == slot(ENTRY))
        .ok_or_else(|| invalid("Profile-entry slot is missing"))?;
    match definition.initial() {
        ActivityValue::BoundedCounterMap(values)
            if values.len() == 4 && values.iter().map(|(key, _)| *key).eq([1, 2, 3, 4]) =>
        {
            Ok(values)
        }
        _ => Err(invalid("Profile-entry slot contract drift")),
    }
}

fn marker(id: u32) -> Result<u64, UniverseCatalogLoadError> {
    PROFILE_RULE_MARKER_BASE
        .checked_add(u64::from(id))
        .ok_or_else(|| invalid("Profile-entry marker overflow"))
}

fn counter(
    state: &ActivityTransactionState,
    slot_id: u32,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    match state.slot(slot(slot_id)) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(invalid("Profile-entry counter slot drift")),
    }
}

fn require_counter(slot_id: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Equal(
        counter_expression(slot_id, key),
        integer(value),
    ))
}

fn add_counter(slot_id: u32, key: u64, value: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot: slot(slot_id),
        key,
        delta: integer(value),
    }
}

fn counter_expression(slot_id: u32, key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: slot(slot_id),
        key,
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn slot(id: u32) -> ActivitySlotId {
    ActivitySlotId::new(id).expect("static Profile-entry slot ID is non-zero")
}

fn rule_digest(input: &MechanicRuleRuntimeInput) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.profile-entry-rule-runtime");
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

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}

#[cfg(test)]
#[path = "profile_rule_runtime_tests.rs"]
mod tests;
