use super::*;

const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const REPRESENTATIVE_BUNDLE: &[u8] =
    include_bytes!("../../../config/catalog-fixtures/representative/config.sora");

#[test]
fn production_bundle_builds_standard_v1_and_representative_characters() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    assert_eq!(catalog.manifest().game_version, "4.4");
    assert_eq!(
        catalog.manifest().coverage_manifest_sha256,
        "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19"
    );
    assert_eq!(
        catalog.summary(),
        CatalogSummary {
            identity_count: 1118,
            enabled_identity_count: 879,
            ability_count: 165,
            hit_plan_count: 108,
            character_count: 14,
            effect_count: 3,
            ai_graph_count: 17,
            enemy_count: 17,
            encounter_count: 6,
            standard_profile_count: 1,
            standard_scenario_count: 6,
        }
    );
    for (id, abilities, resources, parameters, traces) in [
        (2, 7, 1, 287, 20),
        (8, 6, 1, 101, 18),
        (18, 6, 1, 162, 18),
        (27, 8, 1, 318, 21),
        (45, 6, 1, 178, 20),
        (68, 12, 2, 361, 19),
    ] {
        let character = catalog
            .character(starclock_combat::UnitDefinitionId::new(id).unwrap())
            .expect("frozen representative character");
        assert_eq!(character.stat_row_count(), 86);
        assert_eq!(character.ability_count(), abilities);
        assert_eq!(character.resource_count(), resources);
        assert_eq!(character.ability_parameter_count(), parameters);
        assert_eq!(character.trace_count(), traces);
        assert_eq!(character.eidolon_count(), 6);
    }
    assert_eq!(
        catalog
            .enemy(starclock_combat::EnemyDefinitionId::new(98).unwrap())
            .expect("frozen Cocolia variant")
            .phases()
            .len(),
        2
    );
    assert_eq!(
        catalog
            .enemy(starclock_combat::EnemyDefinitionId::new(105).unwrap())
            .expect("frozen Great Septimus variant")
            .phases()
            .len(),
        3
    );
    assert_eq!(
        catalog
            .enemy(starclock_combat::EnemyDefinitionId::new(96).unwrap())
            .unwrap()
            .links()
            .len(),
        1
    );
    for (id, waves) in [(89, 1), (90, 1), (91, 1), (92, 3), (93, 1), (94, 3)] {
        assert_eq!(
            catalog
                .encounter(starclock_combat::EncounterId::new(id).unwrap())
                .expect("frozen encounter")
                .waves()
                .len(),
            waves
        );
    }
    for id in 278..=283 {
        assert!(
            catalog
                .standard_scenario(starclock_mode_standard::StandardScenarioId::new(id).unwrap())
                .is_some()
        );
    }
    let combat = catalog.combat_catalog();
    let linked = combat
        .linked_unit(starclock_combat::UnitDefinitionId::new(24_505).unwrap())
        .expect("Aglaea linked-unit definition");
    assert_eq!(linked.abilities()[0].get(), 24_610);
    let countdown = combat
        .countdown(24_304)
        .expect("Firefly countdown definition");
    assert_eq!(countdown.definition().ability().get(), 24_611);
    assert!(countdown.definition().ends_transformation());
    let asta = combat
        .effect(starclock_combat::EffectDefinitionId::new(24_006).unwrap())
        .expect("Asta SPD effect");
    assert_eq!(
        asta.modifiers(),
        [starclock_combat::ModifierDefinitionId::new(24_004).unwrap()]
    );
    let clara = combat
        .unit(starclock_combat::UnitDefinitionId::new(18).unwrap())
        .expect("Clara unit definition");
    assert_eq!(
        clara.resources()[0].stable_key(),
        "enhanced-counter-charges"
    );
    let base = starclock_combat::AbilityId::new(20_006).unwrap();
    let maximum = starclock_combat::AbilityId::new(1_000_640_202).unwrap();
    assert_eq!(
        combat.ability_parameter(base, "parameter.01"),
        Some(&starclock_combat::rule::model::RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(1_000_000)
        ))
    );
    assert_eq!(
        combat.ability_parameter(maximum, "parameter.01"),
        Some(&starclock_combat::rule::model::RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(2_800_000)
        ))
    );
    let formula = starclock_combat::rule::model::ValueExpr::Multiply {
        lhs: Box::new(starclock_combat::rule::model::ValueExpr::AbilityParameter {
            key: "parameter.01".into(),
            kind: starclock_combat::rule::model::RuleValueKind::Scalar,
        }),
        rhs: Box::new(starclock_combat::rule::model::ValueExpr::Literal(
            starclock_combat::rule::model::RuleValue::Scalar(
                starclock_combat::Scalar::from_scaled(2_000_000),
            ),
        )),
        rounding: starclock_combat::Rounding::NearestTiesEven,
    };
    let mut input = starclock_combat::rule::model::RuleEvaluationInput {
        event_kind: starclock_combat::rule::model::RuleEventKind::Action,
        event_facts: &starclock_combat::rule::model::RuleEventFacts {
            point: Some(starclock_combat::rule::model::RuleEventPoint::ActionResolved),
            ..starclock_combat::rule::model::RuleEventFacts::default()
        },
        cause: starclock_combat::rule::model::RuleCause {
            owner: None,
            actor: None,
            applier: None,
            target: None,
            source: None,
        },
        occurrence: starclock_combat::rule::model::RuleOccurrence {
            rule_instance: starclock_combat::RuleInstanceId::new(1).unwrap(),
            event: starclock_combat::EventId::new(1).unwrap(),
            hit: None,
            target: None,
            ability: Some(base),
            action: starclock_combat::ActionId::new(1),
            turn_event: None,
            wave: starclock_combat::WaveInstanceId::new(1).unwrap(),
        },
        source_tags: &[],
        slots: &[],
        selectors: &[],
        stat_reader: None,
        ability_parameter_reader: Some(combat),
        resource_reader: None,
        battle_query_reader: None,
    };
    assert_eq!(
        starclock_combat::rule::evaluate::evaluate_value(&formula, input, None).unwrap(),
        starclock_combat::rule::model::RuleValue::Scalar(starclock_combat::Scalar::from_scaled(
            2_000_000
        ))
    );
    input.occurrence.ability = Some(maximum);
    assert_eq!(
        starclock_combat::rule::evaluate::evaluate_value(&formula, input, None).unwrap(),
        starclock_combat::rule::model::RuleValue::Scalar(starclock_combat::Scalar::from_scaled(
            5_600_000
        ))
    );
    assert!(Arc::ptr_eq(&catalog, &Arc::clone(&catalog)));
}

#[test]
fn production_asta_basic_executes_minimum_and_maximum_hit_formulas() {
    use starclock_combat::{
        catalog::action::HitOperationDefinition,
        formula::model::{CombatElement, DamageClass},
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();
    for (raw, coefficient, damage) in [(20_013, 500_000, 1_000), (1_000_640_426, 1_400_000, 2_800)]
    {
        let ability = combat
            .ability(starclock_combat::AbilityId::new(raw).unwrap())
            .expect("Asta Basic effective-level variant");
        let hit = &ability.action().expect("compiled action").hits()[0];
        assert_eq!(hit.operations().len(), 2);
        let HitOperationDefinition::ScalingDamage(definition) = hit.operations()[0] else {
            panic!("first operation must be live-stat damage");
        };
        assert_eq!(definition.scaling_stat(), StatKind::Atk);
        assert_eq!(definition.coefficient().scaled(), coefficient);
        assert_eq!(definition.class(), DamageClass::Direct);
        assert_eq!(definition.element(), CombatElement::Fire);
        let formula = definition
            .resolve(starclock_combat::Scalar::checked_from_integer(2_000).unwrap())
            .unwrap();
        assert_eq!(
            formula
                .base_damage()
                .rounded_integer(starclock_combat::Rounding::Floor)
                .unwrap(),
            damage
        );

        let HitOperationDefinition::ReduceToughness(definition) = hit.operations()[1] else {
            panic!("second operation must reduce Toughness");
        };
        assert_eq!(definition.element, CombatElement::Fire);
        assert_eq!(definition.reduction.base.get(), 30);
        assert_eq!(
            starclock_combat::formula::toughness::reduction(definition.reduction)
                .unwrap()
                .attempted
                .get(),
            30
        );
    }
}

#[test]
fn production_bounce_and_blast_hits_use_exact_payload_overrides() {
    use starclock_combat::catalog::action::HitOperationDefinition;

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();
    for (raw, coefficient) in [(20_011, 250_000), (1_000_640_367, 625_000)] {
        let ability = combat
            .ability(starclock_combat::AbilityId::new(raw).unwrap())
            .expect("Asta Skill effective-level variant");
        let hits = ability.action().unwrap().hits();
        assert_eq!(hits.len(), 5);
        for hit in hits {
            let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
                panic!("bounce hit must own scaling damage");
            };
            assert_eq!(damage.coefficient().scaled(), coefficient);
            let HitOperationDefinition::ReduceToughness(toughness) = hit.operations()[1] else {
                panic!("bounce hit must own Toughness reduction");
            };
            assert_eq!(toughness.reduction.base.get(), 30);
        }
    }

    for (raw, expected) in [
        (20_029, [(800_000, 60), (300_000, 30)]),
        (1_000_640_943, [(2_000_000, 60), (750_000, 30)]),
    ] {
        let ability = combat
            .ability(starclock_combat::AbilityId::new(raw).unwrap())
            .expect("Kafka Skill effective-level variant");
        for (hit, (coefficient, toughness)) in ability.action().unwrap().hits().iter().zip(expected)
        {
            let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
                panic!("Blast hit must own scaling damage");
            };
            assert_eq!(damage.coefficient().scaled(), coefficient);
            let HitOperationDefinition::ReduceToughness(reduction) = hit.operations()[1] else {
                panic!("Blast hit must own Toughness reduction");
            };
            assert_eq!(reduction.reduction.base.get(), toughness);
        }
    }
}

#[test]
fn production_silver_wolf_executes_released_elation_envelopes() {
    use starclock_combat::{
        catalog::action::{
            AbilityProgramTiming, AbilityTag, HitOperationDefinition, HitTargetGroup,
        },
        formula::model::DamageClass,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();
    let enhanced = combat
        .ability(starclock_combat::AbilityId::new(20_036).unwrap())
        .expect("Silver Wolf enhanced Basic");
    let action = enhanced.action().expect("enhanced Basic action");
    assert_eq!(action.hit_count(), 101);
    assert_eq!(action.resources().skill_point_gain(), 0);
    assert_eq!(
        combat
            .ability(starclock_combat::AbilityId::new(20_042).unwrap())
            .unwrap()
            .action()
            .unwrap()
            .resources()
            .skill_point_gain(),
        1
    );
    assert_eq!(
        combat
            .ability(starclock_combat::AbilityId::new(20_045).unwrap())
            .unwrap()
            .action()
            .unwrap()
            .resources()
            .skill_point_cost(),
        1
    );
    assert_eq!(action.hits()[0].target_group(), HitTargetGroup::Primary);
    assert!(
        action.hits()[1..100]
            .iter()
            .all(|hit| hit.target_group() == HitTargetGroup::BounceDraw)
    );
    assert_eq!(action.hits()[100].target_group(), HitTargetGroup::All);

    let mut toughness = 0;
    for hit in &action.hits()[..100] {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("bounce must execute scaling damage");
        };
        assert_eq!(damage.coefficient().scaled(), 12_000);
        for operation in hit.operations() {
            if let HitOperationDefinition::ReduceToughness(reduction) = operation {
                toughness += reduction.reduction.base.get();
            }
        }
    }
    let final_hit = action.hits().last().unwrap();
    let HitOperationDefinition::ScalingDamage(damage) = final_hit.operations()[0] else {
        panic!("final hit must execute scaling damage");
    };
    assert_eq!(damage.coefficient().scaled(), 500_000);
    let HitOperationDefinition::ReduceToughness(reduction) = final_hit.operations()[1] else {
        panic!("final hit must execute Toughness reduction");
    };
    toughness += reduction.reduction.base.get();
    assert_eq!(toughness, 60);

    let elation_skill = combat
        .ability(starclock_combat::AbilityId::new(20_039).unwrap())
        .expect("Silver Wolf enhanced Elation Skill");
    assert!(
        elation_skill
            .action()
            .unwrap()
            .tags()
            .contains(AbilityTag::ElationSkill)
    );
    assert_eq!(elation_skill.action().unwrap().hit_count(), 6);
    for hit in elation_skill.action().unwrap().hits() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Elation bounce must execute scaling damage");
        };
        assert_eq!(damage.coefficient().scaled(), 450_000);
        assert_eq!(damage.class(), DamageClass::Elation);
    }

    let pro_gamer = combat
        .ability(starclock_combat::AbilityId::new(20_043).unwrap())
        .expect("Silver Wolf Elation Skill");
    assert_eq!(pro_gamer.programs().len(), 1);
    assert_eq!(
        pro_gamer.programs()[0].timing(),
        AbilityProgramTiming::Resolved
    );
    assert_eq!(pro_gamer.programs()[0].program().get(), 24_623);

    let silver = combat
        .unit(starclock_combat::UnitDefinitionId::new(68).unwrap())
        .expect("Silver Wolf form");
    let mmr = silver
        .resources()
        .iter()
        .find(|resource| resource.stable_key() == "hidden-mmr")
        .expect("Hidden MMR resource");
    assert_eq!(mmr.maximum().scaled(), 300_000_000);
}

#[test]
fn production_c01_executes_exact_bounce_and_hp_skill_envelopes() {
    use starclock_combat::{
        catalog::action::{HitOperationDefinition, HitTargetGroup},
        formula::model::CombatElement,
        modifier::model::StatKind,
    };

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    let combat = catalog.combat_catalog();

    let anaxa = combat
        .ability(starclock_combat::AbilityId::new(30_010).unwrap())
        .expect("Anaxa Skill");
    let hits = anaxa.action().expect("Anaxa damage action").hits();
    assert_eq!(hits.len(), 5);
    assert_eq!(hits[0].target_group(), HitTargetGroup::Primary);
    assert!(
        hits[1..]
            .iter()
            .all(|hit| hit.target_group() == HitTargetGroup::BounceDraw)
    );
    for (index, hit) in hits.iter().enumerate() {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Anaxa hit must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Atk);
        assert_eq!(damage.element(), CombatElement::Wind);
        assert_eq!(
            damage.coefficient().scaled(),
            if index == 0 { 350_000 } else { 200_000 }
        );
        let HitOperationDefinition::ReduceToughness(toughness) = hit.operations()[1] else {
            panic!("Anaxa hit must execute Toughness reduction");
        };
        assert_eq!(toughness.reduction.base.get(), 30);
    }

    let aventurine = combat
        .ability(starclock_combat::AbilityId::new(30_044).unwrap())
        .expect("Aventurine follow-up");
    let hits = aventurine
        .action()
        .expect("Aventurine damage action")
        .hits();
    assert_eq!(hits.len(), 7);
    for hit in hits {
        let HitOperationDefinition::ScalingDamage(damage) = hit.operations()[0] else {
            panic!("Aventurine hit must execute scaling damage");
        };
        assert_eq!(damage.scaling_stat(), StatKind::Def);
        assert_eq!(damage.element(), CombatElement::Imaginary);
        assert_eq!(damage.coefficient().scaled(), 125_000);
    }

    let arlan_skill = combat
        .ability(starclock_combat::AbilityId::new(30_033).unwrap())
        .expect("Arlan HP-funded Skill");
    assert_eq!(
        arlan_skill.action().unwrap().resources().skill_point_cost(),
        0
    );
    assert_eq!(
        combat
            .ability(starclock_combat::AbilityId::new(30_031).unwrap())
            .unwrap()
            .action()
            .unwrap()
            .resources()
            .skill_point_gain(),
        1
    );
}

#[test]
fn production_characters_compile_at_e0_and_complete_e6() {
    use starclock_build::{
        ability::AbilityInvestment,
        compiler::LoadoutCompiler,
        patch::BuildPatch,
        spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    };
    use starclock_combat::UnitLevel;

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    for raw in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 18, 27, 45, 68] {
        let form = starclock_combat::UnitDefinitionId::new(raw).unwrap();
        let character = catalog
            .build_catalog()
            .character(form)
            .expect("production character build definition");
        let investments = character
            .ability_levels()
            .iter()
            .map(|table| AbilityInvestment::new(table.family(), table.invested_cap()))
            .collect::<Vec<_>>();
        let base = CombatantBuildSpec::new(
            form,
            UnitLevel::new(80).unwrap(),
            PromotionStage::new(6).unwrap(),
        )
        .with_ability_levels(investments)
        .unwrap();
        let e0 = LoadoutCompiler
            .compile(catalog.build_catalog(), catalog.combat_catalog(), &base)
            .expect("production E0 build");
        assert!(e0.combatant().modifiers().is_empty());

        let graph = character.trace_graph().expect("complete Trace graph");
        let trace_modifiers = graph
            .nodes()
            .iter()
            .flat_map(|node| node.patches())
            .filter(|patch| matches!(patch, BuildPatch::AddModifier(_)))
            .count();
        assert!(trace_modifiers > 0);
        let e6_spec = base
            .with_traces(graph.canonical_order().to_vec())
            .unwrap()
            .with_eidolon(EidolonLevel::new(6).unwrap());
        let e6 = LoadoutCompiler
            .compile(catalog.build_catalog(), catalog.combat_catalog(), &e6_spec)
            .expect("production E6 build");
        assert_eq!(e6.combatant().modifiers().len(), trace_modifiers);
    }
}

#[test]
fn real_fixture_bundle_builds_representative_private_definitions() {
    let catalog = load_with_mode(REPRESENTATIVE_BUNDLE, LoadMode::Fixture)
        .expect("representative catalog must load");
    assert_eq!(
        catalog.manifest().data_revision,
        "catalog-representative-v1"
    );
    assert_eq!(
        catalog.summary(),
        CatalogSummary {
            identity_count: 3,
            enabled_identity_count: 0,
            ability_count: 1,
            hit_plan_count: 1,
            character_count: 1,
            effect_count: 0,
            ai_graph_count: 0,
            enemy_count: 0,
            encounter_count: 0,
            standard_profile_count: 0,
            standard_scenario_count: 0,
        }
    );
    let ability = &catalog.combat.abilities[0];
    assert_eq!(ability.id.get(), 2);
    assert_eq!(ability.level_cap, 6);
    assert_eq!(ability.phases[0].sequence, 1);
    let hit = &catalog.combat.hit_plans[0].hits[0];
    assert_eq!(hit.damage_ratio.scaled(), 1_000_000);
    assert_eq!(hit.toughness_ratio.scaled(), 1_000_000);
    let character = &catalog.builds.characters[0];
    assert_eq!(character.id.get(), 1);
    assert_eq!(character.base_energy.scaled(), 120_000_000);
    assert_eq!(character.base_aggro.scaled(), 100_000_000);
    assert_eq!(character.stats.len(), 2);
    assert_eq!(character.abilities[0].ability.get(), 2);
}

#[test]
fn production_loader_rejects_fixture_labels() {
    let error = load(REPRESENTATIVE_BUNDLE).expect_err("fixture cannot enter production");
    assert_eq!(error.kind(), CatalogLoadErrorKind::Metadata);
    assert!(error.to_string().contains("synthetic") || error.to_string().contains("fixture"));
}

#[test]
fn canonical_decimal_parser_has_no_floating_point_path() {
    for (source, expected) in [
        ("0", 0),
        ("1", 1_000_000),
        ("0.000001", 1),
        ("-12.345678", -12_345_678),
        ("9223372036854.775807", i64::MAX),
        ("-9223372036854.775808", i64::MIN),
    ] {
        assert_eq!(parse_decimal(source), Ok(expected));
    }
    for invalid in ["", "+1", "01", "-0", "1.", "1.0", "1e2", "0.0000001"] {
        assert!(parse_decimal(invalid).is_err(), "accepted {invalid}");
    }
}
