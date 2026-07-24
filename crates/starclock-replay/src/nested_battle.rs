//! Component-replay payloads for commands and payload-direct event boundaries.

use starclock_combat::{BattleEvent, BattleStateHash, Command};

use crate::{
    battle::{
        BattleCommandPayloadError, decode_battle_command_payload, encode_battle_command_payload,
    },
    battle_event::{BattleEventPayloadError, encode_battle_event_payload},
    codec::{CodecError, Decoder, Encoder},
    digest::StateDigest,
    record::MAX_RECORD_PAYLOAD_BYTES,
};

pub const NESTED_BATTLE_COMMAND_PAYLOAD_VERSION: u16 = 1;
pub const NESTED_BATTLE_STATE_PAYLOAD_VERSION: u16 = 1;
pub const MAX_NESTED_BATTLE_EVENTS_PER_COMMAND: u32 = 1_000_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedBattleCommandPayload {
    controller: u8,
    command: Command,
}

impl NestedBattleCommandPayload {
    #[must_use]
    pub const fn new(controller: u8, command: Command) -> Self {
        Self {
            controller,
            command,
        }
    }
    #[must_use]
    pub const fn controller(&self) -> u8 {
        self.controller
    }
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
}

pub fn encode_nested_battle_command_payload(
    value: &NestedBattleCommandPayload,
) -> Result<Vec<u8>, NestedBattlePayloadError> {
    let command = encode_battle_command_payload(value.command())?;
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_COMMAND_PAYLOAD_VERSION);
    encoder.u8(value.controller);
    encoder.bytes(&command)?;
    Ok(encoder.into_inner())
}

pub fn decode_nested_battle_command_payload(
    bytes: &[u8],
) -> Result<NestedBattleCommandPayload, NestedBattlePayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if version != NESTED_BATTLE_COMMAND_PAYLOAD_VERSION {
        return Err(NestedBattlePayloadError::UnsupportedCommandVersion(version));
    }
    let controller = decoder.u8()?;
    let command = decode_battle_command_payload(decoder.bytes(MAX_RECORD_PAYLOAD_BYTES)?)?;
    decoder.finish()?;
    Ok(NestedBattleCommandPayload::new(controller, command))
}

pub fn encode_nested_battle_state_payload(
    state_hash: BattleStateHash,
    events: &[BattleEvent],
) -> Result<Vec<u8>, NestedBattlePayloadError> {
    if events.len() > MAX_NESTED_BATTLE_EVENTS_PER_COMMAND as usize {
        return Err(NestedBattlePayloadError::TooManyEvents);
    }
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(NESTED_BATTLE_STATE_PAYLOAD_VERSION);
    encoder.raw(&state_hash.bytes());
    encoder.u32(u32::try_from(events.len()).map_err(|_| CodecError::LengthOverflow)?);
    for event in events {
        encoder.bytes(&encode_battle_event_payload(event)?)?;
    }
    Ok(encoder.into_inner())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedNestedBattleState<'a> {
    state_hash: StateDigest,
    event_payloads: Box<[&'a [u8]]>,
}

impl<'a> DecodedNestedBattleState<'a> {
    #[must_use]
    pub const fn state_hash(&self) -> StateDigest {
        self.state_hash
    }
    #[must_use]
    pub fn event_payloads(&self) -> &[&'a [u8]] {
        &self.event_payloads
    }
}

pub fn decode_nested_battle_state_payload(
    bytes: &[u8],
) -> Result<DecodedNestedBattleState<'_>, NestedBattlePayloadError> {
    let mut decoder = Decoder::new(bytes);
    let version = decoder.u16()?;
    if version != NESTED_BATTLE_STATE_PAYLOAD_VERSION {
        return Err(NestedBattlePayloadError::UnsupportedStateVersion(version));
    }
    let state_hash = StateDigest::new(
        decoder
            .take(32)?
            .try_into()
            .expect("fixed-length state digest"),
    );
    let count = decoder.u32()?;
    if count > MAX_NESTED_BATTLE_EVENTS_PER_COMMAND {
        return Err(NestedBattlePayloadError::TooManyEvents);
    }
    let mut event_payloads = Vec::with_capacity(count as usize);
    for _ in 0..count {
        event_payloads.push(decoder.bytes(MAX_RECORD_PAYLOAD_BYTES)?);
    }
    decoder.finish()?;
    Ok(DecodedNestedBattleState {
        state_hash,
        event_payloads: event_payloads.into_boxed_slice(),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NestedBattlePayloadError {
    Codec(CodecError),
    Command(BattleCommandPayloadError),
    Event(BattleEventPayloadError),
    UnsupportedCommandVersion(u16),
    UnsupportedStateVersion(u16),
    TooManyEvents,
}

impl From<CodecError> for NestedBattlePayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}
impl From<BattleCommandPayloadError> for NestedBattlePayloadError {
    fn from(value: BattleCommandPayloadError) -> Self {
        Self::Command(value)
    }
}
impl From<BattleEventPayloadError> for NestedBattlePayloadError {
    fn from(value: BattleEventPayloadError) -> Self {
        Self::Event(value)
    }
}
