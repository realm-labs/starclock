//! Deterministic complete-run execution used by the frozen Goal 14 matrix.

use std::collections::{BTreeSet, VecDeque};

use starclock_activity::{
    ActivityCause, ActivityDefinitionIdentity, ActivityEdgeId, ActivityInstanceId,
    ActivityMasterSeed, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityRngContext, ActivityRngStreams, ActivityStateHash, ActivityTerminalOutcome,
    ActivityTransactionOutcome, ActivityTransactionState, AttemptId, BattleResult,
    BattleResultDigest, BattleResultIdentity, BattleSequence, NodeId,
};

use crate::{
    battle_materialization::UniverseBattleRoster, digest::Encoder,
    nested_battle_executor::NestedBattleExecutionReport,
};

use super::{
    GoldAndGearsBattleAssemblyContext, GoldAndGearsBattleExecutionError, GoldAndGearsEncounterRole,
    GoldAndGearsEntryError, GoldAndGearsExtrapolationContext, GoldAndGearsRuntimeInstance,
};

/// Stable deterministic runner contract used by the P0-frozen seeded matrix.
pub const GOLD_AND_GEARS_SEEDED_RUN_REVISION: &str = "gold-and-gears-seeded-run-v1";

const TRAVERSE_PROGRAM_BASE: u32 = 0x7f74_0000;
const MAX_STEPS: usize = 256;
const DEFAULT_BOSS: &str = "gold-gears.boss-choice.1013014";
const FINAL_BOSS: &str = "gold-gears.boss-choice.8024011";

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

pub(super) struct GoldAndGearsSeededBattleRecord {
    pub(super) start_identity: BattleResultIdentity,
    pub(super) result: BattleResult,
    pub(super) report: NestedBattleExecutionReport,
}

pub(super) struct GoldAndGearsSeededReplayStep {
    pub(super) action: GoldAndGearsSeededRunAction,
    pub(super) state_hash: ActivityStateHash,
    pub(super) battle: Option<GoldAndGearsSeededBattleRecord>,
}

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
        let mut state = ActivityTransactionState::new(
            self.state_definition().clone(),
            self.graph_definition().entry(),
        );
        let mut rng = ActivityRngStreams::new(ActivityRngContext::new(
            ActivityMasterSeed::from_u64(request.seed),
            request.identity.id(),
            request.identity.definition_digest(),
            request.identity.config_digest(),
            self.graph_definition().digest(),
            request.activity_instance,
            None,
            Some(self.graph_definition().entry()),
            None,
            0,
        ));
        let starts = self.plane_starts().collect::<Vec<_>>();
        let mut created = [false; 3];
        let mut steps = Vec::new();
        let mut replay = Vec::new();
        let mut battle_count = 0_u32;

        for _ in 0..MAX_STEPS {
            if let Some(terminal) = state.terminal() {
                if terminal != ActivityTerminalOutcome::Completed {
                    return Err(GoldAndGearsSeededRunError::UnexpectedTerminal(terminal));
                }
                let final_state_hash = state.state_hash(
                    request.identity,
                    self.graph_definition(),
                    request.activity_instance,
                    &rng,
                );
                let transcript_digest = transcript_digest(
                    request.seed,
                    terminal,
                    final_state_hash,
                    battle_count,
                    &steps,
                );
                return Ok(GoldAndGearsRecordedExecution {
                    report: GoldAndGearsSeededRunReport {
                        seed: request.seed,
                        terminal,
                        final_state_hash,
                        battle_count,
                        steps: steps.into_boxed_slice(),
                        transcript_digest,
                    },
                    replay: replay.into_boxed_slice(),
                });
            }

            let node = state.current_node();
            if let Some(plane) = starts.iter().position(|candidate| *candidate == node)
                && !created[plane]
            {
                let program = self
                    .compile_plane_creation(plane, &mut rng)
                    .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
                apply_program(self, &mut state, program)?;
                created[plane] = true;
                let accepted = step(
                    GoldAndGearsSeededRunStepKind::PlaneCreation,
                    node,
                    &state,
                    &rng,
                    request,
                    None,
                    self,
                );
                replay.push(GoldAndGearsSeededReplayStep {
                    action: GoldAndGearsSeededRunAction::PlaneCreation {
                        source_node: node,
                        plane: u8::try_from(plane + 1)
                            .expect("the frozen run has exactly three planes"),
                    },
                    state_hash: accepted.state_hash,
                    battle: None,
                });
                steps.push(accepted);
                continue;
            }

            if let Some(role) = self.encounter_role_for_node(&state, node)
                && !state.current_battle_attempt_is_settled()
            {
                if matches!(
                    role,
                    GoldAndGearsEncounterRole::FirstPlaneBoss
                        | GoldAndGearsEncounterRole::SecondPlaneBoss
                        | GoldAndGearsEncounterRole::FinalBoss
                ) {
                    let plane = match role {
                        GoldAndGearsEncounterRole::FirstPlaneBoss => 1,
                        GoldAndGearsEncounterRole::SecondPlaneBoss => 2,
                        GoldAndGearsEncounterRole::FinalBoss => 3,
                        GoldAndGearsEncounterRole::Combat | GoldAndGearsEncounterRole::Elite => {
                            unreachable!("guard admits only boss roles")
                        }
                    };
                    let boss = if role == GoldAndGearsEncounterRole::FinalBoss {
                        FINAL_BOSS
                    } else {
                        DEFAULT_BOSS
                    };
                    let program = self
                        .compile_boss_selection(plane, boss)
                        .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
                    apply_program(self, &mut state, program)?;
                    let accepted = step(
                        GoldAndGearsSeededRunStepKind::BossSelection,
                        node,
                        &state,
                        &rng,
                        request,
                        None,
                        self,
                    );
                    replay.push(GoldAndGearsSeededReplayStep {
                        action: GoldAndGearsSeededRunAction::BossSelection {
                            source_node: node,
                            plane,
                            boss: boss.into(),
                        },
                        state_hash: accepted.state_hash,
                        battle: None,
                    });
                    steps.push(accepted);
                }

                let selection = self
                    .select_current_encounter(&state, &mut rng)
                    .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
                battle_count = battle_count
                    .checked_add(1)
                    .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?;
                let mut context = GoldAndGearsBattleAssemblyContext::new(Vec::new(), false);
                if role == GoldAndGearsEncounterRole::FinalBoss {
                    let extrapolation = self
                        .compile_resonance_extrapolation(
                            GoldAndGearsExtrapolationContext::new(3, true, self.path()),
                            &mut rng,
                        )
                        .map_err(GoldAndGearsSeededRunError::InvalidInput)?;
                    context = context.with_extrapolation(extrapolation);
                }
                let expected = state.state_hash(
                    request.identity,
                    self.graph_definition(),
                    request.activity_instance,
                    &rng,
                );
                let start = self
                    .start_current_battle(
                        &mut state,
                        &rng,
                        expected,
                        request.identity,
                        request.activity_instance,
                        AttemptId::new(battle_count)
                            .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
                        BattleSequence::new(battle_count)
                            .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?,
                        &selection,
                        roster,
                        &context,
                    )
                    .map_err(GoldAndGearsSeededRunError::Battle)?;
                let execution = self
                    .execute_started_battle(
                        &mut state,
                        &rng,
                        request.identity,
                        request.activity_instance,
                        &start,
                    )
                    .map_err(GoldAndGearsSeededRunError::Battle)?;
                if let Some(fault) = execution.report().terminal_fault() {
                    return Err(GoldAndGearsSeededRunError::BattleFault {
                        role,
                        group: selection.group().into(),
                        fault,
                    });
                }
                if let Some(fault) = execution.post_battle_events().iter().find_map(|event| {
                    if let starclock_activity::ActivityTransactionEventKind::Faulted(fault) =
                        event.kind()
                    {
                        Some(*fault)
                    } else {
                        None
                    }
                }) {
                    return Err(GoldAndGearsSeededRunError::PostBattleFault { role, fault, node });
                }
                if let Some(terminal) = state.terminal()
                    && terminal != ActivityTerminalOutcome::Completed
                {
                    return Err(GoldAndGearsSeededRunError::UnexpectedBattleTerminal {
                        role,
                        terminal,
                        node,
                    });
                }
                let accepted = step(
                    GoldAndGearsSeededRunStepKind::Battle(role),
                    node,
                    &state,
                    &rng,
                    request,
                    Some(execution.result().actual_digest()),
                    self,
                );
                replay.push(GoldAndGearsSeededReplayStep {
                    action: GoldAndGearsSeededRunAction::Battle {
                        source_node: node,
                        role,
                        group: selection.group().into(),
                        member: selection.source_rogue_monster_id().into(),
                        effective_level: selection.effective_level(),
                    },
                    state_hash: accepted.state_hash,
                    battle: Some(GoldAndGearsSeededBattleRecord {
                        start_identity: start.handoff().identity(),
                        result: execution.result().clone(),
                        report: execution.report().clone(),
                    }),
                });
                steps.push(accepted);
                continue;
            }

            let plane = self
                .graph_definition()
                .node(node)
                .and_then(|definition| usize::try_from(definition.section().get()).ok())
                .and_then(|section| section.checked_sub(1))
                .ok_or(GoldAndGearsSeededRunError::NoRoute(node))?;
            let target = self
                .plane_ends()
                .nth(plane)
                .ok_or(GoldAndGearsSeededRunError::NoRoute(node))?;
            let edge = next_route(self, &state, node, target)
                .ok_or(GoldAndGearsSeededRunError::NoRoute(node))?;
            let id = TRAVERSE_PROGRAM_BASE
                .checked_add(edge.get())
                .and_then(ActivityProgramId::new)
                .ok_or(GoldAndGearsSeededRunError::StepBudgetExceeded)?;
            let program =
                ActivityProgramDefinition::new(id, vec![ActivityOperation::Traverse(edge)])
                    .map_err(|_| GoldAndGearsSeededRunError::ProgramRejected)?;
            apply_program(self, &mut state, program)?;
            let accepted = step(
                GoldAndGearsSeededRunStepKind::Traverse,
                node,
                &state,
                &rng,
                request,
                None,
                self,
            );
            replay.push(GoldAndGearsSeededReplayStep {
                action: GoldAndGearsSeededRunAction::Traverse {
                    source_node: node,
                    edge,
                },
                state_hash: accepted.state_hash,
                battle: None,
            });
            steps.push(accepted);
        }
        Err(GoldAndGearsSeededRunError::StepBudgetExceeded)
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

fn next_route(
    instance: &GoldAndGearsRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    target: NodeId,
) -> Option<starclock_activity::ActivityEdgeId> {
    let mut visited = BTreeSet::from([source]);
    let mut queue = VecDeque::new();
    for edge in instance.graph_definition().outgoing(source) {
        if instance.legal_routes(state, source).contains(&edge.id()) {
            queue.push_back((edge.to(), edge.id()));
        }
    }
    while let Some((node, first)) = queue.pop_front() {
        if node == target {
            return Some(first);
        }
        if !visited.insert(node) {
            continue;
        }
        let legal = instance.legal_routes(state, node);
        for edge in instance.graph_definition().outgoing(node) {
            if legal.contains(&edge.id()) {
                queue.push_back((edge.to(), first));
            }
        }
    }
    None
}

fn apply_program(
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
fn step(
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

fn transcript_digest(
    seed: u64,
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    battle_count: u32,
    steps: &[GoldAndGearsSeededRunStep],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.seeded-run.v1");
    encoder.text(GOLD_AND_GEARS_SEEDED_RUN_REVISION);
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
