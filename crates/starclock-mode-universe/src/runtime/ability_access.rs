//! Ability Tree projections and Activity-owned reward controls.

use starclock_activity::{ActivityStateHash, ActivityValue};
use starclock_combat::Ratio;

use crate::ability_runtime::{
    AbilityActivityProjection, AbilityExecutionContext, AbilityRuntimeError,
    AbilityRuntimeProjection, AbilityTarget,
};

use super::StandardUniverseActivity;

/// Run-level capabilities materialized from the selected Ability Tree.
///
/// This is the engine-facing seam for systems such as consumable inventory:
/// the Standard Universe runtime authorizes use, while account inventory and
/// item effects remain owned by an external adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardUniverseRunCapabilities {
    formation_slots: u8,
    reviver: bool,
    reviver_restored_hp_ratio: Ratio,
    consumable_use: bool,
}

impl StandardUniverseRunCapabilities {
    #[must_use]
    pub const fn formation_slots(self) -> u8 {
        self.formation_slots
    }

    #[must_use]
    pub const fn reviver(self) -> bool {
        self.reviver
    }

    #[must_use]
    pub const fn reviver_restored_hp_ratio(self) -> Ratio {
        self.reviver_restored_hp_ratio
    }

    #[must_use]
    pub const fn consumable_use(self) -> bool {
        self.consumable_use
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseRunCapabilityError {
    MissingAbilityProjection,
    MissingFormationCapability,
    InvalidBoolean(AbilityTarget, i64),
    InvalidFormationSlots(i64),
    InvalidRestoredHpRatio(i64),
}

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

    pub fn run_capabilities(
        &self,
    ) -> Result<StandardUniverseRunCapabilities, StandardUniverseRunCapabilityError> {
        let view = self.graph.debug_view();
        let projection = view
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.ability_projection_slot)
            .map(|slot| slot.value())
            .ok_or(StandardUniverseRunCapabilityError::MissingAbilityProjection)?;
        let formation_slots = view
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.formation_capability_slot)
            .map(|slot| slot.value())
            .ok_or(StandardUniverseRunCapabilityError::MissingFormationCapability)?;
        capabilities_from_values(projection, formation_slots)
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

pub(crate) fn capabilities_from_values(
    projection: &ActivityValue,
    formation_slots: &ActivityValue,
) -> Result<StandardUniverseRunCapabilities, StandardUniverseRunCapabilityError> {
    let ActivityValue::BoundedCounterMap(projection) = projection else {
        return Err(StandardUniverseRunCapabilityError::MissingAbilityProjection);
    };
    let ActivityValue::BoundedInteger(formation_slots) = formation_slots else {
        return Err(StandardUniverseRunCapabilityError::MissingFormationCapability);
    };
    let formation_slots = u8::try_from(*formation_slots)
        .ok()
        .filter(|value| *value <= 3)
        .ok_or(StandardUniverseRunCapabilityError::InvalidFormationSlots(
            *formation_slots,
        ))?;
    let reviver = boolean_capability(projection, AbilityTarget::ServiceReviver)?;
    let consumable_use = boolean_capability(projection, AbilityTarget::RunConsumableUse)?;
    let reviver_restored_hp_ratio =
        projection_value(projection, AbilityTarget::ServiceReviverRestoredHpRatio);
    if !(0..=1_000_000).contains(&reviver_restored_hp_ratio) {
        return Err(StandardUniverseRunCapabilityError::InvalidRestoredHpRatio(
            reviver_restored_hp_ratio,
        ));
    }
    Ok(StandardUniverseRunCapabilities {
        formation_slots,
        reviver,
        reviver_restored_hp_ratio: Ratio::from_scaled(reviver_restored_hp_ratio),
        consumable_use,
    })
}

fn boolean_capability(
    projection: &[(u64, i64)],
    target: AbilityTarget,
) -> Result<bool, StandardUniverseRunCapabilityError> {
    match projection_value(projection, target) {
        0 => Ok(false),
        1_000_000 => Ok(true),
        value => Err(StandardUniverseRunCapabilityError::InvalidBoolean(
            target, value,
        )),
    }
}

fn projection_value(projection: &[(u64, i64)], target: AbilityTarget) -> i64 {
    projection
        .binary_search_by_key(&target.activity_key(), |entry| entry.0)
        .ok()
        .map_or(0, |index| projection[index].1)
}
