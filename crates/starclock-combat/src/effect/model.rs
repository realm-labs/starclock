//! Generic authored effect semantics. Content identities remain catalog data.

use crate::{
    EffectDefinitionId, Probability, Ratio, Scalar, SourceDefinitionId,
    catalog::action::{OrdinaryDamageDefinition, OrdinaryDamageMultipliers},
    formula::model::{CombatElement, DamageClass},
    rule::model::ValueExpr,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EffectCategory {
    Buff,
    Debuff,
    Control,
    Dot,
    Mark,
    Field,
    Shield,
    NeutralState,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DispelCategory {
    DispellableBuff,
    DispellableDebuff,
    CleanseableControl,
    NonDispellable,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EffectStackPolicy {
    Replace,
    Refresh,
    RefreshAndAddStacks,
    StrongestWins,
    IndependentBySource,
    IndependentInstances,
    UniqueGlobal,
    UniquePerSource,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DurationClock {
    Permanent,
    OwnerTurnStart,
    OwnerTurnEnd,
    TargetTurnStart,
    TargetTurnEnd,
    ActionEnd,
    WaveEnd,
    BattleEnd,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EffectTickPhase {
    None,
    TurnStart,
    TurnEnd,
    ActionStart,
    ActionEnd,
    AfterEvent,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EffectSnapshotPolicy {
    Dynamic,
    OnApplication,
    OnActionStart,
    OnPhaseStart,
    OnHitStart,
    SourceSnapshotTargetDynamic,
    SourceDynamicTargetSnapshot,
    RecomputeOnStackChange,
    ExplicitFields,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EffectTeardownPolicy {
    RemoveWithOwner,
    TransferToTeam,
    FreezeSnapshot,
    PersistByScope,
    ExplicitRule,
}

/// Named action families that a control effect may suppress.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ControlledAction {
    NormalAction,
    Ultimate,
    FollowUp,
    Counter,
    SummonAction,
}

/// DoT-specific captured damage and selection metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DotDefinition {
    formula: OrdinaryDamageDefinition,
    element: CombatElement,
    detonation_tag: Option<SourceDefinitionId>,
}

impl DotDefinition {
    #[must_use]
    pub const fn new(
        formula: OrdinaryDamageDefinition,
        element: CombatElement,
        detonation_tag: Option<SourceDefinitionId>,
    ) -> Self {
        Self {
            formula,
            element,
            detonation_tag,
        }
    }
    #[must_use]
    pub const fn formula(self) -> OrdinaryDamageDefinition {
        self.formula
    }
    #[must_use]
    pub const fn element(self) -> CombatElement {
        self.element
    }
    #[must_use]
    pub const fn detonation_tag(self) -> Option<SourceDefinitionId> {
        self.detonation_tag
    }
}

/// Immutable generic runtime portion of one catalog effect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRuntimeDefinition {
    category: EffectCategory,
    dispel: DispelCategory,
    stack_limit: u16,
    duration: Option<u16>,
    duration_clock: DurationClock,
    tick_phase: EffectTickPhase,
    stack_policy: EffectStackPolicy,
    snapshot_policy: EffectSnapshotPolicy,
    teardown_policy: EffectTeardownPolicy,
    application_priority: i32,
    magnitude: Scalar,
    tags: Box<[SourceDefinitionId]>,
    controlled_actions: Box<[ControlledAction]>,
    dot: Option<DotDefinition>,
}

/// Authored effect semantics whose expression-backed values are resolved at application time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectRuntimeTemplate {
    category: EffectCategory,
    dispel: DispelCategory,
    stack_limit: u16,
    duration: Option<ValueExpr>,
    duration_clock: DurationClock,
    tick_phase: EffectTickPhase,
    stack_policy: EffectStackPolicy,
    magnitude: Option<ValueExpr>,
    snapshot_policy: EffectSnapshotPolicy,
    teardown_policy: EffectTeardownPolicy,
    application_priority: i32,
    dot: Option<(CombatElement, Option<SourceDefinitionId>)>,
}

impl EffectRuntimeTemplate {
    #[must_use]
    pub fn new(
        category: EffectCategory,
        dispel: DispelCategory,
        stack_limit: u16,
        duration: Option<ValueExpr>,
        duration_clock: DurationClock,
        tick_phase: EffectTickPhase,
        stack_policy: EffectStackPolicy,
    ) -> Option<Self> {
        if stack_limit == 0 || (duration_clock == DurationClock::Permanent) != duration.is_none() {
            return None;
        }
        Some(Self {
            category,
            dispel,
            stack_limit,
            duration,
            duration_clock,
            tick_phase,
            stack_policy,
            magnitude: None,
            snapshot_policy: EffectSnapshotPolicy::Dynamic,
            teardown_policy: EffectTeardownPolicy::RemoveWithOwner,
            application_priority: 0,
            dot: None,
        })
    }

    #[must_use]
    pub fn with_comparison(mut self, magnitude: Option<ValueExpr>, priority: i32) -> Self {
        self.magnitude = magnitude;
        self.application_priority = priority;
        self
    }

    #[must_use]
    pub const fn with_snapshot(mut self, policy: EffectSnapshotPolicy) -> Self {
        self.snapshot_policy = policy;
        self
    }

    #[must_use]
    pub const fn with_teardown(mut self, policy: EffectTeardownPolicy) -> Self {
        self.teardown_policy = policy;
        self
    }

    #[must_use]
    pub fn with_dot(
        mut self,
        element: CombatElement,
        detonation_tag: Option<SourceDefinitionId>,
    ) -> Option<Self> {
        if self.category != EffectCategory::Dot {
            return None;
        }
        self.dot = Some((element, detonation_tag));
        Some(self)
    }

    #[must_use]
    pub const fn duration_expression(&self) -> Option<&ValueExpr> {
        self.duration.as_ref()
    }

    #[must_use]
    pub const fn magnitude_expression(&self) -> Option<&ValueExpr> {
        self.magnitude.as_ref()
    }

    /// Materializes immutable runtime state from values evaluated for one target.
    #[must_use]
    pub fn resolve(
        &self,
        duration: Option<u16>,
        magnitude: Scalar,
    ) -> Option<EffectRuntimeDefinition> {
        let mut runtime = EffectRuntimeDefinition::new(
            self.category,
            self.dispel,
            self.stack_limit,
            duration,
            self.duration_clock,
            self.tick_phase,
            self.stack_policy,
        )?
        .with_comparison(magnitude, self.application_priority)
        .with_snapshot(self.snapshot_policy)
        .with_teardown(self.teardown_policy);
        if self.category == EffectCategory::Dot {
            let (element, detonation_tag) = self.dot?;
            let formula = OrdinaryDamageDefinition::new(
                magnitude,
                OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).ok()?,
            )
            .ok()?
            .with_class(DamageClass::Dot);
            runtime = runtime.with_dot(DotDefinition::new(formula, element, detonation_tag))?;
        }
        Some(runtime)
    }
}

impl EffectRuntimeDefinition {
    #[must_use]
    pub fn new(
        category: EffectCategory,
        dispel: DispelCategory,
        stack_limit: u16,
        duration: Option<u16>,
        duration_clock: DurationClock,
        tick_phase: EffectTickPhase,
        stack_policy: EffectStackPolicy,
    ) -> Option<Self> {
        if stack_limit == 0
            || (duration_clock == DurationClock::Permanent) != duration.is_none()
            || duration == Some(0)
        {
            return None;
        }
        Some(Self {
            category,
            dispel,
            stack_limit,
            duration,
            duration_clock,
            tick_phase,
            stack_policy,
            snapshot_policy: EffectSnapshotPolicy::Dynamic,
            teardown_policy: EffectTeardownPolicy::RemoveWithOwner,
            application_priority: 0,
            magnitude: Scalar::ZERO,
            tags: Box::new([]),
            controlled_actions: Box::new([]),
            dot: None,
        })
    }
    #[must_use]
    pub fn with_tags(mut self, mut tags: Vec<SourceDefinitionId>) -> Option<Self> {
        tags.sort_unstable();
        if tags.windows(2).any(|pair| pair[0] == pair[1]) {
            return None;
        }
        self.tags = tags.into_boxed_slice();
        Some(self)
    }
    #[must_use]
    pub fn with_control(mut self, mut actions: Vec<ControlledAction>) -> Option<Self> {
        if self.category != EffectCategory::Control {
            return None;
        }
        actions.sort_unstable();
        actions.dedup();
        self.controlled_actions = actions.into_boxed_slice();
        Some(self)
    }
    #[must_use]
    pub const fn with_comparison(mut self, magnitude: Scalar, priority: i32) -> Self {
        self.magnitude = magnitude;
        self.application_priority = priority;
        self
    }
    #[must_use]
    pub const fn with_snapshot(mut self, policy: EffectSnapshotPolicy) -> Self {
        self.snapshot_policy = policy;
        self
    }
    #[must_use]
    pub const fn with_teardown(mut self, policy: EffectTeardownPolicy) -> Self {
        self.teardown_policy = policy;
        self
    }
    #[must_use]
    pub fn with_dot(mut self, dot: DotDefinition) -> Option<Self> {
        if !matches!(self.category, EffectCategory::Dot) {
            return None;
        }
        self.dot = Some(dot);
        Some(self)
    }
    #[must_use]
    pub const fn category(&self) -> EffectCategory {
        self.category
    }
    #[must_use]
    pub const fn dispel(&self) -> DispelCategory {
        self.dispel
    }
    #[must_use]
    pub const fn stack_limit(&self) -> u16 {
        self.stack_limit
    }
    #[must_use]
    pub const fn duration(&self) -> Option<u16> {
        self.duration
    }
    #[must_use]
    pub const fn duration_clock(&self) -> DurationClock {
        self.duration_clock
    }
    #[must_use]
    pub const fn tick_phase(&self) -> EffectTickPhase {
        self.tick_phase
    }
    #[must_use]
    pub const fn stack_policy(&self) -> EffectStackPolicy {
        self.stack_policy
    }
    #[must_use]
    pub const fn snapshot_policy(&self) -> EffectSnapshotPolicy {
        self.snapshot_policy
    }
    #[must_use]
    pub const fn teardown_policy(&self) -> EffectTeardownPolicy {
        self.teardown_policy
    }
    #[must_use]
    pub const fn application_priority(&self) -> i32 {
        self.application_priority
    }
    #[must_use]
    pub const fn magnitude(&self) -> Scalar {
        self.magnitude
    }
    #[must_use]
    pub fn tags(&self) -> &[SourceDefinitionId] {
        &self.tags
    }
    #[must_use]
    pub fn controlled_actions(&self) -> &[ControlledAction] {
        &self.controlled_actions
    }
    #[must_use]
    pub const fn dot(&self) -> Option<DotDefinition> {
        self.dot
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EffectChancePolicy {
    Guaranteed,
    Fixed {
        chance: Probability,
    },
    Resistible {
        base_chance: Probability,
        attacker_effect_hit_rate: Ratio,
        target_effect_resistance: Ratio,
        target_specific_resistance: Ratio,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectApplicationDefinition {
    pub effect: EffectDefinitionId,
    pub chance: EffectChancePolicy,
    pub stacks: u16,
}

impl EffectApplicationDefinition {
    #[must_use]
    pub const fn new(
        effect: EffectDefinitionId,
        chance: EffectChancePolicy,
        stacks: u16,
    ) -> Option<Self> {
        if stacks == 0 {
            None
        } else {
            Some(Self {
                effect,
                chance,
                stacks,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DotDetonationDefinition {
    pub fraction: Ratio,
    pub required_tag: Option<SourceDefinitionId>,
}

impl DotDetonationDefinition {
    #[must_use]
    pub fn new(fraction: Ratio, required_tag: Option<SourceDefinitionId>) -> Option<Self> {
        if fraction.scaled() < 0 {
            None
        } else {
            Some(Self {
                fraction,
                required_tag,
            })
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EffectRemovalDefinition {
    pub category: DispelCategory,
    pub required_definition: Option<EffectDefinitionId>,
    pub required_tag: Option<SourceDefinitionId>,
    pub maximum: u16,
}

impl EffectRemovalDefinition {
    #[must_use]
    pub const fn new(
        category: DispelCategory,
        required_tag: Option<SourceDefinitionId>,
        maximum: u16,
    ) -> Option<Self> {
        if maximum == 0 {
            None
        } else {
            Some(Self {
                category,
                required_definition: None,
                required_tag,
                maximum,
            })
        }
    }

    #[must_use]
    pub const fn exact(definition: EffectDefinitionId, maximum: u16) -> Option<Self> {
        if maximum == 0 {
            None
        } else {
            Some(Self {
                category: DispelCategory::NonDispellable,
                required_definition: Some(definition),
                required_tag: None,
                maximum,
            })
        }
    }
}
