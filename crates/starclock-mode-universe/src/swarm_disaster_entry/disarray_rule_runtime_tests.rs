use starclock_activity::{ActivityCause, ActivityTransactionOutcome, ActivityTransactionState};

use crate::{
    error::UniverseCatalogLoadErrorKind,
    swarm_disaster_content::mechanic_access::MechanicRuleRuntimeInput,
};

use super::DisarrayRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

const FAMILIES: [&str; 3] = [
    "boss-decay-stack",
    "countdown-lifecycle",
    "planar-disarray-transition",
];

#[test]
fn exact_sora_partition_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let instance = factory.compile_entry(entry()).unwrap();
    assert_eq!(
        hex(instance.disarray_rule_runtime_digest()),
        "01ae0ca55d1fa4db3290c9fe2209f18219e7b3b76112b3faf5bb25fddc08cf12"
    );
    for family in FAMILIES {
        let mut inputs = inputs(&factory);
        inputs
            .iter_mut()
            .find(|input| input.family.as_ref() == family)
            .unwrap()
            .source_disposition = "RuntimeEnabled".into();
        assert_eq!(
            DisarrayRuleRuntimeCatalog::compile(inputs)
                .unwrap_err()
                .kind(),
            UniverseCatalogLoadErrorKind::InvalidReference
        );
    }
}

#[test]
fn countdown_enters_disarray_once_and_projects_capped_modifiers() {
    let instance = factory().compile_entry(entry()).unwrap();
    let mut state = state(&instance);
    assert_eq!(instance.countdown(&state).unwrap(), 20);
    assert_eq!(instance.disarray_level(&state).unwrap(), 0);
    for _ in 0..20 {
        apply_move(&instance, &mut state);
    }
    assert_eq!(instance.countdown(&state).unwrap(), 0);
    assert_eq!(instance.disarray_level(&state).unwrap(), 0);
    apply_move(&instance, &mut state);
    assert_eq!(instance.countdown(&state).unwrap(), -1);
    assert_eq!(instance.disarray_level(&state).unwrap(), 1);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (5, 4, 0));
    for _ in 1..=20 {
        apply_move(&instance, &mut state);
    }
    assert_eq!(instance.disarray_level(&state).unwrap(), 21);
    assert_eq!(instance.disarray_modifiers(&state).unwrap(), (275, 80, 125));
}

#[test]
fn boss_decay_thresholds_stack_once_and_gate_plane_completion() {
    let instance = factory().compile_entry(entry()).unwrap();
    let mut state = state(&instance);
    let program = instance
        .compile_boss_decay_selection(
            &state,
            &[
                "swarm-disaster.boss-decay.25",
                "swarm-disaster.boss-decay.1",
            ],
        )
        .unwrap();
    apply(&instance, &mut state, &program);
    assert_eq!(
        instance
            .disarray_rules
            .completion_requirements(&instance.countdown, &state, 3)
            .unwrap()
            .len(),
        2
    );
    assert!(
        instance
            .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.2"])
            .is_err()
    );
    assert!(
        instance
            .compile_boss_decay_selection(&state, &["swarm-disaster.boss-decay.101"])
            .is_err()
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn entry() -> crate::swarm_disaster_entry::SwarmDisasterEntry {
    super::super::tests::released_entry(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        super::super::tests::participants(super::super::tests::policy()),
    )
}

fn inputs(factory: &SwarmDisasterRuntimeFactory) -> [MechanicRuleRuntimeInput; 3] {
    FAMILIES.map(|family| factory.content.mechanic_rule_runtime_input(family).unwrap())
}

fn state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn apply_move(instance: &SwarmDisasterRuntimeInstance, state: &mut ActivityTransactionState) {
    let program = instance.compile_countdown_move(state, &[]).unwrap();
    apply(instance, state, &program);
}

fn apply(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    program: &starclock_activity::ActivityProgramDefinition,
) {
    assert!(matches!(
        state.apply_program(
            program,
            ActivityCause::new(
                state.command_sequence() + 1,
                program.id(),
                state.current_node(),
            )
            .unwrap(),
            instance.graph_definition(),
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
