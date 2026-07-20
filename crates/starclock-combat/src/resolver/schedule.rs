//! Cause-relative lowering into the deterministic reaction queue.

use crate::{
    BattleEventKind, EventId,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    catalog::action::{QueuedActor, QueuedTarget},
    event::cause::{Cause, CauseActor},
    operation::QueueActionOp,
};

use super::transaction::Transaction;

pub(super) fn execute_queue_action(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: QueueActionOp,
) -> Result<EventId, BattleFault> {
    let definition = operation.definition;
    let actor = match definition.actor() {
        QueuedActor::CauseOwner => cause.owner(),
        QueuedActor::CauseApplier => cause.applier(),
        QueuedActor::PrimaryTarget => cause.primary_target(),
    }
    .ok_or_else(|| invariant_fault(50))?;
    let primary = match definition.target() {
        QueuedTarget::CauseActor => match cause.actor() {
            Some(CauseActor::Unit(unit)) => Some(unit),
            _ => return Err(invariant_fault(51)),
        },
        QueuedTarget::CauseOwner => cause.owner(),
        QueuedTarget::CauseApplier => cause.applier(),
        QueuedTarget::PrimaryTarget => cause.primary_target(),
        QueuedTarget::None => None,
    };
    let ability = catalog
        .ability(definition.ability())
        .ok_or_else(|| invariant_fault(52))?;
    let action = ability.action().ok_or_else(|| invariant_fault(53))?;
    let selector = catalog
        .selector(ability.selector())
        .and_then(|selector| selector.unit_targets())
        .ok_or_else(|| invariant_fault(54))?;
    let targets = crate::target::select::commit(
        &txn.state.units,
        &txn.state.formations,
        actor,
        selector,
        action.invalidation(),
        primary,
    )
    .map_err(|_| invariant_fault(55))?;
    let (side, formation, spawn) = txn
        .state
        .units
        .get(actor)
        .map(|unit| (unit.side, unit.formation, unit.spawn))
        .ok_or_else(|| invariant_fault(56))?;
    let insertion = txn.allocate_reaction();
    let queued = txn.emit(
        cause.with_parent(parent),
        BattleEventKind::Action(crate::ActionEventData::Queued {
            insertion,
            actor,
            ability: definition.ability(),
            origin: definition.origin(),
            boundary: definition.boundary(),
        }),
    );
    txn.reactions.push(crate::reaction::queue::QueuedAction {
        order: crate::reaction::queue::ReactionOrder {
            boundary: definition.boundary(),
            priority: definition.priority(),
            side,
            formation,
            spawn,
            source: crate::SourceDefinitionId::new(definition.ability().get())
                .ok_or_else(|| invariant_fault(57))?,
            rule: None,
            instance: None,
            trigger: None,
            actor,
            ability: definition.ability(),
            insertion,
        },
        root: cause.root_command(),
        parent: queued,
        actor,
        ability: definition.ability(),
        origin: definition.origin(),
        targets,
    });
    Ok(queued)
}

fn invariant_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3200 + context,
        None,
    )
}
