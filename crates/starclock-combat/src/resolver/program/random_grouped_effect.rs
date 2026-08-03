//! Deterministic grouped random-target effect application for typed Rule IR.

use crate::{
    EffectDefinitionId, EventId, SelectorId, UnitId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    event::cause::Cause,
    operation::HitOperationScratch,
    rng::types::DrawPurpose,
    rule::model::{RuleEffectChancePolicy, RuleEvaluationInput, RuleValue},
};

use super::{emission_targets, program_fault};
use crate::resolver::{
    operation::execute_operation, program_effect::apply_effect_operation, transaction::Transaction,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn execute_random_grouped_effect(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    input: RuleEvaluationInput<'_>,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    selector: SelectorId,
    effect: EffectDefinitionId,
    groups: RuleValue,
    applications_per_group: u16,
    stacks: RuleValue,
    choice_rng_purpose: DrawPurpose,
    chance: RuleEffectChancePolicy,
    base_chance: Option<RuleValue>,
    chance_rng_purpose: Option<DrawPurpose>,
    current_target: Option<UnitId>,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let RuleValue::Integer(groups) = groups else {
        return Err(program_fault(80, 0));
    };
    if !(0..=64).contains(&groups) {
        return Err(program_fault(80, groups));
    }
    if groups == 0 {
        return Ok(parent);
    }
    let candidates = emission_targets(catalog, resolved, selector, current_target)?.into_vec();
    if candidates.is_empty() {
        return Ok(parent);
    }
    let draw_count = usize::from(applications_per_group).min(candidates.len());
    for _ in 0..groups {
        let mut pool = candidates.clone();
        for _ in 0..draw_count {
            let index = txn
                .choose_index(choice_rng_purpose, pool.len())?
                .ok_or_else(|| program_fault(80, 1))?;
            let target = pool.remove(index);
            let selected = [(selector, vec![target].into_boxed_slice())];
            let operation = apply_effect_operation(
                catalog,
                input,
                txn.allocate_operation(),
                &selected,
                selector,
                current_target,
                effect,
                stacks.clone(),
                chance,
                base_chance.clone(),
                chance_rng_purpose,
            )?;
            parent = execute_operation(catalog, txn, cause, parent, operation, scratch)?;
        }
    }
    Ok(parent)
}
