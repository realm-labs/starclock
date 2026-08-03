//! Deterministic complete-run execution for the frozen Goal 20 matrix.

use crate::error::UniverseCatalogLoadError;
use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionIdentity, ActivityInstanceId,
    ActivityProgramDefinition, ActivityRngStreams, ActivityStateHash, ActivityTerminalOutcome,
    ActivityTransactionOutcome, ActivityTransactionState, BattleResult, BattleResultDigest,
    BattleResultIdentity, NodeId,
};

use crate::{
    battle_materialization::UniverseBattleRoster,
    nested_battle_executor::NestedBattleExecutionReport,
};

use super::{
    SwarmDisasterRuntimeInstance,
    baseline_controller::{
        SwarmBaselineController, SwarmBaselineDecision, SwarmBaselineError, select_offered,
    },
    encounter_runtime::{EncounterRole, EncounterSelection},
    incremental_run::SwarmDisasterIncrementalRun,
    replay_action::SwarmSeededRunAction,
};

pub(super) const MAXIMUM_STEPS: usize = 256;
pub(super) const PLANE_ONE_DECAY: &str = "swarm-disaster.boss-decay.1";
pub(super) const PLANE_TWO_DECAY: &str = "swarm-disaster.boss-decay.25";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
// Boundary variants are exercised by the frozen release matrix while the
// public baseline replay entry deliberately constructs only `Baseline`.
#[allow(dead_code)]
pub(super) enum SwarmSeededBoundary {
    Baseline,
    InitialCountdown,
    MoveOneToZero,
    EnterDisarrayOne,
    ReachDisarray(u8),
    CrossPlaneCountdownCarry,
    FinalBossDecay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SwarmSeededRunRequest {
    pub(super) seed: u64,
    pub(super) identity: ActivityDefinitionIdentity,
    pub(super) activity_instance: ActivityInstanceId,
    pub(super) config_digest: ActivityConfigDigest,
    pub(super) boundary: SwarmSeededBoundary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SwarmSeededStepKind {
    ProfileEntry,
    AudienceInitialization,
    TrailRunStart,
    CountdownSetup,
    PlaneCreation(u8),
    DiceRoll,
    Traverse,
    BossSelection(u8),
    Battle(EncounterRole),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct SwarmSeededRunStep {
    pub(super) kind: SwarmSeededStepKind,
    pub(super) source_node: NodeId,
    pub(super) state_hash: ActivityStateHash,
    pub(super) result_digest: Option<BattleResultDigest>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SwarmSeededRunReport {
    pub(super) terminal: ActivityTerminalOutcome,
    pub(super) final_state_hash: ActivityStateHash,
    pub(super) transcript_digest: [u8; 32],
    pub(super) battle_count: u32,
    pub(super) step_count: u32,
    pub(super) maximum_disarray_level: i64,
    pub(super) cross_plane_countdown_carried: bool,
    pub(super) steps: Box<[SwarmSeededRunStep]>,
}

#[derive(Clone)]
pub(super) struct SwarmSeededBattleRecord {
    pub(super) start_identity: BattleResultIdentity,
    pub(super) result: BattleResult,
    pub(super) report: NestedBattleExecutionReport,
}

#[derive(Clone)]
pub(super) struct SwarmSeededReplayStep {
    pub(super) action: SwarmSeededRunAction,
    pub(super) state_hash: ActivityStateHash,
    pub(super) battle: Option<SwarmSeededBattleRecord>,
}

pub(super) struct SwarmRecordedExecution {
    pub(super) report: SwarmSeededRunReport,
    pub(super) replay: Box<[SwarmSeededReplayStep]>,
}

pub(super) enum SwarmSeededRunError {
    Catalog(UniverseCatalogLoadError),
    Controller(SwarmBaselineError),
    ProgramRejected,
    MissingRoute(NodeId),
    MissingBossChoice(NodeId),
    BattleNotWon(EncounterRole),
    UnexpectedTerminal(ActivityTerminalOutcome),
    Incomplete,
    CommandNotOffered,
    BoundaryNotObserved(SwarmSeededBoundary),
    #[cfg(test)]
    ReplayDivergence,
    StepBudgetExceeded,
}

impl core::fmt::Debug for SwarmSeededRunError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Catalog(error) => formatter.debug_tuple("Catalog").field(error).finish(),
            Self::Controller(error) => formatter.debug_tuple("Controller").field(error).finish(),
            Self::ProgramRejected => formatter.write_str("ProgramRejected"),
            Self::MissingRoute(node) => formatter.debug_tuple("MissingRoute").field(node).finish(),
            Self::MissingBossChoice(node) => formatter
                .debug_tuple("MissingBossChoice")
                .field(node)
                .finish(),
            Self::BattleNotWon(role) => formatter.debug_tuple("BattleNotWon").field(role).finish(),
            Self::UnexpectedTerminal(terminal) => formatter
                .debug_tuple("UnexpectedTerminal")
                .field(terminal)
                .finish(),
            Self::Incomplete => formatter.write_str("Incomplete"),
            Self::CommandNotOffered => formatter.write_str("CommandNotOffered"),
            Self::BoundaryNotObserved(boundary) => formatter
                .debug_tuple("BoundaryNotObserved")
                .field(boundary)
                .finish(),
            #[cfg(test)]
            Self::ReplayDivergence => formatter.write_str("ReplayDivergence"),
            Self::StepBudgetExceeded => formatter.write_str("StepBudgetExceeded"),
        }
    }
}

impl From<UniverseCatalogLoadError> for SwarmSeededRunError {
    fn from(error: UniverseCatalogLoadError) -> Self {
        Self::Catalog(error)
    }
}

impl SwarmDisasterRuntimeInstance {
    #[cfg(test)]
    pub(super) fn execute_seeded_run(
        &self,
        request: SwarmSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<SwarmSeededRunReport, SwarmSeededRunError> {
        self.execute_seeded_run_recorded(request, roster)
            .map(|execution| execution.report)
    }

    pub(super) fn execute_seeded_run_recorded(
        &self,
        request: SwarmSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<SwarmRecordedExecution, SwarmSeededRunError> {
        self.execute_seeded_run_recorded_with_decisions(request, roster, None)
    }

    pub(super) fn execute_seeded_run_recorded_with_decisions(
        &self,
        request: SwarmSeededRunRequest,
        roster: &UniverseBattleRoster,
        mut decisions: Option<&mut Vec<SwarmBaselineDecision>>,
    ) -> Result<SwarmRecordedExecution, SwarmSeededRunError> {
        let mut run = SwarmDisasterIncrementalRun::start_request(self, request);
        let controller = SwarmBaselineController::default();
        loop {
            run.settle_automatic_internal(self, roster)?;
            if run.terminal().is_some() {
                return run.recorded_execution(self);
            }
            let offers = run.offered_swarm_commands(self)?;
            let selected = select_offered(controller, run.decision_id()?, &offers, &mut decisions)?;
            run.apply_swarm_command(self, &selected)?;
        }
    }

    #[cfg(test)]
    pub(super) fn verify_seeded_run(
        &self,
        request: SwarmSeededRunRequest,
        roster: &UniverseBattleRoster,
        expected: &SwarmSeededRunReport,
    ) -> Result<SwarmSeededRunReport, SwarmSeededRunError> {
        let actual = self.execute_seeded_run(request, roster)?;
        if &actual != expected {
            return Err(SwarmSeededRunError::ReplayDivergence);
        }
        Ok(actual)
    }
}

pub(super) fn configure_boundary(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rng: &ActivityRngStreams,
    request: SwarmSeededRunRequest,
    steps: &mut Vec<SwarmSeededRunStep>,
    replay: &mut Vec<SwarmSeededReplayStep>,
) -> Result<(), SwarmSeededRunError> {
    let delta = match request.boundary {
        SwarmSeededBoundary::MoveOneToZero => -19,
        SwarmSeededBoundary::EnterDisarrayOne | SwarmSeededBoundary::ReachDisarray(_) => -20,
        SwarmSeededBoundary::CrossPlaneCountdownCarry => -10,
        SwarmSeededBoundary::Baseline
        | SwarmSeededBoundary::InitialCountdown
        | SwarmSeededBoundary::FinalBossDecay => return Ok(()),
    };
    let program = instance.compile_countdown_adjustments(state, &[(0x6d04, delta)])?;
    let source = state.current_node();
    apply_and_record(
        instance,
        state,
        rng,
        request,
        program,
        SwarmSeededStepKind::CountdownSetup,
        SwarmSeededRunAction::CountdownSetup {
            source_node: source,
            delta,
        },
        steps,
        replay,
    )
}

pub(super) fn create_plane(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rng: &mut ActivityRngStreams,
    request: SwarmSeededRunRequest,
    plane: usize,
    steps: &mut Vec<SwarmSeededRunStep>,
    replay: &mut Vec<SwarmSeededReplayStep>,
) -> Result<(), SwarmSeededRunError> {
    let program = instance.compile_plane_creation(plane, rng)?;
    let source = state.current_node();
    let plane = u8::try_from(plane + 1).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
    apply_and_record(
        instance,
        state,
        rng,
        request,
        program,
        SwarmSeededStepKind::PlaneCreation(plane),
        SwarmSeededRunAction::PlaneCreation {
            source_node: source,
            plane,
        },
        steps,
        replay,
    )
}

pub(super) fn preview_encounter(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &mut ActivityRngStreams,
) -> Result<EncounterSelection, SwarmSeededRunError> {
    let mut selected = None;
    let _: Result<(), ()> = rng.transact(|working| {
        selected = Some(instance.select_current_encounter(state, working));
        Err(())
    });
    selected
        .ok_or(SwarmSeededRunError::Incomplete)?
        .map_err(Into::into)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_and_record(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rng: &ActivityRngStreams,
    request: SwarmSeededRunRequest,
    program: ActivityProgramDefinition,
    kind: SwarmSeededStepKind,
    action: SwarmSeededRunAction,
    steps: &mut Vec<SwarmSeededRunStep>,
    replay: &mut Vec<SwarmSeededReplayStep>,
) -> Result<(), SwarmSeededRunError> {
    let source = state.current_node();
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .map_err(|_| SwarmSeededRunError::ProgramRejected)?;
    let cause = ActivityCause::new(
        state
            .command_sequence()
            .checked_add(1)
            .ok_or(SwarmSeededRunError::StepBudgetExceeded)?,
        program.id(),
        source,
    )
    .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
    if !matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ) {
        return Err(SwarmSeededRunError::ProgramRejected);
    }
    let accepted = step(instance, state, rng, request, kind, source, None);
    replay.push(SwarmSeededReplayStep {
        action,
        state_hash: accepted.state_hash,
        battle: None,
    });
    steps.push(accepted);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn step(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
    request: SwarmSeededRunRequest,
    kind: SwarmSeededStepKind,
    source_node: NodeId,
    result_digest: Option<BattleResultDigest>,
) -> SwarmSeededRunStep {
    SwarmSeededRunStep {
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

pub(super) fn is_battle_domain(domain: Option<&str>) -> bool {
    matches!(
        domain,
        Some(
            "swarm-disaster.domain.monsternormal"
                | "swarm-disaster.domain.monsterelite"
                | "swarm-disaster.domain.monsterboss"
                | "swarm-disaster.domain.monsterswarm"
                | "swarm-disaster.domain.monsterswarmboss"
        )
    )
}

pub(super) const fn is_boss(role: EncounterRole) -> bool {
    !matches!(role, EncounterRole::Combat | EncounterRole::Elite)
}

pub(super) fn validate_boundary(
    boundary: SwarmSeededBoundary,
    maximum_disarray_level: i64,
    observed_one_to_zero: bool,
    observed_entry_one: bool,
    cross_plane_countdown_carried: bool,
) -> Result<(), SwarmSeededRunError> {
    let observed = match boundary {
        SwarmSeededBoundary::Baseline
        | SwarmSeededBoundary::InitialCountdown
        | SwarmSeededBoundary::FinalBossDecay => true,
        SwarmSeededBoundary::MoveOneToZero => observed_one_to_zero,
        SwarmSeededBoundary::EnterDisarrayOne => observed_entry_one,
        SwarmSeededBoundary::ReachDisarray(level) => {
            observed_entry_one && maximum_disarray_level >= i64::from(level)
        }
        SwarmSeededBoundary::CrossPlaneCountdownCarry => cross_plane_countdown_carried,
    };
    if observed {
        Ok(())
    } else {
        Err(SwarmSeededRunError::BoundaryNotObserved(boundary))
    }
}
