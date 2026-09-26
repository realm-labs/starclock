//! Mode-owned authored choices at an explicitly selected logical checkpoint.
//! Inventory grant and headless dialogue completion are executable; acquisition
//! effects and original room-selection parity remain incomplete.

use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityExpression,
    ActivityGeneratedBoundaryResolution, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivitySlotId, ActivityStateHash, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError, NodeId,
};
use starclock_data::{
    divergent_universe_decisions::DecisionChoiceId,
    divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId,
};

use super::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    decision_rewards::{DecisionRewardError, DecisionRewardGrant, DecisionRewardRuntime},
    state::{
        OCCURRENCE_REWARD_ACCEPTED_SLOT, ROOM_CONTENT_ENABLED_SLOT, ROOM_CONTENT_UPDATED_SLOT,
        ROOM_DIALOGUE_FINISHED_SLOT, ROOM_DIALOGUE_PROGRAM_SLOT, ROOM_DOORS_OPEN_SLOT,
        ROOM_FINISHED_SLOT, ROOM_PREDICATE_SATISFIED_SLOT,
    },
};
use crate::digest::CanonicalDigestBuilder;

#[derive(Clone, Debug)]
pub(super) struct OccurrenceBinding {
    variant: DivergentUniverseOccurrenceVariantId,
    key: u64,
    choices: Box<[BoundChoice]>,
    rewards: DecisionRewardRuntime,
}

#[derive(Clone, Debug)]
struct BoundChoice {
    id: DecisionChoiceId,
    option: ActivityOptionId,
    enabled: ActivityCondition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OccurrenceBindingError {
    UnknownVariant,
    InvalidCheckpoint,
    Reward(DecisionRewardError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OccurrenceExecutionError {
    NotBound,
    DefinitionMismatch,
    NotOffered,
    RoomStateMismatch,
    Reward(DecisionRewardError),
    Activity(GraphActivityCommandError),
}

impl OccurrenceBinding {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
        variant: &DivergentUniverseOccurrenceVariantId,
    ) -> Result<Self, OccurrenceBindingError> {
        let authored = factory
            .decision_catalog()
            .occurrences()
            .iter()
            .find(|event| &event.variant == variant)
            .ok_or(OccurrenceBindingError::UnknownVariant)?;
        let rewards = factory
            .decision_reward_runtime()
            .map_err(OccurrenceBindingError::Reward)?;
        let choices = authored
            .choices
            .iter()
            .map(|choice| {
                Ok(BoundChoice {
                    id: choice.id.clone(),
                    option: ActivityOptionId::new(u64::from(choice.ordinal))
                        .ok_or(OccurrenceBindingError::InvalidCheckpoint)?,
                    enabled: rewards
                        .availability_condition(&choice.id)
                        .map_err(OccurrenceBindingError::Reward)?,
                })
            })
            .collect::<Result<Vec<_>, OccurrenceBindingError>>()?;
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.divergent-universe.explicit-checkpoint-dialogue.v1");
        hash.update(variant.as_str().as_bytes());
        let digest = hash.finalize();
        let mut bytes = [0_u8; 8];
        bytes.copy_from_slice(&digest[..8]);
        Ok(Self {
            variant: variant.clone(),
            key: u64::from_le_bytes(bytes).max(1),
            choices: choices.into_boxed_slice(),
            rewards,
        })
    }

    pub(super) fn variant(&self) -> &DivergentUniverseOccurrenceVariantId {
        &self.variant
    }

    pub(super) fn wrap_checkpoint(
        &self,
        mut operations: Vec<ActivityOperation>,
    ) -> Result<Vec<ActivityOperation>, OccurrenceBindingError> {
        let route = operations
            .pop()
            .ok_or(OccurrenceBindingError::InvalidCheckpoint)?;
        if !matches!(
            route,
            ActivityOperation::Offer {
                kind: ActivityDecisionKind::Route | ActivityDecisionKind::Encounter,
                ..
            }
        ) {
            return Err(OccurrenceBindingError::InvalidCheckpoint);
        }
        operations.push(set(
            ROOM_DIALOGUE_PROGRAM_SLOT,
            ActivityValue::OptionalId(Some(self.key)),
        ));
        operations.push(set(
            OCCURRENCE_REWARD_ACCEPTED_SLOT,
            ActivityValue::OptionalId(None),
        ));
        for slot in [
            ROOM_DIALOGUE_FINISHED_SLOT,
            ROOM_PREDICATE_SATISFIED_SLOT,
            ROOM_CONTENT_UPDATED_SLOT,
            ROOM_FINISHED_SLOT,
            ROOM_DOORS_OPEN_SLOT,
        ] {
            operations.push(set(slot, ActivityValue::Boolean(false)));
        }
        let options = self
            .choices
            .iter()
            .map(|choice| {
                let mut selected = vec![
                    ActivityOperation::Require(ActivityCondition::All(
                        vec![
                            equals(
                                ROOM_DIALOGUE_PROGRAM_SLOT,
                                ActivityValue::OptionalId(Some(self.key)),
                            ),
                            equals(
                                OCCURRENCE_REWARD_ACCEPTED_SLOT,
                                ActivityValue::OptionalId(Some(choice.option.get())),
                            ),
                            ActivityCondition::Boolean(ActivityExpression::Slot(
                                ROOM_CONTENT_ENABLED_SLOT,
                            )),
                            ActivityCondition::Not(Box::new(ActivityCondition::Boolean(
                                ActivityExpression::Slot(ROOM_FINISHED_SLOT),
                            ))),
                        ]
                        .into_boxed_slice(),
                    )),
                    set(
                        OCCURRENCE_REWARD_ACCEPTED_SLOT,
                        ActivityValue::OptionalId(None),
                    ),
                ];
                // The implemented grant's operations precede these in one transaction.
                // Preserve finish-before-door order, then expose the real route.
                // This does not claim the missing acquisition effects are executed.
                for slot in [
                    ROOM_DIALOGUE_FINISHED_SLOT,
                    ROOM_PREDICATE_SATISFIED_SLOT,
                    ROOM_CONTENT_UPDATED_SLOT,
                    ROOM_FINISHED_SLOT,
                    ROOM_DOORS_OPEN_SLOT,
                ] {
                    selected.push(set(slot, ActivityValue::Boolean(true)));
                }
                selected.push(route.clone());
                ActivityOptionDefinition::new(choice.option, 0, choice.enabled.clone(), selected)
            })
            .collect::<Vec<_>>();
        operations.push(ActivityOperation::Offer {
            kind: ActivityDecisionKind::Choice,
            options: options.into_boxed_slice(),
        });
        Ok(operations)
    }

    fn execute(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<DecisionRewardGrant>, OccurrenceExecutionError>
    {
        self.execute_at_node(
            activity,
            expected,
            decision,
            option,
            NodeId::new(1).expect("initial logical checkpoint"),
        )
    }

    pub(super) fn execute_at_node(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
        node: NodeId,
    ) -> Result<ActivityGeneratedBoundaryResolution<DecisionRewardGrant>, OccurrenceExecutionError>
    {
        if expected != activity.state_hash() {
            return Err(OccurrenceExecutionError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        let view = activity.player_view();
        if view.current_node() != node
            || !view.decision().is_some_and(|offered| {
                offered.id() == decision
                    && offered.kind() == ActivityDecisionKind::Choice
                    && offered
                        .options()
                        .iter()
                        .any(|candidate| candidate.id() == option)
            })
        {
            return Err(OccurrenceExecutionError::NotOffered);
        }
        if !view.slots().iter().any(|slot| {
            slot.id() == ROOM_DIALOGUE_PROGRAM_SLOT
                && slot.value() == &ActivityValue::OptionalId(Some(self.key))
        }) {
            return Err(OccurrenceExecutionError::RoomStateMismatch);
        }
        let selected = self
            .choices
            .iter()
            .find(|choice| choice.option == option)
            .ok_or(OccurrenceExecutionError::NotOffered)?;
        self.rewards
            .check_choice(&view, &selected.id)
            .map_err(OccurrenceExecutionError::Reward)?;
        let mut generation_error = None;
        let result = activity.choose_option_with_generated_prefix(
            expected,
            decision,
            option,
            |view, rng| match self.rewards.generate_choice(view, &selected.id, rng) {
                Ok(program) => {
                    let (mut operations, grant) = program.into_parts();
                    operations.push(set(
                        OCCURRENCE_REWARD_ACCEPTED_SLOT,
                        ActivityValue::OptionalId(Some(option.get())),
                    ));
                    Ok((operations, grant))
                }
                Err(error) => {
                    generation_error = Some(error);
                    Err(GraphActivityCommandError::Runtime(
                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                    ))
                }
            },
        );
        result.map_err(|error| {
            generation_error.map_or(
                OccurrenceExecutionError::Activity(error),
                OccurrenceExecutionError::Reward,
            )
        })
    }
}

impl DivergentUniverseFlowInstance {
    /// Non-mutating current authored event observation. Explicit source-position
    /// placement is not original event-pool membership or a recovered NPC graph.
    #[must_use]
    pub fn offered_occurrence(
        &self,
        activity: &GraphActivity,
    ) -> Option<&DivergentUniverseOccurrenceVariantId> {
        if let Some(rooms) = &self.position_battles {
            return rooms
                .occurrence(activity)
                .and_then(|room| room.offered(activity));
        }
        (activity.definition().identity() == self.definition.identity()
            && activity.current_node() == NodeId::new(1).expect("initial logical checkpoint")
            && activity
                .player_view()
                .decision()
                .is_some_and(|offer| offer.kind() == ActivityDecisionKind::Choice))
        .then(|| self.initial_occurrence())
        .flatten()
    }
    /// Explicitly bound authored variant, not an original room-pool membership claim.
    #[must_use]
    pub fn initial_occurrence(&self) -> Option<&DivergentUniverseOccurrenceVariantId> {
        self.occurrence_binding
            .as_deref()
            .map(OccurrenceBinding::variant)
    }

    /// Selects an offered occurrence through the shared reward/option transaction.
    /// Direct GraphActivity option selection cannot bypass its accepted-grant gate.
    /// Stale, mismatched, unavailable and repeated choices leave state/RNG intact.
    pub fn choose_occurrence_option(
        &self,
        factory: &DivergentUniverseRuntimeFactory,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<DecisionRewardGrant>, OccurrenceExecutionError>
    {
        if let Some(rooms) = &self.position_battles {
            let room = rooms
                .occurrence(activity)
                .ok_or(OccurrenceExecutionError::NotOffered)?;
            if !room.matches_factory(factory) {
                return Err(OccurrenceExecutionError::DefinitionMismatch);
            }
            return room.choose(activity, expected, decision, option);
        }
        let binding = self
            .occurrence_binding
            .as_deref()
            .ok_or(OccurrenceExecutionError::NotBound)?;
        if activity.definition().identity() != self.definition.identity()
            || binding.rewards.configuration_digest() != factory.decision_catalog().digest()
        {
            return Err(OccurrenceExecutionError::DefinitionMismatch);
        }
        binding.execute(activity, expected, decision, option)
    }
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
