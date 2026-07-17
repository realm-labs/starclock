use crate::command::model::DecisionPoint;

/// Top-level battle lifecycle state. `Resolving` is never externally suspended.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BattlePhase {
    /// Only the offered start command is legal.
    Initializing = 0,
    /// The controller may select one command from the current decision.
    AwaitingCommand = 1,
    /// Accepted work is draining synchronously inside `Battle::apply`.
    Resolving = 2,
    /// Player victory; terminal.
    Won = 3,
    /// Player loss or authorized concession; terminal.
    Lost = 4,
    /// Deterministic internal fault; terminal for the rules revision.
    Faulted = 5,
}

impl BattlePhase {
    /// Returns whether no further external command can be accepted.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Won | Self::Lost | Self::Faulted)
    }
}

/// Owned result of one accepted command at a stable decision/terminal boundary.
///
/// Events, cause records and the canonical state digest are added by the
/// transaction batch; all fields remain private so that expansion is additive.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    phase: BattlePhase,
    next_decision: Option<DecisionPoint>,
    committed_revision: u64,
    rng_draw_count: u64,
}

impl Resolution {
    pub(crate) fn new(
        phase: BattlePhase,
        next_decision: Option<DecisionPoint>,
        committed_revision: u64,
        rng_draw_count: u64,
    ) -> Self {
        Self {
            phase,
            next_decision,
            committed_revision,
            rng_draw_count,
        }
    }

    /// Returns the committed lifecycle phase.
    #[must_use]
    pub const fn phase(&self) -> BattlePhase {
        self.phase
    }
    /// Returns the next offered decision, or `None` at a terminal boundary.
    #[must_use]
    pub const fn next_decision(&self) -> Option<&DecisionPoint> {
        self.next_decision.as_ref()
    }
    /// Returns the monotonic count of accepted command commits.
    #[must_use]
    pub const fn committed_revision(&self) -> u64 {
        self.committed_revision
    }
    /// Returns the authoritative raw RNG draw count after the commit.
    #[must_use]
    pub const fn rng_draw_count(&self) -> u64 {
        self.rng_draw_count
    }
}
