use starclock_activity::{
    ActivityCause, ActivityProgramDefinition, ActivityProgramId, ActivityTransactionOutcome,
    ActivityTransactionState,
};

use crate::{
    error::UniverseCatalogLoadErrorKind,
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::PathRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

const FAMILIES: [&str; 2] = ["path-and-propagation-unlock", "resonance-interplay"];

#[test]
fn exact_sora_partition_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    assert_eq!(
        hex(instance.path_rule_runtime_digest()),
        "a421ce1b0868170f00273ee9e72f021399732b9b154d46126aad5a50821d4cc6"
    );
    for family in FAMILIES {
        let mut inputs = inputs(&factory);
        inputs
            .iter_mut()
            .find(|input| input.family.as_ref() == family)
            .unwrap()
            .source_disposition = "Production".into();
        assert_eq!(
            PathRuleRuntimeCatalog::compile(inputs).unwrap_err().kind(),
            UniverseCatalogLoadErrorKind::InvalidReference
        );
    }
}

#[test]
fn propagation_selection_requires_the_exact_released_unlock_binding() {
    let propagation = instance(
        &factory(),
        "universe.path.propagation",
        "swarm-disaster.audience-die.8",
    );
    assert!(propagation.path_is_propagation());
    assert_eq!(
        propagation.path_progression_unlock_id(),
        Some("swarm-disaster.pathstrider-unlock.1000008")
    );
    assert_eq!(propagation.path_resonance_bindings().count(), 4);
    assert_eq!(
        propagation.path_boost_binding(),
        ("swarm-disaster.path-boost.641270", "StageAbility_641270")
    );
}

#[test]
fn interplays_activate_in_stable_order_once_and_reject_stale_programs() {
    let instance = instance(
        &factory(),
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
    );
    let mut state = state(&instance);
    let counts = vec![
        ("universe.path.preservation".to_owned(), 3),
        ("universe.path.nihility".to_owned(), 3),
        ("universe.path.remembrance".to_owned(), 3),
    ];
    let program = instance
        .compile_resonance_interplays(&state, &counts)
        .unwrap()
        .unwrap();
    let stale = program.clone();
    commit(&instance, &mut state, program);
    assert_eq!(
        instance.active_resonance_interplays(&state).unwrap().len(),
        2
    );
    assert!(
        instance
            .compile_resonance_interplays(&state, &counts)
            .unwrap()
            .is_none()
    );
    let sequence = state.command_sequence();
    assert!(matches!(
        state.apply_program(
            &stale,
            cause(&state, stale.id()),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state.command_sequence(), sequence);
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    path: &str,
    die: &str,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::super::tests::released_entry(
            "swarm-disaster.area.201",
            path,
            die,
            super::super::tests::participants(super::super::tests::policy()),
        ))
        .unwrap()
}

fn inputs(factory: &SwarmDisasterRuntimeFactory) -> [MechanicRuleRuntimeInput; 2] {
    FAMILIES.map(|family| factory.content.mechanic_rule_runtime_input(family).unwrap())
}

fn state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn commit(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: ActivityProgramDefinition,
) {
    assert!(matches!(
        state.apply_program(
            &program,
            cause(state, program.id()),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(state: &ActivityTransactionState, program: ActivityProgramId) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
