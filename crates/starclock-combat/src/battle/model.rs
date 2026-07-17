use crate::{
    codec::BattleStateHash, command::model::DecisionPoint, event::model::BattleEvent, id::CommandId,
};

use super::fault::BattleFault;

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resolution {
    phase: BattlePhase,
    next_decision: Option<DecisionPoint>,
    committed_revision: u64,
    rng_draw_count: u64,
    root_command: CommandId,
    events: Vec<BattleEvent>,
    state_hash: BattleStateHash,
    fault: Option<BattleFault>,
}

pub(crate) struct ResolutionBoundary {
    pub(crate) phase: BattlePhase,
    pub(crate) next_decision: Option<DecisionPoint>,
    pub(crate) committed_revision: u64,
    pub(crate) rng_draw_count: u64,
    pub(crate) root_command: CommandId,
    pub(crate) events: Vec<BattleEvent>,
    pub(crate) state_hash: BattleStateHash,
    pub(crate) fault: Option<BattleFault>,
}

impl Resolution {
    pub(crate) fn new(boundary: ResolutionBoundary) -> Self {
        Self {
            phase: boundary.phase,
            next_decision: boundary.next_decision,
            committed_revision: boundary.committed_revision,
            rng_draw_count: boundary.rng_draw_count,
            root_command: boundary.root_command,
            events: boundary.events,
            state_hash: boundary.state_hash,
            fault: boundary.fault,
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
    /// Returns the accepted command identity at every event's root.
    #[must_use]
    pub const fn root_command(&self) -> CommandId {
        self.root_command
    }
    /// Returns authoritative facts in exact emission order.
    #[must_use]
    pub fn events(&self) -> &[BattleEvent] {
        &self.events
    }
    /// Returns the streaming SHA-256 digest of committed canonical state.
    #[must_use]
    pub const fn state_hash(&self) -> BattleStateHash {
        self.state_hash
    }
    /// Returns the stable fault committed by this accepted command, if any.
    #[must_use]
    pub const fn fault(&self) -> Option<BattleFault> {
        self.fault
    }
}
