//! Exact normalized build selection accepted by the compiler.

use starclock_combat::{UnitDefinitionId, UnitLevel};

use crate::{ability::AbilityInvestment, id::TraceNodeId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PromotionStage(u8);

impl PromotionStage {
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw <= 6 { Some(Self(raw)) } else { None }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Minimal exact build input. Later Phase 5 batches extend this value with
/// ability, Trace, Eidolon and equipment selections.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatantBuildSpec {
    form: UnitDefinitionId,
    level: UnitLevel,
    promotion: PromotionStage,
    ability_levels: Box<[AbilityInvestment]>,
    traces: Box<[TraceNodeId]>,
}

impl CombatantBuildSpec {
    #[must_use]
    pub fn new(form: UnitDefinitionId, level: UnitLevel, promotion: PromotionStage) -> Self {
        Self {
            form,
            level,
            promotion,
            ability_levels: Box::new([]),
            traces: Box::new([]),
        }
    }
    pub fn with_ability_levels(
        mut self,
        mut ability_levels: Vec<AbilityInvestment>,
    ) -> Result<Self, BuildSpecError> {
        ability_levels.sort_unstable_by_key(|entry| entry.family());
        if ability_levels
            .windows(2)
            .any(|pair| pair[0].family() == pair[1].family())
        {
            return Err(BuildSpecError::DuplicateAbilityFamily);
        }
        self.ability_levels = ability_levels.into_boxed_slice();
        Ok(self)
    }
    pub fn with_traces(mut self, mut traces: Vec<TraceNodeId>) -> Result<Self, BuildSpecError> {
        traces.sort_unstable();
        if traces.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(BuildSpecError::DuplicateTrace);
        }
        self.traces = traces.into_boxed_slice();
        Ok(self)
    }
    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub const fn level(&self) -> UnitLevel {
        self.level
    }
    #[must_use]
    pub const fn promotion(&self) -> PromotionStage {
        self.promotion
    }
    #[must_use]
    pub fn ability_levels(&self) -> &[AbilityInvestment] {
        &self.ability_levels
    }
    #[must_use]
    pub fn traces(&self) -> &[TraceNodeId] {
        &self.traces
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildSpecError {
    DuplicateAbilityFamily,
    DuplicateTrace,
}

impl std::fmt::Display for BuildSpecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid exact build input: {self:?}")
    }
}

impl std::error::Error for BuildSpecError {}
