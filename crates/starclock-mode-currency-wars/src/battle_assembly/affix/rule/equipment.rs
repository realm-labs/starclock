//! Equipment-count predicates and reactions for enemy Affixes.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use starclock_combat::{
    ResolvedCombatantSpec, Rounding, Scalar, SelectorId, SourceDefinitionId,
    catalog::{
        definition::{ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    rule::model::{
        ConditionExpr, EventFilter, EventValueProperty, OnceScope, RuleEventPoint,
        RuleOperationTemplate, RuleSource, RuleValue, SourceClass, TriggerDef, ValueExpr,
    },
};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsEnemyAffixBehavior,
    CurrencyWarsEnemyAffixSemantic, CurrencyWarsRoleId,
    battle_assembly::{
        CurrencyWarsBattleAssemblyError, combatant_overlay::attach_source_tag, error,
    },
};

use super::{
    SOURCE_ID, enemies, operation, program_definition, program_id_for, scalar_parameter, source_id,
    trigger,
};

const SELECTOR_BASE: u32 = 0x7d80_0020;
const TAG_BASE: u32 = 0x7d80_0030;

pub(super) fn tag_combatants(
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &mut BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    if !snapshot
        .enemy_affix_behaviors
        .iter()
        .any(|behavior| behavior.semantic == CurrencyWarsEnemyAffixSemantic::ExtraStrike)
    {
        return Ok(());
    }
    for role in &snapshot.roles {
        let Some(combatant) = combatants.get(&role.role.id) else {
            continue;
        };
        let missing = 3_u8.saturating_sub(u8::try_from(role.equipment.len()).unwrap_or(u8::MAX));
        if missing == 0 {
            continue;
        }
        let source = tag_source(snapshot.digest.bytes(), missing)?;
        let replacement = attach_source_tag(
            combatant,
            source,
            b"starclock.currency-wars.enemy-affix-equipment-tag.v1",
            tag_digest(snapshot.digest.bytes(), missing),
        )?;
        combatants.insert(role.role.id, replacement);
    }
    Ok(())
}

pub(super) fn compile_extra_strike(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    for missing in 1_u8..=3 {
        let program = program_id_for(behavior, u32::from(missing))?;
        let coefficient = ValueExpr::Multiply {
            lhs: Box::new(scalar_parameter(behavior, 0)?),
            rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
                Scalar::checked_from_integer(i64::from(missing))
                    .expect("equipment slot count is a small scalar"),
            ))),
            rounding: Rounding::NearestTiesAway,
        };
        programs.push(program_definition(
            program,
            Vec::new(),
            Vec::new(),
            vec![operation(RuleOperationTemplate::TrueDamage {
                selector: missing_selector(missing)?,
                amount: ValueExpr::Multiply {
                    lhs: Box::new(ValueExpr::ReadEventProperty(
                        EventValueProperty::DamageAmount,
                    )),
                    rhs: Box::new(coefficient),
                    rounding: Rounding::NearestTiesAway,
                },
            })],
        ));
        triggers.push(trigger(
            behavior,
            10 + u32::from(missing),
            RuleEventPoint::DamageApplied,
            EventFilter {
                actor_selector: Some(enemies()),
                target_selector: Some(missing_selector(missing)?),
                excluded_source: Some(source_id(SOURCE_ID)?),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            OnceScope::Event,
            program,
        )?);
    }
    Ok(())
}

pub(super) fn selectors() -> Result<Vec<SelectorDefinition>, CurrencyWarsBattleAssemblyError> {
    (1_u8..=3).map(selector_definition).collect()
}

pub(super) fn selector_ids() -> Vec<SelectorId> {
    (1_u8..=3)
        .map(|missing| {
            missing_selector(missing).expect("reserved equipment selector IDs are non-zero")
        })
        .collect()
}

fn selector_definition(missing: u8) -> Result<SelectorDefinition, CurrencyWarsBattleAssemblyError> {
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        32,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars equipment Affix selector is invalid"))?
    .with_predicates(vec![RuleSelectorPredicate::HasTag(tag_id(missing)?)]);
    Ok(SelectorDefinition::new(missing_selector(missing)?).with_rule_units(selector))
}

fn missing_selector(missing: u8) -> Result<SelectorId, CurrencyWarsBattleAssemblyError> {
    SELECTOR_BASE
        .checked_add(u32::from(missing))
        .and_then(SelectorId::new)
        .ok_or_else(|| error("Currency Wars equipment Affix selector ID is invalid"))
}

fn tag_id(missing: u8) -> Result<SourceDefinitionId, CurrencyWarsBattleAssemblyError> {
    TAG_BASE
        .checked_add(u32::from(missing))
        .and_then(SourceDefinitionId::new)
        .ok_or_else(|| error("Currency Wars equipment Affix source tag ID is invalid"))
}

fn tag_source(root: [u8; 32], missing: u8) -> Result<RuleSource, CurrencyWarsBattleAssemblyError> {
    Ok(RuleSource::new(
        tag_id(missing)?,
        SourceClass::Mode,
        Vec::new(),
        tag_digest(root, missing),
    ))
}

fn tag_digest(root: [u8; 32], missing: u8) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"starclock.currency-wars.enemy-affix-equipment-tag.v1");
    hash.update(root);
    hash.update([missing]);
    hash.finalize().into()
}
