use std::collections::BTreeMap;

use starclock_activity::{
    ActivityCondition, ActivityExpression, ActivityOperation, ActivityOptionId, ActivityProgramId,
    ActivitySlotId, ActivityStateSource, ActivityValue, SectionId, TechniqueContributionDigest,
};

use super::{BOND_SELECTIONS, BONDS, CurrencyWarsRuntimeError, debug_error, error};
use crate::{CurrencyWarsBondSnapshot, CurrencyWarsEquipmentId, CurrencyWarsNode};

pub(super) fn set_integer(raw: u32, value: i64) -> ActivityOperation {
    set_value(raw, ActivityValue::BoundedInteger(value))
}

pub(super) fn set_value(raw: u32, value: ActivityValue) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot: slot(raw),
        value: ActivityExpression::Literal(value),
    }
}

pub(super) fn set_counter_map(raw: u32, values: Box<[(u64, i64)]>) -> ActivityOperation {
    ActivityOperation::SetCounterMap {
        slot: slot(raw),
        values,
    }
}

pub(super) fn encode_equipment_inventory(
    inventory: &BTreeMap<CurrencyWarsEquipmentId, u32>,
) -> Box<[(u64, i64)]> {
    inventory
        .iter()
        .map(|(id, count)| (u64::from(id.get()), i64::from(*count)))
        .collect()
}

pub(super) fn add_equipment_inventory(
    inventory: &mut BTreeMap<CurrencyWarsEquipmentId, u32>,
    equipment: CurrencyWarsEquipmentId,
) -> Result<(), CurrencyWarsRuntimeError> {
    let count = inventory.entry(equipment).or_default();
    *count = count
        .checked_add(1)
        .ok_or_else(|| error("Currency Wars equipment inventory count overflow"))?;
    Ok(())
}

pub(super) fn bond_operations(snapshot: &CurrencyWarsBondSnapshot) -> Vec<ActivityOperation> {
    vec![
        set_counter_map(
            BONDS,
            snapshot
                .active_bonds
                .iter()
                .map(|bond| (u64::from(bond.id.get()), i64::from(bond.level)))
                .collect(),
        ),
        set_counter_map(
            BOND_SELECTIONS,
            snapshot
                .selected_subtraits
                .iter()
                .map(|(parent, child)| (u64::from(parent.get()), i64::from(child.get())))
                .collect(),
        ),
    ]
}

pub(super) fn remove_equipment_inventory(
    inventory: &mut BTreeMap<CurrencyWarsEquipmentId, u32>,
    equipment: CurrencyWarsEquipmentId,
) -> Result<(), CurrencyWarsRuntimeError> {
    let count = inventory
        .get_mut(&equipment)
        .ok_or_else(|| error("Currency Wars equipment is not in inventory"))?;
    *count = count
        .checked_sub(1)
        .ok_or_else(|| error("Currency Wars equipment inventory count underflow"))?;
    if *count == 0 {
        inventory.remove(&equipment);
    }
    Ok(())
}

pub(super) fn set_ordered_ids(raw: u32, values: Box<[u64]>) -> ActivityOperation {
    ActivityOperation::SetOrderedIdSet {
        slot: slot(raw),
        values,
    }
}

pub(super) fn add_integer(raw: u32, delta: i64) -> ActivityOperation {
    ActivityOperation::AddToSlot {
        slot: slot(raw),
        delta: literal_integer(delta),
    }
}

pub(super) fn literal_integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}

pub(super) fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

pub(super) fn encounter_option(index: usize) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    2_000_000_000_u64
        .checked_add(u64::try_from(index).map_err(debug_error)?)
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars encounter option ID overflow"))
}

pub(super) fn preparation_option(
    index: usize,
) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    3_000_000_000_u64
        .checked_add(u64::try_from(index).map_err(debug_error)?)
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars preparation option ID overflow"))
}

pub(super) fn checkpoint_option(
    index: usize,
) -> Result<ActivityOptionId, CurrencyWarsRuntimeError> {
    4_000_000_000_u64
        .checked_add(u64::try_from(index).map_err(debug_error)?)
        .and_then(ActivityOptionId::new)
        .ok_or_else(|| error("Currency Wars checkpoint option ID overflow"))
}

pub(super) fn supply_option() -> ActivityOptionId {
    ActivityOptionId::new(5_000_000_000).expect("Currency Wars supply option ID is non-zero")
}

pub(super) fn plane_option(plane: u8) -> ActivityOptionId {
    ActivityOptionId::new(6_000_000_000 + u64::from(plane))
        .expect("Currency Wars Plane option ID is non-zero")
}

pub(super) fn program_id(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).expect("Currency Wars boundary program ID is non-zero")
}

pub(super) fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("Currency Wars slot ID is non-zero")
}

pub(super) fn source(raw: u32) -> ActivityStateSource {
    ActivityStateSource::new(u64::from(raw)).expect("Currency Wars state source is non-zero")
}

pub(super) fn section(plane: u8) -> Result<SectionId, CurrencyWarsRuntimeError> {
    SectionId::new(u32::from(plane)).ok_or_else(|| error("Currency Wars Plane is zero"))
}

pub(super) fn contribution(node: &CurrencyWarsNode) -> TechniqueContributionDigest {
    let mut bytes = [0_u8; 32];
    bytes[..4].copy_from_slice(&node.id.get().to_le_bytes());
    TechniqueContributionDigest::new(bytes).expect("Currency Wars node contribution is non-zero")
}

pub(super) fn seed_label(node: &CurrencyWarsNode) -> Box<str> {
    format!("currency-wars/node/{}", node.id.get()).into_boxed_str()
}
