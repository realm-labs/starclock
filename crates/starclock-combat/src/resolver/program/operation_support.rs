//! Focused construction helpers for evaluated Rule IR operations.

use super::*;

pub(super) fn queue_owner(
    cause: Cause,
    context: &AbilityProgramContext,
    owner: RuleActionOwner,
) -> Result<UnitId, BattleFault> {
    match owner {
        RuleActionOwner::Actor => Some(context.actor),
        RuleActionOwner::CauseOwner => cause.owner(),
        RuleActionOwner::CauseApplier => cause.applier(),
    }
    .ok_or_else(|| program_fault(54, 0))
}

pub(super) fn queue_payment(
    txn: &Transaction<'_>,
    owner: UnitId,
    payment: Option<RuleActionPaymentPolicy>,
) -> Result<Option<SkillPointPaymentPolicy>, BattleFault> {
    payment
        .map(|payment| match payment {
            RuleActionPaymentPolicy::TeamSkillPoints => {
                Ok(SkillPointPaymentPolicy::TeamSkillPoints)
            }
            RuleActionPaymentPolicy::Suppressed => Ok(SkillPointPaymentPolicy::Suppressed),
            RuleActionPaymentPolicy::TeamResource(stable_key) => {
                let side = txn
                    .state
                    .units
                    .get(owner)
                    .ok_or_else(|| program_fault(55, 0))?
                    .side;
                let id = txn
                    .state
                    .teams
                    .get(side)
                    .keyed_by_name(&stable_key)
                    .ok_or_else(|| program_fault(55, 1))?
                    .id;
                Ok(SkillPointPaymentPolicy::TeamResource(id))
            }
        })
        .transpose()
}

pub(super) fn queue_origin(
    catalog: &CombatCatalog,
    ability: AbilityId,
    forced: bool,
) -> Result<ActionOrigin, BattleFault> {
    if forced {
        return Ok(ActionOrigin::Forced);
    }
    let kind = catalog
        .ability(ability)
        .and_then(AbilityDefinition::action)
        .map(AbilityActionDefinition::kind)
        .ok_or_else(|| program_fault(56, i64::from(ability.get())))?;

    use crate::{ActionOrigin as O, catalog::action::AbilityKind as K};
    match kind {
        K::Ultimate => Some(O::UltimateInterrupt),
        K::FollowUp => Some(O::FollowUp),
        K::Counter => Some(O::Counter),
        K::ExtraTurn => Some(O::ExtraTurn),
        K::ExtraAction => Some(O::ExtraAction),
        K::DelayedAction => Some(O::DelayedAction),
        K::Summon => Some(O::SummonAction),
        K::Memosprite => Some(O::MemospriteAction),
        K::Countdown => Some(O::Countdown),
        K::Basic | K::Skill => None,
    }
    .ok_or_else(|| program_fault(57, i64::from(ability.get())))
}

pub(super) fn shift_action(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    targets: Box<[UnitId]>,
    amount: RuleValue,
    advance: bool,
) -> Result<EventId, BattleFault> {
    program_timeline::shift_actions(txn, cause, parent, targets, ratio(amount)?, advance)
}

pub(super) fn replace_ability(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    targets: Box<[UnitId]>,
    old: AbilityId,
    new: AbilityId,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    if catalog.ability(new).is_none() {
        return Err(program_fault(29, i64::from(new.get())));
    }
    for target in targets {
        let state = txn
            .state
            .units
            .get(target)
            .cloned()
            .ok_or_else(|| program_fault(30, 0))?;
        let mut abilities = state.abilities.into_vec();
        if let Ok(index) = abilities.binary_search(&old) {
            abilities[index] = new;
            abilities.sort_unstable();
            abilities.dedup();
        }
        txn.set_unit_definition(
            target,
            state.form,
            abilities.into_boxed_slice(),
            state.presence,
            state.transformation,
        )?;
    }
    Ok(parent)
}

pub(super) fn toughness_reduction(
    element: CombatElement,
    base: RawToughness,
) -> ToughnessReductionDefinition {
    ToughnessReductionDefinition {
        element,
        ignores_weakness: false,
        reduction: ToughnessReductionContext {
            base,
            additive: RawToughness::new(0).expect("zero is valid"),
            reduction_increase: Ratio::ZERO,
            weakness_break_efficiency: Ratio::ZERO,
            weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
            toughness_vulnerability: Ratio::ZERO,
            ability_multiplier: Ratio::ONE,
        },
        break_damage: BreakDamageDefinition {
            attacker_level_multiplier: Scalar::ONE,
            ability_multiplier: Ratio::ONE,
            break_effect: Ratio::ZERO,
            break_damage_increase: Ratio::ZERO,
            defense_multiplier: Ratio::ONE,
            resistance_multiplier: Ratio::ONE,
            vulnerability_multiplier: Ratio::ONE,
            mitigation_multiplier: Ratio::ONE,
            unbroken_multiplier: Ratio::ONE,
        },
        break_effect_chance: Probability::ONE,
    }
}

pub(super) fn super_break(
    _context: &AbilityProgramContext,
    multiplier: Ratio,
    element: CombatElement,
) -> SuperBreakDefinition {
    SuperBreakDefinition {
        element,
        attacker_level_multiplier: Scalar::ONE,
        ability_multiplier: multiplier,
        break_effect: Ratio::ZERO,
        break_damage_increase: Ratio::ZERO,
        super_break_increase: Ratio::ZERO,
        defense_multiplier: Ratio::ONE,
        resistance_multiplier: Ratio::ONE,
        vulnerability_multiplier: Ratio::ONE,
        mitigation_multiplier: Ratio::ONE,
        broken_multiplier: Ratio::ONE,
    }
}
