//! Cause-relative lowering into the deterministic reaction queue.

use crate::{
    AbilityId, ActionEventData, ActionOrigin, BattleEventKind, DiagnosticRecord, EventId,
    LinkedEntity, RuleId, RuleInstanceId, SourceDefinitionId, TriggerId, UnitId,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    catalog::{
        CombatCatalog,
        action::{
            QueuedActor, QueuedOwner, QueuedTarget, ReactionBoundary, SkillPointPaymentPolicy,
            TargetPattern, TargetRelation,
        },
    },
    event::cause::{Cause, CauseActor},
    operation::{QueueActionOp, QueueRuleActionOp},
    reaction::queue::{QueuedAction, ReactionOrder, ReactionTier},
    target::{model::TargetCommitment, select::commit},
};

use super::transaction::Transaction;

pub(super) fn execute_queue_action(
    catalog: &CombatCatalog,
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
                    let LinkedEntity::Unit(unit) = link.entity else {
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
    let targets = commit(
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
        SourceDefinitionId::new(definition.ability().get()).ok_or_else(|| invariant_fault(57))?,
        None,
        None,
        None,
        targets,
        definition.payment(),
    )
}

pub(super) fn execute_queue_rule_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: QueueRuleActionOp,
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
            (TargetRelation::SelfUnit, _) | (_, TargetPattern::All) => None,
            (_, TargetPattern::Single | TargetPattern::Blast) => operation.targets.first().copied(),
        };
        let targets = commit(
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
            operation.rule,
            operation.instance,
            operation.trigger,
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
    actor: UnitId,
    owner: UnitId,
    ability: AbilityId,
    origin: ActionOrigin,
    boundary: ReactionBoundary,
    priority: i16,
    source: SourceDefinitionId,
    rule: Option<RuleId>,
    instance: Option<RuleInstanceId>,
    trigger: Option<TriggerId>,
    targets: TargetCommitment,
    payment: Option<SkillPointPaymentPolicy>,
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
        BattleEventKind::Action(ActionEventData::Queued {
            insertion,
            actor,
            ability,
            origin,
            boundary,
        }),
    );
    let order = ReactionOrder {
        boundary,
        tier: ReactionTier::for_origin(origin),
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
    };
    txn.record_diagnostic(|| DiagnosticRecord::ReactionQueued {
        event: queued,
        actor,
        owner,
        ability,
        origin,
        order: order.into(),
        targets: (&targets).into(),
    });
    txn.reactions.push(QueuedAction {
        order,
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
