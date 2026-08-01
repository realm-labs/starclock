use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityExpression, ActivityInstanceId, ActivityMasterSeed,
    ActivityOperation, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivityScope, ActivitySlotId,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue, SlotCarryPolicy,
};

use super::{
    SWARM_DISASTER_DICE_CONTROL_REVISION, SwarmDisasterEntry, SwarmDisasterRuntimeFactory,
    SwarmDisasterRuntimeInstance, dice_control::DiceControlRuntimeCatalog,
};

#[test]
fn four_authored_controls_compile_exact_policy_and_unlock() {
    let factory = factory();
    assert_eq!(
        SWARM_DISASTER_DICE_CONTROL_REVISION,
        "swarm-disaster-dice-control-v1"
    );
    assert_eq!(factory.dice_controls.denominators(), (4, "1000022"));
    let instance = instance(&factory, false);
    let resources = instance
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id().get() == super::state::RESOURCES)
        .unwrap();
    assert_eq!(resources.owner(), ActivityScope::Activity);
    assert_eq!(resources.carry(), SlotCarryPolicy::CarryExact);
    let resolution = instance
        .state_definition()
        .slots()
        .iter()
        .find(|slot| slot.id().get() == super::state::DICE_RESOLUTION)
        .unwrap();
    assert_eq!(resolution.owner(), ActivityScope::Attempt);
    assert_eq!(resolution.carry(), SlotCarryPolicy::Reset);

    let unknown = entry(false).with_dice_control_unlocks(vec!["unknown".into()]);
    assert!(factory.compile_entry(unknown).is_err());
    let duplicate =
        entry(false).with_dice_control_unlocks(vec!["1000022".into(), "1000022".into()]);
    assert!(factory.compile_entry(duplicate).is_err());

    let mut drift = factory.unique.dice_control_runtime_input();
    drift[0].abandon_reward = "\"11\"".into();
    assert!(DiceControlRuntimeCatalog::compile(&drift).is_err());
}

#[test]
fn roll_uses_one_spawn_draw_and_authored_candidates() {
    let factory = factory();
    let instance = instance(&factory, false);
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 0x2003_3201);
    let before = rng.snapshots();
    assert!(instance.dice_roll_available(&state).unwrap());

    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    assert_one_spawn_draw(&before, &rng.snapshots());
    commit(&instance, &mut state, roll);
    assert!(
        instance
            .audience_die_faces()
            .any(|face| Some(face) == instance.dice_resolution_face(&state))
    );
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(1));
    assert!(!instance.dice_roll_available(&state).unwrap());

    let snapshots = rng.snapshots();
    let bytes = state_bytes(&instance, &state, &rng);
    assert!(instance.compile_dice_roll(&state, &mut rng).is_err());
    assert_eq!(rng.snapshots(), snapshots);
    assert_eq!(state_bytes(&instance, &state, &rng), bytes);
}

#[test]
fn reroll_and_cheat_consume_exact_charges_with_isolated_rng() {
    let factory = factory();
    let instance = instance(&factory, false);
    let mut state = new_state(&instance);
    grant_charges(&instance, &mut state, 1, 1);
    let mut rng = activity_rng(&instance, 0x2003_3202);

    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, roll);
    let before_reroll = rng.snapshots();
    let reroll = instance.compile_dice_reroll(&state, &mut rng).unwrap();
    assert_one_spawn_draw(&before_reroll, &rng.snapshots());
    commit(&instance, &mut state, reroll);
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(2));
    assert_eq!(resource(&state, super::dice_control::REROLL_CHARGE_KEY), 0);

    let selected = instance.audience_die_faces().next().unwrap().to_owned();
    let before_cheat = rng.snapshots();
    let cheat = instance
        .compile_dice_cheat(&state, selected.as_str())
        .unwrap();
    assert_eq!(rng.snapshots(), before_cheat);
    commit(&instance, &mut state, cheat);
    assert_eq!(
        instance.dice_resolution_face(&state),
        Some(selected.as_str())
    );
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(3));
    assert_eq!(resource(&state, super::dice_control::CHEAT_CHARGE_KEY), 0);

    let before = state_bytes(&instance, &state, &rng);
    let snapshots = rng.snapshots();
    assert!(instance.compile_dice_reroll(&state, &mut rng).is_err());
    assert!(instance.compile_dice_cheat(&state, "missing").is_err());
    assert_eq!(state_bytes(&instance, &state, &rng), before);
    assert_eq!(rng.snapshots(), snapshots);
}

#[test]
fn abandon_requires_unlock_rewards_once_and_closes_attempt() {
    let factory = factory();
    let locked = instance(&factory, false);
    let mut locked_state = new_state(&locked);
    let mut locked_rng = activity_rng(&locked, 0x2003_3203);
    let roll = locked
        .compile_dice_roll(&locked_state, &mut locked_rng)
        .unwrap();
    commit(&locked, &mut locked_state, roll);
    let locked_bytes = state_bytes(&locked, &locked_state, &locked_rng);
    assert!(locked.compile_dice_abandon(&locked_state).is_err());
    assert_eq!(
        state_bytes(&locked, &locked_state, &locked_rng),
        locked_bytes
    );

    let instance = instance(&factory, true);
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 0x2003_3203);
    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, roll);
    assert!(instance.dice_abandon_available(&state).unwrap());
    let snapshots = rng.snapshots();
    let abandon = instance.compile_dice_abandon(&state).unwrap();
    assert_eq!(rng.snapshots(), snapshots);
    commit(&instance, &mut state, abandon);
    assert_eq!(instance.dice_resolution_face(&state), None);
    assert_eq!(instance.dice_resolution_kind(&state).unwrap(), Some(4));
    assert_eq!(resource(&state, 1), 60);
    assert!(!instance.dice_roll_available(&state).unwrap());
    assert!(!instance.dice_reroll_available(&state).unwrap());
    assert!(!instance.dice_cheat_available(&state).unwrap());
    assert!(!instance.dice_abandon_available(&state).unwrap());
    assert!(instance.compile_dice_roll(&state, &mut rng).is_err());
}

#[test]
fn empty_candidates_and_late_stale_programs_reject_atomically() {
    let factory = factory();
    let instance = instance(&factory, false);
    let mut state = new_state(&instance);
    let mut rng = activity_rng(&instance, 0x2003_3204);
    let before = rng.snapshots();
    assert!(
        instance
            .dice_controls
            .compile_roll(&state, &[], &mut rng)
            .is_err()
    );
    assert_eq!(rng.snapshots(), before);

    let first = instance.compile_dice_roll(&state, &mut rng).unwrap();
    let stale = instance.compile_dice_roll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, first);
    grant_charges(&instance, &mut state, 1, 0);
    let no_candidate_bytes = state_bytes(&instance, &state, &rng);
    let no_candidate_rng = rng.snapshots();
    assert!(
        instance
            .dice_controls
            .compile_reroll(&state, &[], &mut rng)
            .is_err()
    );
    assert_eq!(state_bytes(&instance, &state, &rng), no_candidate_bytes);
    assert_eq!(rng.snapshots(), no_candidate_rng);
    let bytes = state_bytes(&instance, &state, &rng);
    let snapshots = rng.snapshots();
    let cause = cause(&state, stale.id());
    assert!(matches!(
        state.apply_program(&stale, cause, instance.graph_definition()),
        ActivityTransactionOutcome::Rejected(_)
    ));
    assert_eq!(state_bytes(&instance, &state, &rng), bytes);
    assert_eq!(rng.snapshots(), snapshots);
}

#[test]
fn seeded_control_sequence_freezes_state_and_rng_hash() {
    let factory = factory();
    let instance = instance(&factory, true);
    let mut state = new_state(&instance);
    grant_charges(&instance, &mut state, 1, 1);
    let mut rng = activity_rng(&instance, 0x2003_3205);
    let roll = instance.compile_dice_roll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, roll);
    let reroll = instance.compile_dice_reroll(&state, &mut rng).unwrap();
    commit(&instance, &mut state, reroll);
    let selected = instance.audience_die_faces().last().unwrap().to_owned();
    let cheat = instance.compile_dice_cheat(&state, &selected).unwrap();
    commit(&instance, &mut state, cheat);
    let abandon = instance.compile_dice_abandon(&state).unwrap();
    commit(&instance, &mut state, abandon);

    assert_eq!(
        state_hash(&instance, &state, &rng),
        "1480a1997138978ba76f5f4514d689274fffd500d5cdef1298efaa516fa7a6e2"
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::tests::BUNDLE).unwrap()
}

fn entry(abandon: bool) -> SwarmDisasterEntry {
    let entry = super::tests::released_entry(
        "swarm-disaster.area.201",
        "universe.path.preservation",
        "swarm-disaster.audience-die.1",
        super::tests::participants(super::tests::policy()),
    );
    if abandon {
        entry.with_dice_control_unlocks(vec!["1000022".into()])
    } else {
        entry
    }
}

fn instance(factory: &SwarmDisasterRuntimeFactory, abandon: bool) -> SwarmDisasterRuntimeInstance {
    factory.compile_entry(entry(abandon)).unwrap()
}

fn new_state(instance: &SwarmDisasterRuntimeInstance) -> ActivityTransactionState {
    ActivityTransactionState::new(
        instance.state_definition().clone(),
        instance.graph_definition().entry(),
    )
}

fn grant_charges(
    instance: &SwarmDisasterRuntimeInstance,
    state: &mut ActivityTransactionState,
    rerolls: i64,
    cheats: i64,
) {
    let program = ActivityProgramDefinition::new(
        ActivityProgramId::new(0x5320_ff00).unwrap(),
        vec![
            ActivityOperation::AddCounter {
                slot: ActivitySlotId::new(super::state::RESOURCES).unwrap(),
                key: super::dice_control::REROLL_CHARGE_KEY,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(rerolls)),
            },
            ActivityOperation::AddCounter {
                slot: ActivitySlotId::new(super::state::RESOURCES).unwrap(),
                key: super::dice_control::CHEAT_CHARGE_KEY,
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(cheats)),
            },
        ],
    )
    .unwrap();
    commit(instance, state, program);
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

fn cause(state: &ActivityTransactionState, program: ActivityProgramId) -> ActivityCause {
    ActivityCause::new(state.command_sequence() + 1, program, state.current_node()).unwrap()
}

fn resource(state: &ActivityTransactionState, key: u64) -> i64 {
    match state
        .slot(ActivitySlotId::new(super::state::RESOURCES).unwrap())
        .unwrap()
    {
        ActivityValue::BoundedCounterMap(values) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        _ => panic!("resources slot changed kind"),
    }
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

fn assert_one_spawn_draw(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
) {
    assert_eq!(before.len(), after.len());
    for (old, new) in before.iter().zip(after) {
        assert_eq!(old.label(), new.label());
        assert_eq!(
            new.draw_count(),
            old.draw_count() + u64::from(old.label() == ActivityRngLabel::Spawn)
        );
    }
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
