//! Public evolution choices use the shared authenticated generated prefix.

use super::{BoundEvent, EvolutionEvents, invalid, option_reward, set};
use crate::divergent_universe::{
    DivergentUniverseFlowInstance,
    curio_runtime::DivergentUniverseCurioLifecycleState,
    economy::DivergentUniverseCurrencyKind,
    state::{OCCURRENCE_REWARD_ACCEPTED_SLOT, ROOM_DIALOGUE_PROGRAM_SLOT},
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityOperation, ActivityOptionId,
    ActivityPlayerView, ActivityRngLabel, ActivityRngStreams, ActivityStateHash, ActivityValue,
    GraphActivity, GraphActivityCommandError,
};
use starclock_data::divergent_universe_decisions::{
    EvolutionEventDefinition, EvolutionOptionDefinition, EvolutionOptionEffect,
};

impl DivergentUniverseFlowInstance {
    /// Current visible policy-bound event, including bilingual option labels.
    /// Returning a locator does not establish original random-room membership.
    #[must_use]
    pub fn offered_evolution_event(
        &self,
        activity: &GraphActivity,
    ) -> Option<&EvolutionEventDefinition> {
        self.evolution_binding(activity)
            .map(|binding| &binding.definition)
    }

    /// Authenticates the exact visible choice before costs or RNG. Effects,
    /// sacrifice/evolution, immediate and event grants, dialogue completion and
    /// encounter selection commit atomically. Raw graph selection cannot bypass
    /// the accepted-effect gate. Rejections leave state and RNG byte-identical.
    pub fn choose_evolution_option(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let binding = self
            .evolution_binding(activity)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        let runtime = self.evolution_events.as_ref().ok_or_else(invalid)?;
        activity.choose_option_with_generated_prefix(
            expected,
            decision,
            selected,
            |view, rng| {
                let (option, _) = binding
                    .options
                    .iter()
                    .find(|(option, _)| u64::from(option.ordinal) == selected.get())
                    .ok_or_else(invalid)?;
                let mut operations = runtime.generate(self, binding, option, view, rng)?;
                operations.push(set(
                    OCCURRENCE_REWARD_ACCEPTED_SLOT,
                    ActivityValue::OptionalId(Some(selected.get())),
                ));
                Ok((operations, ()))
            },
        )?;
        Ok(())
    }

    fn evolution_binding(&self, activity: &GraphActivity) -> Option<&BoundEvent> {
        if activity.definition().identity() != self.definition.identity()
            || activity.definition().graph().digest() != self.definition.graph().digest()
        {
            return None;
        }
        let view = activity.player_view();
        if view.decision()?.kind() != ActivityDecisionKind::Service {
            return None;
        }
        self.evolution_events
            .as_ref()?
            .bindings
            .iter()
            .find(|binding| {
                binding.node == view.current_node()
                    && view.slots().iter().any(|slot| {
                        slot.id() == ROOM_DIALOGUE_PROGRAM_SLOT
                            && slot.value() == &ActivityValue::OptionalId(Some(binding.key))
                    })
            })
    }
}

impl EvolutionEvents {
    fn generate(
        &self,
        flow: &DivergentUniverseFlowInstance,
        binding: &BoundEvent,
        option: &EvolutionOptionDefinition,
        view: &ActivityPlayerView,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let edge = &binding.definition.evolution;
        let owned = self.curios.owned_from_view(view).map_err(|_| invalid())?;
        if !owned.iter().any(|owned| {
            owned.state() == &edge.from
                && owned.lifecycle() == DivergentUniverseCurioLifecycleState::Active
        }) || self.rewards.fragment_balance(view).map_err(|_| invalid())? < option.fragment_cost
        {
            return Err(invalid());
        }
        if matches!(
            option.effect,
            EvolutionOptionEffect::Evolve | EvolutionOptionEffect::Chance { .. }
        ) && !self
            .curios
            .acquisition_rewards_available(view, &edge.to)
            .map_err(|_| invalid())?
        {
            return Err(invalid());
        }
        let fragments = flow
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment);
        let mut operations = Vec::new();
        if option.fragment_cost != 0 {
            operations.push(
                fragments
                    .spend_operation(option.fragment_cost)
                    .map_err(|_| invalid())?,
            );
        }
        let evolves = match option.effect {
            EvolutionOptionEffect::Evolve => true,
            EvolutionOptionEffect::Chance {
                numerator,
                denominator,
            } => {
                let draw = rng
                    .choose_index(ActivityRngLabel::Reward, 24_001, denominator)
                    .map_err(|_| invalid())?
                    .ok_or_else(invalid)?;
                draw.value() < u64::from(numerator)
            }
            EvolutionOptionEffect::Curios { .. } | EvolutionOptionEffect::Sacrifice { .. } => {
                let sacrificed = matches!(option.effect, EvolutionOptionEffect::Sacrifice { .. })
                    .then_some(&edge.from);
                operations.extend(
                    self.rewards
                        .curio_reward_operations(view, option_reward(option), sacrificed, rng)
                        .map_err(|_| invalid())?,
                );
                false
            }
        };
        if evolves {
            operations.extend(
                self.curios
                    .evolution_operations(view, &edge.from, &edge.to, rng)
                    .map_err(|_| invalid())?,
            );
        }
        if option.fragment_grant != 0 {
            operations.extend(
                fragments
                    .credit_operations(option.fragment_grant)
                    .map_err(|_| invalid())?,
            );
        }
        Ok(operations)
    }
}
