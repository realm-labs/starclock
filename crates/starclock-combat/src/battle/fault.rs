/// State boundary selected by a subsystem when deterministic resolution fails.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FaultPolicy {
    /// Discard uncommitted work and commit a fault derived from the prior state.
    Rollback = 0,
    /// Preserve completed atomic operations and append the fault boundary.
    CommitFault = 1,
}

/// Stable internal failure category with no platform error text.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FaultKind {
    /// A checked numeric operation could not produce a domain value.
    Numeric = 0,
    /// A rules-revision resolution budget was exhausted.
    BudgetExceeded = 1,
    /// An authoritative state or transaction invariant failed.
    InvariantViolation = 2,
    /// A fixed-width monotonic identity or revision was exhausted.
    SequenceExhausted = 3,
}

/// Stable resolver boundary at which a fault was detected.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FaultBoundary {
    /// Preparing or allocating transaction-local state.
    Transaction = 0,
    /// Resolving the accepted command.
    Command = 1,
    /// Verifying the final state before commit.
    Commit = 2,
}

/// Deterministic terminal failure committed into battle state and events.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BattleFault {
    kind: FaultKind,
    boundary: FaultBoundary,
    policy: FaultPolicy,
    context_code: u32,
    numeric_context: Option<i64>,
}

impl BattleFault {
    /// Reconstructs an exact fault received through a verified replay or
    /// activity-result transport.
    #[must_use]
    pub const fn from_parts(
        kind: FaultKind,
        boundary: FaultBoundary,
        policy: FaultPolicy,
        context_code: u32,
        numeric_context: Option<i64>,
    ) -> Self {
        Self {
            kind,
            boundary,
            policy,
            context_code,
            numeric_context,
        }
    }

    pub(crate) const fn new(
        kind: FaultKind,
        boundary: FaultBoundary,
        policy: FaultPolicy,
        context_code: u32,
        numeric_context: Option<i64>,
    ) -> Self {
        Self::from_parts(kind, boundary, policy, context_code, numeric_context)
    }

    /// Returns the stable failure category.
    #[must_use]
    pub const fn kind(self) -> FaultKind {
        self.kind
    }
    /// Returns the resolver boundary that selected the failure.
    #[must_use]
    pub const fn boundary(self) -> FaultBoundary {
        self.boundary
    }
    /// Returns whether earlier working mutations were discarded or committed.
    #[must_use]
    pub const fn policy(self) -> FaultPolicy {
        self.policy
    }
    /// Returns a subsystem-owned stable context discriminator.
    #[must_use]
    pub const fn context_code(self) -> u32 {
        self.context_code
    }
    /// Returns optional exact numeric context, never formatted platform text.
    #[must_use]
    pub const fn numeric_context(self) -> Option<i64> {
        self.numeric_context
    }
}
