//! Defeat buffs and damage retaliation enemy Affixes.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId, Rounding,
    Scalar, SelectorId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::toughness::EnemyRank,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, RuleEffectChancePolicy, RuleEventPoint,
        RuleOperationTemplate, RuleValue, RuleValueKind, TriggerDef, ValueExpr,
    },
};

use crate::{
    CurrencyWarsEnemyAffixBehavior,
    battle_assembly::{CurrencyWarsBattleAssemblyError, debug_error, error},
};

use super::{
    actor, apply_effect, definition_raw, enemies, integer_parameter, integer_value, operation,
    players, program_definition, program_id_for, scalar_parameter, trigger,
};

const DEFEATED_ENEMY_SELECTOR: u32 = 0x7d80_0040;
const DAMAGE_PURPOSES: [FormulaPurpose; 7] = [
    FormulaPurpose::OrdinaryDamage,
    FormulaPurpose::Dot,
    FormulaPurpose::Break,
    FormulaPurpose::SuperBreak,
    FormulaPurpose::AdditionalDamage,
    FormulaPurpose::JointDamage,
    FormulaPurpose::ElationDamage,
];

pub(super) fn compile_blazing_vengeance(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let effect = effect_id(behavior, 20)?;
    let group = ModifierStackingGroupId::new(definition_raw(behavior, 21)?)
        .ok_or_else(|| error("Currency Wars vengeance modifier group ID is invalid"))?;
    builder.add_modifier_group(ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    });
    let stack_value = ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::Convert {
            value: Box::new(ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::Owner,
                effect,
            }),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesAway,
        }),
        rhs: Box::new(scalar_parameter(behavior, 0)?),
        rounding: Rounding::NearestTiesAway,
    };
    let mut modifiers = Vec::new();
    for (index, purpose) in DAMAGE_PURPOSES.into_iter().enumerate() {
        let offset = 22_u32
            .checked_add(u32::try_from(index).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars vengeance modifier ID overflow"))?;
        let id = ModifierDefinitionId::new(definition_raw(behavior, offset)?)
            .ok_or_else(|| error("Currency Wars vengeance modifier ID is invalid"))?;
        builder.add_modifier(ModifierDefinition {
            id,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose,
            value: stack_value.clone(),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::RecomputeOnStackChange,
            source_stack_slot: None,
            filters: Box::new([ModifierFilter::FormulaSubject(FormulaSubject::Source)]),
        });
        modifiers.push(id);
    }
    let duration = u16::try_from(integer_parameter(behavior, 1)?).map_err(debug_error)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        32,
        Some(integer_value(u32::from(duration))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or_else(|| error("Currency Wars vengeance effect is invalid"))?;
    builder.add_effect(
        EffectDefinition::new(effect, Vec::new(), modifiers).with_runtime_template(runtime),
    );
    let program = program_id_for(behavior, 1)?;
    programs.push(program_definition(
        program,
        Vec::new(),
        vec![effect],
        vec![apply_effect(
            enemies(),
            effect,
            RuleEffectChancePolicy::Guaranteed,
            None,
        )],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::UnitDefeated,
        EventFilter {
            target_selector: Some(defeated_enemy()),
            ..EventFilter::default()
        },
        ConditionExpr::Not(Box::new(ConditionExpr::EnemyRank(
            defeated_enemy(),
            EnemyRank::Boss,
        ))),
        OnceScope::Event,
        program,
    )?);
    Ok(())
}

pub(super) fn compile_self_defense(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let program = program_id_for(behavior, 1)?;
    // The released combat text specifies 120% ATK; AffixConfig's sole `1`
    // parameter is not that damage coefficient.
    let released_attack_ratio =
        ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(1_200_000)));
    programs.push(program_definition(
        program,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::NonlethalTrueDamage {
            selector: actor(),
            amount: ValueExpr::Multiply {
                lhs: Box::new(ValueExpr::QueryStat {
                    subject: StatQuerySubject::EventTarget,
                    stat: StatKind::Atk,
                    purpose: FormulaPurpose::Stat,
                }),
                rhs: Box::new(released_attack_ratio),
                rounding: Rounding::NearestTiesAway,
            },
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::DamageApplied,
        EventFilter {
            actor_selector: Some(players()),
            target_selector: Some(enemies()),
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
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Defeated,
        RulePresencePredicate::Any,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars defeated enemy selector is invalid"))?;
    Ok(vec![
        SelectorDefinition::new(defeated_enemy()).with_rule_units(selector),
    ])
}

pub(super) fn selector_ids() -> Vec<SelectorId> {
    vec![defeated_enemy()]
}

fn defeated_enemy() -> SelectorId {
    SelectorId::new(DEFEATED_ENEMY_SELECTOR).expect("reserved defeated enemy selector is non-zero")
}

fn effect_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<EffectDefinitionId, CurrencyWarsBattleAssemblyError> {
    EffectDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars vengeance effect ID is invalid"))
}
