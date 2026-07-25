use crate::battle::fault::BattleFault;

use super::{
    journal::{AllocationKind, MutationField, QueueKind},
    transaction::{Transaction, action_fault},
};

impl Transaction<'_> {
    pub(super) fn enqueue_extra_turn(&mut self, unit: crate::UnitId) -> Result<u64, BattleFault> {
        let insertion = self
            .state
            .sequences
            .try_extra_turn()
            .ok_or_else(|| action_fault(74))?;
        let before = self.state.timeline.extra_turns.len() as u64;
        self.state
            .timeline
            .push_extra_turn(crate::timeline::state::PendingExtraTurn { insertion, unit });
        self.journal
            .allocation(AllocationKind::ExtraTurn, insertion);
        self.journal
            .queue_insertion(QueueKind::ExtraTurn, insertion);
        self.journal.mutation(
            MutationField::Timeline,
            before,
            self.state.timeline.extra_turns.len() as u64,
        );
        Ok(insertion)
    }

    pub(super) fn pop_extra_turn(&mut self) -> Option<crate::timeline::state::PendingExtraTurn> {
        let before = self.state.timeline.extra_turns.len() as u64;
        let pending = self.state.timeline.pop_extra_turn()?;
        self.journal.mutation(
            MutationField::Timeline,
            before,
            self.state.timeline.extra_turns.len() as u64,
        );
        Some(pending)
    }

    pub(super) fn clear_extra_turns(&mut self) {
        let before = self.state.timeline.extra_turns.len() as u64;
        if before != 0 {
            self.state.timeline.extra_turns.clear();
            self.journal.mutation(MutationField::Timeline, before, 0);
        }
    }
}
