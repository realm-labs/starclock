use super::*;
use starclock_combat::{
    EffectRemovalOrder,
    catalog::selector::{RuleSelectorChoice, RuleSelectorOrigin},
    formula::model::DamageClass,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{
        EventValueProperty, ProgramStep, RuleEffectChancePolicy, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};

const DEWDROP_DISPEL: (&str, u32) = ("universe.blessing.612342", 2);
const HEALING_ATTACK: (&str, u32) = ("universe.blessing.612343", 2);
const HP_ADDITIONAL_DAMAGE: (&str, u32) = ("universe.blessing.612344", 2);
const FULL_HP_DEFENSE: (&str, u32) = ("universe.blessing.612345", 2);
const ALLY_HEALING_BONUS: (&str, u32) = ("universe.blessing.612346", 2);
const BLESSING_MAXIMUM_HP: (&str, u32) = ("universe.blessing.612350", 1);

#[test]
fn goal07_p2_m05_s02_materializes_all_assigned_mechanics_without_native_handlers() {
    let catalog = catalog();
    let selected = [
        DEWDROP_DISPEL,
        HEALING_ATTACK,
        HP_ADDITIONAL_DAMAGE,
        FULL_HP_DEFENSE,
        ALLY_HEALING_BONUS,
        BLESSING_MAXIMUM_HP,
    ];
    let contributions =
        contributions_many(&catalog, "universe.path.abundance", &selected, None, false);
    let roster = roster_for_forms(&catalog, [10, 1, 2, 3], None);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612342",
        "StageAbility_612343",
        "StageAbility_612344",
        "StageAbility_612345",
        "StageAbility_612346",
        "StageAbility_612350",
    ] {
        let binding = binding(&contributions, key);
        let rule = combat
            .rule(binding.rule())
            .expect("assigned executable rule");
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} stays in generic Rule IR"
        );
    }

    let dispel = binding(&contributions, "StageAbility_612342");
    let mut saw_fixed_chance = false;
    let mut saw_cleanse = false;
    for step in rule_steps(combat, dispel.rule()) {
        match step {
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                chance: RuleEffectChancePolicy::Fixed,
                base_chance: Some(ValueExpr::Literal(RuleValue::Scalar(value))),
                ..
            }) => {
                saw_fixed_chance = value.scaled() == 1_000_000;
            }
            ProgramStep::Operation(RuleOperationTemplate::Cleanse {
                maximum: 1,
                order: EffectRemovalOrder::OldestFirst,
                ..
            }) => saw_cleanse = true,
            _ => {}
        }
    }
    assert!(saw_fixed_chance && saw_cleanse);

    let healing_attack = binding(&contributions, "StageAbility_612343");
    let attack_effect = first_effect(combat, healing_attack.rule());
    let attack_runtime = attack_effect.runtime_template().unwrap();
    assert!(matches!(
        attack_runtime.duration_expression(),
        Some(ValueExpr::Literal(RuleValue::Integer(1)))
    ));
    let attack_modifier = combat.modifier(attack_effect.modifiers()[0]).unwrap();
    assert_eq!(
        (
            attack_modifier.stat,
            attack_modifier.stage,
            attack_modifier.purpose
        ),
        (
            StatKind::Atk,
            FormulaStage::PercentOfBase,
            FormulaPurpose::Stat
        )
    );
    assert_eq!(literal_scalar(&attack_modifier.value), 500_000);

    let additional = binding(&contributions, "StageAbility_612344");
    let additional_selector = rule_steps(combat, additional.rule())
        .into_iter()
        .find_map(|step| match step {
            ProgramStep::Operation(RuleOperationTemplate::DamageFromEventElement {
                selector,
                amount,
                class: DamageClass::Additional,
                can_crit: false,
                can_defeat: true,
            }) => {
                assert!(expression_has_stat(amount, StatKind::Hp));
                assert!(expression_has_scalar(amount, 420_000));
                Some(*selector)
            }
            _ => None,
        })
        .expect("enhanced HP damage inherits the event element");
    let selector = combat
        .selector(additional_selector)
        .unwrap()
        .rule_units()
        .unwrap();
    assert_eq!(selector.origin(), RuleSelectorOrigin::EventTargets);
    assert_eq!(selector.choice(), RuleSelectorChoice::RngUniform);
    assert_eq!(selector.rng_purpose(), Some("bounce-target"));

    let defense = binding(&contributions, "StageAbility_612345");
    let defense_effect = first_effect(combat, defense.rule());
    assert_eq!(
        defense_effect.modifiers().len(),
        8,
        "seven damage purposes plus enhanced Effect RES"
    );
    let modifiers = defense_effect
        .modifiers()
        .iter()
        .map(|id| combat.modifier(*id).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        modifiers
            .iter()
            .filter(|modifier| modifier.stage == FormulaStage::Mitigation)
            .count(),
        7
    );
    let resistance = modifiers
        .iter()
        .find(|modifier| modifier.stat == StatKind::EffectResistance)
        .unwrap();
    assert_eq!(literal_scalar(&resistance.value), 270_000);

    let bonus = binding(&contributions, "StageAbility_612346");
    let bonus_step = rule_steps(combat, bonus.rule())
        .into_iter()
        .find_map(|step| match step {
            ProgramStep::Operation(RuleOperationTemplate::Heal {
                amount,
                apply_formula_modifiers: false,
                ..
            }) => Some(amount),
            _ => None,
        })
        .unwrap();
    assert!(matches!(
        bonus_step,
        ValueExpr::Multiply { lhs, .. }
            if matches!(
                lhs.as_ref(),
                ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount)
            )
    ));
    assert!(expression_has_scalar(bonus_step, 450_000));

    let maximum_hp = binding(&contributions, "StageAbility_612350");
    let maximum_hp_modifier = first_modifier(combat, maximum_hp.rule());
    assert_eq!(
        (
            maximum_hp_modifier.stat,
            maximum_hp_modifier.stage,
            maximum_hp_modifier.purpose
        ),
        (
            StatKind::Hp,
            FormulaStage::PercentOfBase,
            FormulaPurpose::Stat
        )
    );
    assert_eq!(
        literal_scalar(&maximum_hp_modifier.value),
        300_000,
        "level 1 counts the six selected Abundance Blessings and applies its six-stack cap"
    );
}

#[test]
fn enhanced_hp_additional_damage_executes_once_on_an_actual_attack_target() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.abundance",
        &[HP_ADDITIONAL_DAMAGE],
        None,
        false,
    );
    let roster = roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let (mut battle, started) = start(
        &materialization,
        durable_spec_with_two_enemies(&materialization, 0xa1),
        0xa2,
    );
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = first_normal_action(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let source = binding(&contributions, "StageAbility_612344")
        .source()
        .definition();
    let attacked = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.class == DamageClass::Direct => Some(data.target),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    let damage = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data)
                if event.cause().source_definition() == Some(source)
                    && data.class == DamageClass::Additional =>
            {
                Some(data)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(damage.len(), 1, "{:?}", resolution.events());
    assert!(
        attacked.contains(&damage[0].target),
        "the random retarget remains inside the committed attack target list"
    );
}

fn binding<'a>(
    contributions: &'a UniverseBattleContributionSet,
    key: &str,
) -> &'a starclock_mode_universe::battle_contribution::UniverseBattleRuleBinding {
    contributions
        .rules()
        .iter()
        .find(|binding| binding.source_binding_key() == Some(key))
        .unwrap()
}

fn rule_steps(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> Vec<&ProgramStep> {
    combat
        .rule(rule)
        .unwrap()
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.steps())
        .collect()
}

fn first_effect(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::catalog::definition::EffectDefinition {
    combat
        .rule(rule)
        .unwrap()
        .programs()
        .iter()
        .filter_map(|program| combat.program(*program))
        .flat_map(|program| program.effects())
        .find_map(|effect| combat.effect(*effect))
        .unwrap()
}

fn first_modifier(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::modifier::model::ModifierDefinition {
    let effect = first_effect(combat, rule);
    combat.modifier(effect.modifiers()[0]).unwrap()
}

fn literal_scalar(value: &ValueExpr) -> i64 {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled(),
        _ => panic!("expected literal scalar: {value:?}"),
    }
}

fn expression_has_scalar(value: &ValueExpr, expected: i64) -> bool {
    match value {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_has_scalar(lhs, expected) || expression_has_scalar(rhs, expected)
        }
        _ => false,
    }
}

fn expression_has_stat(value: &ValueExpr, expected: StatKind) -> bool {
    match value {
        ValueExpr::QueryStat { stat, .. } => *stat == expected,
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_stat(lhs, expected) || expression_has_stat(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_has_stat(lhs, expected) || expression_has_stat(rhs, expected)
        }
        _ => false,
    }
}
