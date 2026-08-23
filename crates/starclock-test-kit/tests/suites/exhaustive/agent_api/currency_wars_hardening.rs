use starclock_agent_api::{
    activity_session::PlayActivityActionRequest,
    currency_wars_activity_session::{
        AgentCurrencyWarsGambit, CreateCurrencyWarsActivitySessionRequest,
        CurrencyWarsActivityAgentSessionFactory,
    },
    error::AgentErrorCode,
    schema::{AgentUInt, IdempotencyKey, SessionId},
};

#[test]
fn malformed_replay_corpus_is_bounded_and_never_mutates_a_live_session() {
    let factory = CurrencyWarsActivityAgentSessionFactory::load_production().expect("factory");
    let session = create(&factory, "cw_hardening_replay");
    let before = session.state_hash();
    for bytes in [
        &[][..],
        b"SCRP".as_slice(),
        b"SCRP\0currency-wars".as_slice(),
        &[0xff; 128][..],
    ] {
        assert_eq!(
            factory.verify_replay(bytes).unwrap_err().code,
            AgentErrorCode::ReplayDiverged
        );
        assert_eq!(session.state_hash(), before);
    }
}

#[test]
fn stale_and_idempotency_conflict_rejections_are_state_inert() {
    let factory = CurrencyWarsActivityAgentSessionFactory::load_production().expect("factory");
    let mut session = create(&factory, "cw_hardening_stale");
    let initial = session.observe().expect("observation");
    let action = initial.legal_actions.first().expect("action");
    let accepted = PlayActivityActionRequest {
        session_id: initial.session_id.clone(),
        boundary_id: initial.boundary_id.clone().expect("boundary"),
        expected_state_hash: initial.state_hash.clone(),
        action_token: action.token.clone(),
        idempotency_key: IdempotencyKey::parse("cw_hardening_key").expect("key"),
    };
    session.apply_action(accepted.clone()).expect("accepted");
    let after = session.state_hash();
    assert_eq!(
        session
            .apply_action(accepted)
            .expect("retry")
            .observation
            .state_hash,
        after
    );

    let current = session.observe().expect("current");
    let conflict = PlayActivityActionRequest {
        session_id: current.session_id,
        boundary_id: current.boundary_id.expect("boundary"),
        expected_state_hash: current.state_hash,
        action_token: current.legal_actions.first().expect("action").token.clone(),
        idempotency_key: IdempotencyKey::parse("cw_hardening_key").expect("key"),
    };
    assert_eq!(
        session.apply_action(conflict).unwrap_err().code,
        AgentErrorCode::IdempotencyConflict
    );
    assert_eq!(session.state_hash(), after);
}

#[test]
fn same_seed_sessions_keep_rng_and_observation_identity_isolated() {
    let factory = CurrencyWarsActivityAgentSessionFactory::load_production().expect("factory");
    let mut left = create(&factory, "cw_hardening_left");
    let mut right = create(&factory, "cw_hardening_right");
    for ordinal in 0..4 {
        let left_view = left.observe().expect("left observation");
        let right_view = right.observe().expect("right observation");
        assert_eq!(left_view.state_hash, right_view.state_hash);
        play(&mut left, left_view, &format!("cw_left_{ordinal}"));
        play(&mut right, right_view, &format!("cw_right_{ordinal}"));
    }
    assert_eq!(left.state_hash(), right.state_hash());
}

fn create(
    factory: &CurrencyWarsActivityAgentSessionFactory,
    id: &str,
) -> starclock_agent_api::currency_wars_activity_session::CurrencyWarsActivityAgentSession {
    factory
        .create(CreateCurrencyWarsActivitySessionRequest {
            session_id: SessionId::parse(id).expect("session ID"),
            route_id: AgentUInt::from_u64(801),
            difficulty_id: AgentUInt::from_u64(1),
            gambit: AgentCurrencyWarsGambit::Standard,
            seed: AgentUInt::from_u64(31_000_501),
        })
        .expect("session")
}

fn play(
    session: &mut starclock_agent_api::currency_wars_activity_session::CurrencyWarsActivityAgentSession,
    observation: starclock_agent_api::activity_observation::AgentActivityObservation,
    key: &str,
) {
    session
        .apply_action(PlayActivityActionRequest {
            session_id: observation.session_id,
            boundary_id: observation.boundary_id.expect("boundary"),
            expected_state_hash: observation.state_hash,
            action_token: observation
                .legal_actions
                .first()
                .expect("action")
                .token
                .clone(),
            idempotency_key: IdempotencyKey::parse(key).expect("key"),
        })
        .expect("accepted action");
}
