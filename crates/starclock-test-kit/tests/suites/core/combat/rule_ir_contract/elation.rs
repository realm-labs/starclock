use super::{definition, input, source, trigger};
use starclock_combat::{
    ProgramId, Ratio, Scalar,
    catalog::{
        CombatCatalog,
        action::elation::ElationDamageDefinition,
        builder::{CatalogBuildError, CatalogBuildErrorKind, CombatCatalogBuilder},
        definition::{ProgramDefinition, RuleDefinition, SelectorDefinition},
    },
    formula::model::{CombatElement, ResistanceInput},
    rule::{
        evaluate::{
            EvaluationBudget, RuleEvaluationErrorKind, evaluate_program,
            evaluate_replacement_program,
        },
        model::{
            BattleRuleDefinition, ProgramStep, RuleEmission, RuleOperationTemplate, RuleValue,
            ValueExpr, elation::ElationDamageExpressions,
        },
    },
};
use std::sync::Arc;

fn scalar(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(value)))
}
fn operands() -> ElationDamageExpressions {
    ElationDamageExpressions {
        base_damage: scalar(100_900_000),
        original_multiplier: scalar(800_000),
        meter_multiplier: scalar(2_000_000),
        merrymaking: scalar(-500_000),
        target_resistance: scalar(-200_000),
        penetration: scalar(100_000),
        resistance_minimum: scalar(-1_000_000),
        resistance_maximum: scalar(900_000),
        unbroken_multiplier: scalar(900_000),
    }
}
fn catalog(
    inputs: ElationDamageExpressions,
    validate: bool,
) -> Result<Arc<CombatCatalog>, CatalogBuildError> {
    let mut builder = CombatCatalogBuilder::new([0xe6; 32]);
    let program: ProgramId = definition(1);
    builder.add_selector(SelectorDefinition::new(definition(1)));
    builder.add_program(
        ProgramDefinition::new(program, vec![], vec![definition(1)], vec![], vec![]).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::ElationDamage {
                    selector: definition(1),
                    inputs: Box::new(inputs),
                    element: CombatElement::Fire,
                    can_crit: true,
                },
            )],
        ),
    );
    if validate {
        builder.add_rule(
            RuleDefinition::new(definition(1), vec![program], vec![definition(1)]).with_runtime(
                BattleRuleDefinition::new(source(1), vec![], vec![trigger(1, program)], None),
            ),
        );
    }
    builder.build()
}
#[test]
fn rule_elation_proposal_is_typed_exact_and_rejected_by_replacement_execution() {
    let catalog = catalog(operands(), true).unwrap();
    let context = input(&[], definition(1), &[]);
    let emissions = evaluate_program(
        &*catalog,
        definition(1),
        context,
        EvaluationBudget::STANDARD,
    )
    .unwrap();
    let expected = ElationDamageDefinition::new(
        Scalar::from_scaled(100_900_000),
        Ratio::from_scaled(800_000),
        Ratio::from_scaled(2_000_000),
        Ratio::from_scaled(-500_000),
        ResistanceInput {
            target_resistance: Ratio::from_scaled(-200_000),
            penetration: Ratio::from_scaled(100_000),
            minimum: Ratio::from_scaled(-1_000_000),
            maximum: Ratio::from_scaled(900_000),
        },
        Ratio::from_scaled(900_000),
        CombatElement::Fire,
    )
    .unwrap();
    assert_eq!(
        emissions,
        [RuleEmission::ElationDamage {
            selector: definition(1),
            definition: expected,
            can_crit: true,
            current_target: None
        }]
    );
    assert_eq!(
        evaluate_replacement_program(
            &*catalog,
            definition(1),
            context,
            EvaluationBudget::STANDARD
        )
        .unwrap_err()
        .kind(),
        RuleEvaluationErrorKind::TypeMismatch
    );
    assert_eq!(
        evaluate_program(
            &*catalog,
            definition(1),
            context,
            EvaluationBudget {
                maximum_emissions: 0,
                ..EvaluationBudget::STANDARD
            }
        )
        .unwrap_err()
        .kind(),
        RuleEvaluationErrorKind::BudgetExceeded
    );
}
#[test]
fn rule_elation_catalog_and_evaluator_reject_every_non_scalar_operand() {
    let mutations: [fn(&mut ElationDamageExpressions); 9] = [
        |x| x.base_damage = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.original_multiplier = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.meter_multiplier = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.merrymaking = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.target_resistance = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.penetration = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.resistance_minimum = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.resistance_maximum = ValueExpr::Literal(RuleValue::Integer(1)),
        |x| x.unbroken_multiplier = ValueExpr::Literal(RuleValue::Integer(1)),
    ];
    for mutate in mutations {
        let mut inputs = operands();
        mutate(&mut inputs);
        assert_eq!(
            catalog(inputs.clone(), true).unwrap_err().kind(),
            CatalogBuildErrorKind::InvalidDefinition
        );
        let catalog = catalog(inputs, false).unwrap();
        let error = evaluate_program(
            &*catalog,
            definition(1),
            input(&[], definition(1), &[]),
            EvaluationBudget::STANDARD,
        )
        .unwrap_err();
        assert_eq!(error.kind(), RuleEvaluationErrorKind::TypeMismatch);
        assert_eq!(error.context(), 110);
    }
}
#[test]
fn rule_elation_expression_overflow_and_definition_domain_errors_fail_closed() {
    let mutations: [fn(&mut ElationDamageExpressions); 7] = [
        |x| x.base_damage = scalar(-1),
        |x| x.original_multiplier = scalar(-1),
        |x| x.meter_multiplier = scalar(-1),
        |x| x.merrymaking = scalar(-1_000_001),
        |x| x.resistance_minimum = scalar(900_001),
        |x| x.resistance_maximum = scalar(1_000_001),
        |x| x.unbroken_multiplier = scalar(-1),
    ];
    for mutate in mutations {
        let mut inputs = operands();
        mutate(&mut inputs);
        let catalog = catalog(inputs, true).unwrap();
        let error = evaluate_program(
            &*catalog,
            definition(1),
            input(&[], definition(1), &[]),
            EvaluationBudget::STANDARD,
        )
        .unwrap_err();
        assert_eq!(error.kind(), RuleEvaluationErrorKind::Numeric);
        assert_eq!(error.context(), 111);
    }
    let mut inputs = operands();
    inputs.base_damage = ValueExpr::Add(Box::new(scalar(i64::MAX)), Box::new(scalar(1)));
    let catalog = catalog(inputs, true).unwrap();
    assert_eq!(
        evaluate_program(
            &*catalog,
            definition(1),
            input(&[], definition(1), &[]),
            EvaluationBudget::STANDARD
        )
        .unwrap_err()
        .kind(),
        RuleEvaluationErrorKind::Numeric
    );
}
