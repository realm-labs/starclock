use starclock_activity::{
    ActivityCause, ActivityProgramDefinition, ActivityProgramId, ActivityTransactionOutcome,
    ActivityTransactionState,
};

use crate::error::UniverseCatalogLoadErrorKind;

use super::CurioRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

#[test]
fn exact_sora_rule_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let _instance = instance(&factory);

    let mut input = factory
        .content
        .mechanic_rule_runtime_input("curio-lifecycle")
        .unwrap();
    input.domain = "Activity".into();
    assert_eq!(
        CurioRuleRuntimeCatalog::compile(input).unwrap_err().kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

#[test]
fn charged_curio_transitions_once_and_rejects_stale_use() {
    let instance = instance(&factory());
    let mut state = state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(1001).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_charge_use(1001, 2).unwrap(),
    );
    let terminal = instance.compile_curio_charge_use(1001, 1).unwrap();
    let stale = terminal.clone();
    commit(&instance, &mut state, terminal);
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

#[test]
fn repair_replacement_and_teardown_share_the_activity_transaction_owner() {
    let instance = instance(&factory());
    let mut state = state(&instance);
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(1045).unwrap(),
    );
    for progress in 0..3 {
        let program = instance
            .compile_curio_repair_progress(1045, progress)
            .unwrap();
        commit(&instance, &mut state, program);
    }
    commit(
        &instance,
        &mut state,
        instance.compile_curio_acquisition(1001).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_replacement(1001, 1101).unwrap(),
    );
    commit(
        &instance,
        &mut state,
        instance.compile_curio_teardown(1101).unwrap(),
    );
    assert!(instance.compile_curio_replacement(1045, 1045).is_err());
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
    program
        .validate_against(instance.state_definition(), instance.graph_definition())
        .unwrap();
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
