//! Shared primitives introduced by Goal 07 Occurrence partition S12.

use super::*;

const TAG_MIRROR_RESCUE: u8 = 42;
const TAG_CUCKOO_ACQUIRE: u8 = 43;
const TAG_CUCKOO_DISCARD_ALL: u8 = 44;
const TAG_CUCKOO_EXCHANGE_CURIOS: u8 = 45;
const TAG_CUCKOO_EXCHANGE_BLESSINGS: u8 = 46;
const PREFIX: &str = "universe.occurrence-s12.";
pub(super) const MIRROR_PART_TWO_KEY: u64 = 0x5f00_0000_0000_0005;

#[derive(Clone)]
pub(super) enum Operation {
    MirrorRescue {
        state_slot: ActivitySlotId,
    },
    CuckooAcquire {
        bindings: CurioActivityBindings,
        clocks: Vec<CurioActivityRecord>,
        clock_quantity: u8,
        blessing_inventory: ActivityInventoryId,
        blessings: Vec<u64>,
    },
    CuckooDiscardAll {
        bindings: CurioActivityBindings,
        clocks: Vec<CurioActivityRecord>,
    },
    CuckooExchangeCurios {
        bindings: CurioActivityBindings,
        clocks: Vec<CurioActivityRecord>,
        rewards: Vec<CurioActivityRecord>,
    },
    CuckooExchangeBlessings {
        bindings: CurioActivityBindings,
        clocks: Vec<CurioActivityRecord>,
        blessing_inventory: ActivityInventoryId,
        blessings: Vec<u64>,
    },
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::MirrorRescue { state_slot } => {
                output.push(TAG_MIRROR_RESCUE);
                output.extend_from_slice(&state_slot.get().to_le_bytes());
            }
            Self::CuckooAcquire {
                bindings,
                clocks,
                clock_quantity,
                blessing_inventory,
                blessings,
            } => {
                output.push(TAG_CUCKOO_ACQUIRE);
                encode_bindings(output, bindings);
                encode_records(output, clocks)?;
                output.push(clock_quantity);
                output.extend_from_slice(&blessing_inventory.get().to_le_bytes());
                encode_ids_allow_empty(output, blessings)?;
            }
            Self::CuckooDiscardAll { bindings, clocks } => {
                output.push(TAG_CUCKOO_DISCARD_ALL);
                encode_bindings(output, bindings);
                encode_records(output, clocks)?;
            }
            Self::CuckooExchangeCurios {
                bindings,
                clocks,
                rewards,
            } => {
                output.push(TAG_CUCKOO_EXCHANGE_CURIOS);
                encode_bindings(output, bindings);
                encode_records(output, clocks)?;
                encode_records(output, rewards)?;
            }
            Self::CuckooExchangeBlessings {
                bindings,
                clocks,
                blessing_inventory,
                blessings,
            } => {
                output.push(TAG_CUCKOO_EXCHANGE_BLESSINGS);
                encode_bindings(output, bindings);
                encode_records(output, clocks)?;
                output.extend_from_slice(&blessing_inventory.get().to_le_bytes());
                encode_ids_allow_empty(output, blessings)?;
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::MirrorRescue { .. } | Self::CuckooDiscardAll { .. } => None,
            Self::CuckooAcquire {
                clocks, blessings, ..
            } => candidate_product(clocks.len(), blessings.len()),
            Self::CuckooExchangeCurios { rewards, .. } => u32::try_from(rewards.len()).ok(),
            Self::CuckooExchangeBlessings { blessings, .. } => u32::try_from(blessings.len()).ok(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
    state_slot: ActivitySlotId,
) -> Result<Option<Operation>, OccurrenceInteractionError> {
    let Some(kind) = marker(outcome) else {
        return Ok(None);
    };
    let operation = match kind {
        "mirror-rescue" => Operation::MirrorRescue { state_slot },
        "mirror-all-candles-lit" => return Ok(None),
        "cuckoo-acquire-one" => Operation::CuckooAcquire {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
            clock_quantity: 1,
            blessing_inventory,
            blessings: Vec::new(),
        },
        "cuckoo-acquire-two" => Operation::CuckooAcquire {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
            clock_quantity: 2,
            blessing_inventory,
            blessings: Vec::new(),
        },
        "cuckoo-acquire-one-rarity-two-blessing" => Operation::CuckooAcquire {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
            clock_quantity: 1,
            blessing_inventory,
            blessings: blessings(catalog, 2),
        },
        "cuckoo-discard-all" => Operation::CuckooDiscardAll {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
        },
        "cuckoo-acquire-one-rarity-three-blessing" => Operation::CuckooAcquire {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
            clock_quantity: 1,
            blessing_inventory,
            blessings: blessings(catalog, 3),
        },
        "cuckoo-exchange-all-for-curios" => Operation::CuckooExchangeCurios {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
            rewards: positive_curios(catalog, records)?,
        },
        "cuckoo-exchange-all-for-blessings" => Operation::CuckooExchangeBlessings {
            bindings,
            clocks: referenced_clocks(outcome, catalog, records)?,
            blessing_inventory,
            blessings: catalog
                .blessings()
                .iter()
                .map(|value| u64::from(value.id().get()))
                .collect(),
        },
        _ => return Err(OccurrenceInteractionError::InvalidChoice),
    };
    Ok(Some(operation))
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_MIRROR_RESCUE => decode_mirror_rescue(input, decoder, operations)?,
        TAG_CUCKOO_ACQUIRE => decode_acquire(input, decoder, operations)?,
        TAG_CUCKOO_DISCARD_ALL => decode_discard_all(input, decoder, operations)?,
        TAG_CUCKOO_EXCHANGE_CURIOS => decode_exchange_curios(input, decoder, operations)?,
        TAG_CUCKOO_EXCHANGE_BLESSINGS => {
            decode_exchange_blessings(input, decoder, operations)?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_mirror_rescue(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let state_slot = slot(decoder.u32()?)?;
    if counter_value(input, state_slot, MIRROR_PART_TWO_KEY)? == 0 {
        operations.push(add_counter(state_slot, MIRROR_PART_TWO_KEY, 1));
    }
    Ok(())
}

fn decode_acquire(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let clocks = decode_records(decoder)?;
    let clock_quantity = usize::from(decoder.u8()?);
    let blessing_inventory = inventory(decoder.u32()?)?;
    let blessings = decode_ids_allow_empty(decoder)?;
    let random = input.random_index().ok_or_else(invalid_state)?;
    if clock_quantity == 0 {
        return Err(invalid_payload());
    }
    let eligible_clocks = clocks
        .iter()
        .copied()
        .filter(|record| {
            inventory_count(input, bindings.inventory, u64::from(record.id().get())) == Some(0)
        })
        .collect::<Vec<_>>();
    let clock_entropy = if blessings.is_empty() {
        random
    } else {
        random / u32::try_from(blessings.len()).map_err(|_| invalid_state())?
    };
    for selected in select_records(&eligible_clocks, clock_entropy, clock_quantity)? {
        operations.extend(acquisition_operations(selected, bindings));
    }
    if !blessings.is_empty() {
        let content = select_candidates(
            input,
            blessing_inventory,
            &blessings,
            false,
            Some(random),
            1,
        )?[0];
        operations.push(ActivityOperation::AddInventory {
            inventory: blessing_inventory,
            content,
            count: integer(1),
        });
    }
    Ok(())
}

fn decode_discard_all(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let clocks = decode_records(decoder)?;
    let owned = owned_records(input, bindings.inventory, &clocks)?;
    if owned.is_empty() {
        return Err(invalid_state());
    }
    for record in owned {
        operations.extend(teardown_operations(record.id(), bindings));
    }
    Ok(())
}

fn decode_exchange_curios(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let clocks = decode_records(decoder)?;
    let rewards = decode_records(decoder)?;
    let owned = owned_records(input, bindings.inventory, &clocks)?;
    if owned.is_empty() {
        return Err(invalid_state());
    }
    let eligible = rewards
        .iter()
        .copied()
        .filter(|record| {
            inventory_count(input, bindings.inventory, u64::from(record.id().get())) == Some(0)
        })
        .collect::<Vec<_>>();
    let selected = select_records(
        &eligible,
        input.random_index().ok_or_else(invalid_state)?,
        owned.len(),
    )?;
    for record in owned {
        operations.extend(teardown_operations(record.id(), bindings));
    }
    for record in selected {
        operations.extend(acquisition_operations(record, bindings));
    }
    Ok(())
}

fn decode_exchange_blessings(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let clocks = decode_records(decoder)?;
    let blessing_inventory = inventory(decoder.u32()?)?;
    let blessings = decode_ids_allow_empty(decoder)?;
    let owned = owned_records(input, bindings.inventory, &clocks)?;
    if owned.is_empty() || blessings.is_empty() {
        return Err(invalid_state());
    }
    let selected = select_candidates(
        input,
        blessing_inventory,
        &blessings,
        false,
        input.random_index(),
        owned.len(),
    )?;
    for record in owned {
        operations.extend(teardown_operations(record.id(), bindings));
    }
    for content in selected {
        operations.push(ActivityOperation::AddInventory {
            inventory: blessing_inventory,
            content,
            count: integer(1),
        });
    }
    Ok(())
}

fn referenced_clocks(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    let keys = outcome
        .parameter_refs()
        .iter()
        .filter(|value| value.starts_with("universe.curio."))
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();
    if keys.len() != 6 {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    let mut clocks = keys
        .into_iter()
        .map(|key| {
            let id = catalog
                .curios()
                .iter()
                .find(|value| value.stable_key() == key)
                .map(|value| value.id())
                .ok_or(OccurrenceInteractionError::InvalidChoice)?;
            records
                .iter()
                .copied()
                .find(|value| value.id() == id)
                .ok_or(OccurrenceInteractionError::InvalidChoice)
        })
        .collect::<Result<Vec<_>, _>>()?;
    clocks.sort_unstable_by_key(|value| value.id());
    clocks.dedup_by_key(|value| value.id());
    (clocks.len() == 6)
        .then_some(clocks)
        .ok_or(OccurrenceInteractionError::InvalidChoice)
}

fn positive_curios(
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    let rewards = records
        .iter()
        .copied()
        .filter(|record| {
            catalog
                .curios()
                .iter()
                .find(|value| value.id() == record.id())
                .is_some_and(|value| {
                    value
                        .pool_tags()
                        .iter()
                        .any(|tag| tag.as_ref() == "polarity:positive")
                })
        })
        .collect::<Vec<_>>();
    (!rewards.is_empty())
        .then_some(rewards)
        .ok_or(OccurrenceInteractionError::InvalidChoice)
}

fn blessings(catalog: &UniverseCatalog, rarity: u8) -> Vec<u64> {
    catalog
        .blessings()
        .iter()
        .filter(|value| value.rarity() == rarity)
        .map(|value| u64::from(value.id().get()))
        .collect()
}

fn owned_records(
    input: ActivityHandlerInput<'_>,
    inventory: ActivityInventoryId,
    records: &[CurioActivityRecord],
) -> Result<Vec<CurioActivityRecord>, ActivityHandlerFault> {
    let entries = input
        .view()
        .inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .ok_or_else(invalid_state)?
        .entries();
    Ok(records
        .iter()
        .copied()
        .filter(|record| {
            entries
                .iter()
                .any(|entry| entry.0 == u64::from(record.id().get()) && entry.1 > 0)
        })
        .collect())
}

fn inventory_count(
    input: ActivityHandlerInput<'_>,
    inventory: ActivityInventoryId,
    content: u64,
) -> Option<u32> {
    input
        .view()
        .inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .map(|value| {
            value
                .entries()
                .iter()
                .find(|entry| entry.0 == content)
                .map_or(0, |entry| entry.1)
        })
}

fn select_records(
    records: &[CurioActivityRecord],
    random: u32,
    quantity: usize,
) -> Result<Vec<CurioActivityRecord>, ActivityHandlerFault> {
    if records.len() < quantity || quantity == 0 {
        return Err(invalid_state());
    }
    let start = random as usize % records.len();
    Ok((0..quantity)
        .map(|offset| records[(start + offset) % records.len()])
        .collect())
}

fn candidate_product(clocks: usize, blessings: usize) -> Option<u32> {
    let clocks = u32::try_from(clocks).ok()?;
    if blessings == 0 {
        return Some(clocks);
    }
    clocks.checked_mul(u32::try_from(blessings).ok()?)
}

fn marker(outcome: &OccurrenceOutcome) -> Option<&str> {
    outcome
        .parameter_refs()
        .iter()
        .find_map(|value| value.strip_prefix(PREFIX))
}

fn counter_value(
    input: ActivityHandlerInput<'_>,
    slot: ActivitySlotId,
    key: u64,
) -> Result<i64, ActivityHandlerFault> {
    input
        .view()
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(entries) => Some(
                entries
                    .iter()
                    .find(|entry| entry.0 == key)
                    .map_or(0, |entry| entry.1),
            ),
            _ => None,
        })
        .ok_or_else(invalid_state)
}

fn add_counter(slot: ActivitySlotId, key: u64, delta: i64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot,
        key,
        delta: integer(delta),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

fn encode_bindings(output: &mut Vec<u8>, bindings: CurioActivityBindings) {
    for value in [
        bindings.inventory.get(),
        bindings.state_slot.get(),
        bindings.charge_slot.get(),
        bindings.event_slot.get(),
        bindings.fragments_slot.get(),
    ] {
        output.extend_from_slice(&value.to_le_bytes());
    }
}

fn decode_bindings(
    decoder: &mut Decoder<'_>,
) -> Result<CurioActivityBindings, ActivityHandlerFault> {
    Ok(CurioActivityBindings {
        inventory: inventory(decoder.u32()?)?,
        state_slot: slot(decoder.u32()?)?,
        charge_slot: slot(decoder.u32()?)?,
        event_slot: slot(decoder.u32()?)?,
        fragments_slot: slot(decoder.u32()?)?,
    })
}

fn encode_ids_allow_empty(
    output: &mut Vec<u8>,
    values: Vec<u64>,
) -> Result<(), OccurrenceInteractionError> {
    output.extend_from_slice(
        &u16::try_from(values.len())
            .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
            .to_le_bytes(),
    );
    for value in values {
        output.extend_from_slice(&value.to_le_bytes());
    }
    Ok(())
}

fn decode_ids_allow_empty(decoder: &mut Decoder<'_>) -> Result<Vec<u64>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    (0..count).map(|_| decoder.u64()).collect()
}

fn encode_records(
    output: &mut Vec<u8>,
    records: Vec<CurioActivityRecord>,
) -> Result<(), OccurrenceInteractionError> {
    output.extend_from_slice(
        &u16::try_from(records.len())
            .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
            .to_le_bytes(),
    );
    for record in records {
        output.extend_from_slice(&record.id().get().to_le_bytes());
        output.extend_from_slice(&record.initial_state().get().to_le_bytes());
        output.push(record.initial_charges());
        output.extend_from_slice(
            &record
                .acquisition_fragment_divisor()
                .unwrap_or(0)
                .to_le_bytes(),
        );
        output.extend_from_slice(
            &record
                .acquisition_fragment_stack_divisor()
                .unwrap_or(0)
                .to_le_bytes(),
        );
    }
    Ok(())
}

fn decode_records(
    decoder: &mut Decoder<'_>,
) -> Result<Vec<CurioActivityRecord>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    (0..count)
        .map(|_| {
            let record = CurioActivityRecord::new(
                CurioId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
                CurioStateId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
                decoder.u8()?,
                match decoder.i64()? {
                    0 => None,
                    value => Some(value),
                },
            );
            Ok(match decoder.i64()? {
                0 => record,
                value => record.with_fragment_stack_capture(value),
            })
        })
        .collect()
}
