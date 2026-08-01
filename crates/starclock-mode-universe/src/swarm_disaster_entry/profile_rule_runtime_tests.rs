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
    assert_eq!(
        SWARM_DISASTER_PROFILE_RULE_RUNTIME_REVISION,
        "swarm-disaster-profile-entry-rule-runtime-v1"
    );
    assert!(factory.profile_rule.id > 0);
    assert_eq!(
        hex(factory.profile_rule.digest()),
        "3576fde8e5ae0c6ac5382548c8d2e68f1b27f7bfe3707e1d63578c357f4735ec"
    );
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
            "e66af553b6d0bf4343c587c87b7cf5e63c73b5deb320ff295e7518899ea8e43d",
            "e6a770162ce777a389f3de4bd3210f212c39eb30e19f671ceeb7298a1a9547b7",
            "916294881d4bbe44f60578fa7fab9e3a25a9c13442804fb441309afdb9771931",
            "72ab8a7d6f77853aa8dc07d1d2added9319a6e07d3ff786a27d0fe0aacc4d845",
            "42c6d8dcc65e306e85e8c14dd9e8cb7d3ef8a3d75ae5f5b7241041c2cc417b92",
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
                "f3f24eb7809c6d78c2c48526b8e65fff735c6936f981a4def147f535e836533c".to_owned(),
            ),
            (
                102,
                "e5bbb12d901fc9d3962bfead9b23ab084108c82283553274b59433fc389e856e".to_owned(),
            ),
            (
                103,
                "4cfaece29da022778b6df8e7c9cfad125abc041a40d3a1b5e3b7c0f443f7cbf6".to_owned(),
            ),
            (104, "Unaffordable".to_owned()),
            (
                105,
                "f4791aa2c1bbbfc81ee7218847a4e293b3c09e1ab066a0bda3dcb89f906e9ff2".to_owned(),
            ),
            (
                106,
                "858afcd25d5c03e7100d6e149c9bafd04ebe72917fc3b7d8ea4b05db89d29530".to_owned(),
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
