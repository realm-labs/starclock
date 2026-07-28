use crate::{TimelineActorId, UnitId, battle::fault::BattleFault, numeric::domain::ActionGauge};

use super::transaction::{Transaction, action_fault};

impl Transaction<'_> {
    pub(super) fn add_timeline_elapsed(&mut self, delta: i64) -> Result<(), BattleFault> {
        self.timeline_elapsed_scaled = self
            .timeline_elapsed_scaled
            .checked_add(delta)
            .ok_or_else(|| action_fault(101))?;
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
