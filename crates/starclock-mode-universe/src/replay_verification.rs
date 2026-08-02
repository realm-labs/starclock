//! Standard Universe replay transport and first-divergence boundary.
//!
//! The battle executor and Activity command semantics remain shared. Every
//! nested battle records explicit component, assembly, combat-input,
//! handoff and result identities.

use std::sync::Arc;

use starclock_activity::BattleResultIdentity;
use starclock_combat::catalog::CombatCatalog;
use starclock_replay::{
    activity::{
        decode_nested_battle_end_payload, decode_nested_battle_start_payload,
        encode_nested_battle_end_payload, encode_nested_battle_start_payload,
        nested_identity::{
            NestedBattleEnd, NestedBattleIdentityDivergence, NestedBattleIdentityPayloadError,
            NestedBattleStart, decode_nested_battle_end, decode_nested_battle_start,
            encode_nested_battle_end, encode_nested_battle_start,
        },
    },
    component::ConfigurationComponentSet,
    envelope::{
        DecodedReplay, ReplayEnvironment, ReplayError, ReplayHeader, decode_replay, encode_replay,
    },
    nested_battle::encode_nested_battle_state_payload,
    record::{RecordKind, RecordRef, ReplayFormatError},
};

use crate::{
    baseline_runner::{
        DynamicNestedBattleExecutor, NestedBattleExecutionError, StandardUniverseBaselinePolicy,
        StandardUniverseBaselineRunner,
    },
    dynamic_battle_assembler::{
        StandardUniverseBattleAssembler, StandardUniverseDynamicBattleStart,
    },
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
    replay_execution::{
        RecordedStandardUniverseRun, ReplayExecutionError, StandardUniverseReplayReport,
        encode_trace_for_verification, execute_standard_universe_replay,
        execute_standard_universe_replay_dynamic, standard_universe_header,
    },
    runtime::StandardUniverseActivity,
    universe_replay::{
        StandardUniverseReplayAction, StandardUniverseReplayError as TraceReplayError,
        StandardUniverseTraceEntry, recorded_from_report,
    },
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReplayDivergenceKind {
    Component,
    Assembly,
    CombatInput,
    Command,
    Event,
    State,
    Result,
    Activity,
}

pub fn record_baseline_run(
    activity: &mut StandardUniverseActivity,
    policy: &StandardUniverseBaselinePolicy,
    assembler: &StandardUniverseBattleAssembler,
    executor: &mut UniverseNestedBattleExecutor,
) -> Result<RecordedStandardUniverseRun, ReplayVerificationError> {
    let first_report = executor.reports().len();
    let mut capture = CapturingDynamicExecutor {
        inner: executor,
        results: Vec::new(),
    };
    let report = StandardUniverseBaselineRunner::default()
        .run_to_terminal_dynamic(activity, policy, assembler, &mut capture)
        .map_err(|error| {
            ReplayVerificationError::Core(ReplayExecutionError::Trace(TraceReplayError::Runner(
                error,
            )))
        })?;
    let recorded = recorded_from_report(report, policy, capture.results)
        .map_err(|error| ReplayVerificationError::Core(ReplayExecutionError::Trace(error)))?;
    let battles = executor.reports()[first_report..]
        .to_vec()
        .into_boxed_slice();
    let expected = recorded
        .trace()
        .iter()
        .filter(|entry| matches!(entry.action(), StandardUniverseReplayAction::Battle { .. }))
        .count();
    if battles.len() != expected {
        return Err(ReplayVerificationError::Core(
            ReplayExecutionError::CapturedBattleMismatch,
        ));
    }
    Ok(RecordedStandardUniverseRun::new(
        recorded.report().clone(),
        recorded.trace().to_vec().into_boxed_slice(),
        battles,
    ))
}

struct CapturingDynamicExecutor<'a> {
    inner: &'a mut UniverseNestedBattleExecutor,
    results: Vec<starclock_activity::BattleResult>,
}

impl DynamicNestedBattleExecutor for CapturingDynamicExecutor<'_> {
    fn execute_dynamic(
        &mut self,
        start: &StandardUniverseDynamicBattleStart,
    ) -> Result<starclock_activity::BattleResult, NestedBattleExecutionError> {
        let result = self.inner.execute_dynamic(start)?;
        self.results.push(result.clone());
        Ok(result)
    }
}

pub fn standard_universe_replay_header(
    environment: ReplayEnvironment,
    components: ConfigurationComponentSet,
    master_seed: u64,
    activity: &StandardUniverseActivity,
    profile_id: &str,
) -> Result<ReplayHeader, ReplayVerificationError> {
    standard_universe_header(environment, components, master_seed, activity, profile_id)
        .map_err(ReplayVerificationError::Core)
}

pub fn encode_standard_universe_replay(
    header_template: &ReplayHeader,
    recorded: &RecordedStandardUniverseRun,
) -> Result<Vec<u8>, ReplayVerificationError> {
    encode_standard_universe_replay_parts(header_template, recorded.trace(), recorded.battles())
}

pub fn encode_standard_universe_replay_parts(
    header_template: &ReplayHeader,
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<Vec<u8>, ReplayVerificationError> {
    let core_encoded = encode_trace_for_verification(header_template, trace, battles)
        .map_err(ReplayVerificationError::Core)?;
    let decoded = decode_replay(&core_encoded).map_err(ReplayVerificationError::Envelope)?;
    let mut payloads = Vec::with_capacity(decoded.records().len());
    let mut open_identity = None;
    let mut battle_steps = battles.iter().flat_map(|battle| battle.trace().iter());
    for record in decoded.records() {
        let payload = match record.kind() {
            RecordKind::NestedBattleStart => {
                let identity = decode_nested_battle_start_payload(record.payload())
                    .map_err(ReplayVerificationError::ActivityPayload)?;
                if open_identity.replace(identity).is_some() {
                    return Err(ReplayVerificationError::RecordLayout);
                }
                encode_nested_battle_start(&NestedBattleStart::new(
                    header_template.components().root(),
                    identity,
                ))?
            }
            RecordKind::NestedBattleEnd => {
                let identity = open_identity
                    .take()
                    .ok_or(ReplayVerificationError::RecordLayout)?;
                let digest = decode_nested_battle_end_payload(record.payload())
                    .map_err(ReplayVerificationError::ActivityPayload)?;
                encode_nested_battle_end(NestedBattleEnd::new(identity, digest))
            }
            RecordKind::ExpectedBattleState => {
                let step = battle_steps
                    .next()
                    .ok_or(ReplayVerificationError::RecordLayout)?;
                encode_nested_battle_state_payload(step.state_hash(), step.events())
                    .map_err(ReplayVerificationError::NestedPayload)?
            }
            _ => record.payload().to_vec(),
        };
        payloads.push((record.kind(), payload));
    }
    if open_identity.is_some() {
        return Err(ReplayVerificationError::RecordLayout);
    }
    if battle_steps.next().is_some() {
        return Err(ReplayVerificationError::RecordLayout);
    }
    encode_envelope_from_payloads(header_template, &payloads)
}

#[allow(clippy::too_many_arguments)]
pub fn verify_standard_universe_replay(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    catalog: Arc<CombatCatalog>,
    actual_components: &ConfigurationComponentSet,
    actual_environment: &ReplayEnvironment,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, ReplayVerificationError> {
    let replay = decode_replay(bytes).map_err(ReplayVerificationError::Envelope)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(|_| ReplayVerificationError::divergence(ReplayDivergenceKind::Component))?;
    let prepared = prepare_core_verification(&replay)?;
    let verification = execute_standard_universe_replay(
        &prepared.bytes,
        activity,
        catalog,
        actual_components,
        actual_environment,
        expected_profile_id,
    );
    map_core_verification(&prepared, verification)
}

#[allow(clippy::too_many_arguments)]
pub fn verify_standard_universe_replay_dynamic(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    assembler: &StandardUniverseBattleAssembler,
    actual_components: &ConfigurationComponentSet,
    actual_environment: &ReplayEnvironment,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, ReplayVerificationError> {
    let replay = decode_replay(bytes).map_err(ReplayVerificationError::Envelope)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(|_| ReplayVerificationError::divergence(ReplayDivergenceKind::Component))?;
    let prepared = prepare_core_verification(&replay)?;
    let verification = execute_standard_universe_replay_dynamic(
        &prepared.bytes,
        activity,
        assembler,
        actual_components,
        actual_environment,
        expected_profile_id,
    );
    map_core_verification(&prepared, verification)
}

fn map_core_verification(
    prepared: &PreparedVerification,
    verification: Result<StandardUniverseReplayReport, ReplayExecutionError>,
) -> Result<StandardUniverseReplayReport, ReplayVerificationError> {
    match verification {
        Err(error) => {
            let mapped = map_verification_error(error);
            Err(mapped)
        }
        Ok(_) if prepared.result_identity_mismatch => Err(ReplayVerificationError::divergence(
            ReplayDivergenceKind::Result,
        )),
        Ok(report) => Ok(report),
    }
}

struct PreparedVerification {
    bytes: Vec<u8>,
    result_identity_mismatch: bool,
}

fn prepare_core_verification(
    replay: &DecodedReplay<'_>,
) -> Result<PreparedVerification, ReplayVerificationError> {
    let mut payloads = Vec::with_capacity(replay.records().len());
    let mut open_identity: Option<BattleResultIdentity> = None;
    let mut result_identity_mismatch = false;
    for record in replay.records() {
        let payload = match record.kind() {
            RecordKind::NestedBattleStart => {
                let start = decode_nested_battle_start(record.payload())?;
                if start.component_root() != replay.header().components().root() {
                    return Err(ReplayVerificationError::divergence(
                        ReplayDivergenceKind::Component,
                    ));
                }
                if open_identity.replace(start.handoff_identity()).is_some() {
                    return Err(ReplayVerificationError::RecordLayout);
                }
                encode_nested_battle_start_payload(start.handoff_identity())
            }
            RecordKind::NestedBattleEnd => {
                let end = decode_nested_battle_end(record.payload())?;
                let start = open_identity
                    .take()
                    .ok_or(ReplayVerificationError::RecordLayout)?;
                if end.result_identity() != start {
                    result_identity_mismatch = true;
                }
                encode_nested_battle_end_payload(end.result_digest())
            }
            _ => record.payload().to_vec(),
        };
        payloads.push((record.kind(), payload));
    }
    if open_identity.is_some() {
        return Err(ReplayVerificationError::RecordLayout);
    }
    Ok(PreparedVerification {
        bytes: encode_core_verification_payloads(replay.header(), &payloads)?,
        result_identity_mismatch,
    })
}

fn encode_envelope_from_payloads(
    template: &ReplayHeader,
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<u8>, ReplayVerificationError> {
    let header = header_with_count(template, payloads.len())?;
    let records = record_refs(payloads)?;
    encode_replay(&header, &records, Vec::new()).map_err(ReplayVerificationError::Envelope)
}

fn encode_core_verification_payloads(
    template: &ReplayHeader,
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<u8>, ReplayVerificationError> {
    let header = header_with_count(template, payloads.len())?;
    let records = record_refs(payloads)?;
    encode_replay(&header, &records, Vec::new()).map_err(ReplayVerificationError::Envelope)
}

fn header_with_count(
    template: &ReplayHeader,
    count: usize,
) -> Result<ReplayHeader, ReplayVerificationError> {
    ReplayHeader::new(
        template.environment().clone(),
        template.components().clone(),
        template.master_seed(),
        template.entry().clone(),
        u32::try_from(count).map_err(|_| ReplayVerificationError::RecordLayout)?,
    )
    .map_err(ReplayVerificationError::Envelope)
}

fn record_refs(
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<RecordRef<'_>>, ReplayVerificationError> {
    payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| {
            RecordRef::new(*kind, index as u64, payload).map_err(ReplayVerificationError::Format)
        })
        .collect()
}

fn map_verification_error(error: ReplayExecutionError) -> ReplayVerificationError {
    use ReplayExecutionError as Execution;
    let kind = match &error {
        Execution::ComponentDivergence(_) => Some(ReplayDivergenceKind::Component),
        Execution::NestedStartDivergence {
            expected, actual, ..
        } if expected.assembly_digest() != actual.assembly_digest() => {
            Some(ReplayDivergenceKind::Assembly)
        }
        Execution::NestedStartDivergence {
            expected, actual, ..
        } if expected.combat_input_digest() != actual.combat_input_digest() => {
            Some(ReplayDivergenceKind::CombatInput)
        }
        Execution::DecisionDivergence { .. }
        | Execution::ActivityCommandRejected { .. }
        | Execution::NestedStartDivergence { .. }
        | Execution::ControllerDivergence { .. }
        | Execution::BattleCommandRejected { .. } => Some(ReplayDivergenceKind::Command),
        Execution::BattleEventDivergence { .. } => Some(ReplayDivergenceKind::Event),
        Execution::BattleStateDivergence { .. } => Some(ReplayDivergenceKind::State),
        Execution::NestedBattleIncomplete { .. } | Execution::NestedResultDivergence { .. } => {
            Some(ReplayDivergenceKind::Result)
        }
        Execution::ActivityStateDivergence { .. } | Execution::IncompleteActivity => {
            Some(ReplayDivergenceKind::Activity)
        }
        _ => None,
    };
    kind.map_or(
        ReplayVerificationError::Core(error),
        ReplayVerificationError::divergence,
    )
}

#[derive(Debug)]
pub enum ReplayVerificationError {
    Envelope(ReplayError),
    Format(ReplayFormatError),
    Payload(NestedBattleIdentityPayloadError),
    NestedPayload(starclock_replay::nested_battle::NestedBattlePayloadError),
    ActivityPayload(starclock_replay::activity::ActivityCommandPayloadError),
    Core(ReplayExecutionError),
    RecordLayout,
    FirstDivergence { kind: ReplayDivergenceKind },
}

impl ReplayVerificationError {
    fn divergence(kind: ReplayDivergenceKind) -> Self {
        Self::FirstDivergence { kind }
    }

    #[must_use]
    pub const fn first_divergence(&self) -> Option<ReplayDivergenceKind> {
        match self {
            Self::FirstDivergence { kind } => Some(*kind),
            _ => None,
        }
    }
}

impl From<NestedBattleIdentityPayloadError> for ReplayVerificationError {
    fn from(value: NestedBattleIdentityPayloadError) -> Self {
        Self::Payload(value)
    }
}

impl From<NestedBattleIdentityDivergence> for ReplayVerificationError {
    fn from(value: NestedBattleIdentityDivergence) -> Self {
        let kind = match value {
            NestedBattleIdentityDivergence::Component => ReplayDivergenceKind::Component,
            NestedBattleIdentityDivergence::Assembly => ReplayDivergenceKind::Assembly,
            NestedBattleIdentityDivergence::CombatInput => ReplayDivergenceKind::CombatInput,
            NestedBattleIdentityDivergence::Handoff => ReplayDivergenceKind::Command,
            NestedBattleIdentityDivergence::Result => ReplayDivergenceKind::Result,
        };
        Self::divergence(kind)
    }
}
