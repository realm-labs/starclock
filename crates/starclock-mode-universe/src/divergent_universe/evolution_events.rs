//! Conditional, policy-authored evolution events at existing layer entry nodes.

#[path = "evolution_event_execution.rs"]
mod execution;

use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseRuntimeFactory,
    battle_route::layer_entry_node,
    curio_runtime::DivergentUniverseCurioRuntime,
    decision_rewards::DecisionRewardRuntime,
    state::{
        CURIO_STATES_SLOT, OCCURRENCE_REWARD_ACCEPTED_SLOT, ROOM_CONTENT_UPDATED_SLOT,
        ROOM_DIALOGUE_FINISHED_SLOT, ROOM_DIALOGUE_PROGRAM_SLOT, ROOM_DOORS_OPEN_SLOT,
        ROOM_FINISHED_SLOT, ROOM_PREDICATE_SATISFIED_SLOT,
    },
};
use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityExpression, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivitySlotId,
    ActivityValue, GraphActivityCommandError, GraphActivityNodeProgram, GraphActivityRuntimeError,
    NodeId,
};
use starclock_data::divergent_universe_decisions::{
    DecisionReward, EvolutionEventDefinition, EvolutionEventPolicy, EvolutionOptionDefinition,
    EvolutionOptionEffect,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub(super) struct EvolutionEvents {
    bindings: Box<[BoundEvent]>,
    curios: DivergentUniverseCurioRuntime,
    rewards: DecisionRewardRuntime,
}

#[derive(Clone, Debug)]
struct BoundEvent {
    node: NodeId,
    key: u64,
    definition: EvolutionEventDefinition,
    options: Box<[(EvolutionOptionDefinition, ActivityCondition)]>,
}

impl EvolutionEvents {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
        layer_count: usize,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let invalid = || DivergentUniverseEntryFlowError::InvalidActivityDefinition;
        let curios = factory.curio_runtime().map_err(|_| invalid())?;
        let rewards = factory.decision_reward_runtime().map_err(|_| invalid())?;
        let count = u32::try_from(layer_count).map_err(|_| invalid())?;
        let mut bindings = Vec::new();
        let mut keys = BTreeSet::new();
        for definition in factory.decision_catalog().evolution_events() {
            if u32::from(definition.layer_ordinal) > count {
                continue;
            }
            match definition.policy { EvolutionEventPolicy::VersionedProjectPolicyActiveTreasuresAtLayerEntryStableSequence => {} }
            let state = curios
                .states()
                .iter()
                .find(|state| state.id() == &definition.evolution.from)
                .ok_or_else(invalid)?;
            let active = ActivityCondition::Equal(
                ActivityExpression::CounterValue {
                    slot: CURIO_STATES_SLOT,
                    key: state.state_key(),
                },
                ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
            );
            let options = definition
                .options
                .iter()
                .map(|option| {
                    let condition = rewards
                        .reward_availability_condition(option.fragment_cost, option_reward(option))
                        .map_err(|_| invalid())?;
                    let mut conditions = vec![active.clone(), condition];
                    if matches!(
                        option.effect,
                        EvolutionOptionEffect::Evolve | EvolutionOptionEffect::Chance { .. }
                    ) && let Some(condition) = rewards
                        .acquisition_availability_condition(&definition.evolution.to)
                        .map_err(|_| invalid())?
                    {
                        conditions.push(condition);
                    }
                    Ok((
                        option.clone(),
                        ActivityCondition::All(conditions.into_boxed_slice()),
                    ))
                })
                .collect::<Result<Vec<_>, DivergentUniverseEntryFlowError>>()?;
            let mut hash = CanonicalDigestBuilder::new();
            hash.update(b"starclock.divergent-universe.evolution-dialogue");
            hash.update(definition.key.as_bytes());
            let digest = hash.finalize();
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&digest[..8]);
            let key = u64::from_le_bytes(bytes).max(1);
            if !keys.insert(key) {
                return Err(invalid());
            }
            bindings.push(BoundEvent {
                node: layer_entry_node(count, u32::from(definition.layer_ordinal))?,
                key,
                definition: definition.clone(),
                options: options.into_boxed_slice(),
            });
        }
        Ok(Self {
            bindings: bindings.into_boxed_slice(),
            curios,
            rewards,
        })
    }

    pub(super) fn wrap_programs(
        &self,
        programs: &mut [GraphActivityNodeProgram],
    ) -> Result<(), DivergentUniverseEntryFlowError> {
        let mut groups = BTreeMap::<NodeId, Vec<&BoundEvent>>::new();
        for binding in &self.bindings {
            groups.entry(binding.node).or_default().push(binding);
        }
        for (node, mut bindings) in groups {
            bindings.sort_by(|left, right| left.definition.key.cmp(&right.definition.key));
            let program = programs
                .iter_mut()
                .find(|program| program.node() == node)
                .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
            let mut operations = program.program().operations().to_vec();
            let next = operations
                .pop()
                .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
            if !matches!(next, ActivityOperation::Traverse(_)) {
                return Err(DivergentUniverseEntryFlowError::InvalidActivityDefinition);
            }
            // Compile the bounded suffix into each accepted option. The shared
            // interpreter evaluates the next event against the just-committed
            // effects; there is no mode-owned event queue or replay state machine.
            let mut continuation = vec![next];
            for binding in bindings.into_iter().rev() {
                continuation = event_program(binding, &continuation);
            }
            operations.extend(continuation);
            *program = GraphActivityNodeProgram::new(
                node,
                ActivityProgramDefinition::new(program.program().id(), operations)
                    .map_err(DivergentUniverseEntryFlowError::EvolutionProgram)?,
            );
        }
        Ok(())
    }
}

fn event_program(
    binding: &BoundEvent,
    continuation: &[ActivityOperation],
) -> Vec<ActivityOperation> {
    let options = binding
        .options
        .iter()
        .map(|(option, enabled)| {
            let mut selected = vec![
                ActivityOperation::Require(ActivityCondition::All(
                    vec![
                        equals(
                            OCCURRENCE_REWARD_ACCEPTED_SLOT,
                            ActivityValue::OptionalId(Some(u64::from(option.ordinal))),
                        ),
                        equals(
                            ROOM_DIALOGUE_PROGRAM_SLOT,
                            ActivityValue::OptionalId(Some(binding.key)),
                        ),
                    ]
                    .into_boxed_slice(),
                )),
                set(
                    OCCURRENCE_REWARD_ACCEPTED_SLOT,
                    ActivityValue::OptionalId(None),
                ),
            ];
            selected.extend(room_flags(true));
            selected.extend_from_slice(continuation);
            ActivityOptionDefinition::new(option_id(option.ordinal), 0, enabled.clone(), selected)
        })
        .collect::<Vec<_>>();
    let mut offered = vec![
        set(
            ROOM_DIALOGUE_PROGRAM_SLOT,
            ActivityValue::OptionalId(Some(binding.key)),
        ),
        set(
            OCCURRENCE_REWARD_ACCEPTED_SLOT,
            ActivityValue::OptionalId(None),
        ),
    ];
    offered.extend(room_flags(false));
    offered.push(ActivityOperation::Offer {
        kind: ActivityDecisionKind::Service,
        options: options.into_boxed_slice(),
    });
    vec![ActivityOperation::Conditional {
        condition: ActivityCondition::Any(
            binding
                .options
                .iter()
                .map(|(_, condition)| condition.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        if_true: offered.into_boxed_slice(),
        if_false: continuation.to_vec().into_boxed_slice(),
    }]
}

fn option_reward(option: &EvolutionOptionDefinition) -> DecisionReward {
    match option.effect {
        EvolutionOptionEffect::Curios { count, rarity }
        | EvolutionOptionEffect::Sacrifice { count, rarity } => {
            DecisionReward::Curios { count, rarity }
        }
        EvolutionOptionEffect::Evolve | EvolutionOptionEffect::Chance { .. } => {
            DecisionReward::Fragments(0)
        }
    }
}
fn room_flags(value: bool) -> Vec<ActivityOperation> {
    [
        ROOM_DIALOGUE_FINISHED_SLOT,
        ROOM_PREDICATE_SATISFIED_SLOT,
        ROOM_CONTENT_UPDATED_SLOT,
        ROOM_FINISHED_SLOT,
        ROOM_DOORS_OPEN_SLOT,
    ]
    .into_iter()
    .map(|slot| set(slot, ActivityValue::Boolean(value)))
    .collect()
}
fn set(slot: ActivitySlotId, value: ActivityValue) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(value),
    }
}
fn equals(slot: ActivitySlotId, value: ActivityValue) -> ActivityCondition {
    ActivityCondition::Equal(
        ActivityExpression::Slot(slot),
        ActivityExpression::Literal(value),
    )
}
fn option_id(ordinal: u16) -> ActivityOptionId {
    ActivityOptionId::new(u64::from(ordinal)).expect("validated positive evolution option ordinal")
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
