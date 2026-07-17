use core::mem;

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
    timeline::{
        queue::InterruptQueue,
        select::plan_next_turn,
        state::{InterruptWindowKind, InterruptWindowState, NormalTurnState},
    },
};

use super::{
    action::execute_action_plan,
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

#[derive(Debug)]
pub(crate) struct ResolutionScratch {
    working: BattleState,
    journal: MutationJournal,
    #[cfg(feature = "benchmark-instrumentation")]
    last_metrics: super::journal::JournalMetrics,
    #[cfg(test)]
    preparations: u64,
}

impl ResolutionScratch {
    pub(crate) fn from_state(state: &BattleState) -> Self {
        Self {
            working: state.semantic_clone(),
            journal: MutationJournal::default(),
            #[cfg(feature = "benchmark-instrumentation")]
            last_metrics: super::journal::JournalMetrics::default(),
            #[cfg(test)]
            preparations: 1,
        }
    }

    pub(crate) fn prepare(&mut self, state: &BattleState) {
        self.working.clone_from_semantics(state);
        self.journal.clear();
        #[cfg(test)]
        {
            self.preparations += 1;
        }
    }

    pub(crate) fn commit_into(&mut self, authoritative: &mut BattleState) {
        mem::swap(&mut self.working, authoritative);
        #[cfg(feature = "benchmark-instrumentation")]
        {
            self.last_metrics = self.journal.metrics();
        }
        self.journal.release_bounded();
    }

    #[cfg(feature = "benchmark-instrumentation")]
    pub(crate) const fn last_metrics(&self) -> super::journal::JournalMetrics {
        self.last_metrics
    }

    #[cfg(test)]
    pub(crate) const fn preparations(&self) -> u64 {
        self.preparations
    }
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
        let failure = execute(catalog, &mut txn, root, command, injection).err();
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
            begin_turn(catalog, txn, root, started)?;
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
            offer_decision(txn, root, Some(closed), decision);
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
            let mut plan = lower_normal_action(catalog, txn, actor, turn.actor, ability, targets)
                .ok_or_else(|| action_fault(5))?;
            let action_resolved = execute_action_plan(txn, root, closed, &mut plan)?;
            let boundary_cause = action_cause(root, &plan)?;
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
            txn.set_active_turn(None);
            if let ActionBoundary::Continue(parent) =
                settle_after_action(txn, boundary_cause, ended)?
            {
                begin_turn(catalog, txn, root, parent)?;
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
            let mut plan = lower_interrupt_action(catalog, txn, actor, ability, targets)
                .ok_or_else(|| action_fault(12))?;
            let resolved = execute_action_plan(txn, root, closed, &mut plan)?;
            let boundary_cause = action_cause(root, &plan)?;
            if let ActionBoundary::Continue(parent) =
                settle_after_action(txn, boundary_cause, resolved)?
            {
                offer_interrupt_decision(catalog, txn, root, parent)?;
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
    maybe_inject(injection, FaultInjectionPoint::AfterCommandMutation)?;
    txn.bump_revision()?;
    Ok(())
}

fn action_cause(
    root: CommandId,
    plan: &crate::action::model::ActionPlan,
) -> Result<Cause, BattleFault> {
    let source = SourceDefinitionId::new(plan.ability.get()).ok_or_else(|| action_fault(42))?;
    Ok(Cause::for_action(
        root,
        plan.id,
        plan.actor,
        CauseActor::Unit(plan.actor),
        source,
    )
    .with_primary_target(plan.targets.primary)
    .with_applier(plan.actor))
}

fn commit_targets(
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

fn begin_turn(
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
    txn.set_active_turn(Some(turn));
    let started = txn.emit(
        Cause::for_turn(root, turn.owner, turn.actor).with_parent(parent),
        BattleEventKind::Turn(TurnEventData::Started {
            actor: turn.actor,
            owner: turn.owner,
        }),
    );
    let turn_cause = Cause::for_turn(root, turn.owner, turn.actor);
    let mut started = started;
    for (operation, element) in txn.tick_temporary_weaknesses(turn.owner)? {
        started = txn.emit(
            turn_cause
                .with_parent(started)
                .with_primary_target(Some(turn.owner)),
            BattleEventKind::Toughness(crate::ToughnessEventData::WeaknessRemoved {
                operation,
                target: turn.owner,
                element,
            }),
        );
    }
    let (mut started, frozen_skip) =
        super::operation::settle_break_effects_at_turn_start(txn, turn_cause, started, turn.owner)?;
    match settle_after_action(txn, turn_cause, started)? {
        ActionBoundary::Terminal(_) => return Ok(()),
        ActionBoundary::Continue(parent) => started = parent,
    }
    let alive = txn
        .state
        .units
        .get(turn.owner)
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
        started = txn.emit(
            turn_cause.with_parent(started),
            BattleEventKind::Turn(TurnEventData::Ended {
                actor: turn.actor,
                owner: turn.owner,
            }),
        );
        return begin_turn(catalog, txn, root, started);
    }
    let was_broken = txn
        .state
        .units
        .get(turn.owner)
        .map(|unit| unit.weakness_broken)
        .ok_or_else(|| action_fault(60))?;
    if was_broken {
        let changes = txn.recover_toughness(turn.owner)?;
        txn.set_weakness_broken(turn.owner, false)?;
        for (layer_key, before, after) in changes {
            started = txn.emit(
                turn_cause
                    .with_parent(started)
                    .with_primary_target(Some(turn.owner)),
                BattleEventKind::Toughness(crate::ToughnessEventData::Recovered {
                    target: turn.owner,
                    layer_key,
                    before,
                    after,
                    exited_global_broken: true,
                }),
            );
        }
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
        catalog,
    );
    offer_decision(txn, root, Some(started), decision);
    Ok(())
}

fn offer_interrupt_decision(
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
        catalog,
    );
    offer_decision(txn, root, Some(parent), decision);
    Ok(())
}

fn offer_decision(
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
}

impl<'a> Transaction<'a> {
    fn new(state: &'a mut BattleState, journal: &'a mut MutationJournal) -> Self {
        Self {
            state,
            journal,
            events: Vec::new(),
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
        }
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

    fn allocate_decision(&mut self) -> DecisionId {
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

    fn allocate_operation(&mut self) -> OperationId {
        let id = self
            .state
            .sequences
            .try_operation()
            .expect("rules-revision operation budget prevents u64 identity exhaustion");
        self.journal.allocation(AllocationKind::Operation, id.get());
        id
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
            .sample_below(DrawPurpose::EFFECT_CHANCE, 1_000_000)
            .map_err(|_| action_fault(51))?;
        for index in before..self.state.rng.draw_count() {
            self.journal
                .rng_draw(index, DrawPurpose::EFFECT_CHANCE.code());
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

    pub(super) fn resolve_hit_targets(
        &mut self,
        actor: crate::UnitId,
        commitment: &mut crate::target::model::TargetCommitment,
    ) -> Result<Box<[crate::UnitId]>, BattleFault> {
        let units = &self.state.units;
        let formations = &self.state.formations;
        let rng = &mut self.state.rng;
        let journal = &mut self.journal;
        select::resolve_for_hit(units, formations, actor, commitment, |count| {
            let before = rng.draw_count();
            let selection = rng
                .choose_index(DrawPurpose::BOUNCE_TARGET, count)
                .map_err(|_| select::TargetError::ChoiceFailed)?
                .ok_or(select::TargetError::ChoiceFailed)?;
            for index in before..rng.draw_count() {
                journal.rng_draw(index, DrawPurpose::BOUNCE_TARGET.code());
            }
            usize::try_from(selection.value()).map_err(|_| select::TargetError::ChoiceFailed)
        })
        .map_err(|_| action_fault(32))
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
