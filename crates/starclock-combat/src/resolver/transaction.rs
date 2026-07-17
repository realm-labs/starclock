use core::mem;

use crate::{
    action::{
        lower::{ActionIdentityAllocator, lower_normal_action},
        model::ActionPlan,
    },
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
            ActionEventData, BattleEvent, BattleEventData, BattleEventKind, DecisionEventData,
            FaultEventData, HitEventData, PhaseEventData, TurnEventData,
        },
    },
    id::{
        ActionId, CommandId, DecisionId, EventId, HitId, PhaseId, SourceDefinitionId,
        TimelineActorId,
    },
    numeric::domain::ActionGauge,
    timeline::{
        queue::InterruptQueue,
        select::plan_next_turn,
        state::{InterruptWindowKind, InterruptWindowState, NormalTurnState},
    },
};

use super::journal::{AllocationKind, MutationField, MutationJournal, phase_code};

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
    #[cfg(test)]
    preparations: u64,
}

impl ResolutionScratch {
    pub(crate) fn from_state(state: &BattleState) -> Self {
        Self {
            working: state.semantic_clone(),
            journal: MutationJournal::default(),
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
        self.journal.release_bounded();
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
            begin_turn(txn, root, started)?;
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
            let concede = txn.state.concede;
            let decision_id = txn.allocate_decision();
            let decision = legal::normal_action(
                decision_id,
                turn.side,
                turn.owner,
                &abilities,
                catalog,
                concede,
            );
            offer_decision(txn, root, Some(closed), decision);
        }
        ValidatedCommand::UseAbility { actor, ability } => {
            let closed = close_active_decision(txn, root)?;
            let turn = txn
                .state
                .timeline
                .active_turn
                .ok_or_else(|| action_fault(3))?;
            if turn.owner != actor || txn.state.timeline.interrupt.is_some() {
                return Err(action_fault(4));
            }
            let plan = lower_normal_action(catalog, txn, actor, turn.actor, ability)
                .ok_or_else(|| action_fault(5))?;
            let action_resolved = execute_action_plan(txn, root, closed, &plan)?;
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
            begin_turn(txn, root, ended)?;
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
    txn.set_interrupt(Some(InterruptWindowState {
        kind: InterruptWindowKind::PreAction,
        turn,
        pending: InterruptQueue::default(),
    }));
    let decision_id = txn.allocate_decision();
    let decision = legal::interrupt_window(decision_id, turn.side);
    offer_decision(txn, root, Some(started), decision);
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

fn execute_action_plan(
    txn: &mut Transaction<'_>,
    root: CommandId,
    command_parent: EventId,
    plan: &ActionPlan,
) -> Result<EventId, BattleFault> {
    let _normal_turn = plan.normal_turn.ok_or_else(|| action_fault(9))?;
    let source = SourceDefinitionId::new(plan.ability.get()).ok_or_else(|| action_fault(7))?;
    let base = Cause::for_action(
        root,
        plan.id,
        plan.actor,
        CauseActor::Unit(plan.actor),
        source,
    );
    let mut parent = txn.emit(
        base.with_parent(command_parent),
        BattleEventKind::Action(ActionEventData::Declared {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
        }),
    );
    parent = txn.emit(
        base.with_parent(parent),
        BattleEventKind::Action(ActionEventData::Started {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
        }),
    );
    for phase in &plan.phases {
        let phase_cause = base.with_phase(phase.id);
        parent = txn.emit(
            phase_cause.with_parent(parent),
            BattleEventKind::Phase(PhaseEventData::Started {
                action: plan.id,
                phase: phase.id,
            }),
        );
        for hit in &phase.hits {
            let hit_cause = phase_cause.with_hit(hit.id);
            parent = txn.emit(
                hit_cause.with_parent(parent),
                BattleEventKind::Hit(HitEventData::Started {
                    action: plan.id,
                    phase: phase.id,
                    hit: hit.id,
                }),
            );
            parent = txn.emit(
                hit_cause.with_parent(parent),
                BattleEventKind::Hit(HitEventData::Ended {
                    action: plan.id,
                    phase: phase.id,
                    hit: hit.id,
                }),
            );
        }
        parent = txn.emit(
            phase_cause.with_parent(parent),
            BattleEventKind::Phase(PhaseEventData::Ended {
                action: plan.id,
                phase: phase.id,
            }),
        );
    }
    let resolved = txn.emit(
        base.with_parent(parent),
        BattleEventKind::Action(ActionEventData::Resolved {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
        }),
    );
    Ok(resolved)
}

fn action_fault(context: u32) -> BattleFault {
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

struct Transaction<'a> {
    state: &'a mut BattleState,
    journal: &'a mut MutationJournal,
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

    fn set_phase(&mut self, phase: BattlePhase) {
        let before = self.state.phase;
        if before != phase {
            self.state.phase = phase;
            self.journal
                .mutation(MutationField::Phase, phase_code(before), phase_code(phase));
        }
    }

    fn set_decision(&mut self, decision: Option<DecisionPoint>) {
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

    fn set_active_turn(&mut self, turn: Option<NormalTurnState>) {
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

    fn set_interrupt(&mut self, interrupt: Option<InterruptWindowState>) {
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

    fn set_actor_gauge(
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

    fn emit(&mut self, cause: Cause, kind: BattleEventKind) -> EventId {
        let id = self.allocate_event();
        self.events.push(BattleEvent::new(id, cause, kind));
        self.journal.event(id);
        id
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
