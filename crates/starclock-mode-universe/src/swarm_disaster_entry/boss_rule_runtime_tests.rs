use starclock_activity::ActivityTransactionState;

use crate::error::UniverseCatalogLoadErrorKind;

use super::BossRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

#[test]
fn exact_sora_partition_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let _instance = instance(&factory);

    let mut inputs = inputs(&factory);
    inputs[0].domain = "Activity".into();
    assert_eq!(
        BossRuleRuntimeCatalog::compile(inputs).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

#[test]
fn released_boss_choices_remain_stable_and_fixture_choice_compiles() {
    let instance = instance(&factory());
    assert_eq!(
        instance.boss_choices().collect::<Vec<_>>(),
        [
            "swarm-disaster.boss-choice.8003051",
            "swarm-disaster.boss-choice.8024010",
        ]
    );
    assert!(
        instance
            .compile_boss_selection(1, "swarm-disaster.boss-choice.8003051")
            .is_ok()
    );
    assert!(instance.compile_boss_selection(0, "missing").is_err());
}

#[test]
fn final_boss_inputs_reuse_decay_selection_and_explicit_choice_programs() {
    let instance = instance(&factory());
    let state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    );
    let decay = instance
        .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.1"])
        .unwrap();
    assert!(!decay.operations().is_empty());
    let final_choice = instance
        .compile_boss_selection(3, "swarm-disaster.boss-choice.8024010")
        .unwrap();
    assert_eq!(final_choice.operations().len(), 2);
    assert!(final_choice.operations().iter().all(|operation| matches!(
        operation,
        starclock_activity::ActivityOperation::AddCounter { .. }
    )));
}

fn inputs(
    factory: &SwarmDisasterRuntimeFactory,
) -> [crate::swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput; 2] {
    [
        factory
            .content
            .mechanic_rule_runtime_input("boss-choice-consequence")
            .unwrap(),
        factory
            .content
            .mechanic_rule_runtime_input("final-boss-consequence")
            .unwrap(),
    ]
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
