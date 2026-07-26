use super::*;
use starclock_combat::{
    EffectDefinitionId,
    catalog::action::TargetPattern,
    modifier::model::{FormulaPurpose, FormulaStage, FormulaSubject, StatKind},
    rule::model::{
        EventValueProperty, ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};

const MEMORY: (&str, u32) = ("universe.blessing.612842", 1);
const TACTILE: (&str, u32) = ("universe.blessing.612843", 2);
const SUBLIMINAL: (&str, u32) = ("universe.blessing.612844", 1);
const STRIATED: (&str, u32) = ("universe.blessing.612845", 2);
const SALTATORY: (&str, u32) = ("universe.blessing.612846", 2);
const GEARS: (&str, u32) = ("universe.blessing.612850", 1);
const LOCAL_EFFECT_BASE: u32 = 0x7e20_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7e60_0000;

#[test]
fn goal07_p2_m10_s02_materializes_all_assigned_rules_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612842",
        "StageAbility_612843",
        "StageAbility_612844",
        "StageAbility_612845",
        "StageAbility_612846",
        "StageAbility_612850",
    ] {
        let rule = combat
            .rule(binding(&contributions, key).rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }
}

#[test]
fn striated_cortex_uses_generic_aoe_shape_and_exact_original_damage_fraction() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_612845").rule())
        .unwrap();
    let runtime = rule.runtime().unwrap();

    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::ActionResolved
            && trigger.filter.target_pattern == Some(TargetPattern::All)
    }));
    assert!(rule.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| match step {
                ProgramStep::Operation(RuleOperationTemplate::AddSlot { value, .. }) => {
                    expression_has_event(value, EventValueProperty::DamageRawAmount)
                }
                _ => false,
            })
    }));
    assert!(rule.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| match step {
                ProgramStep::Operation(RuleOperationTemplate::TrueDamage { amount, .. }) => {
                    expression_has_scalar(amount, 600_000)
                }
                _ => false,
            })
    }));
}

#[test]
fn striated_cortex_executes_one_exact_fixed_hit_after_an_aoe_skill_with_one_enemy() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.erudition",
        &[STRIATED],
        None,
        false,
    );
    let roster = roster_for_forms_with_ability_kinds(
        &catalog,
        [18, 2, 3, 4],
        None,
        &[AbilityKind::Skill],
        false,
    );
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let ability_definition = materialization
        .combat_catalog()
        .ability(AbilityId::new(20019).unwrap())
        .unwrap();
    assert_eq!(
        materialization
            .combat_catalog()
            .selector(ability_definition.selector())
            .unwrap()
            .unit_targets()
            .unwrap()
            .pattern(),
        TargetPattern::All
    );
    assert!(
        materialization
            .combat_catalog()
            .trigger_ids(
                starclock_combat::rule::model::RuleEventKind::Damage,
                starclock_combat::rule::model::TriggerPhase::AfterEvent,
            )
            .any(|(rule, _)| rule == binding(&contributions, "StageAbility_612845").rule())
    );
    let (mut battle, started) = start(
        &materialization,
        durable_spec(&materialization, 0xd1, false),
        0xd2,
    );
    assert!(started.fault().is_none(), "{:#?}", started.events());
    let striated_rule = binding(&contributions, "StageAbility_612845").rule();
    assert!(
        battle
            .view()
            .rule_instances_by_id()
            .any(|instance| instance.rule() == striated_rule && instance.owner().is_some())
    );
    if battle
        .decision()
        .is_some_and(|decision| decision.kind() == starclock_combat::DecisionKind::InterruptWindow)
    {
        battle
            .apply(Command::PassInterruptWindow {
                decision: battle.decision().unwrap().id(),
            })
            .unwrap();
    }
    let aoe = AbilityId::new(20019).unwrap();
    let command = battle
        .decision()
        .unwrap()
        .legal_commands()
        .iter()
        .find(|command| matches!(command, Command::UseAbility { ability, .. } if *ability == aoe))
        .expect("form 18 AoE Skill is legal")
        .clone();
    let resolution = battle.apply(command).unwrap();
    assert!(resolution.fault().is_none(), "{:#?}", resolution.events());
    let direct_raw = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if event.cause().source_definition()
                    == Some(starclock_combat::SourceDefinitionId::new(aoe.get()).unwrap()) =>
            {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .sum::<i64>();
    let fixed_raw = resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if event.cause().source_definition()
                    == Some(
                        binding(&contributions, "StageAbility_612845")
                            .source()
                            .definition(),
                    ) =>
            {
                Some(data.raw.scaled())
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "Striated Cortex emits one fixed damage event; events={:#?}",
                resolution.events()
            )
        });
    assert!(direct_raw > 0);
    assert_eq!(fixed_raw, direct_raw * 600_000 / 1_000_000);
}

#[test]
fn subliminal_and_gears_author_exact_ultimate_modifiers_and_entry_energy() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let subliminal = combat
        .rule(binding(&contributions, "StageAbility_612844").rule())
        .unwrap();
    let effect = EffectDefinitionId::new(
        LOCAL_EFFECT_BASE
            + (binding(&contributions, "StageAbility_612844").rule().get() & 0xffff) * 16,
    )
    .unwrap();
    let modifier = combat
        .modifier(combat.effect(effect).unwrap().modifiers()[0])
        .unwrap();
    assert_eq!(
        (modifier.stat, modifier.stage, modifier.purpose),
        (
            StatKind::Atk,
            FormulaStage::DamageBoost,
            FormulaPurpose::OrdinaryDamage
        )
    );
    assert!(modifier.filters.iter().any(|filter| {
        matches!(
            filter,
            starclock_combat::modifier::model::ModifierFilter::FormulaSubject(
                FormulaSubject::Source
            )
        )
    }));
    assert!(expression_has_scalar(&modifier.value, 500_000));
    assert!(subliminal.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| match step {
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    amount, ..
                }) => expression_has_scalar(amount, 600_000),
                _ => false,
            })
    }));

    let gears_raw = binding(&contributions, "StageAbility_612850").rule().get();
    let gears_modifier = combat
        .modifier(
            starclock_combat::ModifierDefinitionId::new(
                LOCAL_MODIFIER_BASE + (gears_raw & 0xffff) * 16,
            )
            .unwrap(),
        )
        .unwrap();
    assert!(
        expression_has_scalar(&gears_modifier.value, 420_000),
        "six selected Erudition blessings are capped at 6 × 7%"
    );
}

#[test]
fn tactile_and_saltatory_retain_released_caps_and_per_enemy_reset_points() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let tactile = combat
        .rule(binding(&contributions, "StageAbility_612843").rule())
        .unwrap();
    assert!(tactile.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| match step {
                ProgramStep::Operation(RuleOperationTemplate::DamageFromEventElement {
                    amount,
                    ..
                }) => expression_has_scalar(amount, 200_000) && expression_has_integer(amount, 5),
                _ => false,
            })
    }));

    let saltatory = combat
        .rule(binding(&contributions, "StageAbility_612846").rule())
        .unwrap();
    let runtime = saltatory.runtime().unwrap();
    assert!(runtime.triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::WeaknessBroken
            && trigger.filter.target_selector.is_some()
    }));
    assert!(saltatory.programs().iter().any(|program| {
        combat
            .program(*program)
            .unwrap()
            .steps()
            .iter()
            .any(|step| match step {
                ProgramStep::Operation(RuleOperationTemplate::DelayAction { amount, .. }) => {
                    expression_has_scalar(amount, 240_000)
                }
                _ => false,
            })
    }));
    assert_eq!(
        runtime.state_slots()[0].maximum(),
        Some(&RuleValue::Integer(3))
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.erudition",
        &[MEMORY, TACTILE, SUBLIMINAL, STRIATED, SALTATORY, GEARS],
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
    expression_contains(
        value,
        |value| matches!(value, RuleValue::Scalar(value) if value.scaled() == expected),
    )
}

fn expression_has_integer(value: &ValueExpr, expected: i64) -> bool {
    expression_contains(
        value,
        |value| matches!(value, RuleValue::Integer(value) if *value == expected),
    )
}

fn expression_contains(value: &ValueExpr, predicate: impl Fn(&RuleValue) -> bool + Copy) -> bool {
    match value {
        ValueExpr::Literal(value) => predicate(value),
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_contains(lhs, predicate) || expression_contains(rhs, predicate)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_contains(value, predicate)
        }
        _ => false,
    }
}

fn expression_has_event(value: &ValueExpr, expected: EventValueProperty) -> bool {
    match value {
        ValueExpr::ReadEventProperty(property) => *property == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_event(lhs, expected) || expression_has_event(rhs, expected)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_event(value, expected)
        }
        _ => false,
    }
}
