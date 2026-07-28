//! Shared primitives introduced by Goal 07 Occurrence partition S08.

use starclock_combat::{Hp, LifeState, Ratio};

use super::*;

const TAG_HISTORY_PATH: u8 = 21;
const TAG_SEQUENCE: u8 = 22;
const TAG_ENHANCED_INVENTORY: u8 = 23;
const TAG_YU_MARK: u8 = 24;
const TAG_YU_MIX: u8 = 25;
const TAG_REQUIRE_STAGE: u8 = 26;
const PREFIX: &str = "universe.occurrence-s08.";
const COSMIC_START_KEY: u64 = 0x5800_0000_0000_0001;
const COSMIC_REPEAT_KEY: u64 = 0x5900_0000_0000_0001;
const YU_REPEAT_KEY: u64 = 0x5900_0000_0000_0002;
const YU_SUGAR_KEY: u64 = 0x5800_0000_0000_0010;
const YU_TOOTHPASTE_KEY: u64 = 0x5800_0000_0000_0011;
const YU_THIEF_KEY: u64 = 0x5800_0000_0000_0012;
const YU_ELITE_KEY: u64 = 0x5800_0000_0000_0013;

#[derive(Clone)]
pub(super) enum Operation {
    HistoryPath {
        inventory: ActivityInventoryId,
        quantity: u16,
        selected: u16,
        groups: Vec<Vec<u64>>,
    },
    Sequence {
        effect_slot: ActivitySlotId,
    },
    EnhancedInventory {
        inventory: ActivityInventoryId,
        candidates: Vec<u64>,
    },
    YuMark {
        effect_slot: ActivitySlotId,
        stage_key: u64,
    },
    YuMix {
        bindings: CurioActivityBindings,
        effect_slot: ActivitySlotId,
        required_stage: u64,
        kind: u8,
        curio: CurioActivityRecord,
    },
    RequireStage {
        effect_slot: ActivitySlotId,
        stage_key: u64,
    },
}

pub(super) struct ExternalChoice {
    pub(super) content: u64,
    pub(super) operations: Vec<PayloadOperation>,
    pub(super) random_candidate_count: Option<u32>,
}

pub(super) struct ExternalLowering {
    pub(super) choices: Vec<ExternalChoice>,
    pub(super) repeat_key: Option<u64>,
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::HistoryPath {
                inventory,
                quantity,
                selected,
                groups,
            } => {
                output.push(TAG_HISTORY_PATH);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(&quantity.to_le_bytes());
                output.extend_from_slice(&selected.to_le_bytes());
                encode_groups(output, groups)?;
            }
            Self::Sequence { effect_slot } => {
                output.push(TAG_SEQUENCE);
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
            }
            Self::EnhancedInventory {
                inventory,
                candidates,
            } => {
                output.push(TAG_ENHANCED_INVENTORY);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                encode_ids(output, candidates)?;
            }
            Self::YuMark {
                effect_slot,
                stage_key,
            } => {
                output.push(TAG_YU_MARK);
                encode_stage(output, effect_slot, stage_key);
            }
            Self::YuMix {
                bindings,
                effect_slot,
                required_stage,
                kind,
                curio,
            } => {
                output.push(TAG_YU_MIX);
                encode_bindings(output, bindings);
                output.extend_from_slice(&effect_slot.get().to_le_bytes());
                output.extend_from_slice(&required_stage.to_le_bytes());
                output.push(kind);
                encode_curio(output, curio);
            }
            Self::RequireStage {
                effect_slot,
                stage_key,
            } => {
                output.push(TAG_REQUIRE_STAGE);
                encode_stage(output, effect_slot, stage_key);
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::HistoryPath {
                selected, groups, ..
            } => groups
                .get(usize::from(*selected))
                .and_then(|group| u32::try_from(group.len()).ok()),
            Self::EnhancedInventory { candidates, .. } => u32::try_from(candidates.len()).ok(),
            Self::YuMix { .. } => Some(100),
            Self::Sequence { .. } | Self::YuMark { .. } | Self::RequireStage { .. } => None,
        }
    }
}

pub(super) fn externalize(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    cosmic_fragments: ActivitySlotId,
    curio_bindings: CurioActivityBindings,
    curio_records: &[CurioActivityRecord],
    effect_slot: ActivitySlotId,
) -> Result<Option<ExternalLowering>, OccurrenceInteractionError> {
    if has_marker(outcome, "history-best-path") {
        return history_choices(outcome, catalog, blessing_inventory).map(Some);
    }
    if has_marker(outcome, "cosmic-crescendo") {
        return cosmic_choices(
            catalog,
            blessing_inventory,
            cosmic_fragments,
            curio_bindings,
            curio_records,
            effect_slot,
        )
        .map(Some);
    }
    Ok(None)
}

pub(super) fn lower(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    curio_bindings: CurioActivityBindings,
    curio_records: &[CurioActivityRecord],
    effect_slot: ActivitySlotId,
) -> Result<Option<(Operation, Option<u64>, bool)>, OccurrenceInteractionError> {
    let marker = outcome
        .parameter_refs()
        .iter()
        .find_map(|value| value.strip_prefix(PREFIX));
    let Some(marker) = marker else {
        return Ok(None);
    };
    let value = match marker {
        "yu-add-sugar" => (
            Operation::YuMark {
                effect_slot,
                stage_key: YU_SUGAR_KEY,
            },
            Some(YU_REPEAT_KEY),
            true,
        ),
        "yu-add-toothpaste" => (
            Operation::YuMark {
                effect_slot,
                stage_key: YU_TOOTHPASTE_KEY,
            },
            Some(YU_REPEAT_KEY),
            true,
        ),
        "yu-sugar-vigorous" => (
            yu_mix(
                catalog,
                curio_bindings,
                curio_records,
                effect_slot,
                YU_SUGAR_KEY,
                1,
                "universe.curio.122",
            )?,
            Some(YU_REPEAT_KEY),
            true,
        ),
        "yu-sugar-gentle" => (
            yu_mix(
                catalog,
                curio_bindings,
                curio_records,
                effect_slot,
                YU_SUGAR_KEY,
                2,
                "universe.curio.122",
            )?,
            Some(YU_REPEAT_KEY),
            true,
        ),
        "yu-toothpaste-vigorous" => (
            yu_mix(
                catalog,
                curio_bindings,
                curio_records,
                effect_slot,
                YU_TOOTHPASTE_KEY,
                3,
                "universe.curio.121",
            )?,
            Some(YU_REPEAT_KEY),
            true,
        ),
        "yu-toothpaste-gentle" => (
            yu_mix(
                catalog,
                curio_bindings,
                curio_records,
                effect_slot,
                YU_TOOTHPASTE_KEY,
                4,
                "universe.curio.121",
            )?,
            Some(YU_REPEAT_KEY),
            true,
        ),
        "yu-resolve-elite" => (
            Operation::RequireStage {
                effect_slot,
                stage_key: YU_ELITE_KEY,
            },
            None,
            false,
        ),
        "yu-resolve-thief" => (
            Operation::RequireStage {
                effect_slot,
                stage_key: YU_THIEF_KEY,
            },
            None,
            false,
        ),
        "history-best-path" | "cosmic-crescendo" => return Ok(None),
        _ => return Err(OccurrenceInteractionError::InvalidChoice),
    };
    Ok(Some(value))
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_HISTORY_PATH => decode_history(input, decoder, operations)?,
        TAG_SEQUENCE => decode_sequence(input, decoder, operations)?,
        TAG_ENHANCED_INVENTORY => decode_enhanced(input, decoder, operations)?,
        TAG_YU_MARK => decode_yu_mark(input, decoder, operations)?,
        TAG_YU_MIX => decode_yu_mix(input, decoder, operations)?,
        TAG_REQUIRE_STAGE => decode_require_stage(input, decoder, operations)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn history_choices(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    inventory: ActivityInventoryId,
) -> Result<ExternalLowering, OccurrenceInteractionError> {
    let quantity = outcome
        .numeric_literals()
        .first()
        .copied()
        .map(exact_integer)
        .transpose()?
        .and_then(|value| u16::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    let rarity = outcome
        .parameter_refs()
        .iter()
        .find_map(|value| value.strip_prefix("universe.blessing-pool.rarity."))
        .and_then(|value| value.parse::<u8>().ok())
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    let mut paths = catalog.paths().iter().collect::<Vec<_>>();
    paths.sort_unstable_by_key(|path| path.id());
    let groups = paths
        .iter()
        .map(|path| {
            catalog
                .blessings()
                .iter()
                .filter(|blessing| blessing.path() == path.id() && blessing.rarity() == rarity)
                .map(|blessing| u64::from(blessing.id().get()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    if groups
        .iter()
        .any(|group| group.len() < usize::from(quantity))
    {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    let choices = paths
        .iter()
        .enumerate()
        .map(|(selected, path)| {
            let operation = Operation::HistoryPath {
                inventory,
                quantity,
                selected: u16::try_from(selected)
                    .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?,
                groups: groups.clone(),
            };
            Ok(ExternalChoice {
                content: u64::from(path.id().get()),
                random_candidate_count: operation.random_candidate_count(),
                operations: vec![PayloadOperation::S08(operation)],
            })
        })
        .collect::<Result<Vec<_>, OccurrenceInteractionError>>()?;
    Ok(ExternalLowering {
        choices,
        repeat_key: None,
    })
}

fn cosmic_choices(
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    fragments: ActivitySlotId,
    curio_bindings: CurioActivityBindings,
    curio_records: &[CurioActivityRecord],
    effect_slot: ActivitySlotId,
) -> Result<ExternalLowering, OccurrenceInteractionError> {
    let blessings = |rarities: &[u8]| {
        catalog
            .blessings()
            .iter()
            .filter(|blessing| rarities.contains(&blessing.rarity()))
            .map(|blessing| u64::from(blessing.id().get()))
            .collect::<Vec<_>>()
    };
    let negative = curio_records
        .iter()
        .copied()
        .filter(|record| {
            catalog
                .curios()
                .iter()
                .find(|curio| curio.id() == record.id())
                .is_some_and(|curio| curio.tags().iter().any(|tag| tag.as_ref() == "negative"))
        })
        .collect::<Vec<_>>();
    let all_blessings = blessings(&[1, 2, 3]);
    let mut effects = vec![
        vec![PayloadOperation::CurioInventory {
            bindings: curio_bindings,
            delta: 1,
            quantity: 1,
            owned_only: false,
            candidates: curio_records.to_vec(),
        }],
        vec![fragment_scalar(fragments, curio_bindings.inventory, 100)],
        vec![inventory_operation(
            blessing_inventory,
            1,
            false,
            blessings(&[1]),
        )],
        vec![inventory_operation(
            blessing_inventory,
            1,
            true,
            all_blessings.clone(),
        )],
        vec![fragment_percent(fragments, curio_bindings.inventory, 50, 1)],
        vec![PayloadOperation::CurioInventory {
            bindings: curio_bindings,
            delta: -1,
            quantity: 1,
            owned_only: true,
            candidates: negative.clone(),
        }],
        vec![inventory_operation(
            blessing_inventory,
            1,
            false,
            blessings(&[2, 3]),
        )],
        vec![PayloadOperation::S08(Operation::EnhancedInventory {
            inventory: blessing_inventory,
            candidates: all_blessings.clone(),
        })],
        vec![fragment_percent(
            fragments,
            curio_bindings.inventory,
            20,
            -1,
        )],
        vec![PayloadOperation::CurioInventory {
            bindings: curio_bindings,
            delta: -1,
            quantity: 1,
            owned_only: true,
            candidates: curio_records.to_vec(),
        }],
        vec![
            inventory_operation(blessing_inventory, -1, true, blessings(&[2])),
            inventory_operation(blessing_inventory, 1, false, blessings(&[1])),
        ],
        vec![inventory_operation(
            blessing_inventory,
            -1,
            true,
            blessings(&[1, 2]),
        )],
        vec![PayloadOperation::CurioInventory {
            bindings: curio_bindings,
            delta: 1,
            quantity: 1,
            owned_only: false,
            candidates: negative,
        }],
        vec![fragment_percent(
            fragments,
            curio_bindings.inventory,
            40,
            -1,
        )],
        vec![
            inventory_operation(blessing_inventory, -1, true, blessings(&[3])),
            inventory_operation(blessing_inventory, 1, false, blessings(&[2])),
        ],
        vec![inventory_operation(
            blessing_inventory,
            -1,
            true,
            blessings(&[2, 3]),
        )],
    ];
    if effects.iter().any(Vec::is_empty) {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    let choices = effects
        .iter_mut()
        .enumerate()
        .map(|(index, operations)| {
            operations.insert(
                0,
                PayloadOperation::S08(Operation::Sequence { effect_slot }),
            );
            Ok(ExternalChoice {
                content: u64::try_from(index + 1)
                    .map_err(|_| OccurrenceInteractionError::Arithmetic)?,
                random_candidate_count: operation_candidate_count(operations),
                operations: std::mem::take(operations),
            })
        })
        .collect::<Result<Vec<_>, OccurrenceInteractionError>>()?;
    Ok(ExternalLowering {
        choices,
        repeat_key: Some(COSMIC_REPEAT_KEY),
    })
}

fn yu_mix(
    catalog: &UniverseCatalog,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
    effect_slot: ActivitySlotId,
    required_stage: u64,
    kind: u8,
    curio_key: &str,
) -> Result<Operation, OccurrenceInteractionError> {
    let id = catalog
        .curios()
        .iter()
        .find(|curio| curio.stable_key() == curio_key)
        .map(|curio| curio.id())
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    let curio = records
        .iter()
        .copied()
        .find(|record| record.id() == id)
        .ok_or(OccurrenceInteractionError::InvalidChoice)?;
    Ok(Operation::YuMix {
        bindings,
        effect_slot,
        required_stage,
        kind,
        curio,
    })
}

fn decode_history(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let quantity = usize::from(decoder.u16()?);
    let selected = usize::from(decoder.u16()?);
    let groups = decode_groups(decoder)?;
    let selected_group = groups.get(selected).ok_or_else(invalid_payload)?;
    let entries = input
        .view()
        .inventories()
        .iter()
        .find(|value| value.id() == inventory)
        .map(|value| value.entries())
        .ok_or_else(invalid_state)?;
    let owned = |group: &[u64]| {
        group
            .iter()
            .filter(|candidate| inventory_count_entries(entries, **candidate) == 1)
            .count()
    };
    let selected_owned = owned(selected_group);
    if quantity == 0
        || selected_owned < quantity
        || groups.iter().any(|group| owned(group) > selected_owned)
    {
        return Err(invalid_state());
    }
    let eligible = selected_group
        .iter()
        .copied()
        .filter(|candidate| inventory_count_entries(entries, *candidate) == 1)
        .collect::<Vec<_>>();
    let start = input
        .random_index()
        .map_or(0, |value| value as usize % eligible.len());
    for offset in 0..quantity {
        operations.push(ActivityOperation::AddInventory {
            inventory,
            content: eligible[(start + offset) % eligible.len()],
            count: integer(1),
        });
    }
    Ok(())
}

fn decode_sequence(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let effect_slot = slot(decoder.u32()?)?;
    if counter_value(input, effect_slot, COSMIC_START_KEY)? == 0 {
        operations.push(add_counter(effect_slot, COSMIC_START_KEY, 1));
        operations.push(add_counter(effect_slot, COSMIC_REPEAT_KEY, 9));
    }
    Ok(())
}

fn decode_enhanced(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let candidates = decode_ids(decoder)?;
    let selected = select_candidates(
        input,
        inventory,
        &candidates,
        false,
        input.random_index(),
        1,
    )?;
    operations.push(ActivityOperation::AddInventory {
        inventory,
        content: selected[0],
        count: integer(2),
    });
    Ok(())
}

fn decode_yu_mark(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let (effect_slot, stage_key) = decode_stage(decoder)?;
    if counter_value(input, effect_slot, stage_key)? != 0 {
        return Err(invalid_state());
    }
    operations.push(add_counter(effect_slot, stage_key, 1));
    operations.push(add_counter(effect_slot, YU_REPEAT_KEY, 1));
    Ok(())
}

fn decode_yu_mix(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let effect_slot = slot(decoder.u32()?)?;
    let required_stage = decoder.u64()?;
    let kind = decoder.u8()?;
    let curio = decode_curio(decoder)?;
    if counter_value(input, effect_slot, required_stage)? != 1 {
        return Err(invalid_state());
    }
    operations.push(add_counter(effect_slot, required_stage, -1));
    let roll = u8::try_from(input.random_index().ok_or_else(invalid_state)? % 100)
        .expect("modulo fits u8");
    match (kind, roll) {
        (1, 0..=69) | (2, 0..=49) | (3, 0..=79) | (4, 0..=49) => {
            operations.extend(acquisition_operations(curio, bindings));
        }
        (1, 70..=89) | (4, 50..=79) => lose_hp(input, operations),
        (1, 90..=99) | (2, 50..=99) => {
            operations.push(add_counter(effect_slot, YU_THIEF_KEY, 1));
            operations.push(add_counter(effect_slot, YU_REPEAT_KEY, 1));
        }
        (3, 80..=99) | (4, 80..=99) => {
            operations.push(add_counter(effect_slot, YU_ELITE_KEY, 1));
            operations.push(add_counter(effect_slot, YU_REPEAT_KEY, 1));
        }
        _ => return Err(invalid_payload()),
    }
    Ok(())
}

fn decode_require_stage(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let (effect_slot, stage_key) = decode_stage(decoder)?;
    if counter_value(input, effect_slot, stage_key)? != 1 {
        return Err(invalid_state());
    }
    operations.push(add_counter(effect_slot, stage_key, -1));
    Ok(())
}

fn lose_hp(input: ActivityHandlerInput<'_>, operations: &mut Vec<ActivityOperation>) {
    operations.extend(
        input
            .view()
            .participant_carry()
            .iter()
            .filter(|state| state.life() == LifeState::Alive)
            .map(|state| ActivityOperation::LoseParticipantCurrentHpRatio {
                participant: state.participant(),
                hp_ratio: Ratio::from_scaled(800_000),
                minimum_hp: Hp::new(1).expect("one HP is valid"),
            }),
    );
}

fn has_marker(outcome: &OccurrenceOutcome, marker: &str) -> bool {
    outcome
        .parameter_refs()
        .iter()
        .any(|value| value.as_ref() == format!("{PREFIX}{marker}"))
}

fn operation_candidate_count(operations: &[PayloadOperation]) -> Option<u32> {
    operations
        .iter()
        .filter_map(|operation| match operation {
            PayloadOperation::Inventory { candidates, .. } => u32::try_from(candidates.len()).ok(),
            PayloadOperation::CurioInventory { candidates, .. } => {
                u32::try_from(candidates.len()).ok()
            }
            PayloadOperation::S08(operation) => operation.random_candidate_count(),
            _ => None,
        })
        .try_fold(1_u32, checked_lcm)
}

fn inventory_operation(
    inventory: ActivityInventoryId,
    delta: i8,
    owned_only: bool,
    candidates: Vec<u64>,
) -> PayloadOperation {
    PayloadOperation::Inventory {
        inventory,
        delta,
        quantity: 1,
        owned_only,
        candidates,
    }
}

fn fragment_scalar(
    slot: ActivitySlotId,
    gain_inventory: ActivityInventoryId,
    delta: i64,
) -> PayloadOperation {
    PayloadOperation::FragmentScalar {
        slot,
        gain_inventory,
        delta,
    }
}

fn fragment_percent(
    slot: ActivitySlotId,
    gain_inventory: ActivityInventoryId,
    coefficient: i64,
    sign: i8,
) -> PayloadOperation {
    PayloadOperation::FragmentPercent {
        slot,
        gain_inventory,
        coefficient,
        scale: 0,
        sign,
    }
}

fn encode_stage(output: &mut Vec<u8>, slot: ActivitySlotId, key: u64) {
    output.extend_from_slice(&slot.get().to_le_bytes());
    output.extend_from_slice(&key.to_le_bytes());
}

fn decode_stage(decoder: &mut Decoder<'_>) -> Result<(ActivitySlotId, u64), ActivityHandlerFault> {
    Ok((slot(decoder.u32()?)?, decoder.u64()?))
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

fn encode_curio(output: &mut Vec<u8>, record: CurioActivityRecord) {
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

fn decode_curio(decoder: &mut Decoder<'_>) -> Result<CurioActivityRecord, ActivityHandlerFault> {
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
        encode_ids(output, group)?;
    }
    Ok(())
}

fn decode_groups(decoder: &mut Decoder<'_>) -> Result<Vec<Vec<u64>>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    (0..count).map(|_| decode_ids(decoder)).collect()
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
    (0..count).map(|_| decoder.u64()).collect()
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

fn inventory_count_entries(entries: &[(u64, u32)], content: u64) -> u32 {
    entries
        .iter()
        .find(|entry| entry.0 == content)
        .map_or(0, |entry| entry.1)
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
