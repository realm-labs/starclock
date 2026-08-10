//! Executable trigger phase/observation-point matrix.

use super::model::{RuleEventPoint, TriggerPhase};

impl RuleEventPoint {
    /// Runtime phases owned by the committed-event dispatcher.
    ///
    /// Prospective `Replace` observations use `evaluate_replacement_program`
    /// and require a typed operation consumer, so they are not a post-commit
    /// phase.
    #[must_use]
    pub const fn runtime_phases(self) -> &'static [TriggerPhase] {
        use RuleEventPoint as P;
        use TriggerPhase as T;
        const POST: &[T] = &[T::AfterMutation, T::AfterEvent];
        const START: &[T] = &[T::Before, T::AfterMutation, T::AfterEvent, T::Boundary];
        const END: &[T] = &[T::AfterMutation, T::AfterEvent, T::Boundary];
        const DEFEAT: &[T] = &[T::AfterMutation, T::AfterDefeatSettlement, T::AfterEvent];
        const ACTION_END: &[T] = &[T::AfterMutation, T::AfterEvent, T::AfterAction, T::Boundary];
        const UNOBSERVED: &[T] = &[];
        match self {
            P::BattleStarted
            | P::WaveStarted
            | P::CycleStarted
            | P::TurnStarted
            | P::ActionDeclared
            | P::ActionStarted
            | P::PhaseStarted
            | P::HitStarted => START,
            P::BattleWon
            | P::BattleLost
            | P::WaveEnded
            | P::TurnEnded
            | P::PhaseEnded
            | P::HitEnded
            | P::DecisionRequested
            | P::FaultRaised
            | P::EncounterTransition => END,
            P::ActionResolved => ACTION_END,
            P::UnitDefeated => DEFEAT,
            P::BattleFaulted | P::DamageCalculated | P::TimelineChanged => UNOBSERVED,
            _ => POST,
        }
    }
}
