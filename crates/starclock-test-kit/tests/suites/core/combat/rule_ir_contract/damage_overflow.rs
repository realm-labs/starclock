//! Missing settlement facts must not turn into a guessed zero.
use super::{definition, input};
use starclock_combat::{
    Scalar,
    rule::{
        evaluate::{RuleEvaluationErrorKind, evaluate_value},
        model::{EventValueProperty, RuleEventFacts, RuleValue, ValueExpr},
    },
};

#[test]
fn damage_overflow_reads_exact_zero_positive_and_missing_event_facts() {
    let expression = ValueExpr::ReadEventProperty(EventValueProperty::DamageOverflow);
    for amount in [
        None,
        Some(Scalar::ZERO),
        Some(Scalar::from_scaled(123_000_000)),
    ] {
        let facts = RuleEventFacts {
            damage_overflow: amount,
            ..RuleEventFacts::default()
        };
        let mut context = input(&[], definition(1), &[]);
        context.event_facts = &facts;
        let result = evaluate_value(&expression, context, None);
        match amount {
            Some(amount) => assert_eq!(result.unwrap(), RuleValue::Scalar(amount)),
            None => assert_eq!(
                result.unwrap_err().kind(),
                RuleEvaluationErrorKind::MissingValue
            ),
        }
    }
}
