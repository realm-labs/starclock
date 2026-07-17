use core::mem;

use crate::{
    battle::{
        fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
        model::BattlePhase,
        state::BattleState,
    },
    codec::{BattleStateHash, hash_state},
    command::{legal, model::DecisionPoint, validate::ValidatedCommand},
    event::{
        cause::Cause,
        model::{BattleEvent, BattleEventData, BattleEventKind, DecisionEventData, FaultEventData},
    },
    id::{CommandId, DecisionId, EventId},
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
    before: &BattleState,
    scratch: &mut ResolutionScratch,
    command: ValidatedCommand,
    injection: Option<FaultInjection>,
) -> TransactionOutput {
    let (mut events, root_command, failure) = {
        let mut txn = Transaction::new(&mut scratch.working, &mut scratch.journal);
        let root = txn.begin_command();
        let failure = execute(&mut txn, root, command, injection).err();
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
            let decision_id = txn.allocate_decision();
            let decision = legal::initial_player_action(decision_id, txn.state.concede);
            let decision_fact = DecisionEventData::Offered {
                decision: decision_id,
                kind: decision.kind(),
                owner: decision.owner(),
            };
            txn.set_decision(Some(decision));
            txn.set_phase(BattlePhase::AwaitingCommand);
            txn.emit(
                Cause::root(root).with_parent(started),
                BattleEventKind::Decision(decision_fact),
            );
        }
        ValidatedCommand::Concede => {
            txn.set_decision(None);
            txn.set_phase(BattlePhase::Lost);
            txn.emit(
                Cause::root(root),
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

fn fault_code(fault: BattleFault) -> u64 {
    u64::from(fault.context_code()) << 24
        | (fault.kind() as u64) << 16
        | (fault.boundary() as u64) << 8
        | fault.policy() as u64
}

fn event_ids(events: &[BattleEvent]) -> Vec<EventId> {
    events.iter().map(BattleEvent::id).collect()
}
