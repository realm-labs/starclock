//! Deterministic complete-run execution used by the frozen Goal 14 matrix.

use starclock_activity::{
    ActivityCause, ActivityDecisionId, ActivityDefinitionIdentity, ActivityEdgeId,
    ActivityInstanceId, ActivityProgramDefinition, ActivityRngStreams, ActivityStateHash,
    ActivityTerminalOutcome, ActivityTransactionOutcome, ActivityTransactionState, BattleResult,
    BattleResultDigest, BattleResultIdentity, NodeId,
};

use crate::{
    battle_materialization::UniverseBattleRoster, digest::Encoder,
    nested_battle_executor::NestedBattleExecutionReport,
};

use super::{
    GoldAndGearsBaselineController, GoldAndGearsBaselineDecision, GoldAndGearsBaselineError,
    GoldAndGearsBattleExecutionError, GoldAndGearsEncounterRole, GoldAndGearsEntryError,
    GoldAndGearsOfferedCommand, GoldAndGearsRuntimeInstance,
    incremental_run::GoldAndGearsIncrementalRun,
};

/// Deterministic runner used by the explicit exhaustive seeded matrix.
pub(super) const MAX_SEEDED_RUN_STEPS: usize = 256;

/// Stable inputs which bind one deterministic complete-run execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsSeededRunRequest {
    seed: u64,
    identity: ActivityDefinitionIdentity,
    activity_instance: ActivityInstanceId,
}

impl GoldAndGearsSeededRunRequest {
    #[must_use]
    pub const fn new(
        seed: u64,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
    ) -> Self {
        Self {
            seed,
            identity,
            activity_instance,
        }
    }

    #[must_use]
    pub const fn seed(self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn identity(self) -> ActivityDefinitionIdentity {
        self.identity
    }

    #[must_use]
    pub const fn activity_instance(self) -> ActivityInstanceId {
        self.activity_instance
    }
}

/// One accepted boundary in a seeded complete-run transcript.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsSeededRunStepKind {
    PlaneCreation,
    BossSelection,
    Traverse,
    Battle(GoldAndGearsEncounterRole),
}

/// Exact accepted action retained by component-addressed replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsSeededRunAction {
    PlaneCreation {
        source_node: NodeId,
        plane: u8,
    },
    BossSelection {
        source_node: NodeId,
        plane: u8,
        boss: Box<str>,
    },
    Traverse {
        source_node: NodeId,
        edge: ActivityEdgeId,
    },
    Battle {
        source_node: NodeId,
        role: GoldAndGearsEncounterRole,
        group: Box<str>,
        member: Box<str>,
        effective_level: u16,
    },
}

#[derive(Clone)]
pub(super) struct GoldAndGearsSeededBattleRecord {
    pub(super) start_identity: BattleResultIdentity,
    pub(super) result: BattleResult,
    pub(super) report: NestedBattleExecutionReport,
}

#[derive(Clone)]
pub(super) struct GoldAndGearsSeededReplayStep {
    pub(super) action: GoldAndGearsSeededRunAction,
    pub(super) state_hash: ActivityStateHash,
    pub(super) battle: Option<GoldAndGearsSeededBattleRecord>,
}

#[derive(Clone)]
pub(super) struct GoldAndGearsRecordedExecution {
    pub(super) report: GoldAndGearsSeededRunReport,
    pub(super) replay: Box<[GoldAndGearsSeededReplayStep]>,
}

/// Canonical state evidence after one accepted seeded-run boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsSeededRunStep {
    kind: GoldAndGearsSeededRunStepKind,
    source_node: NodeId,
    state_hash: ActivityStateHash,
    result_digest: Option<BattleResultDigest>,
}

impl GoldAndGearsSeededRunStep {
    #[must_use]
    pub const fn kind(self) -> GoldAndGearsSeededRunStepKind {
        self.kind
    }

    #[must_use]
    pub const fn source_node(self) -> NodeId {
        self.source_node
    }

    #[must_use]
    pub const fn state_hash(self) -> ActivityStateHash {
        self.state_hash
    }

    #[must_use]
    pub const fn result_digest(self) -> Option<BattleResultDigest> {
        self.result_digest
    }
}

/// Terminal deterministic evidence for one complete frozen-matrix row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsSeededRunReport {
    seed: u64,
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    battle_count: u32,
    steps: Box<[GoldAndGearsSeededRunStep]>,
    transcript_digest: [u8; 32],
}

impl GoldAndGearsSeededRunReport {
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub const fn terminal(&self) -> ActivityTerminalOutcome {
        self.terminal
    }

    #[must_use]
    pub const fn final_state_hash(&self) -> ActivityStateHash {
        self.final_state_hash
    }

    #[must_use]
    pub const fn battle_count(&self) -> u32 {
        self.battle_count
    }

    #[must_use]
    pub fn steps(&self) -> &[GoldAndGearsSeededRunStep] {
        &self.steps
    }

    #[must_use]
    pub const fn transcript_digest(&self) -> [u8; 32] {
        self.transcript_digest
    }
}

/// Stable failures for complete execution and fresh transcript verification.
#[derive(Debug)]
pub enum GoldAndGearsSeededRunError {
    InvalidInput(GoldAndGearsEntryError),
    Controller(GoldAndGearsBaselineError),
    ProgramRejected,
    Battle(GoldAndGearsBattleExecutionError),
    BattleFault {
        role: GoldAndGearsEncounterRole,
        group: Box<str>,
        fault: starclock_combat::BattleFault,
    },
    UnexpectedBattleTerminal {
        role: GoldAndGearsEncounterRole,
        terminal: ActivityTerminalOutcome,
        node: NodeId,
    },
    PostBattleFault {
        role: GoldAndGearsEncounterRole,
        fault: starclock_activity::ActivityFault,
        node: NodeId,
    },
    StepBudgetExceeded,
    UnexpectedTerminal(ActivityTerminalOutcome),
    NoRoute(NodeId),
    CommandNotOffered,
    IncompleteRun,
    ReplayDivergence {
        step: usize,
    },
}

impl GoldAndGearsRuntimeInstance {
    /// Executes one complete three-plane run through ordinary Activity
    /// programs and the real nested combat boundary.
    pub fn execute_seeded_run(
        &self,
        request: GoldAndGearsSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<GoldAndGearsSeededRunReport, GoldAndGearsSeededRunError> {
        self.execute_seeded_run_recorded(request, roster)
            .map(|execution| execution.report)
    }

    pub(super) fn execute_seeded_run_recorded(
        &self,
        request: GoldAndGearsSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<GoldAndGearsRecordedExecution, GoldAndGearsSeededRunError> {
        self.execute_seeded_run_recorded_with_decisions(request, roster, None)
    }

    pub(super) fn execute_seeded_run_recorded_with_decisions(
        &self,
        request: GoldAndGearsSeededRunRequest,
        roster: &UniverseBattleRoster,
        mut decisions: Option<&mut Vec<GoldAndGearsBaselineDecision>>,
    ) -> Result<GoldAndGearsRecordedExecution, GoldAndGearsSeededRunError> {
        let mut run = GoldAndGearsIncrementalRun::start(self, request);
        let controller = GoldAndGearsBaselineController::default();
        loop {
            run.settle_automatic(self, roster)?;
            if run.terminal().is_some() {
                return run.recorded_execution(self);
            }
            let offers = run.offered_commands(self)?;
            let selected = select_offered(controller, run.decision_id()?, &offers, &mut decisions)?;
            run.apply_offered_command(self, &selected)?;
        }
    }
    /// Re-executes a seeded transcript against this freshly compiled instance
    /// and reports the first deterministic boundary which differs.
    pub fn verify_seeded_run(
        &self,
        request: GoldAndGearsSeededRunRequest,
        roster: &UniverseBattleRoster,
        expected: &GoldAndGearsSeededRunReport,
    ) -> Result<GoldAndGearsSeededRunReport, GoldAndGearsSeededRunError> {
        let actual = self.execute_seeded_run(request, roster)?;
        if expected.seed != actual.seed {
            return Err(GoldAndGearsSeededRunError::ReplayDivergence { step: 0 });
        }
        let shared = expected.steps.len().min(actual.steps.len());
        if let Some(step) = (0..shared).find(|index| expected.steps[*index] != actual.steps[*index])
        {
            return Err(GoldAndGearsSeededRunError::ReplayDivergence { step });
        }
        if expected.steps.len() != actual.steps.len()
            || expected.terminal != actual.terminal
            || expected.final_state_hash != actual.final_state_hash
            || expected.battle_count != actual.battle_count
            || expected.transcript_digest != actual.transcript_digest
        {
            return Err(GoldAndGearsSeededRunError::ReplayDivergence { step: shared });
        }
        Ok(actual)
    }
}

fn select_offered(
    controller: GoldAndGearsBaselineController,
    decision: ActivityDecisionId,
    offers: &[GoldAndGearsOfferedCommand],
    decisions: &mut Option<&mut Vec<GoldAndGearsBaselineDecision>>,
) -> Result<GoldAndGearsOfferedCommand, GoldAndGearsSeededRunError> {
    let selected = controller
        .decide(decision, offers)
        .map_err(GoldAndGearsSeededRunError::Controller)?;
    let command = selected.selected().clone();
    if let Some(decisions) = decisions.as_deref_mut() {
        decisions.push(selected);
    }
    Ok(command)
}

pub(super) fn apply_program(
    instance: &GoldAndGearsRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) -> Result<(), GoldAndGearsSeededRunError> {
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .map_err(|_| GoldAndGearsSeededRunError::ProgramRejected)?;
    let cause = ActivityCause::new(
        state
            .command_sequence()
            .checked_add(1)
            .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
        program.id(),
        state.current_node(),
    )
    .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?;
    match state.apply_program(&program, cause, instance.graph_definition()) {
        ActivityTransactionOutcome::Committed(_) => Ok(()),
        ActivityTransactionOutcome::Rejected(_) | ActivityTransactionOutcome::Faulted(..) => {
            Err(GoldAndGearsSeededRunError::ProgramRejected)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn step(
    kind: GoldAndGearsSeededRunStepKind,
    source_node: NodeId,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
    request: GoldAndGearsSeededRunRequest,
    result_digest: Option<BattleResultDigest>,
    instance: &GoldAndGearsRuntimeInstance,
) -> GoldAndGearsSeededRunStep {
    GoldAndGearsSeededRunStep {
        kind,
        source_node,
        state_hash: state.state_hash(
            request.identity,
            instance.graph_definition(),
            request.activity_instance,
            rng,
        ),
        result_digest,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn terminal_execution(
    instance: &GoldAndGearsRuntimeInstance,
    request: GoldAndGearsSeededRunRequest,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
    battle_count: u32,
    steps: &[GoldAndGearsSeededRunStep],
    replay: &[GoldAndGearsSeededReplayStep],
) -> Result<GoldAndGearsRecordedExecution, GoldAndGearsSeededRunError> {
    let terminal = state
        .terminal()
        .ok_or(GoldAndGearsSeededRunError::IncompleteRun)?;
    if terminal != ActivityTerminalOutcome::Completed {
        return Err(GoldAndGearsSeededRunError::UnexpectedTerminal(terminal));
    }
    let final_state_hash = state.state_hash(
        request.identity(),
        instance.graph_definition(),
        request.activity_instance(),
        rng,
    );
    Ok(GoldAndGearsRecordedExecution {
        report: GoldAndGearsSeededRunReport {
            seed: request.seed(),
            terminal,
            final_state_hash,
            battle_count,
            steps: steps.to_vec().into_boxed_slice(),
            transcript_digest: transcript_digest(
                request.seed(),
                terminal,
                final_state_hash,
                battle_count,
                steps,
            ),
        },
        replay: replay.to_vec().into_boxed_slice(),
    })
}

fn transcript_digest(
    seed: u64,
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    battle_count: u32,
    steps: &[GoldAndGearsSeededRunStep],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.seeded-run");
    encoder.u64(seed);
    encoder.u8(terminal_code(terminal));
    encoder.digest(final_state_hash.bytes());
    encoder.u32(battle_count);
    encoder.u32(u32::try_from(steps.len()).expect("bounded seeded steps fit u32"));
    for step in steps {
        match step.kind {
            GoldAndGearsSeededRunStepKind::PlaneCreation => encoder.u8(0),
            GoldAndGearsSeededRunStepKind::BossSelection => encoder.u8(1),
            GoldAndGearsSeededRunStepKind::Traverse => encoder.u8(2),
            GoldAndGearsSeededRunStepKind::Battle(role) => {
                encoder.u8(3);
                encoder.u8(role_code(role));
            }
        }
        encoder.u32(step.source_node.get());
        encoder.digest(step.state_hash.bytes());
        encoder.optional_digest(step.result_digest.map(BattleResultDigest::bytes));
    }
    encoder.finish()
}

fn terminal_code(terminal: ActivityTerminalOutcome) -> u8 {
    match terminal {
        ActivityTerminalOutcome::Completed => 0,
        ActivityTerminalOutcome::Failed => 1,
        ActivityTerminalOutcome::Abandoned => 2,
        ActivityTerminalOutcome::Faulted => 3,
    }
}

fn role_code(role: GoldAndGearsEncounterRole) -> u8 {
    match role {
        GoldAndGearsEncounterRole::Combat => 0,
        GoldAndGearsEncounterRole::Elite => 1,
        GoldAndGearsEncounterRole::FirstPlaneBoss => 2,
        GoldAndGearsEncounterRole::SecondPlaneBoss => 3,
        GoldAndGearsEncounterRole::FinalBoss => 4,
    }
}
