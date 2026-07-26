use super::*;
use starclock_combat::{
    DurationClock, EffectCategory, ModifierDefinitionId,
    catalog::{action::AbilityTag, selector::RuleSelectorChoice},
    formula::model::DamageClass,
    modifier::model::{FormulaPurpose, FormulaStage, StatKind},
    rule::model::{
        ProgramStep, RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};

const RANDOM: (&str, u32) = ("universe.blessing.612630", 2);
const CHAMPION: (&str, u32) = ("universe.blessing.612632", 2);
const HOURGLASS: (&str, u32) = ("universe.blessing.612642", 2);
const ALBATROSS: (&str, u32) = ("universe.blessing.612643", 2);
const MONKEYS: (&str, u32) = ("universe.blessing.612644", 2);
const AIDEN: (&str, u32) = ("universe.blessing.612645", 2);
const MILITARY: (&str, u32) = ("universe.blessing.612646", 2);
const EXEMPLARY: (&str, u32) = ("universe.blessing.612650", 1);

#[test]
fn goal07_p2_m08_s02_materializes_every_selected_level_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize_with_roster(
        &catalog,
        &super::nihility_s02::kafka_roster(&catalog),
        &contributions,
    );
    let combat = materialization.combat_catalog();
    for key in [
        "StageAbility_612642",
        "StageAbility_612643",
        "StageAbility_612644",
        "StageAbility_612645",
        "StageAbility_612646",
        "StageAbility_612650",
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
fn hourglass_and_twelve_monkeys_preserve_distinct_type_and_per_hit_clocks() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let hourglass = combat
        .rule(binding(&contributions, "StageAbility_612642").rule())
        .unwrap();
    assert_eq!(hourglass.programs().len(), 7);
    let hourglass_effects = hourglass
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .filter_map(|effect| combat.effect(*effect))
        .collect::<Vec<_>>();
    assert_eq!(hourglass_effects.len(), 7);
    assert!(hourglass_effects.iter().all(|effect| {
        let runtime = effect.runtime_template().unwrap();
        let modifier = combat.modifier(effect.modifiers()[0]).unwrap();
        runtime.duration_clock() == DurationClock::TargetActionEnd
            && modifier.stat == StatKind::Atk
            && modifier.stage == FormulaStage::PercentOfBase
            && expression_has_scalar(&modifier.value, -60_000)
    }));

    let monkeys = combat
        .rule(binding(&contributions, "StageAbility_612644").rule())
        .unwrap();
    let effect = monkeys
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .find_map(|effect| combat.effect(*effect))
        .expect("per-hit ramp effect");
    let runtime = effect.runtime_template().unwrap();
    assert_eq!(runtime.duration_clock(), DurationClock::ActionEnd);
    assert_eq!(runtime.stack_limit(), 64);
    assert!(effect.modifiers().iter().all(|modifier| {
        let modifier = combat.modifier(*modifier).unwrap();
        modifier.source_stack_slot.is_some()
            && modifier.stage == FormulaStage::DamageBoost
            && modifier.purpose == FormulaPurpose::OrdinaryDamage
            && expression_has_scalar(&modifier.value, 60_000)
    }));
    assert!(monkeys.runtime().unwrap().triggers().iter().any(|trigger| {
        trigger.event_point == RuleEventPoint::HitStarted
            && trigger.filter.ability_tag == Some(AbilityTag::Ultimate)
    }));
}

#[test]
fn albatross_aiden_and_exemplary_conduct_keep_exact_structural_semantics() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();

    let albatross = combat
        .rule(binding(&contributions, "StageAbility_612643").rule())
        .unwrap();
    assert!(albatross.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::DamageFromEventElement {
                        amount,
                        class: DamageClass::Additional,
                        ..
                    }) if expression_has_scalar(amount, 360_000)
                )
            })
        })
    }));
    assert!(albatross.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program
                .steps()
                .iter()
                .any(|step| matches!(step, ProgramStep::ForEach { maximum: 16, .. }))
        })
    }));

    let aiden = combat
        .rule(binding(&contributions, "StageAbility_612645").rule())
        .unwrap();
    assert!(aiden.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.selectors().iter().any(|selector| {
                combat
                    .selector(*selector)
                    .and_then(|definition| definition.rule_units())
                    .is_some_and(|selector| {
                        selector.choice() == RuleSelectorChoice::RngUniform
                            && selector.maximum() == 16
                            && !selector.repeated()
                    })
            })
        })
    }));
    let aiden_effect = aiden
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().effects())
        .find_map(|effect| combat.effect(*effect))
        .expect("enhanced Aiden control");
    assert_eq!(
        aiden_effect.runtime_template().unwrap().category(),
        EffectCategory::Control
    );
    assert!(aiden.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                        chance: RuleEffectChancePolicy::Resistible,
                        base_chance: Some(value),
                        ..
                    }) if expression_has_scalar(value, 100_000)
                )
            })
        })
    }));
    let delays = aiden
        .programs()
        .iter()
        .flat_map(|program| combat.program(*program).unwrap().steps())
        .filter_map(|step| match step {
            ProgramStep::Operation(RuleOperationTemplate::DelayAction { amount, .. }) => {
                literal_scalar(amount)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(delays.contains(&120_000) && delays.contains(&200_000));

    let exemplary = combat
        .rule(binding(&contributions, "StageAbility_612650").rule())
        .unwrap();
    let raw = binding(&contributions, "StageAbility_612650").rule().get();
    let modifiers = [0x76c0_0000, 0x79e7_0000, 0x79ec_0000]
        .into_iter()
        .map(|base| {
            combat
                .modifier(ModifierDefinitionId::new(base + raw).expect("reserved modifier ID"))
                .expect("Exemplary Conduct modifier")
        })
        .collect::<Vec<_>>();
    assert!(
        modifiers.iter().all(|modifier| {
            modifier.source_stack_slot.is_none() && expression_has_scalar(&modifier.value, 540_000)
        }),
        "six-count cap yields 54% for follow-up, counter, and Champion Ultimate"
    );
    assert!(exemplary.runtime().unwrap().triggers().is_empty());
}

#[test]
fn military_rule_level_one_uses_one_fixed_65_percent_roll_per_action() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.elation",
        &[("universe.blessing.612646", 1)],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let rule = combat
        .rule(binding(&contributions, "StageAbility_612646").rule())
        .unwrap();
    assert!(rule.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                        chance: RuleEffectChancePolicy::Fixed,
                        base_chance: Some(value),
                        ..
                    }) if expression_has_scalar(value, 650_000)
                )
            })
        })
    }));
    assert!(rule.runtime().unwrap().triggers().iter().all(|trigger| {
        trigger.event_point != RuleEventPoint::ActionResolved
            || trigger.once_scope == starclock_combat::rule::model::OnceScope::Action
    }));
}

#[test]
fn production_kafka_ultimate_drives_repeated_aoe_ramp_delay_and_skill_point_rules() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let roster = super::nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = durable_spec_with_two_enemy_hp(
        &materialization,
        0xec,
        [
            Hp::new(2_000_000_000).unwrap(),
            Hp::new(2_000_000_000).unwrap(),
        ],
    );
    let (mut battle, started) = start(&materialization, spec, 0xed);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let before_skill_points = battle.view().team(TeamSide::Player).skill_points();
    let resolution = super::nihility_s02::use_kafka_ultimate(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:#?}",
        resolution.fault(),
        resolution.events()
    );

    let albatross_source = binding(&contributions, "StageAbility_612643")
        .source()
        .definition();
    let additional = resolution
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Damage(data)
                    if data.class == DamageClass::Additional
                        && event.cause().source_definition() == Some(albatross_source)
            )
        })
        .count();
    assert_eq!(
        additional,
        4,
        "two hit enemies each cause one additional-damage pass over both enemies: {:#?}",
        resolution.events()
    );
    assert_eq!(
        battle.view().team(TeamSide::Player).skill_points(),
        before_skill_points.saturating_add(1).min(5),
        "enhanced Military Rule recovers exactly one team Skill Point"
    );
    assert!(resolution.events().iter().any(|event| {
        matches!(
            event.kind(),
            BattleEventKind::Effect(starclock_combat::EffectEventData::Applied { .. })
        ) && event.cause().source_definition()
            == Some(
                binding(&contributions, "StageAbility_612644")
                    .source()
                    .definition(),
            )
    }));
    let aiden_source = binding(&contributions, "StageAbility_612645")
        .source()
        .definition();
    let delayed = resolution
        .events()
        .iter()
        .filter_map(|event| match event.kind() {
            BattleEventKind::Turn(starclock_combat::TurnEventData::ActionGaugeChanged {
                owner,
                kind: starclock_combat::ActionGaugeChangeKind::Delay,
                amount,
                ..
            }) if event.cause().source_definition() == Some(aiden_source)
                && amount.scaled() == 120_000 =>
            {
                Some(*owner)
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        delayed.len(),
        2,
        "Aiden unconditionally delays every enemy hit by the production Ultimate"
    );
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.elation",
        &[
            RANDOM, CHAMPION, HOURGLASS, ALBATROSS, MONKEYS, AIDEN, MILITARY, EXEMPLARY,
        ],
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

fn expression_has_scalar(expression: &ValueExpr, expected: i64) -> bool {
    match expression {
        ValueExpr::Literal(RuleValue::Scalar(value)) => value.scaled() == expected,
        ValueExpr::Negate(value) => expression_has_scalar(value, expected),
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

fn literal_scalar(expression: &ValueExpr) -> Option<i64> {
    match expression {
        ValueExpr::Literal(RuleValue::Scalar(value)) => Some(value.scaled()),
        _ => None,
    }
}
