use super::*;
use crate::curio_activity::CAVITY_CRITICAL_STACK_KEY;
use crate::curio_activity::CurioActivityBindings;
use crate::curio_activity::DESTRUCTIBLE_DESTROYED_COUNT_KEY;
use crate::curio_activity::ROBE_FRAGMENT_SNAPSHOT_KEY;
use crate::curio_activity::cavity_critical_stacks;
use crate::curio_activity::destroyed_curio_count;
use crate::curio_activity::destructible_destroyed_count;
use crate::curio_activity::negative::FISSION_EXTRA_COPY_KEY;
use crate::curio_activity::negative::fission_extra_copies;

impl StandardUniverseActivity {
    pub fn curio_contributions(&self) -> Result<CurioContributionSet, CurioRuntimeError> {
        let view = self.graph.player_view();
        let inventory = view
            .inventories()
            .iter()
            .find(|inventory| inventory.id() == self.curio_inventory)
            .ok_or(CurioRuntimeError::MissingInventory)?;
        let state = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.curio_state_slot)
            .ok_or(CurioRuntimeError::InvalidStateSlot)?;
        let charges = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.curio_charge_slot)
            .ok_or(CurioRuntimeError::InvalidChargeSlot)?;
        let debug = self.graph.debug_view();
        let event_value = debug
            .all_slots()
            .iter()
            .find(|slot| slot.id() == self.curio_event_slot)
            .map(|slot| slot.value());
        let destroyed = event_value.and_then(destroyed_curio_count).unwrap_or(0);
        let cavity_stacks = event_value.and_then(cavity_critical_stacks).unwrap_or(0);
        let destructibles = event_value
            .and_then(destructible_destroyed_count)
            .unwrap_or(0);
        let fission_extra_copies = event_value.and_then(fission_extra_copies).unwrap_or(0);
        let fragments = view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.cosmic_fragments_slot)
            .and_then(|slot| match slot.value() {
                ActivityValue::BoundedInteger(value) => Some(*value),
                _ => None,
            })
            .ok_or(CurioRuntimeError::InvalidStateSlot)?;
        self.curio_runtime
            .contributions(inventory, state, charges)
            .map(|contributions| {
                let mut contributions = contributions.with_destroyed_curios(destroyed);
                if cavity_stacks != 0 {
                    contributions =
                        contributions.with_runtime_value(CAVITY_CRITICAL_STACK_KEY, cavity_stacks);
                }
                if fragments != 0
                    && contributions
                        .entries()
                        .iter()
                        .any(|entry| entry.state().source_effect_id() == "14")
                {
                    contributions =
                        contributions.with_runtime_value(ROBE_FRAGMENT_SNAPSHOT_KEY, fragments);
                }
                if destructibles != 0
                    && contributions
                        .entries()
                        .iter()
                        .any(|entry| entry.state().source_effect_id() == "58")
                {
                    contributions = contributions.with_runtime_value(
                        DESTRUCTIBLE_DESTROYED_COUNT_KEY,
                        i64::from(destructibles),
                    );
                }
                if fission_extra_copies != 0 {
                    contributions = contributions.with_runtime_value(
                        FISSION_EXTRA_COPY_KEY,
                        i64::from(fission_extra_copies),
                    );
                }
                contributions
            })
    }

    pub fn curio_effects(
        &self,
        event: CurioEvent,
        facts: CurioEffectFacts,
    ) -> Result<Box<[AppliedCurioEffect]>, StandardUniverseCurioEffectError> {
        let contributions = self
            .curio_contributions()
            .map_err(StandardUniverseCurioEffectError::Contribution)?;
        let mut effects = Vec::new();
        for contribution in contributions.entries() {
            if !self
                .curio_effect_runtime
                .curio_ids()
                .any(|candidate| candidate == contribution.curio())
            {
                continue;
            }
            effects.extend(
                self.curio_effect_runtime
                    .execute(contribution.curio(), event, facts)
                    .map_err(StandardUniverseCurioEffectError::Effect)?,
            );
        }
        Ok(effects.into_boxed_slice())
    }

    pub fn curio_activity_projection(
        &self,
        curio: CurioId,
        event: CurioEvent,
        mut facts: CurioEffectFacts,
    ) -> Result<CurioActivityProjection, StandardUniverseCurioActivityError> {
        if !self
            .curio_contributions()
            .map_err(StandardUniverseCurioActivityError::Contribution)?
            .entries()
            .iter()
            .any(|contribution| contribution.curio() == curio)
        {
            return Err(StandardUniverseCurioActivityError::NotOwned);
        }
        let fragments = self
            .cosmic_fragments()
            .map_err(StandardUniverseCurioActivityError::Fragments)?;
        facts.cosmic_fragments = u32::try_from(fragments.get()).map_err(|_| {
            StandardUniverseCurioActivityError::Fragments(RunRuntimeError::InvalidFragmentAmount)
        })?;
        let effects = self
            .curio_effect_runtime
            .execute(curio, event, facts)
            .map_err(StandardUniverseCurioActivityError::Effect)?;
        lower_curio_effects(
            curio,
            event,
            &effects,
            facts.cosmic_fragments,
            CurioActivityBindings {
                inventory: self.curio_inventory,
                state_slot: self.curio_state_slot,
                charge_slot: self.curio_charge_slot,
                event_slot: self.curio_event_slot,
                fragments_slot: self.cosmic_fragments_slot,
            },
        )
        .map_err(StandardUniverseCurioActivityError::Projection)
    }

    pub fn negative_curio_effects(
        &self,
        event: NegativeCurioEvent,
    ) -> Result<Box<[AppliedCurioEffect]>, StandardUniverseCurioEffectError> {
        let contributions = self
            .curio_contributions()
            .map_err(StandardUniverseCurioEffectError::Contribution)?;
        let mut effects = Vec::new();
        for contribution in contributions.entries() {
            if !self
                .negative_curio_runtime
                .contains_curio(contribution.curio())
            {
                continue;
            }
            effects.extend(
                self.negative_curio_runtime
                    .execute(contribution, event)
                    .map_err(StandardUniverseCurioEffectError::NegativeEffect)?,
            );
        }
        Ok(effects.into_boxed_slice())
    }
}
