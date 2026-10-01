//! The Rule IR uses the same dedicated hit operation, with frozen expressions.
use super::{Scenario, attack, battle, damages, definition, elation, fixture, hit, start};
use starclock_combat::{
    Battle, BattleEvent, BattlePhase, Command, Ratio, Scalar,
    catalog::{
        CombatCatalog,
        action::{AbilityProgramBinding, AbilityProgramTiming, HitCritPolicy, HitTargetGroup},
        builder::CombatCatalogBuilder,
        definition::{AbilityDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    formula::model::CombatElement,
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleValue, ValueExpr,
        elation::ElationDamageExpressions,
    },
};
use std::sync::Arc;

fn literal(scaled: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(Scalar::from_scaled(scaled)))
}
fn inputs(base: i64) -> ElationDamageExpressions {
    ElationDamageExpressions {
        base_damage: literal(base),
        original_multiplier: literal(1_000_000),
        meter_multiplier: literal(1_000_000),
        merrymaking: literal(0),
        target_resistance: literal(0),
        penetration: literal(0),
        resistance_minimum: literal(-1_000_000),
        resistance_maximum: literal(900_000),
        unbroken_multiplier: literal(1_000_000),
    }
}
fn damage(inputs: ElationDamageExpressions, selector: u32, can_crit: bool) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ElationDamage {
        selector: definition(selector),
        inputs: Box::new(inputs),
        element: CombatElement::Fire,
        can_crit,
    })
}
fn catalog(
    scenario: &Scenario,
    first: Vec<ProgramStep>,
    later: Vec<ProgramStep>,
    looping: bool,
) -> Arc<CombatCatalog> {
    let base = fixture(scenario);
    let mut builder = CombatCatalogBuilder::from_catalog(&base, [0xe5; 32]);
    for (id, origin, side) in [
        (201, RuleSelectorOrigin::Owner, RuleSelectorSide::Same),
        (
            202,
            // A phase program uses its explicitly authored enemy pool. The
            // all-target ability has no controller-selected primary target.
            RuleSelectorOrigin::Encounter,
            RuleSelectorSide::Opposing,
        ),
        (
            203,
            RuleSelectorOrigin::CurrentSubject,
            RuleSelectorSide::Any,
        ),
    ] {
        builder.add_selector(
            SelectorDefinition::new(definition(id)).with_rule_units(
                RuleUnitSelector::new(
                    origin,
                    side,
                    RuleLifePredicate::Alive,
                    RulePresencePredicate::Present,
                    RuleSelectorReference::CurrentState,
                    RuleSelectorOrdering::StableId,
                    0,
                    2,
                    RuleEmptyPoolPolicy::NoOp,
                    RuleSelectorChoice::All,
                    None,
                    false,
                )
                .unwrap(),
            ),
        );
    }
    builder.add_program(
        ProgramDefinition::new(
            definition(301),
            if looping {
                vec![definition(302)]
            } else {
                vec![]
            },
            vec![definition(201), definition(202), definition(203)],
            vec![definition(12)],
            vec![],
        )
        .with_steps(first),
    );
    builder.add_program(
        ProgramDefinition::new(
            definition(302),
            vec![],
            vec![definition(202), definition(203)],
            vec![],
            vec![],
        )
        .with_steps(later),
    );
    let mut programs =
        vec![AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, definition(301)).unwrap()];
    if !looping {
        programs.push(
            AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, definition(302))
                .unwrap(),
        );
    }
    assert!(
        builder.replace_ability(
            AbilityDefinition::new(
                definition(1),
                definition(3),
                definition(3),
                vec![definition(10), definition(11), definition(12)]
            )
            .with_action(
                base.ability(definition(1))
                    .unwrap()
                    .action()
                    .unwrap()
                    .clone()
            )
            .with_programs(programs)
        )
    );
    builder.build().unwrap()
}
fn execute(catalog: Arc<CombatCatalog>, scenario: &Scenario) -> (Battle, Vec<BattleEvent>) {
    let mut running = battle(Arc::clone(&catalog), scenario);
    let mut fresh = battle(catalog, scenario);
    start(&mut running, &mut fresh);
    let events = attack(&mut running, &mut fresh);
    (running, events)
}

#[test]
fn rule_elation_freezes_meter_expressions_but_queries_live_property_at_execution() {
    let scenario = Scenario::new(vec![hit(vec![elation(100_000_000)], HitCritPolicy::Never)]);
    let mut operands = inputs(100_000_000);
    operands.meter_multiplier = ValueExpr::Add(
        Box::new(literal(1_000_000)),
        Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::Actor,
            stat: StatKind::Elation,
            purpose: FormulaPurpose::ElationDamage,
        }),
    );
    let apply = ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: definition(201),
        effect: definition(12),
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    });
    let catalog = catalog(
        &scenario,
        vec![apply, damage(operands.clone(), 202, false)],
        vec![damage(operands, 202, false)],
        false,
    );
    let (battle, events) = execute(catalog, &scenario);
    assert_eq!(
        damages(&events)
            .iter()
            .map(|d| d.calculated.get())
            .collect::<Vec<_>>(),
        [150, 150, 225]
    );
    assert_eq!(battle.view().rng_draw_count(), 0);
}

#[test]
fn rule_elation_and_native_operations_retain_hit_share_and_fractional_floor() {
    let scenario = Scenario::new(vec![
        hit(vec![elation(100_900_000)], HitCritPolicy::Never).with_profile(
            HitTargetGroup::Selected,
            Ratio::from_scaled(500_000),
            Ratio::ONE,
            HitCritPolicy::Never,
        ),
    ]);
    let mut operands = inputs(100_900_000);
    operands.original_multiplier = literal(800_000);
    operands.meter_multiplier = literal(2_000_000);
    operands.merrymaking = literal(200_000);
    let catalog = catalog(&scenario, vec![damage(operands, 202, false)], vec![], false);
    let (_, events) = execute(catalog, &scenario);
    let mut values = damages(&events)
        .iter()
        .map(|d| (d.raw.scaled(), d.calculated.get()))
        .collect::<Vec<_>>();
    values.sort_unstable();
    assert_eq!(values, [(50_450_000, 50), (96_864_000, 96)]);
}

#[test]
fn rule_elation_shares_the_same_crit_draw_group_as_native_hit_damage() {
    for (policy, draws) in [
        (HitCritPolicy::Never, 0),
        (HitCritPolicy::PerTarget, 2),
        (HitCritPolicy::Shared, 1),
    ] {
        let mut scenario = Scenario::new(vec![hit(vec![elation(10_000_000)], policy)]);
        scenario.enemies = 2;
        scenario.crit_rate = 500_000;
        let catalog = catalog(
            &scenario,
            vec![damage(inputs(10_000_000), 202, true)],
            vec![],
            false,
        );
        let (battle, events) = execute(catalog, &scenario);
        let d = damages(&events);
        assert_eq!(d.len(), 4);
        assert_eq!(
            (d[0].target, d[0].calculated),
            (d[2].target, d[2].calculated)
        );
        assert_eq!(
            (d[1].target, d[1].calculated),
            (d[3].target, d[3].calculated)
        );
        assert_eq!(battle.view().rng_draw_count(), draws);
    }
}

#[test]
fn rule_elation_foreach_retains_each_current_subject_and_stat_read() {
    let mut scenario = Scenario::new(vec![hit(vec![], HitCritPolicy::Never)]);
    scenario.enemies = 2;
    scenario.target_defense = 100_000_000;
    let mut operands = inputs(0);
    operands.base_damage = ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Def,
        purpose: FormulaPurpose::Stat,
    };
    let loop_step = ProgramStep::ForEach {
        selector: definition(202),
        body: definition(302),
        maximum: 2,
    };
    let catalog = catalog(
        &scenario,
        vec![loop_step],
        vec![damage(operands, 203, false)],
        true,
    );
    let (_, events) = execute(catalog, &scenario);
    let d = damages(&events);
    assert_eq!(d.len(), 2);
    assert_ne!(d[0].target, d[1].target);
    assert!(
        d.iter()
            .all(|d| (d.raw.scaled(), d.calculated.get()) == (90_909_100, 90))
    );
}

#[test]
fn rule_elation_invalid_later_operand_faults_before_any_emission_commits() {
    let scenario = Scenario::new(vec![hit(vec![], HitCritPolicy::Never)]);
    let mut invalid = inputs(100_000_000);
    invalid.meter_multiplier = literal(-1);
    let catalog = catalog(
        &scenario,
        vec![
            damage(inputs(100_000_000), 202, false),
            damage(invalid, 202, false),
        ],
        vec![],
        false,
    );
    let mut running = battle(Arc::clone(&catalog), &scenario);
    let mut fresh = battle(catalog, &scenario);
    start(&mut running, &mut fresh);
    let command = running
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|c| matches!(c, Command::UseAbility { .. }))
        .unwrap()
        .clone();
    let a = running.apply(command.clone()).unwrap();
    let b = fresh.apply(command).unwrap();
    assert!(a.fault().is_some());
    assert_eq!(a.fault(), b.fault());
    assert_eq!(a.events(), b.events());
    assert!(damages(a.events()).is_empty());
    assert_eq!(running.view().phase(), BattlePhase::Faulted);
    assert_eq!(running.state_hash(), fresh.state_hash());
    assert_eq!(running.view().rng_draw_count(), 0);
    assert!(
        running
            .view()
            .units_by_id()
            .all(|u| u.current_hp().get() == 1000)
    );
}
