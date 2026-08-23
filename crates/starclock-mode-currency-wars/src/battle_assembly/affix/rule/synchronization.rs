//! Action-advance synchronization enemy Affix.

use starclock_combat::{
    ActionGaugeChangeKind, SelectorId,
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
    battle_assembly::{CurrencyWarsBattleAssemblyError, error},
};

use super::{operation, players, program_definition, program_id_for, scalar_parameter, trigger};

const ELITE_OR_BOSS_SELECTOR: u32 = 0x7d80_0042;

pub(super) fn compile(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let program = program_id_for(behavior, 1)?;
    programs.push(program_definition(
        program,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::AdvanceAction {
            selector: elite_or_boss(),
            amount: scalar_parameter(behavior, 0)?,
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::TimelineChanged,
        EventFilter {
            target_selector: Some(players()),
            action_gauge_change: Some(ActionGaugeChangeKind::Advance),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::Event,
        program,
    )?);
    Ok(())
}

pub(super) fn selectors() -> Result<Vec<SelectorDefinition>, CurrencyWarsBattleAssemblyError> {
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("behavior-choice".into()),
        false,
    )
    .ok_or_else(|| error("Currency Wars synchronized enemy selector is invalid"))?
    .with_predicates(vec![RuleSelectorPredicate::EnemyRankEliteOrBoss]);
    Ok(vec![
        SelectorDefinition::new(elite_or_boss()).with_rule_units(selector),
    ])
}

pub(super) fn selector_ids() -> Vec<SelectorId> {
    vec![elite_or_boss()]
}

fn elite_or_boss() -> SelectorId {
    SelectorId::new(ELITE_OR_BOSS_SELECTOR).expect("reserved Elite or Boss selector ID is non-zero")
}
