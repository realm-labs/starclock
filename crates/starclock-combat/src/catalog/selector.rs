//! Typed unit-selector plans used by authored Rule IR programs.

use super::CombatCatalog;
use crate::{
    EffectDefinitionId, SelectorId, SourceDefinitionId,
    formula::model::CombatElement,
    modifier::model::StatKind,
    rule::model::{Comparison, ConditionExpr, ValueExpr},
};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorOrigin {
    Source,
    Owner,
    Actor,
    Applier,
    PrimaryTarget,
    CurrentSubject,
    Team,
    Encounter,
    /// Restricts the candidate pool to the ordered targets carried by the event.
    EventTargets,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorSide {
    Same,
    Opposing,
    Any,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleLifePredicate {
    Any,
    Alive,
    Downed,
    Defeated,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RulePresencePredicate {
    Any,
    Present,
    Reserved,
    Departed,
    Untargetable,
    Linked,
    Transformed,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorReference {
    CurrentState,
    EventSnapshot,
    ActionSnapshot,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorOrdering {
    Formation,
    Timeline,
    HpRatioAscending,
    HpRatioDescending,
    StatAscending,
    StatDescending,
    EventOrder,
    StableId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleSelectorChoice {
    All,
    First,
    PrimaryPlusAdjacent,
    AdjacentToPrimary,
    RngUniform,
    RngWeighted,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RuleEmptyPoolPolicy {
    NoOp,
    Skip,
    CancelRemaining,
    Fault,
}

/// One ordered, mutation-free candidate predicate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RuleSelectorPredicate {
    FormationRange {
        minimum: u8,
        maximum: u8,
    },
    /// Keeps candidates exactly one formation slot from the event primary target.
    AdjacentToPrimary,
    HasMark(EffectDefinitionId),
    HasWeakness(CombatElement),
    LacksWeakness(CombatElement),
    HasEffect(EffectDefinitionId),
    HasTag(SourceDefinitionId),
    OwnedBy(SelectorId),
    /// Removes every unit selected by the referenced selector.
    Excludes(SelectorId),
    StatCompare {
        stat: StatKind,
        comparison: Comparison,
        value: ValueExpr,
    },
}

/// Complete typed selector plan retained in the immutable combat catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RuleUnitSelector {
    pub(crate) origin: RuleSelectorOrigin,
    pub(crate) side: RuleSelectorSide,
    pub(crate) life: RuleLifePredicate,
    pub(crate) presence: RulePresencePredicate,
    pub(crate) reference: RuleSelectorReference,
    pub(crate) ordering: RuleSelectorOrdering,
    pub(crate) minimum: u16,
    pub(crate) maximum: u16,
    pub(crate) empty_pool: RuleEmptyPoolPolicy,
    pub(crate) choice: RuleSelectorChoice,
    pub(crate) rng_purpose: Option<Box<str>>,
    pub(crate) repeated: bool,
    pub(crate) predicates: Box<[RuleSelectorPredicate]>,
    pub(crate) weight: Option<ValueExpr>,
}

impl RuleUnitSelector {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        origin: RuleSelectorOrigin,
        side: RuleSelectorSide,
        life: RuleLifePredicate,
        presence: RulePresencePredicate,
        reference: RuleSelectorReference,
        ordering: RuleSelectorOrdering,
        minimum: u16,
        maximum: u16,
        empty_pool: RuleEmptyPoolPolicy,
        choice: RuleSelectorChoice,
        rng_purpose: Option<Box<str>>,
        repeated: bool,
    ) -> Option<Self> {
        (maximum > 0 && minimum <= maximum).then_some(Self {
            origin,
            side,
            life,
            presence,
            reference,
            ordering,
            minimum,
            maximum,
            empty_pool,
            choice,
            rng_purpose,
            repeated,
            predicates: Box::new([]),
            weight: None,
        })
    }

    #[must_use]
    pub fn with_predicates(mut self, predicates: Vec<RuleSelectorPredicate>) -> Self {
        self.predicates = predicates.into_boxed_slice();
        self
    }

    #[must_use]
    pub fn with_weight(mut self, weight: Option<ValueExpr>) -> Self {
        self.weight = weight;
        self
    }

    #[must_use]
    pub const fn origin(&self) -> RuleSelectorOrigin {
        self.origin
    }
    #[must_use]
    pub const fn side(&self) -> RuleSelectorSide {
        self.side
    }
    #[must_use]
    pub const fn life(&self) -> RuleLifePredicate {
        self.life
    }
    #[must_use]
    pub const fn presence(&self) -> RulePresencePredicate {
        self.presence
    }
    #[must_use]
    pub const fn reference(&self) -> RuleSelectorReference {
        self.reference
    }
    #[must_use]
    pub const fn ordering(&self) -> RuleSelectorOrdering {
        self.ordering
    }
    #[must_use]
    pub const fn minimum(&self) -> u16 {
        self.minimum
    }
    #[must_use]
    pub const fn maximum(&self) -> u16 {
        self.maximum
    }
    #[must_use]
    pub const fn empty_pool(&self) -> RuleEmptyPoolPolicy {
        self.empty_pool
    }
    #[must_use]
    pub const fn choice(&self) -> RuleSelectorChoice {
        self.choice
    }
    #[must_use]
    pub fn rng_purpose(&self) -> Option<&str> {
        self.rng_purpose.as_deref()
    }
    #[must_use]
    pub const fn repeated(&self) -> bool {
        self.repeated
    }
    #[must_use]
    pub fn predicates(&self) -> &[RuleSelectorPredicate] {
        &self.predicates
    }
    #[must_use]
    pub const fn weight(&self) -> Option<&ValueExpr> {
        self.weight.as_ref()
    }

    pub(crate) fn dependencies(&self) -> BTreeSet<SelectorId> {
        let mut output = BTreeSet::new();
        for predicate in &self.predicates {
            match predicate {
                RuleSelectorPredicate::OwnedBy(selector)
                | RuleSelectorPredicate::Excludes(selector) => {
                    output.insert(*selector);
                }
                RuleSelectorPredicate::StatCompare { value, .. } => {
                    value_dependencies(value, &mut output);
                }
                _ => {}
            }
        }
        if let Some(weight) = &self.weight {
            value_dependencies(weight, &mut output);
        }
        output
    }
}

impl CombatCatalog {
    pub(crate) fn needs_selector_snapshots(&self) -> bool {
        self.selectors.values().any(|definition| {
            definition
                .rule_units()
                .is_some_and(|selector| selector.reference() != RuleSelectorReference::CurrentState)
        })
    }
}

fn value_dependencies(expression: &ValueExpr, output: &mut BTreeSet<SelectorId>) {
    match expression {
        ValueExpr::ReadResource { selector, .. } | ValueExpr::SelectorCount(selector) => {
            output.insert(*selector);
        }
        ValueExpr::SelectorSum { selector, value } => {
            output.insert(*selector);
            value_dependencies(value, output);
        }
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs)
        | ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Divide { lhs, rhs, .. } => {
            value_dependencies(lhs, output);
            value_dependencies(rhs, output);
        }
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            value_dependencies(value, output);
            value_dependencies(minimum, output);
            value_dependencies(maximum, output);
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            value_dependencies(value, output);
        }
        ValueExpr::Choose {
            condition,
            when_true,
            when_false,
        } => {
            condition_dependencies(condition, output);
            value_dependencies(when_true, output);
            value_dependencies(when_false, output);
        }
        ValueExpr::Literal(_)
        | ValueExpr::Slot(_)
        | ValueExpr::AbilityParameter { .. }
        | ValueExpr::ReadEventProperty(_)
        | ValueExpr::EventId
        | ValueExpr::EventOwner
        | ValueExpr::EventActor
        | ValueExpr::EventApplier
        | ValueExpr::EventTarget
        | ValueExpr::CurrentTarget
        | ValueExpr::QueryStat { .. }
        | ValueExpr::QueryBaseStat { .. }
        | ValueExpr::QueryShield { .. }
        | ValueExpr::QueryHp { .. }
        | ValueExpr::QueryMaximumEnergy(_)
        | ValueExpr::QueryEffectStacks { .. }
        | ValueExpr::QueryEffectCategoryStacks { .. } => {}
    }
}

fn condition_dependencies(condition: &ConditionExpr, output: &mut BTreeSet<SelectorId>) {
    use crate::rule::model::ConditionExpr;
    match condition {
        ConditionExpr::Not(value) => condition_dependencies(value, output),
        ConditionExpr::All(values) | ConditionExpr::Any(values) => {
            for value in values {
                condition_dependencies(value, output);
            }
        }
        ConditionExpr::Compare { lhs, rhs, .. } => {
            value_dependencies(lhs, output);
            value_dependencies(rhs, output);
        }
        ConditionExpr::SelectorCardinality { selector, .. }
        | ConditionExpr::LifePresence { selector, .. }
        | ConditionExpr::EffectExists { selector, .. }
        | ConditionExpr::IsFrozen(selector)
        | ConditionExpr::HasWeakness { selector, .. }
        | ConditionExpr::IsBroken(selector)
        | ConditionExpr::EnemyRank(selector, _) => {
            output.insert(*selector);
        }
        ConditionExpr::Literal(_)
        | ConditionExpr::EventKind(_)
        | ConditionExpr::SourceTag(_)
        | ConditionExpr::CurrentTargetIsBroken => {}
    }
}
