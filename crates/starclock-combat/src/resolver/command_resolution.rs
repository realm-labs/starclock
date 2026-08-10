use super::{
    action::drain_reactions,
    action_execution::execute_action_plan,
    operation as parent_operation,
    settle::{ActionBoundary, settle_after_action},
    transaction::{FaultInjection, FaultInjectionPoint, Transaction, action_fault},
    turn,
};
use crate::{
    AbilityId, UnitId,
    action::{
        lower::{
            TimelineActionContext, lower_action_segment, lower_normal_action,
            lower_segmented_ultimate_header, lower_ultimate_action,
            restore_segmented_ultimate_header,
        },
        model::ActionPlan,
    },
    battle::{
        fault::{BattleFault, FaultBoundary, FaultKind},
        model::BattlePhase,
        spec::TeamSide as SpecTeamSide,
    },
    catalog::{
        CombatCatalog,
        action::{
            ActionSegmentDefinition, AutomaticSegmentTarget, InitialActionSegment, ReactionBoundary,
        },
    },
    command::{legal, model::ActionFrameInput, validate::ValidatedCommand},
    event::{
        cause::{Cause, CauseActor},
        model::{ActionBoundaryEventData, BattleEventData, BattleEventKind, DecisionEventData},
    },
    id::{CommandId, EventId, SourceDefinitionId},
    target::{model::TargetCommitment, select},
    timeline::state::{ActionFrameState, PreparedActionState},
};

use super::action_execution::{
    execute_action_segment, execute_parent_action_segment, finish_action_envelope,
    start_action_envelope,
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
        ValidatedCommand::CommitActionFrame { input } => {
            commit_action_frame(catalog, txn, root, input)?;
        }
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
    let plan = lower_normal_action(
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
    let action_resolved = execute_action_plan(catalog, txn, root, closed, plan)?;
    let boundary_cause = action_cause(root, &action_resolved.plan)?;
    let action_resolved = parent_operation::settle_effects_at_action_end(
        catalog,
        txn,
        boundary_cause,
        action_resolved.parent,
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
    if catalog
        .ability(ability)
        .and_then(|definition| definition.action())
        .and_then(|action| action.segmented_flow())
        .is_some()
    {
        return start_segmented_action(catalog, txn, root, parent, prepared, owner, targets);
    }
    let plan = lower_ultimate_action(catalog, txn, actor, owner, ability, targets)
        .ok_or_else(|| action_fault(12))?;
    let resolved = execute_action_plan(catalog, txn, root, parent, plan)?;
    let boundary_cause = action_cause(root, &resolved.plan)?;
    let resolved = parent_operation::settle_effects_at_action_end(
        catalog,
        txn,
        boundary_cause,
        resolved.parent,
    )?;
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

fn start_segmented_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    prepared: PreparedActionState,
    owner: UnitId,
    targets: TargetCommitment,
) -> Result<(), BattleFault> {
    let ability = prepared.ability;
    let actor = prepared.actor;
    let initial = catalog
        .ability(ability)
        .and_then(|definition| definition.action())
        .and_then(|action| action.segmented_flow())
        .map(|flow| flow.initial())
        .ok_or_else(|| action_fault(81))?;
    let header =
        lower_segmented_ultimate_header(catalog, txn, actor, owner, ability, targets.clone())
            .ok_or_else(|| action_fault(81))?;
    let started = start_action_envelope(catalog, txn, root, parent, header)?;
    let mut frame = ActionFrameState {
        id: txn.allocate_action_frame(started.plan.id),
        action: started.plan.id,
        actor,
        owner,
        ability,
        boundary: prepared.boundary,
        cursor: 0,
        retained_targets: targets.clone(),
        inputs: targets
            .primary
            .map(ActionFrameInput::Target)
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        parent: started.parent,
        paid: true,
    };
    frame.parent = txn.emit(
        Cause::root(root).with_parent(frame.parent),
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::ActionFrameOpened {
            boundary: frame.boundary.id,
            frame: frame.id,
            action: frame.action,
        }),
    );
    if initial == InitialActionSegment::ExecuteParent {
        let plan = lower_action_segment(
            catalog,
            txn,
            frame.action,
            frame.actor,
            frame.owner,
            ability,
            targets,
        )
        .ok_or_else(|| action_fault(84))?;
        frame.parent =
            execute_parent_action_segment(catalog, txn, root, frame.parent, plan)?.parent;
    }
    advance_action_frame(catalog, txn, root, frame)
}

fn commit_action_frame(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    input: ActionFrameInput,
) -> Result<(), BattleFault> {
    let closed = close_active_decision(txn, root)?;
    let mut frame = txn
        .state
        .timeline
        .action_frame
        .clone()
        .ok_or_else(|| action_fault(82))?;
    txn.set_action_frame(None);
    frame.parent = closed;
    let flow = catalog
        .ability(frame.ability)
        .and_then(|definition| definition.action())
        .and_then(|action| action.segmented_flow())
        .ok_or_else(|| action_fault(82))?;
    let step = flow
        .steps()
        .get(usize::from(frame.cursor))
        .ok_or_else(|| action_fault(82))?;
    let (ability, targets) = match (step, input) {
        (ActionSegmentDefinition::SelectTarget { ability }, ActionFrameInput::Target(target)) => (
            *ability,
            commit_targets(catalog, txn, frame.actor, *ability, Some(target))?,
        ),
        (
            ActionSegmentDefinition::SelectOption { abilities },
            ActionFrameInput::Option(ability),
        ) if abilities.binary_search(&ability).is_ok() => (ability, frame.retained_targets.clone()),
        _ => return Err(action_fault(82)),
    };
    frame.parent = txn.emit(
        Cause::root(root).with_parent(frame.parent),
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::ActionFrameInputCommitted {
            frame: frame.id,
            action: frame.action,
            cursor: frame.cursor,
            input,
        }),
    );
    frame.parent = execute_segment(catalog, txn, root, frame.parent, &frame, ability, targets)?;
    let mut inputs = frame.inputs.into_vec();
    inputs.push(input);
    frame.inputs = inputs.into_boxed_slice();
    frame.cursor = frame
        .cursor
        .checked_add(1)
        .ok_or_else(|| action_fault(82))?;
    advance_action_frame(catalog, txn, root, frame)
}

fn advance_action_frame(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    mut frame: ActionFrameState,
) -> Result<(), BattleFault> {
    loop {
        let step = catalog
            .ability(frame.ability)
            .and_then(|definition| definition.action())
            .and_then(|action| action.segmented_flow())
            .and_then(|flow| flow.steps().get(usize::from(frame.cursor)))
            .cloned();
        match step {
            Some(ActionSegmentDefinition::Automatic { ability, target }) => {
                let targets = match target {
                    AutomaticSegmentTarget::Retained => frame.retained_targets.clone(),
                    AutomaticSegmentTarget::AbilitySelector => {
                        commit_targets(catalog, txn, frame.actor, ability, None)?
                    }
                };
                frame.parent =
                    execute_segment(catalog, txn, root, frame.parent, &frame, ability, targets)?;
                frame.cursor = frame
                    .cursor
                    .checked_add(1)
                    .ok_or_else(|| action_fault(83))?;
            }
            Some(ActionSegmentDefinition::SelectTarget { .. })
            | Some(ActionSegmentDefinition::SelectOption { .. }) => {
                let parent = frame.parent;
                txn.set_action_frame(Some(frame));
                let decision = txn.allocate_decision();
                let offered = legal::action_frame(decision, catalog, txn.state)
                    .ok_or_else(|| action_fault(83))?;
                turn::offer_decision(txn, root, Some(parent), offered);
                return Ok(());
            }
            None => return finish_segmented_action(catalog, txn, root, frame),
        }
    }
}

fn execute_segment(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    frame: &ActionFrameState,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Result<EventId, BattleFault> {
    let plan = lower_action_segment(
        catalog,
        txn,
        frame.action,
        frame.actor,
        frame.owner,
        ability,
        targets,
    )
    .ok_or_else(|| action_fault(84))?;
    Ok(execute_action_segment(catalog, txn, root, parent, plan)?.parent)
}

fn finish_segmented_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    frame: ActionFrameState,
) -> Result<(), BattleFault> {
    debug_assert!(frame.paid);
    txn.set_action_frame(None);
    let completed = txn.emit(
        Cause::root(root).with_parent(frame.parent),
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::ActionFrameCompleted {
            frame: frame.id,
            action: frame.action,
        }),
    );
    let mut targets = frame.retained_targets;
    let mut all_targets = targets.targets.into_vec();
    for input in &frame.inputs {
        if let ActionFrameInput::Target(target) = input
            && !all_targets.contains(target)
        {
            all_targets.push(*target);
        }
    }
    targets.targets = all_targets.into_boxed_slice();
    let header = restore_segmented_ultimate_header(
        catalog,
        frame.action,
        frame.actor,
        frame.owner,
        frame.ability,
        targets,
    )
    .ok_or_else(|| action_fault(85))?;
    let finished = finish_action_envelope(catalog, txn, root, completed, header)?;
    let boundary_cause = action_cause(root, &finished.plan)?;
    let parent = parent_operation::settle_effects_at_action_end(
        catalog,
        txn,
        boundary_cause,
        finished.parent,
    )?;
    let parent = drain_reactions(catalog, txn, ReactionBoundary::AfterAction, parent)?;
    if let ActionBoundary::Continue(parent) =
        settle_after_action(catalog, txn, boundary_cause, parent)?
    {
        turn::enter_action_boundary(
            catalog,
            txn,
            root,
            parent,
            frame.boundary.turn,
            frame.boundary.continuation,
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
    txn.set_action_frame(None);
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
