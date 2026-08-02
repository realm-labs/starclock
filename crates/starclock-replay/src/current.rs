//! Current component-addressed activity replay envelope.
//!
//! The project supports only bytes produced by the current tree. Callers do
//! not select a format generation or compatibility revision.

pub use crate::format_v2::ReplayCompatibilityV2 as ReplayCompatibility;
pub use crate::format_v3::{
    DecodedReplayV3 as DecodedReplay, ReplayHeaderV3 as ReplayHeader, ReplayV3Error as ReplayError,
    decode_replay_v3 as decode_replay, encode_replay_v3 as encode_replay,
};
