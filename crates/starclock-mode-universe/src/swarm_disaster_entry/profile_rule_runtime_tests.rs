use starclock_activity::{
    ActivityCause, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngStreams, ActivitySlotId, ActivityTransactionOutcome,
    ActivityTransactionState, ActivityValue,
};

use crate::swarm_disaster_entry::{
    SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    state::{COUNTDOWN, RESOURCES},
    tests::{BUNDLE, participants, policy, released_entry},
};

use super::*;

#[test]
fn profile_partition_binds_the_exact_frozen_production_rule() {
    let factory = factory();
    assert!(factory.profile_rule.id > 0);
    assert_eq!(
        factory
            .path_runtime
            .profile_bonus_keys()
            .collect::<Vec<_>>(),
        [
            "swarm-disaster.trailblaze-bonus.101",
            "swarm-disaster.trailblaze-bonus.102",
            "swarm-disaster.trailblaze-bonus.103",
            "swarm-disaster.trailblaze-bonus.104",
            "swarm-disaster.trailblaze-bonus.105",
            "swarm-disaster.trailblaze-bonus.106",
        ]
    );
    let mut malformed = factory
        .content
        .mechanic_rule_runtime_input("profile-entry")
        .unwrap();
    malformed.program = malformed
        .program
        .replace("ReviewFiveFormalDifficulties", "SkipDifficultyReview")
        .into();
    assert!(ProfileRuleRuntimeCatalog::compile(malformed, &factory.path_runtime).is_err());
}

#[test]
fn all_five_formal_entries_execute_once_and_stale_programs_reject_atomically() {
    let factory = factory();
    let mut hashes = Vec::new();
    for difficulty in 1_u8..=5 {
        let instance = instance(&factory, difficulty, None);
        let mut state = new_state(&instance);
        let program = instance.compile_profile_entry_rule(&state).unwrap();
        assert_eq!(program.operations().len(), 6);
        commit(&instance, &mut state, program.clone());
        assert_eq!(
            counter(&state, DEFERRED, marker(instance.profile_rule.id).unwrap()),
            1
        );
        assert!(instance.compile_profile_entry_rule(&state).is_err());
        let sequence = state.command_sequence();
        assert!(matches!(
            state.apply_program(
                &program,
                cause(&state, program.id()),
                instance.graph_definition()
            ),
            ActivityTransactionOutcome::Rejected(_)
        ));
        assert_eq!(state.command_sequence(), sequence);
        hashes.push(state_hash(&instance, &state, 0));
    }
    assert_eq!(
        hashes,
        [
            "6616852351cfcf29b3b5637f76ba25fc6040803d8126ffa91d90cc062ce9a466",
            "c155b35ba452affbac9ab0234000e83739102103321b9f5afccc7156eeecebbf",
            "b71b56b89e8117f25b30ba4a398d53985b4895ca8fc8db3a9e2687d7d8b405c1",
            "8faeb81b5ffd0b9b90bbd3fbb941231a2865c0ca8ecd65b8e67b177d842e503f",
            "3e8df53165db586ddfd484b6ad42063ed5f5ca341fa739326d7b2a5af5232c3e",
        ]
    );
}

#[test]
fn bonuses_101_through_106_bind_to_the_same_guarded_profile_rule() {
    let factory = factory();
    let mut observed = Vec::new();
    for bonus in 101_u16..=106 {
        let instance = instance(&factory, 1, Some(bonus));
        let mut state = new_state(&instance);
        if bonus == 104 {
            assert!(instance.compile_profile_entry_rule(&state).is_err());
            assert_eq!(state.command_sequence(), 0);
            assert_eq!(counter(&state, RESOURCES, 1), 50);
            observed.push((bonus, "Unaffordable".to_owned()));
            continue;
        }
        let program = instance.compile_profile_entry_rule(&state).unwrap();
        commit(&instance, &mut state, program);
        match bonus {
            101 => assert_eq!(counter(&state, RESOURCES, 1), 200),
            102 | 103 => assert_eq!(pending_count(&instance, &state), 1),
            105 => assert_eq!(pending_count(&instance, &state), 3),
            106 => {
                assert_eq!(integer_slot(&state, COUNTDOWN), 18);
                assert_eq!(pending_count(&instance, &state), 3);
            }
            _ => unreachable!(),
        }
        observed.push((bonus, state_hash(&instance, &state, 0)));
    }
    assert_eq!(
        observed,
        [
            (
                101,
                "ed84fee79c415216be56437e5600b261ddd292f7501ade997764db3b12783c5c".to_owned(),
            ),
            (
                102,
                "2dc54f94440544c449423d89ffa4ca5fffef5d3de357c439f2e52fc57d67e8ce".to_owned(),
            ),
            (
                103,
                "a1f9fec89dd48441a3566da06e6ad6fe11c28f66314380caee5101e5e785ef61".to_owned(),
            ),
            (104, "Unaffordable".to_owned()),
            (
                105,
                "782bd6528989652b52bdde4f5d2274e2fd3390478caa20c23c658720a77b0c3c".to_owned(),
            ),
            (
                106,
                "47c66e54ca6cb9fdb970cdba5db00d5b9b5966d28b575ebc0726f79ec3a162ed".to_owned(),
            ),
        ]
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}

fn instance(
    factory: &SwarmDisasterRuntimeFactory,
    difficulty: u8,
    bonus: Option<u16>,
) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(
            released_entry(
                format!("swarm-disaster.area.20{difficulty}"),
                "universe.path.destruction",
                "swarm-disaster.audience-die.6",
                participants(policy()),
            )
            .with_progression(
                vec![],
                vec![],
                bonus.map(|id| format!("swarm-disaster.trailblaze-bonus.{id}")),
            ),
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
    assert!(matches!(
        state.apply_program(
            &program,
            cause(state, program.id()),
            instance.graph_definition()
        ),
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
    match state.slot(ActivitySlotId::new(slot_id).unwrap()) {
        Some(ActivityValue::BoundedCounterMap(values)) => values
            .binary_search_by_key(&key, |(candidate, _)| *candidate)
            .ok()
            .map_or(0, |index| values[index].1),
        value => panic!("unexpected counter slot: {value:?}"),
    }
}

fn integer_slot(state: &ActivityTransactionState, slot_id: u32) -> i64 {
    match state.slot(ActivitySlotId::new(slot_id).unwrap()) {
        Some(ActivityValue::BoundedInteger(value)) => *value,
        value => panic!("unexpected integer slot: {value:?}"),
    }
}

fn pending_count(instance: &SwarmDisasterRuntimeInstance, state: &ActivityTransactionState) -> u16 {
    instance
        .path_runtime
        .pending_content_requests(state)
        .unwrap()
        .iter()
        .map(|request| request.count)
        .sum()
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

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
