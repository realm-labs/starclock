use crate::{
    BattleFault, OperationId, Ratio, SelectorId, StateSlotDefinitionId, UnitId,
    catalog::{
        CombatCatalog,
        action::{AbilityKind, HealingDefinition, HitOperationDefinition},
        definition::SelectorDefinition,
        selector::RuleSelectorOrigin,
    },
    formula::model::CombatElement,
    operation::{HealOp, ModifyStateSlotOp, Operation},
    resolver::transaction::Transaction,
    rule::model::{RuleEmission, RuleSlotMutationDefinition, RuleValue, StateSlotUpdateKind},
};

use super::{AbilityProgramContext, non_negative_scalar, program_fault};

pub(in crate::resolver) fn actor_basic_element(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    actor: UnitId,
) -> Result<CombatElement, BattleFault> {
    let unit = txn
        .state
        .units
        .get(actor)
        .ok_or_else(|| program_fault(84, 0))?;
    let mut element = None;
    for ability in unit.abilities.iter().filter_map(|id| catalog.ability(*id)) {
        let Some(action) = ability.action() else {
            continue;
        };
        if action.kind() != AbilityKind::Basic {
            continue;
        }
        for authored in action.hits().iter().flat_map(|hit| hit.operations()) {
            let HitOperationDefinition::ScalingDamage(damage) = authored else {
                continue;
            };
            match element {
                None => element = Some(damage.element()),
                Some(current) if current == damage.element() => {}
                Some(_) => return Err(program_fault(85, i64::from(ability.id().get()))),
            }
        }
    }
    element.ok_or_else(|| program_fault(86, i64::try_from(actor.get()).unwrap_or(i64::MAX)))
}

pub(in crate::resolver) fn emission_targets(
    catalog: &CombatCatalog,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    selector: SelectorId,
    current_target: Option<UnitId>,
) -> Result<Box<[UnitId]>, BattleFault> {
    let is_current_subject = catalog
        .selector(selector)
        .and_then(SelectorDefinition::rule_units)
        .is_some_and(|definition| definition.origin() == RuleSelectorOrigin::CurrentSubject);
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
    id: OperationId,
    slot: StateSlotDefinitionId,
    update: StateSlotUpdateKind,
    value: RuleValue,
) -> Result<ModifyStateSlotOp, BattleFault> {
    let rule = context.rule.ok_or_else(|| program_fault(52, 0))?;
    let instance = context.rule_instance.ok_or_else(|| program_fault(53, 0))?;
    Ok(ModifyStateSlotOp {
        id,
        owner: context.owner,
        instance: Some(instance),
        definition: RuleSlotMutationDefinition {
            rule,
            slot,
            update,
            value,
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn healing_operation(
    catalog: &CombatCatalog,
    resolved: &[(SelectorId, Box<[UnitId]>)],
    id: OperationId,
    selector: SelectorId,
    amount: RuleValue,
    current_target: Option<UnitId>,
    apply_formula_modifiers: bool,
) -> Result<Operation, BattleFault> {
    let amount = non_negative_scalar(amount)?;
    let formula =
        HealingDefinition::new(amount, Ratio::ZERO, Ratio::ZERO, Ratio::ZERO).map_err(|_| {
            program_fault(
                if apply_formula_modifiers { 3 } else { 78 },
                amount.scaled(),
            )
        })?;
    Ok(Operation::Heal(HealOp {
        id,
        targets: emission_targets(catalog, resolved, selector, current_target)?,
        formula,
        apply_formula_modifiers,
    }))
}

pub(super) const fn emission_current_target(emission: &RuleEmission) -> Option<UnitId> {
    match emission {
        RuleEmission::SetSlot { current_target, .. }
        | RuleEmission::AddSlot { current_target, .. }
        | RuleEmission::Damage { current_target, .. }
        | RuleEmission::DamageFromActorBasicElement { current_target, .. }
        | RuleEmission::UltimateDamageFromActorBasicElement { current_target, .. }
        | RuleEmission::UnboostedDamage { current_target, .. }
        | RuleEmission::RandomRepeatedDamage { current_target, .. }
        | RuleEmission::RandomRepeatedTrueDamage { current_target, .. }
        | RuleEmission::TrueDamage { current_target, .. }
        | RuleEmission::Heal { current_target, .. }
        | RuleEmission::Shield { current_target, .. }
        | RuleEmission::RemoveShield { current_target, .. }
        | RuleEmission::ConsumeHp { current_target, .. }
        | RuleEmission::ReduceToughness { current_target, .. }
        | RuleEmission::Break { current_target, .. }
        | RuleEmission::SuperBreak { current_target, .. }
        | RuleEmission::AddWeakness { current_target, .. }
        | RuleEmission::AddWeaknessFromAlliedElements { current_target, .. }
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
