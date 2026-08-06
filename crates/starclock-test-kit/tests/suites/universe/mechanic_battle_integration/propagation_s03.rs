use super::*;
use starclock_combat::{
    EffectEventData,
    modifier::model::{FormulaPurpose, FormulaStage, FormulaSubject, ModifierFilter, StatKind},
    rule::model::{
        Comparison, ProgramStep, RuleActionKind, RuleEventPoint, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};

const OSSEUS: (&str, u32) = ("universe.blessing.612750", 2);
const SPINAL: (&str, u32) = ("universe.blessing.612751", 2);
const NEEDLE: (&str, u32) = ("universe.blessing.612752", 2);
const CONJUNCTIVA: (&str, u32) = ("universe.blessing.612753", 2);
const WING: (&str, u32) = ("universe.blessing.612754", 2);
const EYE: (&str, u32) = ("universe.blessing.612755", 2);

#[test]
fn goal07_p2_m09_s03_materializes_exact_generic_rules() {
    let catalog = catalog();
    let contributions = complete_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612750",
        "StageAbility_612751",
        "StageAbility_612752",
        "StageAbility_612753",
        "StageAbility_612754",
        "StageAbility_612755",
    ] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"))
            .runtime()
            .expect("generic runtime");
        assert!(runtime.native_handler().is_none());
    }

    assert_basic_stat(
        combat,
        &contributions,
        "StageAbility_612751",
        StatKind::CritRate,
        360_000,
    );
    assert_basic_stat(
        combat,
        &contributions,
        "StageAbility_612752",
        StatKind::CritDamage,
        600_000,
    );
    assert_timed_basic_stat(
        combat,
        &contributions,
        "StageAbility_612753",
        StatKind::Def,
        400_000,
        2,
    );
    assert_timed_basic_stat(
        combat,
        &contributions,
        "StageAbility_612754",
        StatKind::Spd,
        160_000,
        2,
    );

    let eye = combat
        .rule(binding(&contributions, "StageAbility_612755").rule())
        .unwrap();
    let runtime = eye.runtime().unwrap();
    assert_eq!(runtime.state_slots().len(), 1);
    assert_eq!(
        runtime.state_slots()[0].maximum(),
        Some(&RuleValue::Integer(5))
    );
    let trigger = runtime.triggers().first().expect("turn trigger");
    assert_eq!(trigger.event_point, RuleEventPoint::TurnEnded);
    assert_eq!(
        trigger.once_scope,
        starclock_combat::rule::model::OnceScope::Turn
    );
    assert!(matches!(
        trigger.condition,
        starclock_combat::rule::model::ConditionExpr::Compare {
            operator: Comparison::Less,
            ..
        }
    ));
    assert!(eye.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                        resource: starclock_combat::rule::model::RuleResourceKind::SkillPoints,
                        ..
                    })
                )
            })
        })
    }));

    let osseus = combat
        .rule(binding(&contributions, "StageAbility_612750").rule())
        .unwrap();
    assert!(referenced_modifiers(combat, osseus).iter().any(|modifier| {
        modifier.stage == FormulaStage::DamageBoost
            && expression_has_scalar(&modifier.value, 720_000)
    }));
}

#[test]
fn basic_attack_applies_replace_timed_defense_and_speed_effects() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.propagation",
        &[CONJUNCTIVA, WING],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let expected = ["StageAbility_612753", "StageAbility_612754"]
        .map(|key| rule_effect(combat, &contributions, key));
    let spec = durable_spec(&materialization, 0xd1, false);
    let (mut battle, started) = start(&materialization, spec, 0xd2);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    for effect in expected {
        assert!(
            resolution.events().iter().any(|event| {
                matches!(
                    event.kind(),
                    BattleEventKind::Effect(EffectEventData::Applied {
                        definition,
                        ..
                    }) if *definition == effect
                )
            }),
            "timed Basic effect {effect:?} was not applied: {:#?}",
            resolution.events()
        );
    }
}

#[test]
fn compound_eye_recovers_one_extra_team_skill_point_after_an_ally_turn() {
    let catalog = catalog();
    let baseline = contributions_many(&catalog, "universe.path.propagation", &[], None, false);
    let with_eye = contributions_many(&catalog, "universe.path.propagation", &[EYE], None, false);
    let baseline = materialize(&catalog, &baseline);
    let with_eye = materialize(&catalog, &with_eye);
    let (mut baseline_battle, baseline_started) =
        start(&baseline, durable_spec(&baseline, 0xd3, false), 0xd4);
    let (mut eye_battle, eye_started) =
        start(&with_eye, durable_spec(&with_eye, 0xd5, false), 0xd6);
    assert!(baseline_started.fault().is_none());
    assert!(eye_started.fault().is_none());
    let baseline_resolution = first_normal_action(&mut baseline_battle);
    let eye_resolution = first_normal_action(&mut eye_battle);
    assert!(baseline_resolution.fault().is_none());
    assert!(
        eye_resolution.fault().is_none(),
        "{:#?}",
        eye_resolution.events()
    );
    complete_action_events(&mut baseline_battle, &baseline_resolution);
    complete_action_events(&mut eye_battle, &eye_resolution);
    assert_eq!(
        eye_battle.view().team(TeamSide::Player).skill_points(),
        baseline_battle
            .view()
            .team(TeamSide::Player)
            .skill_points()
            .saturating_add(1)
            .min(5)
    );
}

fn complete_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.propagation",
        &[OSSEUS, SPINAL, NEEDLE, CONJUNCTIVA, WING, EYE],
        None,
        false,
    )
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    key: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(key))
        .unwrap_or_else(|| panic!("{key} selected"))
}

fn assert_basic_stat(
    combat: &starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
    stat: StatKind,
    value: i64,
) {
    let rule = combat.rule(binding(contributions, key).rule()).unwrap();
    assert!(referenced_modifiers(combat, rule).iter().any(|modifier| {
        modifier.stat == stat
            && modifier.purpose == FormulaPurpose::OrdinaryDamage
            && expression_has_scalar(&modifier.value, value)
            && modifier
                .filters
                .contains(&ModifierFilter::FormulaSubject(FormulaSubject::Source))
            && modifier.filters.iter().any(
                |filter| matches!(filter, ModifierFilter::AbilityTag(tag) if tag.as_ref() == "basic"),
            )
    }));
}

fn assert_timed_basic_stat(
    combat: &starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
    stat: StatKind,
    value: i64,
    turns: i64,
) {
    let rule = combat.rule(binding(contributions, key).rule()).unwrap();
    assert!(referenced_modifiers(combat, rule).iter().any(|modifier| {
        modifier.stat == stat
            && modifier.stage == FormulaStage::PercentOfBase
            && modifier.purpose == FormulaPurpose::Stat
            && expression_has_scalar(&modifier.value, value)
    }));
    let trigger = rule.runtime().unwrap().triggers().first().unwrap();
    assert_eq!(trigger.event_point, RuleEventPoint::ActionResolved);
    assert_eq!(trigger.filter.action_kind, Some(RuleActionKind::Basic));
    let effect = rule_effect(combat, contributions, key);
    let runtime = combat.effect(effect).unwrap().runtime_template().unwrap();
    assert_eq!(runtime.stack_limit(), 1);
    assert_eq!(
        runtime.duration_expression(),
        Some(&ValueExpr::Literal(RuleValue::Integer(turns)))
    );
}

fn rule_effect(
    combat: &starclock_combat::catalog::CombatCatalog,
    contributions: &UniverseBattleContributionSet,
    key: &str,
) -> starclock_combat::EffectDefinitionId {
    let rule = combat.rule(binding(contributions, key).rule()).unwrap();
    rule.programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.effects().iter().copied())
        .next()
        .expect("rule effect")
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_scalar(value, expected)
        }
        _ => false,
    }
}

fn referenced_modifiers<'a>(
    combat: &'a starclock_combat::catalog::CombatCatalog,
    rule: &starclock_combat::catalog::definition::RuleDefinition,
) -> Vec<&'a starclock_combat::modifier::model::ModifierDefinition> {
    rule.programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| {
            program
                .modifiers()
                .iter()
                .copied()
                .chain(program.effects().iter().flat_map(|effect| {
                    combat
                        .effect(*effect)
                        .into_iter()
                        .flat_map(|effect| effect.modifiers().iter().copied())
                }))
        })
        .filter_map(|modifier| combat.modifier(modifier))
        .collect()
}
