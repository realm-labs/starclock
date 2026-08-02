use std::collections::BTreeSet;

use super::{ExecutionKind, fixture_bindings};
use crate::{
    swarm_disaster_content::types::ReviewFixtureQuality,
    swarm_disaster_entry::SwarmDisasterRuntimeFactory,
};

#[test]
fn all_23_semantic_fixture_families_bind_exactly_once() {
    let factory = factory();
    let (bindings, _, _) = fixture_bindings(&factory.content).unwrap();
    assert_eq!(bindings.len(), 23);
    assert!(bindings.windows(2).all(|pair| {
        pair[0].family_id < pair[1].family_id && pair[0].fixture_id < pair[1].fixture_id
    }));
    assert!(bindings.iter().all(|binding| {
        binding.fixture_id.as_ref() == format!("swarm-disaster.fixture.{}", binding.family_id)
            && !binding.production_regression.is_empty()
    }));
}

#[test]
fn fixture_payload_denominators_and_evidence_labels_are_exact() {
    let factory = factory();
    let (bindings, _, _) = fixture_bindings(&factory.content).unwrap();
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.ordered_operation_count)
            .sum::<usize>(),
        85
    );
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.expected_fact_count)
            .sum::<usize>(),
        108
    );
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.source_record_count)
            .sum::<usize>(),
        76
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.quality == ReviewFixtureQuality::ExactStructured)
            .count(),
        3
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.quality == ReviewFixtureQuality::ProjectPolicy)
            .count(),
        20
    );
}

#[test]
fn all_production_regressions_are_unique_and_runtime_backed() {
    let factory = factory();
    let (bindings, _, _) = fixture_bindings(&factory.content).unwrap();
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.production_regression.as_ref())
            .collect::<BTreeSet<_>>()
            .len(),
        23
    );
    assert_eq!(
        bindings
            .iter()
            .filter(|binding| binding.execution_kind == ExecutionKind::ProductionRuntime)
            .count(),
        22
    );
}

#[test]
fn encounter_fixture_is_catalog_bound_without_claiming_phase6_selection() {
    let factory = factory();
    let (bindings, _, encounter_shape) = fixture_bindings(&factory.content).unwrap();
    assert_eq!(encounter_shape, (1, 1, 3, 1));
    let probes = bindings
        .iter()
        .filter(|binding| binding.execution_kind == ExecutionKind::ProductionCatalogProbe)
        .collect::<Vec<_>>();
    assert_eq!(probes.len(), 1);
    assert_eq!(probes[0].family_id.as_ref(), "encounter-selection");
    assert_eq!(
        probes[0].production_regression.as_ref(),
        "all_formal_difficulties_share_one_immutable_encounter_contract"
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}
