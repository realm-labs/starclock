use crate::rule::model::RuleEmission;

use super::{AbilityProgramContext, non_negative_scalar, program_fault};

pub(in crate::resolver) fn emission_targets(
    catalog: &crate::catalog::CombatCatalog,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
    selector: crate::SelectorId,
    current_target: Option<crate::UnitId>,
) -> Result<Box<[crate::UnitId]>, crate::BattleFault> {
    let is_current_subject = catalog
        .selector(selector)
        .and_then(crate::catalog::definition::SelectorDefinition::rule_units)
        .is_some_and(|definition| {
            definition.origin() == crate::catalog::selector::RuleSelectorOrigin::CurrentSubject
        });
    if is_current_subject && let Some(target) = current_target {
        return Ok(vec![target].into_boxed_slice());
    }
    resolved
        .binary_search_by_key(&selector, |(id, _)| *id)
        .ok()
        .map(|index| resolved[index].1.clone())
        .ok_or_else(|| program_fault(20, i64::from(selector.get())))
}

pub(super) fn slot_operation(
    context: &AbilityProgramContext,
    id: crate::OperationId,
    slot: crate::StateSlotDefinitionId,
    update: crate::rule::model::StateSlotUpdateKind,
    value: crate::rule::model::RuleValue,
) -> Result<crate::operation::ModifyStateSlotOp, crate::BattleFault> {
    let rule = context.rule.ok_or_else(|| program_fault(52, 0))?;
    let instance = context.rule_instance.ok_or_else(|| program_fault(53, 0))?;
    Ok(crate::operation::ModifyStateSlotOp {
        id,
        owner: context.owner,
        instance: Some(instance),
        definition: crate::rule::model::RuleSlotMutationDefinition {
            rule,
            slot,
            update,
            value,
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn healing_operation(
    catalog: &crate::catalog::CombatCatalog,
    resolved: &[(crate::SelectorId, Box<[crate::UnitId]>)],
    id: crate::OperationId,
    selector: crate::SelectorId,
    amount: crate::rule::model::RuleValue,
    current_target: Option<crate::UnitId>,
    apply_formula_modifiers: bool,
) -> Result<crate::operation::Operation, crate::BattleFault> {
    let amount = non_negative_scalar(amount)?;
    let formula = crate::catalog::action::HealingDefinition::new(
        amount,
        crate::Ratio::ZERO,
        crate::Ratio::ZERO,
        crate::Ratio::ZERO,
    )
    .map_err(|_| {
        program_fault(
            if apply_formula_modifiers { 3 } else { 78 },
            amount.scaled(),
        )
    })?;
    Ok(crate::operation::Operation::Heal(
        crate::operation::HealOp {
            id,
            targets: emission_targets(catalog, resolved, selector, current_target)?,
            formula,
            apply_formula_modifiers,
        },
    ))
}

pub(super) const fn emission_current_target(emission: &RuleEmission) -> Option<crate::UnitId> {
    match emission {
        RuleEmission::SetSlot { current_target, .. }
        | RuleEmission::AddSlot { current_target, .. }
        | RuleEmission::Damage { current_target, .. }
        | RuleEmission::DamageFromActorBasicElement { current_target, .. }
        | RuleEmission::UnboostedDamage { current_target, .. }
        | RuleEmission::RandomRepeatedDamage { current_target, .. }
        | RuleEmission::TrueDamage { current_target, .. }
        | RuleEmission::Heal { current_target, .. }
        | RuleEmission::Shield { current_target, .. }
        | RuleEmission::RemoveShield { current_target, .. }
        | RuleEmission::ConsumeHp { current_target, .. }
        | RuleEmission::ReduceToughness { current_target, .. }
        | RuleEmission::Break { current_target, .. }
        | RuleEmission::SuperBreak { current_target, .. }
        | RuleEmission::AddWeakness { current_target, .. }
        | RuleEmission::RemoveWeakness { current_target, .. }
        | RuleEmission::CreateToughnessLayer { current_target, .. }
        | RuleEmission::RemoveToughnessLayer { current_target, .. }
        | RuleEmission::ModifyResource { current_target, .. }
        | RuleEmission::ApplyEffect { current_target, .. }
        | RuleEmission::ApplyRandomEffect { current_target, .. }
        | RuleEmission::RandomGroupedEffect { current_target, .. }
        | RuleEmission::AdjustEffectStacks { current_target, .. }
        | RuleEmission::RemoveEffect { current_target, .. }
        | RuleEmission::Cleanse { current_target, .. }
        | RuleEmission::DetonateDot { current_target, .. }
        | RuleEmission::ModifyStateSlot { current_target, .. }
        | RuleEmission::AdvanceAction { current_target, .. }
        | RuleEmission::DelayAction { current_target, .. }
        | RuleEmission::QueueAction { current_target, .. }
        | RuleEmission::GrantExtraTurn { current_target, .. }
        | RuleEmission::Summon { current_target, .. }
        | RuleEmission::Despawn { current_target, .. }
        | RuleEmission::Transform { current_target, .. }
        | RuleEmission::ReplaceAbility { current_target, .. }
        | RuleEmission::ChangePresence { current_target, .. }
        | RuleEmission::CreateCountdown { current_target, .. }
        | RuleEmission::Informational { current_target, .. }
        | RuleEmission::Replacement { current_target, .. }
        | RuleEmission::InvokeNative { current_target, .. } => *current_target,
    }
}
