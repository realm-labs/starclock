//! Shared primitives introduced by Goal 07 Occurrence partition S05.

use super::*;

const TAG_ENHANCE_BEST_INVENTORY_GROUP: u8 = 11;
const TAG_PROGRESSIVE_INVENTORY_DRAW: u8 = 12;
const TAG_RESET_PROGRESSIVE_DRAW: u8 = 13;
const PROGRESSIVE_MARKER_PREFIX: &str = "universe.occurrence-progressive.key.";
const PROGRESSIVE_ATTEMPT_KEY_BASE: u64 = 0x5000_0000_0000_0000;
const PROGRESSIVE_REPEAT_KEY_BASE: u64 = 0x5100_0000_0000_0000;

#[derive(Clone)]
pub(super) enum Operation {
    EnhanceBestInventoryGroup {
        inventory: ActivityInventoryId,
        quantity: u16,
        groups: Vec<Vec<u64>>,
    },
    ProgressiveInventoryDraw {
        inventory: ActivityInventoryId,
        effect_slot: ActivitySlotId,
        key: u64,
        candidates: Vec<u64>,
        chances: Vec<[u8; 3]>,
    },
    ResetProgressiveDraw {
        effect_slot: ActivitySlotId,
        key: u64,
    },
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::EnhanceBestInventoryGroup {
                inventory,
                quantity,
                groups,
            } => {
                output.push(TAG_ENHANCE_BEST_INVENTORY_GROUP);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(&quantity.to_le_bytes());
                encode_groups(output, groups)?;
            }
            Self::ProgressiveInventoryDraw {
                inventory,
                effect_slot,
                key,
                candidates,
                chances,
            } => {
                output.push(TAG_PROGRESSIVE_INVENTORY_DRAW);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                output.extend_from_slice(&key.to_le_bytes());
                output.push(
                    u8::try_from(chances.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?,
                );
                for row in chances {
                    output.extend_from_slice(&row);
                }
                output.extend_from_slice(
                    &u16::try_from(candidates.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for candidate in candidates {
                    output.extend_from_slice(&candidate.to_le_bytes());
                }
            }
            Self::ResetProgressiveDraw { effect_slot, key } => {
                output.push(TAG_RESET_PROGRESSIVE_DRAW);
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                output.extend_from_slice(&key.to_le_bytes());
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::EnhanceBestInventoryGroup { groups, .. } => groups
                .iter()
                .filter_map(|group| u32::try_from(group.len()).ok())
                .try_fold(1_u32, checked_lcm),
            Self::ProgressiveInventoryDraw { candidates, .. } => {
                u32::try_from(candidates.len()).ok()?.checked_mul(100)
            }
            Self::ResetProgressiveDraw { .. } => None,
        }
    }
}

pub(super) fn referenced_blessing_groups(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
) -> Result<Vec<Vec<u64>>, OccurrenceInteractionError> {
    let mut groups = s02::referenced_blessing_groups(outcome, catalog)?;
    let rarity = outcome.parameter_refs().iter().find_map(|reference| {
        reference
            .strip_prefix("universe.blessing-pool.rarity.")
            .and_then(|value| value.parse::<u8>().ok())
    });
    if let Some(rarity) = rarity {
        let eligible = catalog
            .blessings()
            .iter()
            .filter(|blessing| blessing.rarity() == rarity)
            .map(|blessing| u64::from(blessing.id().get()))
            .collect::<std::collections::BTreeSet<_>>();
        for group in &mut groups {
            group.retain(|candidate| eligible.contains(candidate));
            if group.is_empty() {
                return Err(OccurrenceInteractionError::InvalidChoice);
            }
        }
    }
    Ok(groups)
}

pub(super) fn lower_progressive(
    outcome: &OccurrenceOutcome,
    blessing_inventory: ActivityInventoryId,
    effect_slot: ActivitySlotId,
    blessing_ids: &[u64],
) -> Result<Option<(Operation, u64)>, OccurrenceInteractionError> {
    let Some(key) = progressive_key(outcome)? else {
        return Ok(None);
    };
    if outcome.chance_percentages().is_empty() {
        return Ok(None);
    }
    if outcome.operations()
        != [
            OccurrenceOperation::Obtain,
            OccurrenceOperation::Battle,
            OccurrenceOperation::Special,
        ]
        || outcome.targets() != [OccurrenceTarget::Blessing]
        || !outcome.chance_percentages().len().is_multiple_of(3)
    {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    let chances = outcome
        .chance_percentages()
        .chunks_exact(3)
        .map(|row| {
            let row = row
                .iter()
                .copied()
                .map(exact_percentage)
                .collect::<Result<Vec<_>, _>>()?;
            let row: [u8; 3] = row
                .try_into()
                .map_err(|_| OccurrenceInteractionError::InvalidChoice)?;
            if row.iter().map(|value| u16::from(*value)).sum::<u16>() != 100 {
                return Err(OccurrenceInteractionError::InvalidChoice);
            }
            Ok(row)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if chances.is_empty() || chances.len() > 8 || blessing_ids.is_empty() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(Some((
        Operation::ProgressiveInventoryDraw {
            inventory: blessing_inventory,
            effect_slot,
            key,
            candidates: blessing_ids.to_vec(),
            chances,
        },
        repeat_key(key),
    )))
}

pub(super) fn reset_progressive(
    outcome: &OccurrenceOutcome,
    effect_slot: ActivitySlotId,
) -> Result<Option<Operation>, OccurrenceInteractionError> {
    if !outcome.chance_percentages().is_empty() {
        return Ok(None);
    }
    Ok(progressive_key(outcome)?.map(|key| Operation::ResetProgressiveDraw { effect_slot, key }))
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_ENHANCE_BEST_INVENTORY_GROUP => {
            decode_enhance_best_group(input, decoder, operations)?;
        }
        TAG_PROGRESSIVE_INVENTORY_DRAW => {
            decode_progressive_draw(input, decoder, operations)?;
        }
        TAG_RESET_PROGRESSIVE_DRAW => {
            let slot = slot(decoder.u32()?)?;
            let key = decoder.u64()?;
            operations.push(reset_counter(slot, attempt_key(key)));
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_enhance_best_group(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let quantity = usize::from(decoder.u16()?);
    let groups = decode_groups(decoder)?;
    if quantity == 0 {
        return Err(invalid_payload());
    }
    let entries = inventory_entries(input, inventory)?;
    let selected = groups
        .iter()
        .max_by_key(|group| {
            group
                .iter()
                .filter(|candidate| inventory_count(entries, **candidate) > 0)
                .count()
        })
        .ok_or_else(invalid_payload)?;
    let eligible = selected
        .iter()
        .copied()
        .filter(|candidate| inventory_count(entries, *candidate) == 1)
        .collect::<Vec<_>>();
    if eligible.len() < quantity {
        return Err(invalid_state());
    }
    let start = input
        .random_index()
        .map_or(0, |index| index as usize % eligible.len());
    for offset in 0..quantity {
        operations.push(ActivityOperation::AddInventory {
            inventory,
            content: eligible[(start + offset) % eligible.len()],
            count: integer(1),
        });
    }
    Ok(())
}

fn decode_progressive_draw(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let effect_slot = slot(decoder.u32()?)?;
    let key = decoder.u64()?;
    let row_count = usize::from(decoder.u8()?);
    if row_count == 0 {
        return Err(invalid_payload());
    }
    let mut chances = Vec::with_capacity(row_count);
    for _ in 0..row_count {
        let row = [decoder.u8()?, decoder.u8()?, decoder.u8()?];
        if row.iter().map(|value| u16::from(*value)).sum::<u16>() != 100 {
            return Err(invalid_payload());
        }
        chances.push(row);
    }
    let candidate_count = usize::from(decoder.u16()?);
    if candidate_count == 0 {
        return Err(invalid_payload());
    }
    let mut candidates = Vec::with_capacity(candidate_count);
    for _ in 0..candidate_count {
        candidates.push(decoder.u64()?);
    }
    let random = input.random_index().ok_or_else(invalid_state)?;
    let attempt_value = counter_value(input, effect_slot, attempt_key(key))?;
    let attempt = usize::try_from(attempt_value)
        .map_err(|_| invalid_state())?
        .min(chances.len() - 1);
    let row = chances[attempt];
    let roll = u8::try_from(random % 100).expect("modulo fits u8");
    if roll < row[0] {
        let entries = inventory_entries(input, inventory)?;
        let eligible = candidates
            .iter()
            .copied()
            .filter(|candidate| inventory_count(entries, *candidate) == 0)
            .collect::<Vec<_>>();
        if eligible.is_empty() {
            return Err(invalid_state());
        }
        let selection = usize::try_from(random / 100).map_err(|_| invalid_state())?;
        operations.push(ActivityOperation::AddInventory {
            inventory,
            content: eligible[selection % eligible.len()],
            count: integer(1),
        });
        operations.push(increment_counter(effect_slot, attempt_key(key)));
        operations.push(increment_counter(effect_slot, repeat_key(key)));
    } else if roll < row[0].saturating_add(row[1]) {
        operations.push(reset_counter(effect_slot, attempt_key(key)));
    } else {
        operations.push(increment_counter(effect_slot, attempt_key(key)));
        operations.push(increment_counter(effect_slot, repeat_key(key)));
    }
    Ok(())
}

fn progressive_key(outcome: &OccurrenceOutcome) -> Result<Option<u64>, OccurrenceInteractionError> {
    let keys = outcome
        .parameter_refs()
        .iter()
        .filter_map(|reference| reference.strip_prefix(PROGRESSIVE_MARKER_PREFIX))
        .map(|value| {
            value
                .parse::<u64>()
                .ok()
                .filter(|value| *value != 0)
                .ok_or(OccurrenceInteractionError::InvalidChoice)
        })
        .collect::<Result<Vec<_>, _>>()?;
    match keys.as_slice() {
        [] => Ok(None),
        [key] => Ok(Some(*key)),
        _ => Err(OccurrenceInteractionError::InvalidChoice),
    }
}

const fn attempt_key(key: u64) -> u64 {
    PROGRESSIVE_ATTEMPT_KEY_BASE | key
}

pub(super) const fn repeat_key(key: u64) -> u64 {
    PROGRESSIVE_REPEAT_KEY_BASE | key
}

fn exact_percentage(value: crate::path::ExactParameter) -> Result<u8, OccurrenceInteractionError> {
    let divisor = 10_i64
        .checked_pow(u32::from(value.scale()))
        .ok_or(OccurrenceInteractionError::Arithmetic)?;
    if value.coefficient() % divisor != 0 {
        return Err(OccurrenceInteractionError::NonIntegerScalar);
    }
    u8::try_from(value.coefficient() / divisor)
        .ok()
        .filter(|value| *value <= 100)
        .ok_or(OccurrenceInteractionError::InvalidChoice)
}

fn encode_groups(
    output: &mut Vec<u8>,
    groups: Vec<Vec<u64>>,
) -> Result<(), OccurrenceInteractionError> {
    output.extend_from_slice(
        &u16::try_from(groups.len())
            .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
            .to_le_bytes(),
    );
    for group in groups {
        output.extend_from_slice(
            &u16::try_from(group.len())
                .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                .to_le_bytes(),
        );
        for content in group {
            output.extend_from_slice(&content.to_le_bytes());
        }
    }
    Ok(())
}

fn decode_groups(decoder: &mut Decoder<'_>) -> Result<Vec<Vec<u64>>, ActivityHandlerFault> {
    let group_count = usize::from(decoder.u16()?);
    if group_count == 0 {
        return Err(invalid_payload());
    }
    let mut groups = Vec::with_capacity(group_count);
    for _ in 0..group_count {
        let member_count = usize::from(decoder.u16()?);
        if member_count == 0 {
            return Err(invalid_payload());
        }
        let mut group = Vec::with_capacity(member_count);
        for _ in 0..member_count {
            group.push(decoder.u64()?);
        }
        groups.push(group);
    }
    Ok(groups)
}

fn inventory_entries(
    input: ActivityHandlerInput<'_>,
    inventory: ActivityInventoryId,
) -> Result<&[(u64, u32)], ActivityHandlerFault> {
    input
        .view()
        .inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .map(|value| value.entries())
        .ok_or_else(invalid_state)
}

fn inventory_count(entries: &[(u64, u32)], content: u64) -> u32 {
    entries
        .iter()
        .find(|entry| entry.0 == content)
        .map_or(0, |entry| entry.1)
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

fn increment_counter(slot: ActivitySlotId, key: u64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot,
        key,
        delta: integer(1),
    }
}

fn reset_counter(slot: ActivitySlotId, key: u64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot,
        key,
        delta: ActivityExpression::Negate(Box::new(ActivityExpression::CounterValue { slot, key })),
    }
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
