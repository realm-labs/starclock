//! Non-authoritative measurements exposed only to the versioned benchmark harness.

/// Structural measurements for the latest accepted command boundary.
///
/// These values are excluded from canonical state and compatibility. Enabling
/// this feature must not change commands, events, RNG draws or hashes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BattlePerformanceSnapshot {
    canonical_state_bytes: u64,
    journal_entries: u64,
    event_entries: u64,
    operation_allocations: u64,
    journal_retained_bytes: u64,
}

impl BattlePerformanceSnapshot {
    pub(crate) const fn new(
        canonical_state_bytes: u64,
        journal_entries: u64,
        event_entries: u64,
        operation_allocations: u64,
        journal_retained_bytes: u64,
    ) -> Self {
        Self {
            canonical_state_bytes,
            journal_entries,
            event_entries,
            operation_allocations,
            journal_retained_bytes,
        }
    }

    /// Canonical bytes semantically copied into reusable scratch for a command.
    #[must_use]
    pub const fn semantic_state_copy_bytes(self) -> u64 {
        self.canonical_state_bytes
    }

    /// Exact journal entries produced by the latest accepted command.
    #[must_use]
    pub const fn journal_entries(self) -> u64 {
        self.journal_entries
    }

    /// Event facts recorded in that journal.
    #[must_use]
    pub const fn event_entries(self) -> u64 {
        self.event_entries
    }

    /// Operation identities allocated by that command.
    #[must_use]
    pub const fn operation_allocations(self) -> u64 {
        self.operation_allocations
    }

    /// Bytes retained by the bounded journal allocation after settlement.
    #[must_use]
    pub const fn journal_retained_bytes(self) -> u64 {
        self.journal_retained_bytes
    }
}
