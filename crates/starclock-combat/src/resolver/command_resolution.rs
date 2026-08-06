use super::{
    action::{drain_reactions, execute_action_plan},
    operation as parent_operation,
    settle::{ActionBoundary, settle_after_action},
    transaction::{FaultInjection, FaultInjectionPoint, Transaction, action_fault},
    turn,
};
use crate::{
    AbilityId, UnitId,
    action::{
        lower::{TimelineActionContext, lower_normal_action, lower_ultimate_action},
        model::ActionPlan,
    },
    battle::{
        fault::{BattleFault, FaultBoundary, FaultKind},
        model::BattlePhase,
        spec::TeamSide as SpecTeamSide,
    },
    catalog::{CombatCatalog, action::ReactionBoundary},
    command::{legal, validate::ValidatedCommand},
    event::{
        cause::{Cause, CauseActor},
        model::{ActionBoundaryEventData, BattleEventData, BattleEventKind, DecisionEventData},
    },
    id::{CommandId, EventId, SourceDefinitionId},
    target::{model::TargetCommitment, select},
    timeline::state::PreparedActionState,
};

pub(super) fn execute(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    command: ValidatedCommand,
    injection: Option<FaultInjection>,
) -> Result<(), BattleFault> {
    txn.set_phase(BattlePhase::Resolving);
    maybe_inject(injection, FaultInjectionPoint::AfterResolvingPhase)?;

    match command {
        ValidatedCommand::StartBattle => turn::start_battle(catalog, txn, root)?,
        ValidatedCommand::Advance => {
            let boundary = txn
                .state
                .timeline
                .boundary
                .clone()
                .ok_or_else(|| action_fault(1))?;
            turn::advance_action_boundary(catalog, txn, root, boundary)?;
        }
        ValidatedCommand::UseAbility {
            actor,
            ability,
            primary_target,
        } => execute_normal_action(catalog, txn, root, actor, ability, primary_target)?,
        ValidatedCommand::RequestUltimate { actor, ability } => {
            request_ultimate(catalog, txn, root, actor, ability)?;
        }
        ValidatedCommand::CommitPreparedAction { primary_target } => {
            commit_prepared_action(catalog, txn, root, primary_target)?;
        }
        ValidatedCommand::CancelPreparedAction => cancel_prepared_action(catalog, txn, root)?,
        ValidatedCommand::Concede => concede(txn, root)?,
    }
    maybe_inject(injection, FaultInjectionPoint::AfterCommandMutation)?;
    txn.bump_revision()?;
    Ok(())
}

fn execute_normal_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    actor: UnitId,
    ability: AbilityId,
    primary_target: Option<UnitId>,
) -> Result<(), BattleFault> {
    let closed = close_active_decision(txn, root)?;
    let turn = txn
        .state
        .timeline
        .active_turn
        .ok_or_else(|| action_fault(3))?;
    if turn.owner != actor {
        return Err(action_fault(4));
    }
    txn.set_action_boundary(None);
    let targets = commit_targets(catalog, txn, actor, ability, primary_target)?;
    let owner =
        legal::ability_owner(txn.state, catalog, actor, ability).ok_or_else(|| action_fault(5))?;
    let mut plan = lower_normal_action(
        catalog,
        txn,
        TimelineActionContext {
            actor,
            owner,
            timeline_actor: turn.actor,
            origin: turn.origin,
        },
        ability,
        targets,
    )
    .ok_or_else(|| action_fault(5))?;
    let action_resolved = execute_action_plan(catalog, txn, root, closed, &mut plan)?;
    let boundary_cause = action_cause(root, &plan)?;
    let action_resolved = parent_operation::settle_effects_at_action_end(
        catalog,
        txn,
        boundary_cause,
        action_resolved,
    )?;
    let action_resolved =
        drain_reactions(catalog, txn, ReactionBoundary::AfterAction, action_resolved)?;
    turn::pause_completed_turn(
        catalog,
        txn,
        root,
        action_resolved,
        turn::TurnCompletion::selected(turn, boundary_cause),
    )
}

fn request_ultimate(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    actor: UnitId,
    ability: AbilityId,
) -> Result<(), BattleFault> {
    let boundary = txn
        .state
        .timeline
        .boundary
        .clone()
        .ok_or_else(|| action_fault(11))?;
    let closed = if txn.state.decision.is_some() {
        Some(close_active_decision(txn, root)?)
    } else {
        None
    };
    let cause = closed.map_or_else(
        || Cause::root(root),
        |parent| Cause::root(root).with_parent(parent),
    );
    let parent = txn.emit(
        cause,
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::UltimateRequested {
            boundary: boundary.id,
            actor,
            ability,
        }),
    );
    txn.set_decision(None);
    txn.set_action_boundary(None);
    let prepared = PreparedActionState {
        id: txn.allocate_prepared_action(),
        actor,
        ability,
        boundary,
    };
    txn.set_prepared_action(Some(prepared));
    let decision = txn.allocate_decision();
    let offered = legal::prepared_action(decision, actor, ability, catalog, txn.state)
        .ok_or_else(|| action_fault(12))?;
    turn::offer_decision(txn, root, Some(parent), offered);
    Ok(())
}

fn commit_prepared_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    primary_target: Option<UnitId>,
) -> Result<(), BattleFault> {
    let parent = close_active_decision(txn, root)?;
    let prepared = txn
        .state
        .timeline
        .prepared_action
        .clone()
        .ok_or_else(|| action_fault(12))?;
    txn.set_prepared_action(None);
    let actor = prepared.actor;
    let ability = prepared.ability;
    let targets = commit_targets(catalog, txn, actor, ability, primary_target)?;
    let owner =
        legal::ability_owner(txn.state, catalog, actor, ability).ok_or_else(|| action_fault(12))?;
    let mut plan = lower_ultimate_action(catalog, txn, actor, owner, ability, targets)
        .ok_or_else(|| action_fault(12))?;
    let resolved = execute_action_plan(catalog, txn, root, parent, &mut plan)?;
    let boundary_cause = action_cause(root, &plan)?;
    let resolved =
        parent_operation::settle_effects_at_action_end(catalog, txn, boundary_cause, resolved)?;
    let resolved = drain_reactions(catalog, txn, ReactionBoundary::AfterAction, resolved)?;
    if let ActionBoundary::Continue(parent) =
        settle_after_action(catalog, txn, boundary_cause, resolved)?
    {
        turn::enter_action_boundary(
            catalog,
            txn,
            root,
            parent,
            prepared.boundary.turn,
            prepared.boundary.continuation,
        )?;
    }
    Ok(())
}

fn cancel_prepared_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
) -> Result<(), BattleFault> {
    let closed = close_active_decision(txn, root)?;
    let prepared = txn
        .state
        .timeline
        .prepared_action
        .clone()
        .ok_or_else(|| action_fault(13))?;
    txn.set_prepared_action(None);
    let cancelled = txn.emit(
        Cause::root(root).with_parent(closed),
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::UltimateCancelled {
            boundary: prepared.boundary.id,
            actor: prepared.actor,
            ability: prepared.ability,
        }),
    );
    turn::enter_action_boundary(
        catalog,
        txn,
        root,
        cancelled,
        prepared.boundary.turn,
        prepared.boundary.continuation,
    )
}

fn concede(txn: &mut Transaction<'_>, root: CommandId) -> Result<(), BattleFault> {
    let closed = close_active_decision(txn, root)?;
    txn.set_decision(None);
    txn.set_action_boundary(None);
    txn.set_prepared_action(None);
    txn.clear_extra_turns();
    txn.clear_reactions();
    txn.set_phase(BattlePhase::Lost);
    txn.emit(
        Cause::root(root).with_parent(closed),
        BattleEventKind::Battle(BattleEventData::Conceded {
            side: SpecTeamSide::Player,
        }),
    );
    Ok(())
}

pub(super) fn action_cause(root: CommandId, plan: &ActionPlan) -> Result<Cause, BattleFault> {
    let source = SourceDefinitionId::new(plan.ability.get()).ok_or_else(|| action_fault(42))?;
    Ok(Cause::for_action(
        root,
        plan.id,
        plan.owner,
        CauseActor::Unit(plan.actor),
        source,
    )
    .with_primary_target(plan.targets.primary)
    .with_applier(plan.owner))
}

pub(super) fn commit_targets(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    actor: UnitId,
    ability: AbilityId,
    primary: Option<UnitId>,
) -> Result<TargetCommitment, BattleFault> {
    let definition = catalog.ability(ability).ok_or_else(|| action_fault(14))?;
    let action = definition.action().ok_or_else(|| action_fault(15))?;
    let selector = catalog
        .selector(definition.selector())
        .and_then(|definition| definition.unit_targets())
        .ok_or_else(|| action_fault(16))?;
    select::commit(
        &txn.state.units,
        &txn.state.formations,
        actor,
        selector,
        action.invalidation(),
        primary,
    )
    .map_err(|_| action_fault(17))
}

fn close_active_decision(
    txn: &mut Transaction<'_>,
    root: CommandId,
) -> Result<EventId, BattleFault> {
    let decision = txn
        .state
        .decision
        .as_ref()
        .ok_or_else(|| action_fault(10))?
        .id();
    Ok(txn.emit(
        Cause::root(root),
        BattleEventKind::Decision(DecisionEventData::Closed { decision }),
    ))
}

fn maybe_inject(
    injection: Option<FaultInjection>,
    point: FaultInjectionPoint,
) -> Result<(), BattleFault> {
    match injection {
        Some(injection) if injection.point == point => Err(BattleFault::new(
            FaultKind::InvariantViolation,
            FaultBoundary::Command,
            injection.policy,
            0xF001,
            Some(7),
        )),
        _ => Ok(()),
    }
}
