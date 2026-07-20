mod state;

#[cfg(feature = "benchmark-instrumentation")]
pub(crate) use state::canonical_state_len;
pub(crate) use state::hash_state;
#[cfg(test)]
pub(crate) use state::{collect_state, hash_collected_state};

/// SHA-256 digest of the complete canonical battle state at one boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BattleStateHash([u8; 32]);

impl BattleStateHash {
    /// Reconstructs an exact battle-state hash received through a verified
    /// replay or activity result envelope.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) const fn new(bytes: [u8; 32]) -> Self {
        Self::from_bytes(bytes)
    }

    /// Returns the exact canonical digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}
