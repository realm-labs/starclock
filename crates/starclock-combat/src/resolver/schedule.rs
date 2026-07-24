//! Cause-relative lowering into the deterministic reaction queue.

use crate::{
    BattleEventKind, EventId,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    catalog::action::{QueuedActor, QueuedOwner, QueuedTarget},
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
        QueuedActor::SharedEntity(kind) => {
            let provider = cause
                .owner()
                .or(cause.applier())
                .ok_or_else(|| invariant_fault(58))?;
            let side = txn
                .state
                .units
                .get(provider)
                .ok_or_else(|| invariant_fault(59))?
                .side;
            let mut matches = txn
                .state
                .links
                .canonical_entries()
                .iter()
                .filter_map(|link| {
                    if !link.active || link.kind != kind {
                        return None;
                    }
                    let crate::LinkedEntity::Unit(unit) = link.entity else {
                        return None;
                    };
                    txn.state
                        .units
                        .get(unit)
                        .filter(|state| state.side == side)
                        .map(|_| unit)
                });
            let actor = matches.next().ok_or_else(|| invariant_fault(60))?;
            if matches.next().is_some() {
                return Err(invariant_fault(61));
            }
            Some(actor)
        }
    }
    .ok_or_else(|| invariant_fault(50))?;
    let owner = match definition.owner() {
        QueuedOwner::Actor => Some(actor),
        QueuedOwner::CauseOwner => cause.owner(),
        QueuedOwner::CauseApplier => cause.applier(),
    }
    .ok_or_else(|| invariant_fault(62))?;
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
    enqueue(
        txn,
        cause,
        parent,
        actor,
        owner,
        definition.ability(),
        definition.origin(),
        definition.boundary(),
        definition.priority(),
        crate::SourceDefinitionId::new(definition.ability().get())
            .ok_or_else(|| invariant_fault(57))?,
        None,
        None,
        None,
        targets,
        definition.payment(),
    )
}

pub(super) fn execute_queue_rule_action(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: crate::operation::QueueRuleActionOp,
) -> Result<EventId, BattleFault> {
    let ability = catalog
        .ability(operation.ability)
        .ok_or_else(|| invariant_fault(63))?;
    let action = ability.action().ok_or_else(|| invariant_fault(64))?;
    let selector = catalog
        .selector(ability.selector())
        .and_then(|definition| definition.unit_targets())
        .ok_or_else(|| invariant_fault(65))?;
    for actor in operation.actors {
        let primary = match (selector.relation(), selector.pattern()) {
            (crate::catalog::action::TargetRelation::SelfUnit, _)
            | (_, crate::catalog::action::TargetPattern::All) => None,
            (
                _,
                crate::catalog::action::TargetPattern::Single
                | crate::catalog::action::TargetPattern::Blast,
            ) => operation.targets.first().copied(),
        };
        let targets = crate::target::select::commit(
            &txn.state.units,
            &txn.state.formations,
            actor,
            selector,
            action.invalidation(),
            primary,
        )
        .map_err(|_| invariant_fault(66))?;
        if targets.targets.as_ref() != operation.targets.as_ref() {
            return Err(invariant_fault(67));
        }
        parent = enqueue(
            txn,
            cause,
            parent,
            actor,
            operation.owner,
            operation.ability,
            operation.origin,
            operation.boundary,
            operation.priority,
            operation.source,
            Some(operation.rule),
            Some(operation.instance),
            Some(operation.trigger),
            targets,
            operation.payment,
        )?;
    }
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn enqueue(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    actor: crate::UnitId,
    owner: crate::UnitId,
    ability: crate::AbilityId,
    origin: crate::ActionOrigin,
    boundary: crate::catalog::action::ReactionBoundary,
    priority: i16,
    source: crate::SourceDefinitionId,
    rule: Option<crate::RuleId>,
    instance: Option<crate::RuleInstanceId>,
    trigger: Option<crate::TriggerId>,
    targets: crate::target::model::TargetCommitment,
    payment: Option<crate::catalog::action::SkillPointPaymentPolicy>,
) -> Result<EventId, BattleFault> {
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
            ability,
            origin,
            boundary,
        }),
    );
    txn.reactions.push(crate::reaction::queue::QueuedAction {
        order: crate::reaction::queue::ReactionOrder {
            boundary,
            priority,
            side,
            formation,
            spawn,
            source,
            rule,
            instance,
            trigger,
            actor,
            ability,
            insertion,
        },
        root: cause.root_command(),
        parent: queued,
        actor,
        owner,
        ability,
        origin,
        targets,
        payment,
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
