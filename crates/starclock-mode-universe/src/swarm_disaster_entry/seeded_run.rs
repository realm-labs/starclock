//! Deterministic complete-run execution for the frozen Goal 20 matrix.

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionIdentity, ActivityInstanceId,
    ActivityMasterSeed, ActivityProgramDefinition, ActivityRngContext, ActivityRngStreams,
    ActivityStateHash, ActivityTerminalOutcome, ActivityTransactionOutcome,
    ActivityTransactionState, AttemptId, BattleResult, BattleResultDigest, BattleResultIdentity,
    BattleSequence, NodeId,
};

use crate::{
    battle_materialization::UniverseBattleRoster,
    nested_battle_executor::NestedBattleExecutionReport,
};

use super::{
    SwarmDisasterRuntimeInstance,
    encounter_runtime::{EncounterRole, EncounterSelection},
    replay_action::SwarmSeededRunAction,
    seeded_run_digest::transcript_digest,
    seeded_run_route::{explicit_face_target, longest_legal_route, movement_program, route_edge},
};

pub(super) const SWARM_DISASTER_SEEDED_RUN_REVISION: &str = "swarm-disaster-seeded-run-v1";
const MAXIMUM_STEPS: usize = 256;
const PLANE_ONE_DECAY: &str = "swarm-disaster.boss-decay.1";
const PLANE_TWO_DECAY: &str = "swarm-disaster.boss-decay.25";

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
    steps: Box<[SwarmSeededRunStep]>,
}

pub(super) struct SwarmSeededBattleRecord {
    pub(super) start_identity: BattleResultIdentity,
    pub(super) result: BattleResult,
    pub(super) report: NestedBattleExecutionReport,
}

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
    Catalog(crate::error::UniverseCatalogLoadError),
    ProgramRejected,
    MissingRoute(NodeId),
    MissingBossChoice(NodeId),
    BattleNotWon(EncounterRole),
    UnexpectedTerminal(ActivityTerminalOutcome),
    Incomplete,
    BoundaryNotObserved(SwarmSeededBoundary),
    #[cfg(test)]
    ReplayDivergence,
    StepBudgetExceeded,
}

impl core::fmt::Debug for SwarmSeededRunError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Catalog(error) => formatter.debug_tuple("Catalog").field(error).finish(),
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

impl From<crate::error::UniverseCatalogLoadError> for SwarmSeededRunError {
    fn from(error: crate::error::UniverseCatalogLoadError) -> Self {
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
        let mut state = ActivityTransactionState::new(
            self.state_definition().clone(),
            self.graph_definition().entry(),
        );
        let mut rng = ActivityRngStreams::new(ActivityRngContext::new(
            ActivityMasterSeed::from_u64(request.seed),
            request.identity.id(),
            request.identity.definition_digest(),
            request.config_digest,
            self.graph_definition().digest(),
            request.activity_instance,
            None,
            Some(self.graph_definition().entry()),
            None,
            0,
        ));
        let mut steps = Vec::new();
        let mut replay = Vec::new();
        if self.countdown(&state)? != 20 {
            return Err(SwarmSeededRunError::BoundaryNotObserved(
                SwarmSeededBoundary::InitialCountdown,
            ));
        }
        let profile = self.compile_profile_entry_rule(&state)?;
        let source = state.current_node();
        apply_and_record(
            self,
            &mut state,
            &rng,
            request,
            profile,
            SwarmSeededStepKind::ProfileEntry,
            SwarmSeededRunAction::ProfileEntry {
                source_node: source,
            },
            &mut steps,
            &mut replay,
        )?;
        let audience = self.compile_audience_initialization(&state)?;
        let source = state.current_node();
        apply_and_record(
            self,
            &mut state,
            &rng,
            request,
            audience,
            SwarmSeededStepKind::AudienceInitialization,
            SwarmSeededRunAction::AudienceInitialization {
                source_node: source,
            },
            &mut steps,
            &mut replay,
        )?;
        let trail = self.compile_trail_run_start(&state)?;
        let source = state.current_node();
        apply_and_record(
            self,
            &mut state,
            &rng,
            request,
            trail,
            SwarmSeededStepKind::TrailRunStart,
            SwarmSeededRunAction::TrailRunStart {
                source_node: source,
            },
            &mut steps,
            &mut replay,
        )?;
        configure_boundary(self, &mut state, &rng, request, &mut steps, &mut replay)?;
        create_plane(
            self,
            &mut state,
            &mut rng,
            request,
            0,
            &mut steps,
            &mut replay,
        )?;

        let plane_ends = self.plane_ends().collect::<Vec<_>>();
        let plane_starts = self.plane_starts().collect::<Vec<_>>();
        let mut plane = 0_usize;
        let mut battle_count = 0_u32;
        let mut maximum_disarray_level = self.disarray_level(&state)?;
        let mut observed_one_to_zero = false;
        let mut observed_entry_one = false;
        let mut cross_plane_countdown_carried = false;

        while state.terminal().is_none() {
            if steps.len() >= MAXIMUM_STEPS {
                return Err(SwarmSeededRunError::StepBudgetExceeded);
            }
            let node = state.current_node();
            let domain = self.map.node_domain_key(&state, node)?;
            if is_battle_domain(domain) && !state.current_battle_attempt_is_settled() {
                let selection = preview_encounter(self, &state, &mut rng)?;
                let role = selection.role;
                if is_boss(role) {
                    prepare_boss(
                        self,
                        &mut state,
                        request,
                        &mut steps,
                        &mut replay,
                        &mut rng,
                        role,
                        plane,
                    )?;
                }
                let before_transition = (self.countdown(&state)?, self.disarray_level(&state)?);
                let expected = state.state_hash(
                    request.identity,
                    self.graph_definition(),
                    request.activity_instance,
                    &rng,
                );
                let sequence = battle_count
                    .checked_add(1)
                    .and_then(BattleSequence::new)
                    .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
                let start = self.start_current_battle(
                    &mut state,
                    &mut rng,
                    expected,
                    request.identity,
                    request.activity_instance,
                    AttemptId::new(1).expect("static seeded Attempt is non-zero"),
                    sequence,
                    roster,
                )?;
                let start_identity = start.handoff().identity();
                let (result, report, _) = self.execute_started_battle(
                    &mut state,
                    &rng,
                    request.identity,
                    request.activity_instance,
                    &start,
                    false,
                )?;
                if report.outcome() != starclock_activity::BattleOutcome::Won {
                    return Err(SwarmSeededRunError::BattleNotWon(role));
                }
                battle_count = sequence.get();
                let accepted = step(
                    self,
                    &state,
                    &rng,
                    request,
                    SwarmSeededStepKind::Battle(role),
                    node,
                    Some(result.actual_digest()),
                );
                replay.push(SwarmSeededReplayStep {
                    action: SwarmSeededRunAction::Battle {
                        source_node: node,
                        role,
                        group: selection.group,
                        member: selection.source_rogue_monster_id,
                        effective_level: selection.effective_level,
                    },
                    state_hash: accepted.state_hash,
                    battle: Some(SwarmSeededBattleRecord {
                        start_identity,
                        result,
                        report,
                    }),
                });
                steps.push(accepted);
                if is_boss(role) && state.terminal().is_none() {
                    let after_transition = (self.countdown(&state)?, self.disarray_level(&state)?);
                    cross_plane_countdown_carried |= before_transition == after_transition;
                    plane = plane
                        .checked_add(1)
                        .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
                    if plane >= plane_starts.len() || state.current_node() != plane_starts[plane] {
                        return Err(SwarmSeededRunError::Incomplete);
                    }
                    create_plane(
                        self,
                        &mut state,
                        &mut rng,
                        request,
                        plane,
                        &mut steps,
                        &mut replay,
                    )?;
                }
                maximum_disarray_level = maximum_disarray_level.max(self.disarray_level(&state)?);
                continue;
            }

            if node == plane_ends[plane] {
                return Err(SwarmSeededRunError::Incomplete);
            }
            let target = longest_legal_route(self, &state, node, plane_ends[plane])?;
            let edge = route_edge(self, node, target)?;
            let before = (self.countdown(&state)?, self.disarray_level(&state)?);
            let program = if self.dice_roll_available(&state)? {
                let roll = self.compile_dice_roll(&state, &mut rng)?;
                apply_and_record(
                    self,
                    &mut state,
                    &rng,
                    request,
                    roll,
                    SwarmSeededStepKind::DiceRoll,
                    SwarmSeededRunAction::DiceRoll { source_node: node },
                    &mut steps,
                    &mut replay,
                )?;
                let explicit_face_target = explicit_face_target(self, &state, &mut rng)?;
                self.compile_simultaneous_resolution(
                    &state,
                    Some((target, &[])),
                    explicit_face_target,
                    None,
                    (None, None),
                    &mut rng,
                )?
            } else {
                movement_program(self, &state, target)?
            };
            apply_and_record(
                self,
                &mut state,
                &rng,
                request,
                program,
                SwarmSeededStepKind::Traverse,
                SwarmSeededRunAction::Traverse {
                    source_node: node,
                    edge,
                },
                &mut steps,
                &mut replay,
            )?;
            let after = (self.countdown(&state)?, self.disarray_level(&state)?);
            observed_one_to_zero |= before == (1, 0) && after == (0, 0);
            observed_entry_one |= before == (0, 0) && after == (-1, 1);
            maximum_disarray_level = maximum_disarray_level.max(after.1);
        }

        let terminal = state.terminal().ok_or(SwarmSeededRunError::Incomplete)?;
        if terminal != ActivityTerminalOutcome::Completed {
            return Err(SwarmSeededRunError::UnexpectedTerminal(terminal));
        }
        validate_boundary(
            request.boundary,
            maximum_disarray_level,
            observed_one_to_zero,
            observed_entry_one,
            cross_plane_countdown_carried,
        )?;
        let final_state_hash = state.state_hash(
            request.identity,
            self.graph_definition(),
            request.activity_instance,
            &rng,
        );
        let step_count =
            u32::try_from(steps.len()).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
        let transcript_digest = transcript_digest(
            request.seed,
            terminal,
            final_state_hash,
            battle_count,
            maximum_disarray_level,
            cross_plane_countdown_carried,
            &steps,
        );
        Ok(SwarmRecordedExecution {
            report: SwarmSeededRunReport {
                terminal,
                final_state_hash,
                transcript_digest,
                battle_count,
                step_count,
                maximum_disarray_level,
                cross_plane_countdown_carried,
                steps: steps.into_boxed_slice(),
            },
            replay: replay.into_boxed_slice(),
        })
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

fn configure_boundary(
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

fn create_plane(
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

#[allow(clippy::too_many_arguments)]
fn prepare_boss(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    request: SwarmSeededRunRequest,
    steps: &mut Vec<SwarmSeededRunStep>,
    replay: &mut Vec<SwarmSeededReplayStep>,
    encounter_rng: &mut ActivityRngStreams,
    role: EncounterRole,
    plane: usize,
) -> Result<(), SwarmSeededRunError> {
    let layer = u8::try_from(plane + 1).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
    if layer == 1 {
        let program = instance.compile_boss_decay_selection(state, &[PLANE_ONE_DECAY])?;
        let source = state.current_node();
        apply_and_record(
            instance,
            state,
            encounter_rng,
            request,
            program,
            SwarmSeededStepKind::BossSelection(layer),
            SwarmSeededRunAction::BossDecaySelection {
                source_node: source,
                plane: layer,
                decay: PLANE_ONE_DECAY.into(),
            },
            steps,
            replay,
        )?;
    } else if layer == 2 {
        let program = instance.compile_boss_decay_selection(state, &[PLANE_TWO_DECAY])?;
        let source = state.current_node();
        apply_and_record(
            instance,
            state,
            encounter_rng,
            request,
            program,
            SwarmSeededStepKind::BossSelection(layer),
            SwarmSeededRunAction::BossDecaySelection {
                source_node: source,
                plane: layer,
                decay: PLANE_TWO_DECAY.into(),
            },
            steps,
            replay,
        )?;
    } else if instance.countdown.selected_boss_decay(state)?.len() != 2 {
        return Err(SwarmSeededRunError::BoundaryNotObserved(
            SwarmSeededBoundary::FinalBossDecay,
        ));
    }
    let preview = preview_encounter(instance, state, encounter_rng)?;
    if preview.role != role {
        return Err(SwarmSeededRunError::Incomplete);
    }
    let boss = preview
        .waves
        .iter()
        .flat_map(|wave| wave.slots.iter())
        .flat_map(|slot| slot.boss_choices.iter())
        .map(AsRef::as_ref)
        .next()
        .or_else(|| instance.boss_choices().next())
        .ok_or(SwarmSeededRunError::MissingBossChoice(state.current_node()))?;
    let program = instance.compile_boss_selection(layer, boss)?;
    let source = state.current_node();
    apply_and_record(
        instance,
        state,
        encounter_rng,
        request,
        program,
        SwarmSeededStepKind::BossSelection(layer),
        SwarmSeededRunAction::BossSelection {
            source_node: source,
            plane: layer,
            boss: boss.into(),
        },
        steps,
        replay,
    )
}

fn preview_encounter(
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

// This boundary keeps the accepted Activity transcript and replay trace paired
// with the same explicit state/RNG/program inputs.
#[allow(clippy::too_many_arguments)]
fn apply_and_record(
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
fn step(
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

fn is_battle_domain(domain: Option<&str>) -> bool {
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

const fn is_boss(role: EncounterRole) -> bool {
    !matches!(role, EncounterRole::Combat | EncounterRole::Elite)
}

fn validate_boundary(
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
