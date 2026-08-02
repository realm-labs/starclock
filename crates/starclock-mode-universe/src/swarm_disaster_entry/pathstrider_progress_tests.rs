use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngStreams, ActivityTransactionOutcome, ActivityTransactionState,
};

use super::{
    SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    pathstrider_progress::PathstriderRuntimeCatalog,
};

#[test]
fn frozen_pathstrider_catalog_retains_exact_enabled_and_fail_closed_denominators() {
    let factory = factory();
    assert_eq!(factory.pathstrider.denominators(), (31, 15, 87, 15, 95));
    let instance = instance(&factory, all_points());
    assert_eq!(instance.pathstrider_finish_conditions().count(), 15);
    assert_eq!(instance.mechanical_chapters().count(), 13);
    assert_eq!(
        instance
            .mechanical_chapters()
            .filter(|(_, _, _, unresolved)| *unresolved)
            .count(),
        3
    );
    assert!(
        instance
            .mechanical_chapters()
            .any(|(key, layer, threshold, _)| {
                key == "swarm-disaster.mechanical-chapter.13" && layer == 3 && threshold.is_none()
            })
    );
}

#[test]
fn catalog_drift_fails_closed_before_entry_compilation() {
    let factory = factory();

    let mut objective = factory.unique.pathstrider_runtime_input();
    objective.objectives[0].progress_policy = r#"{"comparison":"Completed"}"#.into();
    assert!(PathstriderRuntimeCatalog::compile(objective).is_err());

    let mut unlock = factory.unique.pathstrider_runtime_input();
    let enabled = unlock
        .unlocks
        .iter_mut()
        .find(|row| row.key.as_ref() == "swarm-disaster.pathstrider-unlock.1000003")
        .unwrap();
    enabled.consequence = enabled
        .consequence
        .replace(
            "\"enabled_for_swarm_compilation\":true",
            "\"enabled_for_swarm_compilation\":false",
        )
        .into();
    assert!(PathstriderRuntimeCatalog::compile(unlock).is_err());

    let mut chapter = factory.unique.pathstrider_runtime_input();
    chapter.chapters[0].mechanical_unlock = chapter.chapters[0]
        .mechanical_unlock
        .replace("ChapterAvailabilityOnly", "UnresolvedFailClosed")
        .into();
    assert!(PathstriderRuntimeCatalog::compile(chapter).is_err());
}

#[test]
fn external_objective_routes_to_its_exact_cabinet_and_commits_once() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    let mut state = new_state(&instance);
    let condition = "swarm-disaster.external-quest-condition.6013222";
    let program = instance
        .compile_pathstrider_objective_completion(&state, condition)
        .unwrap();
    commit(&instance, &mut state, program);
    assert!(
        (1..=7)
            .map(|id| instance
                .communing_points(&state, &format!("swarm-disaster.communing-dimension.{id}"))
                .unwrap())
            .sum::<i64>()
            > 0
    );
    assert!(
        instance
            .compile_pathstrider_objective_completion(&state, condition)
            .is_err()
    );
    assert!(
        instance
            .compile_pathstrider_objective_completion(
                &state,
                "swarm-disaster.external-quest-condition.9999999"
            )
            .is_err()
    );
}

#[test]
fn external_progress_is_nondecreasing_unlocks_once_and_rejects_stale_programs() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    let mut state = new_state(&instance);
    let condition = "swarm-disaster.pathstrider-finish-condition.1000302";
    let unlock = "swarm-disaster.pathstrider-unlock.1000302";
    let stale = instance
        .compile_pathstrider_progress(&state, condition, 13)
        .unwrap()
        .unwrap();
    let partial = instance
        .compile_pathstrider_progress(&state, condition, 5)
        .unwrap()
        .unwrap();
    commit(&instance, &mut state, partial);
    assert!(!instance.pathstrider_unlock_applied(&state, unlock).unwrap());
    let stale_cause = cause(&state, stale.id());
    assert!(matches!(
        state.apply_program(&stale, stale_cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
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

#[test]
fn finish_parameters_are_exact_and_unresolved_shared_rows_stay_disabled() {
    let factory = factory();
    let instance = instance(&factory, vec![]);
    assert_eq!(
        instance
            .pathstrider_finish_parameters("swarm-disaster.pathstrider-finish-condition.1000008")
            .unwrap()
            .collect::<Vec<_>>(),
        ["8013106"]
    );
    let descriptor = instance
        .pathstrider_finish_conditions()
        .find(|(key, _, _, _)| *key == "swarm-disaster.pathstrider-finish-condition.1000015")
        .unwrap();
    assert_eq!(
        (descriptor.1, descriptor.2, descriptor.3),
        ("RogueFinishUnlock", "ListContain", 5)
    );

    let state = new_state(&instance);
    assert!(
        instance
            .compile_pathstrider_progress(
                &state,
                "swarm-disaster.pathstrider-finish-condition.1000001",
                1
            )
            .is_err()
    );
    assert!(
        instance
            .pathstrider_unlock_applied(&state, "swarm-disaster.pathstrider-unlock.1000001")
            .is_err()
    );
}

#[test]
fn chapter_availability_uses_current_plane_and_persistent_communing_points() {
    let factory = factory();
    let zero = instance(&factory, vec![]);
    let zero_state = new_state(&zero);
    assert!(
        zero.compile_mechanical_chapter_availability(&zero_state)
            .unwrap()
            .is_none()
    );

    let instance = instance(&factory, all_points());
    let mut first = new_state(&instance);
    let program = instance
        .compile_mechanical_chapter_availability(&first)
        .unwrap()
        .unwrap();
    commit(&instance, &mut first, program);
    assert!(
        instance
            .mechanical_chapter_available(&first, "swarm-disaster.mechanical-chapter.4")
            .unwrap()
    );
    assert!(
        !instance
            .mechanical_chapter_available(&first, "swarm-disaster.mechanical-chapter.2")
            .unwrap()
    );

    let mut third = ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.plane_starts().nth(2).unwrap(),
    );
    let all = instance
        .compile_mechanical_chapter_availability(&third)
        .unwrap()
        .unwrap();
    let stale = all.clone();
    commit(&instance, &mut third, all);
    assert!(
        instance
            .mechanical_chapters()
            .all(|(key, _, _, _)| instance.mechanical_chapter_available(&third, key).unwrap())
    );
    let stale_cause = cause(&third, stale.id());
    assert!(matches!(
        third.apply_program(&stale, stale_cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    let rng = activity_rng(&instance, 0x2042_0001);
    assert_eq!(
        state_hash(&instance, &third, &rng),
        "b62c9b911b0fd3b96a0f7ea6f615c84a9b28b2a2bd279694381754b16323b5c4"
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    points: Vec<(String, u16)>,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(
            super::tests::released_entry(
                "swarm-disaster.area.201",
                "universe.path.destruction",
                "swarm-disaster.audience-die.6",
                super::tests::participants(super::tests::policy()),
            )
            .with_progression(points, vec![], None),
        )
        .unwrap()
}

fn all_points() -> Vec<(String, u16)> {
    (1..=7)
        .map(|id| (format!("swarm-disaster.communing-dimension.{id}"), 20))
        .collect()
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

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    )
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
