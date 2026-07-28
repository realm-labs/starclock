//! Shared primitives introduced by Goal 07 Occurrence partition S11.

use starclock_combat::{Hp, LifeState, Ratio};

use super::*;

const TAG_BEAUTY_FEED_BLESSING: u8 = 33;
const TAG_BEAUTY_FEED_FRAGMENTS: u8 = 34;
const TAG_ACE_TRASH: u8 = 35;
const TAG_SHOPPING_DOUGHNUTS: u8 = 36;
const TAG_SHOPPING_LOTUS: u8 = 37;
const TAG_SHOPPING_BOX: u8 = 38;
const TAG_DANCER: u8 = 39;
const TAG_MIRROR_LIGHT: u8 = 40;
const TAG_MIRROR_WISH: u8 = 41;
const PREFIX: &str = "universe.occurrence-s11.";
const BEAUTY_BUG_UNLOCK_KEY: u64 = 0x5d00_0000_0000_0001;
pub(super) const DANCER_STAGE_KEY: u64 = 0x5e00_0000_0000_0001;
pub(super) const DANCER_REPEAT_KEY: u64 = 0x5e00_0000_0000_0002;
pub(super) const MIRROR_CANDLE_KEY: u64 = 0x5f00_0000_0000_0001;
pub(super) const MIRROR_WISH_KEY: u64 = 0x5f00_0000_0000_0002;
pub(super) const MIRROR_PART_THREE_KEY: u64 = 0x5f00_0000_0000_0003;
pub(super) const MIRROR_REPEAT_KEY: u64 = 0x5f00_0000_0000_0004;
const SHOPPING_LOTUS_RANDOM_CANDIDATES: u32 = 50_400;
const SHOPPING_BOX_RANDOM_CANDIDATES: u32 = 65_000;
const MIRROR_WISH_RANDOM_CANDIDATES: u32 = 64_800;

#[derive(Clone)]
pub(super) enum Operation {
    BeautyFeedBlessing {
        inventory: ActivityInventoryId,
        content: u64,
        state_slot: ActivitySlotId,
    },
    BeautyFeedFragments {
        fragments_slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        state_slot: ActivitySlotId,
    },
    AceTrash {
        bindings: CurioActivityBindings,
        discard: CurioActivityRecord,
        rewards: Vec<CurioActivityRecord>,
    },
    ShoppingDoughnuts,
    ShoppingLotus {
        inventory: ActivityInventoryId,
        one_star: Vec<u64>,
        two_star: Vec<u64>,
    },
    ShoppingBox {
        bindings: CurioActivityBindings,
        positive: Vec<CurioActivityRecord>,
        negative: Vec<CurioActivityRecord>,
    },
    Dancer {
        fragments_slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        blessing_inventory: ActivityInventoryId,
        blessing: u64,
        state_slot: ActivitySlotId,
    },
    MirrorLight {
        state_slot: ActivitySlotId,
    },
    MirrorWish {
        fragments_slot: ActivitySlotId,
        gain_inventory: ActivityInventoryId,
        blessing_inventory: ActivityInventoryId,
        bindings: CurioActivityBindings,
        all_blessings: Vec<u64>,
        two_star: Vec<u64>,
        three_star: Vec<u64>,
        positive: Vec<CurioActivityRecord>,
        state_slot: ActivitySlotId,
    },
}

pub(super) struct Lowering {
    pub(super) operations: Vec<PayloadOperation>,
    pub(super) repeat_key: Option<u64>,
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::BeautyFeedBlessing {
                inventory,
                content,
                state_slot,
            } => {
                output.push(TAG_BEAUTY_FEED_BLESSING);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                output.extend_from_slice(&content.to_le_bytes());
                output.extend_from_slice(&state_slot.get().to_le_bytes());
            }
            Self::BeautyFeedFragments {
                fragments_slot,
                gain_inventory,
                state_slot,
            } => {
                output.push(TAG_BEAUTY_FEED_FRAGMENTS);
                output.extend_from_slice(&fragments_slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&state_slot.get().to_le_bytes());
            }
            Self::AceTrash {
                bindings,
                discard,
                rewards,
            } => {
                output.push(TAG_ACE_TRASH);
                encode_bindings(output, bindings);
                encode_record(output, discard);
                encode_records(output, rewards)?;
            }
            Self::ShoppingDoughnuts => output.push(TAG_SHOPPING_DOUGHNUTS),
            Self::ShoppingLotus {
                inventory,
                one_star,
                two_star,
            } => {
                output.push(TAG_SHOPPING_LOTUS);
                output.extend_from_slice(&inventory.get().to_le_bytes());
                encode_ids(output, one_star)?;
                encode_ids(output, two_star)?;
            }
            Self::ShoppingBox {
                bindings,
                positive,
                negative,
            } => {
                output.push(TAG_SHOPPING_BOX);
                encode_bindings(output, bindings);
                encode_records(output, positive)?;
                encode_records(output, negative)?;
            }
            Self::Dancer {
                fragments_slot,
                gain_inventory,
                blessing_inventory,
                blessing,
                state_slot,
            } => {
                output.push(TAG_DANCER);
                output.extend_from_slice(&fragments_slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&blessing_inventory.get().to_le_bytes());
                output.extend_from_slice(&blessing.to_le_bytes());
                output.extend_from_slice(&state_slot.get().to_le_bytes());
            }
            Self::MirrorLight { state_slot } => {
                output.push(TAG_MIRROR_LIGHT);
                output.extend_from_slice(&state_slot.get().to_le_bytes());
            }
            Self::MirrorWish {
                fragments_slot,
                gain_inventory,
                blessing_inventory,
                bindings,
                all_blessings,
                two_star,
                three_star,
                positive,
                state_slot,
            } => {
                output.push(TAG_MIRROR_WISH);
                output.extend_from_slice(&fragments_slot.get().to_le_bytes());
                output.extend_from_slice(&gain_inventory.get().to_le_bytes());
                output.extend_from_slice(&blessing_inventory.get().to_le_bytes());
                encode_bindings(output, bindings);
                encode_ids(output, all_blessings)?;
                encode_ids(output, two_star)?;
                encode_ids(output, three_star)?;
                encode_records(output, positive)?;
                output.extend_from_slice(&state_slot.get().to_le_bytes());
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::BeautyFeedBlessing { .. }
            | Self::BeautyFeedFragments { .. }
            | Self::Dancer { .. }
            | Self::ShoppingDoughnuts => Some(100),
            Self::AceTrash { rewards, .. } => u32::try_from(rewards.len()).ok(),
            Self::ShoppingLotus { .. } => Some(SHOPPING_LOTUS_RANDOM_CANDIDATES),
            Self::ShoppingBox { .. } => Some(SHOPPING_BOX_RANDOM_CANDIDATES),
            Self::MirrorWish { .. } => Some(MIRROR_WISH_RANDOM_CANDIDATES),
            Self::MirrorLight { .. } => None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn externalize(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    fragments_slot: ActivitySlotId,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
    state_slot: ActivitySlotId,
) -> Result<Option<external::Lowering>, OccurrenceInteractionError> {
    let Some(kind) = marker(outcome) else {
        return Ok(None);
    };
    let choices = match kind {
        "beauty-bug-feed-blessing" => catalog
            .blessings()
            .iter()
            .map(|blessing| {
                let operation = Operation::BeautyFeedBlessing {
                    inventory: blessing_inventory,
                    content: u64::from(blessing.id().get()),
                    state_slot,
                };
                external::Choice {
                    content: u64::from(blessing.id().get()),
                    random_candidate_count: operation.random_candidate_count(),
                    operations: vec![PayloadOperation::S11(operation)],
                }
            })
            .collect(),
        "beauty-bug-life-favor" => blessings(catalog, 3)
            .into_iter()
            .map(|content| external::Choice {
                content,
                random_candidate_count: None,
                operations: vec![PayloadOperation::Inventory {
                    inventory: blessing_inventory,
                    delta: 1,
                    quantity: 1,
                    owned_only: false,
                    candidates: vec![content],
                }],
            })
            .collect(),
        "ace-trash-exchange" => {
            let rewards = curio_pool(catalog, records, "polarity:positive")?;
            records
                .iter()
                .copied()
                .map(|discard| {
                    let operation = Operation::AceTrash {
                        bindings,
                        discard,
                        rewards: rewards.clone(),
                    };
                    external::Choice {
                        content: u64::from(discard.id().get()),
                        random_candidate_count: operation.random_candidate_count(),
                        operations: vec![PayloadOperation::S11(operation)],
                    }
                })
                .collect()
        }
        "universal-dancer-fortune" => blessings(catalog, 3)
            .into_iter()
            .map(|blessing| {
                let operation = Operation::Dancer {
                    fragments_slot,
                    gain_inventory: bindings.inventory,
                    blessing_inventory,
                    blessing,
                    state_slot,
                };
                external::Choice {
                    content: blessing,
                    random_candidate_count: operation.random_candidate_count(),
                    operations: vec![PayloadOperation::S11(operation)],
                }
            })
            .collect(),
        _ => return Ok(None),
    };
    Ok(Some(external::Lowering {
        choices,
        repeat_key: (kind == "universal-dancer-fortune").then_some(DANCER_REPEAT_KEY),
    }))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    fragments_slot: ActivitySlotId,
    bindings: CurioActivityBindings,
    records: &[CurioActivityRecord],
    state_slot: ActivitySlotId,
) -> Result<Option<Lowering>, OccurrenceInteractionError> {
    let Some(kind) = marker(outcome) else {
        return Ok(None);
    };
    let operation = |value| Lowering {
        operations: vec![PayloadOperation::S11(value)],
        repeat_key: None,
    };
    let no_change = || Lowering {
        operations: vec![PayloadOperation::Transition],
        repeat_key: None,
    };
    let value = match kind {
        "beauty-bug-feed-blessing"
        | "beauty-bug-life-favor"
        | "ace-trash-exchange"
        | "universal-dancer-fortune" => return Ok(None),
        "beauty-bug-feed-fragments" => operation(Operation::BeautyFeedFragments {
            fragments_slot,
            gain_inventory: bindings.inventory,
            state_slot,
        }),
        "beauty-bug-heartfelt-gift" => Lowering {
            operations: vec![PayloadOperation::CurioInventory {
                bindings,
                delta: 1,
                quantity: 5,
                owned_only: false,
                candidates: curio_pool(catalog, records, "polarity:positive")?,
            }],
            repeat_key: None,
        },
        "beauty-bug-refuse"
        | "ace-trash-leave"
        | "shopping-leave"
        | "universal-dancer-refuse"
        | "mirror-leave" => no_change(),
        "shopping-doughnuts" => operation(Operation::ShoppingDoughnuts),
        "shopping-lotus" => operation(Operation::ShoppingLotus {
            inventory: blessing_inventory,
            one_star: blessings(catalog, 1),
            two_star: blessings(catalog, 2),
        }),
        "shopping-mechanical-box" => operation(Operation::ShoppingBox {
            bindings,
            positive: curio_pool(catalog, records, "polarity:positive")?,
            negative: curio_pool(catalog, records, "polarity:negative")?,
        }),
        "mirror-light-candle" => Lowering {
            operations: vec![PayloadOperation::S11(Operation::MirrorLight { state_slot })],
            repeat_key: Some(MIRROR_REPEAT_KEY),
        },
        "mirror-random-wish" => operation(Operation::MirrorWish {
            fragments_slot,
            gain_inventory: bindings.inventory,
            blessing_inventory,
            bindings,
            all_blessings: catalog
                .blessings()
                .iter()
                .map(|value| u64::from(value.id().get()))
                .collect(),
            two_star: blessings(catalog, 2),
            three_star: blessings(catalog, 3),
            positive: curio_pool(catalog, records, "polarity:positive")?,
            state_slot,
        }),
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
        TAG_BEAUTY_FEED_BLESSING => decode_beauty_blessing(input, decoder, operations)?,
        TAG_BEAUTY_FEED_FRAGMENTS => decode_beauty_fragments(input, decoder, operations)?,
        TAG_ACE_TRASH => decode_ace_trash(input, decoder, operations)?,
        TAG_SHOPPING_DOUGHNUTS => decode_doughnuts(input, operations)?,
        TAG_SHOPPING_LOTUS => decode_lotus(input, decoder, operations)?,
        TAG_SHOPPING_BOX => decode_box(input, decoder, operations)?,
        TAG_DANCER => decode_dancer(input, decoder, operations)?,
        TAG_MIRROR_LIGHT => decode_mirror_light(input, decoder, operations)?,
        TAG_MIRROR_WISH => decode_mirror_wish(input, decoder, operations)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_beauty_blessing(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let content = decoder.u64()?;
    let state_slot = slot(decoder.u32()?)?;
    operations.push(ActivityOperation::Require(ActivityCondition::LessThan(
        integer(0),
        ActivityExpression::InventoryCount { inventory, content },
    )));
    operations.push(ActivityOperation::RemoveInventory {
        inventory,
        content,
        count: integer(1),
    });
    beauty_result(input, operations, state_slot)
}

fn decode_beauty_fragments(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let fragments = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let state_slot = slot(decoder.u32()?)?;
    operations.push(require_at_least(fragments, 100)?);
    operations.push(fragment_delta(fragments, gain_inventory, -100));
    beauty_result(input, operations, state_slot)
}

fn beauty_result(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    state_slot: ActivitySlotId,
) -> Result<(), ActivityHandlerFault> {
    if input.random_index().ok_or_else(invalid_state)? < 70 {
        if counter_value(input, state_slot, BEAUTY_BUG_UNLOCK_KEY)? == 0 {
            operations.push(add_counter(state_slot, BEAUTY_BUG_UNLOCK_KEY, 1));
        }
    } else {
        lose_current_hp(input, operations, 300_000);
    }
    Ok(())
}

fn decode_ace_trash(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let discard = decode_record(decoder)?;
    let rewards = decode_records(decoder)?;
    let random = input.random_index().ok_or_else(invalid_state)?;
    if rewards.len() < 2 {
        return Err(invalid_payload());
    }
    operations.push(ActivityOperation::Require(ActivityCondition::LessThan(
        integer(0),
        ActivityExpression::InventoryCount {
            inventory: bindings.inventory,
            content: u64::from(discard.id().get()),
        },
    )));
    operations.extend(teardown_operations(discard.id(), bindings));
    let ids = rewards
        .iter()
        .map(|value| u64::from(value.id().get()))
        .collect::<Vec<_>>();
    for id in select_candidates(input, bindings.inventory, &ids, false, Some(random), 2)? {
        let record = rewards
            .iter()
            .find(|value| u64::from(value.id().get()) == id)
            .ok_or_else(invalid_payload)?;
        operations.extend(acquisition_operations(*record, bindings));
    }
    Ok(())
}

fn decode_doughnuts(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    if input.random_index().ok_or_else(invalid_state)? < 80 {
        for state in living_participants(input) {
            operations.push(ActivityOperation::HealParticipantMaximumHpRatio {
                participant: state.participant(),
                hp_ratio: Ratio::from_scaled(1_000_000),
            });
        }
    } else {
        lose_current_hp(input, operations, 200_000);
    }
    Ok(())
}

fn decode_lotus(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let one_star = decode_ids(decoder)?;
    let two_star = decode_ids(decoder)?;
    let random = input.random_index().ok_or_else(invalid_state)?;
    let (discard, obtain) = if random % 100 < 80 {
        (&one_star, &two_star)
    } else {
        (&two_star, &one_star)
    };
    let entropy = random / 100;
    let removed = select_candidates(input, inventory, discard, true, Some(entropy), 1)?[0];
    let added = select_candidates(input, inventory, obtain, false, Some(entropy), 1)?[0];
    operations.push(ActivityOperation::RemoveInventory {
        inventory,
        content: removed,
        count: integer(1),
    });
    operations.push(ActivityOperation::AddInventory {
        inventory,
        content: added,
        count: integer(1),
    });
    Ok(())
}

fn decode_box(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let positive = decode_records(decoder)?;
    let negative = decode_records(decoder)?;
    let random = input.random_index().ok_or_else(invalid_state)?;
    let pool = if random % 100 < 80 {
        &positive
    } else {
        &negative
    };
    let index = usize::try_from(random / 100).map_err(|_| invalid_state())? % pool.len();
    operations.extend(acquisition_operations(pool[index], bindings));
    Ok(())
}

fn decode_dancer(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let fragments = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let blessing_inventory = inventory(decoder.u32()?)?;
    let blessing = decoder.u64()?;
    let state_slot = slot(decoder.u32()?)?;
    let stage = counter_value(input, state_slot, DANCER_STAGE_KEY)?;
    let threshold = match stage {
        0 => 30,
        1 => 65,
        2 => 100,
        _ => return Err(invalid_state()),
    };
    operations.push(require_at_least(fragments, 50)?);
    operations.push(fragment_delta(fragments, gain_inventory, -50));
    if input.random_index().ok_or_else(invalid_state)? < threshold {
        operations.push(ActivityOperation::AddInventory {
            inventory: blessing_inventory,
            content: blessing,
            count: integer(1),
        });
        if stage > 0 {
            operations.push(add_counter(state_slot, DANCER_STAGE_KEY, -stage));
        }
    } else {
        operations.push(fragment_delta(fragments, gain_inventory, 100));
        operations.push(add_counter(state_slot, DANCER_STAGE_KEY, 1));
        operations.push(add_counter(state_slot, DANCER_REPEAT_KEY, 1));
    }
    Ok(())
}

fn decode_mirror_light(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let state_slot = slot(decoder.u32()?)?;
    let candles = counter_value(input, state_slot, MIRROR_CANDLE_KEY)?;
    if !(0..3).contains(&candles) {
        return Err(invalid_state());
    }
    operations.push(add_counter(state_slot, MIRROR_CANDLE_KEY, 1));
    operations.push(add_counter(state_slot, MIRROR_REPEAT_KEY, 1));
    Ok(())
}

fn decode_mirror_wish(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let fragments = slot(decoder.u32()?)?;
    let gain_inventory = inventory(decoder.u32()?)?;
    let blessing_inventory = inventory(decoder.u32()?)?;
    let bindings = decode_bindings(decoder)?;
    let all_blessings = decode_ids(decoder)?;
    let two_star = decode_ids(decoder)?;
    let three_star = decode_ids(decoder)?;
    let positive = decode_records(decoder)?;
    let state_slot = slot(decoder.u32()?)?;
    if counter_value(input, state_slot, MIRROR_CANDLE_KEY)? <= 0 {
        return Err(invalid_state());
    }
    let random = input.random_index().ok_or_else(invalid_state)?;
    let bucket = random % 20;
    let entropy = random / 20;
    match bucket {
        0..=1 => operations.push(fragment_delta(fragments, gain_inventory, 50)),
        2..=5 => operations.push(fragment_delta(fragments, gain_inventory, 150)),
        6 => operations.push(fragment_delta(fragments, gain_inventory, 300)),
        7..=10 => add_blessings(input, operations, blessing_inventory, &two_star, entropy, 1)?,
        11..=12 => add_blessings(
            input,
            operations,
            blessing_inventory,
            &three_star,
            entropy,
            1,
        )?,
        13..=14 => add_curios(input, operations, bindings, &positive, entropy, 2)?,
        15..=16 => enhance_blessings(
            input,
            operations,
            blessing_inventory,
            &all_blessings,
            entropy,
            3,
        )?,
        17..=19 => add_blessings(
            input,
            operations,
            blessing_inventory,
            &all_blessings,
            entropy,
            2,
        )?,
        _ => return Err(invalid_state()),
    }
    let wishes = counter_value(input, state_slot, MIRROR_WISH_KEY)?;
    operations.push(add_counter(state_slot, MIRROR_WISH_KEY, 1));
    if wishes == 2 && counter_value(input, state_slot, MIRROR_PART_THREE_KEY)? == 0 {
        operations.push(add_counter(state_slot, MIRROR_PART_THREE_KEY, 1));
    }
    Ok(())
}

fn add_blessings(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    inventory: ActivityInventoryId,
    candidates: &[u64],
    random: u32,
    quantity: usize,
) -> Result<(), ActivityHandlerFault> {
    for content in select_candidates(input, inventory, candidates, false, Some(random), quantity)? {
        operations.push(ActivityOperation::AddInventory {
            inventory,
            content,
            count: integer(1),
        });
    }
    Ok(())
}

fn enhance_blessings(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    inventory: ActivityInventoryId,
    candidates: &[u64],
    random: u32,
    quantity: usize,
) -> Result<(), ActivityHandlerFault> {
    for content in select_candidates(input, inventory, candidates, true, Some(random), quantity)? {
        operations.push(ActivityOperation::AddInventory {
            inventory,
            content,
            count: integer(1),
        });
    }
    Ok(())
}

fn add_curios(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    bindings: CurioActivityBindings,
    candidates: &[CurioActivityRecord],
    random: u32,
    quantity: usize,
) -> Result<(), ActivityHandlerFault> {
    let ids = candidates
        .iter()
        .map(|value| u64::from(value.id().get()))
        .collect::<Vec<_>>();
    for id in select_candidates(
        input,
        bindings.inventory,
        &ids,
        false,
        Some(random),
        quantity,
    )? {
        let record = candidates
            .iter()
            .find(|value| u64::from(value.id().get()) == id)
            .ok_or_else(invalid_payload)?;
        operations.extend(acquisition_operations(*record, bindings));
    }
    Ok(())
}

fn living_participants(
    input: ActivityHandlerInput<'_>,
) -> impl Iterator<Item = &starclock_activity::ActivityParticipantCarryState> {
    input
        .view()
        .participant_carry()
        .iter()
        .filter(|state| state.life() == LifeState::Alive)
}

fn lose_current_hp(
    input: ActivityHandlerInput<'_>,
    operations: &mut Vec<ActivityOperation>,
    scaled_ratio: i64,
) {
    let ratio = Ratio::from_scaled(scaled_ratio);
    operations.extend(living_participants(input).map(|state| {
        ActivityOperation::LoseParticipantCurrentHpRatio {
            participant: state.participant(),
            hp_ratio: ratio,
            minimum_hp: Hp::new(1).expect("one HP is valid"),
        }
    }));
}

fn marker(outcome: &OccurrenceOutcome) -> Option<&str> {
    outcome
        .parameter_refs()
        .iter()
        .find_map(|value| value.strip_prefix(PREFIX))
}

fn blessings(catalog: &UniverseCatalog, rarity: u8) -> Vec<u64> {
    catalog
        .blessings()
        .iter()
        .filter(|value| value.rarity() == rarity)
        .map(|value| u64::from(value.id().get()))
        .collect()
}

fn curio_pool(
    catalog: &UniverseCatalog,
    records: &[CurioActivityRecord],
    tag: &str,
) -> Result<Vec<CurioActivityRecord>, OccurrenceInteractionError> {
    if catalog.curios().len() != records.len() {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    let values = records
        .iter()
        .copied()
        .filter(|record| {
            catalog
                .curios()
                .iter()
                .find(|curio| curio.id() == record.id())
                .is_some_and(|curio| curio.pool_tags().iter().any(|value| value.as_ref() == tag))
        })
        .collect::<Vec<_>>();
    (!values.is_empty())
        .then_some(values)
        .ok_or(OccurrenceInteractionError::InvalidChoice)
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

fn encode_ids(output: &mut Vec<u8>, values: Vec<u64>) -> Result<(), OccurrenceInteractionError> {
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
    if count == 0 {
        return Err(invalid_payload());
    }
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
        encode_record(output, record);
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
    (0..count).map(|_| decode_record(decoder)).collect()
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
