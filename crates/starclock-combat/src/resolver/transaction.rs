use crate::{
    action::lower::{ActionIdentityAllocator, lower_interrupt_action, lower_normal_action},
    battle::{
        fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
        model::BattlePhase,
        state::BattleState,
    },
    catalog::CombatCatalog,
    codec::{BattleStateHash, hash_state},
    command::{legal, model::DecisionPoint, validate::ValidatedCommand},
    event::{
        cause::{Cause, CauseActor},
        model::{
            BattleEvent, BattleEventData, BattleEventKind, DecisionEventData, FaultEventData,
            TurnEventData,
        },
    },
    id::{
        ActionId, CommandId, DecisionId, EffectInstanceId, EventId, HitId, OperationId, PhaseId,
        ShieldInstanceId, SourceDefinitionId, TimelineActorId, WaveInstanceId,
    },
    numeric::domain::ActionGauge,
    rng::types::DrawPurpose,
    target::select,
    timeline::state::{InterruptWindowState, NormalTurnState},
};

mod scratch;

pub(crate) use scratch::ResolutionScratch;

use super::{
    action::{drain_reactions, execute_action_plan},
    journal::{AllocationKind, MutationField, MutationJournal, phase_code},
    settle::{ActionBoundary, settle_after_action},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FaultInjectionPoint {
    AfterResolvingPhase,
    AfterCommandMutation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FaultInjection {
    pub(crate) point: FaultInjectionPoint,
    pub(crate) policy: FaultPolicy,
}

pub(crate) struct TransactionOutput {
    pub(crate) events: Vec<BattleEvent>,
    pub(crate) state_hash: BattleStateHash,
    pub(crate) root_command: CommandId,
    pub(crate) fault: Option<BattleFault>,
}

pub(crate) fn resolve_prepared(
    catalog: &CombatCatalog,
    before: &BattleState,
    scratch: &mut ResolutionScratch,
    command: ValidatedCommand,
    injection: Option<FaultInjection>,
) -> TransactionOutput {
    let (mut events, root_command, failure) = {
        let mut txn = Transaction::new(&mut scratch.working, &mut scratch.journal);
        let root = txn.begin_command();
        let failure = execute(catalog, &mut txn, root, command, injection)
            .and_then(|()| {
                let parent = txn.events.last().map(BattleEvent::id).ok_or_else(|| {
                    BattleFault::new(
                        FaultKind::InvariantViolation,
                        FaultBoundary::Command,
                        FaultPolicy::Rollback,
                        0x33ff,
                        None,
                    )
                })?;
                super::rule::dispatch_pending_after_events(catalog, &mut txn, parent).map(drop)
            })
            .err();
        (txn.events, root, failure)
    };

    let mut committed_fault = failure;
    if committed_fault.is_none() && !scratch.journal.verify(&event_ids(&events)) {
        committed_fault = Some(BattleFault::new(
            FaultKind::InvariantViolation,
            FaultBoundary::Commit,
            FaultPolicy::Rollback,
            1,
            None,
        ));
    }

    if let Some(fault) = committed_fault {
        if fault.policy() == FaultPolicy::Rollback {
            scratch.prepare(before);
            let mut txn = Transaction::new(&mut scratch.working, &mut scratch.journal);
            let rollback_root = txn.begin_command();
            debug_assert_eq!(rollback_root, root_command);
            events = txn.commit_fault(rollback_root, fault);
        } else {
            let txn = Transaction::with_events(&mut scratch.working, &mut scratch.journal, events);
            events = txn.commit_fault(root_command, fault);
        }
    }

    let event_ids = event_ids(&events);
    if !scratch.journal.verify(&event_ids) {
        // A fault settlement that cannot journal itself is a programmer defect,
        // not a recoverable battle condition.
        panic!("fault settlement produced an inconsistent forward journal");
    }
    let state_hash = hash_state(&scratch.working);
    TransactionOutput {
        events,
        state_hash,
        root_command,
        fault: committed_fault,
    }
}

fn execute(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    command: ValidatedCommand,
    injection: Option<FaultInjection>,
) -> Result<(), BattleFault> {
    txn.set_phase(BattlePhase::Resolving);
    maybe_inject(injection, FaultInjectionPoint::AfterResolvingPhase)?;

    match command {
        ValidatedCommand::StartBattle => {
            let started = txn.emit(
                Cause::root(root),
                BattleEventKind::Battle(BattleEventData::Started),
            );
            super::turn::begin_turn(catalog, txn, root, started)?;
        }
        ValidatedCommand::PassInterruptWindow => {
            let closed = close_active_decision(txn, root)?;
            let turn = txn
                .state
                .timeline
                .active_turn
                .ok_or_else(|| action_fault(1))?;
            txn.set_interrupt(None);
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
            super::turn::offer_decision(txn, root, Some(closed), decision);
        }
        ValidatedCommand::UseAbility {
            actor,
            ability,
            primary_target,
        } => {
            let closed = close_active_decision(txn, root)?;
            let turn = txn
                .state
                .timeline
                .active_turn
                .ok_or_else(|| action_fault(3))?;
            if turn.owner != actor || txn.state.timeline.interrupt.is_some() {
                return Err(action_fault(4));
            }
            let targets = commit_targets(catalog, txn, actor, ability, primary_target)?;
            let owner = legal::ability_owner(txn.state, catalog, actor, ability)
                .ok_or_else(|| action_fault(5))?;
            let mut plan =
                lower_normal_action(catalog, txn, actor, owner, turn.actor, ability, targets)
                    .ok_or_else(|| action_fault(5))?;
            let action_resolved = execute_action_plan(catalog, txn, root, closed, &mut plan)?;
            let boundary_cause = action_cause(root, &plan)?;
            let action_resolved = super::operation::settle_effects_at_action_end(
                txn,
                boundary_cause,
                action_resolved,
            )?;
            let action_resolved = drain_reactions(
                catalog,
                txn,
                crate::catalog::action::ReactionBoundary::AfterAction,
                action_resolved,
            )?;
            txn.set_actor_gauge(
                turn.actor,
                ActionGauge::from_scaled(10_000_000_000).map_err(|_| action_fault(6))?,
            )?;
            let ended = txn.emit(
                Cause::for_turn(root, turn.owner, turn.actor).with_parent(action_resolved),
                BattleEventKind::Turn(TurnEventData::Ended {
                    actor: turn.actor,
                    owner: turn.owner,
                }),
            );
            let ended = super::operation::settle_effects_at_turn_end(
                txn,
                boundary_cause,
                ended,
                turn.owner,
            )?;
            txn.set_active_turn(None);
            if let ActionBoundary::Continue(parent) =
                settle_after_action(catalog, txn, boundary_cause, ended)?
            {
                let parent = drain_reactions(
                    catalog,
                    txn,
                    crate::catalog::action::ReactionBoundary::BeforeTimeline,
                    parent,
                )?;
                if let ActionBoundary::Continue(parent) =
                    settle_after_action(catalog, txn, boundary_cause, parent)?
                {
                    super::turn::begin_turn(catalog, txn, root, parent)?;
                }
            }
        }
        ValidatedCommand::UseInterrupt {
            actor,
            ability,
            primary_target,
        } => {
            let closed = close_active_decision(txn, root)?;
            if txn.state.timeline.interrupt.is_none() {
                return Err(action_fault(11));
            }
            let targets = commit_targets(catalog, txn, actor, ability, primary_target)?;
            let owner = legal::ability_owner(txn.state, catalog, actor, ability)
                .ok_or_else(|| action_fault(12))?;
            let mut plan = lower_interrupt_action(catalog, txn, actor, owner, ability, targets)
                .ok_or_else(|| action_fault(12))?;
            let resolved = execute_action_plan(catalog, txn, root, closed, &mut plan)?;
            let boundary_cause = action_cause(root, &plan)?;
            let resolved =
                super::operation::settle_effects_at_action_end(txn, boundary_cause, resolved)?;
            let resolved = drain_reactions(
                catalog,
                txn,
                crate::catalog::action::ReactionBoundary::AfterAction,
                resolved,
            )?;
            if let ActionBoundary::Continue(parent) =
                settle_after_action(catalog, txn, boundary_cause, resolved)?
            {
                super::turn::offer_interrupt_decision(catalog, txn, root, parent)?;
            }
        }
        ValidatedCommand::Concede => {
            let closed = close_active_decision(txn, root)?;
            txn.set_decision(None);
            txn.set_phase(BattlePhase::Lost);
            txn.emit(
                Cause::root(root).with_parent(closed),
                BattleEventKind::Battle(BattleEventData::Conceded {
                    side: crate::battle::spec::TeamSide::Player,
                }),
            );
        }
    }
    if !txn.reactions.is_empty() {
        return Err(action_fault(73));
    }
    maybe_inject(injection, FaultInjectionPoint::AfterCommandMutation)?;
    txn.bump_revision()?;
    Ok(())
}

pub(super) fn action_cause(
    root: CommandId,
    plan: &crate::action::model::ActionPlan,
) -> Result<Cause, BattleFault> {
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
    actor: crate::UnitId,
    ability: crate::AbilityId,
    primary: Option<crate::UnitId>,
) -> Result<crate::target::model::TargetCommitment, BattleFault> {
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

pub(super) fn action_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3100 + context,
        None,
    )
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

pub(super) struct Transaction<'a> {
    pub(super) state: &'a mut BattleState,
    pub(super) journal: &'a mut MutationJournal,
    events: Vec<BattleEvent>,
    next_rule_event: usize,
    pub(super) reactions: crate::reaction::queue::ReactionQueue,
    resolved_reactions: usize,
    next_reaction: u64,
}

impl<'a> Transaction<'a> {
    fn new(state: &'a mut BattleState, journal: &'a mut MutationJournal) -> Self {
        Self {
            state,
            journal,
            events: Vec::new(),
            next_rule_event: 0,
            reactions: crate::reaction::queue::ReactionQueue::default(),
            resolved_reactions: 0,
            next_reaction: 1,
        }
    }

    fn with_events(
        state: &'a mut BattleState,
        journal: &'a mut MutationJournal,
        events: Vec<BattleEvent>,
    ) -> Self {
        Self {
            state,
            journal,
            events,
            next_rule_event: 0,
            reactions: crate::reaction::queue::ReactionQueue::default(),
            resolved_reactions: 0,
            next_reaction: 1,
        }
    }

    pub(super) fn next_pending_rule_event(&mut self) -> Option<BattleEvent> {
        let event = self.events.get(self.next_rule_event)?.clone();
        self.next_rule_event += 1;
        Some(event)
    }

    fn begin_command(&mut self) -> CommandId {
        let command = self
            .state
            .sequences
            .try_command()
            .expect("rules-revision command budget prevents u64 identity exhaustion");
        self.journal
            .allocation(AllocationKind::Command, command.get());
        command
    }

    pub(super) fn allocate_decision(&mut self) -> DecisionId {
        let decision = self
            .state
            .sequences
            .try_decision()
            .expect("rules-revision decision budget prevents u64 identity exhaustion");
        self.journal
            .allocation(AllocationKind::Decision, decision.get());
        decision
    }

    fn allocate_event(&mut self) -> EventId {
        let event = self
            .state
            .sequences
            .try_event()
            .expect("rules-revision event budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Event, event.get());
        event
    }

    fn allocate_action(&mut self) -> ActionId {
        let id = self
            .state
            .sequences
            .try_action()
            .expect("rules-revision action budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Action, id.get());
        id
    }

    fn allocate_phase(&mut self) -> PhaseId {
        let id = self
            .state
            .sequences
            .try_phase()
            .expect("rules-revision phase budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Phase, id.get());
        id
    }

    fn allocate_hit(&mut self) -> HitId {
        let id = self
            .state
            .sequences
            .try_hit()
            .expect("rules-revision hit budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Hit, id.get());
        id
    }

    pub(super) fn allocate_operation(&mut self) -> OperationId {
        let id = self
            .state
            .sequences
            .try_operation()
            .expect("rules-revision operation budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Operation, id.get());
        id
    }

    pub(super) fn allocate_reaction(&mut self) -> u64 {
        let insertion = self.next_reaction;
        self.next_reaction = insertion
            .checked_add(1)
            .expect("rules-revision reaction budget prevents sequence exhaustion");
        self.journal.allocation(AllocationKind::Reaction, insertion);
        self.journal
            .queue_insertion(super::journal::QueueKind::Reaction, insertion);
        insertion
    }

    pub(super) fn allocate_unit(&mut self) -> crate::UnitId {
        let id = self.state.sequences.unit();
        self.journal.allocation(AllocationKind::Unit, id.get());
        id
    }

    pub(super) fn allocate_actor(&mut self) -> TimelineActorId {
        let id = self.state.sequences.actor();
        self.journal.allocation(AllocationKind::Actor, id.get());
        id
    }

    pub(super) fn allocate_spawn(&mut self) -> crate::SpawnSequence {
        let id = self.state.sequences.spawn();
        self.journal.allocation(AllocationKind::Spawn, id.get());
        id
    }

    pub(super) fn allocate_rule(&mut self) -> crate::RuleInstanceId {
        let id = self.state.sequences.rule();
        self.journal.allocation(AllocationKind::Rule, id.get());
        id
    }

    pub(super) fn allocate_modifier(&mut self) -> crate::ModifierInstanceId {
        let id = self.state.sequences.modifier();
        self.journal.allocation(AllocationKind::Modifier, id.get());
        id
    }
    pub(super) fn consume_reaction_budget(&mut self, maximum: usize) -> bool {
        let Some(next) = self.resolved_reactions.checked_add(1) else {
            return false;
        };
        if next > maximum {
            return false;
        }
        self.resolved_reactions = next;
        true
    }

    pub(super) fn allocate_shield(&mut self) -> ShieldInstanceId {
        let id = self
            .state
            .sequences
            .try_shield()
            .expect("rules-revision shield budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Shield, id.get());
        id
    }

    pub(super) fn allocate_effect(&mut self) -> EffectInstanceId {
        let id = self
            .state
            .sequences
            .try_effect()
            .expect("rules-revision effect budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Effect, id.get());
        id
    }

    pub(super) fn allocate_wave(&mut self) -> WaveInstanceId {
        let id = self
            .state
            .sequences
            .try_wave()
            .expect("rules-revision wave budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Wave, id.get());
        id
    }

    pub(super) fn set_phase(&mut self, phase: BattlePhase) {
        let before = self.state.phase;
        if before != phase {
            self.state.phase = phase;
            self.journal
                .mutation(MutationField::Phase, phase_code(before), phase_code(phase));
        }
    }

    pub(super) fn set_decision(&mut self, decision: Option<DecisionPoint>) {
        let before = self
            .state
            .decision
            .as_ref()
            .map_or(0, |value| value.id().get());
        let after = decision.as_ref().map_or(0, |value| value.id().get());
        if self.state.decision != decision {
            self.state.decision = decision;
            self.journal
                .mutation(MutationField::Decision, before, after);
        }
    }

    fn set_fault(&mut self, fault: BattleFault) {
        let before = self.state.fault.map_or(0, fault_code);
        let after = fault_code(fault);
        if self.state.fault != Some(fault) {
            self.state.fault = Some(fault);
            self.journal.mutation(MutationField::Fault, before, after);
        }
    }

    pub(super) fn set_active_turn(&mut self, turn: Option<NormalTurnState>) {
        let before = self
            .state
            .timeline
            .active_turn
            .map_or(0, |value| value.actor.get());
        let after = turn.map_or(0, |value| value.actor.get());
        if self.state.timeline.active_turn != turn {
            self.state.timeline.active_turn = turn;
            self.journal
                .mutation(MutationField::Timeline, before, after);
        }
    }

    pub(super) fn set_interrupt(&mut self, interrupt: Option<InterruptWindowState>) {
        let before = self
            .state
            .timeline
            .interrupt
            .as_ref()
            .map_or(0, |value| value.turn.actor.get());
        let after = interrupt.as_ref().map_or(0, |value| value.turn.actor.get());
        if self.state.timeline.interrupt != interrupt {
            self.state.timeline.interrupt = interrupt;
            self.journal
                .mutation(MutationField::Timeline, before, after);
        }
    }

    pub(super) fn set_actor_gauge(
        &mut self,
        actor: TimelineActorId,
        gauge: ActionGauge,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .actors
            .get_mut(actor)
            .ok_or_else(|| action_fault(8))?;
        let before = state.gauge.scaled();
        let after = gauge.scaled();
        if before != after {
            state.gauge = gauge;
            self.journal.mutation(
                MutationField::ActionGauge,
                before.cast_unsigned(),
                after.cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_actor_active(
        &mut self,
        actor: TimelineActorId,
        active: bool,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .actors
            .get_mut(actor)
            .ok_or_else(|| action_fault(74))?;
        if state.active != active {
            let before = u64::from(state.active);
            state.active = active;
            self.journal
                .mutation(MutationField::ActorActive, before, u64::from(active));
        }
        Ok(())
    }

    pub(super) fn insert_unit(&mut self, state: crate::actor::store::UnitState) {
        let id = state.id;
        self.state.units.insert(state);
        self.journal.mutation(MutationField::UnitStore, 0, id.get());
    }

    pub(super) fn insert_actor(&mut self, state: crate::actor::store::TimelineActorState) {
        let id = state.id;
        self.state.actors.insert(state);
        self.journal
            .mutation(MutationField::ActorStore, 0, id.get());
    }

    pub(super) fn insert_modifier(
        &mut self,
        state: crate::modifier::model::ActiveModifier,
    ) -> Result<(), BattleFault> {
        let id = state.instance;
        if !self.state.modifiers.insert(state) {
            return Err(action_fault(76));
        }
        self.journal
            .mutation(MutationField::ModifierStore, 0, id.get());
        Ok(())
    }

    pub(super) fn remove_effect_attachments(&mut self, effect: crate::EffectInstanceId) {
        for modifier in self.state.modifiers.remove_by_effect(effect) {
            self.journal
                .mutation(MutationField::EffectAttachment, modifier.get(), 0);
        }
        for rule in self.state.rules.remove_by_effect(effect) {
            self.journal
                .mutation(MutationField::EffectAttachment, rule.get(), 0);
        }
    }
    pub(super) fn insert_formation(&mut self, entry: crate::actor::store::FormationEntry) {
        self.state.formations.push(entry);
        self.journal
            .mutation(MutationField::Formation, 0, entry.unit.get());
    }

    pub(super) fn insert_link(
        &mut self,
        state: crate::actor::store::LinkState,
    ) -> Result<(), BattleFault> {
        let code = match state.entity {
            crate::LinkedEntity::Unit(unit) => unit.get(),
            crate::LinkedEntity::TimelineActor(actor) => actor.get() | (1_u64 << 63),
        };
        if !self.state.links.insert(state) {
            return Err(action_fault(75));
        }
        self.journal.mutation(MutationField::LinkStore, 0, code);
        Ok(())
    }

    pub(super) fn set_link_active(
        &mut self,
        entity: crate::LinkedEntity,
        active: bool,
    ) -> Result<(), BattleFault> {
        let link = self
            .state
            .links
            .get_mut(entity)
            .ok_or_else(|| action_fault(76))?;
        if link.active != active {
            let before = u64::from(link.active);
            link.active = active;
            self.journal
                .mutation(MutationField::LinkStore, before, u64::from(active));
        }
        Ok(())
    }

    pub(super) fn set_unit_definition(
        &mut self,
        unit: crate::UnitId,
        form: crate::UnitDefinitionId,
        abilities: Box<[crate::AbilityId]>,
        presence: crate::PresenceState,
        transformation: Option<crate::actor::store::TransformationState>,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(77))?;
        if state.form != form {
            let before = state.form.get();
            state.form = form;
            self.journal.mutation(
                MutationField::UnitDefinition,
                u64::from(before),
                u64::from(form.get()),
            );
        }
        if state.abilities != abilities {
            state.abilities = abilities;
            self.journal.mutation(MutationField::UnitAbilities, 1, 2);
        }
        let before_presence = state.presence;
        if before_presence != presence {
            state.presence = presence;
            self.journal.mutation(
                MutationField::UnitPresence,
                before_presence as u64,
                presence as u64,
            );
        }
        if state.transformation != transformation {
            let (before_transform, after_transform) =
                match (state.transformation.as_ref(), transformation.as_ref()) {
                    (None, Some(_)) => (0, 1),
                    (Some(_), None) => (1, 0),
                    (Some(_), Some(_)) => (1, 2),
                    (None, None) => unreachable!("unequal transformations cannot both be absent"),
                };
            state.transformation = transformation;
            self.journal.mutation(
                MutationField::Transformation,
                before_transform,
                after_transform,
            );
        }
        Ok(())
    }

    pub(super) fn set_enemy_runtime(
        &mut self,
        unit: crate::UnitId,
        enemy: crate::actor::store::EnemyRuntimeState,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(97))?;
        let before = state.enemy.ok_or_else(|| action_fault(98))?;
        if before != enemy {
            state.enemy = Some(enemy);
            let before_code = before.phase.map_or(0, |phase| u64::from(phase.get()));
            let after_code = enemy.phase.map_or(0, |phase| u64::from(phase.get()));
            self.journal
                .mutation(MutationField::EnemyOrchestration, before_code, after_code);
        }
        Ok(())
    }

    pub(super) fn delay_unit(
        &mut self,
        owner: crate::UnitId,
        scaled: i64,
    ) -> Result<(), BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(43))?;
        let before = self
            .state
            .actors
            .get(actor)
            .ok_or_else(|| action_fault(44))?
            .gauge;
        let after = ActionGauge::from_scaled(
            before
                .scaled()
                .checked_add(scaled)
                .ok_or_else(|| action_fault(45))?,
        )
        .map_err(|_| action_fault(46))?;
        self.set_actor_gauge(actor, after)
    }

    pub(super) fn unit_speed(&self, owner: crate::UnitId) -> Result<crate::Speed, BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(52))?;
        self.state
            .actors
            .get(actor)
            .map(|state| state.speed)
            .ok_or_else(|| action_fault(53))
    }

    pub(super) fn set_unit_speed(
        &mut self,
        owner: crate::UnitId,
        speed: crate::Speed,
    ) -> Result<(), BattleFault> {
        let actor = self
            .state
            .actors
            .id_for_owner(owner)
            .ok_or_else(|| action_fault(54))?;
        let state = self
            .state
            .actors
            .get_mut(actor)
            .ok_or_else(|| action_fault(55))?;
        let before = state.speed;
        if before != speed {
            state.speed = speed;
            self.journal.mutation(
                MutationField::Speed,
                before.scaled() as u64,
                speed.scaled() as u64,
            );
        }
        Ok(())
    }

    pub(super) fn roll_probability(
        &mut self,
        probability: crate::Probability,
        purpose: DrawPurpose,
    ) -> Result<bool, BattleFault> {
        let threshold = probability.millionths();
        if threshold == 0 {
            return Ok(false);
        }
        if threshold == 1_000_000 {
            return Ok(true);
        }
        let before = self.state.rng.draw_count();
        let draw = self
            .state
            .rng
            .sample_below(purpose, 1_000_000)
            .map_err(|_| action_fault(51))?;
        for index in before..self.state.rng.draw_count() {
            self.journal.rng_draw(index, purpose.code());
        }
        Ok(draw.value() < u64::from(threshold))
    }

    pub(super) fn set_skill_points(&mut self, side: crate::TeamSide, value: u16) {
        let state = self.state.teams.get_mut(side);
        let before = state.skill_points;
        if before != value {
            state.skill_points = value;
            self.journal.mutation(
                MutationField::TeamSkillPoints,
                u64::from(before),
                u64::from(value),
            );
        }
    }

    pub(super) fn set_team_resource(
        &mut self,
        side: crate::TeamSide,
        resource: crate::SourceDefinitionId,
        value: u16,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .teams
            .get_mut(side)
            .keyed_mut(resource)
            .ok_or_else(|| action_fault(53))?;
        let before = state.current;
        if value > state.maximum {
            return Err(action_fault(54));
        }
        if before != value {
            state.current = value;
            self.journal.mutation(
                MutationField::TeamKeyedResource,
                u64::from(before),
                u64::from(value),
            );
        }
        Ok(())
    }

    pub(super) fn set_character_resource(
        &mut self,
        unit: crate::UnitId,
        stable_key: &str,
        value: crate::Scalar,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .and_then(|unit| unit.resource_mut(stable_key))
            .ok_or_else(|| action_fault(55))?;
        let before = state.current;
        if value.scaled() < 0 || value > state.maximum {
            return Err(action_fault(56));
        }
        if before != value {
            state.current = value;
            self.journal.mutation(
                MutationField::UnitCharacterResource,
                before.scaled().cast_unsigned(),
                value.scaled().cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_energy(
        &mut self,
        unit: crate::UnitId,
        value: crate::Energy,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(31))?;
        let before = state.current_energy;
        if before != value {
            state.current_energy = value;
            self.journal.mutation(
                MutationField::UnitEnergy,
                before.scaled().cast_unsigned(),
                value.scaled().cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_hp(
        &mut self,
        unit: crate::UnitId,
        value: crate::Hp,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(33))?;
        let before = state.current_hp;
        if before != value {
            state.current_hp = value;
            self.journal.mutation(
                MutationField::UnitHp,
                before.get().cast_unsigned(),
                value.get().cast_unsigned(),
            );
        }
        Ok(())
    }

    pub(super) fn set_life(
        &mut self,
        unit: crate::UnitId,
        value: crate::LifeState,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(34))?;
        let before = state.life;
        if before != value {
            state.life = value;
            self.journal
                .mutation(MutationField::UnitLife, before as u64, value as u64);
        }
        Ok(())
    }

    pub(super) fn set_presence(
        &mut self,
        unit: crate::UnitId,
        value: crate::PresenceState,
    ) -> Result<(), BattleFault> {
        let state = self
            .state
            .units
            .get_mut(unit)
            .ok_or_else(|| action_fault(35))?;
        let before = state.presence;
        if before != value {
            state.presence = value;
            self.journal
                .mutation(MutationField::UnitPresence, before as u64, value as u64);
        }
        Ok(())
    }

    pub(super) fn set_encounter_wave(&mut self, wave: WaveInstanceId, number: u16) {
        let before = self.state.encounter.number;
        if self.state.encounter.wave != wave || before != number {
            self.state.encounter.wave = wave;
            self.state.encounter.number = number;
            self.journal.mutation(
                MutationField::Encounter,
                u64::from(before),
                u64::from(number),
            );
        }
    }

    fn bump_revision(&mut self) -> Result<(), BattleFault> {
        let before = self.state.committed_revision;
        let after = before.checked_add(1).ok_or_else(|| {
            BattleFault::new(
                FaultKind::SequenceExhausted,
                FaultBoundary::Commit,
                FaultPolicy::Rollback,
                2,
                None,
            )
        })?;
        self.state.committed_revision = after;
        self.journal
            .mutation(MutationField::CommittedRevision, before, after);
        Ok(())
    }

    pub(super) fn emit(&mut self, cause: Cause, kind: BattleEventKind) -> EventId {
        let id = self.allocate_event();
        self.events.push(BattleEvent::new(id, cause, kind));
        self.journal.event(id);
        id
    }

    pub(super) fn snapshot(&mut self, operation: OperationId) {
        self.journal.snapshot(operation.get());
    }

    pub(super) fn record_shield_change(
        &mut self,
        before: crate::ShieldAmount,
        after: crate::ShieldAmount,
    ) {
        if before != after {
            self.journal.mutation(
                MutationField::ShieldRemaining,
                u64::try_from(before.get()).expect("shield is non-negative"),
                u64::try_from(after.get()).expect("shield is non-negative"),
            );
        }
    }

    pub(super) fn record_effect_change(&mut self, before: u64, after: u64, identity: u64) {
        let encoded_before = before.checked_mul(2).expect("effect budget is bounded");
        let mut encoded_after = after.checked_mul(2).expect("effect budget is bounded");
        if encoded_before == encoded_after {
            encoded_after = encoded_after
                .checked_add(identity | 1)
                .expect("effect identity is bounded");
        }
        self.journal
            .mutation(MutationField::Effect, encoded_before, encoded_after);
    }

    pub(super) fn record_rule_state_change(
        &mut self,
        instance: crate::RuleInstanceId,
        slot: crate::StateSlotDefinitionId,
        before: &crate::rule::model::RuleValue,
        after: &crate::rule::model::RuleValue,
    ) {
        if before != after {
            let key = instance.get().rotate_left(17) ^ u64::from(slot.get());
            self.journal
                .mutation(MutationField::RuleState, key, key ^ 1);
        }
    }

    pub(super) fn reset_rule_slots(
        &mut self,
        boundary: crate::rule::model::SlotResetPoint,
        owner: Option<crate::UnitId>,
    ) {
        let count = self.state.rules.reset(boundary, owner);
        if count > 0 {
            self.journal
                .mutation(MutationField::RuleState, 0, count as u64);
        }
    }

    fn commit_fault(mut self, root: CommandId, fault: BattleFault) -> Vec<BattleEvent> {
        self.set_decision(None);
        self.set_interrupt(None);
        self.set_active_turn(None);
        self.set_fault(fault);
        self.set_phase(BattlePhase::Faulted);
        if let Err(revision_fault) = self.bump_revision() {
            debug_assert_eq!(fault.kind(), FaultKind::SequenceExhausted);
            debug_assert_eq!(revision_fault.kind(), FaultKind::SequenceExhausted);
        }
        let cause = self.events.last().map_or_else(
            || Cause::root(root),
            |event| Cause::root(root).with_parent(event.id()),
        );
        self.emit(cause, BattleEventKind::Fault(FaultEventData::new(fault)));
        self.events
    }
}

impl ActionIdentityAllocator for Transaction<'_> {
    fn action(&mut self) -> ActionId {
        self.allocate_action()
    }

    fn phase(&mut self) -> PhaseId {
        self.allocate_phase()
    }

    fn hit(&mut self) -> HitId {
        self.allocate_hit()
    }

    fn operation(&mut self) -> OperationId {
        self.allocate_operation()
    }
}

fn fault_code(fault: BattleFault) -> u64 {
    u64::from(fault.context_code()) << 24
        | (fault.kind() as u64) << 16
        | (fault.boundary() as u64) << 8
        | fault.policy() as u64
}

fn event_ids(events: &[BattleEvent]) -> Vec<EventId> {
    events.iter().map(BattleEvent::id).collect()
}
