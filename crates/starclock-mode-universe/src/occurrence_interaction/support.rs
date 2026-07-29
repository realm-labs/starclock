use super::*;

const DEFERRED_EFFECT_KEY_BASE: u64 = 1 << 63;

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_pairs(
    output: &mut Vec<PayloadOperation>,
    pairs: impl IntoIterator<
        Item = (
            OccurrenceOperation,
            Option<OccurrenceTarget>,
            Option<AuthoredScalar>,
        ),
    >,
    choice: OccurrenceChoiceId,
    cosmic_fragments: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_bindings: CurioActivityBindings,
    deferred_effects: ActivitySlotId,
    blessing_ids: &[u64],
    blessing_groups: &[Vec<u64>],
    curio_records: &[CurioActivityRecord],
    battle_ready: bool,
) -> Result<(), OccurrenceInteractionError> {
    for (index, (operation, target, scalar)) in pairs.into_iter().enumerate() {
        let sign = operation_sign(operation);
        match target {
            _ if operation == OccurrenceOperation::Battle && battle_ready => {
                output.push(PayloadOperation::Transition);
            }
            _ if operation == OccurrenceOperation::Special => {
                output.push(PayloadOperation::Transition);
            }
            Some(OccurrenceTarget::CosmicFragments) if sign != 0 => {
                let scalar = scalar.unwrap_or_else(default_scalar);
                match scalar.unit() {
                    AuthoredScalarUnit::Scalar => {
                        let value = exact_integer(scalar)?;
                        let delta = value
                            .checked_mul(i64::from(sign))
                            .ok_or(OccurrenceInteractionError::Arithmetic)?;
                        output.push(PayloadOperation::FragmentScalar {
                            slot: cosmic_fragments,
                            gain_inventory: curio_bindings.inventory,
                            delta,
                        });
                    }
                    AuthoredScalarUnit::Percent => {
                        output.push(PayloadOperation::FragmentPercent {
                            slot: cosmic_fragments,
                            gain_inventory: curio_bindings.inventory,
                            coefficient: scalar.value().coefficient(),
                            scale: scalar.value().scale(),
                            sign,
                        });
                    }
                }
            }
            Some(OccurrenceTarget::Blessing) if sign != 0 => {
                if operation == OccurrenceOperation::Enhance && !blessing_groups.is_empty() {
                    let count = scalar
                        .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
                        .map(exact_integer)
                        .transpose()?
                        .unwrap_or(1)
                        .max(1);
                    output.push(PayloadOperation::S05(
                        s05::Operation::EnhanceBestInventoryGroup {
                            inventory: blessing_inventory,
                            quantity: u16::try_from(count)
                                .map_err(|_| OccurrenceInteractionError::Arithmetic)?,
                            groups: blessing_groups.to_vec(),
                        },
                    ));
                } else if sign > 0 && !blessing_groups.is_empty() {
                    output.push(PayloadOperation::EnsureInventoryGroup {
                        inventory: blessing_inventory,
                        groups: blessing_groups.to_vec(),
                    });
                } else {
                    let count = scalar
                        .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
                        .map(exact_integer)
                        .transpose()?
                        .unwrap_or(1)
                        .max(1);
                    output.push(PayloadOperation::Inventory {
                        inventory: blessing_inventory,
                        delta: sign,
                        quantity: u16::try_from(count)
                            .map_err(|_| OccurrenceInteractionError::Arithmetic)?,
                        owned_only: sign < 0 || operation == OccurrenceOperation::Enhance,
                        candidates: blessing_ids.to_vec(),
                    });
                }
            }
            Some(OccurrenceTarget::Curio)
                if sign != 0 && operation != OccurrenceOperation::Enhance =>
            {
                let count = scalar
                    .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
                    .map(exact_integer)
                    .transpose()?
                    .unwrap_or(1)
                    .max(1);
                output.push(PayloadOperation::CurioInventory {
                    bindings: curio_bindings,
                    delta: sign,
                    quantity: u16::try_from(count)
                        .map_err(|_| OccurrenceInteractionError::Arithmetic)?,
                    owned_only: sign < 0,
                    candidates: curio_records.to_vec(),
                });
            }
            Some(OccurrenceTarget::Hp)
                if operation == OccurrenceOperation::Lose
                    && scalar.is_some_and(|value| value.unit() == AuthoredScalarUnit::Percent) =>
            {
                output.push(PayloadOperation::ParticipantHpLoss {
                    scaled_ratio: s02::percent_ratio_scaled(
                        scalar.expect("guarded percentage scalar is present"),
                    )?,
                });
            }
            Some(OccurrenceTarget::Hp)
                if operation == OccurrenceOperation::Restore
                    && scalar.is_some_and(|value| value.unit() == AuthoredScalarUnit::Percent) =>
            {
                output.push(PayloadOperation::ParticipantHpRestore {
                    scaled_ratio: s02::percent_ratio_scaled(
                        scalar.expect("guarded percentage scalar is present"),
                    )?,
                });
            }
            _ => output.push(PayloadOperation::DeferredEffect {
                slot: deferred_effects,
                key: deferred_effect_key(choice, index, operation, target)?,
            }),
        }
    }
    Ok(())
}

pub(super) fn referenced_curios(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    let references = outcome
        .parameter_refs()
        .iter()
        .filter(|value| value.starts_with("universe.curio."))
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    if references.is_empty() {
        return Ok(records.to_vec());
    }
    let mut selected = Vec::with_capacity(references.len());
    for reference in references {
        let id = catalog
            .curios()
            .iter()
            .find(|value| value.stable_key() == reference)
            .map(|value| value.id())
            .ok_or(OccurrenceInteractionError::InvalidChoice)?;
        let record = records
            .iter()
            .copied()
            .find(|value| value.id() == id)
            .ok_or(OccurrenceInteractionError::InvalidChoice)?;
        selected.push(record);
    }
    selected.sort_unstable_by_key(|value| value.id());
    selected.dedup_by_key(|value| value.id());
    Ok(selected)
}

pub(super) fn outcome_pairs(
    outcome: &OccurrenceOutcome,
) -> Vec<(
    OccurrenceOperation,
    Option<OccurrenceTarget>,
    Option<AuthoredScalar>,
)> {
    if outcome.operations().len() == 1 && outcome.targets().len() > 1 {
        return outcome
            .targets()
            .iter()
            .enumerate()
            .map(|(index, target)| {
                (
                    outcome.operations()[0],
                    Some(*target),
                    outcome
                        .numeric_literals()
                        .get(index)
                        .or_else(|| outcome.numeric_literals().first())
                        .copied(),
                )
            })
            .collect();
    }
    outcome
        .operations()
        .iter()
        .enumerate()
        .map(|(index, operation)| {
            (
                *operation,
                outcome
                    .targets()
                    .get(index)
                    .or_else(|| outcome.targets().first())
                    .copied(),
                outcome
                    .numeric_literals()
                    .get(index)
                    .or_else(|| outcome.numeric_literals().first())
                    .copied(),
            )
        })
        .collect()
}

pub(super) fn default_scalar() -> AuthoredScalar {
    AuthoredScalar::new(
        crate::path::ExactParameter::new(1, 0),
        AuthoredScalarUnit::Scalar,
    )
}

pub(super) fn lower_costs(
    output: &mut Vec<PayloadOperation>,
    choice: &OccurrenceChoiceDefinition,
    cosmic_fragments: ActivitySlotId,
    blessing_inventory: ActivityInventoryId,
    curio_inventory: ActivityInventoryId,
    blessing_ids: &[u64],
    curio_ids: &[u64],
) -> Result<(), OccurrenceInteractionError> {
    for cost in choice.costs() {
        for target in cost.targets() {
            match target {
                OccurrenceTarget::CosmicFragments => {
                    output.push(PayloadOperation::RequireFragment {
                        slot: cosmic_fragments,
                        amount: 1,
                    });
                }
                OccurrenceTarget::Blessing => {
                    output.push(PayloadOperation::RequireInventory {
                        inventory: blessing_inventory,
                        candidates: blessing_ids.to_vec(),
                    });
                }
                OccurrenceTarget::Curio => {
                    output.push(PayloadOperation::RequireInventory {
                        inventory: curio_inventory,
                        candidates: curio_ids.to_vec(),
                    });
                }
                OccurrenceTarget::Character | OccurrenceTarget::Hp => {}
            }
        }
    }
    Ok(())
}

pub(super) fn deferred_effect_key(
    choice: OccurrenceChoiceId,
    index: usize,
    operation: OccurrenceOperation,
    target: Option<OccurrenceTarget>,
) -> Result<u64, OccurrenceInteractionError> {
    let index = u64::try_from(index).map_err(|_| OccurrenceInteractionError::Arithmetic)?;
    Ok(DEFERRED_EFFECT_KEY_BASE
        | (u64::from(choice.get()) << 24)
        | (index << 8)
        | (u64::from(operation as u8) << 4)
        | target.map_or(15, |value| u64::from(value as u8)))
}

pub(super) const fn operation_sign(operation: OccurrenceOperation) -> i8 {
    match operation {
        OccurrenceOperation::Obtain | OccurrenceOperation::Enhance => 1,
        OccurrenceOperation::Consume | OccurrenceOperation::Discard | OccurrenceOperation::Lose => {
            -1
        }
        _ => 0,
    }
}

pub(crate) fn exact_integer(value: AuthoredScalar) -> Result<i64, OccurrenceInteractionError> {
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
