//! Generated Sora selector rows to typed combat selector plans.

use crate::catalog::{CatalogLoadError, domain_fail, positive};

#[derive(Debug)]
pub(super) struct SelectorDataDefinition {
    pub(super) id: starclock_combat::SelectorId,
    pub(super) units: starclock_combat::catalog::selector::RuleUnitSelector,
}

pub(super) fn lower(
    row: &crate::generated::selector::Selector,
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
    if row.weight_expression_id.is_some() {
        return Err(domain_fail(format!(
            "selector {} uses weighted choice before selector expressions are executable",
            row.id
        )));
    }
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
    .ok_or_else(|| domain_fail(format!("selector {} has invalid bounds", row.id)))?;
    Ok(SelectorDataDefinition {
        id: starclock_combat::SelectorId::new(positive(row.id, "Selector.id")?)
            .ok_or_else(|| domain_fail("selector ID is zero"))?,
        units,
    })
}
