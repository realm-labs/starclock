//! Current Standard Universe replay transport and first-divergence boundary.
//!
//! The battle executor and Activity command semantics remain shared. Every
//! nested battle records explicit component, assembly, combat-input,
//! handoff and result identities. Only bytes produced by the current tree are
//! accepted by public callers.

use std::sync::Arc;

use starclock_activity::BattleResultIdentity;
use starclock_combat::catalog::CombatCatalog;
use starclock_replay::{
    activity::{
        decode_nested_battle_end_payload, decode_nested_battle_start_payload,
        encode_nested_battle_end_payload, encode_nested_battle_start_payload,
        v3::{
            NestedBattleEndV3, NestedBattleIdentityDivergence, NestedBattleStartV3,
            NestedBattleV3PayloadError, decode_nested_battle_end_v3, decode_nested_battle_start_v3,
            encode_nested_battle_end_v3, encode_nested_battle_start_v3,
        },
    },
    component::ConfigurationComponentSet,
    current::{
        DecodedReplay, ReplayCompatibility, ReplayError, ReplayHeader, decode_replay, encode_replay,
    },
    format_v2::{ReplayHeaderV2, decode_replay_v2, encode_replay_v2},
    nested_battle::encode_nested_battle_state_payload,
    record::{RecordKind, RecordRef, ReplayFormatError},
};

pub type RecordedStandardUniverseRun = RecordedStandardUniverseRunV2;
pub type StandardUniverseReplayReport = StandardUniverseReplayReportV2;

use crate::{
    baseline_runner::{
        DynamicNestedBattleExecutor, NestedBattleExecutionError, StandardUniverseBaselinePolicy,
        StandardUniverseBaselineRunner,
    },
    dynamic_battle_assembler::{
        StandardUniverseBattleAssembler, StandardUniverseDynamicBattleStart,
    },
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
    runtime::StandardUniverseActivity,
    universe_replay::{
        StandardUniverseReplayAction, StandardUniverseReplayError, StandardUniverseTraceEntry,
        recorded_from_report,
    },
    universe_replay_v2::{
        RecordedStandardUniverseRunV2, StandardUniverseReplayReportV2,
        StandardUniverseReplayV2Error, encode_current_trace_parts_for_core,
        standard_universe_header_v2, verify_standard_universe_replay_v2,
        verify_standard_universe_replay_v2_dynamic,
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
) -> Result<RecordedStandardUniverseRun, CurrentReplayError> {
    let first_report = executor.reports().len();
    let mut capture = CapturingDynamicExecutor {
        inner: executor,
        results: Vec::new(),
    };
    let report = StandardUniverseBaselineRunner::default()
        .run_to_terminal_dynamic(activity, policy, assembler, &mut capture)
        .map_err(|error| {
            CurrentReplayError::Core(StandardUniverseReplayV2Error::Legacy(
                StandardUniverseReplayError::Runner(error),
            ))
        })?;
    let recorded = recorded_from_report(report, policy, capture.results)
        .map_err(|error| CurrentReplayError::Core(StandardUniverseReplayV2Error::Legacy(error)))?;
    let battles = executor.reports()[first_report..]
        .to_vec()
        .into_boxed_slice();
    let expected = recorded
        .trace()
        .iter()
        .filter(|entry| matches!(entry.action(), StandardUniverseReplayAction::Battle { .. }))
        .count();
    if battles.len() != expected {
        return Err(CurrentReplayError::Core(
            StandardUniverseReplayV2Error::CapturedBattleMismatch,
        ));
    }
    Ok(RecordedStandardUniverseRunV2::new(
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
    compatibility: ReplayCompatibility,
    components: ConfigurationComponentSet,
    master_seed: u64,
    activity: &StandardUniverseActivity,
    profile_id: &str,
) -> Result<ReplayHeader, CurrentReplayError> {
    standard_universe_header_v2(compatibility, components, master_seed, activity, profile_id)
        .map_err(CurrentReplayError::Core)
}

pub fn encode_standard_universe_replay(
    header_template: &ReplayHeader,
    recorded: &RecordedStandardUniverseRun,
) -> Result<Vec<u8>, CurrentReplayError> {
    encode_standard_universe_replay_parts(header_template, recorded.trace(), recorded.battles())
}

pub fn encode_standard_universe_replay_parts(
    header_template: &ReplayHeader,
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<Vec<u8>, CurrentReplayError> {
    let core_encoded = encode_current_trace_parts_for_core(header_template, trace, battles)
        .map_err(CurrentReplayError::Core)?;
    let decoded = decode_replay_v2(&core_encoded).map_err(CurrentReplayError::Envelope)?;
    let mut payloads = Vec::with_capacity(decoded.records().len());
    let mut open_identity = None;
    let mut battle_steps = battles.iter().flat_map(|battle| battle.trace().iter());
    for record in decoded.records() {
        let payload = match record.kind() {
            RecordKind::NestedBattleStart => {
                let identity = decode_nested_battle_start_payload(record.payload())
                    .map_err(CurrentReplayError::ActivityPayload)?;
                if open_identity.replace(identity).is_some() {
                    return Err(CurrentReplayError::RecordLayout);
                }
                encode_nested_battle_start_v3(&NestedBattleStartV3::new(
                    header_template.components().root(),
                    starclock_combat::COMBAT_INPUT_CODEC_REVISION,
                    identity,
                )?)?
            }
            RecordKind::NestedBattleEnd => {
                let identity = open_identity
                    .take()
                    .ok_or(CurrentReplayError::RecordLayout)?;
                let digest = decode_nested_battle_end_payload(record.payload())
                    .map_err(CurrentReplayError::ActivityPayload)?;
                encode_nested_battle_end_v3(NestedBattleEndV3::new(identity, digest))
            }
            RecordKind::ExpectedBattleState => {
                let step = battle_steps
                    .next()
                    .ok_or(CurrentReplayError::RecordLayout)?;
                encode_nested_battle_state_payload(step.state_hash(), step.events())
                    .map_err(CurrentReplayError::NestedPayload)?
            }
            _ => record.payload().to_vec(),
        };
        payloads.push((record.kind(), payload));
    }
    if open_identity.is_some() {
        return Err(CurrentReplayError::RecordLayout);
    }
    if battle_steps.next().is_some() {
        return Err(CurrentReplayError::RecordLayout);
    }
    encode_current_from_payloads(header_template, &payloads)
}

#[allow(clippy::too_many_arguments)]
pub fn verify_standard_universe_replay(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    catalog: Arc<CombatCatalog>,
    actual_components: &ConfigurationComponentSet,
    actual_compatibility: &ReplayCompatibility,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, CurrentReplayError> {
    let replay = decode_replay(bytes).map_err(CurrentReplayError::Envelope)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(|_| CurrentReplayError::divergence(ReplayDivergenceKind::Component))?;
    let prepared = prepare_core_verification(&replay)?;
    let verification = verify_standard_universe_replay_v2(
        &prepared.bytes,
        activity,
        catalog,
        actual_components,
        actual_compatibility,
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
    actual_compatibility: &ReplayCompatibility,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, CurrentReplayError> {
    let replay = decode_replay(bytes).map_err(CurrentReplayError::Envelope)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(|_| CurrentReplayError::divergence(ReplayDivergenceKind::Component))?;
    let prepared = prepare_core_verification(&replay)?;
    let verification = verify_standard_universe_replay_v2_dynamic(
        &prepared.bytes,
        activity,
        assembler,
        actual_components,
        actual_compatibility,
        expected_profile_id,
    );
    map_core_verification(&prepared, verification)
}

fn map_core_verification(
    prepared: &PreparedVerification,
    verification: Result<StandardUniverseReplayReportV2, StandardUniverseReplayV2Error>,
) -> Result<StandardUniverseReplayReportV2, CurrentReplayError> {
    match verification {
        Err(error) => {
            let mapped = map_verification_error(error);
            if prepared.codec_revision_mismatch
                && mapped.first_divergence() != Some(ReplayDivergenceKind::Assembly)
            {
                Err(CurrentReplayError::divergence(
                    ReplayDivergenceKind::CombatInput,
                ))
            } else {
                Err(mapped)
            }
        }
        Ok(_) if prepared.codec_revision_mismatch => Err(CurrentReplayError::divergence(
            ReplayDivergenceKind::CombatInput,
        )),
        Ok(_) if prepared.result_identity_mismatch => {
            Err(CurrentReplayError::divergence(ReplayDivergenceKind::Result))
        }
        Ok(report) => Ok(report),
    }
}

struct PreparedVerification {
    bytes: Vec<u8>,
    codec_revision_mismatch: bool,
    result_identity_mismatch: bool,
}

fn prepare_core_verification(
    replay: &DecodedReplay<'_>,
) -> Result<PreparedVerification, CurrentReplayError> {
    let mut payloads = Vec::with_capacity(replay.records().len());
    let mut open_identity: Option<BattleResultIdentity> = None;
    let mut codec_revision_mismatch = false;
    let mut result_identity_mismatch = false;
    for record in replay.records() {
        let payload = match record.kind() {
            RecordKind::NestedBattleStart => {
                let start = decode_nested_battle_start_v3(record.payload())?;
                if start.component_root() != replay.header().components().root() {
                    return Err(CurrentReplayError::divergence(
                        ReplayDivergenceKind::Component,
                    ));
                }
                if start.combat_input_codec_revision()
                    != starclock_combat::COMBAT_INPUT_CODEC_REVISION
                {
                    codec_revision_mismatch = true;
                }
                if open_identity.replace(start.handoff_identity()).is_some() {
                    return Err(CurrentReplayError::RecordLayout);
                }
                encode_nested_battle_start_payload(start.handoff_identity())
            }
            RecordKind::NestedBattleEnd => {
                let end = decode_nested_battle_end_v3(record.payload())?;
                let start = open_identity
                    .take()
                    .ok_or(CurrentReplayError::RecordLayout)?;
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
        return Err(CurrentReplayError::RecordLayout);
    }
    Ok(PreparedVerification {
        bytes: encode_core_verification_payloads(replay.header(), &payloads)?,
        codec_revision_mismatch,
        result_identity_mismatch,
    })
}

fn encode_current_from_payloads(
    template: &ReplayHeader,
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<u8>, CurrentReplayError> {
    let header = header_with_count(template, payloads.len())?;
    let records = record_refs(payloads)?;
    encode_replay(&header, &records, Vec::new()).map_err(CurrentReplayError::Envelope)
}

fn encode_core_verification_payloads(
    template: &ReplayHeader,
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<u8>, CurrentReplayError> {
    let header = header_with_count(template, payloads.len())?;
    let records = record_refs(payloads)?;
    encode_replay_v2(&header, &records, Vec::new()).map_err(CurrentReplayError::Envelope)
}

fn header_with_count(
    template: &ReplayHeader,
    count: usize,
) -> Result<ReplayHeaderV2, CurrentReplayError> {
    ReplayHeaderV2::new(
        template.compatibility().clone(),
        template.components().clone(),
        template.master_seed(),
        template.entry().clone(),
        u32::try_from(count).map_err(|_| CurrentReplayError::RecordLayout)?,
    )
    .map_err(CurrentReplayError::Envelope)
}

fn record_refs(
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<RecordRef<'_>>, CurrentReplayError> {
    payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| {
            RecordRef::new(*kind, index as u64, payload).map_err(CurrentReplayError::Format)
        })
        .collect()
}

fn map_verification_error(error: StandardUniverseReplayV2Error) -> CurrentReplayError {
    use StandardUniverseReplayV2Error as V2;
    let kind = match &error {
        V2::ComponentDivergence(_) => Some(ReplayDivergenceKind::Component),
        V2::NestedStartDivergence {
            expected, actual, ..
        } if expected.assembly_digest() != actual.assembly_digest() => {
            Some(ReplayDivergenceKind::Assembly)
        }
        V2::NestedStartDivergence {
            expected, actual, ..
        } if expected.combat_input_digest() != actual.combat_input_digest() => {
            Some(ReplayDivergenceKind::CombatInput)
        }
        V2::DecisionDivergence { .. }
        | V2::ActivityCommandRejected { .. }
        | V2::NestedStartDivergence { .. }
        | V2::ControllerDivergence { .. }
        | V2::BattleCommandRejected { .. } => Some(ReplayDivergenceKind::Command),
        V2::BattleEventDivergence { .. } => Some(ReplayDivergenceKind::Event),
        V2::BattleStateDivergence { .. } => Some(ReplayDivergenceKind::State),
        V2::NestedBattleIncomplete { .. } | V2::NestedResultDivergence { .. } => {
            Some(ReplayDivergenceKind::Result)
        }
        V2::ActivityStateDivergence { .. } | V2::IncompleteActivity => {
            Some(ReplayDivergenceKind::Activity)
        }
        _ => None,
    };
    kind.map_or(
        CurrentReplayError::Core(error),
        CurrentReplayError::divergence,
    )
}

#[derive(Debug)]
pub enum CurrentReplayError {
    Envelope(ReplayError),
    Format(ReplayFormatError),
    Payload(NestedBattleV3PayloadError),
    NestedPayload(starclock_replay::nested_battle::NestedBattlePayloadError),
    ActivityPayload(starclock_replay::activity::ActivityCommandPayloadError),
    Core(StandardUniverseReplayV2Error),
    RecordLayout,
    FirstDivergence { kind: ReplayDivergenceKind },
}

impl CurrentReplayError {
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

impl From<NestedBattleV3PayloadError> for CurrentReplayError {
    fn from(value: NestedBattleV3PayloadError) -> Self {
        Self::Payload(value)
    }
}

impl From<NestedBattleIdentityDivergence> for CurrentReplayError {
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
