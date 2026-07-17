use crate::{Energy, NumericError, Ratio, Scalar};

/// Shared semantic family used by legality, resources and event filters.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityKind {
    /// Ordinary turn action that commonly generates Skill Points.
    Basic = 0,
    /// Ordinary turn action that commonly spends Skill Points.
    Skill = 1,
    /// Out-of-order action offered through an interrupt window.
    Ultimate = 2,
}

/// Formation relationship evaluated relative to the action actor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TargetRelation {
    /// Select the acting unit without a controller-supplied primary target.
    SelfUnit = 0,
    /// Select targetable units on the actor's formation side.
    Allied = 1,
    /// Select targetable units on the opposing formation side.
    Opposing = 2,
}

/// Baseline deterministic target pattern compiled by a selector.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TargetPattern {
    /// One offered primary target.
    Single = 0,
    /// Offered primary plus valid adjacent formation neighbors.
    Blast = 1,
    /// Every currently legal target in formation order.
    All = 2,
}

/// Explicit behavior when a committed target becomes illegal before a hit.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum TargetInvalidationPolicy {
    /// Remove an illegal target from this and later hit work.
    CancelRemainingForTarget = 0,
    /// Retain a target while it remains present, including a downed target.
    KeepIfPresent = 1,
    /// Select a replacement from the same stable pool using battle RNG.
    RetargetSamePool = 2,
    /// Replace an illegal primary and rebuild its authored pattern.
    RetargetPrimaryThenRebuildPattern = 3,
    /// Roll back the action if invalidation occurs before a completed mutation.
    FailAction = 4,
}

/// Target semantics attached to one catalog selector definition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UnitTargetSelector {
    relation: TargetRelation,
    pattern: TargetPattern,
    repeated_targets: bool,
}

impl UnitTargetSelector {
    /// Creates a deterministic unit selector.
    #[must_use]
    pub const fn new(relation: TargetRelation, pattern: TargetPattern) -> Option<Self> {
        if matches!(relation, TargetRelation::SelfUnit) && !matches!(pattern, TargetPattern::Single)
        {
            None
        } else {
            Some(Self {
                relation,
                pattern,
                repeated_targets: false,
            })
        }
    }
    /// Permits the same runtime target to occupy multiple selector positions.
    #[must_use]
    pub const fn with_repeated_targets(mut self) -> Self {
        self.repeated_targets = true;
        self
    }
    /// Returns the side relationship evaluated from the actor.
    #[must_use]
    pub const fn relation(self) -> TargetRelation {
        self.relation
    }
    /// Returns the formation pattern.
    #[must_use]
    pub const fn pattern(self) -> TargetPattern {
        self.pattern
    }
    /// Returns whether authored selection permits repeated target identities.
    #[must_use]
    pub const fn repeated_targets(self) -> bool {
        self.repeated_targets
    }
}

/// Costs and gains applied at their common action-envelope boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionResourcePolicy {
    skill_point_cost: u16,
    skill_point_gain: u16,
    energy_cost: Energy,
    energy_gain: Energy,
}

impl ActionResourcePolicy {
    /// Creates an explicit resource policy; zero values disable a component.
    #[must_use]
    pub const fn new(
        skill_point_cost: u16,
        skill_point_gain: u16,
        energy_cost: Energy,
        energy_gain: Energy,
    ) -> Self {
        Self {
            skill_point_cost,
            skill_point_gain,
            energy_cost,
            energy_gain,
        }
    }
    /// Returns the team Skill Point cost.
    #[must_use]
    pub const fn skill_point_cost(self) -> u16 {
        self.skill_point_cost
    }
    /// Returns the ordinary team Skill Point gain.
    #[must_use]
    pub const fn skill_point_gain(self) -> u16 {
        self.skill_point_gain
    }
    /// Returns the personal Energy cost.
    #[must_use]
    pub const fn energy_cost(self) -> Energy {
        self.energy_cost
    }
    /// Returns the ordinary personal Energy gain.
    #[must_use]
    pub const fn energy_gain(self) -> Energy {
        self.energy_gain
    }
}

/// Explicit ordinary-damage multiplier blocks evaluated in formula order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrdinaryDamageMultipliers {
    original_damage: Ratio,
    crit: Ratio,
    damage_boost: Ratio,
    weaken: Ratio,
    defense: Ratio,
    resistance: Ratio,
    vulnerability: Ratio,
    mitigation: Ratio,
    broken: Ratio,
}

impl OrdinaryDamageMultipliers {
    /// Creates the nine named multiplier blocks used by ordinary damage.
    pub fn new(values: [Ratio; 9]) -> Result<Self, NumericError> {
        if values.iter().any(|value| value.scaled() < 0) {
            return Err(NumericError::OutOfDomain);
        }
        Ok(Self {
            original_damage: values[0],
            crit: values[1],
            damage_boost: values[2],
            weaken: values[3],
            defense: values[4],
            resistance: values[5],
            vulnerability: values[6],
            mitigation: values[7],
            broken: values[8],
        })
    }

    /// Returns formula factors in their normative evaluation order.
    #[must_use]
    pub const fn ordered(self) -> [Ratio; 9] {
        [
            self.original_damage,
            self.crit,
            self.damage_boost,
            self.weaken,
            self.defense,
            self.resistance,
            self.vulnerability,
            self.mitigation,
            self.broken,
        ]
    }
}

/// Fully resolved initial ordinary-damage formula input for one target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OrdinaryDamageDefinition {
    base_damage: Scalar,
    multipliers: OrdinaryDamageMultipliers,
}

impl OrdinaryDamageDefinition {
    /// Creates a non-negative base amount with explicit named multipliers.
    pub fn new(
        base_damage: Scalar,
        multipliers: OrdinaryDamageMultipliers,
    ) -> Result<Self, NumericError> {
        if base_damage.scaled() < 0 {
            Err(NumericError::OutOfDomain)
        } else {
            Ok(Self {
                base_damage,
                multipliers,
            })
        }
    }
    /// Returns the fixed-point base damage before multipliers.
    #[must_use]
    pub const fn base_damage(self) -> Scalar {
        self.base_damage
    }
    /// Returns all named multiplier blocks.
    #[must_use]
    pub const fn multipliers(self) -> OrdinaryDamageMultipliers {
        self.multipliers
    }
}

/// Fully resolved initial healing formula input for one target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealingDefinition {
    base_healing: Scalar,
    outgoing_boost: Ratio,
    incoming_boost: Ratio,
    incoming_reduction: Ratio,
}

/// Fully resolved shield creation input plus its explicit absorption policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldDefinition {
    base_shield: Scalar,
    bonus: Ratio,
    policy: crate::formula::shield::ShieldAbsorptionPolicy,
}

impl ShieldDefinition {
    /// Creates a non-negative resolved shield formula.
    pub fn new(
        base_shield: Scalar,
        bonus: Ratio,
        policy: crate::formula::shield::ShieldAbsorptionPolicy,
    ) -> Result<Self, NumericError> {
        if base_shield.scaled() <= 0 || bonus.scaled() < 0 {
            Err(NumericError::OutOfDomain)
        } else {
            Ok(Self {
                base_shield,
                bonus,
                policy,
            })
        }
    }

    #[must_use]
    pub const fn base_shield(self) -> Scalar {
        self.base_shield
    }

    #[must_use]
    pub const fn bonus(self) -> Ratio {
        self.bonus
    }

    #[must_use]
    pub const fn policy(self) -> crate::formula::shield::ShieldAbsorptionPolicy {
        self.policy
    }
}

/// Checked HP-consumption request that cannot defeat its target below `floor`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HpConsumptionDefinition {
    requested: crate::Hp,
    floor: crate::Hp,
}

impl HpConsumptionDefinition {
    #[must_use]
    pub const fn new(requested: crate::Hp, floor: crate::Hp) -> Self {
        Self { requested, floor }
    }

    #[must_use]
    pub const fn requested(self) -> crate::Hp {
        self.requested
    }

    #[must_use]
    pub const fn floor(self) -> crate::Hp {
        self.floor
    }
}

impl HealingDefinition {
    /// Creates the base healing and the three additive modifier components.
    pub fn new(
        base_healing: Scalar,
        outgoing_boost: Ratio,
        incoming_boost: Ratio,
        incoming_reduction: Ratio,
    ) -> Result<Self, NumericError> {
        if base_healing.scaled() < 0
            || outgoing_boost.scaled() < 0
            || incoming_boost.scaled() < 0
            || incoming_reduction.scaled() < 0
        {
            Err(NumericError::OutOfDomain)
        } else {
            Ok(Self {
                base_healing,
                outgoing_boost,
                incoming_boost,
                incoming_reduction,
            })
        }
    }
    /// Returns base healing before the additive multiplier block.
    #[must_use]
    pub const fn base_healing(self) -> Scalar {
        self.base_healing
    }
    /// Returns outgoing healing boost.
    #[must_use]
    pub const fn outgoing_boost(self) -> Ratio {
        self.outgoing_boost
    }
    /// Returns incoming healing boost.
    #[must_use]
    pub const fn incoming_boost(self) -> Ratio {
        self.incoming_boost
    }
    /// Returns incoming healing reduction.
    #[must_use]
    pub const fn incoming_reduction(self) -> Ratio {
        self.incoming_reduction
    }
}

/// Closed initial operation language allowed inside one authored hit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HitOperationDefinition {
    /// Ordinary HP damage through the general multiplier pipeline.
    Damage(OrdinaryDamageDefinition),
    /// HP restoration through the additive healing multiplier block.
    Heal(HealingDefinition),
    /// Creates one separately retained shield instance.
    Shield(ShieldDefinition),
    /// Consumes HP without treating the loss as damage.
    ConsumeHp(HpConsumptionDefinition),
}

/// Ordered operation templates owned by one authored hit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionHitDefinition {
    operations: Box<[HitOperationDefinition]>,
}

impl ActionHitDefinition {
    /// Creates one hit; an empty list is a structural hit with no mutation.
    #[must_use]
    pub fn new(operations: Vec<HitOperationDefinition>) -> Self {
        Self {
            operations: operations.into_boxed_slice(),
        }
    }
    /// Returns operations in authored execution order.
    #[must_use]
    pub fn operations(&self) -> &[HitOperationDefinition] {
        &self.operations
    }
}

/// Finite action structure attached to an executable ability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActionDefinition {
    kind: AbilityKind,
    hits: Box<[ActionHitDefinition]>,
    invalidation: TargetInvalidationPolicy,
    resources: ActionResourcePolicy,
}

impl AbilityActionDefinition {
    /// Creates an action with one to 64 authored hits.
    #[must_use]
    pub fn new(
        kind: AbilityKind,
        hit_count: u16,
        invalidation: TargetInvalidationPolicy,
        resources: ActionResourcePolicy,
    ) -> Option<Self> {
        if hit_count == 0 || hit_count > 64 {
            None
        } else {
            Some(Self {
                kind,
                hits: (0..hit_count)
                    .map(|_| ActionHitDefinition::new(Vec::new()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                invalidation,
                resources,
            })
        }
    }
    /// Replaces structural hits with one to 64 concrete authored hit plans.
    #[must_use]
    pub fn with_hits(mut self, hits: Vec<ActionHitDefinition>) -> Option<Self> {
        if hits.is_empty() || hits.len() > 64 {
            None
        } else {
            self.hits = hits.into_boxed_slice();
            Some(self)
        }
    }
    /// Returns the shared ability family.
    #[must_use]
    pub const fn kind(&self) -> AbilityKind {
        self.kind
    }
    /// Returns the finite authored hit count.
    #[must_use]
    pub fn hit_count(&self) -> u16 {
        u16::try_from(self.hits.len()).expect("action hit count is validated at 64 or fewer")
    }
    /// Returns hit templates in authored order.
    #[must_use]
    pub fn hits(&self) -> &[ActionHitDefinition] {
        &self.hits
    }
    /// Returns the target revalidation policy applied by every hit.
    #[must_use]
    pub const fn invalidation(&self) -> TargetInvalidationPolicy {
        self.invalidation
    }
    /// Returns explicit action-boundary costs and gains.
    #[must_use]
    pub const fn resources(&self) -> ActionResourcePolicy {
        self.resources
    }
}
