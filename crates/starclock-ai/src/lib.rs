//! Deterministic controller boundary for combat decisions.
//!
//! Controllers consume immutable views and offered commands; they never gain
//! mutable access to a battle aggregate.

#![forbid(unsafe_code)]

mod controller;
mod select;
#[cfg(test)]
mod tests;

/// Deterministic baseline player scoring and diagnostics.
pub mod baseline;

use starclock_combat::{
    AiCandidateId, AiGraphId, AiStateId, Command, UnitId,
    rng::{engine::DeterministicRng, types::DrawSample},
};
use std::collections::BTreeMap;

/// Stable failure category produced without mutating combat state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyDecisionError {
    MissingState,
    AutomaticTransitionBudget,
    NoLegalFallback,
    InvalidWeightedGroup,
    NoTargetFault,
    SkipActionUnsupported,
    Random,
}

/// Auditable authored choice returned alongside the exact offered command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EnemyDecision {
    state: AiStateId,
    candidate: Option<AiCandidateId>,
    command: Command,
    draw: Option<DrawSample>,
}

impl EnemyDecision {
    #[must_use]
    pub const fn state(&self) -> AiStateId {
        self.state
    }
    #[must_use]
    pub const fn candidate(&self) -> Option<AiCandidateId> {
        self.candidate
    }
    #[must_use]
    pub const fn command(&self) -> &Command {
        &self.command
    }
    #[must_use]
    pub const fn draw(&self) -> Option<DrawSample> {
        self.draw
    }
}

/// Isolated deterministic authored-enemy controller stream.
pub struct EnemyController {
    rng: DeterministicRng,
    cursors: BTreeMap<UnitId, EnemyCursor>,
}

#[derive(Clone, Copy)]
struct EnemyCursor {
    graph: AiGraphId,
    state: AiStateId,
    turns: u16,
}
