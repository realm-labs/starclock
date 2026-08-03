//! Stable repeated random-element damage emitted by typed Rule IR.

use crate::catalog::CombatCatalog;
use crate::formula::model::CombatElement;
use crate::formula::model::DamageClass;
use crate::{
    EventId, Ratio, SelectorId, UnitId,
    battle::fault::BattleFault,
    catalog::action::{HitCritPolicy, OrdinaryDamageDefinition, OrdinaryDamageMultipliers},
    event::cause::Cause,
    operation::{DamageOp, HitOperationScratch, Operation},
    rng::types::DrawPurpose,
    rule::model::{RuleEvaluationInput, RuleValue},
};

use super::{AbilityProgramContext, emission_targets, non_negative_scalar, program_fault, scale};
use crate::resolver::{operation::execute_operation, transaction::Transaction};

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_random_repeated_damage(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    context: &AbilityProgramContext,
    input: RuleEvaluationInput<'_>,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    selector: SelectorId,
    amount: RuleValue,
    class: DamageClass,
    elements: &[CombatElement],
    minimum_hits: u16,
    maximum_hits: u16,
    count_rng_purpose: DrawPurpose,
    element_rng_purpose: DrawPurpose,
    exclude_event_element: bool,
    can_crit: bool,
    can_defeat: bool,
    current_target: Option<UnitId>,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let candidates = elements
        .iter()
        .copied()
        .filter(|element| !exclude_event_element || Some(*element) != input.event_facts.element)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Err(program_fault(79, 0));
    }
    let hit_count = if minimum_hits == maximum_hits {
        minimum_hits
    } else {
        let width = usize::from(maximum_hits - minimum_hits) + 1;
        let offset = txn
            .choose_index(count_rng_purpose, width)?
            .ok_or_else(|| program_fault(79, i64::try_from(width).unwrap_or(i64::MAX)))?;
        minimum_hits
            .checked_add(u16::try_from(offset).map_err(|_| program_fault(79, 1))?)
            .ok_or_else(|| program_fault(79, 2))?
    };
    let amount = scale(non_negative_scalar(amount)?, context.damage_share)?;
    let formula = OrdinaryDamageDefinition::new(
        amount,
        OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).expect("neutral multipliers are valid"),
    )
    .map_err(|_| program_fault(79, amount.scaled()))?
    .with_class(class);
    let targets = emission_targets(catalog, resolved, selector, current_target)?;
    for _ in 0..hit_count {
        let index = txn
            .choose_index(element_rng_purpose, candidates.len())?
            .ok_or_else(|| program_fault(79, 3))?;
        let element = *candidates
            .get(index)
            .ok_or_else(|| program_fault(79, i64::try_from(index).unwrap_or(i64::MAX)))?;
        let request = Operation::Damage(DamageOp {
            id: txn.allocate_operation(),
            targets: targets.clone(),
            formula,
            element: Some(element),
            crit_policy: if can_crit {
                context.crit_policy
            } else {
                HitCritPolicy::Never
            },
            apply_source_modifiers: true,
            ultimate_semantics: false,
            minimum_hp: i64::from(!can_defeat),
        });
        parent = execute_operation(catalog, txn, cause, parent, request, scratch)?;
    }
    Ok(parent)
}
