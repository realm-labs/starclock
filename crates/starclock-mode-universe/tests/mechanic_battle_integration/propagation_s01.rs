use super::*;
use starclock_combat::{
    EffectDefinitionId,
    catalog::action::AbilityTag,
    formula::model::{CombatElement, DamageClass},
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{
        EventValueProperty, ProgramStep, RuleEventPoint, RuleOperationTemplate, ValueExpr,
    },
};

const DISCHARGE: (&str, u32) = ("universe.blessing.612730", 2);
const FUNGAL: (&str, u32) = ("universe.blessing.612731", 2);
const SCYTHE: (&str, u32) = ("universe.blessing.612732", 2);
const ULCER: (&str, u32) = ("universe.blessing.612740", 2);
const LYTIC: (&str, u32) = ("universe.blessing.612741", 2);
const SPORE_RAW: u32 = 0x7b00_0001;

#[test]
fn goal07_p2_m09_s01_materializes_all_rules_and_shared_spore_engine() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612730",
        "StageAbility_612731",
        "StageAbility_612732",
        "StageAbility_612740",
        "StageAbility_612741",
    ] {
        let binding = binding(&contributions, key);
        let runtime = combat
            .rule(binding.rule())
            .unwrap_or_else(|| panic!("{key} is executable"))
            .runtime()
            .expect("generic runtime");
        assert!(runtime.native_handler().is_none());
    }

    let spore = combat
        .effect(EffectDefinitionId::new(SPORE_RAW).unwrap())
        .expect("shared Spore effect");
    let runtime = spore.runtime_template().expect("Spore runtime");
    assert_eq!(runtime.stack_limit(), 9);
    assert_eq!(
        runtime.duration_clock(),
        starclock_combat::DurationClock::Permanent
    );

    let first = binding(&contributions, "StageAbility_612730");
    let engine = combat.rule(first.rule()).unwrap().runtime().unwrap();
    assert!(engine.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::DamageApplied
            && trigger.once_scope == starclock_combat::rule::model::OnceScope::TargetWithinAction
    }));
    assert!(engine.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::UnitDefeated
            && trigger.phase == starclock_combat::rule::model::TriggerPhase::AfterDefeatSettlement
    }));
    assert!(
        combat
            .rule(first.rule())
            .unwrap()
            .programs()
            .iter()
            .any(|id| {
                combat.program(*id).is_some_and(|program| {
                    program.steps().iter().any(|step| {
                        matches!(
                            step,
                            ProgramStep::Operation(RuleOperationTemplate::UnboostedDamage {
                                class: DamageClass::Additional,
                                element: CombatElement::Wind,
                                can_defeat: true,
                                ..
                            })
                        )
                    })
                })
            })
    );
}

#[test]
fn fungal_pustule_uses_exact_grouped_random_target_contract() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_612731").rule())
        .unwrap();
    let operation = rule
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().steps())
        .find_map(|step| match step {
            ProgramStep::Operation(
                operation @ RuleOperationTemplate::RandomGroupedEffect {
                    effect,
                    applications_per_group: 2,
                    groups,
                    stacks,
                    ..
                },
            ) if effect.get() == SPORE_RAW
                && matches!(
                    groups,
                    ValueExpr::Convert { value, .. }
                        if matches!(
                            value.as_ref(),
                            ValueExpr::ReadEventProperty(EventValueProperty::ResourceDelta)
                        )
                )
                && matches!(
                    stacks,
                    ValueExpr::Literal(starclock_combat::rule::model::RuleValue::Integer(1))
                ) =>
            {
                Some(operation)
            }
            _ => None,
        });
    assert!(operation.is_some());
}

#[test]
fn enhanced_scythe_limbs_is_per_owner_and_expires_after_attack() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let binding = binding(&contributions, "StageAbility_612732");
    let rule = combat.rule(binding.rule()).unwrap();
    let runtime = rule.runtime().unwrap();
    assert_eq!(runtime.state_slots().len(), 1);
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && trigger.filter.ability_tag == Some(AbilityTag::Ultimate)
    }));
    assert_eq!(
        runtime
            .triggers()
            .iter()
            .filter(|trigger| trigger.event_point == RuleEventPoint::ResourceChanged)
            .count(),
        2
    );
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && trigger.filter.ability_tag == Some(AbilityTag::Attack)
    }));
    let modifier = combat
        .modifier(
            starclock_combat::ModifierDefinitionId::new(0x76c0_0000 + binding.rule().get())
                .unwrap(),
        )
        .expect("CRIT DMG modifier");
    assert_eq!(
        (modifier.stat, modifier.stage, modifier.purpose),
        (
            StatKind::CritDamage,
            FormulaStage::Flat,
            FormulaPurpose::Stat
        )
    );
    assert!(expression_has_scalar(&modifier.value, 450_000));
}

#[test]
fn recovered_skill_point_applies_one_spore_to_each_of_two_random_enemies() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let spec = durable_spec_with_two_enemies(&materialization, 0xb1);
    let (mut battle, started) = start(&materialization, spec, 0xb2);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    let spore = EffectDefinitionId::new(SPORE_RAW).unwrap();
    let stacks = battle
        .view()
        .effects_by_id()
        .filter(|effect| effect.definition() == spore)
        .map(|effect| effect.stacks())
        .collect::<Vec<_>>();
    assert_eq!(stacks, vec![1, 1], "{:#?}", resolution.events());
    assert!(battle.view().rng_draw_count() >= 2);
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.propagation",
        &[DISCHARGE, FUNGAL, SCYTHE, ULCER, LYTIC],
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

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(starclock_combat::rule::model::RuleValue::Scalar(value)) => {
            value.scaled() == expected
        }
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
