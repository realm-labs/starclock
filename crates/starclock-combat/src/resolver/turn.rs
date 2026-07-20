use crate::{
    ActionGauge, BattlePhase,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    command::{legal, model::DecisionPoint},
    event::{
        cause::Cause,
        model::{BattleEventKind, DecisionEventData, TurnEventData},
    },
    id::{CommandId, EventId},
    timeline::{
        queue::InterruptQueue,
        select::plan_next_turn,
        state::{InterruptWindowKind, InterruptWindowState},
    },
};

use super::{
    action::{drain_reactions, execute_action_plan},
    settle::{ActionBoundary, settle_after_action},
    transaction::{Transaction, action_cause, action_fault, commit_targets},
};

pub(super) fn begin_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
) -> Result<(), BattleFault> {
    let advance = plan_next_turn(&txn.state.units, &txn.state.actors)?;
    for (actor, gauge) in advance.gauges {
        txn.set_actor_gauge(actor, gauge)?;
    }
    let turn = advance.turn;
    txn.reset_rule_slots(
        crate::rule::model::SlotResetPoint::TurnStart,
        Some(turn.unit),
    );
    txn.set_active_turn(Some(turn));
    let mut parent = txn.emit(
        Cause::for_turn(root, turn.owner, turn.actor).with_parent(parent),
        BattleEventKind::Turn(TurnEventData::Started {
            actor: turn.actor,
            owner: turn.owner,
        }),
    );
    let turn_cause = Cause::for_turn(root, turn.owner, turn.actor);
    for (operation, element) in txn.tick_temporary_weaknesses(turn.unit)? {
        parent = txn.emit(
            turn_cause
                .with_parent(parent)
                .with_primary_target(Some(turn.unit)),
            BattleEventKind::Toughness(crate::ToughnessEventData::WeaknessRemoved {
                operation,
                target: turn.unit,
                element,
            }),
        );
    }
    let (mut parent, frozen_skip) =
        super::operation::settle_break_effects_at_turn_start(txn, turn_cause, parent, turn.unit)?;
    parent = super::operation::settle_effects_at_turn_start(txn, turn_cause, parent, turn.unit)?;
    match settle_after_action(catalog, txn, turn_cause, parent)? {
        ActionBoundary::Terminal(_) => return Ok(()),
        ActionBoundary::Continue(next) => parent = next,
    }
    let alive = txn
        .state
        .units
        .get(turn.unit)
        .map(|unit| unit.life == crate::LifeState::Alive)
        .ok_or_else(|| action_fault(58))?;
    if frozen_skip || !alive {
        txn.set_active_turn(None);
        txn.set_actor_gauge(
            turn.actor,
            ActionGauge::from_scaled(if frozen_skip {
                5_000_000_000
            } else {
                10_000_000_000
            })
            .map_err(|_| action_fault(59))?,
        )?;
        parent = txn.emit(
            turn_cause.with_parent(parent),
            BattleEventKind::Turn(TurnEventData::Ended {
                actor: turn.actor,
                owner: turn.owner,
            }),
        );
        return begin_turn(catalog, txn, root, parent);
    }
    let was_broken = txn
        .state
        .units
        .get(turn.unit)
        .map(|unit| unit.weakness_broken)
        .ok_or_else(|| action_fault(60))?;
    if was_broken {
        let changes = txn.recover_toughness(turn.unit)?;
        txn.set_weakness_broken(turn.unit, false)?;
        for (layer_key, before, after) in changes {
            parent = txn.emit(
                turn_cause
                    .with_parent(parent)
                    .with_primary_target(Some(turn.unit)),
                BattleEventKind::Toughness(crate::ToughnessEventData::Recovered {
                    target: turn.unit,
                    layer_key,
                    before,
                    after,
                    exited_global_broken: true,
                }),
            );
        }
    }
    if let Some((ability, origin)) = turn.automatic {
        return execute_automatic_turn(catalog, txn, root, parent, turn, ability, origin);
    }
    txn.set_interrupt(Some(InterruptWindowState {
        kind: InterruptWindowKind::PreAction,
        turn,
        pending: InterruptQueue::default(),
    }));
    let decision_id = txn.allocate_decision();
    let decision = legal::interrupt_window(
        decision_id,
        turn.side,
        &txn.state.units,
        &txn.state.formations,
        &txn.state.teams,
        &txn.state.effects,
        catalog,
    );
    offer_decision(txn, root, Some(parent), decision);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn execute_automatic_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: crate::timeline::state::NormalTurnState,
    ability: crate::AbilityId,
    origin: crate::ActionOrigin,
) -> Result<(), BattleFault> {
    let targets = commit_targets(catalog, txn, turn.unit, ability, None)?;
    let mut plan = crate::action::lower::lower_timeline_action(
        catalog,
        txn,
        crate::action::lower::TimelineActionContext {
            actor: turn.unit,
            owner: turn.owner,
            timeline_actor: turn.actor,
            origin,
        },
        ability,
        targets,
    )
    .ok_or_else(|| action_fault(98))?;
    let mut parent = execute_action_plan(catalog, txn, root, parent, &mut plan)?;
    let cause = action_cause(root, &plan)?;
    parent = super::operation::settle_effects_at_action_end(txn, cause, parent)?;
    parent = drain_reactions(
        catalog,
        txn,
        crate::catalog::action::ReactionBoundary::AfterAction,
        parent,
    )?;
    if txn
        .state
        .actors
        .get(turn.actor)
        .is_some_and(|actor| actor.active)
    {
        txn.set_actor_gauge(
            turn.actor,
            ActionGauge::from_scaled(10_000_000_000).map_err(|_| action_fault(99))?,
        )?;
    }
    parent = txn.emit(
        Cause::for_turn(root, turn.owner, turn.actor).with_parent(parent),
        BattleEventKind::Turn(TurnEventData::Ended {
            actor: turn.actor,
            owner: turn.owner,
        }),
    );
    parent = super::operation::settle_effects_at_turn_end(txn, cause, parent, turn.unit)?;
    txn.set_active_turn(None);
    if let ActionBoundary::Continue(parent) = settle_after_action(catalog, txn, cause, parent)? {
        let parent = drain_reactions(
            catalog,
            txn,
            crate::catalog::action::ReactionBoundary::BeforeTimeline,
            parent,
        )?;
        if let ActionBoundary::Continue(parent) = settle_after_action(catalog, txn, cause, parent)?
        {
            begin_turn(catalog, txn, root, parent)?;
        }
    }
    Ok(())
}

pub(super) fn offer_interrupt_decision(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
) -> Result<(), BattleFault> {
    let side = txn
        .state
        .timeline
        .interrupt
        .as_ref()
        .ok_or_else(|| action_fault(13))?
        .turn
        .side;
    let decision_id = txn.allocate_decision();
    let decision = legal::interrupt_window(
        decision_id,
        side,
        &txn.state.units,
        &txn.state.formations,
        &txn.state.teams,
        &txn.state.effects,
        catalog,
    );
    offer_decision(txn, root, Some(parent), decision);
    Ok(())
}

pub(super) fn offer_decision(
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: Option<EventId>,
    decision: DecisionPoint,
) {
    let fact = DecisionEventData::Offered {
        decision: decision.id(),
        kind: decision.kind(),
        owner: decision.owner(),
    };
    txn.set_decision(Some(decision));
    txn.set_phase(BattlePhase::AwaitingCommand);
    let cause = parent.map_or_else(
        || Cause::root(root),
        |event| Cause::root(root).with_parent(event),
    );
    txn.emit(cause, BattleEventKind::Decision(fact));
}
