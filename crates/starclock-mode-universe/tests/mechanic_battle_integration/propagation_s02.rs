use super::*;
use starclock_combat::{
    BattleEventKind,
    catalog::selector::{RuleSelectorChoice, RuleSelectorPredicate},
    formula::model::DamageClass,
    modifier::model::{FormulaPurpose, FormulaStage},
    rule::model::{ProgramStep, RuleDamageClass, RuleEventPoint, RuleOperationTemplate},
};

const METABOLIC: (&str, u32) = ("universe.blessing.612742", 2);
const EXCITATORY: (&str, u32) = ("universe.blessing.612743", 2);
const EXPOSED: (&str, u32) = ("universe.blessing.612744", 2);
const MEMBRANE: (&str, u32) = ("universe.blessing.612745", 2);
const CATALYST: (&str, u32) = ("universe.blessing.612746", 2);
const OSSEUS: (&str, u32) = ("universe.blessing.612750", 1);

#[test]
fn goal07_p2_m09_s02_materializes_every_rule_without_native_handlers() {
    let catalog = catalog();
    let contributions = complete_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612742",
        "StageAbility_612743",
        "StageAbility_612744",
        "StageAbility_612745",
        "StageAbility_612746",
        "StageAbility_612750",
    ] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"))
            .runtime()
            .expect("generic runtime");
        assert!(runtime.native_handler().is_none());
    }

    let metabolic = combat
        .rule(binding(&contributions, "StageAbility_612742").rule())
        .unwrap();
    assert!(
        metabolic
            .runtime()
            .unwrap()
            .triggers()
            .iter()
            .any(|trigger| { trigger.event_point == RuleEventPoint::InformationalRule })
    );
    assert!(
        referenced_modifiers(combat, metabolic)
            .iter()
            .any(|modifier| {
                modifier.stage == FormulaStage::Mitigation
                    && expression_has_scalar(&modifier.value, 8_000)
            })
    );

    let membrane = combat
        .rule(binding(&contributions, "StageAbility_612745").rule())
        .unwrap();
    assert!(
        referenced_modifiers(combat, membrane)
            .iter()
            .any(|modifier| {
                modifier.stage == FormulaStage::Mitigation
                    && expression_has_scalar(&modifier.value, 80_000)
            })
    );

    let catalyst = combat
        .rule(binding(&contributions, "StageAbility_612746").rule())
        .unwrap();
    let catalyst_runtime = catalyst.runtime().unwrap();
    assert!(
        catalyst_runtime
            .triggers()
            .iter()
            .any(|trigger| { trigger.event_point == RuleEventPoint::ActionStarted })
    );
    assert!(
        catalyst_runtime
            .triggers()
            .iter()
            .any(|trigger| { trigger.event_point == RuleEventPoint::HitStarted })
    );
    assert!(
        catalyst_runtime
            .triggers()
            .iter()
            .any(|trigger| { trigger.event_point == RuleEventPoint::ActionResolved })
    );
    assert!(
        referenced_modifiers(combat, catalyst)
            .iter()
            .any(|modifier| {
                modifier.stage == FormulaStage::DamageBoost
                    && expression_has_scalar(&modifier.value, 300_000)
            })
    );

    let osseus = combat
        .rule(binding(&contributions, "StageAbility_612750").rule())
        .unwrap();
    assert!(referenced_modifiers(combat, osseus).iter().any(|modifier| {
        modifier.purpose == FormulaPurpose::OrdinaryDamage
            && expression_has_scalar(&modifier.value, 540_000)
            && modifier.filters.iter().any(|filter| {
                matches!(
                    filter,
                    starclock_combat::modifier::model::ModifierFilter::AbilityTag(tag)
                        if tag.as_ref() == "basic"
                )
            })
    }));
}

#[test]
fn exposed_brain_matter_uses_adjacent_selection_and_unboosted_event_element_damage() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.propagation",
        &[("universe.blessing.612744", 1)],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_612744").rule())
        .unwrap();
    let trigger = rule
        .runtime()
        .unwrap()
        .triggers()
        .first()
        .expect("Exposed Brain Matter trigger");
    assert_eq!(trigger.filter.damage_class, Some(RuleDamageClass::Ordinary));
    let selector = rule
        .selectors()
        .iter()
        .filter_map(|id| combat.selector(*id).and_then(|value| value.rule_units()))
        .find(|selector| {
            selector.choice() == RuleSelectorChoice::RngUniform
                && selector
                    .predicates()
                    .contains(&RuleSelectorPredicate::AdjacentToPrimary)
        })
        .expect("random adjacent selector");
    assert_eq!(selector.maximum(), 1);
    assert!(rule.programs().iter().any(|id| {
        combat.program(*id).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(
                        RuleOperationTemplate::UnboostedDamageFromEventElement {
                            class: DamageClass::Additional,
                            can_defeat: true,
                            ..
                        }
                    )
                )
            })
        })
    }));
}

#[test]
fn basic_attack_executes_one_random_adjacent_additional_damage_event() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.propagation",
        &[("universe.blessing.612744", 1)],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let spec = durable_spec_with_two_enemies(&materialization, 0xc1);
    let (mut battle, started) = start(&materialization, spec, 0xc2);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let draws_before = battle.view().rng_draw_count();
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    let adjacent = resolution
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Damage(value)
                    if value.class == DamageClass::Additional
            )
        })
        .count();
    assert_eq!(adjacent, 1, "{:#?}", resolution.events());
    assert!(battle.view().rng_draw_count() > draws_before);
}

#[test]
fn metabolic_cavity_tracks_global_spores_as_each_ally_modifier_stack() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.propagation",
        &[METABOLIC],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_612742").rule())
        .unwrap();
    let mitigation_effect = rule
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.effects())
        .copied()
        .find(|effect| {
            combat.effect(*effect).is_some_and(|effect| {
                effect.modifiers().iter().any(|modifier| {
                    combat
                        .modifier(*modifier)
                        .is_some_and(|modifier| modifier.stage == FormulaStage::Mitigation)
                })
            })
        })
        .expect("global-Spore mitigation effect");
    let spec = durable_spec_with_two_enemies(&materialization, 0xc3);
    let (mut battle, started) = start(&materialization, spec, 0xc4);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    let stacks = battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition() == mitigation_effect)
        .map(|effect| effect.stacks())
        .collect::<Vec<_>>();
    assert_eq!(stacks, vec![2, 2, 2, 2], "{:#?}", resolution.events());
}

fn complete_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.propagation",
        &[METABOLIC, EXCITATORY, EXPOSED, MEMBRANE, CATALYST, OSSEUS],
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

fn expression_has_scalar(value: &starclock_combat::rule::model::ValueExpr, expected: i64) -> bool {
    use starclock_combat::rule::model::{RuleValue, ValueExpr};
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
