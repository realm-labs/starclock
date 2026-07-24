//! Activity command/result payloads, nested boundaries and one-shot verification.

#[path = "activity_identity.rs"]
mod identity;
#[path = "activity_v2.rs"]
pub mod v2;
#[path = "activity_v3.rs"]
pub mod v3;

use core::fmt;

use starclock_activity::{
    Activity, ActivityCommand, ActivityCommandErrorKind, ActivityDecision, ActivityPhase,
    ActivityStateHash, BattleOutcome, BattleResult, BattleResultDigest, BattleResultIdentity,
    EventDigest, MetricValue, ParticipantBattleState, ParticipantId, ProjectedValue,
};
use starclock_combat::{
    BattleFault, BattleStateHash, Energy, FaultBoundary, FaultKind, FaultPolicy, Hp, LifeState,
    PresenceState,
};

use crate::{
    codec::{CodecError, Decoder, Encoder},
    digest::{ConfigBundleDigest, StateDigest},
    format::{DecodedReplay, ReplayEntry, ReplayHeader, decode_replay, encode_replay},
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use identity::{decode_identity, decode_identity_legacy, encode_identity};

pub const ACTIVITY_COMMAND_PAYLOAD_VERSION: u16 = 2;
pub const NESTED_BATTLE_PAYLOAD_VERSION: u16 = 2;
pub const CONTROLLER_DIAGNOSTIC_PAYLOAD_VERSION: u16 = 1;
pub const BATTLE_RESULT_PAYLOAD_VERSION: u16 = 2;
pub const MAX_CONTROLLER_OPTIONS: u32 = 4_096;

/// Generic controller family retained only as replay diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ControllerDecisionKind {
    Activity = 0,
    BattlePlayer = 1,
    EnemyAuthored = 2,
}

/// Total score for one canonically ordered offered option.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControllerOptionScore {
    ordinal: u32,
    total: i64,
}

impl ControllerOptionScore {
    #[must_use]
    pub const fn new(ordinal: u32, total: i64) -> Self {
        Self { ordinal, total }
    }
    #[must_use]
    pub const fn ordinal(self) -> u32 {
        self.ordinal
    }
    #[must_use]
    pub const fn total(self) -> i64 {
        self.total
    }
}

/// Optional non-authoritative selection diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControllerDiagnostic {
    kind: ControllerDecisionKind,
    decision_sequence: u64,
    selected_ordinal: u32,
    draw_count: Option<u64>,
    scores: Box<[ControllerOptionScore]>,
}

impl ControllerDiagnostic {
    pub fn new(
        kind: ControllerDecisionKind,
        decision_sequence: u64,
        selected_ordinal: u32,
        draw_count: Option<u64>,
        mut scores: Vec<ControllerOptionScore>,
    ) -> Result<Self, ControllerDiagnosticError> {
        if scores.is_empty() || scores.len() > MAX_CONTROLLER_OPTIONS as usize {
            return Err(ControllerDiagnosticError::InvalidOptionCount);
        }
        scores.sort_by_key(|score| score.ordinal);
        if scores
            .windows(2)
            .any(|pair| pair[0].ordinal == pair[1].ordinal)
        {
            return Err(ControllerDiagnosticError::DuplicateOrdinal);
        }
        let selected = scores
            .iter()
            .find(|score| score.ordinal == selected_ordinal)
            .ok_or(ControllerDiagnosticError::MissingSelection)?;
        let expected = scores
            .iter()
            .max_by(|left, right| {
                left.total
                    .cmp(&right.total)
                    .then_with(|| right.ordinal.cmp(&left.ordinal))
            })
            .expect("non-empty score list");
        if selected != expected {
            return Err(ControllerDiagnosticError::SelectionMismatch);
        }
        Ok(Self {
            kind,
            decision_sequence,
            selected_ordinal,
            draw_count,
            scores: scores.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn kind(&self) -> ControllerDecisionKind {
        self.kind
    }
    #[must_use]
    pub const fn decision_sequence(&self) -> u64 {
        self.decision_sequence
    }
    #[must_use]
    pub const fn selected_ordinal(&self) -> u32 {
        self.selected_ordinal
    }
    #[must_use]
    pub const fn draw_count(&self) -> Option<u64> {
        self.draw_count
    }
    #[must_use]
    pub fn scores(&self) -> &[ControllerOptionScore] {
        &self.scores
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControllerDiagnosticError {
    InvalidOptionCount,
    NonCanonicalOrdinals,
    DuplicateOrdinal,
    MissingSelection,
    SelectionMismatch,
}

/// Nested battle fact paired with one accepted Activity command.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedBattleBoundary {
    Start(BattleResultIdentity),
    End(BattleResultDigest),
}

/// One recorded accepted Activity command boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityTraceEntry {
    command: ActivityCommand,
    state_hash: ActivityStateHash,
    nested: NestedBattleBoundary,
    diagnostic: Option<ControllerDiagnostic>,
}

impl ActivityTraceEntry {
    pub fn new(
        command: ActivityCommand,
        state_hash: ActivityStateHash,
        nested: NestedBattleBoundary,
        diagnostic: Option<ControllerDiagnostic>,
    ) -> Result<Self, ActivityReplayError> {
        if !matches!(
            (&command, nested),
            (
                ActivityCommand::StartBattle { .. },
                NestedBattleBoundary::Start(_)
            ) | (
                ActivityCommand::SubmitBattleResult { .. },
                NestedBattleBoundary::End(_)
            )
        ) {
            return Err(ActivityReplayError::BoundaryCommandMismatch);
        }
        if let (
            ActivityCommand::SubmitBattleResult { result, .. },
            NestedBattleBoundary::End(digest),
        ) = (&command, nested)
            && result.claimed_digest() != digest
        {
            return Err(ActivityReplayError::BoundaryCommandMismatch);
        }
        Ok(Self {
            command,
            state_hash,
            nested,
            diagnostic,
        })
    }

    #[must_use]
    pub const fn command(&self) -> &ActivityCommand {
        &self.command
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
    #[must_use]
    pub const fn nested(&self) -> NestedBattleBoundary {
        self.nested
    }
    #[must_use]
    pub const fn diagnostic(&self) -> Option<&ControllerDiagnostic> {
        self.diagnostic.as_ref()
    }
}

pub fn activity_record_count(trace: &[ActivityTraceEntry]) -> Result<u32, ReplayFormatError> {
    let diagnostics = trace
        .iter()
        .filter(|entry| entry.diagnostic.is_some())
        .count();
    let records = trace
        .len()
        .checked_mul(3)
        .and_then(|value| value.checked_add(diagnostics))
        .ok_or(ReplayFormatError::TooManyRecords)?;
    let records = u32::try_from(records).map_err(|_| ReplayFormatError::TooManyRecords)?;
    if records > MAX_REPLAY_RECORDS {
        Err(ReplayFormatError::TooManyRecords)
    } else {
        Ok(records)
    }
}

pub fn encode_activity_trace(
    header: &ReplayHeader,
    trace: &[ActivityTraceEntry],
) -> Result<Vec<u8>, ActivityReplayError> {
    let expected = activity_record_count(trace)?;
    if header.record_count() != expected {
        return Err(ReplayFormatError::InvalidRecordSequence.into());
    }
    let mut payloads = Vec::<(RecordKind, Vec<u8>)>::with_capacity(expected as usize);
    for entry in trace {
        if let Some(diagnostic) = &entry.diagnostic {
            payloads.push((
                RecordKind::ControllerDiagnostic,
                encode_controller_diagnostic_payload(diagnostic)?,
            ));
        }
        match entry.nested {
            NestedBattleBoundary::Start(identity) => {
                payloads.push((
                    RecordKind::AcceptedActivityCommand,
                    encode_command(&entry.command)?,
                ));
                payloads.push((
                    RecordKind::ExpectedActivityState,
                    entry.state_hash.bytes().to_vec(),
                ));
                payloads.push((
                    RecordKind::NestedBattleStart,
                    encode_nested_battle_start_payload(identity),
                ));
            }
            NestedBattleBoundary::End(digest) => {
                payloads.push((
                    RecordKind::NestedBattleEnd,
                    encode_nested_battle_end_payload(digest),
                ));
                payloads.push((
                    RecordKind::AcceptedActivityCommand,
                    encode_command(&entry.command)?,
                ));
                payloads.push((
                    RecordKind::ExpectedActivityState,
                    entry.state_hash.bytes().to_vec(),
                ));
            }
        }
    }
    let records = payloads
        .iter()
        .enumerate()
        .map(|(sequence, (kind, payload))| RecordRef::new(*kind, sequence as u64, payload))
        .collect::<Result<Vec<_>, _>>()?;
    encode_replay(header, &records, Vec::new()).map_err(Into::into)
}

/// Successful streaming verification report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityReplayReport {
    command_count: u32,
    diagnostic_count: u32,
    final_hash: StateDigest,
    phase: ActivityPhase,
}

impl ActivityReplayReport {
    #[must_use]
    pub const fn command_count(self) -> u32 {
        self.command_count
    }
    #[must_use]
    pub const fn diagnostic_count(self) -> u32 {
        self.diagnostic_count
    }
    #[must_use]
    pub const fn final_hash(self) -> StateDigest {
        self.final_hash
    }
    #[must_use]
    pub const fn phase(self) -> ActivityPhase {
        self.phase
    }
}

pub fn verify_activity_replay(
    bytes: &[u8],
    mut activity: Activity,
    expected_profile_id: &str,
) -> Result<ActivityReplayReport, ActivityReplayError> {
    let replay = decode_replay(bytes)?;
    validate_identity(&replay, &activity, expected_profile_id)?;
    let records = replay.records();
    if records.is_empty() {
        return Err(ActivityReplayError::InvalidRecordLayout { record_index: 0 });
    }
    let mut record_index = 0usize;
    let mut command_index = 0u32;
    let mut diagnostic_count = 0u32;
    let mut final_hash = StateDigest::new(activity.state_hash().bytes());
    while record_index < records.len() {
        if records[record_index].kind() == RecordKind::ControllerDiagnostic {
            let _ = decode_controller_diagnostic_payload(records[record_index].payload())?;
            diagnostic_count += 1;
            record_index += 1;
        }
        let record = records
            .get(record_index)
            .ok_or(ActivityReplayError::InvalidRecordLayout {
                record_index: record_index as u32,
            })?;
        match record.kind() {
            RecordKind::AcceptedActivityCommand => {
                let command = decode_command(record.payload())?;
                if !matches!(command, ActivityCommand::StartBattle { .. }) {
                    return Err(ActivityReplayError::InvalidRecordLayout {
                        record_index: record_index as u32,
                    });
                }
                let resolution = apply(&mut activity, command, command_index)?;
                record_index += 1;
                compare_state(
                    records,
                    &mut record_index,
                    command_index,
                    resolution.state_hash(),
                    &mut final_hash,
                )?;
                let nested =
                    records
                        .get(record_index)
                        .ok_or(ActivityReplayError::InvalidRecordLayout {
                            record_index: record_index as u32,
                        })?;
                if nested.kind() != RecordKind::NestedBattleStart {
                    return Err(ActivityReplayError::InvalidRecordLayout {
                        record_index: record_index as u32,
                    });
                }
                let actual = decode_nested_battle_start_payload(nested.payload())?;
                let expected = resolution
                    .battle_handoff()
                    .ok_or(ActivityReplayError::MissingBattleHandoff)?
                    .identity();
                if actual != expected {
                    return Err(ActivityReplayError::NestedStartDivergence {
                        command_index,
                        expected: Box::new(expected),
                        actual: Box::new(actual),
                    });
                }
                record_index += 1;
            }
            RecordKind::NestedBattleEnd => {
                let boundary = decode_nested_battle_end_payload(record.payload())?;
                record_index += 1;
                let command_record =
                    records
                        .get(record_index)
                        .ok_or(ActivityReplayError::InvalidRecordLayout {
                            record_index: record_index as u32,
                        })?;
                if command_record.kind() != RecordKind::AcceptedActivityCommand {
                    return Err(ActivityReplayError::InvalidRecordLayout {
                        record_index: record_index as u32,
                    });
                }
                let command = decode_command(command_record.payload())?;
                let ActivityCommand::SubmitBattleResult { result, .. } = &command else {
                    return Err(ActivityReplayError::InvalidRecordLayout {
                        record_index: record_index as u32,
                    });
                };
                if result.claimed_digest() != boundary {
                    return Err(ActivityReplayError::NestedEndDivergence {
                        command_index,
                        expected: result.claimed_digest(),
                        actual: boundary,
                    });
                }
                let resolution = apply(&mut activity, command, command_index)?;
                record_index += 1;
                compare_state(
                    records,
                    &mut record_index,
                    command_index,
                    resolution.state_hash(),
                    &mut final_hash,
                )?;
            }
            _ => {
                return Err(ActivityReplayError::InvalidRecordLayout {
                    record_index: record_index as u32,
                });
            }
        }
        command_index += 1;
    }
    if !matches!(activity.phase(), ActivityPhase::Terminal(_)) {
        return Err(ActivityReplayError::IncompleteActivity);
    }
    Ok(ActivityReplayReport {
        command_count: command_index,
        diagnostic_count,
        final_hash,
        phase: activity.phase(),
    })
}

fn apply(
    activity: &mut Activity,
    command: ActivityCommand,
    command_index: u32,
) -> Result<starclock_activity::ActivityResolution, ActivityReplayError> {
    activity
        .apply(command)
        .map_err(|error| ActivityReplayError::CommandRejected {
            command_index,
            kind: error.kind(),
        })
}

fn compare_state(
    records: &[RecordRef<'_>],
    record_index: &mut usize,
    command_index: u32,
    actual: ActivityStateHash,
    final_hash: &mut StateDigest,
) -> Result<(), ActivityReplayError> {
    let record = records
        .get(*record_index)
        .ok_or(ActivityReplayError::InvalidRecordLayout {
            record_index: *record_index as u32,
        })?;
    if record.kind() != RecordKind::ExpectedActivityState {
        return Err(ActivityReplayError::InvalidRecordLayout {
            record_index: *record_index as u32,
        });
    }
    let expected: [u8; 32] = record
        .payload()
        .try_into()
        .map_err(|_| ActivityReplayError::InvalidStateHashPayload)?;
    if expected != actual.bytes() {
        return Err(ActivityReplayError::StateDivergence {
            command_index,
            expected: StateDigest::new(expected),
            actual: StateDigest::new(actual.bytes()),
        });
    }
    *final_hash = StateDigest::new(actual.bytes());
    *record_index += 1;
    Ok(())
}

fn validate_identity(
    replay: &DecodedReplay<'_>,
    activity: &Activity,
    expected_profile_id: &str,
) -> Result<(), ActivityReplayError> {
    let expected = match activity.decision() {
        ActivityDecision::StartBattle(identity) => identity,
        _ => return Err(ActivityReplayError::ActivityNotInitial),
    };
    let expected_definition_id = activity.definition_identity().id().get();
    let header = replay.header();
    if header.identity().config_bundle()
        != ConfigBundleDigest::new(expected.config_digest().bytes())
    {
        return Err(ActivityReplayError::IdentityMismatch(
            ActivityIdentityField::ConfigBundle,
        ));
    }
    match header.entry() {
        ReplayEntry::Activity {
            profile_id,
            definition_id,
            definition_digest,
            spec_digest,
            ..
        } if profile_id.as_ref() == expected_profile_id
            && *definition_id == expected_definition_id
            && definition_digest.bytes() == expected.definition_digest().bytes()
            && spec_digest.bytes() == expected.spec_digest().bytes() =>
        {
            Ok(())
        }
        ReplayEntry::Activity { profile_id, .. } if profile_id.as_ref() != expected_profile_id => {
            Err(ActivityReplayError::IdentityMismatch(
                ActivityIdentityField::Profile,
            ))
        }
        ReplayEntry::Activity {
            definition_digest, ..
        } if definition_digest.bytes() != expected.definition_digest().bytes() => Err(
            ActivityReplayError::IdentityMismatch(ActivityIdentityField::Definition),
        ),
        ReplayEntry::Activity { spec_digest, .. }
            if spec_digest.bytes() != expected.spec_digest().bytes() =>
        {
            Err(ActivityReplayError::IdentityMismatch(
                ActivityIdentityField::Spec,
            ))
        }
        ReplayEntry::Activity { definition_id, .. } if *definition_id != expected_definition_id => {
            Err(ActivityReplayError::IdentityMismatch(
                ActivityIdentityField::Entry,
            ))
        }
        ReplayEntry::Activity { .. } => Err(ActivityReplayError::IdentityMismatch(
            ActivityIdentityField::Entry,
        )),
        ReplayEntry::Battle { .. } => Err(ActivityReplayError::NotActivityReplay),
    }
}

fn encode_command(command: &ActivityCommand) -> Result<Vec<u8>, ActivityCommandPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(ACTIVITY_COMMAND_PAYLOAD_VERSION);
    match command {
        ActivityCommand::StartBattle {
            expected_state_hash,
        } => {
            encoder.u8(0);
            encoder.raw(&expected_state_hash.bytes());
        }
        ActivityCommand::SubmitBattleResult {
            expected_state_hash,
            result,
        } => {
            encoder.u8(1);
            encoder.raw(&expected_state_hash.bytes());
            encode_result(result, &mut encoder)?;
        }
    }
    Ok(encoder.into_inner())
}

fn decode_command(bytes: &[u8]) -> Result<ActivityCommand, ActivityCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if !matches!(version, 1 | ACTIVITY_COMMAND_PAYLOAD_VERSION) {
        return Err(ActivityCommandPayloadError::UnsupportedVersion(version));
    }
    let kind = decoder.u8()?;
    let expected = ActivityStateHash::new(fixed_digest(&mut decoder)?)
        .expect("activity state hash accepts every byte sequence");
    let command = match kind {
        0 => ActivityCommand::StartBattle {
            expected_state_hash: expected,
        },
        1 => ActivityCommand::SubmitBattleResult {
            expected_state_hash: expected,
            result: Box::new(decode_result(&mut decoder, version == 1)?),
        },
        other => return Err(ActivityCommandPayloadError::UnknownCommand(other)),
    };
    decoder.finish()?;
    Ok(command)
}

pub(super) fn encode_result(
    result: &BattleResult,
    encoder: &mut Encoder<Vec<u8>>,
) -> Result<(), ActivityCommandPayloadError> {
    encode_identity(result.identity(), encoder);
    encoder.u32(u32::try_from(result.values().len()).map_err(|_| CodecError::LengthOverflow)?);
    for value in result.values() {
        encode_projected(value, encoder)?;
    }
    encoder.raw(&result.claimed_digest().bytes());
    Ok(())
}

pub(super) fn decode_result(
    decoder: &mut Decoder<'_>,
    legacy_identity: bool,
) -> Result<BattleResult, ActivityCommandPayloadError> {
    let identity = if legacy_identity {
        decode_identity_legacy(decoder)?
    } else {
        decode_identity(decoder)?
    };
    let count = decoder.u32()?;
    if count == 0 || count > 100 {
        return Err(ActivityCommandPayloadError::InvalidProjection);
    }
    let mut values = Vec::with_capacity(count as usize);
    for _ in 0..count {
        values.push(decode_projected(decoder)?);
    }
    let claimed = BattleResultDigest::new(fixed_digest(decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    Ok(BattleResult::new(identity, values, claimed))
}

pub fn encode_battle_result_payload(
    result: &BattleResult,
) -> Result<Vec<u8>, ActivityCommandPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(BATTLE_RESULT_PAYLOAD_VERSION);
    encode_result(result, &mut encoder)?;
    Ok(encoder.into_inner())
}

pub fn decode_battle_result_payload(
    bytes: &[u8],
) -> Result<BattleResult, ActivityCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if !matches!(version, 1 | BATTLE_RESULT_PAYLOAD_VERSION) {
        return Err(ActivityCommandPayloadError::UnsupportedVersion(version));
    }
    let result = decode_result(&mut decoder, version == 1)?;
    decoder.finish()?;
    Ok(result)
}

fn encode_projected(
    value: &ProjectedValue,
    encoder: &mut Encoder<Vec<u8>>,
) -> Result<(), ActivityCommandPayloadError> {
    match value {
        ProjectedValue::Outcome(outcome) => {
            encoder.u8(0);
            encoder.u8(*outcome as u8);
        }
        ProjectedValue::FinalStateHash(hash) => {
            encoder.u8(1);
            encoder.raw(&hash.bytes());
        }
        ProjectedValue::EventDigest(digest) => {
            encoder.u8(2);
            encoder.raw(&digest.bytes());
        }
        ProjectedValue::TerminalFault(fault) => {
            encoder.u8(3);
            encoder.boolean(fault.is_some());
            if let Some(fault) = fault {
                encode_fault(*fault, encoder);
            }
        }
        ProjectedValue::Metric { key, value } => {
            encoder.u8(4);
            encoder.string(key)?;
            encode_metric(*value, encoder);
        }
        ProjectedValue::ParticipantState(state) => {
            encoder.u8(5);
            encoder.u32(state.participant().get());
            encoder.i64(state.current_hp().get());
            encoder.i64(state.maximum_hp().get());
            encoder.i64(state.current_energy().scaled());
            encoder.i64(state.maximum_energy().scaled());
            encoder.u8(state.life() as u8);
            encoder.u8(state.presence() as u8);
        }
    }
    Ok(())
}

fn decode_projected(
    decoder: &mut Decoder<'_>,
) -> Result<ProjectedValue, ActivityCommandPayloadError> {
    match decoder.u8()? {
        0 => Ok(ProjectedValue::Outcome(match decoder.u8()? {
            0 => BattleOutcome::Won,
            1 => BattleOutcome::Lost,
            2 => BattleOutcome::Faulted,
            other => return Err(ActivityCommandPayloadError::UnknownOutcome(other)),
        })),
        1 => Ok(ProjectedValue::FinalStateHash(BattleStateHash::from_bytes(
            fixed_digest(decoder)?,
        ))),
        2 => Ok(ProjectedValue::EventDigest(
            EventDigest::new(fixed_digest(decoder)?)
                .ok_or(ActivityCommandPayloadError::InvalidDigest)?,
        )),
        3 => {
            let fault = if decoder.boolean()? {
                Some(decode_fault(decoder)?)
            } else {
                None
            };
            Ok(ProjectedValue::TerminalFault(fault))
        }
        4 => {
            let key = Box::<str>::from(decoder.string(120)?);
            Ok(ProjectedValue::Metric {
                key,
                value: decode_metric(decoder)?,
            })
        }
        5 => {
            let participant =
                ParticipantId::new(decoder.u32()?).ok_or(ActivityCommandPayloadError::InvalidId)?;
            let current_hp = Hp::new(decoder.i64()?)
                .map_err(|_| ActivityCommandPayloadError::InvalidProjection)?;
            let maximum_hp = Hp::new(decoder.i64()?)
                .map_err(|_| ActivityCommandPayloadError::InvalidProjection)?;
            let current_energy = Energy::from_scaled(decoder.i64()?)
                .map_err(|_| ActivityCommandPayloadError::InvalidProjection)?;
            let maximum_energy = Energy::from_scaled(decoder.i64()?)
                .map_err(|_| ActivityCommandPayloadError::InvalidProjection)?;
            let life = match decoder.u8()? {
                0 => LifeState::Alive,
                1 => LifeState::Downed,
                2 => LifeState::Defeated,
                other => return Err(ActivityCommandPayloadError::UnknownLifeState(other)),
            };
            let presence = match decoder.u8()? {
                0 => PresenceState::Present,
                1 => PresenceState::Reserved,
                2 => PresenceState::Departed,
                3 => PresenceState::Untargetable,
                4 => PresenceState::Linked,
                5 => PresenceState::Transformed,
                other => return Err(ActivityCommandPayloadError::UnknownPresenceState(other)),
            };
            let state = ParticipantBattleState::new(
                participant,
                current_hp,
                maximum_hp,
                current_energy,
                maximum_energy,
                life,
                presence,
            )
            .ok_or(ActivityCommandPayloadError::InvalidProjection)?;
            Ok(ProjectedValue::ParticipantState(state))
        }
        other => Err(ActivityCommandPayloadError::UnknownProjection(other)),
    }
}

fn encode_metric(value: MetricValue, encoder: &mut Encoder<Vec<u8>>) {
    match value {
        MetricValue::BoundedInteger(value) => {
            encoder.u8(0);
            encoder.i64(value);
        }
        MetricValue::FixedScalar(value) => {
            encoder.u8(1);
            encoder.i64(value);
        }
        MetricValue::Ratio(value) => {
            encoder.u8(2);
            encoder.i64(value);
        }
        MetricValue::Probability(value) => {
            encoder.u8(3);
            encoder.i64(value);
        }
        MetricValue::ActionValue(value) => {
            encoder.u8(4);
            encoder.i64(value);
        }
    }
}

fn decode_metric(decoder: &mut Decoder<'_>) -> Result<MetricValue, ActivityCommandPayloadError> {
    let kind = decoder.u8()?;
    let value = decoder.i64()?;
    match kind {
        0 => Ok(MetricValue::BoundedInteger(value)),
        1 => Ok(MetricValue::FixedScalar(value)),
        2 => Ok(MetricValue::Ratio(value)),
        3 => Ok(MetricValue::Probability(value)),
        4 => Ok(MetricValue::ActionValue(value)),
        other => Err(ActivityCommandPayloadError::UnknownMetric(other)),
    }
}

fn encode_fault(fault: BattleFault, encoder: &mut Encoder<Vec<u8>>) {
    encoder.u8(fault.kind() as u8);
    encoder.u8(fault.boundary() as u8);
    encoder.u8(fault.policy() as u8);
    encoder.u32(fault.context_code());
    encoder.boolean(fault.numeric_context().is_some());
    if let Some(value) = fault.numeric_context() {
        encoder.i64(value);
    }
}

fn decode_fault(decoder: &mut Decoder<'_>) -> Result<BattleFault, ActivityCommandPayloadError> {
    let kind = match decoder.u8()? {
        0 => FaultKind::Numeric,
        1 => FaultKind::BudgetExceeded,
        2 => FaultKind::InvariantViolation,
        3 => FaultKind::SequenceExhausted,
        other => return Err(ActivityCommandPayloadError::UnknownFaultKind(other)),
    };
    let boundary = match decoder.u8()? {
        0 => FaultBoundary::Transaction,
        1 => FaultBoundary::Command,
        2 => FaultBoundary::Commit,
        other => return Err(ActivityCommandPayloadError::UnknownFaultBoundary(other)),
    };
    let policy = match decoder.u8()? {
        0 => FaultPolicy::Rollback,
        1 => FaultPolicy::CommitFault,
        other => return Err(ActivityCommandPayloadError::UnknownFaultPolicy(other)),
    };
    let context = decoder.u32()?;
    let numeric = if decoder.boolean()? {
        Some(decoder.i64()?)
    } else {
        None
    };
    Ok(BattleFault::from_parts(
        kind, boundary, policy, context, numeric,
    ))
}

#[must_use]
pub fn encode_nested_battle_start_payload(identity: BattleResultIdentity) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_PAYLOAD_VERSION);
    encode_identity(identity, &mut encoder);
    encoder.into_inner()
}

pub fn decode_nested_battle_start_payload(
    bytes: &[u8],
) -> Result<BattleResultIdentity, ActivityCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if !matches!(version, 1 | NESTED_BATTLE_PAYLOAD_VERSION) {
        return Err(ActivityCommandPayloadError::UnsupportedNestedVersion(
            version,
        ));
    }
    let identity = if version == 1 {
        decode_identity_legacy(&mut decoder)?
    } else {
        decode_identity(&mut decoder)?
    };
    decoder.finish()?;
    Ok(identity)
}

#[must_use]
pub fn encode_nested_battle_end_payload(digest: BattleResultDigest) -> Vec<u8> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_PAYLOAD_VERSION);
    encoder.raw(&digest.bytes());
    encoder.into_inner()
}

pub fn decode_nested_battle_end_payload(
    bytes: &[u8],
) -> Result<BattleResultDigest, ActivityCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if !matches!(version, 1 | NESTED_BATTLE_PAYLOAD_VERSION) {
        return Err(ActivityCommandPayloadError::UnsupportedNestedVersion(
            version,
        ));
    }
    let digest = BattleResultDigest::new(fixed_digest(&mut decoder)?)
        .ok_or(ActivityCommandPayloadError::InvalidDigest)?;
    decoder.finish()?;
    Ok(digest)
}

/// Encodes one bounded non-authoritative controller diagnostic payload.
pub fn encode_controller_diagnostic_payload(
    value: &ControllerDiagnostic,
) -> Result<Vec<u8>, ActivityCommandPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(CONTROLLER_DIAGNOSTIC_PAYLOAD_VERSION);
    encoder.u8(value.kind as u8);
    encoder.u64(value.decision_sequence);
    encoder.u32(value.selected_ordinal);
    encoder.boolean(value.draw_count.is_some());
    if let Some(draws) = value.draw_count {
        encoder.u64(draws);
    }
    encoder.u32(u32::try_from(value.scores.len()).map_err(|_| CodecError::LengthOverflow)?);
    for score in &value.scores {
        encoder.u32(score.ordinal);
        encoder.i64(score.total);
    }
    Ok(encoder.into_inner())
}

/// Decodes one bounded non-authoritative controller diagnostic payload.
pub fn decode_controller_diagnostic_payload(
    bytes: &[u8],
) -> Result<ControllerDiagnostic, ActivityCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if version != CONTROLLER_DIAGNOSTIC_PAYLOAD_VERSION {
        return Err(ActivityCommandPayloadError::UnsupportedDiagnosticVersion(
            version,
        ));
    }
    let kind = match decoder.u8()? {
        0 => ControllerDecisionKind::Activity,
        1 => ControllerDecisionKind::BattlePlayer,
        2 => ControllerDecisionKind::EnemyAuthored,
        other => return Err(ActivityCommandPayloadError::UnknownControllerKind(other)),
    };
    let sequence = decoder.u64()?;
    let selected = decoder.u32()?;
    let draw_count = if decoder.boolean()? {
        Some(decoder.u64()?)
    } else {
        None
    };
    let count = decoder.u32()?;
    if count == 0 || count > MAX_CONTROLLER_OPTIONS {
        return Err(ActivityCommandPayloadError::InvalidDiagnostic(
            ControllerDiagnosticError::InvalidOptionCount,
        ));
    }
    let mut scores = Vec::with_capacity(count as usize);
    for _ in 0..count {
        scores.push(ControllerOptionScore::new(decoder.u32()?, decoder.i64()?));
    }
    decoder.finish()?;
    if scores
        .windows(2)
        .any(|pair| pair[0].ordinal >= pair[1].ordinal)
    {
        return Err(ActivityCommandPayloadError::InvalidDiagnostic(
            ControllerDiagnosticError::NonCanonicalOrdinals,
        ));
    }
    ControllerDiagnostic::new(kind, sequence, selected, draw_count, scores)
        .map_err(ActivityCommandPayloadError::InvalidDiagnostic)
}

pub(super) fn fixed_digest(decoder: &mut Decoder<'_>) -> Result<[u8; 32], CodecError> {
    Ok(decoder.take(32)?.try_into().expect("fixed length"))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityCommandPayloadError {
    UnsupportedVersion(u16),
    UnsupportedNestedVersion(u16),
    UnsupportedDiagnosticVersion(u16),
    UnknownCommand(u8),
    UnknownOutcome(u8),
    UnknownProjection(u8),
    UnknownMetric(u8),
    UnknownLifeState(u8),
    UnknownPresenceState(u8),
    UnknownFaultKind(u8),
    UnknownFaultBoundary(u8),
    UnknownFaultPolicy(u8),
    UnknownControllerKind(u8),
    InvalidId,
    InvalidDigest,
    InvalidProjection,
    InvalidDiagnostic(ControllerDiagnosticError),
    Codec(CodecError),
}

impl From<CodecError> for ActivityCommandPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityIdentityField {
    ConfigBundle,
    Profile,
    Definition,
    Spec,
    Entry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityReplayError {
    Format(ReplayFormatError),
    CommandPayload(ActivityCommandPayloadError),
    NotActivityReplay,
    ActivityNotInitial,
    IdentityMismatch(ActivityIdentityField),
    InvalidRecordLayout {
        record_index: u32,
    },
    InvalidStateHashPayload,
    BoundaryCommandMismatch,
    MissingBattleHandoff,
    IncompleteActivity,
    CommandRejected {
        command_index: u32,
        kind: ActivityCommandErrorKind,
    },
    StateDivergence {
        command_index: u32,
        expected: StateDigest,
        actual: StateDigest,
    },
    NestedStartDivergence {
        command_index: u32,
        expected: Box<BattleResultIdentity>,
        actual: Box<BattleResultIdentity>,
    },
    NestedEndDivergence {
        command_index: u32,
        expected: BattleResultDigest,
        actual: BattleResultDigest,
    },
}

impl From<ReplayFormatError> for ActivityReplayError {
    fn from(value: ReplayFormatError) -> Self {
        Self::Format(value)
    }
}

impl From<ActivityCommandPayloadError> for ActivityReplayError {
    fn from(value: ActivityCommandPayloadError) -> Self {
        Self::CommandPayload(value)
    }
}

impl fmt::Display for ActivityReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "activity replay error: {self:?}")
    }
}

impl std::error::Error for ActivityReplayError {}

#[cfg(test)]
mod tests {
    use starclock_activity::{
        ActivityConfigDigest, ActivityDefinitionDigest, ActivityInstanceId, AttemptId,
        BattleResultConfiguration, BattleSequence, NodeId, ScopeIdentity, SectionId,
    };
    use starclock_combat::{BattleSeed, BattleSpecDigest};

    use super::*;

    #[test]
    fn submitted_result_payload_round_trips_every_projection_family() {
        let scope = ScopeIdentity::new(
            ActivityInstanceId::new(1).unwrap(),
            SectionId::new(2).unwrap(),
            NodeId::new(3).unwrap(),
            AttemptId::new(4).unwrap(),
        );
        let identity = BattleResultIdentity::new_legacy(
            scope,
            BattleSequence::new(5).unwrap(),
            BattleResultConfiguration::new(
                ActivityDefinitionDigest::new([0x11; 32]).unwrap(),
                ActivityConfigDigest::new([0x12; 32]).unwrap(),
                starclock_activity::ParticipantLockDigest::new([0x13; 32]).unwrap(),
            ),
            BattleSpecDigest::new([0x14; 32]).unwrap(),
            BattleSeed::new([0x15; 32]),
        );
        let fault = BattleFault::from_parts(
            FaultKind::Numeric,
            FaultBoundary::Command,
            FaultPolicy::CommitFault,
            17,
            Some(-9),
        );
        let result = BattleResult::seal(
            identity,
            vec![
                ProjectedValue::Outcome(BattleOutcome::Faulted),
                ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x21; 32])),
                ProjectedValue::EventDigest(EventDigest::new([0x22; 32]).unwrap()),
                ProjectedValue::TerminalFault(Some(fault)),
                ProjectedValue::ParticipantState(
                    ParticipantBattleState::new(
                        ParticipantId::new(9).unwrap(),
                        Hp::new(0).unwrap(),
                        Hp::new(1_000).unwrap(),
                        Energy::from_scaled(25_000_000).unwrap(),
                        Energy::from_scaled(100_000_000).unwrap(),
                        LifeState::Defeated,
                        PresenceState::Departed,
                    )
                    .unwrap(),
                ),
                metric("bounded", MetricValue::BoundedInteger(-1)),
                metric("fixed", MetricValue::FixedScalar(2)),
                metric("ratio", MetricValue::Ratio(3)),
                metric("probability", MetricValue::Probability(4)),
                metric("action", MetricValue::ActionValue(5)),
            ],
        );
        let command = ActivityCommand::SubmitBattleResult {
            expected_state_hash: ActivityStateHash::new([0x31; 32]).unwrap(),
            result: Box::new(result),
        };

        let encoded = encode_command(&command).expect("payload encodes");
        assert_eq!(decode_command(&encoded), Ok(command));
    }

    #[test]
    fn diagnostic_validation_is_order_independent_and_tie_stable() {
        let diagnostic = ControllerDiagnostic::new(
            ControllerDecisionKind::EnemyAuthored,
            8,
            2,
            None,
            vec![
                ControllerOptionScore::new(7, 10),
                ControllerOptionScore::new(2, 10),
            ],
        )
        .expect("lowest canonical ordinal wins equal scores");
        assert_eq!(diagnostic.selected_ordinal(), 2);
        let encoded = encode_controller_diagnostic_payload(&diagnostic).unwrap();
        assert_eq!(
            decode_controller_diagnostic_payload(&encoded),
            Ok(diagnostic)
        );

        let mut noncanonical = encoded;
        let first: [u8; 12] = noncanonical[20..32].try_into().unwrap();
        let second: [u8; 12] = noncanonical[32..44].try_into().unwrap();
        noncanonical[20..32].copy_from_slice(&second);
        noncanonical[32..44].copy_from_slice(&first);
        assert_eq!(
            decode_controller_diagnostic_payload(&noncanonical),
            Err(ActivityCommandPayloadError::InvalidDiagnostic(
                ControllerDiagnosticError::NonCanonicalOrdinals,
            ))
        );
    }

    fn metric(key: &str, value: MetricValue) -> ProjectedValue {
        ProjectedValue::Metric {
            key: key.into(),
            value,
        }
    }
}
