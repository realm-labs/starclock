//! Exact normalized build selection accepted by the compiler.

use starclock_combat::{Scalar, UnitDefinitionId, UnitLevel};

use crate::{
    ability::AbilityInvestment,
    id::{BuildContributionId, LightConeId, TraceNodeId},
    light_cone::{LightConeLevel, Superimposition},
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

/// Exact aggregate stats contributed by selected relic-like equipment.
///
/// This value contains only generic numeric build input. Set identities and
/// battle-visible set abilities remain in their owning content/runtime layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelicStatContribution {
    hp_flat: Scalar,
    attack_flat: Scalar,
    defense_flat: Scalar,
    speed_flat: Scalar,
    hp_ratio: Scalar,
    attack_ratio: Scalar,
    defense_ratio: Scalar,
    speed_ratio: Scalar,
    critical_rate: Scalar,
    critical_damage: Scalar,
    effect_hit_rate: Scalar,
    effect_resistance: Scalar,
    break_effect: Scalar,
    energy_regeneration_rate: Scalar,
    outgoing_healing: Scalar,
    element_damage_boosts: [Scalar; 7],
}

impl Default for RelicStatContribution {
    fn default() -> Self {
        Self::new(
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            Scalar::ZERO,
            [Scalar::ZERO; 7],
        )
    }
}

impl RelicStatContribution {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        hp_flat: Scalar,
        attack_flat: Scalar,
        defense_flat: Scalar,
        speed_flat: Scalar,
        hp_ratio: Scalar,
        attack_ratio: Scalar,
        defense_ratio: Scalar,
        speed_ratio: Scalar,
        critical_rate: Scalar,
        critical_damage: Scalar,
        effect_hit_rate: Scalar,
        effect_resistance: Scalar,
        break_effect: Scalar,
        energy_regeneration_rate: Scalar,
        outgoing_healing: Scalar,
        element_damage_boosts: [Scalar; 7],
    ) -> Self {
        Self {
            hp_flat,
            attack_flat,
            defense_flat,
            speed_flat,
            hp_ratio,
            attack_ratio,
            defense_ratio,
            speed_ratio,
            critical_rate,
            critical_damage,
            effect_hit_rate,
            effect_resistance,
            break_effect,
            energy_regeneration_rate,
            outgoing_healing,
            element_damage_boosts,
        }
    }

    #[must_use]
    pub const fn base_flats(self) -> [Scalar; 4] {
        [
            self.hp_flat,
            self.attack_flat,
            self.defense_flat,
            self.speed_flat,
        ]
    }

    #[must_use]
    pub const fn base_ratios(self) -> [Scalar; 4] {
        [
            self.hp_ratio,
            self.attack_ratio,
            self.defense_ratio,
            self.speed_ratio,
        ]
    }

    #[must_use]
    pub const fn secondary(self) -> [Scalar; 7] {
        [
            self.critical_rate,
            self.critical_damage,
            self.effect_hit_rate,
            self.effect_resistance,
            self.break_effect,
            self.energy_regeneration_rate,
            self.outgoing_healing,
        ]
    }

    /// Returns Physical, Fire, Ice, Lightning, Wind, Quantum and Imaginary.
    #[must_use]
    pub const fn element_damage_boosts(self) -> [Scalar; 7] {
        self.element_damage_boosts
    }
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

/// Exact supported build input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatantBuildSpec {
    form: UnitDefinitionId,
    level: UnitLevel,
    promotion: PromotionStage,
    ability_levels: Box<[AbilityInvestment]>,
    traces: Box<[TraceNodeId]>,
    eidolon: EidolonLevel,
    light_cone: Option<LightConeLoadout>,
    contributions: Box<[BuildContributionId]>,
    relic_stats: RelicStatContribution,
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
            contributions: Box::new([]),
            relic_stats: RelicStatContribution::default(),
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
    pub fn with_contributions(
        mut self,
        mut contributions: Vec<BuildContributionId>,
    ) -> Result<Self, BuildSpecError> {
        contributions.sort_unstable();
        if contributions.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(BuildSpecError::DuplicateContribution);
        }
        self.contributions = contributions.into_boxed_slice();
        Ok(self)
    }
    #[must_use]
    pub const fn with_relic_stats(mut self, relic_stats: RelicStatContribution) -> Self {
        self.relic_stats = relic_stats;
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
    pub fn contributions(&self) -> &[BuildContributionId] {
        &self.contributions
    }
    #[must_use]
    pub const fn relic_stats(&self) -> RelicStatContribution {
        self.relic_stats
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildSpecError {
    DuplicateAbilityFamily,
    DuplicateTrace,
    DuplicateContribution,
}

impl std::fmt::Display for BuildSpecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid exact build input: {self:?}")
    }
}

impl std::error::Error for BuildSpecError {}
