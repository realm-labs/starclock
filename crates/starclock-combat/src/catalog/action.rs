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
    /// Automatically scheduled follow-up action.
    FollowUp = 3,
    /// Follow-up caused by an incoming attack.
    Counter = 4,
    /// Turn-like action that does not own or reset the normal timeline turn.
    ExtraTurn = 5,
    /// Other automatically scheduled out-of-order action.
    ExtraAction = 6,
    /// Action held until an authored later reaction boundary.
    DelayedAction = 7,
    /// Independently scheduled non-memosprite linked actor action.
    Summon = 8,
    /// Independently scheduled target-capable memosprite action.
    Memosprite = 9,
    /// Timeline-only state-ending countdown action.
    Countdown = 10,
}

/// Stable execution boundary for one authored ability-phase program.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityProgramTiming {
    Entry = 0,
    BeforeHits = 1,
    Hits = 2,
    AfterHits = 3,
    Resolved = 4,
}

/// Ordered program binding retained from one authored ability phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AbilityProgramBinding {
    sequence: u16,
    timing: AbilityProgramTiming,
    program: crate::ProgramId,
}

impl AbilityProgramBinding {
    #[must_use]
    pub const fn new(
        sequence: u16,
        timing: AbilityProgramTiming,
        program: crate::ProgramId,
    ) -> Option<Self> {
        if sequence == 0 {
            None
        } else {
            Some(Self {
                sequence,
                timing,
                program,
            })
        }
    }
    #[must_use]
    pub const fn sequence(self) -> u16 {
        self.sequence
    }
    #[must_use]
    pub const fn timing(self) -> AbilityProgramTiming {
        self.timing
    }
    #[must_use]
    pub const fn program(self) -> crate::ProgramId {
        self.program
    }
}

/// Orthogonal semantic labels inspected by rules independently from the action family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum AbilityTag {
    Attack = 0,
    Basic = 1,
    Skill = 2,
    Ultimate = 3,
    FollowUp = 4,
    Counter = 5,
    Summon = 6,
    Memosprite = 7,
    AdditionalDamage = 8,
    Joint = 9,
    ElationSkill = 10,
    /// Skill temporarily offered by an authored provider effect.
    Assist = 11,
}

/// Compact, canonically encoded set of generic ability tags.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AbilityTags(u32);

impl AbilityTags {
    const ALL_BITS: u32 = (1_u32 << 12) - 1;
    #[must_use]
    pub fn new(tags: &[AbilityTag]) -> Self {
        Self(
            tags.iter()
                .fold(0, |bits, tag| bits | (1_u32 << (*tag as u8))),
        )
    }

    #[must_use]
    pub const fn from_bits(bits: u32) -> Option<Self> {
        if bits & !Self::ALL_BITS == 0 {
            Some(Self(bits))
        } else {
            None
        }
    }

    #[must_use]
    pub const fn contains(self, tag: AbilityTag) -> bool {
        self.0 & (1_u32 << (tag as u8)) != 0
    }

    #[must_use]
    pub const fn supports_forced_skill(self) -> bool {
        self.contains(AbilityTag::ElationSkill) || self.contains(AbilityTag::Assist)
    }

    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

impl AbilityKind {
    #[must_use]
    pub const fn is_normal_turn(self) -> bool {
        matches!(self, Self::Basic | Self::Skill)
    }
}

/// Stable point at which queued work becomes eligible to execute.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ReactionBoundary {
    AfterHit = 0,
    AfterPhase = 1,
    AfterAction = 2,
    BeforeTimeline = 3,
}

/// Cause-relative unit that performs an authored queued action.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueuedActor {
    CauseOwner,
    CauseApplier,
    PrimaryTarget,
    /// The unique active linked entity of this generic kind on the provider's side.
    SharedEntity(crate::LinkedEntityKind),
}

/// Explicit attribution owner for a queued action whose actor may differ.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueuedOwner {
    Actor,
    CauseOwner,
    CauseApplier,
}

/// Actual payer selected for an authored Skill Point cost.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SkillPointPaymentPolicy {
    TeamSkillPoints,
    Suppressed,
    TeamResource(crate::SourceDefinitionId),
}

/// Checked mutation selected for a generic team-owned resource.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TeamResourceChange {
    Gain(u16),
    Spend(u16),
    Set(u16),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TeamResourceChangeDefinition {
    resource: crate::SourceDefinitionId,
    change: TeamResourceChange,
}

impl TeamResourceChangeDefinition {
    #[must_use]
    pub const fn new(resource: crate::SourceDefinitionId, change: TeamResourceChange) -> Self {
        Self { resource, change }
    }
    #[must_use]
    pub const fn resource(self) -> crate::SourceDefinitionId {
        self.resource
    }
    #[must_use]
    pub const fn change(self) -> TeamResourceChange {
        self.change
    }
}

/// Cause-relative primary target retained when queued work is created.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum QueuedTarget {
    CauseActor,
    CauseOwner,
    CauseApplier,
    PrimaryTarget,
    None,
}

/// Generic queue request embedded in an authored operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueueActionDefinition {
    ability: crate::AbilityId,
    origin: crate::ActionOrigin,
    actor: QueuedActor,
    target: QueuedTarget,
    boundary: ReactionBoundary,
    priority: i16,
    owner: QueuedOwner,
    payment: Option<SkillPointPaymentPolicy>,
}

impl QueueActionDefinition {
    #[must_use]
    pub const fn new(
        ability: crate::AbilityId,
        origin: crate::ActionOrigin,
        actor: QueuedActor,
        target: QueuedTarget,
        boundary: ReactionBoundary,
        priority: i16,
    ) -> Self {
        Self {
            ability,
            origin,
            actor,
            target,
            boundary,
            priority,
            owner: QueuedOwner::Actor,
            payment: None,
        }
    }
    /// Overrides queued action attribution and cost handling explicitly.
    #[must_use]
    pub const fn with_envelope(
        mut self,
        owner: QueuedOwner,
        payment: Option<SkillPointPaymentPolicy>,
    ) -> Self {
        self.owner = owner;
        self.payment = payment;
        self
    }
    #[must_use]
    pub const fn ability(self) -> crate::AbilityId {
        self.ability
    }
    #[must_use]
    pub const fn origin(self) -> crate::ActionOrigin {
        self.origin
    }
    #[must_use]
    pub const fn actor(self) -> QueuedActor {
        self.actor
    }
    #[must_use]
    pub const fn target(self) -> QueuedTarget {
        self.target
    }
    #[must_use]
    pub const fn boundary(self) -> ReactionBoundary {
        self.boundary
    }
    #[must_use]
    pub const fn priority(self) -> i16 {
        self.priority
    }
    #[must_use]
    pub const fn owner(self) -> QueuedOwner {
        self.owner
    }
    #[must_use]
    pub const fn payment(self) -> Option<SkillPointPaymentPolicy> {
        self.payment
    }
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

/// Per-hit target projection authored independently from the ability selector.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum HitTargetGroup {
    /// The controller-selected or retained primary target only.
    Primary = 0,
    /// Valid selected targets adjacent to the primary, excluding the primary.
    Adjacent = 1,
    /// The complete committed selector result for this hit.
    Selected = 2,
    /// Every target in the committed all-target selector result.
    All = 3,
    /// One deterministic RNG draw from the current legal target pool.
    BounceDraw = 4,
    /// The acting unit.
    SelfTarget = 5,
}

/// Authored CRIT sampling relationship retained for later damage operations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum HitCritPolicy {
    /// Each target owns an independent CRIT sample.
    PerTarget = 0,
    /// All targets share one CRIT sample.
    Shared = 1,
    /// This hit cannot CRIT.
    Never = 2,
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

/// One exact form-scoped resource cost paid when an action starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterResourceCost {
    stable_key: Box<str>,
    amount: Scalar,
}

impl CharacterResourceCost {
    /// Creates a positive cost for one nonempty character-resource key.
    #[must_use]
    pub fn new(stable_key: impl Into<Box<str>>, amount: Scalar) -> Option<Self> {
        let stable_key = stable_key.into();
        if stable_key.trim().is_empty() || amount.scaled() <= 0 {
            return None;
        }
        Some(Self { stable_key, amount })
    }
    /// Returns the exact form-scoped resource key.
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    /// Returns the positive amount paid at action start.
    #[must_use]
    pub const fn amount(&self) -> Scalar {
        self.amount
    }
}

/// One exact side-scoped keyed resource cost paid when an action starts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TeamResourceCost {
    stable_key: Box<str>,
    amount: u16,
}

impl TeamResourceCost {
    /// Creates a positive cost for one nonempty team-resource key.
    #[must_use]
    pub fn new(stable_key: impl Into<Box<str>>, amount: u16) -> Option<Self> {
        let stable_key = stable_key.into();
        if stable_key.trim().is_empty() || amount == 0 {
            return None;
        }
        Some(Self { stable_key, amount })
    }
    /// Returns the exact side-scoped resource key.
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    /// Returns the positive amount paid at action start.
    #[must_use]
    pub const fn amount(&self) -> u16 {
        self.amount
    }
}

/// Costs and gains applied at their common action-envelope boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResourcePolicy {
    skill_point_cost: u16,
    skill_point_gain: u16,
    energy_cost: Energy,
    energy_gain: Energy,
    skill_point_payment: SkillPointPaymentPolicy,
    character_resource_costs: Box<[CharacterResourceCost]>,
    team_resource_costs: Box<[TeamResourceCost]>,
}

impl ActionResourcePolicy {
    /// Creates an explicit resource policy; zero values disable a component.
    #[must_use]
    pub fn new(
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
            skill_point_payment: SkillPointPaymentPolicy::TeamSkillPoints,
            character_resource_costs: Box::new([]),
            team_resource_costs: Box::new([]),
        }
    }
    /// Selects the actual payer while retaining the authored attempted SP cost.
    #[must_use]
    pub const fn with_skill_point_payment(mut self, payment: SkillPointPaymentPolicy) -> Self {
        self.skill_point_payment = payment;
        self
    }
    /// Suppresses every payable cost while retaining authored gains and the
    /// attempted Skill Point amount for deterministic event telemetry.
    #[must_use]
    pub fn with_costs_suppressed(mut self) -> Self {
        self.skill_point_payment = SkillPointPaymentPolicy::Suppressed;
        self.energy_cost = crate::Energy::ZERO;
        self.character_resource_costs = Box::new([]);
        self.team_resource_costs = Box::new([]);
        self
    }
    /// Attaches costs in strictly increasing, unique stable-key order.
    #[must_use]
    pub fn with_character_resource_costs(
        mut self,
        costs: Vec<CharacterResourceCost>,
    ) -> Option<Self> {
        if costs
            .windows(2)
            .any(|pair| pair[0].stable_key() >= pair[1].stable_key())
        {
            return None;
        }
        self.character_resource_costs = costs.into_boxed_slice();
        Some(self)
    }
    /// Attaches side-scoped costs in strictly increasing, unique stable-key order.
    #[must_use]
    pub fn with_team_resource_costs(mut self, costs: Vec<TeamResourceCost>) -> Option<Self> {
        if costs
            .windows(2)
            .any(|pair| pair[0].stable_key() >= pair[1].stable_key())
        {
            return None;
        }
        self.team_resource_costs = costs.into_boxed_slice();
        Some(self)
    }
    /// Returns the team Skill Point cost.
    #[must_use]
    pub const fn skill_point_cost(&self) -> u16 {
        self.skill_point_cost
    }
    /// Returns the ordinary team Skill Point gain.
    #[must_use]
    pub const fn skill_point_gain(&self) -> u16 {
        self.skill_point_gain
    }
    /// Returns the personal Energy cost.
    #[must_use]
    pub const fn energy_cost(&self) -> Energy {
        self.energy_cost
    }
    /// Returns the ordinary personal Energy gain.
    #[must_use]
    pub const fn energy_gain(&self) -> Energy {
        self.energy_gain
    }
    #[must_use]
    pub const fn skill_point_payment(&self) -> SkillPointPaymentPolicy {
        self.skill_point_payment
    }
    /// Returns canonical form-scoped costs paid by the acting unit.
    #[must_use]
    pub fn character_resource_costs(&self) -> &[CharacterResourceCost] {
        &self.character_resource_costs
    }
    /// Returns canonical side-scoped costs paid by the acting unit's team.
    #[must_use]
    pub fn team_resource_costs(&self) -> &[TeamResourceCost] {
        &self.team_resource_costs
    }
    /// Returns whether the action requires any current resource.
    #[must_use]
    pub fn has_payable_cost(&self) -> bool {
        self.skill_point_cost > 0
            || self.energy_cost > Energy::ZERO
            || !self.character_resource_costs.is_empty()
            || !self.team_resource_costs.is_empty()
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
    class: crate::formula::model::DamageClass,
}

/// One live-stat damage coefficient retained by an authored ability hit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScalingDamageDefinition {
    scaling_stat: crate::modifier::model::StatKind,
    coefficient: Ratio,
    class: crate::formula::model::DamageClass,
    element: crate::formula::model::CombatElement,
}

impl ScalingDamageDefinition {
    /// Creates a non-negative coefficient over one explicitly selected actor stat.
    pub fn new(
        scaling_stat: crate::modifier::model::StatKind,
        coefficient: Ratio,
        class: crate::formula::model::DamageClass,
        element: crate::formula::model::CombatElement,
    ) -> Result<Self, NumericError> {
        if coefficient.scaled() < 0 {
            Err(NumericError::OutOfDomain)
        } else {
            Ok(Self {
                scaling_stat,
                coefficient,
                class,
                element,
            })
        }
    }

    #[must_use]
    pub const fn scaling_stat(self) -> crate::modifier::model::StatKind {
        self.scaling_stat
    }

    #[must_use]
    pub const fn coefficient(self) -> Ratio {
        self.coefficient
    }

    #[must_use]
    pub const fn class(self) -> crate::formula::model::DamageClass {
        self.class
    }

    #[must_use]
    pub const fn element(self) -> crate::formula::model::CombatElement {
        self.element
    }

    /// Resolves the live actor stat into the ordinary formula's exact base amount.
    pub fn resolve(self, stat: Scalar) -> Result<OrdinaryDamageDefinition, NumericError> {
        let base = self
            .coefficient
            .checked_apply(stat, crate::Rounding::NearestTiesEven)?;
        OrdinaryDamageDefinition::new(base, OrdinaryDamageMultipliers::new([Ratio::ONE; 9])?)
            .map(|definition| definition.with_class(self.class))
    }
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
                class: crate::formula::model::DamageClass::Direct,
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
    /// Selects the independently queryable ordinary-formula damage class.
    #[must_use]
    pub const fn with_class(mut self, class: crate::formula::model::DamageClass) -> Self {
        self.class = class;
        self
    }
    #[must_use]
    pub const fn class(self) -> crate::formula::model::DamageClass {
        self.class
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

/// Elemental weakness application with an explicit target-turn lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WeaknessApplicationDefinition {
    element: crate::formula::model::CombatElement,
    duration_turns: Option<u8>,
}

impl WeaknessApplicationDefinition {
    #[must_use]
    pub const fn permanent(element: crate::formula::model::CombatElement) -> Self {
        Self {
            element,
            duration_turns: None,
        }
    }
    #[must_use]
    pub const fn timed(
        element: crate::formula::model::CombatElement,
        duration_turns: u8,
    ) -> Option<Self> {
        if duration_turns == 0 {
            None
        } else {
            Some(Self {
                element,
                duration_turns: Some(duration_turns),
            })
        }
    }
    #[must_use]
    pub const fn element(self) -> crate::formula::model::CombatElement {
        self.element
    }
    #[must_use]
    pub const fn duration_turns(self) -> Option<u8> {
        self.duration_turns
    }
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HitOperationDefinition {
    /// Resolves a retained coefficient against the acting unit's live stat.
    ScalingDamage(ScalingDamageDefinition),
    /// Ordinary HP damage through the general multiplier pipeline.
    Damage(OrdinaryDamageDefinition),
    /// HP restoration through the additive healing multiplier block.
    Heal(HealingDefinition),
    /// Creates one separately retained shield instance.
    Shield(ShieldDefinition),
    /// Consumes HP without treating the loss as damage.
    ConsumeHp(HpConsumptionDefinition),
    /// Adds one elemental weakness before later operations in the same hit.
    AddWeakness(WeaknessApplicationDefinition),
    /// Routes a checked reduction through the first eligible authored layer.
    ReduceToughness(crate::ToughnessReductionDefinition),
    /// Converts the preceding effective reduction for each target into Super Break.
    SuperBreak(crate::formula::toughness::SuperBreakDefinition),
    /// Applies one catalog effect using its authored chance and stacking policy.
    ApplyEffect(crate::EffectApplicationDefinition),
    /// Removes a bounded stable-ID query of dispellable/cleanseable effects.
    RemoveEffects(crate::EffectRemovalDefinition),
    /// Replays selected target-local ordinary DoT snapshots without mutating them.
    DetonateDots(crate::DotDetonationDefinition),
    /// Mutates one battle-owned typed slot on the actor's bound rule instance.
    ModifyStateSlot(crate::rule::model::RuleSlotMutationDefinition),
    /// Mutates one generic resource owned by the acting side.
    ModifyTeamResource(TeamResourceChangeDefinition),
    /// Queues one cause-relative action through the deterministic reaction scheduler.
    QueueAction(QueueActionDefinition),
    /// Allocates one linked unit and optional independent timeline actor.
    SummonLinked(crate::LinkedUnitDefinition),
    /// Applies an explicit battlefield-presence transition.
    ChangePresence(crate::PresenceState),
    /// Atomically replaces form/abilities and optionally creates a countdown.
    Transform(crate::TransformationDefinition),
    /// Restores the original form/abilities and removes transform-owned actors.
    EndTransformation,
    /// Restores a downed or defeated unit with authored HP/presence/gauge policy.
    Revive(crate::ReviveDefinition),
    /// Departs a linked unit and deactivates its timeline actor.
    DespawnLinked,
    /// Performs an explicitly authored pending wave transition.
    RequestWaveTransition,
    /// Advances one hostile occurrence to an exact validated boss phase.
    TransitionEnemyPhase(crate::EnemyPhaseId),
}

/// Ordered operation templates owned by one authored hit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionHitDefinition {
    operations: Box<[HitOperationDefinition]>,
    target_group: HitTargetGroup,
    damage_share: Ratio,
    toughness_share: Ratio,
    crit_policy: HitCritPolicy,
}

impl ActionHitDefinition {
    /// Creates one hit; an empty list is a structural hit with no mutation.
    #[must_use]
    pub fn new(operations: Vec<HitOperationDefinition>) -> Self {
        Self {
            operations: operations.into_boxed_slice(),
            target_group: HitTargetGroup::Selected,
            damage_share: Ratio::ONE,
            toughness_share: Ratio::ONE,
            crit_policy: HitCritPolicy::PerTarget,
        }
    }
    /// Attaches the validated authored targeting and ratio metadata for this hit.
    #[must_use]
    pub const fn with_profile(
        mut self,
        target_group: HitTargetGroup,
        damage_share: Ratio,
        toughness_share: Ratio,
        crit_policy: HitCritPolicy,
    ) -> Self {
        self.target_group = target_group;
        self.damage_share = damage_share;
        self.toughness_share = toughness_share;
        self.crit_policy = crit_policy;
        self
    }
    /// Returns operations in authored execution order.
    #[must_use]
    pub fn operations(&self) -> &[HitOperationDefinition] {
        &self.operations
    }
    /// Returns the authored target projection for this hit.
    #[must_use]
    pub const fn target_group(&self) -> HitTargetGroup {
        self.target_group
    }
    /// Returns this hit's exact share of the ability damage payload.
    #[must_use]
    pub const fn damage_share(&self) -> Ratio {
        self.damage_share
    }
    /// Returns this hit's exact share of the ability Toughness payload.
    #[must_use]
    pub const fn toughness_share(&self) -> Ratio {
        self.toughness_share
    }
    /// Returns the authored CRIT sampling relationship.
    #[must_use]
    pub const fn crit_policy(&self) -> HitCritPolicy {
        self.crit_policy
    }
}

/// Finite action structure attached to an executable ability.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbilityActionDefinition {
    kind: AbilityKind,
    tags: AbilityTags,
    hits: Box<[ActionHitDefinition]>,
    invalidation: TargetInvalidationPolicy,
    resources: ActionResourcePolicy,
}

const MAX_ACTION_HITS: usize = 256;

impl AbilityActionDefinition {
    /// Creates an action with one to 256 authored hits across all phases.
    #[must_use]
    pub fn new(
        kind: AbilityKind,
        hit_count: u16,
        invalidation: TargetInvalidationPolicy,
        resources: ActionResourcePolicy,
    ) -> Option<Self> {
        if hit_count == 0 || usize::from(hit_count) > MAX_ACTION_HITS {
            None
        } else {
            Some(Self {
                kind,
                tags: AbilityTags::new(&default_tags(kind)),
                hits: (0..hit_count)
                    .map(|_| ActionHitDefinition::new(Vec::new()))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
                invalidation,
                resources,
            })
        }
    }
    /// Adds orthogonal semantic labels, including explicit Elation Skill identity.
    #[must_use]
    pub fn with_tags(mut self, tags: &[AbilityTag]) -> Self {
        self.tags = AbilityTags::new(tags);
        self
    }
    /// Replaces structural hits with one to 256 concrete authored hits.
    #[must_use]
    pub fn with_hits(mut self, hits: Vec<ActionHitDefinition>) -> Option<Self> {
        if hits.is_empty() || hits.len() > MAX_ACTION_HITS {
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
    /// Returns independently queryable semantic labels.
    #[must_use]
    pub const fn tags(&self) -> AbilityTags {
        self.tags
    }
    /// Returns the finite authored hit count.
    #[must_use]
    pub fn hit_count(&self) -> u16 {
        u16::try_from(self.hits.len()).expect("action hit count is validated at 256 or fewer")
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
    pub const fn resources(&self) -> &ActionResourcePolicy {
        &self.resources
    }
}

fn default_tags(kind: AbilityKind) -> [AbilityTag; 2] {
    let family = match kind {
        AbilityKind::Basic => AbilityTag::Basic,
        AbilityKind::Skill => AbilityTag::Skill,
        AbilityKind::Ultimate => AbilityTag::Ultimate,
        AbilityKind::FollowUp => AbilityTag::FollowUp,
        AbilityKind::Counter => AbilityTag::Counter,
        AbilityKind::Summon => AbilityTag::Summon,
        AbilityKind::Memosprite => AbilityTag::Memosprite,
        AbilityKind::ExtraTurn
        | AbilityKind::ExtraAction
        | AbilityKind::DelayedAction
        | AbilityKind::Countdown => AbilityTag::Attack,
    };
    [AbilityTag::Attack, family]
}
