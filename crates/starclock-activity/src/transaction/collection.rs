use super::*;

impl ActivityTransactionState {
    pub(super) fn add_counter(
        &mut self,
        id: ActivitySlotId,
        key: u64,
        delta: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        let mut values = self.counter_values(id, key)?;
        match values.binary_search_by_key(&key, |item| item.0) {
            Ok(index) => {
                values[index].1 = values[index]
                    .1
                    .checked_add(delta)
                    .ok_or(ActivityFault::ArithmeticOverflow)?
            }
            Err(index) => values.insert(index, (key, delta)),
        }
        self.commit_counter(id, key, values, cause, events)
    }

    pub(super) fn set_counter(
        &mut self,
        id: ActivitySlotId,
        key: u64,
        value: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        let mut values = self.counter_values(id, key)?;
        match values.binary_search_by_key(&key, |item| item.0) {
            Ok(index) => values[index].1 = value,
            Err(index) => values.insert(index, (key, value)),
        }
        self.commit_counter(id, key, values, cause, events)
    }

    fn counter_values(
        &self,
        id: ActivitySlotId,
        key: u64,
    ) -> Result<Vec<(u64, i64)>, ActivityFault> {
        if key == 0 {
            return Err(ActivityFault::TypeMismatch);
        }
        match self.slots.get(&id).ok_or(ActivityFault::MissingSlot(id))? {
            ActivityValue::BoundedCounterMap(values) => Ok(values.to_vec()),
            _ => Err(ActivityFault::TypeMismatch),
        }
    }

    fn commit_counter(
        &mut self,
        id: ActivitySlotId,
        key: u64,
        values: Vec<(u64, i64)>,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        self.set_slot(
            id,
            ActivityValue::BoundedCounterMap(values.into_boxed_slice()),
        )?;
        push(
            events,
            cause,
            ActivityTransactionEventKind::CounterChanged { slot: id, key },
        );
        Ok(())
    }

    pub(super) fn change_inventory(
        &mut self,
        id: ActivityInventoryId,
        content: u64,
        delta: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        if content == 0 {
            return Err(ActivityFault::TypeMismatch);
        }
        let definition = self
            .definition
            .inventories()
            .iter()
            .find(|item| item.id() == id)
            .ok_or(ActivityFault::MissingInventory(id))?;
        let inventory = self
            .inventories
            .get_mut(&id)
            .ok_or(ActivityFault::MissingInventory(id))?;
        let current = i64::from(*inventory.get(&content).unwrap_or(&0));
        let next = current
            .checked_add(delta)
            .ok_or(ActivityFault::ArithmeticOverflow)?;
        if next < 0
            || next > i64::from(definition.maximum_stack())
            || (current == 0
                && next > 0
                && inventory.len() >= definition.maximum_entries() as usize)
        {
            return Err(ActivityFault::InventoryBounds(id));
        }
        if next == 0 {
            inventory.remove(&content);
        } else {
            inventory.insert(content, next as u32);
        }
        push(
            events,
            cause,
            ActivityTransactionEventKind::InventoryChanged {
                inventory: id,
                content,
            },
        );
        Ok(())
    }

    pub(super) fn set_inventory_count(
        &mut self,
        id: ActivityInventoryId,
        content: u64,
        count: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        let current = i64::from(
            *self
                .inventories
                .get(&id)
                .ok_or(ActivityFault::MissingInventory(id))?
                .get(&content)
                .unwrap_or(&0),
        );
        self.change_inventory(
            id,
            content,
            count
                .checked_sub(current)
                .ok_or(ActivityFault::ArithmeticOverflow)?,
            cause,
            events,
        )
    }

    pub(super) fn change_modifier(
        &mut self,
        id: ActivityModifierId,
        delta: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        let definition = self
            .definition
            .modifiers()
            .iter()
            .find(|item| item.id() == id)
            .ok_or(ActivityFault::MissingModifier(id))?;
        let current = i64::from(
            *self
                .modifiers
                .get(&id)
                .ok_or(ActivityFault::MissingModifier(id))?,
        );
        let next = if delta == i64::MIN {
            0
        } else {
            current
                .checked_add(delta)
                .ok_or(ActivityFault::ArithmeticOverflow)?
        };
        if next < 0 || next > i64::from(definition.maximum_stacks()) {
            return Err(ActivityFault::ModifierBounds(id));
        }
        self.modifiers.insert(id, next as u32);
        push(
            events,
            cause,
            ActivityTransactionEventKind::ModifierChanged(id),
        );
        Ok(())
    }

    pub(super) fn set_modifier_stacks(
        &mut self,
        id: ActivityModifierId,
        stacks: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        let current = i64::from(
            *self
                .modifiers
                .get(&id)
                .ok_or(ActivityFault::MissingModifier(id))?,
        );
        self.change_modifier(
            id,
            stacks
                .checked_sub(current)
                .ok_or(ActivityFault::ArithmeticOverflow)?,
            cause,
            events,
        )
    }
}
