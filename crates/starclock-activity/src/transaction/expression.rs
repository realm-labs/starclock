use super::*;

impl ActivityTransactionState {
    pub(super) fn evaluate(
        &self,
        expression: &ActivityExpression,
    ) -> Result<ActivityValue, ActivityFault> {
        match expression {
            ActivityExpression::Literal(value) => Ok(value.clone()),
            ActivityExpression::Slot(slot) => self
                .slots
                .get(slot)
                .cloned()
                .ok_or(ActivityFault::MissingSlot(*slot)),
            ActivityExpression::CounterValue { slot, key } => {
                if *key == 0 {
                    return Err(ActivityFault::TypeMismatch);
                }
                match self
                    .slots
                    .get(slot)
                    .ok_or(ActivityFault::MissingSlot(*slot))?
                {
                    ActivityValue::BoundedCounterMap(values) => Ok(ActivityValue::BoundedInteger(
                        values
                            .binary_search_by_key(key, |item| item.0)
                            .ok()
                            .map(|index| values[index].1)
                            .unwrap_or(0),
                    )),
                    _ => Err(ActivityFault::TypeMismatch),
                }
            }
            ActivityExpression::CounterEntryCount(slot) => match self
                .slots
                .get(slot)
                .ok_or(ActivityFault::MissingSlot(*slot))?
            {
                ActivityValue::BoundedCounterMap(values) => count(values.len()),
                _ => Err(ActivityFault::TypeMismatch),
            },
            ActivityExpression::OrderedIdSetCount(slot) => match self
                .slots
                .get(slot)
                .ok_or(ActivityFault::MissingSlot(*slot))?
            {
                ActivityValue::OrderedIdSet(values) => count(values.len()),
                _ => Err(ActivityFault::TypeMismatch),
            },
            ActivityExpression::InventoryCount { inventory, content } => {
                if *content == 0 {
                    return Err(ActivityFault::TypeMismatch);
                }
                let values = self
                    .inventories
                    .get(inventory)
                    .ok_or(ActivityFault::MissingInventory(*inventory))?;
                Ok(ActivityValue::BoundedInteger(i64::from(
                    *values.get(content).unwrap_or(&0),
                )))
            }
            ActivityExpression::InventoryEntryCount(inventory) => count(
                self.inventories
                    .get(inventory)
                    .ok_or(ActivityFault::MissingInventory(*inventory))?
                    .len(),
            ),
            ActivityExpression::ModifierStacks(modifier) => {
                Ok(ActivityValue::BoundedInteger(i64::from(
                    *self
                        .modifiers
                        .get(modifier)
                        .ok_or(ActivityFault::MissingModifier(*modifier))?,
                )))
            }
            ActivityExpression::Add(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_add)
            }
            ActivityExpression::Subtract(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_sub)
            }
            ActivityExpression::Multiply(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_mul)
            }
            ActivityExpression::Divide(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, |a, b| {
                    (b != 0).then(|| a / b)
                })
            }
            ActivityExpression::Minimum(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, |a, b| Some(a.min(b)))
            }
            ActivityExpression::Maximum(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, |a, b| Some(a.max(b)))
            }
            ActivityExpression::Negate(value) => match self.evaluate(value)? {
                ActivityValue::BoundedInteger(value) => value
                    .checked_neg()
                    .map(ActivityValue::BoundedInteger)
                    .ok_or(ActivityFault::ArithmeticOverflow),
                ActivityValue::FixedScalar(value) => value
                    .checked_neg()
                    .map(ActivityValue::FixedScalar)
                    .ok_or(ActivityFault::ArithmeticOverflow),
                _ => Err(ActivityFault::TypeMismatch),
            },
        }
    }
}

fn count(value: usize) -> Result<ActivityValue, ActivityFault> {
    i64::try_from(value)
        .map(ActivityValue::BoundedInteger)
        .map_err(|_| ActivityFault::ArithmeticOverflow)
}
