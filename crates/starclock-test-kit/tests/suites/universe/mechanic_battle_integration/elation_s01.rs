use super::*;
use starclock_combat::{
    ModifierDefinitionId,
    catalog::{action::AbilityTag, selector::RuleSelectorChoice},
    formula::model::DamageClass,
    modifier::model::{FormulaPurpose, FormulaStage},
    rule::model::{
        EventValueProperty, ProgramStep, RuleEventPoint, RuleOperationTemplate, RuleValue,
        ValueExpr,
    },
};
use super::{nihility_s02};

const RANDOM: (&str, u32) = ("universe.blessing.612630", 2);
const BROKEN: (&str, u32) = ("universe.blessing.612631", 2);
const CHAMPION: (&str, u32) = ("universe.blessing.612632", 2);
const EXTRA: (&str, u32) = ("universe.blessing.612640", 2);
const VULNERABILITY: (&str, u32) = ("universe.blessing.612641", 2);
const KAFKA_ULTIMATE: u32 = 20_033;

#[test]
fn goal07_p2_m08_s01_materializes_every_selected_level_without_native_handlers() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let materialization = materialize_with_roster(
        &catalog,
        &nihility_s02::kafka_roster(&catalog),
        &contributions,
    );
    let combat = materialization.combat_catalog();

    for key in [
        "StageAbility_612630",
        "StageAbility_612631",
        "StageAbility_612632",
        "StageAbility_612640",
        "StageAbility_612641",
    ] {
        let binding = binding(&contributions, key);
        let rule = combat
            .rule(binding.rule())
            .unwrap_or_else(|| panic!("{key} is executable"));
        assert!(
            rule.runtime()
                .is_some_and(|runtime| runtime.native_handler().is_none()),
            "{key} remains generic Rule IR"
        );
    }

    for (offset, expected_tag) in [(2, "follow_up"), (3, "counter"), (4, "ultimate")] {
        let modifier = combat
            .modifier(ModifierDefinitionId::new(0x79e2_0000 + offset).unwrap())
            .expect("Champion modifier");
        assert!(
            modifier.purpose == FormulaPurpose::OrdinaryDamage
                && modifier.stage == FormulaStage::DamageBoost
                && expression_has_scalar(&modifier.value, 550_000)
                && modifier.filters.iter().any(|filter| matches!(
                    filter,
                    starclock_combat::modifier::model::ModifierFilter::AbilityTag(tag)
                        if tag.as_ref() == expected_tag
                ))
        );
    }

    let random = combat
        .rule(binding(&contributions, "StageAbility_612630").rule())
        .unwrap();
    assert!(random.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::RandomRepeatedDamage {
                        class: DamageClass::Elation,
                        minimum_hits: 1,
                        maximum_hits: 3,
                        elements,
                        ..
                    }) if elements.len() == 7
                )
            })
        })
    }));
    assert!(random.programs().iter().any(|program| {
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
    let broken = combat
        .rule(binding(&contributions, "StageAbility_612631").rule())
        .unwrap();
    assert!(broken.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::If {
                        condition:
                            starclock_combat::rule::model::ConditionExpr::CurrentTargetIsBroken,
                        ..
                    }
                )
            })
        })
    }));
    let portrait = combat
        .rule(binding(&contributions, "StageAbility_612640").rule())
        .unwrap();
    assert!(portrait.programs().iter().any(|program| {
        combat.program(*program).is_some_and(|program| {
            program.steps().iter().any(|step| {
                matches!(
                    step,
                    ProgramStep::Operation(RuleOperationTemplate::RandomRepeatedDamage {
                        amount,
                        ..
                    }) if expression_reads(amount, EventValueProperty::DamageRawAmount)
                )
            })
        })
    }));

    for key in ["StageAbility_612630", "StageAbility_612631"] {
        let runtime = combat
            .rule(binding(&contributions, key).rule())
            .unwrap()
            .runtime()
            .unwrap();
        assert!(runtime.triggers().iter().any(|trigger| {
            trigger.event_point == RuleEventPoint::ActionResolved
                && trigger.filter.ability_tag == Some(AbilityTag::Ultimate)
        }));
    }
}

#[test]
fn random_repeated_damage_and_aftertaste_chain_execute_in_a_production_ultimate() {
    let catalog = catalog();
    let contributions = full_contributions(&catalog);
    let roster = nihility_s02::kafka_roster(&catalog);
    let materialization = materialize_with_roster(&catalog, &roster, &contributions);
    let spec = durable_spec_with_two_enemy_hp(
        &materialization,
        0xea,
        [
            Hp::new(2_000_000_000).unwrap(),
            Hp::new(2_000_000_000).unwrap(),
        ],
    );
    let (mut battle, started) = start(&materialization, spec, 0xeb);
    assert!(started.fault().is_none(), "{:?}", started.fault());
    let resolution = nihility_s02::use_kafka_ultimate(&mut battle);
    assert!(
        resolution.fault().is_none(),
        "{:?} {:#?}",
        resolution.fault(),
        resolution.events()
    );

    let random_source = binding(&contributions, "StageAbility_612630")
        .source()
        .definition();
    let broken_source = binding(&contributions, "StageAbility_612631")
        .source()
        .definition();
    let extra_source = binding(&contributions, "StageAbility_612640")
        .source()
        .definition();
    let mut source_counts = [0_u16; 3];
    let mut elation_events = Vec::new();
    for event in resolution.events() {
        let BattleEventKind::Damage(data) = event.kind() else {
            continue;
        };
        if data.class != DamageClass::Elation {
            continue;
        }
        let source = event.cause().source_definition();
        if source == Some(random_source) {
            source_counts[0] += 1;
        } else if source == Some(broken_source) {
            source_counts[1] += 1;
        } else if source == Some(extra_source) {
            source_counts[2] += 1;
        }
        elation_events.push((
            event.id(),
            event.cause().parent_event(),
            source,
            data.element,
        ));
    }
    assert!(
        (2..=6).contains(&source_counts[0]),
        "{source_counts:?} {:#?}",
        resolution.events()
    );
    assert_eq!(
        source_counts[1], 2,
        "each unbroken committed target receives one normal hit"
    );
    assert_eq!(
        source_counts[2],
        source_counts[0] + source_counts[1],
        "Portrait creates exactly one nonrecursive extra Aftertaste per original instance"
    );
    assert!(elation_events.len() >= 4);

    for (_, parent, source, element) in &elation_events {
        if *source != Some(extra_source) {
            continue;
        }
        let original = elation_events
            .iter()
            .find(|(id, _, candidate, _)| Some(*id) == *parent && *candidate != Some(extra_source))
            .expect("extra Aftertaste retains its original instance as cause parent");
        assert_ne!(
            *element, original.3,
            "Portrait chooses an element different from the triggering Aftertaste"
        );
    }
    assert!(
        resolution.events().iter().any(|event| {
            matches!(
                event.kind(),
                BattleEventKind::Action(starclock_combat::ActionEventData::Resolved {
                    ability,
                    ..
                }) if ability.get() == KAFKA_ULTIMATE
            )
        }),
        "the production Ultimate remains the root action"
    );
}

#[test]
fn aftertaste_types_install_distinct_one_turn_vulnerability_effects() {
    let catalog = catalog();
    let contributions = contributions_many(
        &catalog,
        "universe.path.elation",
        &[RANDOM, VULNERABILITY],
        None,
        false,
    );
    let materialization = materialize(&catalog, &contributions);
    let combat = materialization.combat_catalog();
    let effects = (1..=7)
        .map(|offset| {
            combat
                .effect(starclock_combat::EffectDefinitionId::new(0x79e1_0000 + offset).unwrap())
                .expect("per-element Aftertaste vulnerability effect")
        })
        .collect::<Vec<_>>();
    assert_eq!(effects.len(), 7);
    assert!(effects.iter().all(|effect| {
        let runtime = effect.runtime_template().unwrap();
        runtime.duration_expression() == Some(&ValueExpr::Literal(RuleValue::Integer(1)))
            && runtime.duration_clock() == starclock_combat::DurationClock::TargetActionEnd
            && effect.modifiers().len() == 7
    }));
    let values = effects
        .iter()
        .flat_map(|effect| effect.modifiers())
        .map(|modifier| combat.modifier(*modifier).unwrap())
        .filter(|modifier| modifier.purpose == FormulaPurpose::ElationDamage)
        .collect::<Vec<_>>();
    assert_eq!(values.len(), 7);
    assert!(
        values
            .iter()
            .all(|modifier| expression_has_scalar(&modifier.value, 120_000))
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
        .unwrap_or_else(|| panic!("{key} selected"))
}

fn full_contributions(catalog: &Arc<UniverseCatalog>) -> UniverseBattleContributionSet {
    contributions_many(
        catalog,
        "universe.path.elation",
        &[RANDOM, BROKEN, CHAMPION, EXTRA, VULNERABILITY],
        None,
        false,
    )
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

fn expression_reads(expression: &ValueExpr, expected: EventValueProperty) -> bool {
    match expression {
        ValueExpr::ReadEventProperty(property) => *property == expected,
        ValueExpr::Negate(value) => expression_reads(value, expected),
        ValueExpr::Add(lhs, rhs)
        | ValueExpr::Subtract(lhs, rhs)
        | ValueExpr::Minimum(lhs, rhs)
        | ValueExpr::Maximum(lhs, rhs) => {
            expression_reads(lhs, expected) || expression_reads(rhs, expected)
        }
        ValueExpr::Multiply { lhs, rhs, .. } | ValueExpr::Divide { lhs, rhs, .. } => {
            expression_reads(lhs, expected) || expression_reads(rhs, expected)
        }
        _ => false,
    }
}
