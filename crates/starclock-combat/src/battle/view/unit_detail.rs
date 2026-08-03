//! Read-only projections of nested unit-owned runtime records.

use crate::{
    AbilityId, OperationId, PresenceState, Scalar, TimelineActorId, TransformEndPolicy,
    UnitDefinitionId, UnitId,
    actor::store::{CharacterResourceState, TransformationState},
    formula::model::CombatElement,
};

/// Immutable form-scoped character-resource projection.
#[derive(Clone, Copy)]
pub struct CharacterResourceView<'a> {
    pub(super) state: &'a CharacterResourceState,
}

impl<'a> CharacterResourceView<'a> {
    #[must_use]
    pub fn stable_key(self) -> &'a str {
        &self.state.stable_key
    }
    #[must_use]
    pub const fn initial(self) -> Scalar {
        self.state.initial
    }
    #[must_use]
    pub const fn current(self) -> Scalar {
        self.state.current
    }
    #[must_use]
    pub const fn maximum(self) -> Scalar {
        self.state.maximum
    }
}

/// Immutable temporary elemental-weakness projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TemporaryWeaknessView {
    pub(super) element: CombatElement,
    pub(super) applier: UnitId,
    pub(super) source_operation: OperationId,
    pub(super) remaining_turns: u8,
}

impl TemporaryWeaknessView {
    #[must_use]
    pub const fn element(self) -> CombatElement {
        self.element
    }
    #[must_use]
    pub const fn applier(self) -> UnitId {
        self.applier
    }
    #[must_use]
    pub const fn source_operation(self) -> OperationId {
        self.source_operation
    }
    #[must_use]
    pub const fn remaining_turns(self) -> u8 {
        self.remaining_turns
    }
}

/// Immutable active-transformation projection.
#[derive(Clone, Copy)]
pub struct TransformationView<'a> {
    pub(super) state: &'a TransformationState,
}

impl<'a> TransformationView<'a> {
    #[must_use]
    pub const fn source_operation(self) -> OperationId {
        self.state.source_operation
    }
    #[must_use]
    pub const fn original_form(self) -> UnitDefinitionId {
        self.state.original_form
    }
    #[must_use]
    pub fn original_abilities(self) -> &'a [AbilityId] {
        &self.state.original_abilities
    }
    #[must_use]
    pub const fn original_presence(self) -> PresenceState {
        self.state.original_presence
    }
    #[must_use]
    pub const fn countdown_actor(self) -> Option<TimelineActorId> {
        self.state.countdown_actor
    }
    #[must_use]
    pub const fn defeat_policy(self) -> TransformEndPolicy {
        self.state.defeat
    }
    #[must_use]
    pub const fn wave_policy(self) -> TransformEndPolicy {
        self.state.wave
    }
}
