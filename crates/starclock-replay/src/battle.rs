//! Low-level battle command/hash payloads and one-shot replay verification.

use core::fmt;

use starclock_combat::{
    AbilityId, Battle, BattlePhase, BattleStateHash, Command, CommandErrorKind, DecisionId, UnitId,
};

use crate::{
    codec::{CodecError, Decoder, Encoder},
    component::ConfigurationComponentSet,
    digest::StateDigest,
    entry::ReplayEntry,
    format::{DecodedReplay, ReplayError, ReplayHeader, decode_replay, encode_replay},
    record::{MAX_REPLAY_RECORDS, RecordKind, RecordRef, ReplayFormatError},
};

/// Domain payload tag inside `AcceptedBattleCommand` records.
pub const BATTLE_COMMAND_PAYLOAD_TAG: u16 = 1;

/// One accepted command and the resulting full canonical state hash.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleTraceEntry {
    command: Command,
    state_hash: BattleStateHash,
}

impl BattleTraceEntry {
    /// Captures one successfully applied command boundary.
    #[must_use]
    pub const fn new(command: Command, state_hash: BattleStateHash) -> Self {
        Self {
            command,
            state_hash,
        }
    }
    /// Returns the exact accepted command.
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
    /// Returns the resulting authoritative state hash.
    #[must_use]
    pub const fn state_hash(&self) -> BattleStateHash {
        self.state_hash
    }
}

/// Computes the exact alternating command/hash record count.
pub fn battle_record_count(command_count: usize) -> Result<u32, ReplayFormatError> {
    let records = command_count
        .checked_mul(2)
        .ok_or(ReplayFormatError::TooManyRecords)?;
    let records = u32::try_from(records).map_err(|_| ReplayFormatError::TooManyRecords)?;
    if records > MAX_REPLAY_RECORDS {
        Err(ReplayFormatError::TooManyRecords)
    } else {
        Ok(records)
    }
}

/// Encodes an alternating accepted-command/expected-state trace.
pub fn encode_battle_trace(
    header: &ReplayHeader,
    trace: &[BattleTraceEntry],
) -> Result<Vec<u8>, BattleReplayError> {
    let expected = battle_record_count(trace.len())?;
    if header.record_count() != expected {
        return Err(ReplayFormatError::InvalidRecordSequence.into());
    }
    let mut payloads = Vec::with_capacity(expected as usize);
    for entry in trace {
        payloads.push(encode_battle_command_payload(entry.command())?);
        payloads.push(entry.state_hash().bytes().to_vec());
    }
    let mut records = Vec::with_capacity(payloads.len());
    for (sequence, payload) in payloads.iter().enumerate() {
        let kind = if sequence % 2 == 0 {
            RecordKind::AcceptedBattleCommand
        } else {
            RecordKind::ExpectedBattleState
        };
        records.push(RecordRef::new(kind, sequence as u64, payload)?);
    }
    encode_replay(header, &records, Vec::new()).map_err(Into::into)
}

/// Successful one-shot verification summary with bounded retained state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BattleReplayReport {
    command_count: u32,
    final_hash: StateDigest,
    phase: BattlePhase,
}

impl BattleReplayReport {
    /// Returns the number of applied and hash-compared commands.
    #[must_use]
    pub const fn command_count(self) -> u32 {
        self.command_count
    }
    /// Returns the final verified canonical state digest.
    #[must_use]
    pub const fn final_hash(self) -> StateDigest {
        self.final_hash
    }
    /// Returns the final battle phase.
    #[must_use]
    pub const fn phase(self) -> BattlePhase {
        self.phase
    }
}

/// Verifies exact identity, then applies the accepted stream once.
pub fn verify_battle_replay(
    bytes: &[u8],
    mut battle: Battle,
    expected_components: &ConfigurationComponentSet,
) -> Result<BattleReplayReport, BattleReplayError> {
    let replay = decode_replay(bytes)?;
    validate_identity(&replay, &battle, expected_components)?;
    if replay.records().is_empty() || replay.records().len() % 2 != 0 {
        return Err(BattleReplayError::InvalidRecordLayout);
    }
    let mut final_hash = StateDigest::new(battle.state_hash().bytes());
    for (index, pair) in replay.records().chunks_exact(2).enumerate() {
        if pair[0].kind() != RecordKind::AcceptedBattleCommand
            || pair[1].kind() != RecordKind::ExpectedBattleState
        {
            return Err(BattleReplayError::InvalidRecordLayout);
        }
        let command = decode_battle_command_payload(pair[0].payload())?;
        let resolution =
            battle
                .apply(command)
                .map_err(|error| BattleReplayError::CommandRejected {
                    command_index: index as u32,
                    kind: error.kind(),
                })?;
        let expected: [u8; 32] = pair[1]
            .payload()
            .try_into()
            .map_err(|_| BattleReplayError::InvalidStateHashPayload)?;
        let actual = resolution.state_hash().bytes();
        if expected != actual {
            return Err(BattleReplayError::StateDivergence {
                command_index: index as u32,
                expected: StateDigest::new(expected),
                actual: StateDigest::new(actual),
            });
        }
        final_hash = StateDigest::new(actual);
    }
    Ok(BattleReplayReport {
        command_count: u32::try_from(replay.records().len() / 2)
            .expect("replay record bound fits u32 command count"),
        final_hash,
        phase: battle.view().phase(),
    })
}

fn validate_identity(
    replay: &DecodedReplay<'_>,
    battle: &Battle,
    expected_components: &ConfigurationComponentSet,
) -> Result<(), BattleReplayError> {
    let header = replay.header();
    let view = battle.view();
    let battle_identity = view.identity();
    if header
        .components()
        .verify_exact(expected_components)
        .is_err()
    {
        return Err(BattleReplayError::IdentityMismatch(
            BattleIdentityField::Components,
        ));
    }
    match header.entry() {
        ReplayEntry::Battle {
            definition_id,
            spec_digest,
        } if *definition_id == view.encounter().definition().get()
            && spec_digest.bytes() == battle_identity.spec_digest().bytes() =>
        {
            Ok(())
        }
        ReplayEntry::Battle { .. } => Err(BattleReplayError::IdentityMismatch(
            BattleIdentityField::Entry,
        )),
        ReplayEntry::Activity { .. } => Err(BattleReplayError::NotBattleReplay),
    }
}

pub fn encode_battle_command_payload(
    command: &Command,
) -> Result<Vec<u8>, BattleCommandPayloadError> {
    let mut encoder = Encoder::new(Vec::new());
    encoder.u16(BATTLE_COMMAND_PAYLOAD_TAG);
    match command {
        Command::StartBattle { decision } => {
            encoder.u8(0);
            encoder.u64(decision.get());
        }
        Command::UseAbility {
            decision,
            actor,
            ability,
            primary_target,
        } => encode_action_command(
            &mut encoder,
            1,
            *decision,
            *actor,
            *ability,
            *primary_target,
        ),
        Command::UseInterrupt {
            decision,
            actor,
            ability,
            primary_target,
        } => encode_action_command(
            &mut encoder,
            2,
            *decision,
            *actor,
            *ability,
            *primary_target,
        ),
        Command::PassInterruptWindow { decision } => {
            encoder.u8(3);
            encoder.u64(decision.get());
        }
        Command::Concede { decision } => {
            encoder.u8(4);
            encoder.u64(decision.get());
        }
    }
    Ok(encoder.into_inner())
}

fn encode_action_command(
    encoder: &mut Encoder<Vec<u8>>,
    kind: u8,
    decision: DecisionId,
    actor: UnitId,
    ability: AbilityId,
    primary_target: Option<UnitId>,
) {
    encoder.u8(kind);
    encoder.u64(decision.get());
    encoder.u64(actor.get());
    encoder.u32(ability.get());
    encoder.boolean(primary_target.is_some());
    if let Some(target) = primary_target {
        encoder.u64(target.get());
    }
}

pub fn decode_battle_command_payload(bytes: &[u8]) -> Result<Command, BattleCommandPayloadError> {
    let mut decoder = Decoder::new(bytes);
    let tag = decoder.u16()?;
    if tag != BATTLE_COMMAND_PAYLOAD_TAG {
        return Err(BattleCommandPayloadError::UnexpectedTag(tag));
    }
    let command = match decoder.u8()? {
        0 => Command::StartBattle {
            decision: runtime_id(decoder.u64()?)?,
        },
        1 => decode_action_command(&mut decoder, false)?,
        2 => decode_action_command(&mut decoder, true)?,
        3 => Command::PassInterruptWindow {
            decision: runtime_id(decoder.u64()?)?,
        },
        4 => Command::Concede {
            decision: runtime_id(decoder.u64()?)?,
        },
        value => return Err(BattleCommandPayloadError::UnknownCommand(value)),
    };
    decoder.finish()?;
    Ok(command)
}

fn decode_action_command(
    decoder: &mut Decoder<'_>,
    interrupt: bool,
) -> Result<Command, BattleCommandPayloadError> {
    let decision = runtime_id(decoder.u64()?)?;
    let actor = runtime_id(decoder.u64()?)?;
    let ability = AbilityId::new(decoder.u32()?).ok_or(BattleCommandPayloadError::InvalidId)?;
    let primary_target = match decoder.boolean()? {
        false => None,
        true => Some(runtime_id(decoder.u64()?)?),
    };
    Ok(if interrupt {
        Command::UseInterrupt {
            decision,
            actor,
            ability,
            primary_target,
        }
    } else {
        Command::UseAbility {
            decision,
            actor,
            ability,
            primary_target,
        }
    })
}

fn runtime_id<I>(raw: u64) -> Result<I, BattleCommandPayloadError>
where
    I: TryFrom<u64>,
{
    I::try_from(raw).map_err(|_| BattleCommandPayloadError::InvalidId)
}

/// Stable command-payload decoding failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleCommandPayloadError {
    /// The payload tag is not recognized.
    UnexpectedTag(u16),
    /// The command discriminant is not part of the closed combat command family.
    UnknownCommand(u8),
    /// A fixed-width definition/runtime ID was zero or outside its domain.
    InvalidId,
    /// Canonical primitive framing failed.
    Codec(CodecError),
}

impl From<CodecError> for BattleCommandPayloadError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

/// Identity field that rejected replay execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BattleIdentityField {
    Components,
    Entry,
}

/// Stable low-level battle replay construction or verification failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BattleReplayError {
    /// Envelope/framing validation failed.
    Format(ReplayError),
    /// A command payload was malformed or incompatible.
    CommandPayload(BattleCommandPayloadError),
    /// The envelope contains an activity rather than a low-level battle entry.
    NotBattleReplay,
    /// One required identity field differs from the supplied battle.
    IdentityMismatch(BattleIdentityField),
    /// Records are not non-empty alternating command/hash pairs.
    InvalidRecordLayout,
    /// An expected-state payload is not exactly 32 bytes.
    InvalidStateHashPayload,
    /// A recorded accepted command is not legal at the reproduced boundary.
    CommandRejected {
        command_index: u32,
        kind: CommandErrorKind,
    },
    /// The first mismatching command boundary and both exact digests.
    StateDivergence {
        command_index: u32,
        expected: StateDigest,
        actual: StateDigest,
    },
}

impl From<ReplayFormatError> for BattleReplayError {
    fn from(value: ReplayFormatError) -> Self {
        Self::Format(ReplayError::Format(value))
    }
}

impl From<ReplayError> for BattleReplayError {
    fn from(value: ReplayError) -> Self {
        Self::Format(value)
    }
}

impl From<BattleCommandPayloadError> for BattleReplayError {
    fn from(value: BattleCommandPayloadError) -> Self {
        Self::CommandPayload(value)
    }
}

impl fmt::Display for BattleReplayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "battle replay error: {self:?}")
    }
}

impl std::error::Error for BattleReplayError {}
