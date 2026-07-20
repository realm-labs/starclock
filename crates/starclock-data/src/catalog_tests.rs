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
            identity_count: 754,
            enabled_identity_count: 507,
            ability_count: 113,
            hit_plan_count: 72,
            character_count: 6,
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
fn production_representatives_compile_at_e0_and_complete_e6() {
    use starclock_build::{
        ability::AbilityInvestment,
        compiler::LoadoutCompiler,
        patch::BuildPatch,
        spec::{CombatantBuildSpec, EidolonLevel, PromotionStage},
    };
    use starclock_combat::UnitLevel;

    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    for raw in [2, 8, 18, 27, 45, 68] {
        let form = starclock_combat::UnitDefinitionId::new(raw).unwrap();
        let character = catalog
            .build_catalog()
            .character(form)
            .expect("representative build definition");
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
            .expect("representative E0 build");
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
            .expect("representative E6 build");
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
