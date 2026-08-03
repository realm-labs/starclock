//! Generated Sora selector rows to typed combat selector plans.

use crate::{
    effect_lower::lower_element,
    generated::{SoraConfig, selector::Selector, selector_predicate_node::SelectorPredicateNode},
    modifier_lower::{expression, stat as lower_stat},
    rule_lower::lower_comparison,
};
use std::collections::BTreeSet;

use crate::catalog::{CatalogLoadError, domain_fail, positive};

#[derive(Debug)]
pub(super) struct SelectorDataDefinition {
    pub(super) id: starclock_combat::SelectorId,
    pub(super) units: starclock_combat::catalog::selector::RuleUnitSelector,
}

pub(super) fn lower(
    config: &SoraConfig,
    row: &Selector,
) -> Result<SelectorDataDefinition, CatalogLoadError> {
    use crate::generated::{
        empty_pool_policy::EmptyPoolPolicy as E, life_predicate::LifePredicate as L,
        presence_predicate::PresencePredicate as P, selector_choice::SelectorChoice as C,
        selector_ordering::SelectorOrdering as O, selector_origin::SelectorOrigin as G,
        selector_reference_point::SelectorReferencePoint as R,
        side_relationship::SideRelationship as S,
    };
    use starclock_combat::catalog::selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    };
    let mut predicate_rows = config
        .selector_predicate()
        .iter()
        .filter(|predicate| predicate.selector_id == row.id)
        .collect::<Vec<_>>();
    predicate_rows.sort_unstable_by_key(|predicate| predicate.sequence);
    let predicates = predicate_rows
        .into_iter()
        .map(|predicate| lower_predicate(config, &predicate.predicate))
        .collect::<Result<Vec<_>, _>>()?;
    let weight = row
        .weight_expression_id
        .map(|id| expression(config, id, &mut BTreeSet::new()))
        .transpose()?;
    let units = RuleUnitSelector::new(
        match row.origin {
            G::Source => RuleSelectorOrigin::Source,
            G::Owner => RuleSelectorOrigin::Owner,
            G::Actor => RuleSelectorOrigin::Actor,
            G::Applier => RuleSelectorOrigin::Applier,
            G::PrimaryTarget => RuleSelectorOrigin::PrimaryTarget,
            G::CurrentSubject => RuleSelectorOrigin::CurrentSubject,
            G::Team => RuleSelectorOrigin::Team,
            G::Encounter => RuleSelectorOrigin::Encounter,
        },
        match row.side_relationship {
            S::SameSide => RuleSelectorSide::Same,
            S::OpposingSide => RuleSelectorSide::Opposing,
            S::AnySide => RuleSelectorSide::Any,
        },
        match row.life {
            L::Any => RuleLifePredicate::Any,
            L::Alive => RuleLifePredicate::Alive,
            L::Downed => RuleLifePredicate::Downed,
            L::Defeated => RuleLifePredicate::Defeated,
        },
        match row.presence {
            P::Any => RulePresencePredicate::Any,
            P::Present => RulePresencePredicate::Present,
            P::Reserved => RulePresencePredicate::Reserved,
            P::Departed => RulePresencePredicate::Departed,
            P::Untargetable => RulePresencePredicate::Untargetable,
            P::Linked => RulePresencePredicate::Linked,
            P::Transformed => RulePresencePredicate::Transformed,
        },
        match row.reference_point {
            R::CurrentState => RuleSelectorReference::CurrentState,
            R::EventSnapshot => RuleSelectorReference::EventSnapshot,
            R::ActionSnapshot => RuleSelectorReference::ActionSnapshot,
        },
        match row.ordering {
            O::Formation => RuleSelectorOrdering::Formation,
            O::Timeline => RuleSelectorOrdering::Timeline,
            O::HpRatioAscending => RuleSelectorOrdering::HpRatioAscending,
            O::HpRatioDescending => RuleSelectorOrdering::HpRatioDescending,
            O::StatAscending => RuleSelectorOrdering::StatAscending,
            O::StatDescending => RuleSelectorOrdering::StatDescending,
            O::EventOrder => RuleSelectorOrdering::EventOrder,
            O::StableId => RuleSelectorOrdering::StableId,
        },
        u16::try_from(row.minimum_count).map_err(domain_fail)?,
        u16::try_from(row.maximum_count).map_err(domain_fail)?,
        match row.empty_pool_policy {
            E::NoOp => RuleEmptyPoolPolicy::NoOp,
            E::Skip => RuleEmptyPoolPolicy::Skip,
            E::CancelRemaining => RuleEmptyPoolPolicy::CancelRemaining,
            E::Fault => RuleEmptyPoolPolicy::Fault,
        },
        match row.choice {
            C::All => RuleSelectorChoice::All,
            C::First => RuleSelectorChoice::First,
            C::PrimaryPlusAdjacent => RuleSelectorChoice::PrimaryPlusAdjacent,
            C::RngUniform => RuleSelectorChoice::RngUniform,
            C::RngWeighted => RuleSelectorChoice::RngWeighted,
        },
        row.rng_purpose_key.as_deref().map(Into::into),
        row.allow_repeated_targets,
    )
    .ok_or_else(|| domain_fail(format!("selector {} has invalid bounds", row.id)))?
    .with_predicates(predicates)
    .with_weight(weight);
    Ok(SelectorDataDefinition {
        id: starclock_combat::SelectorId::new(positive(row.id, "Selector.id")?)
            .ok_or_else(|| domain_fail("selector ID is zero"))?,
        units,
    })
}

fn lower_predicate(
    config: &SoraConfig,
    node: &SelectorPredicateNode,
) -> Result<starclock_combat::catalog::selector::RuleSelectorPredicate, CatalogLoadError> {
    use crate::generated::selector_predicate_node::SelectorPredicateNode as Node;
    use starclock_combat::catalog::selector::RuleSelectorPredicate as Predicate;
    Ok(match node {
        Node::FormationRange {
            minimum_index,
            maximum_index,
        } => {
            let minimum = u8::try_from(*minimum_index).map_err(domain_fail)?;
            let maximum = u8::try_from(*maximum_index).map_err(domain_fail)?;
            if minimum > maximum {
                return Err(domain_fail("selector formation range is inverted"));
            }
            Predicate::FormationRange { minimum, maximum }
        }
        Node::HasMark { effect_id } => Predicate::HasMark(effect(*effect_id)?),
        Node::HasWeakness { element } => Predicate::HasWeakness(lower_element(*element)),
        Node::HasEffect { effect_id } => Predicate::HasEffect(effect(*effect_id)?),
        Node::HasTag { tag } => {
            let identity = config
                .content_identity()
                .get_by_stable_key(tag)
                .ok_or_else(|| {
                    domain_fail(format!("selector tag {tag} has no content identity"))
                })?;
            Predicate::HasTag(
                starclock_combat::SourceDefinitionId::new(positive(
                    identity.id,
                    "SelectorPredicate.HasTag",
                )?)
                .expect("positive source ID"),
            )
        }
        Node::OwnedBy { owner_selector_id } => Predicate::OwnedBy(selector(*owner_selector_id)?),
        Node::Excludes {
            excluded_selector_id,
        } => Predicate::Excludes(selector(*excluded_selector_id)?),
        Node::StatCompare {
            stat,
            comparison,
            value_expression_id,
        } => Predicate::StatCompare {
            stat: lower_stat(*stat),
            comparison: lower_comparison(*comparison),
            value: expression(config, *value_expression_id, &mut BTreeSet::new())?,
        },
    })
}

fn selector(raw: i32) -> Result<starclock_combat::SelectorId, CatalogLoadError> {
    starclock_combat::SelectorId::new(positive(raw, "SelectorPredicate.selector")?)
        .ok_or_else(|| domain_fail("selector predicate selector ID is zero"))
}

fn effect(raw: i32) -> Result<starclock_combat::EffectDefinitionId, CatalogLoadError> {
    starclock_combat::EffectDefinitionId::new(positive(raw, "SelectorPredicate.effect")?)
        .ok_or_else(|| domain_fail("selector predicate effect ID is zero"))
}
