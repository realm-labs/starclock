//! Resolved levels are typed unit facts, never an inferred owner/stat value.
use super::{definition, input, runtime};
use starclock_combat::{
    EffectDefinitionId, LifeState, PresenceState, Rounding, Scalar, UnitId, UnitLevel,
    formula::model::CombatElement,
    modifier::model::StatQuerySubject,
    rule::{
        evaluate::{BattleQueryReader, RuleEvaluationErrorKind, evaluate_value},
        model::{RuleValue, RuleValueKind, ValueExpr},
    },
};

struct Levels;
impl BattleQueryReader for Levels {
    fn unit_level(&self, subject: UnitId) -> Option<UnitLevel> {
        match subject.get() {
            1 => UnitLevel::new(1),
            2 => UnitLevel::new(40),
            3 => UnitLevel::new(81),
            4 => UnitLevel::new(95),
            5 => UnitLevel::new(13),
            6 => UnitLevel::new(77),
            _ => None,
        }
    }
    fn life_presence(&self, _: UnitId) -> Option<(LifeState, PresenceState)> {
        Some((LifeState::Defeated, PresenceState::Departed))
    }
    fn has_effect(&self, _: UnitId, _: EffectDefinitionId) -> bool {
        false
    }
    fn is_frozen(&self, _: UnitId) -> bool {
        false
    }
    fn has_weakness(&self, _: UnitId, _: CombatElement) -> bool {
        false
    }
    fn is_broken(&self, _: UnitId) -> bool {
        false
    }
    fn current_shield(&self, _: UnitId) -> Option<Scalar> {
        None
    }
    fn effect_stacks(&self, _: UnitId, _: EffectDefinitionId) -> Option<i64> {
        None
    }
}

#[test]
fn unit_level_reads_each_exact_cause_role_and_iteration_subject_as_integer() {
    let mut context = input(&[], definition(1), &[]);
    context.battle_query_reader = Some(&Levels);
    for (subject, expected) in [
        (StatQuerySubject::Owner, 1),
        (StatQuerySubject::Actor, 40),
        (StatQuerySubject::Applier, 81),
        (StatQuerySubject::EventTarget, 95),
        (StatQuerySubject::CurrentTarget, 13),
    ] {
        assert_eq!(
            evaluate_value(
                &ValueExpr::QueryUnitLevel(subject),
                context,
                Some(runtime(5))
            )
            .unwrap(),
            RuleValue::Integer(expected)
        );
    }
    // Rule ownership overrides cause ownership, without changing actor/applier.
    context.rule_owner = Some(runtime(6));
    assert_eq!(
        evaluate_value(
            &ValueExpr::QueryUnitLevel(StatQuerySubject::Owner),
            context,
            None
        )
        .unwrap(),
        RuleValue::Integer(77)
    );
}

#[test]
fn unit_level_missing_reader_unit_and_subject_fail_without_owner_fallback() {
    let mut context = input(&[], definition(1), &[]);
    let expression = ValueExpr::QueryUnitLevel(StatQuerySubject::Actor);
    let error = evaluate_value(&expression, context, None).unwrap_err();
    assert_eq!(error.kind(), RuleEvaluationErrorKind::MissingValue);
    assert_eq!(error.context(), 0x222);
    context.battle_query_reader = Some(&Levels);
    context.cause.actor = Some(runtime(99));
    let error = evaluate_value(&expression, context, None).unwrap_err();
    assert_eq!(error.kind(), RuleEvaluationErrorKind::MissingValue);
    assert_eq!(error.context(), 0x222);
    for subject in [
        StatQuerySubject::Actor,
        StatQuerySubject::Applier,
        StatQuerySubject::EventTarget,
    ] {
        context.cause.actor = None;
        context.cause.applier = None;
        context.cause.target = None;
        let error = evaluate_value(&ValueExpr::QueryUnitLevel(subject), context, None).unwrap_err();
        assert_eq!(error.kind(), RuleEvaluationErrorKind::MissingValue);
        assert_eq!(error.context(), 0x202);
    }
    assert_eq!(
        evaluate_value(
            &ValueExpr::QueryUnitLevel(StatQuerySubject::CurrentTarget),
            context,
            None
        )
        .unwrap_err()
        .kind(),
        RuleEvaluationErrorKind::MissingValue
    );
}

#[test]
fn unit_level_requires_explicit_scalar_conversion_and_checked_arithmetic() {
    let mut context = input(&[], definition(1), &[]);
    context.battle_query_reader = Some(&Levels);
    let level = ValueExpr::QueryUnitLevel(StatQuerySubject::EventTarget);
    let converted = ValueExpr::Convert {
        value: Box::new(level.clone()),
        target: RuleValueKind::Scalar,
        rounding: Rounding::Floor,
    };
    assert_eq!(
        evaluate_value(&converted, context, None).unwrap(),
        RuleValue::Scalar(Scalar::checked_from_integer(95).unwrap())
    );
    let square = ValueExpr::Multiply {
        lhs: Box::new(level.clone()),
        rhs: Box::new(level.clone()),
        rounding: Rounding::Floor,
    };
    assert_eq!(
        evaluate_value(&square, context, None).unwrap(),
        RuleValue::Integer(9_025)
    );
    assert_eq!(
        evaluate_value(
            &ValueExpr::Add(
                Box::new(level.clone()),
                Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ONE)))
            ),
            context,
            None
        )
        .unwrap_err()
        .kind(),
        RuleEvaluationErrorKind::TypeMismatch
    );
    assert_eq!(
        evaluate_value(
            &ValueExpr::Multiply {
                lhs: Box::new(level),
                rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::MAX))),
                rounding: Rounding::Floor,
            },
            context,
            None
        )
        .unwrap_err()
        .kind(),
        RuleEvaluationErrorKind::Numeric
    );
}
