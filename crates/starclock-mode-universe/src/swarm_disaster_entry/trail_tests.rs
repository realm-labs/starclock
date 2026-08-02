use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    trail::TrailRuntimeCatalog,
};

#[test]
fn frozen_trail_catalog_retains_exact_chains_and_projection_denominators() {
    let factory = factory();
    assert_eq!(factory.trail.denominators(), (63, 56, 58));

    let instance = full_instance(&factory, false);
    assert_eq!(instance.communing_trail_nodes().count(), 63);
    assert_eq!(instance.communing_trail_battle_effects().count(), 58);
    assert_eq!(
        instance
            .communing_trail_prerequisites("swarm-disaster.communing-trail.101")
            .unwrap()
            .count(),
        0
    );
    assert_eq!(
        instance
            .communing_trail_prerequisites("swarm-disaster.communing-trail.102")
            .unwrap()
            .collect::<Vec<_>>(),
        ["swarm-disaster.communing-trail.101"]
    );
}

#[test]
fn catalog_drift_fails_closed_before_entry_compilation() {
    let factory = factory();

    let mut effect_drift = factory.unique.trail_runtime_input();
    effect_drift.effects[0].domain = "Activity".into();
    assert!(TrailRuntimeCatalog::compile(effect_drift).is_err());

    let mut predecessor_drift = factory.unique.trail_runtime_input();
    predecessor_drift.prerequisites[0].required_points = "20".into();
    assert!(TrailRuntimeCatalog::compile(predecessor_drift).is_err());

    let mut projection_drift = factory.unique.trail_runtime_input();
    projection_drift.effects[0].battle_projection =
        r#"{"boundary":"NotApplicable","effect_ref":"","enabled":false}"#.into();
    assert!(TrailRuntimeCatalog::compile(projection_drift).is_err());
}

#[test]
fn entry_rejects_unmet_thresholds_and_non_closed_predecessors() {
    let factory = factory();
    let threshold = entry(
        vec![("swarm-disaster.communing-dimension.1".into(), 0)],
        vec!["swarm-disaster.communing-trail.101".into()],
        false,
    );
    assert!(factory.compile_entry(threshold).is_err());

    let missing = entry(
        vec![("swarm-disaster.communing-dimension.1".into(), 3)],
        vec!["swarm-disaster.communing-trail.102".into()],
        false,
    );
    assert!(factory.compile_entry(missing).is_err());

    let valid = entry(
        vec![("swarm-disaster.communing-dimension.1".into(), 3)],
        vec![
            "swarm-disaster.communing-trail.101".into(),
            "swarm-disaster.communing-trail.102".into(),
        ],
        false,
    );
    assert_eq!(
        factory
            .compile_entry(valid)
            .unwrap()
            .communing_trail_nodes()
            .count(),
        2
    );
}

#[test]
fn run_start_activity_effects_commit_once_and_stale_program_rejects() {
    let factory = factory();
    let instance = full_instance(&factory, false);
    assert_eq!(instance.trail.activity_totals(), (100, 1, 10, 2, 1));
    let mut state = new_state(&instance);
    let program = instance.compile_trail_run_start(&state).unwrap();
    let stale = program.clone();
    commit(&instance, &mut state, program);

    assert_eq!(resource(&state, 1), 150);
    assert_eq!(resource(&state, super::dice_control::CHEAT_CHARGE_KEY), 1);
    assert_eq!(instance.countdown(&state).unwrap(), 22);
    assert_eq!(
        counter(
            &state,
            super::state::PROGRESSION,
            super::trail::RUN_START_APPLIED_KEY
        ),
        1
    );
    assert!(instance.compile_trail_run_start(&state).is_err());
    let cause = cause(&state, stale.id());
    assert!(matches!(
        state.apply_program(&stale, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(resource(&state, 1), 150);
    assert_eq!(instance.countdown(&state).unwrap(), 22);
    let rng = activity_rng(&instance, 0x2041_0000);
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "180633b20b894967bfc8e0545a40ec4d18b814c2d74e20ae494873234a719525"
    );
}

#[test]
fn dice_and_plane_effects_execute_at_their_declared_boundaries() {
    let factory = factory();
    let instance = full_instance(&factory, true);
    let mut state = new_state(&instance);
    let run_start = instance.compile_trail_run_start(&state).unwrap();
    commit(&instance, &mut state, run_start);
    let mut rng = activity_rng(&instance, 0x2041_0001);
    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, roll);
    let abandon = instance.compile_dice_abandon(&state).unwrap();
    commit(&instance, &mut state, abandon);
    assert_eq!(resource(&state, 1), 170);

    let mut transition_state = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_ends().next().unwrap(),
    );
    let mut graph_rng = activity_rng(&instance, 0x2041_0002);
    let creation = instance.compile_plane_creation(0, &mut graph_rng).unwrap();
    commit(&instance, &mut transition_state, creation);
    let decay = instance
        .compile_boss_decay_selection(&transition_state, &["swarm-disaster.boss-decay.1"])
        .unwrap();
    commit(&instance, &mut transition_state, decay);
    let boss = instance
        .compile_boss_selection(1, "swarm-disaster.boss-choice.8003051")
        .unwrap();
    commit(&instance, &mut transition_state, boss);
    let completion = instance
        .compile_plane_completion(&transition_state, 1)
        .unwrap();
    commit(&instance, &mut transition_state, completion);
    assert_eq!(
        resource(&transition_state, super::dice_control::REROLL_CHARGE_KEY),
        1
    );
}

#[test]
fn battle_projections_retain_exact_parameters_and_bounded_entry_accounting() {
    let factory = factory();
    let instance = full_instance(&factory, false);
    assert!(
        instance
            .communing_trail_battle_effects()
            .any(|(_, _, effect)| effect == "source-effect.504")
    );
    assert_eq!(
        instance
            .communing_trail_battle_effect_parameters("source-effect.604")
            .unwrap()
            .collect::<Vec<_>>(),
        ["4", "0.99"]
    );
    assert_eq!(
        instance
            .communing_trail_battle_effect_parameters("source-effect.504")
            .unwrap()
            .count(),
        0
    );

    let mut state = new_state(&instance);
    assert!(
        instance
            .compile_trail_battle_entry_accounting(&state, 1, true, true)
            .unwrap()
            .is_none()
    );
    assert!(
        instance
            .compile_trail_battle_entry_accounting(&state, 1, false, false)
            .unwrap()
            .is_none()
    );
    let first = instance
        .compile_trail_battle_entry_accounting(&state, 1, false, true)
        .unwrap()
        .unwrap();
    let stale = first.clone();
    commit(&instance, &mut state, first);
    let stale_cause = cause(&state, stale.id());
    assert!(matches!(
        state.apply_program(&stale, stale_cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    for _ in 0..3 {
        let accounting = instance
            .compile_trail_battle_entry_accounting(&state, 1, false, true)
            .unwrap()
            .unwrap();
        commit(&instance, &mut state, accounting);
    }
    assert!(
        instance
            .compile_trail_battle_entry_accounting(&state, 1, false, true)
            .unwrap()
            .is_none()
    );
    assert_eq!(counter(&state, super::state::PROGRESSION, 3), 4);
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn full_instance(
    factory: &SwarmDisasterRuntimeFactory,
    abandon: bool,
) -> SwarmDisasterRuntimeInstance {
    let progression = factory
        .unique
        .trail_runtime_input()
        .nodes
        .iter()
        .map(|node| node.key.to_string())
        .collect();
    factory
        .compile_entry(entry(all_points(), progression, abandon))
        .unwrap()
}

fn all_points() -> Vec<(String, u16)> {
    (1..=7)
        .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
        .collect()
}

fn entry(
    points: Vec<(String, u16)>,
    progression: Vec<String>,
    abandon: bool,
) -> SwarmDisasterEntry {
    let entry = super::tests::released_entry(
        "swarm-disaster.area.201",
        "universe.path.destruction",
        "swarm-disaster.audience-die.6",
        super::tests::participants(super::tests::policy()),
    )
    .with_progression(points, progression, None);
    if abandon {
        entry.with_dice_control_unlocks(vec!["1000022".into()])
    } else {
        entry
    }
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
    let cause = cause(state, program.id());
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn cause(
    state: &ActivityTransactionState,
    program: starclock_activity::ActivityProgramId,
) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn counter(state: &ActivityTransactionState, slot_id: u32, key: u64) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()).unwrap() {
        ActivityValue::BoundedCounterMap(values) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => panic!("Swarm counter-map slot changed kind"),
    }
}

fn resource(state: &ActivityTransactionState, key: u64) -> i64 {
    counter(state, super::state::RESOURCES, key)
}

fn activity_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = identity();
    ActivityRngStreams::new(ActivityRngContext::new(
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
    ))
}

fn state_hash(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> String {
    state
        .state_hash(
            identity(),
            instance.graph_definition(),
            ActivityInstanceId::new(1).unwrap(),
            rng,
        )
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    )
}
