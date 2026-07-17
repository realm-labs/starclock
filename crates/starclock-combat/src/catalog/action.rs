use crate::Energy;

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

/// Finite action structure attached to an executable ability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityActionDefinition {
    kind: AbilityKind,
    hit_count: u16,
    invalidation: TargetInvalidationPolicy,
    resources: ActionResourcePolicy,
}

impl AbilityActionDefinition {
    /// Creates an action with one to 64 authored hits.
    #[must_use]
    pub const fn new(
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
                hit_count,
                invalidation,
                resources,
            })
        }
    }
    /// Returns the shared ability family.
    #[must_use]
    pub const fn kind(self) -> AbilityKind {
        self.kind
    }
    /// Returns the finite authored hit count.
    #[must_use]
    pub const fn hit_count(self) -> u16 {
        self.hit_count
    }
    /// Returns the target revalidation policy applied by every hit.
    #[must_use]
    pub const fn invalidation(self) -> TargetInvalidationPolicy {
        self.invalidation
    }
    /// Returns explicit action-boundary costs and gains.
    #[must_use]
    pub const fn resources(self) -> ActionResourcePolicy {
        self.resources
    }
}
