use super::*;

pub(super) fn exact_integer(value: AuthoredScalar) -> Result<i64, OccurrenceInteractionError> {
    let divisor = 10_i64
        .checked_pow(u32::from(value.value().scale()))
        .ok_or(OccurrenceInteractionError::Arithmetic)?;
    if value.value().coefficient() % divisor != 0 {
        return Err(OccurrenceInteractionError::NonIntegerScalar);
    }
    Ok(value.value().coefficient() / divisor)
}

pub(super) fn checked_lcm(left: u32, right: u32) -> Option<u32> {
    let gcd = gcd(left, right);
    left.checked_div(gcd)?
        .checked_mul(right)
        .filter(|value| *value <= 65_536)
}

const fn gcd(mut left: u32, mut right: u32) -> u32 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

pub(super) fn select_candidates(
    input: ActivityHandlerInput<'_>,
    inventory: ActivityInventoryId,
    candidates: &[u64],
    owned_only: bool,
    random_index: Option<u32>,
    quantity: usize,
) -> Result<Vec<u64>, ActivityHandlerFault> {
    let eligible = if owned_only {
        let entries = input
            .view()
            .inventories()
            .iter()
            .find(|value| value.id() == inventory)
            .ok_or_else(invalid_state)?
            .entries();
        candidates
            .iter()
            .copied()
            .filter(|candidate| {
                entries
                    .iter()
                    .any(|entry| entry.0 == *candidate && entry.1 > 0)
            })
            .collect::<Vec<_>>()
    } else {
        candidates.to_vec()
    };
    if eligible.len() < quantity {
        return Err(invalid_state());
    }
    let start = random_index.map_or(0, |index| index as usize % eligible.len());
    Ok((0..quantity)
        .map(|offset| eligible[(start + offset) % eligible.len()])
        .collect())
}

pub(super) fn slot_integer(
    input: ActivityHandlerInput<'_>,
    id: ActivitySlotId,
) -> Result<i64, ActivityHandlerFault> {
    input
        .view()
        .slots()
        .iter()
        .find(|value| value.id() == id)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedInteger(value) => Some(*value),
            _ => None,
        })
        .ok_or_else(invalid_state)
}

fn add_slot(slot: ActivitySlotId, delta: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot,
        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(delta)),
    }
}

pub(super) fn fragment_delta(
    slot: ActivitySlotId,
    gain_inventory: ActivityInventoryId,
    delta: i64,
) -> ActivityOperation {
    if delta <= 0 {
        return add_slot(slot, delta);
    }
    ActivityOperation::AddToSlot {
        slot,
        delta: ActivityExpression::Multiply(
            Box::new(ActivityExpression::Literal(ActivityValue::BoundedInteger(
                delta,
            ))),
            Box::new(ActivityExpression::Add(
                Box::new(ActivityExpression::Literal(ActivityValue::BoundedInteger(
                    1,
                ))),
                Box::new(ActivityExpression::InventoryCount {
                    inventory: gain_inventory,
                    content: crate::curio_activity::GOSSIP_CURIO_CONTENT,
                }),
            )),
        ),
    }
}

pub(super) fn require_at_least(
    slot: ActivitySlotId,
    amount: u64,
) -> Result<ActivityOperation, ActivityHandlerFault> {
    let amount = i64::try_from(amount).map_err(|_| arithmetic())?;
    Ok(ActivityOperation::Require(ActivityCondition::Not(
        Box::new(ActivityCondition::LessThan(
            ActivityExpression::Slot(slot),
            ActivityExpression::Literal(ActivityValue::BoundedInteger(amount)),
        )),
    )))
}

pub(super) fn slot(value: u32) -> Result<ActivitySlotId, ActivityHandlerFault> {
    ActivitySlotId::new(value).ok_or_else(invalid_payload)
}

pub(super) fn inventory(value: u32) -> Result<ActivityInventoryId, ActivityHandlerFault> {
    ActivityInventoryId::new(value).ok_or_else(invalid_payload)
}

pub(super) struct Decoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Decoder<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], ActivityHandlerFault> {
        let end = self.cursor.checked_add(count).ok_or_else(invalid_payload)?;
        let value = self
            .bytes
            .get(self.cursor..end)
            .ok_or_else(invalid_payload)?;
        self.cursor = end;
        Ok(value)
    }

    pub(super) fn u8(&mut self) -> Result<u8, ActivityHandlerFault> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn i8(&mut self) -> Result<i8, ActivityHandlerFault> {
        Ok(i8::from_le_bytes([self.u8()?]))
    }

    pub(super) fn u16(&mut self) -> Result<u16, ActivityHandlerFault> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub(super) fn u32(&mut self) -> Result<u32, ActivityHandlerFault> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub(super) fn u64(&mut self) -> Result<u64, ActivityHandlerFault> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub(super) fn i64(&mut self) -> Result<i64, ActivityHandlerFault> {
        Ok(i64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub(super) fn finish(self) -> Result<(), ActivityHandlerFault> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(invalid_payload())
        }
    }
}

pub(super) fn invalid_payload() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidPayload)
}

pub(super) fn invalid_state() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::InvalidState)
}

pub(super) fn arithmetic() -> ActivityHandlerFault {
    ActivityHandlerFault::new(ActivityHandlerFaultKind::Arithmetic)
}
