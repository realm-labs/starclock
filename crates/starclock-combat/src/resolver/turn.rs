use crate::{
    AbilityId, ActionGauge, ActionOrigin, BattlePhase, EffectRuntimeDefinition,
    EffectRuntimeTemplate, ForcedNormalAction, LifeState, ToughnessEventData, UnitId,
    action::{
        lower::{TimelineActionContext, lower_forced_basic_action, lower_timeline_action},
        model::ActionPlan,
    },
    battle::fault::BattleFault,
    catalog::{
        CombatCatalog,
        action::{
            AbilityKind, ReactionBoundary, TargetPattern, TargetRelation, UnitTargetSelector,
        },
    },
    command::{legal, model::DecisionPoint},
    event::{
        cause::Cause,
        model::{
            ActionBoundaryEventData, BattleEventData, BattleEventKind, DecisionEventData,
            TurnEventData,
        },
    },
    id::{CommandId, EventId},
    rng::types::DrawPurpose,
    rule::model::SlotResetPoint,
    target::select::{commit, legal_primary_targets, stable_pool},
    timeline::{
        select::plan_next_turn,
        state::{ActionBoundaryState, NormalTurnState, ResolutionContinuation},
    },
};

use super::{
    action::{drain_reactions, execute_action_plan},
    command_resolution::{action_cause, commit_targets},
    settle::{ActionBoundary, settle_after_action},
    transaction::{Transaction, action_fault},
};
use super::{operation, operation_formula, rule};

pub(super) fn start_battle(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
) -> Result<(), BattleFault> {
    txn.reset_rule_slots(SlotResetPoint::BattleStart, None);
    let mut started = txn.emit(
        Cause::root(root),
        BattleEventKind::Battle(BattleEventData::Started),
    );
    started = rule::dispatch_pending_after_events(catalog, txn, started)?;
    started = drain_reactions(catalog, txn, ReactionBoundary::BeforeTimeline, started)?;
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
    mut parent: EventId,
) -> Result<(), BattleFault> {
    loop {
        while let Some(pending) = txn.pop_extra_turn() {
            let Some(unit) = txn.state.units.get(pending.unit) else {
                continue;
            };
            if unit.life != LifeState::Alive || !unit.presence.is_timeline_eligible() {
                continue;
            }
            let Some(actor) = txn.state.actors.any_id_for_unit(pending.unit) else {
                continue;
            };
            let turn = NormalTurnState {
                actor,
                owner: pending.unit,
                unit: pending.unit,
                automatic: None,
                side: unit.side,
                formation: unit.formation,
                spawn: unit.spawn,
                origin: ActionOrigin::ExtraTurn,
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
            return enter_action_boundary(
                catalog,
                txn,
                root,
                started,
                turn,
                ResolutionContinuation::ContinueActiveTurn,
            );
        }
        match begin_turn(catalog, txn, root, parent)? {
            TurnStartOutcome::Boundary => return Ok(()),
            TurnStartOutcome::Continue(next) => parent = next,
        }
    }
}

enum TurnStartOutcome {
    Boundary,
    Continue(EventId),
}

fn begin_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
) -> Result<TurnStartOutcome, BattleFault> {
    let advance = plan_next_turn(&txn.state.units, &txn.state.actors)?;
    txn.add_timeline_elapsed(advance.elapsed_action_value_scaled)?;
    for (actor, gauge) in advance.gauges {
        txn.set_actor_gauge(actor, gauge)?;
    }
    let turn = advance.turn;
    txn.reset_rule_slots(SlotResetPoint::TurnStart, Some(turn.unit));
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
            BattleEventKind::Toughness(ToughnessEventData::WeaknessRemoved {
                operation,
                target: turn.unit,
                element,
            }),
        );
    }
    let controlled_skip = txn.state.effects.skips_normal_turn_at_start(turn.unit);
    let (mut parent, frozen_skip) =
        operation::settle_break_effects_at_turn_start(catalog, txn, turn_cause, parent, turn.unit)?;
    parent = operation::settle_effects_at_turn_start(catalog, txn, turn_cause, parent, turn.unit)?;
    match settle_after_action(catalog, txn, turn_cause, parent)? {
        ActionBoundary::Terminal(_) => return Ok(TurnStartOutcome::Boundary),
        ActionBoundary::Continue(next) => parent = next,
    }
    let alive = txn
        .state
        .units
        .get(turn.unit)
        .map(|unit| unit.life == LifeState::Alive)
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
        parent =
            operation::settle_effects_at_turn_end(catalog, txn, turn_cause, parent, turn.unit)?;
        txn.reset_rule_slots(SlotResetPoint::TurnEnd, Some(turn.unit));
        parent = rule::dispatch_pending_after_events(catalog, txn, parent)?;
        txn.set_active_turn(None);
        return Ok(TurnStartOutcome::Continue(parent));
    }
    let was_broken = txn
        .state
        .units
        .get(turn.unit)
        .map(|unit| unit.weakness_broken)
        .ok_or_else(|| action_fault(60))?;
    if was_broken {
        let recovery = operation_formula::FormulaInputs::new(txn)?
            .toughness_recovery(catalog, txn, turn.unit)?;
        let changes = txn.recover_toughness(turn.unit, recovery)?;
        txn.set_weakness_broken(turn.unit, false)?;
        for (layer_key, before, after) in changes {
            parent = txn.emit(
                turn_cause
                    .with_parent(parent)
                    .with_primary_target(Some(turn.unit)),
                BattleEventKind::Toughness(ToughnessEventData::Recovered {
                    target: turn.unit,
                    layer_key,
                    before,
                    after,
                    exited_global_broken: true,
                }),
            );
        }
    }
    enter_action_boundary(
        catalog,
        txn,
        root,
        parent,
        turn,
        ResolutionContinuation::ContinueActiveTurn,
    )?;
    Ok(TurnStartOutcome::Boundary)
}

fn forced_normal_action(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    unit: UnitId,
) -> Option<(ForcedNormalAction, UnitId)> {
    txn.state
        .effects
        .iter_by_id()
        .filter(|effect| effect.target == unit)
        .find_map(|effect| {
            let action = catalog
                .effect(effect.definition)?
                .runtime_template()
                .and_then(EffectRuntimeTemplate::forced_normal_action)
                .or_else(|| {
                    catalog
                        .effect(effect.definition)
                        .expect("effect definition was resolved above")
                        .runtime()
                        .and_then(EffectRuntimeDefinition::forced_normal_action)
                })?;
            let valid_applier = action != ForcedNormalAction::BasicAttackApplier
                || txn.state.units.get(effect.applier).is_some_and(|applier| {
                    applier.life == LifeState::Alive
                        && txn
                            .state
                            .units
                            .get(unit)
                            .is_some_and(|target| target.side != applier.side)
                });
            valid_applier.then_some((action, effect.applier))
        })
}

pub(super) fn enter_action_boundary(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: NormalTurnState,
    continuation: ResolutionContinuation,
) -> Result<(), BattleFault> {
    let id = txn.allocate_action_boundary();
    txn.set_action_boundary(Some(ActionBoundaryState {
        id,
        turn,
        continuation,
    }));
    let parent = txn.emit(
        Cause::root(root).with_parent(parent),
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::Opened { boundary: id }),
    );
    if continuation == ResolutionContinuation::ContinueActiveTurn
        && turn.automatic.is_none()
        && forced_normal_action(catalog, txn, turn.unit).is_none()
    {
        offer_normal_decision(catalog, txn, root, parent, turn)?;
    } else {
        txn.set_decision(None);
        txn.set_phase(BattlePhase::ReadyToAdvance);
    }
    Ok(())
}

fn resume_continuation(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: NormalTurnState,
    continuation: ResolutionContinuation,
) -> Result<(), BattleFault> {
    let active = txn
        .state
        .timeline
        .active_turn
        .ok_or_else(|| action_fault(1))?;
    if active != turn {
        return Err(action_fault(108));
    }
    match continuation {
        ResolutionContinuation::ContinueActiveTurn => {
            continue_active_turn(catalog, txn, root, parent, turn)
        }
        ResolutionContinuation::CompleteActiveTurn {
            cause,
            ticks_turn_end,
        } => finish_active_turn(
            catalog,
            txn,
            root,
            parent,
            turn,
            cause.with_root_command(root),
            ticks_turn_end,
        ),
    }
}

fn continue_active_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: NormalTurnState,
) -> Result<(), BattleFault> {
    if let Some((ability, origin)) = turn.automatic {
        return execute_automatic_turn(catalog, txn, root, parent, turn, ability, origin);
    }
    if let Some((action, applier)) = forced_normal_action(catalog, txn, turn.unit) {
        return execute_forced_basic_turn(catalog, txn, root, parent, turn, action, applier);
    }
    offer_normal_decision(catalog, txn, root, parent, turn)
}

fn offer_normal_decision(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: NormalTurnState,
) -> Result<(), BattleFault> {
    let unit = txn
        .state
        .units
        .get(turn.owner)
        .ok_or_else(|| action_fault(2))?;
    let abilities = unit.abilities.clone();
    let decision_id = txn.allocate_decision();
    let decision = legal::normal_action(
        decision_id,
        turn.side,
        turn.owner,
        &abilities,
        catalog,
        txn.state,
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
    turn: NormalTurnState,
    ability: AbilityId,
    origin: ActionOrigin,
) -> Result<(), BattleFault> {
    let selector = catalog
        .ability(ability)
        .and_then(|definition| catalog.selector(definition.selector()))
        .and_then(|definition| definition.unit_targets())
        .ok_or_else(|| action_fault(97))?;
    let primary =
        legal_primary_targets(&txn.state.units, &txn.state.formations, turn.unit, selector)
            .map_err(|_| action_fault(97))?
            .into_iter()
            .next()
            .flatten();
    let targets = commit_targets(catalog, txn, turn.unit, ability, primary)?;
    let mut plan = lower_timeline_action(
        catalog,
        txn,
        TimelineActionContext {
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
    turn: NormalTurnState,
    forced_action: ForcedNormalAction,
    applier: UnitId,
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
                .is_some_and(|action| action.kind() == AbilityKind::Basic)
        })
        .ok_or_else(|| action_fault(101))?;
    let definition = catalog.ability(ability).ok_or_else(|| action_fault(102))?;
    let action = definition.action().ok_or_else(|| action_fault(102))?;
    let authored = catalog
        .selector(definition.selector())
        .and_then(|selector| selector.unit_targets())
        .ok_or_else(|| action_fault(103))?;
    let relation = match forced_action {
        ForcedNormalAction::BasicAttackRandomAlly => TargetRelation::Allied,
        ForcedNormalAction::BasicAttackApplier => TargetRelation::Opposing,
    };
    let mut selector =
        UnitTargetSelector::new(relation, authored.pattern()).ok_or_else(|| action_fault(104))?;
    if authored.repeated_targets() {
        selector = selector.with_repeated_targets();
    }
    let primary = match (forced_action, selector.pattern()) {
        (_, TargetPattern::All) => None,
        (ForcedNormalAction::BasicAttackApplier, _) => Some(applier),
        (ForcedNormalAction::BasicAttackRandomAlly, _) => {
            let mut pool = stable_pool(
                &txn.state.units,
                &txn.state.formations,
                turn.side,
                TargetRelation::Allied,
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
                .choose_index(DrawPurpose::FORCED_ACTION_TARGET, candidates.len())?
                .ok_or_else(|| action_fault(105))?;
            Some(candidates[index])
        }
    };
    let targets = commit(
        &txn.state.units,
        &txn.state.formations,
        turn.unit,
        selector,
        action.invalidation(),
        primary,
    )
    .map_err(|_| action_fault(106))?;
    let mut plan = lower_forced_basic_action(
        catalog,
        txn,
        TimelineActionContext {
            actor: turn.unit,
            owner: turn.owner,
            timeline_actor: turn.actor,
            origin: ActionOrigin::Forced,
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
    turn: NormalTurnState,
    plan: &mut ActionPlan,
) -> Result<(), BattleFault> {
    let mut parent = execute_action_plan(catalog, txn, root, parent, plan)?;
    let cause = action_cause(root, plan)?;
    parent = operation::settle_effects_at_action_end(catalog, txn, cause, parent)?;
    parent = drain_reactions(catalog, txn, ReactionBoundary::AfterAction, parent)?;
    let resets_gauge = txn
        .state
        .actors
        .get(turn.actor)
        .is_some_and(|actor| actor.active);
    pause_completed_turn(
        catalog,
        txn,
        root,
        parent,
        TurnCompletion::planned(turn, cause, resets_gauge),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TurnCompletion {
    turn: NormalTurnState,
    cause: Cause,
    resets_gauge: bool,
    ticks_turn_end: bool,
}

impl TurnCompletion {
    pub(super) const fn selected(turn: NormalTurnState, cause: Cause) -> Self {
        let completes_normal_turn = matches!(turn.origin, ActionOrigin::NormalTurn);
        Self {
            turn,
            cause,
            resets_gauge: completes_normal_turn,
            ticks_turn_end: completes_normal_turn,
        }
    }

    const fn planned(turn: NormalTurnState, cause: Cause, resets_gauge: bool) -> Self {
        Self {
            turn,
            cause,
            resets_gauge,
            ticks_turn_end: true,
        }
    }
}

pub(super) fn pause_completed_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    completion: TurnCompletion,
) -> Result<(), BattleFault> {
    if completion.resets_gauge {
        txn.set_actor_gauge(
            completion.turn.actor,
            ActionGauge::from_scaled(10_000_000_000).map_err(|_| action_fault(99))?,
        )?;
    }
    let parent = match settle_after_action(catalog, txn, completion.cause, parent)? {
        ActionBoundary::Terminal(_) => return Ok(()),
        ActionBoundary::Continue(parent) => parent,
    };
    enter_action_boundary(
        catalog,
        txn,
        root,
        parent,
        completion.turn,
        ResolutionContinuation::CompleteActiveTurn {
            cause: completion.cause,
            ticks_turn_end: completion.ticks_turn_end,
        },
    )
}

fn finish_active_turn(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    turn: NormalTurnState,
    cause: Cause,
    ticks_turn_end: bool,
) -> Result<(), BattleFault> {
    let mut parent = txn.emit(
        Cause::for_turn(root, turn.owner, turn.actor).with_parent(parent),
        BattleEventKind::Turn(TurnEventData::Ended {
            actor: turn.actor,
            owner: turn.owner,
            origin: turn.origin,
        }),
    );
    if ticks_turn_end {
        parent = operation::settle_effects_at_turn_end(catalog, txn, cause, parent, turn.unit)?;
        txn.reset_rule_slots(SlotResetPoint::TurnEnd, Some(turn.unit));
    }
    parent = rule::dispatch_pending_after_events(catalog, txn, parent)?;
    txn.set_active_turn(None);
    if let ActionBoundary::Continue(parent) = settle_after_action(catalog, txn, cause, parent)? {
        let parent = drain_reactions(catalog, txn, ReactionBoundary::BeforeTimeline, parent)?;
        if let ActionBoundary::Continue(parent) = settle_after_action(catalog, txn, cause, parent)?
        {
            begin_next_turn(catalog, txn, root, parent)?;
        }
    }
    Ok(())
}

pub(super) fn advance_action_boundary(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    boundary: ActionBoundaryState,
) -> Result<(), BattleFault> {
    let parent = txn.emit(
        Cause::root(root),
        BattleEventKind::ActionBoundary(ActionBoundaryEventData::Advanced {
            boundary: boundary.id,
        }),
    );
    txn.set_action_boundary(None);
    if txn.state.decision.is_some() {
        txn.set_phase(BattlePhase::AwaitingCommand);
        return Ok(());
    }
    resume_continuation(
        catalog,
        txn,
        root,
        parent,
        boundary.turn,
        boundary.continuation,
    )
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
