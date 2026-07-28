//! Shared primitives introduced by Goal 07 Occurrence partition S06.

use super::*;

const TAG_PROGRESSIVE_CURIO_DRAW: u8 = 14;
const TAG_PROGRESSIVE_FRAGMENT_DRAW: u8 = 15;
const TAG_PREPARE_BATTLE_REWARD_PATHS: u8 = 16;
const PROGRESSIVE_MARKER_PREFIX: &str = "universe.occurrence-progressive.key.";
const BATTLE_REWARD_PATHS_MARKER: &str = "universe.occurrence-battle.reward.paths";
const PROGRESSIVE_ATTEMPT_KEY_BASE: u64 = 0x5000_0000_0000_0000;
const PROGRESSIVE_REPEAT_KEY_BASE: u64 = 0x5100_0000_0000_0000;

#[derive(Clone)]
pub(super) enum Operation {
    ProgressiveCurioDraw {
        bindings: CurioActivityBindings,
        effect_slot: ActivitySlotId,
        key: u64,
        candidates: Vec<CurioActivityRecord>,
        chances: Vec<[u8; 3]>,
    },
    ProgressiveFragmentDraw {
        fragments_slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        effect_slot: ActivitySlotId,
        key: u64,
        amount: i64,
        chances: Vec<[u8; 3]>,
    },
    PrepareBattleRewardPaths {
        effect_slot: ActivitySlotId,
        all_paths: Vec<u64>,
        reward_paths: Vec<u64>,
    },
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::ProgressiveCurioDraw {
                bindings,
                effect_slot,
                key,
                candidates,
                chances,
            } => {
                output.push(TAG_PROGRESSIVE_CURIO_DRAW);
                encode_bindings(output, bindings);
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                output.extend_from_slice(&key.to_le_bytes());
                encode_chances(output, &chances)?;
                output.extend_from_slice(
                    &u16::try_from(candidates.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for candidate in candidates {
                    encode_curio(output, candidate);
                }
            }
            Self::ProgressiveFragmentDraw {
                fragments_slot,
                gain_inventory,
                effect_slot,
                key,
                amount,
                chances,
            } => {
                output.push(TAG_PROGRESSIVE_FRAGMENT_DRAW);
                output.extend_from_slice(&fragments_slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                output.extend_from_slice(&key.to_le_bytes());
                output.extend_from_slice(&amount.to_le_bytes());
                encode_chances(output, &chances)?;
            }
            Self::PrepareBattleRewardPaths {
                effect_slot,
                all_paths,
                reward_paths,
            } => {
                output.push(TAG_PREPARE_BATTLE_REWARD_PATHS);
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                encode_ids(output, &all_paths)?;
                encode_ids(output, &reward_paths)?;
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::ProgressiveCurioDraw { candidates, .. } => {
                u32::try_from(candidates.len()).ok()?.checked_mul(100)
            }
            Self::ProgressiveFragmentDraw { .. } => Some(100),
            Self::PrepareBattleRewardPaths { .. } => None,
        }
    }
}

pub(super) fn lower_progressive(
    outcome: &OccurrenceOutcome,
    cosmic_fragments: ActivitySlotId,
    curio_bindings: CurioActivityBindings,
    effect_slot: ActivitySlotId,
    curios: &[CurioActivityRecord],
) -> Result<Option<(Operation, u64)>, OccurrenceInteractionError> {
    let Some(key) = progressive_key(outcome)? else {
        return Ok(None);
    };
    if outcome.chance_percentages().is_empty() {
        return Ok(None);
    }
    let chances = chances(outcome)?;
    let operation = match outcome.targets() {
        [OccurrenceTarget::Curio]
            if outcome.operations()
                == [
                    OccurrenceOperation::Obtain,
                    OccurrenceOperation::Battle,
                    OccurrenceOperation::Special,
                ]
                && !curios.is_empty() =>
        {
            Operation::ProgressiveCurioDraw {
                bindings: curio_bindings,
                effect_slot,
                key,
                candidates: curios.to_vec(),
                chances,
            }
        }
        [OccurrenceTarget::CosmicFragments]
            if outcome.operations()
                == [
                    OccurrenceOperation::Obtain,
                    OccurrenceOperation::Battle,
                    OccurrenceOperation::Special,
                ] =>
        {
            let amount = outcome
                .numeric_literals()
                .first()
                .copied()
                .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
                .map(exact_integer)
                .transpose()?
                .filter(|value| *value > 0)
                .ok_or(OccurrenceInteractionError::InvalidChoice)?;
            Operation::ProgressiveFragmentDraw {
                fragments_slot: cosmic_fragments,
                gain_inventory: curio_bindings.inventory,
                effect_slot,
                key,
                amount,
                chances,
            }
        }
        _ => return Ok(None),
    };
    Ok(Some((operation, repeat_key(key))))
}

pub(super) fn prepare_reward_paths(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    effect_slot: ActivitySlotId,
) -> Result<Option<Operation>, OccurrenceInteractionError> {
    if !outcome
        .parameter_refs()
        .iter()
        .any(|value| value.as_ref() == BATTLE_REWARD_PATHS_MARKER)
    {
        return Ok(None);
    }
    let all_paths = catalog
        .paths()
        .iter()
        .map(|path| u64::from(path.id().get()))
        .collect::<Vec<_>>();
    let mut reward_paths = outcome
        .parameter_refs()
        .iter()
        .filter_map(|reference| {
            catalog
                .paths()
                .iter()
                .find(|path| path.stable_key() == reference.as_ref())
                .map(|path| u64::from(path.id().get()))
        })
        .collect::<Vec<_>>();
    reward_paths.sort_unstable();
    reward_paths.dedup();
    if all_paths.is_empty() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(Some(Operation::PrepareBattleRewardPaths {
        effect_slot,
        all_paths,
        reward_paths,
    }))
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_PROGRESSIVE_CURIO_DRAW => decode_curio_draw(input, decoder, operations)?,
        TAG_PROGRESSIVE_FRAGMENT_DRAW => decode_fragment_draw(input, decoder, operations)?,
        TAG_PREPARE_BATTLE_REWARD_PATHS => decode_reward_paths(decoder, operations)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_curio_draw(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let effect_slot = slot(decoder.u32()?)?;
    let key = decoder.u64()?;
    let chances = decode_chances(decoder)?;
    let count = usize::from(decoder.u16()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    let mut candidates = Vec::with_capacity(count);
    for _ in 0..count {
        candidates.push(decode_curio(decoder)?);
    }
    let random = input.random_index().ok_or_else(invalid_state)?;
    match draw_result(input, effect_slot, key, &chances, random)? {
        DrawResult::Reward => {
            let eligible = candidates
                .iter()
                .copied()
                .filter(|candidate| {
                    inventory_count(input, bindings.inventory, u64::from(candidate.id().get()))
                        == Some(0)
                })
                .collect::<Vec<_>>();
            if eligible.is_empty() {
                return Err(invalid_state());
            }
            let selection = usize::try_from(random / 100).map_err(|_| invalid_state())?;
            operations.extend(acquisition_operations(
                eligible[selection % eligible.len()],
                bindings,
            ));
            advance(operations, effect_slot, key);
        }
        DrawResult::Battle => reset(operations, effect_slot, key),
        DrawResult::Blank => advance(operations, effect_slot, key),
    }
    Ok(())
}

fn decode_fragment_draw(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let fragments_slot = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let effect_slot = slot(decoder.u32()?)?;
    let key = decoder.u64()?;
    let amount = decoder.i64()?;
    if amount <= 0 {
        return Err(invalid_payload());
    }
    let chances = decode_chances(decoder)?;
    let random = input.random_index().ok_or_else(invalid_state)?;
    match draw_result(input, effect_slot, key, &chances, random)? {
        DrawResult::Reward => {
            operations.push(fragment_delta(fragments_slot, gain_inventory, amount));
            advance(operations, effect_slot, key);
        }
        DrawResult::Battle => reset(operations, effect_slot, key),
        DrawResult::Blank => advance(operations, effect_slot, key),
    }
    Ok(())
}

fn decode_reward_paths(
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let effect_slot = slot(decoder.u32()?)?;
    let all_paths = decode_ids(decoder)?;
    let reward_paths = decode_ids(decoder)?;
    if all_paths.is_empty() || reward_paths.iter().any(|path| !all_paths.contains(path)) {
        return Err(invalid_payload());
    }
    for path in all_paths {
        operations.push(reset_counter(effect_slot, path));
    }
    for path in reward_paths {
        operations.push(ActivityOperation::AddCounter {
            slot: effect_slot,
            key: path,
            delta: integer(1),
        });
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum DrawResult {
    Reward,
    Battle,
    Blank,
}

fn draw_result(
    input: ActivityHandlerInput<'_>,
    effect_slot: ActivitySlotId,
    key: u64,
    chances: &[[u8; 3]],
    random: u32,
) -> Result<DrawResult, ActivityHandlerFault> {
    let attempt = usize::try_from(counter_value(input, effect_slot, attempt_key(key))?)
        .map_err(|_| invalid_state())?
        .min(chances.len() - 1);
    let row = chances[attempt];
    let roll = u8::try_from(random % 100).expect("modulo fits u8");
    Ok(if roll < row[0] {
        DrawResult::Reward
    } else if roll < row[0].saturating_add(row[1]) {
        DrawResult::Battle
    } else {
        DrawResult::Blank
    })
}

fn chances(outcome: &OccurrenceOutcome) -> Result<Vec<[u8; 3]>, OccurrenceInteractionError> {
    if !outcome.chance_percentages().len().is_multiple_of(3) {
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
            (row.iter().map(|value| u16::from(*value)).sum::<u16>() == 100)
                .then_some(row)
                .ok_or(OccurrenceInteractionError::InvalidChoice)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if chances.is_empty() || chances.len() > 8 {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(chances)
}

fn encode_chances(
    output: &mut Vec<u8>,
    chances: &[[u8; 3]],
) -> Result<(), OccurrenceInteractionError> {
    output.push(
        u8::try_from(chances.len()).map_err(|_| OccurrenceInteractionError::TooManyCandidates)?,
    );
    for row in chances {
        output.extend_from_slice(row);
    }
    Ok(())
}

fn decode_chances(decoder: &mut Decoder<'_>) -> Result<Vec<[u8; 3]>, ActivityHandlerFault> {
    let count = usize::from(decoder.u8()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    let mut chances = Vec::with_capacity(count);
    for _ in 0..count {
        let row = [decoder.u8()?, decoder.u8()?, decoder.u8()?];
        if row.iter().map(|value| u16::from(*value)).sum::<u16>() != 100 {
            return Err(invalid_payload());
        }
        chances.push(row);
    }
    Ok(chances)
}

fn encode_bindings(output: &mut Vec<u8>, bindings: CurioActivityBindings) {
    output.extend_from_slice(&bindings.inventory.get().to_le_bytes());
    output.extend_from_slice(&bindings.state_slot.get().to_le_bytes());
    output.extend_from_slice(&bindings.charge_slot.get().to_le_bytes());
    output.extend_from_slice(&bindings.event_slot.get().to_le_bytes());
    output.extend_from_slice(&bindings.fragments_slot.get().to_le_bytes());
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

fn encode_curio(output: &mut Vec<u8>, candidate: CurioActivityRecord) {
    output.extend_from_slice(&candidate.id().get().to_le_bytes());
    output.extend_from_slice(&candidate.initial_state().get().to_le_bytes());
    output.push(candidate.initial_charges());
    output.extend_from_slice(
        &candidate
            .acquisition_fragment_divisor()
            .unwrap_or(0)
            .to_le_bytes(),
    );
    output.extend_from_slice(
        &candidate
            .acquisition_fragment_stack_divisor()
            .unwrap_or(0)
            .to_le_bytes(),
    );
}

fn decode_curio(decoder: &mut Decoder<'_>) -> Result<CurioActivityRecord, ActivityHandlerFault> {
    let id = CurioId::new(decoder.u32()?).ok_or_else(invalid_payload)?;
    let state = CurioStateId::new(decoder.u32()?).ok_or_else(invalid_payload)?;
    let charges = decoder.u8()?;
    let divisor = decoder.i64()?;
    let stack_divisor = decoder.i64()?;
    let mut record =
        CurioActivityRecord::new(id, state, charges, (divisor != 0).then_some(divisor));
    if stack_divisor != 0 {
        record = record.with_fragment_stack_capture(stack_divisor);
    }
    Ok(record)
}

fn encode_ids(output: &mut Vec<u8>, values: &[u64]) -> Result<(), OccurrenceInteractionError> {
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

fn decode_ids(decoder: &mut Decoder<'_>) -> Result<Vec<u64>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        values.push(decoder.u64()?);
    }
    Ok(values)
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
        .and_then(|value| {
            value
                .entries()
                .iter()
                .find(|entry| entry.0 == content)
                .map(|entry| entry.1)
                .or(Some(0))
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

fn advance(operations: &mut Vec<ActivityOperation>, effect_slot: ActivitySlotId, key: u64) {
    operations.push(ActivityOperation::AddCounter {
        slot: effect_slot,
        key: attempt_key(key),
        delta: integer(1),
    });
    operations.push(ActivityOperation::AddCounter {
        slot: effect_slot,
        key: repeat_key(key),
        delta: integer(1),
    });
}

fn reset(operations: &mut Vec<ActivityOperation>, effect_slot: ActivitySlotId, key: u64) {
    operations.push(reset_counter(effect_slot, attempt_key(key)));
}

fn reset_counter(slot: ActivitySlotId, key: u64) -> ActivityOperation {
    ActivityOperation::AddCounter {
        slot,
        key,
        delta: ActivityExpression::Negate(Box::new(ActivityExpression::CounterValue { slot, key })),
    }
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

const fn repeat_key(key: u64) -> u64 {
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

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
