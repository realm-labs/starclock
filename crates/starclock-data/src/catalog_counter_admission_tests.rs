//! Real production lowering for the bounded representative Counter guard.
use crate::catalog::{load, tests::PRODUCTION_BUNDLE};
use starclock_combat::{
    RuleId, SelectorId, StateSlotDefinitionId,
    catalog::selector::{RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate},
    rule::model::{Comparison, ConditionExpr, RuleValue, ValueExpr},
};

#[test]
fn production_clara_counter_admission_reads_its_bounded_charge_slot() {
    let catalog = load(PRODUCTION_BUNDLE).unwrap();
    let rule = catalog.battle_rule(RuleId::new(24205).unwrap()).unwrap();
    let slot = StateSlotDefinitionId::new(24206).unwrap();
    assert_eq!(rule.triggers().len(), 1);
    assert_eq!(
        rule.triggers()[0].condition,
        ConditionExpr::All(
            vec![
                ConditionExpr::Compare {
                    lhs: Box::new(ValueExpr::Slot(slot)),
                    operator: Comparison::Greater,
                    rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
                },
                ConditionExpr::All(
                    vec![
                        ConditionExpr::SelectorCardinality {
                            selector: SelectorId::new(24251).unwrap(),
                            operator: Comparison::GreaterOrEqual,
                            count: 1,
                        },
                        ConditionExpr::SelectorCardinality {
                            selector: SelectorId::new(24251).unwrap(),
                            operator: Comparison::LessOrEqual,
                            count: 1,
                        },
                    ]
                    .into_boxed_slice()
                ),
            ]
            .into_boxed_slice()
        )
    );
    let owner = catalog
        .combat_catalog
        .selector(SelectorId::new(24251).unwrap())
        .unwrap()
        .rule_units()
        .unwrap();
    assert_eq!(owner.life(), RuleLifePredicate::Alive);
    assert_eq!(owner.presence(), RulePresencePredicate::Present);
    assert_eq!((owner.minimum(), owner.maximum()), (0, 1));
    assert_eq!(owner.empty_pool(), RuleEmptyPoolPolicy::NoOp);
    assert_eq!(rule.state_slots().len(), 1);
    let definition = &rule.state_slots()[0];
    assert_eq!(definition.id(), slot);
    assert_eq!(definition.initial(), &RuleValue::Integer(2));
    assert_eq!(definition.minimum(), Some(&RuleValue::Integer(0)));
    assert_eq!(definition.maximum(), Some(&RuleValue::Integer(2)));
}
