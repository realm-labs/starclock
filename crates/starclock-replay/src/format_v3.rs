//! Component-addressed replay v3 envelope.
//!
//! V3 deliberately retains the bounded v2 header/record schema while assigning
//! a new format version to the dual combat-input/assembly identity contract.
//! Payload families define their own revisions and v3 rejects unknown records.

use crate::{
    codec::CanonicalSink,
    format_v2::{
        DecodedReplayV2, ReplayCompatibilityV2, ReplayHeaderV2, ReplayV2Error,
        decode_replay_with_version, encode_replay_with_version,
    },
    record::RecordRef,
};

pub const REPLAY_FORMAT_VERSION_V3: u32 = 3;
pub const REPLAY_SCHEMA_VERSION_V3: u32 = 1;

pub type ReplayCompatibilityV3 = ReplayCompatibilityV2;
pub type ReplayHeaderV3 = ReplayHeaderV2;
pub type DecodedReplayV3<'a> = DecodedReplayV2<'a>;
pub type ReplayV3Error = ReplayV2Error;

pub fn encode_replay_v3<S: CanonicalSink>(
    header: &ReplayHeaderV3,
    records: &[RecordRef<'_>],
    sink: S,
) -> Result<S, ReplayV3Error> {
    encode_replay_with_version(header, records, sink, REPLAY_FORMAT_VERSION_V3)
}

pub fn decode_replay_v3(bytes: &[u8]) -> Result<DecodedReplayV3<'_>, ReplayV3Error> {
    decode_replay_with_version(bytes, REPLAY_FORMAT_VERSION_V3)
}
