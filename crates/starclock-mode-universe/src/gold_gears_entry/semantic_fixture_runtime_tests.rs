use std::{collections::BTreeSet, sync::OnceLock};

use super::{
    GOLD_AND_GEARS_SEMANTIC_FIXTURE_EXECUTION_REVISION, GoldAndGearsRuntimeFactory,
    GoldAndGearsSemanticFixtureExecutionKind,
};

const BUNDLE: &[u8] = include_bytes!("../../../../config/gold-and-gears-generated/config.sora");

#[test]
fn all_18_semantic_fixture_families_bind_exactly_once() {
    let factory = factory();
    let bindings = factory.semantic_fixture_bindings();
    assert_eq!(bindings.len(), 18);
    assert!(bindings.windows(2).all(|pair| {
        pair[0].family_id() < pair[1].family_id() && pair[0].fixture_id() < pair[1].fixture_id()
    }));
    assert!(bindings.iter().all(|binding| {
        binding.fixture_id() == format!("gold-gears.fixture.{}", binding.family_id())
            && !binding.production_regression().is_empty()
    }));
    assert_eq!(
        GOLD_AND_GEARS_SEMANTIC_FIXTURE_EXECUTION_REVISION,
        "gold-and-gears-semantic-fixture-execution-v1"
    );
    assert_eq!(
        hex(factory.semantic_fixture_execution_digest()),
        "2b69ec29dde6fde1dc6cac9ea10baea5d34c28f39d3d03a41f74d5f340b52832"
    );
}

#[test]
fn all_nine_mechanic_fixture_families_close_the_1224_rule_denominator() {
    let factory = factory();
    let denominators = [
        ("profile-entry", 5),
        ("conundrum-stats", 6),
        ("conundrum-auxiliary", 6),
        ("neural-network-effect", factory.unique.neural_nodes.len()),
        ("curio-lifecycle", factory.curio_rule_bindings().len()),
        (
            "occurrence-choice",
            factory.occurrence_rule_bindings().len(),
        ),
        (
            "service-and-adventure",
            factory.service_adventure_rule_bindings().len(),
        ),
        ("path-boost", factory.path_boost_rule_bindings().len()),
        (
            "resonance-extrapolation",
            factory.resonance_rule_bindings().len(),
        ),
    ];
    assert_eq!(
        denominators,
        [
            ("profile-entry", 5),
            ("conundrum-stats", 6),
            ("conundrum-auxiliary", 6),
            ("neural-network-effect", 40),
            ("curio-lifecycle", 160),
            ("occurrence-choice", 384),
            ("service-and-adventure", 38),
            ("path-boost", 495),
            ("resonance-extrapolation", 90),
        ]
    );
    assert_eq!(
        denominators.iter().map(|(_, count)| *count).sum::<usize>(),
        1_224
    );
}

#[test]
fn encounter_selection_fixture_is_catalog_bound_pending_p6_execution() {
    let factory = factory();
    assert_eq!(factory.encounter_selection_fixture_shape(), (2, 2));
    let catalog_probes = factory
        .semantic_fixture_bindings()
        .iter()
        .filter(|binding| {
            binding.execution_kind()
                == GoldAndGearsSemanticFixtureExecutionKind::ProductionCatalogProbe
        })
        .collect::<Vec<_>>();
    assert_eq!(catalog_probes.len(), 1);
    assert_eq!(catalog_probes[0].family_id(), "encounter-selection");
    assert_eq!(
        catalog_probes[0].production_regression(),
        "encounter_selection_fixture_is_catalog_bound_pending_p6_execution"
    );
}

#[test]
fn all_fixture_regressions_are_unique_and_production_runtime_backed() {
    let bindings = factory().semantic_fixture_bindings();
    let regressions = bindings
        .iter()
        .map(|binding| binding.production_regression())
        .collect::<BTreeSet<_>>();
    assert_eq!(regressions.len(), 18);
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| {
                binding.execution_kind()
                    == GoldAndGearsSemanticFixtureExecutionKind::ProductionRuntime
            })
            .count(),
        17
    );
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    static FACTORY: OnceLock<GoldAndGearsRuntimeFactory> = OnceLock::new();
    FACTORY.get_or_init(|| GoldAndGearsRuntimeFactory::load_candidate(BUNDLE).unwrap())
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
