//! Mutation-journal recording helpers separated from command resolution.

use super::{journal::MutationField, transaction::Transaction};

impl Transaction<'_> {
    pub(super) fn choose_index(
        &mut self,
        purpose: crate::rng::types::DrawPurpose,
        count: usize,
    ) -> Result<Option<usize>, crate::battle::fault::BattleFault> {
        let count = u32::try_from(count).map_err(|_| super::transaction::action_fault(51))?;
        let before = self.state.rng.draw_count();
        let selected = self
            .state
            .rng
            .choose_index(purpose, count)
            .map_err(|_| super::transaction::action_fault(51))?;
        for index in before..self.state.rng.draw_count() {
            self.journal.rng_draw(index, purpose.code());
        }
        selected
            .map(|value| {
                usize::try_from(value.value()).map_err(|_| super::transaction::action_fault(51))
            })
            .transpose()
    }

    pub(super) fn reset_event_once_keys(&mut self, event: crate::EventId) {
        let count = self.state.rules.reset_once_event(event);
        if count > 0 {
            let before = event.get();
            self.journal.mutation(
                MutationField::RuleState,
                before,
                before ^ u64::try_from(count).expect("rule-state count is bounded"),
            );
        }
    }

    pub(super) fn snapshot(&mut self, operation: crate::OperationId) {
        self.journal.snapshot(operation.get());
    }

    pub(super) fn record_shield_change(
        &mut self,
        before: crate::ShieldAmount,
        after: crate::ShieldAmount,
    ) {
        if before != after {
            self.journal.mutation(
                MutationField::ShieldRemaining,
                u64::try_from(before.get()).expect("shield is non-negative"),
                u64::try_from(after.get()).expect("shield is non-negative"),
            );
        }
    }

    pub(super) fn record_effect_change(&mut self, before: u64, after: u64, identity: u64) {
        let encoded_before = before.checked_mul(2).expect("effect budget is bounded");
        let mut encoded_after = after.checked_mul(2).expect("effect budget is bounded");
        if encoded_before == encoded_after {
            encoded_after = encoded_after
                .checked_add(identity | 1)
                .expect("effect identity is bounded");
        }
        self.journal
            .mutation(MutationField::Effect, encoded_before, encoded_after);
    }

    pub(super) fn record_rule_state_change(
        &mut self,
        instance: crate::RuleInstanceId,
        slot: crate::StateSlotDefinitionId,
        before: &crate::rule::model::RuleValue,
        after: &crate::rule::model::RuleValue,
    ) {
        if before != after {
            let key = instance.get().rotate_left(17) ^ u64::from(slot.get());
            self.journal
                .mutation(MutationField::RuleState, key, key ^ 1);
        }
    }
}
