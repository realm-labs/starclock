//! Reconstruct accepted public selections, checking each boundary before the next.

use starclock_activity::{
    ActivityDecisionId, ActivityInstanceId, ActivityMasterSeed, ActivityOptionId,
    MAX_ACTIVITY_OPTIONS,
};
use starclock_replay::{format::DecodedReplay, record::RecordKind};

use super::{
    DivergentUniverseRecordedRun, DivergentUniverseReplayDivergenceKind,
    DivergentUniverseReplayError, decision_kind, divergence, record_divergence_kind,
    record_divergent_universe_transcript, replay_payloads,
};
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselineRunner,
    DivergentUniverseFlowInstance, DivergentUniverseOfferedSelection,
};

pub(super) fn reconstruct(
    decoded: &DecodedReplay<'_>,
    fixture: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
) -> Result<DivergentUniverseRecordedRun, DivergentUniverseReplayError> {
    let seed = decoded.header().master_seed();
    let mut activity = flow
        .start(
            ActivityInstanceId::new(1).expect("fixed replay instance is non-zero"),
            ActivityMasterSeed::from_u64(seed),
        )
        .map_err(|_| DivergentUniverseReplayError::ActivityStart)?
        .into_activity();
    let policy = fixture
        .policy()
        .map_err(DivergentUniverseReplayError::Fixture)?;
    let mut steps = Vec::new();
    let initial = decoded
        .records()
        .get(1)
        .ok_or_else(|| divergence(DivergentUniverseReplayDivergenceKind::RecordLayout, 1))?;
    if initial.kind() != RecordKind::ExpectedActivityState
        || initial.payload() != activity.state_hash().bytes()
    {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::Activity,
            1,
        ));
    }
    let mut cursor = 2;
    let maximum_actions = policy
        .max_steps()
        .checked_add(flow.additional_service_action_budget())
        .ok_or(DivergentUniverseReplayError::TooManyRecords)?;
    while activity.player_view().terminal().is_none() {
        let index =
            u32::try_from(cursor).map_err(|_| DivergentUniverseReplayError::TooManyRecords)?;
        let invalid = || {
            divergence(
                DivergentUniverseReplayDivergenceKind::ActivityCommand,
                index,
            )
        };
        if steps.len() >= maximum_actions as usize {
            return Err(invalid());
        }
        let record = decoded.records().get(cursor).ok_or_else(|| {
            divergence(DivergentUniverseReplayDivergenceKind::RecordLayout, index)
        })?;
        if record.kind() != RecordKind::AcceptedActivityCommand {
            return Err(divergence(
                DivergentUniverseReplayDivergenceKind::RecordLayout,
                index,
            ));
        }
        let (kind, decision, option) = parse_selection(record.payload()).ok_or_else(invalid)?;
        let view = activity.player_view();
        if !view.decision().is_some_and(|offered| {
            decision_kind(offered.kind()) == kind && offered.id() == decision
        }) {
            return Err(invalid());
        }
        let step = DivergentUniverseBaselineRunner::default()
            .advance_selected(
                fixture.factory(),
                flow,
                &mut activity,
                fixture.core(),
                &policy,
                DivergentUniverseOfferedSelection::new(decision, option),
            )
            .map_err(|_| invalid())?;
        // Scores, nested battle commands/events and state hashes are regenerated,
        // not trusted. Stop at the earliest mismatching record before advancing.
        for (kind, payload) in replay_payloads(std::slice::from_ref(&step))? {
            let index =
                u32::try_from(cursor).map_err(|_| DivergentUniverseReplayError::TooManyRecords)?;
            let expected = decoded.records().get(cursor).ok_or_else(|| {
                divergence(DivergentUniverseReplayDivergenceKind::RecordLayout, index)
            })?;
            if expected.kind() != kind || expected.payload() != payload {
                return Err(divergence(
                    record_divergence_kind(expected.kind(), expected.payload(), &payload),
                    index,
                ));
            }
            cursor += 1;
        }
        steps.push(step);
    }
    if cursor != decoded.records().len() {
        return Err(divergence(
            DivergentUniverseReplayDivergenceKind::RecordLayout,
            u32::try_from(cursor).map_err(|_| DivergentUniverseReplayError::TooManyRecords)?,
        ));
    }
    record_divergent_universe_transcript(fixture, flow, &activity, seed, steps)
}

fn parse_selection(payload: &[u8]) -> Option<(u8, ActivityDecisionId, ActivityOptionId)> {
    if payload.get(..4)? != b"DUA1" {
        return None;
    }
    let kind = *payload.get(4)?;
    let decision =
        ActivityDecisionId::new(u64::from_le_bytes(payload.get(5..13)?.try_into().ok()?))?;
    let option = ActivityOptionId::new(u64::from_le_bytes(payload.get(13..21)?.try_into().ok()?))?;
    let scores = u32::from_le_bytes(payload.get(21..25)?.try_into().ok()?);
    // The current Activity offer contract permits at most 256 options. Validate
    // the full shape before selection; regenerated scores are checked afterward.
    if scores == 0
        || scores as usize > MAX_ACTIVITY_OPTIONS
        || payload.len() != 25 + scores as usize * 28
    {
        return None;
    }
    Some((kind, decision, option))
}
