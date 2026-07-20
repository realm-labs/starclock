//! Checked generic team-resource and typed rule-slot operations.

use crate::{
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    event::{
        cause::Cause,
        model::{BattleEventKind, RuleStateEventData},
    },
    id::EventId,
    operation::ModifyStateSlotOp,
};

use super::transaction::Transaction;

pub(super) fn execute_modify_team_resource(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: crate::operation::ModifyTeamResourceOp,
) -> Result<EventId, BattleFault> {
    let side = txn
        .state
        .units
        .get(operation.actor)
        .ok_or_else(|| invariant_fault(43))?
        .side;
    let resource = operation.definition.resource();
    let state = *txn
        .state
        .teams
        .get(side)
        .keyed(resource)
        .ok_or_else(|| invariant_fault(44))?;
    let (attempted, after, overflow) = match operation.definition.change() {
        crate::catalog::action::TeamResourceChange::Gain(amount) => {
            let uncapped = u32::from(state.current) + u32::from(amount);
            let after = u16::try_from(uncapped.min(u32::from(state.maximum)))
                .map_err(|_| invariant_fault(45))?;
            (
                amount,
                after,
                u16::try_from(uncapped - u32::from(after)).map_err(|_| invariant_fault(46))?,
            )
        }
        crate::catalog::action::TeamResourceChange::Spend(amount) => (
            amount,
            state
                .current
                .checked_sub(amount)
                .ok_or_else(|| invariant_fault(47))?,
            0,
        ),
        crate::catalog::action::TeamResourceChange::Set(value) => {
            if value > state.maximum {
                return Err(invariant_fault(48));
            }
            (value, value, 0)
        }
    };
    txn.set_team_resource(side, resource, after)?;
    Ok(txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Resource(crate::ResourceEventData::TeamResource {
            side,
            resource,
            attempted,
            effective: state.current.abs_diff(after),
            before: state.current,
            after,
            overflow,
        }),
    ))
}

pub(super) fn execute_modify_state_slot(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: ModifyStateSlotOp,
) -> Result<EventId, BattleFault> {
    let instance = operation.instance.or_else(|| {
        txn.state
            .rules
            .instance_for(operation.owner, operation.definition.rule)
    });
    let instance = instance.ok_or_else(|| invariant_fault(41))?;
    let (before, after) = txn
        .state
        .rules
        .update(
            instance,
            operation.definition.slot,
            operation.definition.update,
            operation.definition.value,
        )
        .map_err(|_| invariant_fault(42))?;
    txn.record_rule_state_change(instance, operation.definition.slot, &before, &after);
    Ok(txn.emit(
        cause
            .with_parent(parent)
            .with_primary_target(Some(operation.owner)),
        BattleEventKind::RuleState(RuleStateEventData {
            operation: operation.id,
            instance,
            slot: operation.definition.slot,
            before,
            after,
        }),
    ))
}

fn invariant_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3280 + context,
        None,
    )
}
