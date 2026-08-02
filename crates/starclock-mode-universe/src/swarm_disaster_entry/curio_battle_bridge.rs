//! Exact shared-Curio projection from Swarm-owned inventory and lifecycle state.

use starclock_activity::{
    ActivityInventoryId, ActivitySlotId, ActivityTransactionState, ActivityValue,
};

use crate::{
    curio::CurioStateKind, curio_runtime::CurioContributionSet, error::UniverseCatalogLoadError,
};

use super::{
    content_runtime::ContentRuntimeCatalog,
    state::{CONTENT, CURIO_INVENTORY},
    validate::{error as invalid, reference},
};

const CURIO_STATE_BASE: u64 = 0x5344_6100_0000_0000;
const CURIO_COUNTER_BASE: u64 = 0x5344_6200_0000_0000;
const ACTIVE: i64 = 1;
const REPAIRING: i64 = 2;
const FIXED: i64 = 3;
const DESTROYED: i64 = 4;
const REPLACED: i64 = 5;

pub(super) fn compile(
    catalog: &ContentRuntimeCatalog,
    state: &ActivityTransactionState,
) -> Result<CurioContributionSet, UniverseCatalogLoadError> {
    let inventory_id = ActivityInventoryId::new(CURIO_INVENTORY)
        .expect("static Swarm Curio inventory is non-zero");
    let owned = state
        .inventory_entries(inventory_id)
        .ok_or_else(|| invalid("missing Swarm Curio inventory"))?;
    let mut inventory = Vec::new();
    let mut states = Vec::new();
    let mut charges = Vec::new();
    for (raw, count) in owned {
        let mode_id =
            u32::try_from(raw).map_err(|_| reference("invalid Swarm Curio inventory identity"))?;
        let mode = catalog.curio(mode_id)?;
        let Some(shared_id) = mode.shared_curio else {
            continue;
        };
        let current = content_counter(state, CURIO_STATE_BASE + u64::from(mode_id))?;
        let kind = match current {
            ACTIVE => CurioStateKind::Active,
            REPAIRING => CurioStateKind::Repairing,
            FIXED => CurioStateKind::Fixed,
            DESTROYED | REPLACED => continue,
            _ => return Err(reference("invalid current Swarm Curio state")),
        };
        let definition = catalog
            .shared_curios
            .definition(shared_id)
            .ok_or_else(|| reference("missing shared Swarm Curio definition"))?;
        let shared_state = definition
            .states()
            .iter()
            .find(|candidate| candidate.kind() == kind)
            .ok_or_else(|| reference("missing matching shared Swarm Curio state"))?;
        inventory.push((shared_id, count));
        states.push((shared_id, shared_state.id()));
        if shared_state.maximum_charges().is_some() {
            let remaining = content_counter(state, CURIO_COUNTER_BASE + u64::from(mode_id))?;
            charges.push((
                shared_id,
                u8::try_from(remaining)
                    .map_err(|_| reference("invalid Swarm Curio charge state"))?,
            ));
        }
    }
    catalog
        .shared_curios
        .contributions_from_owned(&inventory, &states, &charges)
        .map_err(|_| reference("invalid shared Swarm Curio battle contribution"))
}

fn content_counter(
    state: &ActivityTransactionState,
    key: u64,
) -> Result<i64, UniverseCatalogLoadError> {
    let slot = ActivitySlotId::new(CONTENT).expect("static Swarm content slot is non-zero");
    match state.slot(slot) {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values
            .binary_search_by_key(&key, |entry| entry.0)
            .ok()
            .map_or(0, |index| values[index].1)),
        _ => Err(invalid("missing Swarm Curio lifecycle state")),
    }
}
