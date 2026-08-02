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
            "01fa4f02481f316e674c0ef7193c7064dda57bbf557518607c3f6acae81ae599",
            "7d23e40ef7deaae27f9cc7ccb5b96b17d79cfc32f202f4632799da55282ec518",
            "cf09c44c3c8c08004c87210dae078decea86445870604c556a521956cfe82009",
            "04ddc5b8b236290a4059b075b2f261d4a9c1b4232b34eaff1ae47180511f8f39",
            "c2004e3a8a40aa60a7c4405453cdb054150e8d455afd1c77e4b63a89b36edcaa",
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
                "eb01fb935e64ae069c97814e915d4104d4432672e8f58a594fd905601077bbee".to_owned(),
            ),
            (
                102,
                "d7e45e456976ef921a31578130756a7dfb967150dd30829dfdf1e424057dfa59".to_owned(),
            ),
            (
                103,
                "88314ba8ab01088954b44f625c093315e7638d96e695eef327385c832268a68d".to_owned(),
            ),
            (104, "Unaffordable".to_owned()),
            (
                105,
                "43bdc02eeceb62a6de9143d004313fa9591f3ef58e3cfbf145872edb303e65f5".to_owned(),
            ),
            (
                106,
                "e379683b9b69109cf81f58d350e8ffff52881f93b3a4b89e152bad7237a5ca97".to_owned(),
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
