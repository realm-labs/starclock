//! Domain-entry conditions and settlements contributed by Curios.

use starclock_activity::{ActivityCondition, ActivityExpression, ActivityOperation};

use super::{
    CurioActivityBindings, destroy_and_count_operations, event_key, fragment_gain, integer, owned,
};
use crate::{curio_effect_runtime::CurioEvent, id::CurioId};

pub(crate) fn gossip_condition(bindings: CurioActivityBindings) -> ActivityCondition {
    owned(bindings.inventory, 110)
}

pub(crate) fn cogwheel_condition(bindings: CurioActivityBindings) -> ActivityCondition {
    owned(bindings.inventory, 10)
}

pub(crate) fn gold_coin_condition(bindings: CurioActivityBindings) -> ActivityCondition {
    owned(bindings.inventory, 21)
}

pub(crate) fn perpetual_motion_condition(bindings: CurioActivityBindings) -> ActivityCondition {
    super::active_condition(
        CurioId::new(67).expect("Perpetual Motion Cuckoo Clock ID is non-zero"),
        bindings,
    )
}

pub(crate) fn sealing_wax_condition(
    bindings: CurioActivityBindings,
    content: u64,
) -> ActivityCondition {
    owned(bindings.inventory, content)
}

pub(crate) fn gold_coin_domain_entry_settlement(
    bindings: CurioActivityBindings,
    finish: &[ActivityOperation],
) -> Vec<ActivityOperation> {
    let id = CurioId::new(21).expect("Gold Coin Curio ID is non-zero");
    vec![
        fragment_gain(
            bindings,
            ActivityExpression::Divide(
                Box::new(ActivityExpression::Multiply(
                    Box::new(ActivityExpression::Slot(bindings.fragments_slot)),
                    Box::new(integer(6)),
                )),
                Box::new(integer(100)),
            ),
        ),
        ActivityOperation::AddCounter {
            slot: bindings.event_slot,
            key: event_key(id, CurioEvent::DomainEntered),
            delta: integer(1),
        },
    ]
    .into_iter()
    .chain(finish.iter().cloned())
    .collect()
}

pub(crate) fn cogwheel_domain_entry_settlement(
    bindings: CurioActivityBindings,
    finish: &[ActivityOperation],
) -> Vec<ActivityOperation> {
    let id = CurioId::new(10).expect("Cogwheel Curio ID is non-zero");
    let mut operations = vec![
        fragment_gain(bindings, integer(50)),
        ActivityOperation::AddCounter {
            slot: bindings.event_slot,
            key: event_key(id, CurioEvent::DomainEntered),
            delta: integer(1),
        },
    ];
    let mut destroy = destroy_and_count_operations(id, bindings);
    destroy.push(ActivityOperation::SetSlot {
        slot: bindings.fragments_slot,
        value: integer(0),
    });
    destroy.extend_from_slice(finish);
    operations.push(ActivityOperation::Conditional {
        condition: ActivityCondition::LessThan(
            integer(500),
            ActivityExpression::Slot(bindings.fragments_slot),
        ),
        if_true: destroy.into_boxed_slice(),
        if_false: finish.to_vec().into_boxed_slice(),
    });
    operations
}

pub(crate) fn perpetual_motion_domain_entry_settlement(
    bindings: CurioActivityBindings,
    finish: &[ActivityOperation],
) -> Vec<ActivityOperation> {
    let id = CurioId::new(67).expect("Perpetual Motion Cuckoo Clock ID is non-zero");
    vec![
        ActivityOperation::AddToSlot {
            slot: bindings.fragments_slot,
            delta: ActivityExpression::Negate(Box::new(ActivityExpression::Divide(
                Box::new(ActivityExpression::Multiply(
                    Box::new(ActivityExpression::Slot(bindings.fragments_slot)),
                    Box::new(integer(5)),
                )),
                Box::new(integer(100)),
            ))),
        },
        ActivityOperation::AddCounter {
            slot: bindings.event_slot,
            key: event_key(id, CurioEvent::DomainEntered),
            delta: integer(1),
        },
    ]
    .into_iter()
    .chain(finish.iter().cloned())
    .collect()
}

#[cfg(test)]
mod tests {
    use starclock_activity::{ActivityInventoryId, ActivitySlotId};

    use super::*;

    #[test]
    fn perpetual_motion_debits_five_percent_before_recording_domain_entry() {
        let bindings = CurioActivityBindings {
            inventory: ActivityInventoryId::new(1).unwrap(),
            state_slot: ActivitySlotId::new(1).unwrap(),
            charge_slot: ActivitySlotId::new(2).unwrap(),
            event_slot: ActivitySlotId::new(3).unwrap(),
            fragments_slot: ActivitySlotId::new(4).unwrap(),
        };
        let operations = perpetual_motion_domain_entry_settlement(bindings, &[]);
        assert_eq!(operations.len(), 2);
        assert_eq!(
            operations[0],
            ActivityOperation::AddToSlot {
                slot: bindings.fragments_slot,
                delta: ActivityExpression::Negate(Box::new(ActivityExpression::Divide(
                    Box::new(ActivityExpression::Multiply(
                        Box::new(ActivityExpression::Slot(bindings.fragments_slot)),
                        Box::new(integer(5)),
                    )),
                    Box::new(integer(100)),
                ))),
            }
        );
        assert!(matches!(
            operations[1],
            ActivityOperation::AddCounter { slot, .. } if slot == bindings.event_slot
        ));
    }
}
