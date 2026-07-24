use starclock_agent_api::{
    activity_action::{AgentActivityActionKind, OfferedActivityAction},
    activity_observation::AgentActivityStatus,
    activity_session::{
        ActivityAgentSession, ActivityAgentSessionFactory, CreateActivitySessionRequest,
        PlayActivityActionRequest,
    },
    error::AgentErrorCode,
    schema::{ActionToken, AgentHash, AgentSchemaRevision, AgentUInt, IdempotencyKey, SessionId},
};

fn create(factory: &ActivityAgentSessionFactory, id: &str) -> ActivityAgentSession {
    factory
        .create(CreateActivitySessionRequest {
            session_id: SessionId::parse(id).unwrap(),
            world: AgentUInt::from_u64(1),
            difficulty_index: AgentUInt::from_u64(0),
            seed: AgentUInt::from_u64(10),
        })
        .unwrap()
}

fn selected(actions: &[OfferedActivityAction]) -> &OfferedActivityAction {
    if let Some(engage) = actions
        .iter()
        .find(|action| action.kind == AgentActivityActionKind::EngageBattle)
    {
        return engage;
    }
    actions
        .iter()
        .max_by(|left, right| {
            let left_priority = left
                .priority
                .as_ref()
                .map_or(0, |value| value.as_str().parse::<i64>().unwrap());
            let right_priority = right
                .priority
                .as_ref()
                .map_or(0, |value| value.as_str().parse::<i64>().unwrap());
            left_priority
                .cmp(&right_priority)
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .unwrap()
}

fn request(session: &ActivityAgentSession, sequence: u64) -> PlayActivityActionRequest {
    let observation = session.observe().unwrap();
    let action = selected(&observation.legal_actions);
    PlayActivityActionRequest {
        schema_revision: AgentSchemaRevision::V1,
        session_id: session.session_id().clone(),
        boundary_id: observation.boundary_id.unwrap(),
        expected_state_hash: observation.state_hash,
        action_token: action.token.clone(),
        idempotency_key: IdempotencyKey::parse(&format!("activity_step_{sequence}")).unwrap(),
    }
}

fn drive_to_terminal(session: &mut ActivityAgentSession) -> u64 {
    let mut external_steps = 0_u64;
    while session.terminal().is_none() {
        assert!(external_steps < 1_000);
        let next = request(session, external_steps);
        session.apply_action(next).unwrap();
        external_steps += 1;
    }
    external_steps
}

#[test]
fn activity_session_exposes_only_tokens_settles_battles_and_round_trips_replay() {
    let factory = ActivityAgentSessionFactory::load_production().unwrap();
    let mut session = create(&factory, "session_activity_loop");
    let initial = session.observe().unwrap();
    assert_eq!(initial.status, AgentActivityStatus::AwaitingAction);
    assert_eq!(initial.legal_actions.len(), 9);
    assert!(
        initial
            .legal_actions
            .iter()
            .all(|action| action.token.as_str().starts_with("u_"))
    );
    let json = serde_json::to_string(&initial).unwrap();
    for private in ["battle_spec", "rng", "generated", "sora"] {
        assert!(!json.contains(private), "projection leaked {private}");
    }

    let first = request(&session, 0);
    let initial_hash = session.state_hash();
    let mut forged = first.clone();
    forged.action_token = ActionToken::parse("u_forged").unwrap();
    assert_eq!(
        session.apply_action(forged).unwrap_err().code,
        AgentErrorCode::InvalidActionToken
    );
    assert_eq!(session.state_hash(), initial_hash);

    let mut stale = first.clone();
    stale.expected_state_hash = AgentHash::from_bytes([0x55; 32]);
    assert_eq!(
        session.apply_action(stale).unwrap_err().code,
        AgentErrorCode::StaleStateHash
    );
    assert_eq!(session.state_hash(), initial_hash);

    let first_response = session.apply_action(first.clone()).unwrap();
    assert_eq!(
        session.apply_action(first).unwrap(),
        first_response,
        "idempotent retry must return byte-equivalent owned data"
    );

    let mut external_steps = 1_u64;
    while session.terminal().is_none() {
        assert!(external_steps < 1_000);
        let next = request(&session, external_steps);
        session.apply_action(next).unwrap();
        external_steps += 1;
    }
    assert_eq!(external_steps, 55);
    assert_eq!(
        session.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Completed)
    );
    assert_eq!(
        session.observe().unwrap().status,
        AgentActivityStatus::Completed
    );

    let replay = session.export_replay().unwrap();
    assert!(replay.complete());
    assert_eq!(replay.action_count().as_str(), "60");
    assert_eq!(replay.bytes().len(), 45_453);
    assert_eq!(
        replay.sha256().as_str(),
        "501b82ae29f573dcfcacb60270b3b59432abfc7c0b2ccd4eeca88b42b957668f"
    );
    assert_eq!(
        replay.action_count().to_u64(),
        session.replay_action_count() as u64
    );
    let verified = session.verify_replay(&factory, replay.bytes()).unwrap();
    assert_eq!(verified.action_count, replay.action_count().clone());
    assert_eq!(verified.final_state_hash, session.state_hash());
    assert_eq!(verified.nested_battles.as_str(), "5");
    assert_eq!(
        verified.final_state_hash.as_str(),
        "f66ad680373b4f21f1857d2f3c2b52c3ecbed95c8f77e316eb443ec77ca7bb00"
    );

    let mut corrupt = replay.bytes().to_vec();
    let last = corrupt.len() - 1;
    corrupt[last] ^= 1;
    assert_eq!(
        session.verify_replay(&factory, &corrupt).unwrap_err().code,
        AgentErrorCode::ReplayDiverged
    );

    session.close();
    assert_eq!(
        session.observe().unwrap().status,
        AgentActivityStatus::Closed
    );
}

#[test]
fn activity_replay_corruption_corpus_is_total_and_live_session_is_inert() {
    const CORPUS_CASES: usize = 3;

    let factory = ActivityAgentSessionFactory::load_production().unwrap();
    let mut session = create(&factory, "session_activity_replay_corpus");
    assert_eq!(drive_to_terminal(&mut session), 55);
    let replay = session.export_replay().unwrap();
    let original_hash = session.state_hash();
    let original_actions = session.replay_action_count();

    for case in 0..CORPUS_CASES {
        let mut malformed = replay.bytes().to_vec();
        match case % 3 {
            0 => malformed.truncate(case * malformed.len() / CORPUS_CASES),
            1 => malformed.push(u8::try_from(case).unwrap()),
            _ => {
                let last = malformed.len() - 1;
                let final_state_byte = last - ((case / 3) % 32);
                malformed[final_state_byte] ^= 1_u8 << (case % 8);
            }
        }
        assert!(
            session.verify_replay(&factory, &malformed).is_err(),
            "malformed replay case {case} unexpectedly verified"
        );
        assert_eq!(session.state_hash(), original_hash);
        assert_eq!(session.replay_action_count(), original_actions);
    }

    let verified = session.verify_replay(&factory, replay.bytes()).unwrap();
    assert_eq!(verified.final_state_hash, original_hash);
}
