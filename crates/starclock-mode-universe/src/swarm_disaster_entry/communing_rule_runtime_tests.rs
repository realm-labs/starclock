use crate::swarm_disaster_entry::tests::{BUNDLE, participants, policy};
use starclock_activity::{
    ActivityCause, ActivityProgramDefinition, ActivityProgramId, ActivityTransactionOutcome,
    ActivityTransactionState,
};

use crate::{
    error::UniverseCatalogLoadErrorKind,
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::CommuningRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
};

const FAMILIES: [&str; 2] = ["communing-choice", "communing-dimension-points"];
const CHOICE: &str = "swarm-disaster.communing-choice.441";
const PATH: &str = "universe.path.preservation";
const ROOT: &str = "swarm-disaster.pathstrider-cabinet.22";
const ROOT_OBJECTIVE: &str = "6013222";
const DIMENSION_SIX: &str = "swarm-disaster.communing-dimension.6";
const DIMENSION_SEVEN: &str = "swarm-disaster.communing-dimension.7";

#[test]
fn exact_sora_partition_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let _instance = instance(&factory, vec![]);
    for family in FAMILIES {
        let mut inputs = inputs(&factory);
        inputs
            .iter_mut()
            .find(|input| input.family.as_ref() == family)
            .unwrap()
            .domain = "CrossBattle".into();
        assert_eq!(
            CommuningRuleRuntimeCatalog::compile(inputs)
                .unwrap_err()
                .kind(),
            UniverseCatalogLoadErrorKind::InvalidReference
        );
    }
}

#[test]
fn choice_execution_updates_one_aeon_once_and_rejects_stale_program() {
    let instance = instance(&factory(), vec![]);
    let mut state = state(&instance);
    let program = instance
        .compile_communing_choice(&state, 4, CHOICE)
        .unwrap();
    let stale = program.clone();
    commit(&instance, &mut state, program);

    assert_eq!(instance.communing_choice_count(&state, PATH).unwrap(), 1);
    assert!(
        !instance
            .communing_choice_available(&state, 4, CHOICE)
            .unwrap()
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
    assert_eq!(instance.communing_choice_count(&state, PATH).unwrap(), 1);
}

#[test]
fn dimension_adjustments_clamp_in_order_and_unlock_cabinet_edges_once() {
    let instance = instance(
        &factory(),
        vec![(DIMENSION_SIX.into(), 19), (DIMENSION_SEVEN.into(), 19)],
    );
    let mut state = state(&instance);
    assert!(
        instance
            .pathstrider_cabinet_available(&state, ROOT)
            .unwrap()
    );
    assert_eq!(
        instance.pathstrider_cabinet_objective(ROOT),
        Some(ROOT_OBJECTIVE)
    );

    let program = instance
        .compile_pathstrider_cabinet_completion(&state, ROOT, ROOT_OBJECTIVE)
        .unwrap();
    commit(&instance, &mut state, program);

    assert_eq!(
        instance.communing_points(&state, DIMENSION_SIX).unwrap(),
        20
    );
    assert_eq!(
        instance.communing_points(&state, DIMENSION_SEVEN).unwrap(),
        20
    );
    assert!(
        !instance
            .pathstrider_cabinet_available(&state, ROOT)
            .unwrap()
    );
    let available = instance.available_pathstrider_cabinets(&state).unwrap();
    assert!(available.contains(&"swarm-disaster.pathstrider-cabinet.24"));
    assert!(available.contains(&"swarm-disaster.pathstrider-cabinet.13"));
    assert!(
        instance
            .compile_pathstrider_cabinet_completion(&state, ROOT, ROOT_OBJECTIVE)
            .is_err()
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    points: Vec<(String, u16)>,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(
            SwarmDisasterEntry::new(
                "swarm-disaster.area.201",
                "universe.path.destruction",
                "swarm-disaster.audience-die.6",
                participants(policy()),
            )
            .with_progression(points, vec![], None),
        )
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
