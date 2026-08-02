//! Standard Universe trace capture and action payload codec.

use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityOptionId, ActivityStateHash, BattleResult,
};
use starclock_replay::{
    activity::{
        ActivityCommandPayloadError, ControllerDecisionKind, ControllerDiagnostic,
        ControllerOptionScore, decode_battle_result_payload, encode_battle_result_payload,
    },
    codec::{CodecError, Decoder, Encoder},
};

use crate::{
    baseline_controller::ActivityBaselineDecision,
    baseline_runner::{
        NestedBattleExecutor, StandardUniverseBaselineError, StandardUniverseBaselinePolicy,
        StandardUniverseBaselineReport, StandardUniverseBaselineRunner,
        StandardUniverseBaselineStep,
    },
    runtime::StandardUniverseActivity,
};

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
    recorded_from_report(report, policy, capture.results)
}

pub(crate) fn recorded_from_report(
    report: StandardUniverseBaselineReport,
    policy: &StandardUniverseBaselinePolicy,
    results: Vec<BattleResult>,
) -> Result<RecordedStandardUniverseRun, StandardUniverseReplayError> {
    if report.steps().len() > MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS as usize {
        return Err(StandardUniverseReplayError::TooManyActions);
    }
    let mut results = results.into_iter();
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
                if result.identity() != **identity || result.actual_digest() != *result_digest {
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
    fn execute(
        &mut self,
        handoff: &starclock_activity::ActivityBattleHandoff,
    ) -> Result<BattleResult, crate::baseline_runner::NestedBattleExecutionError> {
        let result = self.inner.execute(handoff)?;
        self.results.push(result.clone());
        Ok(result)
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

pub(crate) fn encode_action(
    action: &StandardUniverseReplayAction,
) -> Result<Vec<u8>, StandardUniverseReplayError> {
    let mut encoder = Encoder::new(Vec::new());
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

pub(crate) fn decode_action(
    bytes: &[u8],
) -> Result<StandardUniverseReplayAction, StandardUniverseReplayError> {
    let mut decoder = Decoder::new(bytes);
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

pub(crate) fn decode_decision_kind(
    raw: u8,
) -> Result<ActivityDecisionKind, StandardUniverseReplayError> {
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

#[derive(Debug)]
pub enum StandardUniverseReplayError {
    Codec(CodecError),
    Payload(ActivityCommandPayloadError),
    Runner(StandardUniverseBaselineError),
    TooManyActions,
    CapturedBattleMismatch,
    DiagnosticMismatch,
    UnknownAction(u8),
    UnknownDecisionKind(u8),
    InvalidId,
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
