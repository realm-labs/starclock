//! Checked generic team-resource and typed rule-slot operations.

use crate::{
    ResourceEventData,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    catalog::action::TeamResourceChange,
    event::{
        cause::Cause,
        model::{BattleEventKind, RuleStateEventData},
    },
    id::EventId,
    operation::{ModifyStateSlotOp, ModifyTeamResourceOp},
};

use super::transaction::Transaction;

pub(super) fn execute_modify_team_resource(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: ModifyTeamResourceOp,
) -> Result<EventId, BattleFault> {
    let side = txn
        .state
        .units
        .get(operation.actor)
        .ok_or_else(|| invariant_fault(43))?
        .side;
    let resource = operation.definition.resource();
    let state = txn
        .state
        .teams
        .get(side)
        .keyed(resource)
        .ok_or_else(|| invariant_fault(44))?
        .clone();
    let (attempted, after, overflow) =
        team_resource_update(state.current, state.maximum, operation.definition.change())?;
    txn.set_team_resource(side, resource, after)?;
    Ok(txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Resource(ResourceEventData::TeamResource {
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

/// One checked update calculation for hit-plan and Rule IR resource events.
/// Gain clamps only at the declared maximum and retains attempted overflow;
/// spend and set fail rather than saturate. Amounts fit the event's u16 domain.
/// Callers supply a validated current value no greater than its maximum.
pub(super) fn team_resource_update(
    current: u16,
    maximum: u16,
    change: TeamResourceChange,
) -> Result<(u16, u16, u16), BattleFault> {
    match change {
        TeamResourceChange::Gain(amount) => {
            let uncapped = u32::from(current)
                .checked_add(u32::from(amount))
                .ok_or_else(|| invariant_fault(45))?;
            let after =
                u16::try_from(uncapped.min(u32::from(maximum))).map_err(|_| invariant_fault(45))?;
            Ok((
                amount,
                after,
                u16::try_from(
                    uncapped
                        .checked_sub(u32::from(after))
                        .ok_or_else(|| invariant_fault(46))?,
                )
                .map_err(|_| invariant_fault(46))?,
            ))
        }
        TeamResourceChange::Spend(amount) => Ok((
            amount,
            current
                .checked_sub(amount)
                .ok_or_else(|| invariant_fault(47))?,
            0,
        )),
        TeamResourceChange::Set(value) => {
            if value > maximum {
                return Err(invariant_fault(48));
            }
            Ok((value, value, 0))
        }
    }
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

#[cfg(test)]
mod tests {
    use super::team_resource_update;
    use crate::catalog::action::TeamResourceChange::{Gain, Set, Spend};

    #[test]
    fn team_resource_update_preserves_attempted_effective_and_overflow_boundaries() {
        for (before, maximum, change, expected) in [
            (0, 0, Gain(0), (0, 0, 0)),
            (0, 0, Gain(2), (2, 0, 2)),
            (0, 5, Gain(2), (2, 2, 0)),
            (4, 5, Gain(2), (2, 5, 1)),
            (5, 5, Gain(2), (2, 5, 2)),
            (
                u16::MAX,
                u16::MAX,
                Gain(u16::MAX),
                (u16::MAX, u16::MAX, u16::MAX),
            ),
            (2, 5, Spend(2), (2, 0, 0)),
            (2, 5, Set(5), (5, 5, 0)),
            (2, 5, Set(0), (0, 0, 0)),
        ] {
            assert_eq!(
                team_resource_update(before, maximum, change).unwrap(),
                expected
            );
        }
        assert!(team_resource_update(1, 5, Spend(2)).is_err());
        assert!(team_resource_update(1, 5, Set(6)).is_err());
    }
}
