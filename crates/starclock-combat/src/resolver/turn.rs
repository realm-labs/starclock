use crate::{
    ActionGauge, BattlePhase,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    command::{legal, model::DecisionPoint},
    event::{
        cause::Cause,
        model::{BattleEventData, BattleEventKind, DecisionEventData, TurnEventData},
    },
    id::{CommandId, EventId},
    timeline::{
        select::plan_next_turn,
        state::{InterruptWindowKind, InterruptWindowState},
    },
};

use super::{
    action::{drain_reactions, execute_action_plan},
    settle::{ActionBoundary, settle_after_action},
    transaction::{Transaction, action_cause, action_fault, commit_targets},
};

pub(super) fn start_battle(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
) -> Result<(), BattleFault> {
    txn.reset_rule_slots(crate::rule::model::SlotResetPoint::BattleStart, None);
    let mut started = txn.emit(
        Cause::root(root),
        BattleEventKind::Battle(BattleEventData::Started),
    );
    started = super::rule::dispatch_pending_after_events(catalog, txn, started)?;
    started = drain_reactions(
        catalog,
        txn,
        crate::catalog::action::ReactionBoundary::BeforeTimeline,
        started,
    )?;
    if let ActionBoundary::Continue(started) =
        settle_after_action(catalog, txn, Cause::root(root), started)?
    {
        begin_next_turn(catalog, txn, root, started)?;
    }
    Ok(())
}

pub(super) fn begin_next_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
) -> Result<(), BattleFault> {
    while let Some(pending) = txn.pop_extra_turn() {
        let Some(unit) = txn.state.units.get(pending.unit) else {
            continue;
        };
        if unit.life != crate::LifeState::Alive || !unit.presence.is_timeline_eligible() {
            continue;
        }
        let Some(actor) = txn.state.actors.any_id_for_unit(pending.unit) else {
            continue;
        };
        let turn = crate::timeline::state::NormalTurnState {
            actor,
            owner: pending.unit,
            unit: pending.unit,
            automatic: None,
            side: unit.side,
            formation: unit.formation,
            spawn: unit.spawn,
            origin: crate::ActionOrigin::ExtraTurn,
        };
        txn.set_active_turn(Some(turn));
        let started = txn.emit(
            Cause::for_turn(root, turn.owner, turn.actor).with_parent(parent),
            BattleEventKind::Turn(TurnEventData::Started {
                actor: turn.actor,
                owner: turn.owner,
                origin: turn.origin,
            }),
        );
        return offer_turn_decision(catalog, txn, root, started, turn);
    }
    begin_turn(catalog, txn, root, parent)
}

pub(super) fn begin_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
) -> Result<(), BattleFault> {
    let advance = plan_next_turn(&txn.state.units, &txn.state.actors)?;
    txn.add_timeline_elapsed(advance.elapsed_action_value_scaled)?;
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
            origin: turn.origin,
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
    let controlled_skip = txn.state.effects.skips_normal_turn_at_start(turn.unit);
    let forced_normal_action = forced_normal_action(catalog, txn, turn.unit);
    let (mut parent, frozen_skip) = super::operation::settle_break_effects_at_turn_start(
        catalog, txn, turn_cause, parent, turn.unit,
    )?;
    parent = super::operation::settle_effects_at_turn_start(
        catalog, txn, turn_cause, parent, turn.unit,
    )?;
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
    if frozen_skip || controlled_skip || !alive {
        txn.set_actor_gauge(
            turn.actor,
            ActionGauge::from_scaled(if frozen_skip || controlled_skip {
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
                origin: turn.origin,
            }),
        );
        parent = super::operation::settle_effects_at_turn_end(
            catalog, txn, turn_cause, parent, turn.unit,
        )?;
        txn.reset_rule_slots(crate::rule::model::SlotResetPoint::TurnEnd, Some(turn.unit));
        parent = super::rule::dispatch_pending_after_events(catalog, txn, parent)?;
        txn.set_active_turn(None);
        return begin_next_turn(catalog, txn, root, parent);
    }
    let was_broken = txn
        .state
        .units
        .get(turn.unit)
        .map(|unit| unit.weakness_broken)
        .ok_or_else(|| action_fault(60))?;
    if was_broken {
        let recovery = super::operation_formula::FormulaInputs::new(txn)?
            .toughness_recovery(catalog, txn, turn.unit)?;
        let changes = txn.recover_toughness(turn.unit, recovery)?;
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
    if forced_normal_action == Some(crate::ForcedNormalAction::BasicAttackRandomAlly) {
        return execute_forced_basic_turn(catalog, txn, root, parent, turn);
    }
    offer_turn_decision(catalog, txn, root, parent, turn)
}

fn forced_normal_action(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    unit: crate::UnitId,
) -> Option<crate::ForcedNormalAction> {
    txn.state
        .effects
        .iter_by_id()
        .filter(|effect| effect.target == unit)
        .filter_map(|effect| catalog.effect(effect.definition))
        .find_map(|definition| {
            definition
                .runtime_template()
                .and_then(crate::EffectRuntimeTemplate::forced_normal_action)
                .or_else(|| {
                    definition
                        .runtime()
                        .and_then(crate::EffectRuntimeDefinition::forced_normal_action)
                })
        })
}

fn offer_turn_decision(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: crate::timeline::state::NormalTurnState,
) -> Result<(), BattleFault> {
    txn.set_interrupt(Some(InterruptWindowState {
        kind: InterruptWindowKind::PreAction,
        turn,
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
    execute_planned_turn(catalog, txn, root, parent, turn, &mut plan)
}

fn execute_forced_basic_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: crate::timeline::state::NormalTurnState,
) -> Result<(), BattleFault> {
    let abilities = txn
        .state
        .units
        .get(turn.unit)
        .map(|unit| {
            legal::effective_abilities(&unit.abilities, &txn.state.effects, catalog, turn.unit)
        })
        .ok_or_else(|| action_fault(100))?;
    let ability = abilities
        .into_iter()
        .find(|ability| {
            catalog
                .ability(*ability)
                .and_then(|definition| definition.action())
                .is_some_and(|action| action.kind() == crate::catalog::action::AbilityKind::Basic)
        })
        .ok_or_else(|| action_fault(101))?;
    let definition = catalog.ability(ability).ok_or_else(|| action_fault(102))?;
    let action = definition.action().ok_or_else(|| action_fault(102))?;
    let authored = catalog
        .selector(definition.selector())
        .and_then(|selector| selector.unit_targets())
        .ok_or_else(|| action_fault(103))?;
    let mut selector = crate::catalog::action::UnitTargetSelector::new(
        crate::catalog::action::TargetRelation::Allied,
        authored.pattern(),
    )
    .ok_or_else(|| action_fault(104))?;
    if authored.repeated_targets() {
        selector = selector.with_repeated_targets();
    }
    let mut pool = crate::target::select::stable_pool(
        &txn.state.units,
        &txn.state.formations,
        turn.side,
        crate::catalog::action::TargetRelation::Allied,
    );
    let mut candidates = pool
        .iter()
        .copied()
        .filter(|target| *target != turn.unit)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        candidates.append(&mut pool);
    }
    let index = txn
        .choose_index(
            crate::rng::types::DrawPurpose::FORCED_ACTION_TARGET,
            candidates.len(),
        )?
        .ok_or_else(|| action_fault(105))?;
    let primary = match selector.pattern() {
        crate::catalog::action::TargetPattern::All => None,
        crate::catalog::action::TargetPattern::Single
        | crate::catalog::action::TargetPattern::Blast => Some(candidates[index]),
    };
    let targets = crate::target::select::commit(
        &txn.state.units,
        &txn.state.formations,
        turn.unit,
        selector,
        action.invalidation(),
        primary,
    )
    .map_err(|_| action_fault(106))?;
    let mut plan = crate::action::lower::lower_forced_basic_action(
        catalog,
        txn,
        crate::action::lower::TimelineActionContext {
            actor: turn.unit,
            owner: turn.owner,
            timeline_actor: turn.actor,
            origin: crate::ActionOrigin::Forced,
        },
        ability,
        targets,
    )
    .ok_or_else(|| action_fault(107))?;
    execute_planned_turn(catalog, txn, root, parent, turn, &mut plan)
}

fn execute_planned_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: crate::timeline::state::NormalTurnState,
    plan: &mut crate::action::model::ActionPlan,
) -> Result<(), BattleFault> {
    let mut parent = execute_action_plan(catalog, txn, root, parent, plan)?;
    let cause = action_cause(root, plan)?;
    parent = super::operation::settle_effects_at_action_end(catalog, txn, cause, parent)?;
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
            origin: turn.origin,
        }),
    );
    parent = super::operation::settle_effects_at_turn_end(catalog, txn, cause, parent, turn.unit)?;
    txn.reset_rule_slots(crate::rule::model::SlotResetPoint::TurnEnd, Some(turn.unit));
    parent = super::rule::dispatch_pending_after_events(catalog, txn, parent)?;
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
            begin_next_turn(catalog, txn, root, parent)?;
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
