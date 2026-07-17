use core::fmt;

use crate::codec::{CanonicalEncode, CanonicalSink, CodecError, Encoder};

/// Maximum payload accepted before allocation or domain decoding.
pub const MAX_RECORD_PAYLOAD_BYTES: u32 = 16 * 1024 * 1024;
/// Maximum number of records declared by one replay.
pub const MAX_REPLAY_RECORDS: u32 = 1_000_000;

/// Closed record families reserved by replay format version 1.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RecordKind {
    /// One accepted low-level battle command.
    AcceptedBattleCommand = 1,
    /// Expected state hash after a battle command.
    ExpectedBattleState = 2,
    /// Start of one activity-owned nested battle.
    NestedBattleStart = 3,
    /// Terminal result of one activity-owned nested battle.
    NestedBattleEnd = 4,
    /// One accepted activity command.
    AcceptedActivityCommand = 5,
    /// Expected state hash after an activity command.
    ExpectedActivityState = 6,
    /// Optional non-authoritative controller diagnostics.
    ControllerDiagnostic = 7,
}

impl TryFrom<u8> for RecordKind {
    type Error = ReplayFormatError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::AcceptedBattleCommand),
            2 => Ok(Self::ExpectedBattleState),
            3 => Ok(Self::NestedBattleStart),
            4 => Ok(Self::NestedBattleEnd),
            5 => Ok(Self::AcceptedActivityCommand),
            6 => Ok(Self::ExpectedActivityState),
            7 => Ok(Self::ControllerDiagnostic),
            other => Err(ReplayFormatError::UnknownRecordKind(other)),
        }
    }
}

/// Version-1 policy: every unknown record kind is a hard failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UnknownRecordPolicy {
    /// Reject instead of skipping or guessing a payload.
    Reject = 0,
}

/// Borrowed validated record envelope; payload decoding belongs to later batches.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordRef<'a> {
    kind: RecordKind,
    sequence: u64,
    payload: &'a [u8],
}

impl<'a> RecordRef<'a> {
    /// Creates a record after applying format limits.
    pub fn new(
        kind: RecordKind,
        sequence: u64,
        payload: &'a [u8],
    ) -> Result<Self, ReplayFormatError> {
        if payload.len() > MAX_RECORD_PAYLOAD_BYTES as usize {
            return Err(ReplayFormatError::RecordTooLarge);
        }
        Ok(Self {
            kind,
            sequence,
            payload,
        })
    }
    /// Returns the closed record kind.
    #[must_use]
    pub const fn kind(self) -> RecordKind {
        self.kind
    }
    /// Returns the zero-based canonical record sequence.
    #[must_use]
    pub const fn sequence(self) -> u64 {
        self.sequence
    }
    /// Returns borrowed payload bytes.
    #[must_use]
    pub const fn payload(self) -> &'a [u8] {
        self.payload
    }
}

impl CanonicalEncode for RecordRef<'_> {
    fn encode<S: CanonicalSink>(&self, encoder: &mut Encoder<S>) -> Result<(), CodecError> {
        encoder.u8(self.kind as u8);
        encoder.u64(self.sequence);
        encoder.bytes(self.payload)
    }
}

/// Stable replay framing/version failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayFormatError {
    /// File magic is not `SCRP`.
    InvalidMagic,
    /// The replay format version is unsupported.
    UnsupportedFormatVersion(u32),
    /// The schema version is unsupported.
    UnsupportedSchemaVersion(u32),
    /// Version 1 rejects every unknown record kind.
    UnknownRecordKind(u8),
    /// The replay entry kind is unsupported.
    UnknownEntryKind(u8),
    /// Entry definition IDs are fixed-width and non-zero.
    InvalidEntryDefinition,
    /// Unknown-record policy is unsupported.
    UnknownRecordPolicy(u8),
    /// A record exceeds the preallocation limit.
    RecordTooLarge,
    /// Record count exceeds the service/library limit.
    TooManyRecords,
    /// Record sequence is not exact zero-based order.
    InvalidRecordSequence,
    /// Canonical field decoding failed.
    Codec(CodecError),
}

impl From<CodecError> for ReplayFormatError {
    fn from(value: CodecError) -> Self {
        Self::Codec(value)
    }
}

impl fmt::Display for ReplayFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "replay format error: {self:?}")
    }
}

impl std::error::Error for ReplayFormatError {}
