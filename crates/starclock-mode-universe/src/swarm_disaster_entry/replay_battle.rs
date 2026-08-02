//! Nested-battle replay encoding and first-divergence comparison.

use starclock_replay::{
    battle_event::encode_battle_event_payload_for_version,
    codec::{CodecError, Encoder},
    nested_battle::{
        MAX_NESTED_BATTLE_EVENTS_PER_COMMAND, NESTED_BATTLE_STATE_PAYLOAD_VERSION,
        decode_nested_battle_command_payload, decode_nested_battle_state_payload,
    },
    record::{RecordKind, RecordRef},
};

use super::{
    replay::{
        SWARM_DISASTER_REPLAY_EVENT_PAYLOAD_VERSION, SwarmReplayDivergenceKind, SwarmReplayError,
    },
    seeded_run::SwarmSeededBattleRecord,
};

pub(super) fn compare_battle(
    records: &[RecordRef<'_>],
    cursor: &mut usize,
    action_index: u32,
    battle_index: u32,
    actual: &SwarmSeededBattleRecord,
    total_commands: &mut u32,
) -> Result<(), SwarmReplayError> {
    for (command_index, step) in actual.report.trace().iter().enumerate() {
        let command_index =
            u32::try_from(command_index).map_err(|_| SwarmReplayError::TooManyRecords)?;
        let record = records
            .get(*cursor)
            .filter(|record| record.kind() == RecordKind::AcceptedBattleCommand)
            .ok_or_else(|| {
                divergence(
                    SwarmReplayDivergenceKind::BattleCommand,
                    action_index,
                    battle_index,
                    command_index,
                )
            })?;
        let command = decode_nested_battle_command_payload(record.payload()).map_err(|_| {
            divergence(
                SwarmReplayDivergenceKind::BattleCommand,
                action_index,
                battle_index,
                command_index,
            )
        })?;
        if command.controller() != step.controller() as u8 || command.command() != step.command() {
            return Err(divergence(
                SwarmReplayDivergenceKind::BattleCommand,
                action_index,
                battle_index,
                command_index,
            ));
        }
        *cursor += 1;
        let state = records
            .get(*cursor)
            .filter(|record| record.kind() == RecordKind::ExpectedBattleState)
            .ok_or_else(|| {
                divergence(
                    SwarmReplayDivergenceKind::BattleState,
                    action_index,
                    battle_index,
                    command_index,
                )
            })?;
        let decoded = decode_nested_battle_state_payload(state.payload()).map_err(|_| {
            divergence(
                SwarmReplayDivergenceKind::BattleState,
                action_index,
                battle_index,
                command_index,
            )
        })?;
        compare_events(
            decoded.event_payloads(),
            step.events(),
            action_index,
            battle_index,
            command_index,
        )?;
        if decoded.state_hash().bytes() != step.state_hash().bytes() {
            return Err(divergence(
                SwarmReplayDivergenceKind::BattleState,
                action_index,
                battle_index,
                command_index,
            ));
        }
        *cursor += 1;
        *total_commands = total_commands
            .checked_add(1)
            .ok_or(SwarmReplayError::TooManyRecords)?;
    }
    if records
        .get(*cursor)
        .is_some_and(|record| record.kind() == RecordKind::AcceptedBattleCommand)
    {
        return Err(divergence(
            SwarmReplayDivergenceKind::BattleCommand,
            action_index,
            battle_index,
            u32::try_from(actual.report.trace().len()).unwrap_or(u32::MAX),
        ));
    }
    let command_index = u32::try_from(actual.report.trace().len()).unwrap_or(u32::MAX);
    let end = records
        .get(*cursor)
        .filter(|record| record.kind() == RecordKind::NestedBattleEnd)
        .ok_or_else(|| {
            divergence(
                SwarmReplayDivergenceKind::BattleResult,
                action_index,
                battle_index,
                command_index,
            )
        })?;
    let digest = starclock_replay::activity::decode_nested_battle_end_payload(end.payload())
        .map_err(|_| {
            divergence(
                SwarmReplayDivergenceKind::BattleResult,
                action_index,
                battle_index,
                command_index,
            )
        })?;
    if digest != actual.result.actual_digest() {
        return Err(divergence(
            SwarmReplayDivergenceKind::BattleResult,
            action_index,
            battle_index,
            command_index,
        ));
    }
    *cursor += 1;
    Ok(())
}

fn compare_events(
    expected: &[&[u8]],
    actual: &[starclock_combat::BattleEvent],
    action_index: u32,
    battle_index: u32,
    command_index: u32,
) -> Result<(), SwarmReplayError> {
    if expected.len() != actual.len() {
        return Err(divergence(
            SwarmReplayDivergenceKind::Event,
            action_index,
            battle_index,
            command_index,
        ));
    }
    for (payload, event) in expected.iter().zip(actual) {
        let version = payload
            .get(..2)
            .and_then(|bytes| bytes.try_into().ok())
            .map(u16::from_le_bytes);
        if version != Some(SWARM_DISASTER_REPLAY_EVENT_PAYLOAD_VERSION)
            || payload
                != &encode_battle_event_payload_for_version(
                    event,
                    SWARM_DISASTER_REPLAY_EVENT_PAYLOAD_VERSION,
                )?
                .as_slice()
        {
            return Err(divergence(
                SwarmReplayDivergenceKind::Event,
                action_index,
                battle_index,
                command_index,
            ));
        }
    }
    Ok(())
}

pub(super) fn encode_nested_state_v5(
    state_hash: starclock_combat::BattleStateHash,
    events: &[starclock_combat::BattleEvent],
) -> Result<Vec<u8>, SwarmReplayError> {
    if events.len() > MAX_NESTED_BATTLE_EVENTS_PER_COMMAND as usize {
        return Err(SwarmReplayError::TooManyRecords);
    }
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_STATE_PAYLOAD_VERSION);
    encoder.raw(&state_hash.bytes());
    encoder.u32(u32::try_from(events.len()).map_err(|_| CodecError::LengthOverflow)?);
    for event in events {
        encoder.bytes(&encode_battle_event_payload_for_version(
            event,
            SWARM_DISASTER_REPLAY_EVENT_PAYLOAD_VERSION,
        )?)?;
    }
    Ok(encoder.into_inner())
}

fn divergence(
    kind: SwarmReplayDivergenceKind,
    action_index: u32,
    battle_index: u32,
    command_index: u32,
) -> SwarmReplayError {
    SwarmReplayError::FirstDivergence {
        kind,
        action_index,
        battle_index,
        command_index,
    }
}
