//! Versioned full-run replay for the Standard Universe Activity facade.

use starclock_activity::{
    ACTIVITY_STATE_HASH_REVISION, ActivityDecisionId, ActivityDecisionKind,
    ActivityExternalOutcomeId, ActivityOptionId, ActivityStateHash, ActivityTerminalOutcome,
    BattleResult, BattleResultDigest, BattleResultIdentity,
};
use starclock_replay::{
    activity::{
        ActivityCommandPayloadError, ControllerDecisionKind, ControllerDiagnostic,
        ControllerOptionScore, decode_battle_result_payload, decode_controller_diagnostic_payload,
        decode_nested_battle_end_payload, decode_nested_battle_start_payload,
        encode_battle_result_payload, encode_controller_diagnostic_payload,
        encode_nested_battle_end_payload, encode_nested_battle_start_payload,
    },
    codec::{CodecError, Decoder, Encoder},
    digest::{DefinitionDigest, EntrySpecDigest, StateDigest},
    format::{ReplayEntry, ReplayHeader, decode_replay, encode_replay},
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

use crate::{
    baseline_controller::ActivityBaselineDecision,
    baseline_runner::{
        NestedBattleExecutor, StandardUniverseBaselineError, StandardUniverseBaselinePolicy,
        StandardUniverseBaselineReport, StandardUniverseBaselineRunner,
        StandardUniverseBaselineStep,
    },
    runtime::{
        StandardUniverseActivity, StandardUniverseBattleStartError, StandardUniverseEncounterError,
    },
};

pub const STANDARD_UNIVERSE_REPLAY_ACTION_VERSION: u16 = 1;
pub const MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS: u32 = 100_000;

/// One accepted facade action. Nested execution is one atomic replay boundary:
/// both handoff identity and complete returned projection are retained.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StandardUniverseReplayAction {
    Decision {
        decision: ActivityDecisionId,
        kind: ActivityDecisionKind,
        option: ActivityOptionId,
        technique_points: u16,
    },
    Preparation {
        option: ActivityOptionId,
    },
    Battle {
        result: Box<BattleResult>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StandardUniverseTraceEntry {
    action: StandardUniverseReplayAction,
    state_hash: ActivityStateHash,
    diagnostic: Option<ControllerDiagnostic>,
}

impl StandardUniverseTraceEntry {
    #[must_use]
    pub fn new(
        action: StandardUniverseReplayAction,
        state_hash: ActivityStateHash,
        diagnostic: Option<ControllerDiagnostic>,
    ) -> Self {
        Self {
            action,
            state_hash,
            diagnostic,
        }
    }
    #[must_use]
    pub const fn action(&self) -> &StandardUniverseReplayAction {
        &self.action
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
    #[must_use]
    pub const fn diagnostic(&self) -> Option<&ControllerDiagnostic> {
        self.diagnostic.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordedStandardUniverseRun {
    report: StandardUniverseBaselineReport,
    trace: Box<[StandardUniverseTraceEntry]>,
}

impl RecordedStandardUniverseRun {
    #[must_use]
    pub const fn report(&self) -> &StandardUniverseBaselineReport {
        &self.report
    }
    #[must_use]
    pub fn trace(&self) -> &[StandardUniverseTraceEntry] {
        &self.trace
    }
}

/// Drives one baseline run while retaining only the battle results required by
/// authoritative replay. The ordinary baseline runner remains allocation-light.
pub fn record_baseline_run<E: NestedBattleExecutor>(
    activity: &mut StandardUniverseActivity,
    policy: &StandardUniverseBaselinePolicy,
    executor: &mut E,
) -> Result<RecordedStandardUniverseRun, StandardUniverseReplayError> {
    let mut capture = CapturingExecutor {
        inner: executor,
        results: Vec::new(),
    };
    let report = StandardUniverseBaselineRunner::default()
        .run_to_terminal(activity, policy, &mut capture)
        .map_err(StandardUniverseReplayError::Runner)?;
    if report.steps().len() > MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS as usize {
        return Err(StandardUniverseReplayError::TooManyActions);
    }
    let mut results = capture.results.into_iter();
    let mut trace = Vec::with_capacity(report.steps().len());
    for (sequence, step) in report.steps().iter().enumerate() {
        let (action, state_hash, diagnostic) = match step {
            StandardUniverseBaselineStep::Decision {
                decision,
                state_hash,
            } => (
                StandardUniverseReplayAction::Decision {
                    decision: decision.decision(),
                    kind: decision.kind(),
                    option: decision.option(),
                    technique_points: policy.technique_points(),
                },
                *state_hash,
                Some(controller_diagnostic(sequence as u64, decision)?),
            ),
            StandardUniverseBaselineStep::Preparation { option, state_hash } => (
                StandardUniverseReplayAction::Preparation { option: *option },
                *state_hash,
                None,
            ),
            StandardUniverseBaselineStep::Battle {
                identity,
                result_digest,
                state_hash,
                ..
            } => {
                let result = results
                    .next()
                    .ok_or(StandardUniverseReplayError::CapturedBattleMismatch)?;
                if result.identity() != *identity || result.actual_digest() != *result_digest {
                    return Err(StandardUniverseReplayError::CapturedBattleMismatch);
                }
                (
                    StandardUniverseReplayAction::Battle {
                        result: Box::new(result),
                    },
                    *state_hash,
                    None,
                )
            }
        };
        trace.push(StandardUniverseTraceEntry::new(
            action, state_hash, diagnostic,
        ));
    }
    if results.next().is_some() {
        return Err(StandardUniverseReplayError::CapturedBattleMismatch);
    }
    Ok(RecordedStandardUniverseRun {
        report,
        trace: trace.into_boxed_slice(),
    })
}

struct CapturingExecutor<'a, E> {
    inner: &'a mut E,
    results: Vec<BattleResult>,
}

impl<E: NestedBattleExecutor> NestedBattleExecutor for CapturingExecutor<'_, E> {
    fn execute(&mut self, handoff: &starclock_activity::ActivityBattleHandoff) -> BattleResult {
        let result = self.inner.execute(handoff);
        self.results.push(result.clone());
        result
    }
}

fn controller_diagnostic(
    sequence: u64,
    decision: &ActivityBaselineDecision,
) -> Result<ControllerDiagnostic, StandardUniverseReplayError> {
    let selected = decision
        .scores()
        .iter()
        .position(|score| score.option() == decision.option())
        .ok_or(StandardUniverseReplayError::DiagnosticMismatch)?;
    ControllerDiagnostic::new(
        ControllerDecisionKind::Activity,
        sequence,
        u32::try_from(selected).map_err(|_| StandardUniverseReplayError::TooManyActions)?,
        None,
        decision
            .scores()
            .iter()
            .enumerate()
            .map(|(ordinal, score)| {
                Ok(ControllerOptionScore::new(
                    u32::try_from(ordinal)
                        .map_err(|_| StandardUniverseReplayError::TooManyActions)?,
                    score.total(),
                ))
            })
            .collect::<Result<Vec<_>, StandardUniverseReplayError>>()?,
    )
    .map_err(|_| StandardUniverseReplayError::DiagnosticMismatch)
}

pub fn standard_universe_record_count(
    trace: &[StandardUniverseTraceEntry],
) -> Result<u32, StandardUniverseReplayError> {
    if trace.len() > MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS as usize {
        return Err(StandardUniverseReplayError::TooManyActions);
    }
    let mut count = 0_u32;
    for entry in trace {
        count = count
            .checked_add(2)
            .and_then(|value| value.checked_add(u32::from(entry.diagnostic.is_some())))
            .and_then(|value| {
                value.checked_add(
                    u32::from(matches!(
                        entry.action,
                        StandardUniverseReplayAction::Battle { .. }
                    )) * 2,
                )
            })
            .ok_or(StandardUniverseReplayError::TooManyActions)?;
    }
    if count > MAX_REPLAY_RECORDS {
        Err(StandardUniverseReplayError::TooManyActions)
    } else {
        Ok(count)
    }
}

/// Encodes a complete trace using a zero-record header template.
pub fn encode_standard_universe_trace(
    header_template: &ReplayHeader,
    trace: &[StandardUniverseTraceEntry],
) -> Result<Vec<u8>, StandardUniverseReplayError> {
    let count = standard_universe_record_count(trace)?;
    let header = ReplayHeader::new(
        header_template.identity().clone(),
        header_template.controller().clone(),
        header_template.master_seed(),
        header_template.entry().clone(),
        count,
    )?;
    let mut payloads = Vec::<(RecordKind, Vec<u8>)>::with_capacity(count as usize);
    for entry in trace {
        if let Some(diagnostic) = entry.diagnostic() {
            payloads.push((
                RecordKind::ControllerDiagnostic,
                encode_controller_diagnostic_payload(diagnostic)?,
            ));
        }
        if let StandardUniverseReplayAction::Battle { result } = entry.action() {
            payloads.push((
                RecordKind::NestedBattleStart,
                encode_nested_battle_start_payload(result.identity()),
            ));
        }
        payloads.push((
            RecordKind::AcceptedActivityCommand,
            encode_action(entry.action())?,
        ));
        if let StandardUniverseReplayAction::Battle { result } = entry.action() {
            payloads.push((
                RecordKind::NestedBattleEnd,
                encode_nested_battle_end_payload(result.actual_digest()),
            ));
        }
        payloads.push((
            RecordKind::ExpectedActivityState,
            entry.state_hash().bytes().to_vec(),
        ));
    }
    let records = payloads
        .iter()
        .enumerate()
        .map(|(sequence, (kind, payload))| RecordRef::new(*kind, sequence as u64, payload))
        .collect::<Result<Vec<_>, _>>()?;
    encode_replay(&header, &records, Vec::new()).map_err(Into::into)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StandardUniverseReplayReport {
    action_count: u32,
    diagnostic_count: u32,
    nested_battle_count: u32,
    final_state_hash: StateDigest,
    terminal: ActivityTerminalOutcome,
}

impl StandardUniverseReplayReport {
    #[must_use]
    pub const fn action_count(self) -> u32 {
        self.action_count
    }
    #[must_use]
    pub const fn diagnostic_count(self) -> u32 {
        self.diagnostic_count
    }
    #[must_use]
    pub const fn nested_battle_count(self) -> u32 {
        self.nested_battle_count
    }
    #[must_use]
    pub const fn final_state_hash(self) -> StateDigest {
        self.final_state_hash
    }
    #[must_use]
    pub const fn terminal(self) -> ActivityTerminalOutcome {
        self.terminal
    }
}

pub fn verify_standard_universe_replay(
    bytes: &[u8],
    activity: StandardUniverseActivity,
    expected_profile_id: &str,
) -> Result<StandardUniverseReplayReport, StandardUniverseReplayError> {
    verify_standard_universe_replay_with_controller(
        bytes,
        activity,
        expected_profile_id,
        StandardUniverseBaselineRunner::REVISION,
    )
}

/// Verifies the same authoritative Activity trace for a specifically bound
/// external controller. The baseline helper above preserves the original B2
/// contract while agent sessions bind their own truthful controller identity.
pub fn verify_standard_universe_replay_with_controller(
    bytes: &[u8],
    mut activity: StandardUniverseActivity,
    expected_profile_id: &str,
    expected_controller_revision: &str,
) -> Result<StandardUniverseReplayReport, StandardUniverseReplayError> {
    let replay = decode_replay(bytes)?;
    validate_identity(
        replay.header(),
        &activity,
        expected_profile_id,
        expected_controller_revision,
    )?;
    let records = replay.records();
    let mut record_index = 0_usize;
    let mut action_index = 0_u32;
    let mut diagnostic_count = 0_u32;
    let mut nested_battle_count = 0_u32;
    let mut final_state_hash = StateDigest::new(activity.view().state_hash().bytes());
    while record_index < records.len() {
        if action_index >= MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS {
            return Err(StandardUniverseReplayError::TooManyActions);
        }
        let diagnostic = if records[record_index].kind() == RecordKind::ControllerDiagnostic {
            let value = decode_controller_diagnostic_payload(records[record_index].payload())?;
            diagnostic_count += 1;
            record_index += 1;
            Some(value)
        } else {
            None
        };
        let nested_start = if records
            .get(record_index)
            .is_some_and(|record| record.kind() == RecordKind::NestedBattleStart)
        {
            let identity = decode_nested_battle_start_payload(records[record_index].payload())?;
            record_index += 1;
            Some(identity)
        } else {
            None
        };
        let command =
            records
                .get(record_index)
                .ok_or(StandardUniverseReplayError::InvalidRecordLayout {
                    record_index: record_index as u32,
                })?;
        if command.kind() != RecordKind::AcceptedActivityCommand {
            return Err(StandardUniverseReplayError::InvalidRecordLayout {
                record_index: record_index as u32,
            });
        }
        let action = decode_action(command.payload())?;
        record_index += 1;
        apply_replayed_action(
            &mut activity,
            &action,
            diagnostic.as_ref(),
            nested_start,
            records,
            &mut record_index,
            action_index,
            &mut nested_battle_count,
        )?;
        compare_state(
            &activity,
            records,
            &mut record_index,
            action_index,
            &mut final_state_hash,
        )?;
        action_index += 1;
    }
    let terminal = activity
        .view()
        .terminal()
        .ok_or(StandardUniverseReplayError::IncompleteActivity)?;
    Ok(StandardUniverseReplayReport {
        action_count: action_index,
        diagnostic_count,
        nested_battle_count,
        final_state_hash,
        terminal,
    })
}

#[allow(clippy::too_many_arguments)]
fn apply_replayed_action(
    activity: &mut StandardUniverseActivity,
    action: &StandardUniverseReplayAction,
    diagnostic: Option<&ControllerDiagnostic>,
    nested_start: Option<BattleResultIdentity>,
    records: &[RecordRef<'_>],
    record_index: &mut usize,
    action_index: u32,
    nested_battle_count: &mut u32,
) -> Result<(), StandardUniverseReplayError> {
    match action {
        StandardUniverseReplayAction::Decision {
            decision,
            kind,
            option,
            technique_points,
        } => {
            if nested_start.is_some() {
                return invalid_layout(*record_index);
            }
            validate_decision(
                activity,
                *decision,
                *kind,
                *option,
                diagnostic,
                action_index,
            )?;
            let hash = activity.view().state_hash();
            match kind {
                ActivityDecisionKind::Encounter => activity
                    .engage_encounter(hash, *decision, *option, *technique_points)
                    .map(|_| ())
                    .map_err(|error| StandardUniverseReplayError::EncounterRejected {
                        action_index,
                        error,
                    }),
                ActivityDecisionKind::ExternalOutcome => activity
                    .submit_external_outcome(
                        hash,
                        *decision,
                        ActivityExternalOutcomeId::new(option.get())
                            .expect("offered option IDs are non-zero"),
                    )
                    .map(|_| ())
                    .map_err(|_| StandardUniverseReplayError::CommandRejected { action_index }),
                _ => activity
                    .choose_option(hash, *decision, *option)
                    .map(|_| ())
                    .map_err(|_| StandardUniverseReplayError::CommandRejected { action_index }),
            }
        }
        StandardUniverseReplayAction::Preparation { option } => {
            if nested_start.is_some() || diagnostic.is_some() {
                return invalid_layout(*record_index);
            }
            activity
                .choose_preparation_option(activity.view().state_hash(), *option)
                .map(|_| ())
                .map_err(|_| StandardUniverseReplayError::PreparationRejected { action_index })
        }
        StandardUniverseReplayAction::Battle { result } => {
            if diagnostic.is_some() {
                return invalid_layout(*record_index);
            }
            let recorded_start = nested_start
                .ok_or(StandardUniverseReplayError::MissingNestedBoundary { action_index })?;
            let handoff = activity
                .start_pending_battle(activity.view().state_hash())
                .map_err(|error| StandardUniverseReplayError::BattleStartRejected {
                    action_index,
                    error,
                })?;
            if handoff.identity() != recorded_start || result.identity() != recorded_start {
                return Err(StandardUniverseReplayError::NestedStartDivergence {
                    action_index,
                    expected: Box::new(handoff.identity()),
                    actual: Box::new(recorded_start),
                });
            }
            let end = records.get(*record_index).ok_or(
                StandardUniverseReplayError::InvalidRecordLayout {
                    record_index: *record_index as u32,
                },
            )?;
            if end.kind() != RecordKind::NestedBattleEnd {
                return invalid_layout(*record_index);
            }
            let recorded_end = decode_nested_battle_end_payload(end.payload())?;
            *record_index += 1;
            let actual_end = result.actual_digest();
            if recorded_end != actual_end {
                return Err(StandardUniverseReplayError::NestedEndDivergence {
                    action_index,
                    expected: actual_end,
                    actual: recorded_end,
                });
            }
            activity
                .submit_pending_battle_result(activity.view().state_hash(), result.as_ref().clone())
                .map_err(|_| StandardUniverseReplayError::BattleResultRejected { action_index })?;
            *nested_battle_count += 1;
            Ok(())
        }
    }
}

fn validate_decision(
    activity: &StandardUniverseActivity,
    decision: ActivityDecisionId,
    kind: ActivityDecisionKind,
    option: ActivityOptionId,
    diagnostic: Option<&ControllerDiagnostic>,
    action_index: u32,
) -> Result<(), StandardUniverseReplayError> {
    let view = activity.view();
    let offered = view
        .decision()
        .filter(|offered| offered.id() == decision && offered.kind() == kind)
        .ok_or(StandardUniverseReplayError::DecisionDivergence { action_index })?;
    if !offered
        .options()
        .iter()
        .any(|candidate| candidate.id() == option)
    {
        return Err(StandardUniverseReplayError::DecisionDivergence { action_index });
    }
    let diagnostic = diagnostic.ok_or(StandardUniverseReplayError::DiagnosticMismatch)?;
    if diagnostic.kind() != ControllerDecisionKind::Activity
        || diagnostic.decision_sequence() != u64::from(action_index)
        || diagnostic.scores().len() != offered.options().len()
    {
        return Err(StandardUniverseReplayError::DiagnosticMismatch);
    }
    let selected = usize::try_from(diagnostic.selected_ordinal())
        .map_err(|_| StandardUniverseReplayError::DiagnosticMismatch)?;
    if offered.options().get(selected).map(|item| item.id()) != Some(option) {
        return Err(StandardUniverseReplayError::DiagnosticMismatch);
    }
    Ok(())
}

fn compare_state(
    activity: &StandardUniverseActivity,
    records: &[RecordRef<'_>],
    record_index: &mut usize,
    action_index: u32,
    final_hash: &mut StateDigest,
) -> Result<(), StandardUniverseReplayError> {
    let record =
        records
            .get(*record_index)
            .ok_or(StandardUniverseReplayError::InvalidRecordLayout {
                record_index: *record_index as u32,
            })?;
    if record.kind() != RecordKind::ExpectedActivityState {
        return invalid_layout(*record_index);
    }
    let expected: [u8; 32] = record
        .payload()
        .try_into()
        .map_err(|_| StandardUniverseReplayError::InvalidStateHashPayload)?;
    let actual = activity.view().state_hash().bytes();
    if expected != actual {
        return Err(StandardUniverseReplayError::StateDivergence {
            action_index,
            expected: StateDigest::new(expected),
            actual: StateDigest::new(actual),
        });
    }
    *final_hash = StateDigest::new(actual);
    *record_index += 1;
    Ok(())
}

fn validate_identity(
    header: &ReplayHeader,
    activity: &StandardUniverseActivity,
    expected_profile_id: &str,
    expected_controller_revision: &str,
) -> Result<(), StandardUniverseReplayError> {
    if header.controller().revision() != expected_controller_revision
        || header.identity().state_hash_revision() != ACTIVITY_STATE_HASH_REVISION
    {
        return Err(StandardUniverseReplayError::IdentityMismatch);
    }
    let identity = activity.graph().definition().identity();
    match header.entry() {
        ReplayEntry::Activity {
            profile_id,
            definition_id,
            definition_digest,
            spec_digest,
            ..
        } if profile_id.as_ref() == expected_profile_id
            && *definition_id == identity.id().get()
            && definition_digest.bytes() == identity.definition_digest().bytes()
            && spec_digest.bytes() == identity.config_digest().bytes() =>
        {
            Ok(())
        }
        _ => Err(StandardUniverseReplayError::IdentityMismatch),
    }
}

fn encode_action(
    action: &StandardUniverseReplayAction,
) -> Result<Vec<u8>, StandardUniverseReplayError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(STANDARD_UNIVERSE_REPLAY_ACTION_VERSION);
    match action {
        StandardUniverseReplayAction::Decision {
            decision,
            kind,
            option,
            technique_points,
        } => {
            encoder.u8(0);
            encoder.u64(decision.get());
            encoder.u8(*kind as u8);
            encoder.u64(option.get());
            encoder.u16(*technique_points);
        }
        StandardUniverseReplayAction::Preparation { option } => {
            encoder.u8(1);
            encoder.u64(option.get());
        }
        StandardUniverseReplayAction::Battle { result } => {
            encoder.u8(2);
            encoder.bytes(&encode_battle_result_payload(result)?)?;
        }
    }
    Ok(encoder.into_inner())
}

fn decode_action(
    bytes: &[u8],
) -> Result<StandardUniverseReplayAction, StandardUniverseReplayError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if version != STANDARD_UNIVERSE_REPLAY_ACTION_VERSION {
        return Err(StandardUniverseReplayError::UnsupportedActionVersion(
            version,
        ));
    }
    let action = match decoder.u8()? {
        0 => StandardUniverseReplayAction::Decision {
            decision: ActivityDecisionId::new(decoder.u64()?)
                .ok_or(StandardUniverseReplayError::InvalidId)?,
            kind: decode_decision_kind(decoder.u8()?)?,
            option: ActivityOptionId::new(decoder.u64()?)
                .ok_or(StandardUniverseReplayError::InvalidId)?,
            technique_points: decoder.u16()?,
        },
        1 => StandardUniverseReplayAction::Preparation {
            option: ActivityOptionId::new(decoder.u64()?)
                .ok_or(StandardUniverseReplayError::InvalidId)?,
        },
        2 => StandardUniverseReplayAction::Battle {
            result: Box::new(decode_battle_result_payload(
                decoder.bytes(starclock_replay::record::MAX_RECORD_PAYLOAD_BYTES)?,
            )?),
        },
        other => return Err(StandardUniverseReplayError::UnknownAction(other)),
    };
    decoder.finish()?;
    Ok(action)
}

fn decode_decision_kind(raw: u8) -> Result<ActivityDecisionKind, StandardUniverseReplayError> {
    match raw {
        0 => Ok(ActivityDecisionKind::Choice),
        1 => Ok(ActivityDecisionKind::Route),
        2 => Ok(ActivityDecisionKind::Encounter),
        3 => Ok(ActivityDecisionKind::Preparation),
        4 => Ok(ActivityDecisionKind::Reward),
        5 => Ok(ActivityDecisionKind::Shop),
        6 => Ok(ActivityDecisionKind::Service),
        7 => Ok(ActivityDecisionKind::Roster),
        8 => Ok(ActivityDecisionKind::ExternalOutcome),
        9 => Ok(ActivityDecisionKind::BattleReady),
        10 => Ok(ActivityDecisionKind::Checkpoint),
        11 => Ok(ActivityDecisionKind::Abandon),
        other => Err(StandardUniverseReplayError::UnknownDecisionKind(other)),
    }
}

fn invalid_layout<T>(record_index: usize) -> Result<T, StandardUniverseReplayError> {
    Err(StandardUniverseReplayError::InvalidRecordLayout {
        record_index: record_index as u32,
    })
}

#[derive(Debug)]
pub enum StandardUniverseReplayError {
    Format(ReplayFormatError),
    Codec(CodecError),
    Payload(ActivityCommandPayloadError),
    Runner(StandardUniverseBaselineError),
    TooManyActions,
    CapturedBattleMismatch,
    DiagnosticMismatch,
    IdentityMismatch,
    InvalidRecordLayout {
        record_index: u32,
    },
    InvalidStateHashPayload,
    UnsupportedActionVersion(u16),
    UnknownAction(u8),
    UnknownDecisionKind(u8),
    InvalidId,
    DecisionDivergence {
        action_index: u32,
    },
    CommandRejected {
        action_index: u32,
    },
    EncounterRejected {
        action_index: u32,
        error: StandardUniverseEncounterError,
    },
    PreparationRejected {
        action_index: u32,
    },
    BattleStartRejected {
        action_index: u32,
        error: StandardUniverseBattleStartError,
    },
    BattleResultRejected {
        action_index: u32,
    },
    MissingNestedBoundary {
        action_index: u32,
    },
    NestedStartDivergence {
        action_index: u32,
        expected: Box<BattleResultIdentity>,
        actual: Box<BattleResultIdentity>,
    },
    NestedEndDivergence {
        action_index: u32,
        expected: BattleResultDigest,
        actual: BattleResultDigest,
    },
    StateDivergence {
        action_index: u32,
        expected: StateDigest,
        actual: StateDigest,
    },
    IncompleteActivity,
}

impl From<ReplayFormatError> for StandardUniverseReplayError {
    fn from(value: ReplayFormatError) -> Self {
        Self::Format(value)
    }
}
impl From<CodecError> for StandardUniverseReplayError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
impl From<ActivityCommandPayloadError> for StandardUniverseReplayError {
    fn from(value: ActivityCommandPayloadError) -> Self {
        Self::Payload(value)
    }
}

#[must_use]
pub fn replay_entry_for(activity: &StandardUniverseActivity, profile_id: &str) -> ReplayEntry {
    let identity = activity.graph().definition().identity();
    ReplayEntry::Activity {
        profile_id: profile_id.into(),
        definition_id: identity.id().get(),
        definition_digest: DefinitionDigest::new(identity.definition_digest().bytes()),
        spec_digest: EntrySpecDigest::new(identity.config_digest().bytes()),
        builds: None,
    }
}
