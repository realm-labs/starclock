//! Real production lowering for the bounded representative Counter guard.
use crate::catalog::{load, tests::PRODUCTION_BUNDLE};
use starclock_combat::{
    RuleId, StateSlotDefinitionId,
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
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::Slot(slot)),
            operator: Comparison::Greater,
            rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
        }
    );
    assert_eq!(rule.state_slots().len(), 1);
    let definition = &rule.state_slots()[0];
    assert_eq!(definition.id(), slot);
    assert_eq!(definition.initial(), &RuleValue::Integer(2));
    assert_eq!(definition.minimum(), Some(&RuleValue::Integer(0)));
    assert_eq!(definition.maximum(), Some(&RuleValue::Integer(2)));
}
