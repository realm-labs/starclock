//! Construction and read access for typed Rule IR state-slot definitions.

use super::{
    BattleRuleScope, RuleValue, RuleValueKind, SlotPersistence, SlotResetPoint, SlotVisibility,
    StateSlotDef,
};
use crate::StateSlotDefinitionId;

impl StateSlotDef {
    #[must_use]
    pub fn new(
        id: StateSlotDefinitionId,
        kind: RuleValueKind,
        scope: BattleRuleScope,
        initial: RuleValue,
    ) -> Self {
        Self {
            id,
            kind,
            scope,
            initial,
            minimum: None,
            maximum: None,
            visibility: SlotVisibility::Owner,
            persistence: SlotPersistence::ScopeLifetime,
            reset_points: Box::new([]),
        }
    }

    #[must_use]
    pub fn with_bounds(mut self, minimum: RuleValue, maximum: RuleValue) -> Self {
        self.minimum = Some(minimum);
        self.maximum = Some(maximum);
        self
    }

    #[must_use]
    pub fn with_optional_bounds(
        mut self,
        minimum: Option<RuleValue>,
        maximum: Option<RuleValue>,
    ) -> Self {
        self.minimum = minimum;
        self.maximum = maximum;
        self
    }

    #[must_use]
    pub fn with_reset_points(mut self, reset_points: Vec<SlotResetPoint>) -> Self {
        self.reset_points = reset_points.into_boxed_slice();
        self
    }

    #[must_use]
    pub const fn with_policy(
        mut self,
        visibility: SlotVisibility,
        persistence: SlotPersistence,
    ) -> Self {
        self.visibility = visibility;
        self.persistence = persistence;
        self
    }

    #[must_use]
    pub const fn id(&self) -> StateSlotDefinitionId {
        self.id
    }

    #[must_use]
    pub const fn kind(&self) -> RuleValueKind {
        self.kind
    }

    #[must_use]
    pub const fn scope(&self) -> BattleRuleScope {
        self.scope
    }

    #[must_use]
    pub const fn initial(&self) -> &RuleValue {
        &self.initial
    }

    #[must_use]
    pub const fn minimum(&self) -> Option<&RuleValue> {
        self.minimum.as_ref()
    }

    #[must_use]
    pub const fn maximum(&self) -> Option<&RuleValue> {
        self.maximum.as_ref()
    }

    #[must_use]
    pub const fn visibility(&self) -> SlotVisibility {
        self.visibility
    }

    #[must_use]
    pub const fn persistence(&self) -> SlotPersistence {
        self.persistence
    }

    #[must_use]
    pub fn reset_points(&self) -> &[SlotResetPoint] {
        &self.reset_points
    }
}
