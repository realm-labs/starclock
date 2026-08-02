use proptest::{
    collection::vec,
    prelude::*,
    test_runner::{
        Config as ProptestConfig, FileFailurePersistence, RngAlgorithm, RngSeed, TestRunner,
    },
};
use starclock_agent_api::{
    activity_session::PlayActivityActionRequest,
    error::AgentErrorCode,
    schema::{ActionToken, AgentSchemaRevision, AgentUInt, IdempotencyKey, SessionId},
    swarm_disaster_activity_session::{
        CreateSwarmDisasterActivitySessionRequest, SwarmDisasterActivityAgentSessionFactory,
    },
};

const SEED: u64 = 20_001;
const MALFORMED_SEED: u64 = 0x7377_6172_6d2d_7270;

#[test]
fn four_thousand_ninety_six_forged_swarm_actions_preserve_exact_observation() {
    let factory = SwarmDisasterActivityAgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(CreateSwarmDisasterActivitySessionRequest {
            session_id: SessionId::parse("swarm_hardening_rejections").unwrap(),
            seed: AgentUInt::from_u64(SEED),
        })
        .unwrap();
    let before = session.observe().unwrap();
    let state = session.state_hash();
    let actions = session.replay_action_count();
    for index in 0..4_096_u32 {
        let error = session
            .apply_action(PlayActivityActionRequest {
                schema_revision: AgentSchemaRevision::V1,
                session_id: session.session_id().clone(),
                boundary_id: before.boundary_id.clone().unwrap(),
                expected_state_hash: before.state_hash.clone(),
                action_token: ActionToken::parse(&format!("forged_swarm_{index}")).unwrap(),
                idempotency_key: IdempotencyKey::parse(&format!("swarm_reject_{index}")).unwrap(),
            })
            .expect_err("forged token must reject");
        assert_eq!(error.code, AgentErrorCode::InvalidActionToken);
        assert!(!error.committed);
        assert_eq!(session.state_hash(), state);
        assert_eq!(session.replay_action_count(), actions);
        assert_eq!(session.observe().unwrap(), before);
    }
}

#[test]
fn two_hundred_fifty_six_malformed_swarm_replays_fail_repeatably_without_live_mutation() {
    let factory = SwarmDisasterActivityAgentSessionFactory::load_production().unwrap();
    let session = factory
        .create(CreateSwarmDisasterActivitySessionRequest {
            session_id: SessionId::parse("swarm_hardening_replays").unwrap(),
            seed: AgentUInt::from_u64(SEED),
        })
        .unwrap();
    let before = session.observe().unwrap();
    let mut runner = TestRunner::new(ProptestConfig {
        cases: 256,
        max_shrink_iters: 4_096,
        failure_persistence: Some(Box::new(FileFailurePersistence::SourceParallel(
            "proptest-regressions",
        ))),
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(MALFORMED_SEED),
        ..ProptestConfig::default()
    });
    runner
        .run(&vec(any::<u8>(), 0..=4_096), |bytes| {
            let first = session.verify_replay(&factory, &bytes);
            let second = session.verify_replay(&factory, &bytes);
            prop_assert_eq!(&first, &second);
            prop_assert_eq!(
                first
                    .expect_err("short arbitrary replay cannot verify")
                    .code,
                AgentErrorCode::ReplayDiverged
            );
            Ok(())
        })
        .unwrap();
    assert_eq!(session.observe().unwrap(), before);
    assert_eq!(session.replay_action_count(), 5);
}
