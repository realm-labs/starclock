//! One-shot enemy HP-floor Affix lowering.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, Rounding,
    catalog::{
        builder::CombatCatalogBuilder, definition::EffectDefinition, definition::ProgramDefinition,
    },
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        Comparison, ConditionExpr, EventFilter, EventValueProperty, OnceScope,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, TriggerDef, ValueExpr,
    },
};

use crate::{
    CurrencyWarsEnemyAffixBehavior,
    battle_assembly::{CurrencyWarsBattleAssemblyError, error},
};

use super::{
    apply_effect, definition_raw, enemies, event_target, marker_effect, operation,
    program_definition, program_id_for, scalar_parameter, scalar_value, trigger,
};

pub(super) fn compile(
    builder: &mut CombatCatalogBuilder,
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let armed = effect_id(behavior, 20)?;
    let current_action_floor = effect_id(behavior, 21)?;
    let spent = effect_id(behavior, 22)?;
    builder.add_effect(hp_floor_effect(armed, scalar_parameter(behavior, 0)?)?);
    builder.add_effect(hp_floor_effect(
        current_action_floor,
        scalar_parameter(behavior, 1)?,
    )?);
    builder.add_effect(marker_effect(spent, None)?);

    let arm = program_id_for(behavior, 1)?;
    let rescue = program_id_for(behavior, 2)?;
    let cleanup = program_id_for(behavior, 3)?;
    programs.push(program_definition(
        arm,
        Vec::new(),
        vec![armed],
        vec![apply_effect(
            enemies(),
            armed,
            RuleEffectChancePolicy::Guaranteed,
            None,
        )],
    ));
    let maximum_hp = ValueExpr::QueryStat {
        subject: StatQuerySubject::EventTarget,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
    };
    let trigger_floor = ValueExpr::Multiply {
        lhs: Box::new(maximum_hp.clone()),
        rhs: Box::new(scalar_parameter(behavior, 0)?),
        rounding: Rounding::Ceil,
    };
    let restored_hp = ValueExpr::Multiply {
        lhs: Box::new(maximum_hp),
        rhs: Box::new(scalar_parameter(behavior, 1)?),
        rounding: Rounding::Ceil,
    };
    programs.push(program_definition(
        rescue,
        Vec::new(),
        vec![armed, current_action_floor, spent],
        vec![
            operation(RuleOperationTemplate::RemoveEffect {
                selector: event_target(),
                effect: armed,
            }),
            apply_effect(
                event_target(),
                current_action_floor,
                RuleEffectChancePolicy::Guaranteed,
                None,
            ),
            apply_effect(
                event_target(),
                spent,
                RuleEffectChancePolicy::Guaranteed,
                None,
            ),
            operation(RuleOperationTemplate::Heal {
                selector: event_target(),
                amount: ValueExpr::Subtract(
                    Box::new(restored_hp),
                    Box::new(ValueExpr::ReadEventProperty(EventValueProperty::HpAfter)),
                ),
                apply_formula_modifiers: false,
            }),
        ],
    ));
    programs.push(program_definition(
        cleanup,
        Vec::new(),
        vec![current_action_floor],
        vec![operation(RuleOperationTemplate::RemoveEffect {
            selector: enemies(),
            effect: current_action_floor,
        })],
    ));
    triggers.push(trigger(
        behavior,
        10,
        RuleEventPoint::WaveStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Wave,
        arm,
    )?);
    triggers.push(trigger(
        behavior,
        11,
        RuleEventPoint::DamageApplied,
        EventFilter {
            target_selector: Some(enemies()),
            has_action: Some(true),
            ..EventFilter::default()
        },
        ConditionExpr::All(
            vec![
                ConditionExpr::EffectExists {
                    selector: event_target(),
                    effect: armed,
                },
                ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                    selector: event_target(),
                    effect: spent,
                })),
                ConditionExpr::Compare {
                    lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::HpAfter)),
                    operator: Comparison::LessOrEqual,
                    rhs: Box::new(ValueExpr::Add(
                        Box::new(trigger_floor),
                        Box::new(scalar_value(1)),
                    )),
                },
            ]
            .into_boxed_slice(),
        ),
        OnceScope::Event,
        rescue,
    )?);
    triggers.push(trigger(
        behavior,
        12,
        RuleEventPoint::ActionResolved,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        OnceScope::Action,
        cleanup,
    )?);
    Ok(())
}

fn hp_floor_effect(
    id: EffectDefinitionId,
    floor: ValueExpr,
) -> Result<EffectDefinition, CurrencyWarsBattleAssemblyError> {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or_else(|| error("Currency Wars survival floor effect is invalid"))?
    .with_hp_floor(floor);
    Ok(EffectDefinition::new(id, Vec::new(), Vec::new()).with_runtime_template(runtime))
}

fn effect_id(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    offset: u32,
) -> Result<EffectDefinitionId, CurrencyWarsBattleAssemblyError> {
    EffectDefinitionId::new(definition_raw(behavior, offset)?)
        .ok_or_else(|| error("Currency Wars survival effect ID is invalid"))
}
