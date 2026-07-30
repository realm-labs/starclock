use super::*;

impl ActivityTransactionState {
    pub(super) fn insert_ordered_id(
        &mut self,
        slot: ActivitySlotId,
        id: u64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        if id == 0 {
            return Err(ActivityFault::TypeMismatch);
        }
        let mut values = match self
            .slots
            .get(&slot)
            .ok_or(ActivityFault::MissingSlot(slot))?
        {
            ActivityValue::OrderedIdSet(values) => values.to_vec(),
            _ => return Err(ActivityFault::TypeMismatch),
        };
        let Err(index) = values.binary_search(&id) else {
            return Ok(());
        };
        values.insert(index, id);
        self.set_slot(slot, ActivityValue::OrderedIdSet(values.into_boxed_slice()))?;
        push(
            events,
            cause,
            ActivityTransactionEventKind::SlotChanged(slot),
        );
        Ok(())
    }
}
