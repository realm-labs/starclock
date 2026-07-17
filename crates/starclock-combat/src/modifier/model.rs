//! Closed modifier-domain values accepted after generated-row lowering.

use std::collections::BTreeSet;

use crate::{
    ActionId, ModifierDefinitionId, ModifierInstanceId, ModifierStackingGroupId, SelectorId,
    SourceDefinitionId, StateSlotDefinitionId, UnitId,
    rule::model::{RuleValue, SourceClass, ValueExpr},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum StatKind {
    Hp,
    Atk,
    Def,
    Spd,
    CritRate,
    CritDamage,
    EffectHitRate,
    EffectResistance,
    BreakEffect,
    EnergyRegenerationRate,
    OutgoingHealing,
    IncomingHealing,
    ShieldStrength,
    Aggro,
    ToughnessDamage,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FormulaStage {
    BaseAdd,
    PercentOfBase,
    Flat,
    FinalAdd,
    FinalMultiply,
    Crit,
    DamageBoost,
    Weaken,
    Defense,
    Resistance,
    Vulnerability,
    Mitigation,
    Broken,
    Healing,
    Shield,
    Probability,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FormulaPurpose {
    Stat,
    OrdinaryDamage,
    Dot,
    Break,
    SuperBreak,
    AdditionalDamage,
    JointDamage,
    ElationDamage,
    TrueDamage,
    Healing,
    Shield,
    EffectChance,
    Aggro,
    ActionOrder,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ModifierAggregation {
    Sum,
    Product,
    Maximum,
    Minimum,
    Latest,
    Earliest,
    StrongestByComparator,
    UniquePerSource,
    ReplaceGroup,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SnapshotPolicy {
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
pub enum StatQuerySubject {
    Owner,
    Actor,
    Applier,
    EventTarget,
    CurrentTarget,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatQuery {
    pub subject: UnitId,
    pub stat: StatKind,
    pub purpose: FormulaPurpose,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum LifeFilter {
    Any,
    Alive,
    Downed,
    Defeated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PresenceFilter {
    Any,
    Present,
    Reserved,
    Departed,
    Untargetable,
    Linked,
    Transformed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModifierFilter {
    AbilityTag(Box<str>),
    DamageTag(Box<str>),
    Element(u8),
    Action(u8),
    Life(LifeFilter),
    Presence(PresenceFilter),
    Source(SourceClass),
    Target(SelectorId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModifierStackingGroup {
    pub id: ModifierStackingGroupId,
    pub aggregation: ModifierAggregation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModifierDefinition {
    pub id: ModifierDefinitionId,
    pub stat: StatKind,
    pub stage: FormulaStage,
    pub purpose: FormulaPurpose,
    pub value: ValueExpr,
    pub stacking_group: ModifierStackingGroupId,
    pub priority: i32,
    pub floor: Option<crate::Scalar>,
    pub cap: Option<crate::Scalar>,
    pub cap_stage: FormulaStage,
    pub snapshot: SnapshotPolicy,
    pub filters: Box<[ModifierFilter]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActiveModifier {
    pub instance: ModifierInstanceId,
    pub definition: ModifierDefinitionId,
    pub owner: UnitId,
    pub subject: UnitId,
    pub source: SourceDefinitionId,
    pub source_class: SourceClass,
    pub insertion_sequence: u64,
    pub application_action: Option<ActionId>,
    pub slots: Box<[(StateSlotDefinitionId, RuleValue)]>,
    pub captured_value: Option<crate::Scalar>,
    pub captured_stats: Box<[(StatQuery, crate::Scalar)]>,
}

impl ActiveModifier {
    pub fn set_slot(&mut self, slot: StateSlotDefinitionId, value: RuleValue) -> bool {
        match self.slots.binary_search_by_key(&slot, |entry| entry.0) {
            Ok(index) => {
                self.slots[index].1 = value;
                true
            }
            Err(_) => false,
        }
    }
}

/// Action-local distinct-target memory for content that grants credit once per target.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionTargetLedger {
    credited: BTreeSet<(ActionId, UnitId)>,
}

impl ActionTargetLedger {
    /// Grants bounded credit only for the first observation of a target in an action.
    pub fn credit(
        &mut self,
        action: ActionId,
        target: UnitId,
        ordinary: u16,
        conditional_bonus: u16,
        remaining_capacity: u16,
    ) -> u16 {
        if !self.credited.insert((action, target)) {
            return 0;
        }
        ordinary
            .saturating_add(conditional_bonus)
            .min(remaining_capacity)
    }

    /// Drops completed-action memory without affecting other active action envelopes.
    pub fn clear_action(&mut self, action: ActionId) {
        self.credited.retain(|(candidate, _)| *candidate != action);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.credited.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.credited.is_empty()
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModifierQueryContext {
    pub ability_tags: Box<[Box<str>]>,
    pub damage_tags: Box<[Box<str>]>,
    pub element: Option<u8>,
    pub action_kind: Option<u8>,
    pub life: Option<LifeFilter>,
    pub presence: Option<PresenceFilter>,
    pub source_class: Option<SourceClass>,
    pub target: Option<UnitId>,
    pub matched_target_selectors: Box<[SelectorId]>,
}
