use super::*;

const PRODUCTION_BUNDLE: &[u8] = include_bytes!("../../../config/generated/config.sora");
const REPRESENTATIVE_BUNDLE: &[u8] =
    include_bytes!("../../../config/catalog-fixtures/representative/config.sora");

#[test]
fn production_bundle_builds_the_frozen_standard_v1_catalog() {
    let catalog = load(PRODUCTION_BUNDLE).expect("production catalog must load");
    assert_eq!(catalog.manifest().game_version, "4.4");
    assert_eq!(
        catalog.manifest().coverage_manifest_sha256,
        "e2188c7844d678253c98d569db017dbad7101541cf502aba4c2eb80c0435bf19"
    );
    assert_eq!(
        catalog.summary(),
        CatalogSummary {
            identity_count: 424,
            enabled_identity_count: 171,
            ability_count: 63,
            hit_plan_count: 42,
            character_count: 0,
            effect_count: 0,
            ai_graph_count: 17,
            enemy_count: 17,
            encounter_count: 6,
            standard_profile_count: 1,
            standard_scenario_count: 6,
        }
    );
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
    assert!(Arc::ptr_eq(&catalog, &Arc::clone(&catalog)));
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
