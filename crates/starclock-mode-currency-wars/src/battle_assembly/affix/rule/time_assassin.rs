//! Time Assassin's attack-driven Action Value deduction.

use starclock_combat::{
    SelectorId,
    catalog::{
        definition::{ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, RuleEventPoint, RuleOperationTemplate, TriggerDef,
    },
};

use crate::{
    CurrencyWarsEnemyAffixBehavior,
    battle_assembly::{CurrencyWarsBattleAssemblyError, CurrencyWarsBattleResources, error},
};

use super::{operation, players, program_definition, program_id_for, scalar_parameter, trigger};

const SELECTOR: u32 = 0x7d80_0079;
const STABLE_KEY: &str = "enemy.time-assassin.minionlv2.variant.01";

pub(super) fn compile(
    resources: &CurrencyWarsBattleResources,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    resources
        .enemy_form(STABLE_KEY)
        .ok_or_else(|| error("Currency Wars Time Assassin combat form is missing"))?;
    let program = program_id_for(behavior, 1)?;
    programs.push(program_definition(
        program,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::DeductActionValue {
            amount: scalar_parameter(behavior, 0)?,
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::ActionResolved,
        EventFilter {
            actor_selector: Some(selector_id()),
            target_selector: Some(players()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::Action,
        program,
    )?);
    Ok(())
}

pub(super) fn selectors(
    resources: &CurrencyWarsBattleResources,
) -> Result<Vec<SelectorDefinition>, CurrencyWarsBattleAssemblyError> {
    let form = resources
        .enemy_form(STABLE_KEY)
        .ok_or_else(|| error("Currency Wars Time Assassin combat form is missing"))?;
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars Time Assassin selector is invalid"))?
    .with_predicates(vec![RuleSelectorPredicate::UnitForm(form)]);
    Ok(vec![
        SelectorDefinition::new(selector_id()).with_rule_units(selector),
    ])
}

pub(super) fn selector_ids() -> Vec<SelectorId> {
    vec![selector_id()]
}

fn selector_id() -> SelectorId {
    SelectorId::new(SELECTOR).expect("reserved Time Assassin selector ID is non-zero")
}
