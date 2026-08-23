//! Permanent Trauma's battle-local maximum-HP reduction.

use starclock_combat::{
    Rounding, Scalar,
    catalog::definition::ProgramDefinition,
    rule::model::{
        ConditionExpr, EventFilter, EventValueProperty, OnceScope, RuleEventPoint,
        RuleOperationTemplate, RuleValue, TriggerDef, ValueExpr,
    },
};

use crate::{CurrencyWarsEnemyAffixBehavior, battle_assembly::CurrencyWarsBattleAssemblyError};

use super::{
    event_target, operation, players, program_definition, program_id_for, scalar_parameter, trigger,
};

pub(super) fn compile(
    behavior: &CurrencyWarsEnemyAffixBehavior,
    programs: &mut Vec<ProgramDefinition>,
    triggers: &mut Vec<TriggerDef>,
) -> Result<(), CurrencyWarsBattleAssemblyError> {
    let program = program_id_for(behavior, 1)?;
    let lost_hp = ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
        EventValueProperty::HpChangeAmount,
    )));
    let reduction = ValueExpr::Multiply {
        lhs: Box::new(lost_hp),
        rhs: Box::new(scalar_parameter(behavior, 0)?),
        rounding: Rounding::Floor,
    };
    let minimum_ratio = ValueExpr::Subtract(
        Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ONE))),
        Box::new(scalar_parameter(behavior, 1)?),
    );
    programs.push(program_definition(
        program,
        Vec::new(),
        Vec::new(),
        vec![operation(RuleOperationTemplate::ReduceMaximumHp {
            selector: event_target(),
            amount: reduction,
            minimum_ratio,
        })],
    ));
    for (offset, point) in [
        (10, RuleEventPoint::DamageApplied),
        (11, RuleEventPoint::HpChanged),
    ] {
        triggers.push(trigger(
            behavior,
            offset,
            point,
            EventFilter {
                target_selector: Some(players()),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            OnceScope::Event,
            program,
        )?);
    }
    Ok(())
}
