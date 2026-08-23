//! Timed healing, shielding, aggro, and DoT penalties from enemy Affixes.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectSnapshotPolicy, EffectStackPolicy, EffectTickPhase, ModifierDefinitionId,
    ModifierStackingGroupId, Rounding, SelectorId,
    catalog::{
        builder::CombatCatalogBuilder,
        definition::{EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::CombatElement,
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, RuleEffectChancePolicy, RuleEventPoint, TriggerDef,
        ValueExpr,
    },
};

use crate::{
    CurrencyWarsEnemyAffixBehavior,
    battle_assembly::{CurrencyWarsBattleAssemblyError, debug_error, error},
};

use super::{
    apply_effect, definition_raw, enemies, event_target, integer_parameter, integer_value, players,
    program_definition, program_id_for, scalar_parameter, trigger,
};

const FIRST_PLAYER_SELECTOR: u32 = 0x7d80_0041;

pub(super) fn compile_critical_conundrum(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let effect = effect_id(behavior, 20)?;
    let group = modifier_group_id(behavior, 21)?;
    builder.add_modifier_group(sum_group(group));
    let penalty = ValueExpr::Negate(Box::new(scalar_parameter(behavior, 0)?));
    let modifiers = [
        (
            22,
            StatKind::OutgoingHealing,
            FormulaStage::Healing,
            FormulaPurpose::Healing,
        ),
        (
            23,
            StatKind::ShieldStrength,
            FormulaStage::Shield,
            FormulaPurpose::Shield,
        ),
    ]
    .into_iter()
    .map(|(offset, stat, stage, purpose)| {
        let id = modifier_id(behavior, offset)?;
        builder.add_modifier(ModifierDefinition {
            id,
            stat,
            stage,
            purpose,
            value: penalty.clone(),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([ModifierFilter::FormulaSubject(FormulaSubject::Source)]),
        });
        Ok(id)
    })
    .collect::<Result<Vec<_>, CurrencyWarsBattleAssemblyError>>()?;
    let duration = u16::try_from(integer_parameter(behavior, 1)?).map_err(debug_error)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(integer_value(u32::from(duration))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or_else(|| error("Currency Wars critical conundrum effect is invalid"))?;
    builder.add_effect(
        EffectDefinition::new(effect, Vec::new(), modifiers).with_runtime_template(runtime),
    );
    let program = program_id_for(behavior, 1)?;
    programs.push(program_definition(
        program,
        Vec::new(),
        vec![effect],
        vec![apply_effect(
            event_target(),
            effect,
            RuleEffectChancePolicy::Guaranteed,
            None,
        )],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::DamageApplied,
        EventFilter {
            actor_selector: Some(enemies()),
            target_selector: Some(players()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::TargetWithinAction,
        program,
    )?);
    Ok(())
}

pub(super) fn compile_magma_bombardment(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let burn = effect_id(behavior, 20)?;
    let aggro = effect_id(behavior, 21)?;
    let group = modifier_group_id(behavior, 22)?;
    let modifier = modifier_id(behavior, 23)?;
    builder.add_modifier_group(sum_group(group));
    builder.add_modifier(ModifierDefinition {
        id: modifier,
        stat: StatKind::Aggro,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: scalar_parameter(behavior, 2)?,
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    });
    let permanent = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or_else(|| error("Currency Wars magma aggro effect is invalid"))?;
    builder.add_effect(
        EffectDefinition::new(aggro, Vec::new(), vec![modifier]).with_runtime_template(permanent),
    );
    let burn_damage = ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::EventTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rhs: Box::new(scalar_parameter(behavior, 0)?),
        rounding: Rounding::NearestTiesAway,
    };
    let duration = u16::try_from(integer_parameter(behavior, 1)?).map_err(debug_error)?;
    let burn_runtime = EffectRuntimeTemplate::new(
        EffectCategory::Dot,
        DispelCategory::DispellableDebuff,
        32,
        Some(integer_value(u32::from(duration))),
        DurationClock::TargetTurnStart,
        EffectTickPhase::TurnStart,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or_else(|| error("Currency Wars magma Burn effect is invalid"))?
    .with_comparison(Some(burn_damage), 0)
    .with_snapshot(EffectSnapshotPolicy::OnApplication)
    .with_dot(CombatElement::Fire, None)
    .ok_or_else(|| error("Currency Wars magma Burn definition is invalid"))?;
    builder.add_effect(
        EffectDefinition::new(burn, Vec::new(), Vec::new()).with_runtime_template(burn_runtime),
    );
    let enter = program_id_for(behavior, 1)?;
    let apply_burn = program_id_for(behavior, 2)?;
    programs.push(program_definition(
        enter,
        Vec::new(),
        vec![aggro],
        vec![apply_effect(
            first_player(),
            aggro,
            RuleEffectChancePolicy::Guaranteed,
            None,
        )],
    ));
    programs.push(program_definition(
        apply_burn,
        Vec::new(),
        vec![burn],
        vec![apply_effect(
            event_target(),
            burn,
            RuleEffectChancePolicy::Guaranteed,
            None,
        )],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::BattleStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Battle,
        enter,
    )?);
    triggers.push(trigger(
        behavior,
        11,
        RuleEventPoint::DamageApplied,
        EventFilter {
            actor_selector: Some(enemies()),
            target_selector: Some(players()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        OnceScope::TargetWithinAction,
        apply_burn,
    )?);
    Ok(())
}

pub(super) fn selectors() -> Result<Vec<SelectorDefinition>, CurrencyWarsBattleAssemblyError> {
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or_else(|| error("Currency Wars first player selector is invalid"))?;
    Ok(vec![
        SelectorDefinition::new(first_player()).with_rule_units(selector),
    ])
}

pub(super) fn selector_ids() -> Vec<SelectorId> {
    vec![first_player()]
}

fn first_player() -> SelectorId {
    SelectorId::new(FIRST_PLAYER_SELECTOR).expect("reserved first player selector is non-zero")
}

fn effect_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<EffectDefinitionId, CurrencyWarsBattleAssemblyError> {
    EffectDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars penalty effect ID is invalid"))
}

fn modifier_group_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<ModifierStackingGroupId, CurrencyWarsBattleAssemblyError> {
    ModifierStackingGroupId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars penalty modifier group ID is invalid"))
}

fn modifier_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<ModifierDefinitionId, CurrencyWarsBattleAssemblyError> {
    ModifierDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars penalty modifier ID is invalid"))
}

fn sum_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::Sum,
        comparator: None,
    }
}
