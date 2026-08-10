use crate::{
    TimelineActorId, UnitId,
    battle::{fault::BattleFault, state::BattleClockState},
    numeric::domain::ActionGauge,
};

use super::{
    journal::MutationField,
    transaction::{Transaction, action_fault},
};

impl Transaction<'_> {
    pub(super) fn add_timeline_elapsed(&mut self, delta: i64) -> Result<(), BattleFault> {
        self.timeline_elapsed_scaled = self
            .timeline_elapsed_scaled
            .checked_add(delta)
            .ok_or_else(|| action_fault(101))?;
        Ok(())
    }

    pub(super) fn set_cycle_clock(
        &mut self,
        remaining: u16,
        cycle_index: u32,
        elapsed_scaled: i64,
    ) -> Result<(), BattleFault> {
        let Some(BattleClockState::Cycles {
            remaining_cycles,
            cycle_index: current_index,
            elapsed_in_window_scaled,
            ..
        }) = self.state.clock.as_mut()
        else {
            return Err(action_fault(109));
        };
        if *remaining_cycles != remaining {
            self.journal.mutation(
                MutationField::BattleClock,
                u64::from(*remaining_cycles),
                u64::from(remaining),
            );
            *remaining_cycles = remaining;
        }
        if *current_index != cycle_index {
            self.journal.mutation(
                MutationField::BattleClock,
                u64::from(*current_index),
                u64::from(cycle_index),
            );
            *current_index = cycle_index;
        }
        if *elapsed_in_window_scaled != elapsed_scaled {
            self.journal.mutation(
                MutationField::BattleClock,
                *elapsed_in_window_scaled as u64,
                elapsed_scaled as u64,
            );
            *elapsed_in_window_scaled = elapsed_scaled;
        }
        Ok(())
    }

    pub(super) fn set_action_value_clock(
        &mut self,
        remaining_scaled: i64,
    ) -> Result<(), BattleFault> {
        let Some(BattleClockState::ActionValue {
            remaining_scaled: current,
            ..
        }) = self.state.clock.as_mut()
        else {
            return Err(action_fault(110));
        };
        if *current != remaining_scaled {
            self.journal.mutation(
                MutationField::BattleClock,
                *current as u64,
                remaining_scaled as u64,
            );
            *current = remaining_scaled;
        }
        Ok(())
    }

    pub(super) fn delay_unit(&mut self, owner: UnitId, scaled: i64) -> Result<(), BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(43))?;
        let before = self
            .state
            .actors
            .get(actor)
            .ok_or_else(|| action_fault(44))?
            .gauge;
        let shifted = before
            .scaled()
            .checked_add(scaled)
            .ok_or_else(|| action_fault(45))?
            .max(0);
        let after = ActionGauge::from_scaled(shifted).map_err(|_| action_fault(46))?;
        self.set_actor_gauge(actor, after)
    }

    pub(super) fn unit_action_gauge(
        &self,
        owner: UnitId,
    ) -> Result<(TimelineActorId, ActionGauge), BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(43))?;
        let gauge = self
            .state
            .actors
            .get(actor)
            .map(|state| state.gauge)
            .ok_or_else(|| action_fault(44))?;
        Ok((actor, gauge))
    }
}
