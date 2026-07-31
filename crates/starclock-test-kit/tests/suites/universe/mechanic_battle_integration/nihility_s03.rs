use super::*;
use starclock_combat::{
    EffectCategory,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{RuleValue, ValueExpr},
};

const IGNOSTICISM: (&str, u32) = ("universe.blessing.612250", 2);
const QUESTIONING: (&str, u32) = ("universe.blessing.612251", 2);
const BLIND_VISION: (&str, u32) = ("universe.blessing.612252", 2);
const TRAGIC_LECTURE: (&str, u32) = ("universe.blessing.612253", 2);
const SENSORY_LABYRINTH: (&str, u32) = ("universe.blessing.612254", 2);
const EMOTIONAL_DECLUTTERING: (&str, u32) = ("universe.blessing.612255", 2);

#[test]
fn goal07_p2_m04_s03_materializes_every_assigned_modifier_with_exact_values() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[
            ("universe.blessing.612230", 1),
            ("universe.blessing.612231", 1),
            ("universe.blessing.612232", 1),
            ("universe.blessing.612240", 1),
            ("universe.blessing.612241", 1),
            ("universe.blessing.612242", 1),
            ("universe.blessing.612243", 1),
            ("universe.blessing.612244", 1),
            ("universe.blessing.612245", 1),
            IGNOSTICISM,
            QUESTIONING,
            BLIND_VISION,
            TRAGIC_LECTURE,
            SENSORY_LABYRINTH,
            EMOTIONAL_DECLUTTERING,
        ],
        None,
        false,
    );
    let roster = super::nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let combat = materialization.combat_catalog();

    let questioning = first_modifier(
        combat,
        binding(&contributions, "StageAbility_612251").rule(),
    );
    assert_eq!(
        (questioning.stage, questioning.purpose),
        (FormulaStage::DamageBoost, FormulaPurpose::Break)
    );
    assert_eq!(literal_scalar(&questioning.value), 750_000);

    let blind = first_modifier(
        combat,
        binding(&contributions, "StageAbility_612252").rule(),
    );
    assert_eq!(blind.stat, StatKind::EffectResistance);
    assert_eq!(literal_scalar(&blind.value), -180_000);

    let tragic = first_modifier(
        combat,
        binding(&contributions, "StageAbility_612253").rule(),
    );
    assert_eq!(
        (tragic.stage, tragic.purpose),
        (FormulaStage::Vulnerability, FormulaPurpose::Dot)
    );
    assert_eq!(literal_scalar(&tragic.value), 150_000);

    let sensory = first_modifier(
        combat,
        binding(&contributions, "StageAbility_612254").rule(),
    );
    assert_eq!(sensory.stat, StatKind::DotDurationAddition);
    assert_eq!(literal_scalar(&sensory.value), 2_000_000);

    let emotional_rule = binding(&contributions, "StageAbility_612255").rule();
    let emotional_effect = first_effect(combat, emotional_rule);
    assert_eq!(emotional_effect.modifiers().len(), 7);
    for modifier in emotional_effect
        .modifiers()
        .iter()
        .map(|id| combat.modifier(*id).unwrap())
    {
        assert_eq!(modifier.stage, FormulaStage::Vulnerability);
        assert!(
            expression_has_category_stacks(&modifier.value, EffectCategory::Dot)
                && expression_has_scalar(&modifier.value, 200_000),
            "enhanced Emotional Decluttering is 4% per current DoT stack capped at 5"
        );
    }

    let ignosticism = first_modifier(
        combat,
        binding(&contributions, "StageAbility_612250").rule(),
    );
    assert_eq!(
        literal_scalar(&ignosticism.value),
        720_000,
        "enhanced Ignosticism counts at most nine selected Nihility Blessings"
    );
}

#[test]
fn sensory_labyrinth_extends_a_production_kafka_shock_by_two_target_turns() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[SENSORY_LABYRINTH],
        None,
        false,
    );
    let roster = super::nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = durable_spec_with_enemy_hp(
        &materialization,
        0x71,
        false,
        Hp::new(9_000_000_000_000).unwrap(),
    );
    let (mut battle, started) = start(&materialization, spec, 0x72);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = super::nihility_s02::use_kafka_ultimate(&mut battle);
    assert!(resolution.fault().is_none(), "{:?}", resolution.fault());
    let enemy = battle
        .view()
        .units_by_id()
        .find(|unit| unit.side() == TeamSide::Enemy)
        .unwrap()
        .id();
    let shock = battle
        .view()
        .effects_by_id()
        .find(|effect| effect.target() == enemy && effect.category() == EffectCategory::Dot)
        .unwrap_or_else(|| {
            panic!(
                "Kafka Ultimate applies a production DoT: {:?}",
                resolution.events()
            )
        });
    assert_eq!(
        shock.remaining(),
        Some(4),
        "Kafka's ordinary two-turn Shock receives the enhanced +2 DoT duration"
    );
}

#[test]
fn questioning_of_purpose_increases_a_production_initial_break_by_exactly_half() {
    let catalog = catalog();
    let baseline = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[("universe.blessing.612230", 1)],
        None,
        false,
    );
    let questioning = contributions_many(
        &catalog,
        "universe.path.nihility",
        &[("universe.blessing.612251", 1)],
        None,
        false,
    );
    let base = kafka_initial_break(&catalog, &baseline, 0x73);
    let boosted = kafka_initial_break(&catalog, &questioning, 0x74);
    assert_eq!(boosted.scaled() * 2, base.scaled() * 3);
}

fn kafka_initial_break(
    catalog: &Arc<UniverseCatalog>,
    contributions: &UniverseBattleContributionSet,
    marker: u8,
) -> starclock_combat::Scalar {
    let roster = super::nihility_s02::kafka_roster(catalog);
    let materialization = materialize_with_roster(catalog, &roster, contributions);
    let spec = super::nihility_s02::two_enemy_break_spec(&materialization, marker);
    let (mut battle, started) = start(&materialization, spec, marker.wrapping_add(1));
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = super::nihility_s02::use_kafka_ultimate(&mut battle);
    resolution
        .events()
        .iter()
        .find_map(|event| match event.kind() {
            BattleEventKind::BreakDamage(value)
                if value.kind == starclock_combat::BreakDamageKind::Initial =>
            {
                Some(value.raw)
            }
            _ => None,
        })
        .expect("Kafka breaks the Lightning-weak production target")
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

fn first_effect(
    combat: &starclock_combat::catalog::CombatCatalog,
    rule: starclock_combat::RuleId,
) -> &starclock_combat::catalog::definition::EffectDefinition {
    let program = combat
        .program(combat.rule(rule).unwrap().programs()[0])
        .unwrap();
    combat.effect(program.effects()[0]).unwrap()
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
        _ => panic!("expected literal scalar"),
    }
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
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            expression_has_scalar(value, expected)
                || expression_has_scalar(minimum, expected)
                || expression_has_scalar(maximum, expected)
        }
        _ => false,
    }
}

fn expression_has_category_stacks(value: &ValueExpr, expected: EffectCategory) -> bool {
    match value {
        ValueExpr::QueryEffectCategoryStacks { category, .. } => *category == expected,
        ValueExpr::Multiply { lhs, rhs, .. }
        | ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Divide { lhs, rhs, .. }
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_has_category_stacks(lhs, expected)
                || expression_has_category_stacks(rhs, expected)
        }
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            expression_has_category_stacks(value, expected)
                || expression_has_category_stacks(minimum, expected)
                || expression_has_category_stacks(maximum, expected)
        }
        ValueExpr::Negate(value) | ValueExpr::Convert { value, .. } => {
            expression_has_category_stacks(value, expected)
        }
        _ => false,
    }
}
