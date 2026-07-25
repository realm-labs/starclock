//! Ability Tree projections and Activity-owned reward controls.

use starclock_activity::{ActivityStateHash, ActivityValue};

use crate::ability_runtime::{
    AbilityActivityProjection, AbilityExecutionContext, AbilityRuntimeError,
    AbilityRuntimeProjection, AbilityTarget,
};

use super::StandardUniverseActivity;

impl StandardUniverseActivity {
    pub fn ability_tree_projection(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<AbilityRuntimeProjection, AbilityRuntimeError> {
        self.ability_runtime.project(&self.ability_tree, context)
    }

    pub fn ability_activity_projection(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<AbilityActivityProjection, AbilityRuntimeError> {
        self.ability_runtime.project_activity_operations(
            &self.ability_tree,
            context,
            self.ability_projection_slot,
        )
    }

    pub fn ability_activity_delta_projection(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<AbilityActivityProjection, AbilityRuntimeError> {
        let view = self.graph.debug_view();
        let current = view
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.ability_projection_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedCounterMap(entries) => Some(entries.as_ref()),
                _ => None,
            })
            .ok_or(AbilityRuntimeError::NonCanonicalActivityState)?;
        self.ability_runtime.project_activity_delta_operations(
            &self.ability_tree,
            context,
            self.ability_projection_slot,
            current,
        )
    }

    pub fn reroll_blessing_offer(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<
        Box<[starclock_activity::ActivityTransactionEvent]>,
        starclock_activity::GraphActivityRandomOfferError,
    > {
        let unlocked = self
            .graph
            .debug_view()
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.ability_projection_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedCounterMap(entries) => entries
                    .binary_search_by_key(
                        &AbilityTarget::BlessingChoiceResetCount.activity_key(),
                        |entry| entry.0,
                    )
                    .ok()
                    .map(|index| entries[index].1 > 0),
                _ => None,
            })
            .unwrap_or(false);
        if !unlocked {
            return Err(starclock_activity::GraphActivityRandomOfferError::RerollDisabled);
        }
        self.graph.reroll_random_offer(expected_state_hash)
    }
}
