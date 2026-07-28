//! Shared primitives introduced by Goal 07 Occurrence partition S10.

use starclock_combat::{Hp, LifeState, Ratio};

use super::*;

const TAG_BANK: u8 = 31;
const TAG_BEAUTY_BUG: u8 = 32;
const PREFIX: &str = "universe.occurrence-s10.";
const BANK_KEY: u64 = 0x5c00_0000_0000_0001;
const BEAUTY_BUG_UNLOCK_KEY: u64 = 0x5d00_0000_0000_0001;
const BANK_DEPOSIT: u8 = 1;
const BANK_WITHDRAW: u8 = 2;
const BANK_PRESERVE: u8 = 3;
const BANK_LEAVE: u8 = 4;

#[derive(Clone)]
pub(super) enum Operation {
    Bank {
        fragments_slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        state_slot: ActivitySlotId,
        action: u8,
        amount: i64,
        cost: i64,
    },
    BeautyBug {
        bindings: CurioActivityBindings,
        state_slot: ActivitySlotId,
        record: CurioActivityRecord,
    },
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) {
        match self {
            Self::Bank {
                fragments_slot,
                gain_inventory,
                state_slot,
                action,
                amount,
                cost,
            } => {
                output.push(TAG_BANK);
                output.extend_from_slice(&fragments_slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&state_slot.get().to_le_bytes());
                output.push(action);
                output.extend_from_slice(&amount.to_le_bytes());
                output.extend_from_slice(&cost.to_le_bytes());
            }
            Self::BeautyBug {
                bindings,
                state_slot,
                record,
            } => {
                output.push(TAG_BEAUTY_BUG);
                encode_bindings(output, bindings);
                output.extend_from_slice(&state_slot.get().to_le_bytes());
                encode_record(output, record);
            }
        }
    }

    pub(super) const fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::BeautyBug { .. } => Some(100),
            Self::Bank { .. } => None,
        }
    }
}

pub(super) fn externalize(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
    state_slot: ActivitySlotId,
) -> Result<Option<external::Lowering>, OccurrenceInteractionError> {
    if marker(outcome) != Some("beauty-bug-feed-curio") {
        return Ok(None);
    }
    if catalog.curios().len() != records.len() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(Some(external::Lowering {
        choices: records
            .iter()
            .copied()
            .map(|record| external::Choice {
                content: u64::from(record.id().get()),
                operations: vec![PayloadOperation::S10(Operation::BeautyBug {
                    bindings,
                    state_slot,
                    record,
                })],
                random_candidate_count: Some(100),
            })
            .collect(),
        repeat_key: None,
    }))
}

pub(super) fn lower(
    outcome: &OccurrenceOutcome,
    fragments_slot: ActivitySlotId,
    gain_inventory: ActivityInventoryId,
    state_slot: ActivitySlotId,
) -> Result<Option<Operation>, OccurrenceInteractionError> {
    let Some(kind) = marker(outcome) else {
        return Ok(None);
    };
    if kind == "beauty-bug-feed-curio" {
        return Ok(None);
    }
    let (action, amount, cost) = match kind {
        "bank-deposit-200" => (BANK_DEPOSIT, 200, 100),
        "bank-deposit-400" => (BANK_DEPOSIT, 400, 150),
        "bank-deposit-600" => (BANK_DEPOSIT, 600, 200),
        "bank-withdraw-200" => (BANK_WITHDRAW, 200, 0),
        "bank-withdraw-400" => (BANK_WITHDRAW, 400, 0),
        "bank-withdraw-600" => (BANK_WITHDRAW, 600, 0),
        "bank-preserve-200" => (BANK_PRESERVE, 200, 0),
        "bank-preserve-400" => (BANK_PRESERVE, 400, 0),
        "bank-preserve-600" => (BANK_PRESERVE, 600, 0),
        "bank-leave-0" => (BANK_LEAVE, 0, 0),
        _ => return Err(OccurrenceInteractionError::InvalidChoice),
    };
    Ok(Some(Operation::Bank {
        fragments_slot,
        gain_inventory,
        state_slot,
        action,
        amount,
        cost,
    }))
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_BANK => decode_bank(input, decoder, operations)?,
        TAG_BEAUTY_BUG => decode_beauty_bug(input, decoder, operations)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_bank(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let fragments_slot = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let state_slot = slot(decoder.u32()?)?;
    let action = decoder.u8()?;
    let amount = decoder.i64()?;
    let cost = decoder.i64()?;
    let stored = counter_value(input, state_slot, BANK_KEY)?;
    let tier: i64 = match amount {
        200 => 1,
        400 => 2,
        600 => 3,
        0 => 0,
        _ => return Err(invalid_payload()),
    };
    match action {
        BANK_DEPOSIT if amount > 0 && cost > 0 => {
            operations.push(require_at_least(
                fragments_slot,
                u64::try_from(cost).map_err(|_| invalid_payload())?,
            )?);
            operations.push(fragment_delta(fragments_slot, gain_inventory, -cost));
            operations.push(add_counter(
                state_slot,
                BANK_KEY,
                tier.checked_sub(stored).ok_or_else(invalid_payload)?,
            ));
        }
        BANK_WITHDRAW if amount > 0 && cost == 0 && stored == tier => {
            operations.push(fragment_delta(fragments_slot, gain_inventory, amount));
            operations.push(add_counter(state_slot, BANK_KEY, -stored));
        }
        BANK_PRESERVE if amount > 0 && cost == 0 && stored == tier => {}
        BANK_LEAVE if amount == 0 && cost == 0 => {}
        _ => return Err(invalid_state()),
    }
    Ok(())
}

fn decode_beauty_bug(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let state_slot = slot(decoder.u32()?)?;
    let record = decode_record(decoder)?;
    let random = input.random_index().ok_or_else(invalid_state)?;
    operations.extend(teardown_operations(record.id(), bindings));
    if random < 70 {
        let current = counter_value(input, state_slot, BEAUTY_BUG_UNLOCK_KEY)?;
        if current == 0 {
            operations.push(add_counter(state_slot, BEAUTY_BUG_UNLOCK_KEY, 1));
        }
    } else {
        let ratio = Ratio::from_scaled(300_000);
        operations.extend(
            input
                .view()
                .participant_carry()
                .iter()
                .filter(|state| state.life() == LifeState::Alive)
                .map(|state| ActivityOperation::LoseParticipantCurrentHpRatio {
                    participant: state.participant(),
                    hp_ratio: ratio,
                    minimum_hp: Hp::new(1).expect("one HP is valid"),
                }),
        );
    }
    Ok(())
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

fn encode_record(output: &mut Vec<u8>, record: CurioActivityRecord) {
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

fn decode_record(decoder: &mut Decoder<'_>) -> Result<CurioActivityRecord, ActivityHandlerFault> {
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
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
