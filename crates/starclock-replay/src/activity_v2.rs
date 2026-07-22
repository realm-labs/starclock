//! Version-2 graph-Activity command payloads. Full trace orchestration is
//! layered on these stable command bytes by Goal 04 Phase 5.

use starclock_activity::{
    ActivityBattleHandoffId, ActivityDecisionId, ActivityExternalOutcomeId, ActivityOptionId,
    ActivityStateHash, GraphActivityCommand, GraphActivityCommandKind,
};

use crate::{
    activity::{ActivityCommandPayloadError, decode_result, encode_result, fixed_digest},
    codec::{Decoder, Encoder},
};

pub const GRAPH_ACTIVITY_COMMAND_PAYLOAD_VERSION: u16 = 2;

pub fn encode_graph_activity_command(
    command: &GraphActivityCommand,
) -> Result<Vec<u8>, ActivityCommandPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(GRAPH_ACTIVITY_COMMAND_PAYLOAD_VERSION);
    encoder.raw(&command.expected_state_hash().bytes());
    encoder.u64(command.decision().get());
    match command.kind() {
        GraphActivityCommandKind::ChooseOption { option } => {
            encoder.u8(0);
            encoder.u64(option.get());
        }
        GraphActivityCommandKind::StartBattle { handoff } => {
            encoder.u8(1);
            encoder.u64(handoff.get());
        }
        GraphActivityCommandKind::SubmitBattleResult { result } => {
            encoder.u8(2);
            encode_result(result, &mut encoder)?;
        }
        GraphActivityCommandKind::SubmitExternalOutcome { outcome } => {
            encoder.u8(3);
            encoder.u64(outcome.get());
        }
        GraphActivityCommandKind::Abandon => encoder.u8(4),
    }
    Ok(encoder.into_inner())
}

pub fn decode_graph_activity_command(
    bytes: &[u8],
) -> Result<GraphActivityCommand, ActivityCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if version != GRAPH_ACTIVITY_COMMAND_PAYLOAD_VERSION {
        return Err(ActivityCommandPayloadError::UnsupportedVersion(version));
    }
    let expected_state_hash = ActivityStateHash::new(fixed_digest(&mut decoder)?)
        .expect("Activity state hashes accept every byte sequence");
    let decision =
        ActivityDecisionId::new(decoder.u64()?).ok_or(ActivityCommandPayloadError::InvalidId)?;
    let kind = match decoder.u8()? {
        0 => GraphActivityCommandKind::ChooseOption {
            option: ActivityOptionId::new(decoder.u64()?)
                .ok_or(ActivityCommandPayloadError::InvalidId)?,
        },
        1 => GraphActivityCommandKind::StartBattle {
            handoff: ActivityBattleHandoffId::new(decoder.u64()?)
                .ok_or(ActivityCommandPayloadError::InvalidId)?,
        },
        2 => GraphActivityCommandKind::SubmitBattleResult {
            result: Box::new(decode_result(&mut decoder)?),
        },
        3 => GraphActivityCommandKind::SubmitExternalOutcome {
            outcome: ActivityExternalOutcomeId::new(decoder.u64()?)
                .ok_or(ActivityCommandPayloadError::InvalidId)?,
        },
        4 => GraphActivityCommandKind::Abandon,
        other => return Err(ActivityCommandPayloadError::UnknownCommand(other)),
    };
    decoder.finish()?;
    Ok(GraphActivityCommand::new(
        expected_state_hash,
        decision,
        kind,
    ))
}

#[cfg(test)]
mod tests {
    use starclock_activity::{
        ActivityConfigDigest, ActivityDefinitionDigest, ActivityInstanceId, BattleOutcome,
        BattleResult, BattleResultConfiguration, BattleResultIdentity, BattleSequence, EventDigest,
        ProjectedValue, ScopeIdentity,
    };
    use starclock_combat::{BattleSeed, BattleSpecDigest, BattleStateHash};

    use super::*;

    #[test]
    fn all_five_graph_command_kinds_round_trip_under_payload_version_two() {
        let hash = ActivityStateHash::new([0x11; 32]).unwrap();
        let decision = ActivityDecisionId::new(7).unwrap();
        let identity = BattleResultIdentity::new(
            ScopeIdentity::new(
                ActivityInstanceId::new(1).unwrap(),
                starclock_activity::SectionId::new(2).unwrap(),
                starclock_activity::NodeId::new(3).unwrap(),
                starclock_activity::AttemptId::new(4).unwrap(),
            ),
            BattleSequence::new(5).unwrap(),
            BattleResultConfiguration::new(
                ActivityDefinitionDigest::new([0x21; 32]).unwrap(),
                ActivityConfigDigest::new([0x22; 32]).unwrap(),
                starclock_activity::ParticipantLockDigest::new([0x23; 32]).unwrap(),
            ),
            BattleSpecDigest::new([0x24; 32]).unwrap(),
            BattleSeed::new([0x25; 32]),
        );
        let result = BattleResult::seal(
            identity,
            vec![
                ProjectedValue::Outcome(BattleOutcome::Won),
                ProjectedValue::FinalStateHash(BattleStateHash::from_bytes([0x31; 32])),
                ProjectedValue::EventDigest(EventDigest::new([0x32; 32]).unwrap()),
                ProjectedValue::TerminalFault(None),
            ],
        );
        let kinds = vec![
            GraphActivityCommandKind::ChooseOption {
                option: ActivityOptionId::new(8).unwrap(),
            },
            GraphActivityCommandKind::StartBattle {
                handoff: ActivityBattleHandoffId::new(9).unwrap(),
            },
            GraphActivityCommandKind::SubmitBattleResult {
                result: Box::new(result),
            },
            GraphActivityCommandKind::SubmitExternalOutcome {
                outcome: ActivityExternalOutcomeId::new(10).unwrap(),
            },
            GraphActivityCommandKind::Abandon,
        ];
        for kind in kinds {
            let command = GraphActivityCommand::new(hash, decision, kind);
            let encoded = encode_graph_activity_command(&command).unwrap();
            assert_eq!(u16::from_le_bytes(encoded[..2].try_into().unwrap()), 2);
            assert_eq!(decode_graph_activity_command(&encoded), Ok(command));
        }
    }

    #[test]
    fn malformed_payload_corpus_is_total_and_bounded() {
        let mut value = 0x9e37_79b9_u32;
        for length in 0..4_096_u32 {
            let mut bytes = Vec::with_capacity((length % 97) as usize);
            for _ in 0..length % 97 {
                value = value.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                bytes.push((value >> 24) as u8);
            }
            let _ = decode_graph_activity_command(&bytes);
        }
    }
}
