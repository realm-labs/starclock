use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityProgramId, ActivityRngContext, ActivityRngStreams, ActivityTransactionOutcome,
    ActivityTransactionState,
};

use crate::swarm_disaster_entry::{
    SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    tests::{BUNDLE, participants, policy, released_entry},
};

use super::*;

#[test]
fn frozen_catalog_closes_all_path_runtime_rows() {
    let factory = factory();
    assert_eq!(REVISION, "swarm-disaster-path-resonance-runtime-v1");
    assert_eq!(factory.path_runtime.denominators(), (6, 8, 8, 32, 16));

    let propagation = instance(
        &factory,
        "universe.path.propagation",
        "swarm-disaster.audience-die.8",
        None,
    );
    assert!(propagation.path_is_propagation());
    assert_eq!(
        propagation.path_progression_unlock_id(),
        Some("swarm-disaster.pathstrider-unlock.1000008")
    );
    assert_eq!(propagation.path_resonance_bindings().count(), 4);
    assert_eq!(propagation.path_resonance_bindings().next().unwrap().2, 3);
    assert_eq!(
        propagation.path_boost_binding(),
        ("swarm-disaster.path-boost.641270", "StageAbility_641270")
    );
    assert_eq!(
        hex(propagation.path_runtime_digest()),
        "649f1d4c80be34556fd0c0e00bf1dc866815487b27e1371dd88631f464cd11b2"
    );
}

#[test]
fn bonuses_commit_immediate_and_deferred_work_once_with_stale_rejection() {
    let factory = factory();
    let bonus = instance(
        &factory,
        "universe.path.destruction",
        "swarm-disaster.audience-die.6",
        Some("swarm-disaster.trailblaze-bonus.106"),
    );
    let mut state = new_state(&bonus);
    let stale = bonus
        .compile_trailblaze_bonus_run_start(&state)
        .unwrap()
        .unwrap();
    commit(&bonus, &mut state, stale.clone());
    assert_eq!(integer_slot(&state, COUNTDOWN), 18);
    assert_eq!(
        counter(&state, DEFERRED, DEFERRED_BLESSING_BASE + 6 * 16),
        3
    );
    assert!(bonus.compile_trailblaze_bonus_run_start(&state).is_err());
    assert!(matches!(
        state.apply_program(&stale, cause(&state, stale.id()), bonus.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));

    let unaffordable = instance(
        &factory,
        "universe.path.destruction",
        "swarm-disaster.audience-die.6",
        Some("swarm-disaster.trailblaze-bonus.104"),
    );
    let initial = new_state(&unaffordable);
    assert!(
        unaffordable
            .compile_trailblaze_bonus_run_start(&initial)
            .is_err()
    );
    assert_eq!(counter(&initial, RESOURCES, COSMIC_FRAGMENTS_KEY), 50);
}

#[test]
fn interplays_activate_all_newly_satisfied_bindings_once() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        None,
    );
    let mut state = new_state(&instance);
    let counts = vec![
        ("universe.path.preservation".to_owned(), 3),
        ("universe.path.nihility".to_owned(), 3),
        ("universe.path.remembrance".to_owned(), 2),
    ];
    let first = instance
        .compile_resonance_interplays(&state, &counts)
        .unwrap()
        .unwrap();
    let stale = first.clone();
    commit(&instance, &mut state, first);
    assert_eq!(
        instance.active_resonance_interplays(&state).unwrap(),
        [(
            "swarm-disaster.resonance-interplay.1202401",
            "universe.path.nihility",
            "StageAbility_612024"
        )]
    );
    let second_counts = vec![
        ("universe.path.preservation".to_owned(), 3),
        ("universe.path.nihility".to_owned(), 3),
        ("universe.path.remembrance".to_owned(), 3),
    ];
    let second = instance
        .compile_resonance_interplays(&state, &second_counts)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, second);
    assert_eq!(
        instance.active_resonance_interplays(&state).unwrap().len(),
        2
    );
    assert!(
        instance
            .compile_resonance_interplays(&state, &second_counts)
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        state.apply_program(
            &stale,
            cause(&state, stale.id()),
            instance.graph_definition()
        ),
        ActivityTransactionOutcome::Rejected(_)
    ));
}

#[test]
fn malformed_propagation_and_interplay_rows_fail_closed() {
    let factory = factory();
    let mut propagation = factory.unique.path_runtime_input();
    propagation.paths[7].propagation_unlock = propagation.paths[7]
        .propagation_unlock
        .replace("ReleasedUnlockRowBound", "NotApplicable")
        .into();
    assert!(PathRuntimeCatalog::compile(propagation, &factory.pathstrider).is_err());

    let mut threshold = factory.unique.path_runtime_input();
    threshold.interplays[0].thresholds = threshold.interplays[0]
        .thresholds
        .replace("DistinctOwnedBlessingIdentity", "TotalBlessingStacks")
        .into();
    assert!(PathRuntimeCatalog::compile(threshold, &factory.pathstrider).is_err());
}

#[test]
fn seeded_bonus_and_interplay_state_hash_is_stable() {
    let factory = factory();
    let instance = instance(
        &factory,
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        Some("swarm-disaster.trailblaze-bonus.101"),
    );
    let mut state = new_state(&instance);
    let bonus = instance
        .compile_trailblaze_bonus_run_start(&state)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, bonus);
    let counts = vec![
        ("universe.path.preservation".to_owned(), 3),
        ("universe.path.nihility".to_owned(), 3),
        ("universe.path.remembrance".to_owned(), 3),
    ];
    let interplays = instance
        .compile_resonance_interplays(&state, &counts)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, interplays);
    assert_eq!(
        state_hash(&instance, &state, 0x2043_0001),
        "043f3b5b6e84a57bd320278db17ba3966ceb0e06ac21b6f9a3cd162e412f715d"
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    path: &str,
    die: &str,
    bonus: Option<&str>,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(
            released_entry("swarm-disaster.area.201", path, die, participants(policy()))
                .with_progression(vec![], vec![], bonus.map(str::to_owned)),
        )
        .unwrap()
}

fn new_state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
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
            instance.graph_definition()
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(state: &ActivityTransactionState, program: ActivityProgramId) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    counter_value(state, slot_id, key).unwrap()
}

fn integer_slot(state: &ActivityTransactionState, slot_id: u32) -> i64 {
    integer_value(state, slot_id).unwrap()
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn state_hash(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    seed: u64,
) -> String {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    );
    let rng = ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ));
    hex(state
        .state_hash(
            identity,
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            &rng,
        )
        .bytes())
}
