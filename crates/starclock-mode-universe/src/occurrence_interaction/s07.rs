//! Shared primitives introduced by Goal 07 Occurrence partition S07.

use super::*;

const TAG_CURRENT_PATH_BLESSINGS: u8 = 17;
const TAG_CURRENT_PATH_FORMATION: u8 = 18;
const TAG_BLESSING_EXCHANGE: u8 = 19;
const TAG_ALL_CURIOS_FOR_FRAGMENTS: u8 = 20;
const PREFIX: &str = "universe.occurrence-s07.";

#[derive(Clone)]
pub(super) enum Operation {
    CurrentPathBlessings {
        inventory: ActivityInventoryId,
        path_slot: ActivitySlotId,
        quantity: u16,
        groups: Vec<(u64, Vec<u64>)>,
    },
    CurrentPathFormation {
        inventory: ActivityInventoryId,
        path_slot: ActivitySlotId,
        groups: Vec<(u64, Vec<u64>)>,
    },
    BlessingExchange {
        inventory: ActivityInventoryId,
        path_slot: ActivitySlotId,
        lose_quantity: u16,
        gain_quantity: u16,
        lose_candidates: Vec<u64>,
        gain_groups: Vec<(u64, Vec<u64>)>,
    },
    AllCuriosForFragments {
        bindings: CurioActivityBindings,
        fragments_per_curio: i64,
        curios: Vec<CurioActivityRecord>,
    },
}

impl Operation {
    pub(super) fn encode(self, output: &mut Vec<u8>) -> Result<(), OccurrenceInteractionError> {
        match self {
            Self::CurrentPathBlessings {
                inventory,
                path_slot,
                quantity,
                groups,
            } => {
                output.push(TAG_CURRENT_PATH_BLESSINGS);
                encode_inventory_path(output, inventory, path_slot);
                output.extend_from_slice(&quantity.to_le_bytes());
                encode_groups(output, groups)?;
            }
            Self::CurrentPathFormation {
                inventory,
                path_slot,
                groups,
            } => {
                output.push(TAG_CURRENT_PATH_FORMATION);
                encode_inventory_path(output, inventory, path_slot);
                encode_groups(output, groups)?;
            }
            Self::BlessingExchange {
                inventory,
                path_slot,
                lose_quantity,
                gain_quantity,
                lose_candidates,
                gain_groups,
            } => {
                output.push(TAG_BLESSING_EXCHANGE);
                encode_inventory_path(output, inventory, path_slot);
                output.extend_from_slice(&lose_quantity.to_le_bytes());
                output.extend_from_slice(&gain_quantity.to_le_bytes());
                encode_ids(output, lose_candidates)?;
                encode_groups(output, gain_groups)?;
            }
            Self::AllCuriosForFragments {
                bindings,
                fragments_per_curio,
                curios,
            } => {
                output.push(TAG_ALL_CURIOS_FOR_FRAGMENTS);
                encode_bindings(output, bindings);
                output.extend_from_slice(&fragments_per_curio.to_le_bytes());
                output.extend_from_slice(
                    &u16::try_from(curios.len())
                        .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
                        .to_le_bytes(),
                );
                for curio in curios {
                    output.extend_from_slice(&curio.id().get().to_le_bytes());
                }
            }
        }
        Ok(())
    }

    pub(super) fn random_candidate_count(&self) -> Option<u32> {
        match self {
            Self::CurrentPathBlessings { groups, .. }
            | Self::CurrentPathFormation { groups, .. } => group_candidate_count(groups),
            Self::BlessingExchange {
                lose_candidates,
                gain_groups,
                ..
            } => u32::try_from(lose_candidates.len())
                .ok()
                .and_then(|count| checked_lcm(count, group_candidate_count(gain_groups)?)),
            Self::AllCuriosForFragments { .. } => None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lower(
    outcome: &OccurrenceOutcome,
    catalog: &UniverseCatalog,
    blessing_inventory: ActivityInventoryId,
    curio_bindings: CurioActivityBindings,
    curios: &[CurioActivityRecord],
    path_slot: ActivitySlotId,
    formation_inventory: ActivityInventoryId,
) -> Result<Option<Operation>, OccurrenceInteractionError> {
    let markers = outcome
        .parameter_refs()
        .iter()
        .filter_map(|value| value.strip_prefix(PREFIX))
        .collect::<Vec<_>>();
    let Some(kind) = markers.first().copied() else {
        return Ok(None);
    };
    let quantity = || {
        outcome
            .numeric_literals()
            .first()
            .copied()
            .map(exact_integer)
            .transpose()?
            .and_then(|value| u16::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or(OccurrenceInteractionError::InvalidChoice)
    };
    match kind {
        "current-path-blessings" => Ok(Some(Operation::CurrentPathBlessings {
            inventory: blessing_inventory,
            path_slot,
            quantity: quantity()?,
            groups: blessing_groups(catalog, outcome, None)?,
        })),
        "current-path-formation" => Ok(Some(Operation::CurrentPathFormation {
            inventory: formation_inventory,
            path_slot,
            groups: catalog
                .paths()
                .iter()
                .map(|path| {
                    (
                        u64::from(path.id().get()),
                        path.formations()
                            .iter()
                            .map(|id| u64::from(id.get()))
                            .collect(),
                    )
                })
                .collect(),
        })),
        "exchange-rarity-3-for-3" => Ok(Some(Operation::BlessingExchange {
            inventory: blessing_inventory,
            path_slot,
            lose_quantity: 1,
            gain_quantity: 1,
            lose_candidates: blessings(catalog, &[3], None),
            gain_groups: vec![(0, blessings(catalog, &[3], None))],
        })),
        "exchange-rarity-1-2-for-1-2-3" => Ok(Some(Operation::BlessingExchange {
            inventory: blessing_inventory,
            path_slot,
            lose_quantity: 1,
            gain_quantity: 1,
            lose_candidates: blessings(catalog, &[1, 2], None),
            gain_groups: vec![(0, blessings(catalog, &[1, 2, 3], None))],
        })),
        "exchange-four-one-star-for-current-path" => Ok(Some(Operation::BlessingExchange {
            inventory: blessing_inventory,
            path_slot,
            lose_quantity: 4,
            gain_quantity: 4,
            lose_candidates: blessings(catalog, &[1], None),
            gain_groups: blessing_groups(catalog, outcome, Some(&[1, 2, 3]))?,
        })),
        "all-curios-for-fragments-50" => Ok(Some(Operation::AllCuriosForFragments {
            bindings: curio_bindings,
            fragments_per_curio: 50,
            curios: curios.to_vec(),
        })),
        _ => Err(OccurrenceInteractionError::InvalidChoice),
    }
}

pub(super) fn decode(
    tag: u8,
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<bool, ActivityHandlerFault> {
    match tag {
        TAG_CURRENT_PATH_BLESSINGS => decode_current_path(input, decoder, operations, None)?,
        TAG_CURRENT_PATH_FORMATION => decode_current_path(input, decoder, operations, Some(1))?,
        TAG_BLESSING_EXCHANGE => decode_exchange(input, decoder, operations)?,
        TAG_ALL_CURIOS_FOR_FRAGMENTS => decode_all_curios(input, decoder, operations)?,
        _ => return Ok(false),
    }
    Ok(true)
}

fn decode_current_path(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
    fixed_quantity: Option<usize>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let path_slot = slot(decoder.u32()?)?;
    let quantity = match fixed_quantity {
        Some(quantity) => quantity,
        None => usize::from(decoder.u16()?),
    };
    let groups = decode_groups(decoder)?;
    let path = selected_path(input, path_slot)?;
    let candidates = group(&groups, path)?;
    let eligible = candidates
        .iter()
        .copied()
        .filter(|candidate| inventory_count(input, inventory, *candidate) == Some(0))
        .collect::<Vec<_>>();
    let selected = select(&eligible, input.random_index(), quantity)?;
    operations.extend(
        selected
            .into_iter()
            .map(|content| ActivityOperation::AddInventory {
                inventory,
                content,
                count: integer(1),
            }),
    );
    Ok(())
}

fn decode_exchange(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let inventory = inventory(decoder.u32()?)?;
    let path_slot = slot(decoder.u32()?)?;
    let lose_quantity = usize::from(decoder.u16()?);
    let gain_quantity = usize::from(decoder.u16()?);
    let lose_candidates = decode_ids(decoder)?;
    let gain_groups = decode_groups(decoder)?;
    if lose_quantity == 0 || gain_quantity == 0 {
        return Err(invalid_payload());
    }
    let lost = select_candidates(
        input,
        inventory,
        &lose_candidates,
        true,
        input.random_index(),
        lose_quantity,
    )?;
    let gain_candidates = if gain_groups.len() == 1 && gain_groups[0].0 == 0 {
        &gain_groups[0].1
    } else {
        group(&gain_groups, selected_path(input, path_slot)?)?
    };
    let eligible = gain_candidates
        .iter()
        .copied()
        .filter(|candidate| {
            inventory_count(input, inventory, *candidate) == Some(0) && !lost.contains(candidate)
        })
        .collect::<Vec<_>>();
    let gained = select(&eligible, input.random_index(), gain_quantity)?;
    operations.extend(
        lost.into_iter()
            .map(|content| ActivityOperation::RemoveInventory {
                inventory,
                content,
                count: integer(1),
            }),
    );
    operations.extend(
        gained
            .into_iter()
            .map(|content| ActivityOperation::AddInventory {
                inventory,
                content,
                count: integer(1),
            }),
    );
    Ok(())
}

fn decode_all_curios(
    input: ActivityHandlerInput<'_>,
    decoder: &mut Decoder<'_>,
    operations: &mut Vec<ActivityOperation>,
) -> Result<(), ActivityHandlerFault> {
    let bindings = decode_bindings(decoder)?;
    let amount = decoder.i64()?;
    let ids = decode_curio_ids(decoder)?;
    if amount <= 0 || ids.is_empty() {
        return Err(invalid_payload());
    }
    let owned = ids
        .iter()
        .copied()
        .filter(|id| inventory_count(input, bindings.inventory, u64::from(id.get())) == Some(1))
        .collect::<Vec<_>>();
    if owned.is_empty() {
        return Err(invalid_state());
    }
    for _ in &owned {
        operations.push(fragment_delta(
            bindings.fragments_slot,
            bindings.inventory,
            amount,
        ));
    }
    for id in owned {
        operations.extend(teardown_operations(id, bindings));
    }
    Ok(())
}

fn blessing_groups(
    catalog: &UniverseCatalog,
    outcome: &OccurrenceOutcome,
    forced_rarities: Option<&[u8]>,
) -> Result<Vec<(u64, Vec<u64>)>, OccurrenceInteractionError> {
    let rarities = forced_rarities.map_or_else(
        || {
            outcome
                .parameter_refs()
                .iter()
                .filter_map(|value| {
                    value
                        .strip_prefix("universe.blessing-pool.rarity.")
                        .and_then(|value| value.parse::<u8>().ok())
                })
                .collect::<Vec<_>>()
        },
        <[u8]>::to_vec,
    );
    let rarities = if rarities.is_empty() {
        vec![1, 2, 3]
    } else {
        rarities
    };
    let groups = catalog
        .paths()
        .iter()
        .map(|path| {
            (
                u64::from(path.id().get()),
                blessings(catalog, &rarities, Some(u64::from(path.id().get()))),
            )
        })
        .collect::<Vec<_>>();
    if groups.iter().any(|(_, group)| group.is_empty()) {
        return Err(OccurrenceInteractionError::InvalidChoice);
    }
    Ok(groups)
}

fn blessings(catalog: &UniverseCatalog, rarities: &[u8], path: Option<u64>) -> Vec<u64> {
    catalog
        .blessings()
        .iter()
        .filter(|value| {
            rarities.contains(&value.rarity())
                && path.is_none_or(|path| u64::from(value.path().get()) == path)
        })
        .map(|value| u64::from(value.id().get()))
        .collect()
}

fn selected_path(
    input: ActivityHandlerInput<'_>,
    slot: ActivitySlotId,
) -> Result<u64, ActivityHandlerFault> {
    input
        .view()
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .and_then(|value| match value.value() {
            ActivityValue::OptionalId(Some(path)) => Some(*path),
            _ => None,
        })
        .ok_or_else(invalid_state)
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

fn select(
    candidates: &[u64],
    random: Option<u32>,
    quantity: usize,
) -> Result<Vec<u64>, ActivityHandlerFault> {
    if quantity == 0 || candidates.len() < quantity {
        return Err(invalid_state());
    }
    let start = random.map_or(0, |value| value as usize % candidates.len());
    Ok((0..quantity)
        .map(|offset| candidates[(start + offset) % candidates.len()])
        .collect())
}

fn group(groups: &[(u64, Vec<u64>)], key: u64) -> Result<&Vec<u64>, ActivityHandlerFault> {
    groups
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, values)| values)
        .ok_or_else(invalid_state)
}

fn group_candidate_count(groups: &[(u64, Vec<u64>)]) -> Option<u32> {
    groups
        .iter()
        .filter_map(|(_, values)| u32::try_from(values.len()).ok())
        .try_fold(1, checked_lcm)
}

fn encode_inventory_path(
    output: &mut Vec<u8>,
    inventory: ActivityInventoryId,
    path_slot: ActivitySlotId,
) {
    output.extend_from_slice(&inventory.get().to_le_bytes());
    output.extend_from_slice(&path_slot.get().to_le_bytes());
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

fn encode_groups(
    output: &mut Vec<u8>,
    groups: Vec<(u64, Vec<u64>)>,
) -> Result<(), OccurrenceInteractionError> {
    output.extend_from_slice(
        &u16::try_from(groups.len())
            .map_err(|_| OccurrenceInteractionError::TooManyCandidates)?
            .to_le_bytes(),
    );
    for (key, ids) in groups {
        output.extend_from_slice(&key.to_le_bytes());
        encode_ids(output, ids)?;
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

fn decode_groups(decoder: &mut Decoder<'_>) -> Result<Vec<(u64, Vec<u64>)>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    if count == 0 {
        return Err(invalid_payload());
    }
    (0..count)
        .map(|_| Ok((decoder.u64()?, decode_ids(decoder)?)))
        .collect()
}

fn decode_curio_ids(decoder: &mut Decoder<'_>) -> Result<Vec<CurioId>, ActivityHandlerFault> {
    let count = usize::from(decoder.u16()?);
    (0..count)
        .map(|_| CurioId::new(decoder.u32()?).ok_or_else(invalid_payload))
        .collect()
}

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
