use starclock_activity::{
    ActivityCause, ActivityProgramDefinition, ActivityProgramId, ActivityTransactionOutcome,
    ActivityTransactionState,
};

use crate::{
    error::UniverseCatalogLoadErrorKind,
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::ProgressionRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

const FAMILIES: [&str; 2] = ["communing-trail-effect", "pathstrider-progress"];

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
            .domain = "Battle".into();
        assert_eq!(
            ProgressionRuleRuntimeCatalog::compile(inputs)
                .unwrap_err()
                .kind(),
            UniverseCatalogLoadErrorKind::InvalidReference
        );
    }
}

#[test]
fn exact_trail_selection_routes_activity_and_battle_contributions_once() {
    let factory = factory();
    let progression = factory
        .unique
        .trail_runtime_input()
        .nodes
        .iter()
        .map(|node| node.key.to_string())
        .collect();
    let instance = instance(&factory, progression);
    let mut state = state(&instance);
    assert_eq!(instance.communing_trail_nodes().count(), 63);
    assert_eq!(instance.communing_trail_battle_effects().count(), 58);

    let program = instance.compile_trail_run_start(&state).unwrap();
    let stale = program.clone();
    commit(&instance, &mut state, program);
    assert!(instance.compile_trail_run_start(&state).is_err());
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
fn pathstrider_progress_routes_nondecreasing_unlocks_exactly_once() {
    let instance = instance(&factory(), vec![]);
    let mut state = state(&instance);
    let condition = "swarm-disaster.pathstrider-finish-condition.1000302";
    let unlock = "swarm-disaster.pathstrider-unlock.1000302";
    let partial = instance
        .compile_pathstrider_progress(&state, condition, 5)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, partial);
    assert!(!instance.pathstrider_unlock_applied(&state, unlock).unwrap());

    let terminal = instance
        .compile_pathstrider_progress(&state, condition, 13)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, terminal);
    assert!(instance.pathstrider_unlock_applied(&state, unlock).unwrap());
    assert!(
        instance
            .compile_pathstrider_progress(&state, condition, 13)
            .unwrap()
            .is_none()
    );
    assert!(
        instance
            .compile_pathstrider_progress(&state, condition, 12)
            .is_err()
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    progression: Vec<String>,
) -> SwarmDisasterRuntimeInstance {
    let points = (1..=7)
        .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
        .collect();
    factory
        .compile_entry(
            super::super::tests::released_entry(
                "swarm-disaster.area.201",
                "universe.path.destruction",
                "swarm-disaster.audience-die.6",
                super::super::tests::participants(super::super::tests::policy()),
            )
            .with_progression(points, progression, None),
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
