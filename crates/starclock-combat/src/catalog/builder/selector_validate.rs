//! Rule-selector cross-reference, ordering and snapshot-safety validation.

use std::collections::BTreeSet;

use crate::{SelectorId, catalog::CombatCatalog};

use super::{CatalogBuildError, CatalogBuildErrorKind, error};

pub(super) fn validate(catalog: &CombatCatalog) -> Result<(), CatalogBuildError> {
    use crate::catalog::selector::{
        RuleSelectorChoice, RuleSelectorOrdering, RuleSelectorPredicate,
    };

    for id in catalog.selectors.ids() {
        let Some(selector) = catalog
            .selectors
            .get(id)
            .and_then(crate::catalog::definition::SelectorDefinition::rule_units)
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
                    "bounce-target" | "aggro-target" | "behavior-choice"
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
        if selector.reference() != crate::catalog::selector::RuleSelectorReference::CurrentState
            && (selector
                .weight()
                .is_some_and(|expression| !historical_value_safe(expression))
                || selector.predicates().iter().any(|predicate| {
                    matches!(
                        predicate,
                        RuleSelectorPredicate::StatCompare { value, .. }
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
                .and_then(crate::catalog::definition::SelectorDefinition::rule_units)
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
    Ok(())
}

fn validate_predicates(
    catalog: &CombatCatalog,
    id: SelectorId,
    selector: &crate::catalog::selector::RuleUnitSelector,
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
                        != Some(crate::EffectCategory::Mark)
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
            RuleSelectorPredicate::OwnedBy(owner)
                if catalog
                    .selectors
                    .get(*owner)
                    .and_then(crate::catalog::definition::SelectorDefinition::rule_units)
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
            _ => {}
        }
    }
    Ok(())
}

fn historical_value_safe(expression: &crate::rule::model::ValueExpr) -> bool {
    use crate::rule::model::ValueExpr;

    match expression {
        ValueExpr::ReadResource { .. } => false,
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
        | ValueExpr::QueryStat { .. } => true,
    }
}

fn historical_condition_safe(condition: &crate::rule::model::ConditionExpr) -> bool {
    use crate::rule::model::ConditionExpr;

    match condition {
        ConditionExpr::LifePresence { .. }
        | ConditionExpr::EffectExists { .. }
        | ConditionExpr::HasWeakness { .. }
        | ConditionExpr::IsBroken(_) => false,
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
        .and_then(crate::catalog::definition::SelectorDefinition::rule_units)
    {
        for dependency in selector.dependencies() {
            validate_dependencies(catalog, dependency, visiting, visited)?;
        }
    }
    visiting.remove(&id);
    visited.insert(id);
    Ok(())
}
