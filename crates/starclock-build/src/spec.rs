//! Exact normalized build selection accepted by the compiler.

use starclock_combat::{UnitDefinitionId, UnitLevel};

use crate::{
    ability::AbilityInvestment,
    id::{LightConeId, TraceNodeId},
    light_cone::{LightConeLevel, Superimposition},
    relic_boundary::DeferredRelicBoundary,
};

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

/// Exact selected Eidolon level. E0 applies no Eidolon definition; En applies
/// every rank from E1 through En exactly once.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EidolonLevel(u8);

impl EidolonLevel {
    pub const E0: Self = Self(0);
    pub const MAX: u8 = 6;
    #[must_use]
    pub const fn new(raw: u8) -> Option<Self> {
        if raw <= Self::MAX {
            Some(Self(raw))
        } else {
            None
        }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LightConeLoadout {
    definition: LightConeId,
    level: LightConeLevel,
    promotion: PromotionStage,
    superimposition: Superimposition,
}

impl LightConeLoadout {
    #[must_use]
    pub const fn new(
        definition: LightConeId,
        level: LightConeLevel,
        promotion: PromotionStage,
        superimposition: Superimposition,
    ) -> Self {
        Self {
            definition,
            level,
            promotion,
            superimposition,
        }
    }
    #[must_use]
    pub const fn definition(self) -> LightConeId {
        self.definition
    }
    #[must_use]
    pub const fn level(self) -> LightConeLevel {
        self.level
    }
    #[must_use]
    pub const fn promotion(self) -> PromotionStage {
        self.promotion
    }
    #[must_use]
    pub const fn superimposition(self) -> Superimposition {
        self.superimposition
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
    eidolon: EidolonLevel,
    light_cone: Option<LightConeLoadout>,
    relic_boundary: DeferredRelicBoundary,
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
            eidolon: EidolonLevel::E0,
            light_cone: None,
            relic_boundary: DeferredRelicBoundary::EMPTY,
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
    pub fn with_eidolon(mut self, eidolon: EidolonLevel) -> Self {
        self.eidolon = eidolon;
        self
    }
    #[must_use]
    pub fn with_light_cone(mut self, light_cone: LightConeLoadout) -> Self {
        self.light_cone = Some(light_cone);
        self
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
    #[must_use]
    pub const fn eidolon(&self) -> EidolonLevel {
        self.eidolon
    }
    #[must_use]
    pub const fn light_cone(&self) -> Option<LightConeLoadout> {
        self.light_cone
    }
    #[must_use]
    pub const fn relic_boundary(&self) -> DeferredRelicBoundary {
        self.relic_boundary
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
