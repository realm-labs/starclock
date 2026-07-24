//! Replay-v3 Standard Universe transport and first-divergence boundary.
//!
//! The battle executor and Activity command semantics remain shared with the
//! released v2 verifier. V3 changes the envelope and makes every nested
//! battle's component, assembly, combat-input, handoff and result identities
//! explicit. The v2 decoder/verifier remains an independent public path.

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
    format_v2::{ReplayCompatibilityV2, ReplayHeaderV2, decode_replay_v2, encode_replay_v2},
    format_v3::{ReplayHeaderV3, ReplayV3Error, decode_replay_v3, encode_replay_v3},
    record::{RecordKind, RecordRef, ReplayFormatError},
};

use crate::{
    baseline_runner::StandardUniverseBaselinePolicy,
    nested_battle_executor::{NestedBattleExecutionReport, UniverseNestedBattleExecutor},
    runtime::StandardUniverseActivity,
    universe_replay::StandardUniverseTraceEntry,
    universe_replay_v2::{
        RecordedStandardUniverseRunV2, StandardUniverseReplayReportV2,
        StandardUniverseReplayV2Error, encode_standard_universe_trace_parts_v2,
        record_baseline_run_v2, standard_universe_header_v2, verify_standard_universe_replay_v2,
    },
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReplayV3DivergenceKind {
    Component,
    Assembly,
    CombatInput,
    Command,
    Event,
    State,
    Result,
    Activity,
}

pub fn record_baseline_run_v3(
    activity: &mut StandardUniverseActivity,
    policy: &StandardUniverseBaselinePolicy,
    executor: &mut UniverseNestedBattleExecutor,
) -> Result<RecordedStandardUniverseRunV2, StandardUniverseReplayV3Error> {
    record_baseline_run_v2(activity, policy, executor)
        .map_err(StandardUniverseReplayV3Error::Historical)
}

pub fn standard_universe_header_v3(
    compatibility: ReplayCompatibilityV2,
    components: ConfigurationComponentSet,
    master_seed: u64,
    activity: &StandardUniverseActivity,
    profile_id: &str,
) -> Result<ReplayHeaderV3, StandardUniverseReplayV3Error> {
    standard_universe_header_v2(compatibility, components, master_seed, activity, profile_id)
        .map_err(StandardUniverseReplayV3Error::Historical)
}

pub fn encode_standard_universe_trace_v3(
    header_template: &ReplayHeaderV3,
    recorded: &RecordedStandardUniverseRunV2,
) -> Result<Vec<u8>, StandardUniverseReplayV3Error> {
    encode_standard_universe_trace_parts_v3(header_template, recorded.trace(), recorded.battles())
}

pub fn encode_standard_universe_trace_parts_v3(
    header_template: &ReplayHeaderV3,
    trace: &[StandardUniverseTraceEntry],
    battles: &[NestedBattleExecutionReport],
) -> Result<Vec<u8>, StandardUniverseReplayV3Error> {
    let historical = encode_standard_universe_trace_parts_v2(header_template, trace, battles)
        .map_err(StandardUniverseReplayV3Error::Historical)?;
    let decoded = decode_replay_v2(&historical).map_err(StandardUniverseReplayV3Error::Envelope)?;
    let mut payloads = Vec::with_capacity(decoded.records().len());
    let mut open_identity = None;
    for record in decoded.records() {
        let payload = match record.kind() {
            RecordKind::NestedBattleStart => {
                let identity = decode_nested_battle_start_payload(record.payload())
                    .map_err(StandardUniverseReplayV3Error::HistoricalPayload)?;
                if open_identity.replace(identity).is_some() {
                    return Err(StandardUniverseReplayV3Error::RecordLayout);
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
                    .ok_or(StandardUniverseReplayV3Error::RecordLayout)?;
                let digest = decode_nested_battle_end_payload(record.payload())
                    .map_err(StandardUniverseReplayV3Error::HistoricalPayload)?;
                encode_nested_battle_end_v3(NestedBattleEndV3::new(identity, digest))
            }
            _ => record.payload().to_vec(),
        };
        payloads.push((record.kind(), payload));
    }
    if open_identity.is_some() {
        return Err(StandardUniverseReplayV3Error::RecordLayout);
    }
    encode_v3_from_payloads(header_template, &payloads)
}

#[allow(clippy::too_many_arguments)]
pub fn verify_standard_universe_replay_v3(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    catalog: Arc<CombatCatalog>,
    actual_components: &ConfigurationComponentSet,
    actual_compatibility: &ReplayCompatibilityV2,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReportV2, StandardUniverseReplayV3Error> {
    let replay = decode_replay_v3(bytes).map_err(StandardUniverseReplayV3Error::Envelope)?;
    replay
        .header()
        .components()
        .verify_exact(actual_components)
        .map_err(|_| {
            StandardUniverseReplayV3Error::divergence(ReplayV3DivergenceKind::Component)
        })?;
    let translated = translate_v3_to_v2(&replay)?;
    let verification = verify_standard_universe_replay_v2(
        &translated.bytes,
        activity,
        catalog,
        actual_components,
        actual_compatibility,
        expected_profile_id,
    );
    match verification {
        Err(error) => {
            let mapped = map_verification_error(error);
            if translated.codec_revision_mismatch
                && mapped.first_divergence() != Some(ReplayV3DivergenceKind::Assembly)
            {
                Err(StandardUniverseReplayV3Error::divergence(
                    ReplayV3DivergenceKind::CombatInput,
                ))
            } else {
                Err(mapped)
            }
        }
        Ok(_) if translated.codec_revision_mismatch => Err(
            StandardUniverseReplayV3Error::divergence(ReplayV3DivergenceKind::CombatInput),
        ),
        Ok(_) if translated.result_identity_mismatch => Err(
            StandardUniverseReplayV3Error::divergence(ReplayV3DivergenceKind::Result),
        ),
        Ok(report) => Ok(report),
    }
}

struct TranslatedReplayV3 {
    bytes: Vec<u8>,
    codec_revision_mismatch: bool,
    result_identity_mismatch: bool,
}

fn translate_v3_to_v2(
    replay: &starclock_replay::format_v3::DecodedReplayV3<'_>,
) -> Result<TranslatedReplayV3, StandardUniverseReplayV3Error> {
    let mut payloads = Vec::with_capacity(replay.records().len());
    let mut open_identity: Option<BattleResultIdentity> = None;
    let mut codec_revision_mismatch = false;
    let mut result_identity_mismatch = false;
    for record in replay.records() {
        let payload = match record.kind() {
            RecordKind::NestedBattleStart => {
                let start = decode_nested_battle_start_v3(record.payload())?;
                if start.component_root() != replay.header().components().root() {
                    return Err(StandardUniverseReplayV3Error::divergence(
                        ReplayV3DivergenceKind::Component,
                    ));
                }
                if start.combat_input_codec_revision()
                    != starclock_combat::COMBAT_INPUT_CODEC_REVISION
                {
                    codec_revision_mismatch = true;
                }
                if open_identity.replace(start.handoff_identity()).is_some() {
                    return Err(StandardUniverseReplayV3Error::RecordLayout);
                }
                encode_nested_battle_start_payload(start.handoff_identity())
            }
            RecordKind::NestedBattleEnd => {
                let end = decode_nested_battle_end_v3(record.payload())?;
                let start = open_identity
                    .take()
                    .ok_or(StandardUniverseReplayV3Error::RecordLayout)?;
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
        return Err(StandardUniverseReplayV3Error::RecordLayout);
    }
    Ok(TranslatedReplayV3 {
        bytes: encode_v2_from_payloads(replay.header(), &payloads)?,
        codec_revision_mismatch,
        result_identity_mismatch,
    })
}

fn encode_v3_from_payloads(
    template: &ReplayHeaderV3,
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<u8>, StandardUniverseReplayV3Error> {
    let header = header_with_count(template, payloads.len())?;
    let records = record_refs(payloads)?;
    encode_replay_v3(&header, &records, Vec::new()).map_err(StandardUniverseReplayV3Error::Envelope)
}

fn encode_v2_from_payloads(
    template: &ReplayHeaderV3,
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<u8>, StandardUniverseReplayV3Error> {
    let header = header_with_count(template, payloads.len())?;
    let records = record_refs(payloads)?;
    encode_replay_v2(&header, &records, Vec::new()).map_err(StandardUniverseReplayV3Error::Envelope)
}

fn header_with_count(
    template: &ReplayHeaderV3,
    count: usize,
) -> Result<ReplayHeaderV2, StandardUniverseReplayV3Error> {
    ReplayHeaderV2::new(
        template.compatibility().clone(),
        template.components().clone(),
        template.master_seed(),
        template.entry().clone(),
        u32::try_from(count).map_err(|_| StandardUniverseReplayV3Error::RecordLayout)?,
    )
    .map_err(StandardUniverseReplayV3Error::Envelope)
}

fn record_refs(
    payloads: &[(RecordKind, Vec<u8>)],
) -> Result<Vec<RecordRef<'_>>, StandardUniverseReplayV3Error> {
    payloads
        .iter()
        .enumerate()
        .map(|(index, (kind, payload))| {
            RecordRef::new(*kind, index as u64, payload)
                .map_err(StandardUniverseReplayV3Error::Format)
        })
        .collect()
}

fn map_verification_error(error: StandardUniverseReplayV2Error) -> StandardUniverseReplayV3Error {
    use StandardUniverseReplayV2Error as V2;
    let kind = match &error {
        V2::ComponentDivergence(_) => Some(ReplayV3DivergenceKind::Component),
        V2::NestedStartDivergence {
            expected, actual, ..
        } if expected.assembly_digest() != actual.assembly_digest() => {
            Some(ReplayV3DivergenceKind::Assembly)
        }
        V2::NestedStartDivergence {
            expected, actual, ..
        } if expected.combat_input_digest() != actual.combat_input_digest() => {
            Some(ReplayV3DivergenceKind::CombatInput)
        }
        V2::DecisionDivergence { .. }
        | V2::ActivityCommandRejected { .. }
        | V2::NestedStartDivergence { .. }
        | V2::ControllerDivergence { .. }
        | V2::BattleCommandRejected { .. } => Some(ReplayV3DivergenceKind::Command),
        V2::BattleEventDivergence { .. } => Some(ReplayV3DivergenceKind::Event),
        V2::BattleStateDivergence { .. } => Some(ReplayV3DivergenceKind::State),
        V2::NestedBattleIncomplete { .. } | V2::NestedResultDivergence { .. } => {
            Some(ReplayV3DivergenceKind::Result)
        }
        V2::ActivityStateDivergence { .. } | V2::IncompleteActivity => {
            Some(ReplayV3DivergenceKind::Activity)
        }
        _ => None,
    };
    kind.map_or(
        StandardUniverseReplayV3Error::Historical(error),
        StandardUniverseReplayV3Error::divergence,
    )
}

#[derive(Debug)]
pub enum StandardUniverseReplayV3Error {
    Envelope(ReplayV3Error),
    Format(ReplayFormatError),
    Payload(NestedBattleV3PayloadError),
    HistoricalPayload(starclock_replay::activity::ActivityCommandPayloadError),
    Historical(StandardUniverseReplayV2Error),
    RecordLayout,
    FirstDivergence { kind: ReplayV3DivergenceKind },
}

impl StandardUniverseReplayV3Error {
    fn divergence(kind: ReplayV3DivergenceKind) -> Self {
        Self::FirstDivergence { kind }
    }

    #[must_use]
    pub const fn first_divergence(&self) -> Option<ReplayV3DivergenceKind> {
        match self {
            Self::FirstDivergence { kind } => Some(*kind),
            _ => None,
        }
    }
}

impl From<NestedBattleV3PayloadError> for StandardUniverseReplayV3Error {
    fn from(value: NestedBattleV3PayloadError) -> Self {
        Self::Payload(value)
    }
}

impl From<NestedBattleIdentityDivergence> for StandardUniverseReplayV3Error {
    fn from(value: NestedBattleIdentityDivergence) -> Self {
        let kind = match value {
            NestedBattleIdentityDivergence::Component => ReplayV3DivergenceKind::Component,
            NestedBattleIdentityDivergence::Assembly => ReplayV3DivergenceKind::Assembly,
            NestedBattleIdentityDivergence::CombatInput => ReplayV3DivergenceKind::CombatInput,
            NestedBattleIdentityDivergence::Handoff => ReplayV3DivergenceKind::Command,
            NestedBattleIdentityDivergence::Result => ReplayV3DivergenceKind::Result,
        };
        Self::divergence(kind)
    }
}
