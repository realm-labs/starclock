//! Deterministic complete-run execution for the frozen Goal 20 matrix.

use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionIdentity, ActivityInstanceId,
    ActivityMasterSeed, ActivityProgramDefinition, ActivityRngContext, ActivityRngStreams,
    ActivityStateHash, ActivityTerminalOutcome, ActivityTransactionOutcome,
    ActivityTransactionState, AttemptId, BattleResultDigest, BattleSequence, NodeId,
};

use crate::{battle_materialization::UniverseBattleRoster, digest::Encoder};

use super::{
    SwarmDisasterRuntimeInstance,
    encounter_runtime::{EncounterRole, EncounterSelection},
};

pub(super) const SWARM_DISASTER_SEEDED_RUN_REVISION: &str = "swarm-disaster-seeded-run-v1";
const MAXIMUM_STEPS: usize = 256;
const PLANE_ONE_DECAY: &str = "swarm-disaster.boss-decay.1";
const PLANE_TWO_DECAY: &str = "swarm-disaster.boss-decay.25";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
enum SwarmSeededStepKind {
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
struct SwarmSeededRunStep {
    kind: SwarmSeededStepKind,
    source_node: NodeId,
    state_hash: ActivityStateHash,
    result_digest: Option<BattleResultDigest>,
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

pub(super) enum SwarmSeededRunError {
    Catalog(crate::error::UniverseCatalogLoadError),
    ProgramRejected,
    MissingRoute(NodeId),
    MissingBossChoice(NodeId),
    BattleNotWon(EncounterRole),
    UnexpectedTerminal(ActivityTerminalOutcome),
    Incomplete,
    BoundaryNotObserved(SwarmSeededBoundary),
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
    pub(super) fn execute_seeded_run(
        &self,
        request: SwarmSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<SwarmSeededRunReport, SwarmSeededRunError> {
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
        if self.countdown(&state)? != 20 {
            return Err(SwarmSeededRunError::BoundaryNotObserved(
                SwarmSeededBoundary::InitialCountdown,
            ));
        }
        let profile = self.compile_profile_entry_rule(&state)?;
        apply_and_record(
            self,
            &mut state,
            &rng,
            request,
            profile,
            SwarmSeededStepKind::ProfileEntry,
            &mut steps,
        )?;
        let audience = self.compile_audience_initialization(&state)?;
        apply_and_record(
            self,
            &mut state,
            &rng,
            request,
            audience,
            SwarmSeededStepKind::AudienceInitialization,
            &mut steps,
        )?;
        let trail = self.compile_trail_run_start(&state)?;
        apply_and_record(
            self,
            &mut state,
            &rng,
            request,
            trail,
            SwarmSeededStepKind::TrailRunStart,
            &mut steps,
        )?;
        configure_boundary(self, &mut state, &rng, request, &mut steps)?;
        create_plane(self, &mut state, &mut rng, request, 0, &mut steps)?;

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
                let role = preview_encounter(self, &state, &mut rng)?.role;
                if is_boss(role) {
                    prepare_boss(self, &mut state, request, &mut steps, &mut rng, role, plane)?;
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
                let (result, report, _) = self.execute_current_battle(
                    &mut state,
                    &mut rng,
                    expected,
                    request.identity,
                    request.activity_instance,
                    AttemptId::new(1).expect("static seeded Attempt is non-zero"),
                    sequence,
                    roster,
                    false,
                )?;
                if report.outcome() != starclock_activity::BattleOutcome::Won {
                    return Err(SwarmSeededRunError::BattleNotWon(role));
                }
                battle_count = sequence.get();
                steps.push(step(
                    self,
                    &state,
                    &rng,
                    request,
                    SwarmSeededStepKind::Battle(role),
                    node,
                    Some(result.actual_digest()),
                ));
                if is_boss(role) && state.terminal().is_none() {
                    let after_transition = (self.countdown(&state)?, self.disarray_level(&state)?);
                    cross_plane_countdown_carried |= before_transition == after_transition;
                    plane = plane
                        .checked_add(1)
                        .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
                    if plane >= plane_starts.len() || state.current_node() != plane_starts[plane] {
                        return Err(SwarmSeededRunError::Incomplete);
                    }
                    create_plane(self, &mut state, &mut rng, request, plane, &mut steps)?;
                }
                maximum_disarray_level = maximum_disarray_level.max(self.disarray_level(&state)?);
                continue;
            }

            if node == plane_ends[plane] {
                return Err(SwarmSeededRunError::Incomplete);
            }
            let target = longest_legal_route(self, &state, node, plane_ends[plane])?;
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
                    &mut steps,
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
                &mut steps,
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
        Ok(SwarmSeededRunReport {
            terminal,
            final_state_hash,
            transcript_digest,
            battle_count,
            step_count,
            maximum_disarray_level,
            cross_plane_countdown_carried,
            steps: steps.into_boxed_slice(),
        })
    }

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
    apply_and_record(
        instance,
        state,
        rng,
        request,
        program,
        SwarmSeededStepKind::CountdownSetup,
        steps,
    )
}

fn create_plane(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rng: &mut ActivityRngStreams,
    request: SwarmSeededRunRequest,
    plane: usize,
    steps: &mut Vec<SwarmSeededRunStep>,
) -> Result<(), SwarmSeededRunError> {
    let program = instance.compile_plane_creation(plane, rng)?;
    apply_and_record(
        instance,
        state,
        rng,
        request,
        program,
        SwarmSeededStepKind::PlaneCreation(
            u8::try_from(plane + 1).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?,
        ),
        steps,
    )
}

#[allow(clippy::too_many_arguments)]
fn prepare_boss(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    request: SwarmSeededRunRequest,
    steps: &mut Vec<SwarmSeededRunStep>,
    encounter_rng: &mut ActivityRngStreams,
    role: EncounterRole,
    plane: usize,
) -> Result<(), SwarmSeededRunError> {
    let layer = u8::try_from(plane + 1).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
    if layer == 1 {
        let program = instance.compile_boss_decay_selection(state, &[PLANE_ONE_DECAY])?;
        apply_and_record(
            instance,
            state,
            encounter_rng,
            request,
            program,
            SwarmSeededStepKind::BossSelection(layer),
            steps,
        )?;
    } else if layer == 2 {
        let program = instance.compile_boss_decay_selection(state, &[PLANE_TWO_DECAY])?;
        apply_and_record(
            instance,
            state,
            encounter_rng,
            request,
            program,
            SwarmSeededStepKind::BossSelection(layer),
            steps,
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
    apply_and_record(
        instance,
        state,
        encounter_rng,
        request,
        program,
        SwarmSeededStepKind::BossSelection(layer),
        steps,
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

fn explicit_face_target(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &mut ActivityRngStreams,
) -> Result<Option<NodeId>, SwarmSeededRunError> {
    let face = instance
        .dice_resolution_face(state)
        .ok_or(SwarmSeededRunError::Incomplete)?;
    if instance.dice_face_target_contract(face) != Some("caller-explicit-eligible-node") {
        return Ok(None);
    }
    if instance
        .compile_dice_face_activation(state, None, rng)
        .is_ok()
    {
        return Ok(None);
    }
    for node in instance.graph_definition().nodes() {
        if instance
            .compile_dice_face_activation(state, Some(node.id()), rng)
            .is_ok()
        {
            return Ok(Some(node.id()));
        }
    }
    Err(SwarmSeededRunError::Incomplete)
}

fn movement_program(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    target: NodeId,
) -> Result<ActivityProgramDefinition, SwarmSeededRunError> {
    let edge = instance
        .graph_definition()
        .outgoing(state.current_node())
        .find(|edge| edge.to() == target)
        .ok_or(SwarmSeededRunError::MissingRoute(state.current_node()))?;
    let mut operations = instance
        .compile_countdown_move(state, &[])?
        .operations()
        .to_vec();
    operations.push(starclock_activity::ActivityOperation::Traverse(edge.id()));
    ActivityProgramDefinition::new(
        starclock_activity::ActivityProgramId::new(0x7f9a_0001)
            .expect("static seeded movement program ID is non-zero"),
        operations,
    )
    .map_err(|_| SwarmSeededRunError::ProgramRejected)
}

fn longest_legal_route(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    end: NodeId,
) -> Result<NodeId, SwarmSeededRunError> {
    let mut candidates = instance
        .legal_routes(state, source)
        .iter()
        .filter_map(|id| {
            instance
                .graph_definition()
                .edges()
                .iter()
                .find(|edge| edge.id() == *id)
                .and_then(|edge| {
                    longest_distance(instance, state, edge.to(), end)
                        .map(|distance| (distance, *id, edge.to()))
                })
        })
        .collect::<Vec<_>>();
    candidates.sort_unstable_by_key(|(distance, id, _)| (core::cmp::Reverse(*distance), *id));
    candidates
        .first()
        .map(|(_, _, target)| *target)
        .ok_or(SwarmSeededRunError::MissingRoute(source))
}

fn longest_distance(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    end: NodeId,
) -> Option<u32> {
    if source == end {
        return Some(0);
    }
    instance
        .legal_routes(state, source)
        .iter()
        .filter_map(|id| {
            instance
                .graph_definition()
                .edges()
                .iter()
                .find(|edge| edge.id() == *id)
                .and_then(|edge| longest_distance(instance, state, edge.to(), end))
        })
        .max()
        .and_then(|distance| distance.checked_add(1))
}

fn apply_and_record(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rng: &ActivityRngStreams,
    request: SwarmSeededRunRequest,
    program: ActivityProgramDefinition,
    kind: SwarmSeededStepKind,
    steps: &mut Vec<SwarmSeededRunStep>,
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
    steps.push(step(instance, state, rng, request, kind, source, None));
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

fn transcript_digest(
    seed: u64,
    terminal: ActivityTerminalOutcome,
    final_state_hash: ActivityStateHash,
    battle_count: u32,
    maximum_disarray_level: i64,
    cross_plane_countdown_carried: bool,
    steps: &[SwarmSeededRunStep],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.seeded-run.v1");
    encoder.text(SWARM_DISASTER_SEEDED_RUN_REVISION);
    encoder.u64(seed);
    encoder.u8(terminal_code(terminal));
    encoder.digest(final_state_hash.bytes());
    encoder.u32(battle_count);
    encoder.i64(maximum_disarray_level);
    encoder.bool(cross_plane_countdown_carried);
    encoder.u32(u32::try_from(steps.len()).expect("seeded steps are bounded"));
    for step in steps {
        encode_step_kind(&mut encoder, step.kind);
        encoder.u32(step.source_node.get());
        encoder.digest(step.state_hash.bytes());
        encoder.optional_digest(step.result_digest.map(BattleResultDigest::bytes));
    }
    encoder.finish()
}

fn encode_step_kind(encoder: &mut Encoder, kind: SwarmSeededStepKind) {
    match kind {
        SwarmSeededStepKind::ProfileEntry => encoder.u8(0),
        SwarmSeededStepKind::AudienceInitialization => encoder.u8(1),
        SwarmSeededStepKind::TrailRunStart => encoder.u8(2),
        SwarmSeededStepKind::CountdownSetup => encoder.u8(3),
        SwarmSeededStepKind::PlaneCreation(plane) => {
            encoder.u8(4);
            encoder.u8(plane);
        }
        SwarmSeededStepKind::DiceRoll => encoder.u8(5),
        SwarmSeededStepKind::Traverse => encoder.u8(6),
        SwarmSeededStepKind::BossSelection(plane) => {
            encoder.u8(7);
            encoder.u8(plane);
        }
        SwarmSeededStepKind::Battle(role) => {
            encoder.u8(8);
            encoder.u8(role_code(role));
        }
    }
}

const fn role_code(role: EncounterRole) -> u8 {
    match role {
        EncounterRole::Combat => 0,
        EncounterRole::Elite => 1,
        EncounterRole::FirstPlaneBoss => 2,
        EncounterRole::SecondPlaneBoss => 3,
        EncounterRole::FinalBoss => 4,
    }
}

const fn terminal_code(terminal: ActivityTerminalOutcome) -> u8 {
    match terminal {
        ActivityTerminalOutcome::Completed => 0,
        ActivityTerminalOutcome::Failed => 1,
        ActivityTerminalOutcome::Abandoned => 2,
        ActivityTerminalOutcome::Faulted => 3,
    }
}
