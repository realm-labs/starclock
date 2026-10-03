//! Rule-selector cross-reference, ordering and snapshot-safety validation.

use crate::{
    catalog::{
        definition::SelectorDefinition,
        selector::{RuleSelectorReference, RuleUnitSelector},
    },
    modifier::model::StatQuerySubject,
    rule::model::{ConditionExpr, RuleValue, RuleValueKind, ValueExpr},
};
use std::collections::BTreeSet;

use crate::{EffectCategory, SelectorId, catalog::CombatCatalog};

use super::{CatalogBuildError, CatalogBuildErrorKind, error};

pub(super) fn validate(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    use crate::catalog::selector::{
        RuleSelectorChoice, RuleSelectorOrdering, RuleSelectorPredicate,
    };

    for id in catalog.selectors.ids() {
        let Some(selector) = catalog
            .selectors
            .get(id)
            .and_then(SelectorDefinition::rule_units)
        else {
            continue;
        };
        let random = matches!(
            selector.choice(),
            RuleSelectorChoice::RngUniform | RuleSelectorChoice::RngWeighted
        );
        if random
            != selector.rng_purpose().is_some_and(|purpose| {
                matches!(
                    purpose,
                    "bounce-target" | "aggro-target" | "behavior-choice" | "damage-target"
                )
            })
            || selector.repeated() && !random
            || selector.choice() == RuleSelectorChoice::RngWeighted && selector.weight().is_none()
            || matches!(
                selector.ordering(),
                RuleSelectorOrdering::StatAscending | RuleSelectorOrdering::StatDescending
            ) && selector.weight().is_none()
            || selector.weight().is_some()
                && selector.choice() != RuleSelectorChoice::RngWeighted
                && !matches!(
                    selector.ordering(),
                    RuleSelectorOrdering::StatAscending | RuleSelectorOrdering::StatDescending
                )
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!("selector {} has an invalid RNG/order contract", id.get()),
            ));
        }
        if selector.reference() != RuleSelectorReference::CurrentState
            && (selector
                .weight()
                .is_some_and(|expression| !historical_value_safe(expression))
                || selector.predicates().iter().any(|predicate| {
                    matches!(
                        predicate,
                        RuleSelectorPredicate::StatCompare { value, .. }
                            | RuleSelectorPredicate::MaximumValue(value)
                            if !historical_value_safe(value)
                    )
                }))
        {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "selector {} historical expression requires a current-state-only query",
                    id.get()
                ),
            ));
        }
        validate_predicates(catalog, id, selector)?;
        for dependency in selector.dependencies() {
            if catalog
                .selectors
                .get(dependency)
                .and_then(SelectorDefinition::rule_units)
                .is_none()
            {
                return Err(error(
                    CatalogBuildErrorKind::MissingReference,
                    format!(
                        "selector {} expression refers to missing selector {}",
                        id.get(),
                        dependency.get()
                    ),
                ));
            }
        }
    }
    for id in catalog.selectors.ids() {
        validate_dependencies(catalog, id, &mut BTreeSet::new(), &mut BTreeSet::new())?;
    }
    validate_automatic_primaries(catalog)?;
    Ok(())
}

fn validate_automatic_primaries(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    use crate::catalog::{
        action::{TargetPattern, TargetRelation},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorSide,
        },
    };

    for id in catalog.abilities.ids() {
        let ability = catalog
            .abilities
            .get(id)
            .expect("ID originated from this table");
        let Some(primary_id) = ability.automatic_primary_selector() else {
            continue;
        };
        let Some(primary) = catalog
            .selectors
            .get(primary_id)
            .and_then(SelectorDefinition::rule_units)
        else {
            return Err(error(
                CatalogBuildErrorKind::MissingReference,
                format!(
                    "ability {} requires executable primary selector {}",
                    id.get(),
                    primary_id.get()
                ),
            ));
        };
        let target = catalog
            .selectors
            .get(ability.selector())
            .and_then(SelectorDefinition::unit_targets);
        let valid = ability.action().is_some_and(|action| {
            action.kind().is_normal_turn() && action.segmented_flow().is_none()
        }) && target.is_some_and(|target| {
            matches!(
                target.pattern(),
                TargetPattern::Single | TargetPattern::Blast
            ) && match target.relation() {
                TargetRelation::Allied => primary.side() == RuleSelectorSide::Same,
                TargetRelation::Opposing => primary.side() == RuleSelectorSide::Opposing,
                TargetRelation::SelfUnit => false,
            }
        }) && matches!(
            primary.origin(),
            RuleSelectorOrigin::Team | RuleSelectorOrigin::Encounter
        ) && primary.reference() == RuleSelectorReference::CurrentState
            && primary.life() == RuleLifePredicate::Alive
            && primary.presence() == RulePresencePredicate::Present
            && primary.minimum() == 1
            && primary.maximum() == 1
            && primary.empty_pool() == RuleEmptyPoolPolicy::Fault
            && matches!(
                primary.choice(),
                RuleSelectorChoice::First
                    | RuleSelectorChoice::RngUniform
                    | RuleSelectorChoice::RngWeighted
            )
            && !primary.repeated()
            && primary.dependencies().is_empty()
            && primary.weight().is_none_or(primary_value_safe)
            && primary
                .predicates()
                .iter()
                .all(|predicate| match predicate {
                    RuleSelectorPredicate::AdjacentToPrimary => false,
                    RuleSelectorPredicate::StatCompare { value, .. }
                    | RuleSelectorPredicate::MaximumValue(value) => primary_value_safe(value),
                    _ => true,
                });
        if !valid {
            return Err(error(
                CatalogBuildErrorKind::InvalidDefinition,
                format!(
                    "ability {} has an invalid automatic primary selector {}",
                    id.get(),
                    primary_id.get()
                ),
            ));
        }
    }
    Ok(())
}

// Pre-declaration selection has no action/event target, slots or trigger frame.
// Only numeric literals, stat queries and their arithmetic are admitted here.
fn primary_value_safe(expression: &ValueExpr) -> bool {
    match expression {
        ValueExpr::Literal(RuleValue::Scalar(_) | RuleValue::Integer(_)) => true,
        ValueExpr::QueryStat { subject, .. } | ValueExpr::QueryBaseStat { subject, .. } => {
            *subject != StatQuerySubject::EventTarget
        }
        ValueExpr::Negate(value) => primary_value_safe(value),
        ValueExpr::Convert { value, target, .. } => {
            matches!(target, RuleValueKind::Scalar | RuleValueKind::Integer)
                && primary_value_safe(value)
        }
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs)
        | ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Divide { lhs, rhs, .. } => primary_value_safe(lhs) && primary_value_safe(rhs),
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            primary_value_safe(value) && primary_value_safe(minimum) && primary_value_safe(maximum)
        }
        _ => false,
    }
}

fn validate_predicates(
    catalog: &CombatCatalog,
    id: SelectorId,
    selector: &RuleUnitSelector,
) -> Result<(), CatalogBuildError> {
    use crate::catalog::selector::RuleSelectorPredicate;

    for predicate in selector.predicates() {
        match predicate {
            RuleSelectorPredicate::HasMark(effect) | RuleSelectorPredicate::HasEffect(effect)
                if catalog.effects.get(*effect).is_none() =>
            {
                return Err(error(
                    CatalogBuildErrorKind::MissingReference,
                    format!(
                        "selector {} predicate refers to missing effect {}",
                        id.get(),
                        effect.get()
                    ),
                ));
            }
            RuleSelectorPredicate::HasMark(effect)
                if catalog.effects.get(*effect).is_none_or(|definition| {
                    definition
                        .runtime()
                        .map(|runtime| runtime.category())
                        .or_else(|| {
                            definition
                                .runtime_template()
                                .map(|runtime| runtime.category())
                        })
                        != Some(EffectCategory::Mark)
                }) =>
            {
                return Err(error(
                    CatalogBuildErrorKind::InvalidDefinition,
                    format!(
                        "selector {} mark predicate uses non-mark effect {}",
                        id.get(),
                        effect.get()
                    ),
                ));
            }
            RuleSelectorPredicate::OwnedBy(owner) | RuleSelectorPredicate::Excludes(owner)
                if catalog
                    .selectors
                    .get(*owner)
                    .and_then(SelectorDefinition::rule_units)
                    .is_none() =>
            {
                return Err(error(
                    CatalogBuildErrorKind::MissingReference,
                    format!(
                        "selector {} ownership refers to missing selector {}",
                        id.get(),
                        owner.get()
                    ),
                ));
            }
            RuleSelectorPredicate::UnitForm(form) if catalog.units.get(*form).is_none() => {
                return Err(error(
                    CatalogBuildErrorKind::MissingReference,
                    format!(
                        "selector {} predicate refers to missing unit form {}",
                        id.get(),
                        form.get()
                    ),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn historical_value_safe(expression: &ValueExpr) -> bool {
    use crate::rule::model::ValueExpr;

    match expression {
        ValueExpr::ReadResource { .. }
        | ValueExpr::QueryHp { .. }
        | ValueExpr::QueryFormulaStage { .. }
        | ValueExpr::QueryMaximumEnergy(_)
        | ValueExpr::QueryMaximumHp(_)
        | ValueExpr::QueryShield { .. }
        | ValueExpr::QueryEffectShield { .. }
        | ValueExpr::QueryEffectStacks { .. }
        | ValueExpr::QueryEffectCategoryStacks { .. } => false,
        ValueExpr::SelectorSum { value, .. }
        | ValueExpr::Negate(value)
        | ValueExpr::Convert { value, .. } => historical_value_safe(value),
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs)
        | ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Divide { lhs, rhs, .. } => {
            historical_value_safe(lhs) && historical_value_safe(rhs)
        }
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            historical_value_safe(value)
                && historical_value_safe(minimum)
                && historical_value_safe(maximum)
        }
        ValueExpr::Choose {
            condition,
            when_true,
            when_false,
        } => {
            historical_condition_safe(condition)
                && historical_value_safe(when_true)
                && historical_value_safe(when_false)
        }
        ValueExpr::Literal(_)
        | ValueExpr::Slot(_)
        | ValueExpr::AbilityParameter { .. }
        | ValueExpr::ReadEventProperty(_)
        | ValueExpr::SelectorCount(_)
        | ValueExpr::EventId
        | ValueExpr::EventOwner
        | ValueExpr::EventActor
        | ValueExpr::EventApplier
        | ValueExpr::EventTarget
        | ValueExpr::CurrentTarget
        | ValueExpr::QueryStat { .. }
        | ValueExpr::QueryBaseStat { .. } => true,
    }
}

fn historical_condition_safe(condition: &ConditionExpr) -> bool {
    use crate::rule::model::ConditionExpr;

    match condition {
        ConditionExpr::LifePresence { .. }
        | ConditionExpr::EffectExists { .. }
        | ConditionExpr::IsFrozen(_)
        | ConditionExpr::HasWeakness { .. }
        | ConditionExpr::IsBroken(_)
        | ConditionExpr::CurrentTargetIsBroken
        | ConditionExpr::HighestDamageDealer(_)
        | ConditionExpr::EnemyRank { .. }
        | ConditionExpr::EnemyRankEliteOrBoss { .. } => false,
        ConditionExpr::Not(value) => historical_condition_safe(value),
        ConditionExpr::All(values) | ConditionExpr::Any(values) => {
            values.iter().all(historical_condition_safe)
        }
        ConditionExpr::Compare { lhs, rhs, .. } => {
            historical_value_safe(lhs) && historical_value_safe(rhs)
        }
        ConditionExpr::Literal(_)
        | ConditionExpr::EventKind(_)
        | ConditionExpr::SourceTag(_)
        | ConditionExpr::SelectorCardinality { .. } => true,
    }
}

fn validate_dependencies(
    catalog: &CombatCatalog,
    id: SelectorId,
    visiting: &mut BTreeSet<SelectorId>,
    visited: &mut BTreeSet<SelectorId>,
) -> Result<(), CatalogBuildError> {
    if visited.contains(&id) {
        return Ok(());
    }
    if !visiting.insert(id) {
        return Err(error(
            CatalogBuildErrorKind::InvalidDefinition,
            format!("selector ownership cycle at {}", id.get()),
        ));
    }
    if let Some(selector) = catalog
        .selectors
        .get(id)
        .and_then(SelectorDefinition::rule_units)
    {
        for dependency in selector.dependencies() {
            validate_dependencies(catalog, dependency, visiting, visited)?;
        }
    }
    visiting.remove(&id);
    visited.insert(id);
    Ok(())
}
