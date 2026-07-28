//! Shared primitives introduced by Goal 07 Occurrence partition S09.

use crate::curio_activity::negative::{destroyed_available, restore_destroyed_operations};

use super::*;

const TAG_REPAIR_DESTROYED: u8 = 27;
const TAG_DOUBLE_LOTTERY: u8 = 28;
const TAG_ENHANCE_ALL: u8 = 29;
const TAG_PERFECT_CHALLENGE: u8 = 30;
const PREFIX: &str = "universe.occurrence-s09.";
const PERFECT_STAGE_KEY: u64 = 0x5a00_0000_0000_0001;
const PERFECT_REPEAT_KEY: u64 = 0x5b00_0000_0000_0001;
const PERFECT_PAY: u8 = 1;
const PERFECT_LEAVE: u8 = 2;
const PERFECT_CLAY: u8 = 3;
const PERFECT_POPULAR: u8 = 4;

#[derive(Clone)]
pub(super) enum Operation {
    RepairDestroyed {
        bindings: CurioActivityBindings,
        cost: i64,
        records: Vec<CurioActivityRecord>,
    },
    DoubleLottery {
        bindings: CurioActivityBindings,
        records: Vec<CurioActivityRecord>,
    },
    EnhanceAll {
        inventory: ActivityInventoryId,
        candidates: Vec<u64>,
    },
    PerfectChallenge {
        bindings: CurioActivityBindings,
        effect_slot: ActivitySlotId,
        action: u8,
        expected_stage: i64,
        next_stage: i64,
        repeat: bool,
        positive: Vec<CurioActivityRecord>,
        negative: Vec<CurioActivityRecord>,
    },
}

pub(super) struct Lowering {
    pub(super) operations: Vec<PayloadOperation>,
    pub(super) repeat_key: Option<u64>,
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::RepairDestroyed {
                bindings,
                cost,
                records,
            } => {
                output.push(TAG_REPAIR_DESTROYED);
                encode_bindings(output, bindings);
                output.extend_from_slice(&cost.to_le_bytes());
                encode_records(output, records)?;
            }
            Self::DoubleLottery { bindings, records } => {
                output.push(TAG_DOUBLE_LOTTERY);
                encode_bindings(output, bindings);
                encode_records(output, records)?;
            }
            Self::EnhanceAll {
                inventory,
                candidates,
            } => {
                output.push(TAG_ENHANCE_ALL);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                encode_ids(output, candidates)?;
            }
            Self::PerfectChallenge {
                bindings,
                effect_slot,
                action,
                expected_stage,
                next_stage,
                repeat,
                positive,
                negative,
            } => {
                output.push(TAG_PERFECT_CHALLENGE);
                encode_bindings(output, bindings);
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                output.push(action);
                output.extend_from_slice(&expected_stage.to_le_bytes());
                output.extend_from_slice(&next_stage.to_le_bytes());
                output.push(u8::from(repeat));
                encode_records(output, positive)?;
                encode_records(output, negative)?;
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::DoubleLottery { records, .. } => u32::try_from(records.len())
                .ok()
                .and_then(|count| count.checked_mul(count)),
            Self::PerfectChallenge {
                action: PERFECT_CLAY,
                positive,
                negative,
                ..
            } => checked_lcm(
                u32::try_from(positive.len()).ok()?,
                u32::try_from(negative.len()).ok()?,
            )?
            .checked_mul(2),
            Self::PerfectChallenge {
                action: PERFECT_POPULAR,
                positive,
                ..
            } => u32::try_from(positive.len()).ok()?.checked_mul(5),
            Self::RepairDestroyed { .. }
            | Self::EnhanceAll { .. }
            | Self::PerfectChallenge { .. } => None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn externalize(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
) -> Result<Option<external::Lowering>, OccurrenceInteractionError> {
    if !has_marker(outcome, "repair-one-destroyed") {
        return Ok(None);
    }
    let cost = literal(outcome, 0)?;
    let choices = records
        .iter()
        .copied()
        .map(|record| external::Choice {
            content: u64::from(record.id().get()),
            operations: vec![PayloadOperation::S09(Operation::RepairDestroyed {
                bindings,
                cost,
                records: vec![record],
            })],
            random_candidate_count: None,
        })
        .collect();
    if catalog.curios().len() != records.len() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(Some(external::Lowering {
        choices,
        repeat_key: None,
    }))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    selected_path: ActivitySlotId,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
    effect_slot: ActivitySlotId,
) -> Result<Option<Lowering>, OccurrenceInteractionError> {
    let Some(kind) = marker(outcome) else {
        return Ok(None);
    };
    let positive = || curio_pool(catalog, records, "polarity:positive");
    let negative = || curio_pool(catalog, records, "polarity:negative");
    let all_blessings = || {
        catalog
            .blessings()
            .iter()
            .map(|blessing| u64::from(blessing.id().get()))
            .collect::<Vec<_>>()
    };
    let normal_curios = || {
        let candidates = positive()?;
        Ok(PayloadOperation::CurioInventory {
            bindings,
            delta: 1,
            quantity: 10,
            owned_only: false,
            candidates,
        })
    };
    let lowering = match kind {
        "repair-one-destroyed" => return Ok(None),
        "repair-all-destroyed" => lowering(Operation::RepairDestroyed {
            bindings,
            cost: literal(outcome, 0)?,
            records: records.to_vec(),
        }),
        "showman-two-to-four" => Lowering {
            operations: vec![PayloadOperation::S07(s07::Operation::BlessingExchange {
                inventory: blessing_inventory,
                path_slot: selected_path,
                lose_quantity: 2,
                gain_quantity: 4,
                lose_candidates: blessings(catalog, 2),
                gain_groups: vec![(0, blessings(catalog, 2))],
            })],
            repeat_key: None,
        },
        "showman-two-to-two-three" => Lowering {
            operations: vec![PayloadOperation::S07(s07::Operation::BlessingExchange {
                inventory: blessing_inventory,
                path_slot: selected_path,
                lose_quantity: 2,
                gain_quantity: 2,
                lose_candidates: blessings(catalog, 2),
                gain_groups: vec![(0, blessings(catalog, 3))],
            })],
            repeat_key: None,
        },
        "double-lottery-buy" => lowering(Operation::DoubleLottery {
            bindings,
            records: referenced_records(outcome, catalog, records)?,
        }),
        "double-lottery-repair" => lowering(Operation::RepairDestroyed {
            bindings,
            cost: literal(outcome, 0)?,
            records: referenced_records(outcome, catalog, records)?,
        }),
        "ruan-enhance-all" => lowering(Operation::EnhanceAll {
            inventory: blessing_inventory,
            candidates: all_blessings(),
        }),
        "ruan-curios-ten" => Lowering {
            operations: vec![normal_curios()?],
            repeat_key: None,
        },
        "ruan-both" => Lowering {
            operations: vec![
                PayloadOperation::S09(Operation::EnhanceAll {
                    inventory: blessing_inventory,
                    candidates: all_blessings(),
                }),
                normal_curios()?,
            ],
            repeat_key: None,
        },
        value if value.starts_with("perfect-") => {
            perfect(value, bindings, effect_slot, positive()?, negative()?)?
        }
        _ => return Err(OccurrenceInteractionError::InvalidChoice),
    };
    Ok(Some(lowering))
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_REPAIR_DESTROYED => decode_repair(input, decoder, operations)?,
        TAG_DOUBLE_LOTTERY => decode_lottery(input, decoder, operations)?,
        TAG_ENHANCE_ALL => decode_enhance_all(input, decoder, operations)?,
        TAG_PERFECT_CHALLENGE => decode_perfect(input, decoder, operations)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_repair(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let cost = decoder.i64()?;
    let records = decode_records(decoder)?;
    if cost <= 0 || records.is_empty() {
        return Err(invalid_payload());
    }
    operations.push(ActivityOperation::Require(ActivityCondition::Any(
        records
            .iter()
            .map(|record| destroyed_available(record.id(), bindings))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )));
    spend_fragments(input, operations, bindings, cost)?;
    for record in records {
        operations.push(conditional_restore(record, bindings));
    }
    Ok(())
}

fn decode_lottery(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let records = decode_records(decoder)?;
    let random = usize::try_from(input.random_index().ok_or_else(invalid_state)?)
        .map_err(|_| invalid_state())?;
    if records.len() != 2 {
        return Err(invalid_payload());
    }
    spend_fragments(input, operations, bindings, 100)?;
    let repair_first = random % 2;
    let repair_second = 1 - repair_first;
    operations.push(ActivityOperation::Require(ActivityCondition::Any(
        records
            .iter()
            .map(|record| destroyed_available(record.id(), bindings))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )));
    operations.push(ActivityOperation::Conditional {
        condition: destroyed_available(records[repair_first].id(), bindings),
        if_true: restore_destroyed_operations(records[repair_first], bindings).into_boxed_slice(),
        if_false: restore_destroyed_operations(records[repair_second], bindings).into_boxed_slice(),
    });
    let acquire_first = (random / 2) % 2;
    let acquire_second = 1 - acquire_first;
    operations.push(ActivityOperation::Require(ActivityCondition::Any(
        records
            .iter()
            .map(|record| unowned(record.id(), bindings.inventory))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )));
    operations.push(ActivityOperation::Conditional {
        condition: unowned(records[acquire_first].id(), bindings.inventory),
        if_true: acquisition_operations(records[acquire_first], bindings).into_boxed_slice(),
        if_false: acquisition_operations(records[acquire_second], bindings).into_boxed_slice(),
    });
    Ok(())
}

fn decode_enhance_all(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let candidates = decode_ids(decoder)?;
    for content in candidates {
        operations.push(ActivityOperation::Conditional {
            condition: ActivityCondition::Equal(
                ActivityExpression::InventoryCount { inventory, content },
                integer(1),
            ),
            if_true: vec![ActivityOperation::AddInventory {
                inventory,
                content,
                count: integer(1),
            }]
            .into_boxed_slice(),
            if_false: Box::new([]),
        });
    }
    let _ = input;
    Ok(())
}

fn decode_perfect(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let effect_slot = slot(decoder.u32()?)?;
    let action = decoder.u8()?;
    let expected = decoder.i64()?;
    let next = decoder.i64()?;
    let repeat = decoder.u8()? != 0;
    let positive = decode_records(decoder)?;
    let negative = decode_records(decoder)?;
    if counter_value(input, effect_slot, PERFECT_STAGE_KEY)? != expected {
        return Err(invalid_state());
    }
    match action {
        PERFECT_PAY => spend_fragments(input, operations, bindings, 40)?,
        PERFECT_LEAVE => {}
        PERFECT_CLAY | PERFECT_POPULAR => {
            perfect_reward(input, operations, bindings, action, &positive, &negative)?
        }
        _ => return Err(invalid_payload()),
    }
    if expected != next {
        operations.push(add_counter(
            effect_slot,
            PERFECT_STAGE_KEY,
            next.checked_sub(expected).ok_or_else(invalid_payload)?,
        ));
    }
    if repeat {
        operations.push(add_counter(effect_slot, PERFECT_REPEAT_KEY, 1));
    }
    Ok(())
}

fn perfect_reward(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    bindings: CurioActivityBindings,
    action: u8,
    positive: &[CurioActivityRecord],
    negative: &[CurioActivityRecord],
) -> Result<(), ActivityHandlerFault> {
    let random = usize::try_from(input.random_index().ok_or_else(invalid_state)?)
        .map_err(|_| invalid_state())?;
    let candidates = match action {
        PERFECT_CLAY if random % 2 == 0 => positive,
        PERFECT_CLAY => negative,
        PERFECT_POPULAR if random % 5 < 2 => positive,
        PERFECT_POPULAR => return Ok(()),
        _ => return Err(invalid_payload()),
    };
    let eligible = candidates
        .iter()
        .copied()
        .filter(|record| {
            inventory_count(input, bindings.inventory, u64::from(record.id().get())) == Some(0)
        })
        .collect::<Vec<_>>();
    let selected = eligible
        .get((random / if action == PERFECT_CLAY { 2 } else { 5 }) % eligible.len().max(1))
        .copied()
        .ok_or_else(invalid_state)?;
    operations.extend(acquisition_operations(selected, bindings));
    Ok(())
}

fn perfect(
    kind: &str,
    bindings: CurioActivityBindings,
    effect_slot: ActivitySlotId,
    positive: Vec<CurioActivityRecord>,
    negative: Vec<CurioActivityRecord>,
) -> Result<Lowering, OccurrenceInteractionError> {
    let (action, expected, next, repeat) = match kind {
        "perfect-pay-first" => (PERFECT_PAY, 0, 1, true),
        "perfect-leave-first" => (PERFECT_LEAVE, 0, 0, false),
        "perfect-clay-first" => (PERFECT_CLAY, 1, 2, true),
        "perfect-pay-second" => (PERFECT_PAY, 2, 3, true),
        "perfect-leave-second" => (PERFECT_LEAVE, 2, 0, false),
        "perfect-clay-second" => (PERFECT_CLAY, 3, 4, true),
        "perfect-popular-second" => (PERFECT_POPULAR, 3, 4, true),
        "perfect-pay-third" => (PERFECT_PAY, 4, 5, true),
        "perfect-leave-third" => (PERFECT_LEAVE, 4, 0, false),
        "perfect-clay-third" => (PERFECT_CLAY, 5, 0, false),
        "perfect-popular-third" => (PERFECT_POPULAR, 5, 0, false),
        _ => return Err(OccurrenceInteractionError::InvalidChoice),
    };
    let operation = Operation::PerfectChallenge {
        bindings,
        effect_slot,
        action,
        expected_stage: expected,
        next_stage: next,
        repeat,
        positive,
        negative,
    };
    Ok(Lowering {
        repeat_key: repeat.then_some(PERFECT_REPEAT_KEY),
        operations: vec![PayloadOperation::S09(operation)],
    })
}

fn lowering(operation: Operation) -> Lowering {
    Lowering {
        operations: vec![PayloadOperation::S09(operation)],
        repeat_key: None,
    }
}

fn spend_fragments(
    _input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    bindings: CurioActivityBindings,
    amount: i64,
) -> Result<(), ActivityHandlerFault> {
    let required = u64::try_from(amount).map_err(|_| invalid_payload())?;
    operations.push(require_at_least(bindings.fragments_slot, required)?);
    operations.push(fragment_delta(
        bindings.fragments_slot,
        bindings.inventory,
        amount.checked_neg().ok_or_else(invalid_payload)?,
    ));
    Ok(())
}

fn conditional_restore(
    record: CurioActivityRecord,
    bindings: CurioActivityBindings,
) -> ActivityOperation {
    ActivityOperation::Conditional {
        condition: destroyed_available(record.id(), bindings),
        if_true: restore_destroyed_operations(record, bindings).into_boxed_slice(),
        if_false: Box::new([]),
    }
}

fn unowned(id: CurioId, inventory: ActivityInventoryId) -> ActivityCondition {
    ActivityCondition::LessThan(
        ActivityExpression::InventoryCount {
            inventory,
            content: u64::from(id.get()),
        },
        integer(1),
    )
}

fn curio_pool(
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
    tag: &str,
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    let mut selected = catalog
        .curios()
        .iter()
        .filter(|curio| curio.pool_tags().iter().any(|value| value.as_ref() == tag))
        .map(|curio| {
            records
                .iter()
                .copied()
                .find(|record| record.id() == curio.id())
                .ok_or(OccurrenceInteractionError::InvalidChoice)
        })
        .collect::<Result<Vec<_>, _>>()?;
    selected.sort_unstable_by_key(|record| record.id());
    if selected.is_empty() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(selected)
}

fn referenced_records(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    let mut selected = outcome
        .parameter_refs()
        .iter()
        .filter_map(|reference| {
            catalog
                .curios()
                .iter()
                .find(|curio| curio.stable_key() == reference.as_ref())
        })
        .map(|curio| {
            records
                .iter()
                .copied()
                .find(|record| record.id() == curio.id())
                .ok_or(OccurrenceInteractionError::InvalidChoice)
        })
        .collect::<Result<Vec<_>, _>>()?;
    selected.sort_unstable_by_key(|record| record.id());
    selected.dedup_by_key(|record| record.id());
    if selected.is_empty() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(selected)
}

fn blessings(catalog: &UniverseCatalog, rarity: u8) -> Vec<u64> {
    catalog
        .blessings()
        .iter()
        .filter(|blessing| blessing.rarity() == rarity)
        .map(|blessing| u64::from(blessing.id().get()))
        .collect()
}

fn marker(outcome: &OccurrenceOutcome) -> Option<&str> {
    outcome
        .parameter_refs()
        .iter()
        .find_map(|value| value.strip_prefix(PREFIX))
}

fn has_marker(outcome: &OccurrenceOutcome, value: &str) -> bool {
    marker(outcome) == Some(value)
}

fn literal(outcome: &OccurrenceOutcome, index: usize) -> Result<i64, OccurrenceInteractionError> {
    outcome
        .numeric_literals()
        .get(index)
        .copied()
        .map(exact_integer)
        .transpose()?
        .filter(|value| *value > 0)
        .ok_or(OccurrenceInteractionError::InvalidChoice)
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
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let record = CurioActivityRecord::new(
            CurioId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
            CurioStateId::new(decoder.u32()?).ok_or_else(invalid_payload)?,
            decoder.u8()?,
            match decoder.i64()? {
                0 => None,
                value => Some(value),
            },
        );
        records.push(match decoder.i64()? {
            0 => record,
            value => record.with_fragment_stack_capture(value),
        });
    }
    if records.windows(2).any(|pair| pair[0].id() >= pair[1].id()) {
        return Err(invalid_payload());
    }
    Ok(records)
}

fn encode_ids(output: &mut Vec<u8>, ids: Vec<u64>) -> Result<(), OccurrenceInteractionError> {
    output.extend_from_slice(
        &u16::try_from(ids.len())
            .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
            .to_le_bytes(),
    );
    for id in ids {
        output.extend_from_slice(&id.to_le_bytes());
    }
    Ok(())
}

fn decode_ids(decoder: &mut Decoder<'_>) -> Result<Vec<u64>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    let mut ids = Vec::with_capacity(count);
    for _ in 0..count {
        ids.push(decoder.u64()?);
    }
    if ids.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid_payload());
    }
    Ok(ids)
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
