use starclock_activity::{ActivityCause, ActivityTransactionOutcome, ActivityTransactionState};

use crate::error::UniverseCatalogLoadErrorKind;

use super::ServiceRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

#[test]
fn exact_sora_rule_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let _instance = instance(&factory);

    let mut input = factory
        .content
        .mechanic_rule_runtime_input("service-and-adventure")
        .unwrap();
    input.triggers.reverse();
    assert_eq!(
        ServiceRuleRuntimeCatalog::compile(input)
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

#[test]
fn fixture_service_beacon_and_external_adventure_bind_to_production_catalogs() {
    let instance = instance(&factory());
    assert_eq!(instance.service_count(), 15);
    assert_eq!(instance.adventure_count(), 6);
    assert_eq!(
        instance
            .beacon_service_contribution("swarm-disaster.beacon.1")
            .unwrap(),
        (14, "TopologyMutationResolution")
    );
    assert!(
        instance
            .compile_adventure_settlement(
                "swarm-disaster.adventure-outcome.1210601",
                "Tier1",
                0,
                None,
                None,
            )
            .is_ok()
    );
}

#[test]
fn accepted_service_purchase_is_atomic_and_stale_reapplication_rejects() {
    let instance = instance(&factory());
    let purchase = instance
        .compile_service_purchase(
            "swarm-disaster.service.universe-service-reset-blessing-choice",
            30,
            0,
        )
        .unwrap();
    let mut state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let cause = ActivityCause::new(1, purchase.id(), state.current_node()).unwrap();
    assert!(matches!(
        state.apply_program(&purchase, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
    let sequence = state.command_sequence();
    let stale = ActivityCause::new(sequence + 1, purchase.id(), state.current_node()).unwrap();
    assert!(matches!(
        state.apply_program(&purchase, stale, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::super::tests::released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::super::tests::participants(super::super::tests::policy()),
        ))
        .unwrap()
}
