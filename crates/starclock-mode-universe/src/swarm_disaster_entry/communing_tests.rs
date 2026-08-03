use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngStreams, ActivityScope, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue, SlotCarryPolicy,
};

use super::{
    SwarmDisasterEntry, SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    communing::CommuningRuntimeCatalog,
};
use super::{state, tests};

const ROOT: &str = "swarm-disaster.pathstrider-cabinet.22";
const ROOT_OBJECTIVE: &str = "6013222";
const DIMENSION_SIX: &str = "swarm-disaster.communing-dimension.6";
const DIMENSION_SEVEN: &str = "swarm-disaster.communing-dimension.7";

#[test]
fn frozen_communing_catalog_retains_exact_denominators_and_order() {
    let factory = factory();
    assert_eq!(factory.communing.denominators(), (21, 7, 31, 55, 33, 34));
    let instance = instance(&factory, vec![], vec![]);
    assert_eq!(
        instance.communing_choices(4).collect::<Vec<_>>(),
        [
            "swarm-disaster.communing-choice.441",
            "swarm-disaster.communing-choice.461",
            "swarm-disaster.communing-choice.421",
            "swarm-disaster.communing-choice.431",
            "swarm-disaster.communing-choice.411",
            "swarm-disaster.communing-choice.451",
            "swarm-disaster.communing-choice.401",
        ]
    );
    assert_eq!(instance.communing_choices(6).count(), 7);
    assert_eq!(instance.communing_choices(7).count(), 7);
    assert_eq!(instance.communing_choices(5).count(), 0);
    assert_eq!(instance.communing_maximum(DIMENSION_SIX), Some(20));

    let communing_slot = slot_definition(&instance, state::COMMUNING);
    assert_eq!(communing_slot.owner(), ActivityScope::Activity);
    assert_eq!(communing_slot.carry(), SlotCarryPolicy::CarryExact);
    let choice_slot = slot_definition(&instance, state::COMMUNING_CHOICE);
    assert_eq!(choice_slot.owner(), ActivityScope::Attempt);
    assert_eq!(choice_slot.carry(), SlotCarryPolicy::Reset);
}

#[test]
fn catalog_drift_fails_closed_before_entry_compilation() {
    let factory = factory();

    let mut choice_drift = factory.unique.communing_runtime_input();
    choice_drift.choices[0].point_deltas =
        r#"[{"dimension_id":"swarm-disaster.communing-dimension.1","delta":"1"}]"#.into();
    assert!(CommuningRuntimeCatalog::compile(choice_drift).is_err());

    let mut dimension_drift = factory.unique.communing_runtime_input();
    dimension_drift.dimensions[0].maximum = 21;
    assert!(CommuningRuntimeCatalog::compile(dimension_drift).is_err());

    let mut cabinet_drift = factory.unique.communing_runtime_input();
    cabinet_drift
        .cabinets
        .iter_mut()
        .find(|cabinet| cabinet.key.as_ref() == ROOT)
        .unwrap()
        .unlock_keys = Box::new([]);
    assert!(CommuningRuntimeCatalog::compile(cabinet_drift).is_err());
}

#[test]
fn story_choice_increments_only_its_aeon_and_closes_the_stage() {
    let factory = factory();
    let instance = instance(&factory, vec![], vec![]);
    let mut state = new_state(&instance);
    let rng = activity_rng(&instance, 0x2003_3401);
    let snapshots = rng.snapshots();
    let choice = "swarm-disaster.communing-choice.441";

    assert!(
        instance
            .communing_choice_available(&state, 4, choice)
            .unwrap()
    );
    let program = instance
        .compile_communing_choice(&state, 4, choice)
        .unwrap();
    commit(&instance, &mut state, program);

    assert_eq!(rng.snapshots(), snapshots);
    assert_eq!(
        instance
            .communing_choice_count(&state, "universe.path.preservation")
            .unwrap(),
        1
    );
    assert_eq!(instance.communing_points(&state, DIMENSION_SIX).unwrap(), 0);
    assert!(
        !instance
            .communing_choice_available(&state, 4, choice)
            .unwrap()
    );
    assert!(
        instance
            .compile_communing_choice(&state, 4, "swarm-disaster.communing-choice.461")
            .is_err()
    );

    // The API never receives RNG; retain the stream to prove the no-draw rule.
    assert_eq!(rng.snapshots(), snapshots);
}

#[test]
fn root_cabinet_requires_exact_objective_then_unlocks_outgoing_edges() {
    let factory = factory();
    let instance = instance(&factory, vec![], vec![]);
    let mut state = new_state(&instance);

    assert_eq!(
        instance.pathstrider_cabinet_objective(ROOT),
        Some(ROOT_OBJECTIVE)
    );
    assert_eq!(
        instance
            .pathstrider_cabinet_prerequisites(ROOT)
            .unwrap()
            .count(),
        0
    );
    assert!(
        instance
            .pathstrider_cabinet_available(&state, ROOT)
            .unwrap()
    );
    assert!(
        !instance
            .pathstrider_cabinet_available(&state, "swarm-disaster.pathstrider-cabinet.24")
            .unwrap()
    );
    assert!(
        instance
            .compile_pathstrider_cabinet_completion(
                &state,
                "swarm-disaster.pathstrider-cabinet.24",
                "6013224"
            )
            .is_err()
    );
    assert!(
        instance
            .compile_pathstrider_cabinet_completion(&state, ROOT, "wrong-objective")
            .is_err()
    );

    let program = instance
        .compile_pathstrider_cabinet_completion(&state, ROOT, ROOT_OBJECTIVE)
        .unwrap();
    commit(&instance, &mut state, program);
    assert_eq!(instance.communing_points(&state, DIMENSION_SIX).unwrap(), 2);
    assert_eq!(
        instance.communing_points(&state, DIMENSION_SEVEN).unwrap(),
        3
    );
    assert!(
        !instance
            .pathstrider_cabinet_available(&state, ROOT)
            .unwrap()
    );
    let available = instance.available_pathstrider_cabinets(&state).unwrap();
    assert!(available.contains(&"swarm-disaster.pathstrider-cabinet.24"));
    assert!(available.contains(&"swarm-disaster.pathstrider-cabinet.13"));
}

#[test]
fn ordered_cabinet_points_clamp_each_dimension_to_twenty() {
    let factory = factory();
    let instance = instance(
        &factory,
        vec![(DIMENSION_SIX.into(), 19), (DIMENSION_SEVEN.into(), 19)],
        vec![],
    );
    let mut state = new_state(&instance);
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
}

#[test]
fn corrupt_cabinet_completion_state_fails_closed() {
    let factory = factory();
    let instance = instance(&factory, vec![], vec![]);
    let mut state = new_state(&instance);
    let corruption = ActivityProgramDefinition::new(
        ActivityProgramId::new(0x5370_ff00).unwrap(),
        vec![ActivityOperation::AddCounter {
            slot: ActivitySlotId::new(state::PROGRESSION).unwrap(),
            key: 0x2000_0000 + 24,
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(2)),
        }],
    )
    .unwrap();
    commit(&instance, &mut state, corruption);
    assert!(
        instance
            .pathstrider_cabinet_available(&state, ROOT)
            .is_err()
    );
    assert!(
        instance
            .compile_pathstrider_cabinet_completion(&state, ROOT, ROOT_OBJECTIVE)
            .is_err()
    );
}

#[test]
fn stale_cabinet_program_rejects_atomically_and_seeded_hash_is_stable() {
    let factory = factory();
    let instance = instance(&factory, vec![], vec![]);
    let mut state = new_state(&instance);
    let rng = activity_rng(&instance, 0x2003_3402);
    let stale = instance
        .compile_pathstrider_cabinet_completion(&state, ROOT, ROOT_OBJECTIVE)
        .unwrap();
    let accepted = instance
        .compile_pathstrider_cabinet_completion(&state, ROOT, ROOT_OBJECTIVE)
        .unwrap();
    commit(&instance, &mut state, accepted);
    let before = state_bytes(&instance, &state, &rng);
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        stale.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&stale, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    assert_eq!(
        state_hash(&instance, &state, &rng),
        "b518927008b90f89303d5496c7374cc13701b030c7929ee3c6bf30db8f3bb788"
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(tests::BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    points: Vec<(String, u16)>,
    progression: Vec<String>,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(
            SwarmDisasterEntry::new(
                "swarm-disaster.area.201",
                "universe.path.destruction",
                "swarm-disaster.audience-die.6",
                tests::participants(tests::policy()),
            )
            .with_progression(points, progression, None),
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
    let cause = ActivityCause::new(
        state.command_sequence() + 1,
        program.id(),
        state.current_node(),
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(&program, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Committed(_)
    ));
}

fn slot_definition(
    instance: &SwarmDisasterRuntimeInstance,
    raw: u32,
) -> &starclock_activity::ActivitySlotDefinition {
    instance
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id() == ActivitySlotId::new(raw).unwrap())
        .unwrap()
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

fn state_bytes(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    rng: &ActivityRngStreams,
) -> Box<[u8]> {
    state.canonical_state_bytes(
        identity(),
        instance.graph_definition(),
        ActivityInstanceId::new(1).unwrap(),
        rng,
    )
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
