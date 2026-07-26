//! Generic Activity projections for negative Curio lifecycle mechanics.

use starclock_activity::{ActivityCondition, ActivityExpression, ActivityOperation, ActivityValue};

use super::{CurioActivityBindings, CurioActivityRecord, integer, owned};
use crate::id::CurioId;

pub(crate) const DESTROYED_CURIO_ID_KEY_BASE: u64 = 0x7ffe_0000_0000_0000;
pub(crate) const FISSION_EXTRA_COPY_KEY: u64 = 0x7ffd_0000_0000_0001;

pub(crate) const fn destroyed_curio_key(id: CurioId) -> u64 {
    DESTROYED_CURIO_ID_KEY_BASE | id.get() as u64
}

pub(crate) fn record_destroyed_operations(
    id: CurioId,
    bindings: CurioActivityBindings,
) -> [ActivityOperation; 1] {
    [ActivityOperation::AddCounter {
        slot: bindings.event_slot,
        key: destroyed_curio_key(id),
        delta: integer(1),
    }]
}

pub(crate) fn destroyed_curios(value: &ActivityValue) -> Option<Box<[(CurioId, u32)]>> {
    let ActivityValue::BoundedCounterMap(entries) = value else {
        return None;
    };
    entries
        .iter()
        .filter_map(|(key, count)| {
            if key & 0xffff_0000_0000_0000 != DESTROYED_CURIO_ID_KEY_BASE || *count <= 0 {
                return None;
            }
            let raw = u32::try_from(key & 0xffff_ffff).ok()?;
            Some((CurioId::new(raw)?, u32::try_from(*count).ok()?))
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .into()
}

pub(crate) fn fission_extra_copies(value: &ActivityValue) -> Option<u8> {
    let ActivityValue::BoundedCounterMap(entries) = value else {
        return None;
    };
    entries
        .iter()
        .find(|(key, _)| *key == FISSION_EXTRA_COPY_KEY)
        .map_or(Some(0), |(_, value)| u8::try_from(*value).ok())
}

pub(crate) fn restore_destroyed_operations(
    record: CurioActivityRecord,
    bindings: CurioActivityBindings,
) -> Vec<ActivityOperation> {
    let id = record.id();
    let content = u64::from(id.get());
    let mut operations = vec![
        ActivityOperation::Require(ActivityCondition::LessThan(
            ActivityExpression::InventoryCount {
                inventory: bindings.inventory,
                content,
            },
            integer(1),
        )),
        ActivityOperation::Require(ActivityCondition::LessThan(
            integer(0),
            ActivityExpression::CounterValue {
                slot: bindings.event_slot,
                key: destroyed_curio_key(id),
            },
        )),
        ActivityOperation::AddInventory {
            inventory: bindings.inventory,
            content,
            count: integer(1),
        },
        ActivityOperation::AddCounter {
            slot: bindings.state_slot,
            key: content,
            delta: integer(i64::from(record.initial_state().get())),
        },
    ];
    if record.initial_charges() != 0 {
        operations.push(ActivityOperation::AddCounter {
            slot: bindings.charge_slot,
            key: content,
            delta: integer(i64::from(record.initial_charges())),
        });
    }
    operations.push(ActivityOperation::AddCounter {
        slot: bindings.event_slot,
        key: destroyed_curio_key(id),
        delta: integer(-1),
    });
    operations
}

pub(crate) fn destroyed_available(
    id: CurioId,
    bindings: CurioActivityBindings,
) -> ActivityCondition {
    ActivityCondition::All(
        vec![
            ActivityCondition::LessThan(
                integer(0),
                ActivityExpression::CounterValue {
                    slot: bindings.event_slot,
                    key: destroyed_curio_key(id),
                },
            ),
            ActivityCondition::Not(Box::new(owned(bindings.inventory, u64::from(id.get())))),
        ]
        .into_boxed_slice(),
    )
}
