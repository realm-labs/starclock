use super::*;
use crate::observation::VisibilityPolicy;
use starclock_data::standard_v1::SCENARIOS;

fn request(scenario: &str, seed: AgentSeedPolicy) -> CreateSessionRequest {
    CreateSessionRequest {
        session_id: SessionId::parse("session_test").unwrap(),
        scenario_id: ScenarioId::parse(scenario).unwrap(),
        seed,
        visibility_policy: VisibilityPolicy::PlayerVisible,
    }
}

fn play_request(session: &AgentSession, token: ActionToken, key: &str) -> PlayActionRequest {
    PlayActionRequest {
        schema_revision: AgentSchemaRevision::V1,
        session_id: session.session_id().clone(),
        decision_id: session.offered.as_ref().unwrap().decision_id(),
        expected_state_hash: session.state_hash(),
        action_token: token,
        idempotency_key: IdempotencyKey::parse(key).unwrap(),
    }
}

#[test]
fn factory_creates_only_frozen_scenarios_and_settles_internal_start() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let default_seeds = [104_729, 419_431, 314_159, 524_287, 209_759, 629_137];
    for ((scenario, _, encounter), expected_seed) in SCENARIOS.into_iter().zip(default_seeds) {
        let session = factory
            .create(request(scenario, AgentSeedPolicy::ScenarioDefault))
            .unwrap();
        assert_eq!(session.scenario_id().as_str(), scenario);
        assert_eq!(session.encounter().get(), encounter);
        assert_eq!(session.phase(), BattlePhase::AwaitingCommand);
        assert_eq!(session.replay_command_count(), 1);
        assert_eq!(session.master_seed().to_u64(), expected_seed);
        assert_eq!(session.visibility_policy(), VisibilityPolicy::PlayerVisible);
        assert!(!session.offered_actions().is_empty());
        assert_eq!(
            session.controller_records()[0].controller,
            AgentControllerKind::SystemAutomatic
        );
    }
}

#[test]
fn factory_lists_exact_frozen_scenario_identities_and_default_seeds() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let summaries = factory.list_scenarios().unwrap();
    let default_seeds = [104_729, 419_431, 314_159, 524_287, 209_759, 629_137];

    assert_eq!(summaries.len(), SCENARIOS.len());
    for ((summary, (scenario, definition, encounter)), expected_seed) in
        summaries.iter().zip(SCENARIOS).zip(default_seeds)
    {
        assert_eq!(summary.scenario_id.as_str(), scenario);
        assert_eq!(
            summary.scenario_definition_id.to_u64(),
            u64::from(definition)
        );
        assert_eq!(
            summary.encounter_definition_id.to_u64(),
            u64::from(encounter)
        );
        assert_eq!(summary.default_seed.to_u64(), expected_seed);
    }
}

#[test]
fn explicit_seed_is_exact_reproducible_and_operational_identity_is_inert() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let scenario = SCENARIOS[0].0;
    let mut first_request = request(scenario, AgentSeedPolicy::Explicit(AgentUInt::from_u64(7)));
    first_request.session_id = SessionId::parse("session_first").unwrap();
    let mut second_request = first_request.clone();
    second_request.session_id = SessionId::parse("session_second").unwrap();
    let first = factory.create(first_request).unwrap();
    let second = factory.create(second_request).unwrap();
    assert_eq!(first.master_seed().as_str(), "7");
    assert_eq!(first.state_hash(), second.state_hash());
    assert_eq!(first.spec_digest(), second.spec_digest());
}

#[test]
fn unknown_scenario_and_unauthorized_debug_fail_before_session_creation() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let unknown = factory
        .create(request(
            "scenario.standard-v1.not-authored",
            AgentSeedPolicy::ScenarioDefault,
        ))
        .err()
        .expect("unknown scenario must be rejected");
    assert_eq!(unknown.code, AgentErrorCode::ConfigurationRejected);

    let mut debug = request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault);
    debug.visibility_policy = VisibilityPolicy::OmniscientDebug;
    let unauthorized = factory
        .create(debug)
        .err()
        .expect("debug creation must require separate authorization");
    assert_eq!(unauthorized.code, AgentErrorCode::UnauthorizedPolicy);
}

#[test]
fn external_action_settles_and_records_every_controller_boundary() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[4].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let token = session
        .offered_actions()
        .iter()
        .find(|action| action.kind != crate::action::AgentActionKind::Concede)
        .unwrap()
        .token
        .clone();
    let request = play_request(&session, token, "external_action_1");
    let before = session.replay_command_count();
    let controller_before = session.controller_records().len();
    let response = session.apply_action(request).unwrap();
    assert!(response.settlement.accepted_commands.to_u64() >= 1);
    assert!(response.settlement.emitted_events.to_u64() >= 1);
    assert_eq!(
        session.controller_records()[controller_before].controller,
        AgentControllerKind::ExternalPlayer
    );
    assert_eq!(
        session.replay_command_count() - before,
        usize::try_from(response.settlement.accepted_commands.to_u64()).unwrap()
    );
    assert!(
        session
            .controller_records()
            .iter()
            .enumerate()
            .all(|(index, record)| {
                record.sequence.to_u64() == u64::try_from(index + 1).unwrap()
            })
    );
    assert!(session.phase().is_terminal() || !session.offered_actions().is_empty());
}

#[test]
fn response_loss_retry_returns_identical_bytes_without_a_second_commit() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let token = session
        .offered_actions()
        .iter()
        .find(|action| action.kind != crate::action::AgentActionKind::Concede)
        .unwrap()
        .token
        .clone();
    let request = play_request(&session, token, "lost_response_1");
    let first = session.apply_action(request.clone()).unwrap();
    let first_bytes = serde_json::to_vec(&first).unwrap();
    let committed_snapshot = (
        session.state_hash(),
        session.replay_command_count(),
        session.rng_draw_count(),
        session.controller_records().len(),
    );

    let retry = session.apply_action(request).unwrap();
    assert_eq!(serde_json::to_vec(&retry).unwrap(), first_bytes);
    assert_eq!(
        (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
            session.controller_records().len(),
        ),
        committed_snapshot
    );
    assert_eq!(session.idempotency.len(), 1);
}

#[test]
fn stale_forged_conflicting_and_racing_equivalent_requests_are_inert() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let token = session.offered_actions()[0].token.clone();
    let mut stale_hash = play_request(&session, token.clone(), "stale_hash");
    stale_hash.expected_state_hash = AgentHash::from_bytes([0x77; 32]);
    let before = (
        session.state_hash(),
        session.replay_command_count(),
        session.rng_draw_count(),
    );
    assert_eq!(
        session.apply_action(stale_hash).unwrap_err().code,
        AgentErrorCode::StaleStateHash
    );

    let mut forged = play_request(&session, token.clone(), "forged");
    forged.action_token = ActionToken::parse("a_forged").unwrap();
    assert_eq!(
        session.apply_action(forged).unwrap_err().code,
        AgentErrorCode::InvalidActionToken
    );
    assert_eq!(
        (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
        ),
        before
    );

    let committed_request = play_request(&session, token, "one_commit");
    let mut racing_request = committed_request.clone();
    racing_request.idempotency_key = IdempotencyKey::parse("racing_loser").unwrap();
    session.apply_action(committed_request.clone()).unwrap();
    let after_commit = (
        session.state_hash(),
        session.replay_command_count(),
        session.rng_draw_count(),
    );
    let mut conflict = committed_request;
    conflict.action_token = ActionToken::parse("a_conflict").unwrap();
    assert_eq!(
        session.apply_action(conflict).unwrap_err().code,
        AgentErrorCode::IdempotencyConflict
    );
    assert_eq!(
        session.apply_action(racing_request).unwrap_err().code,
        AgentErrorCode::StaleDecision
    );
    assert_eq!(
        (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
        ),
        after_commit
    );
}

#[test]
fn retained_events_page_exclusively_and_reject_expired_or_future_cursors() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    session.replay.events.clear();
    for id in 1..=u64::try_from(MAX_RETAINED_EVENT_SUMMARIES + 1).unwrap() {
        session.replay.retain_event(AgentEventSummary {
            event_id: AgentUInt::from_u64(id),
            kind: "test".into(),
            summary: "Retained test event.".into(),
            root_command_id: AgentUInt::from_u64(1),
        });
    }
    let expired = EventCursor::parse("event_0").unwrap();
    assert_eq!(
        session.observe(&expired).unwrap_err().code,
        AgentErrorCode::EventCursorExpired
    );
    let first_retained = session
        .observe(&EventCursor::parse("event_1").unwrap())
        .unwrap();
    assert_eq!(
        first_retained.events.len(),
        crate::observation::MAX_EVENTS_PER_PAGE
    );
    assert_eq!(first_retained.events[0].event_id.as_str(), "2");
    assert!(first_retained.events_truncated);
    assert_eq!(first_retained.event_cursor.as_str(), "event_257");
    assert_eq!(
        session.replay.trace.len(),
        1,
        "summary eviction cannot erase replay facts"
    );
    assert_eq!(
        session
            .observe(&EventCursor::parse("event_999999").unwrap())
            .unwrap_err()
            .code,
        AgentErrorCode::InvalidRequest
    );
    assert_eq!(
        session
            .observe(&EventCursor::parse("cursor_wrong_family").unwrap())
            .unwrap_err()
            .code,
        AgentErrorCode::InvalidRequest
    );
}

#[test]
fn terminal_action_returns_terminal_observation_and_complete_trace() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let mut preparation = 0;
    while !session
        .offered_actions()
        .iter()
        .any(|action| action.kind == crate::action::AgentActionKind::Concede)
    {
        let token = session.offered_actions()[0].token.clone();
        let request = play_request(&session, token, &format!("prepare_{preparation}"));
        session.apply_action(request).unwrap();
        preparation += 1;
        assert!(preparation < 8);
    }
    let concede = session
        .offered_actions()
        .iter()
        .find(|action| action.kind == crate::action::AgentActionKind::Concede)
        .unwrap()
        .token
        .clone();
    let trace_before = session.replay.trace.len();
    let request = play_request(&session, concede, "terminal_concede");
    let response = session.apply_action(request).unwrap();
    assert_eq!(response.observation.status, AgentBattleStatus::Lost);
    assert_eq!(response.observation.decision_id, None);
    assert!(response.observation.legal_actions.is_empty());
    assert!(!response.observation.events.is_empty());
    assert_eq!(
        response.observation.event_cursor.as_str(),
        session.replay.latest_cursor().as_str()
    );
    assert_eq!(session.replay.trace.len(), trace_before + 1);
    assert_eq!(session.replay.controllers.len(), session.replay.trace.len());
}

#[test]
fn canonical_replay_round_trips_from_a_fresh_battle() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let token = session
        .offered_actions()
        .iter()
        .find(|action| action.kind != crate::action::AgentActionKind::Concede)
        .unwrap()
        .token
        .clone();
    let action = play_request(&session, token, "replay_action_1");
    session.apply_action(action).unwrap();

    let export = session.export_replay().unwrap();
    assert!(!export.bytes().is_empty());
    assert_eq!(export.diagnostics(), session.controller_records());
    assert_eq!(export.diagnostics().len(), session.replay_command_count());
    assert_eq!(
        export.sha256(),
        &AgentHash::from_bytes(Sha256::digest(export.bytes()).into())
    );
    assert!(
        export
            .diagnostics()
            .iter()
            .any(|record| record.controller == AgentControllerKind::SystemAutomatic)
    );
    assert!(
        export
            .diagnostics()
            .iter()
            .any(|record| record.controller == AgentControllerKind::ExternalPlayer)
    );

    let live_snapshot = (
        session.state_hash(),
        session.replay_command_count(),
        session.rng_draw_count(),
    );
    let verification = session.verify_replay(export.bytes()).unwrap();
    assert_eq!(
        verification.command_count.to_u64(),
        u64::try_from(session.replay_command_count()).unwrap()
    );
    assert_eq!(verification.final_state_hash, session.state_hash());
    assert_eq!(verification.phase, replay_phase(session.phase()).unwrap());
    assert_eq!(
        (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
        ),
        live_snapshot
    );
}

#[test]
fn replay_corruption_diverges_without_mutating_the_live_session() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let session = factory
        .create(request(SCENARIOS[1].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let mut bytes = session.export_replay().unwrap().bytes().to_vec();
    let controller_offset = bytes
        .windows(AGENT_REPLAY_CONTROLLER_REVISION.len())
        .position(|window| window == AGENT_REPLAY_CONTROLLER_REVISION.as_bytes())
        .unwrap();
    bytes[controller_offset] ^= 0x01;
    let before = (
        session.state_hash(),
        session.replay_command_count(),
        session.rng_draw_count(),
    );

    assert_eq!(
        session.verify_replay(&bytes).unwrap_err().code,
        AgentErrorCode::ReplayDiverged
    );
    assert_eq!(
        (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
        ),
        before
    );
}

#[test]
fn diagnostic_attribution_cannot_change_canonical_replay_bytes() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = factory
        .create(request(SCENARIOS[2].0, AgentSeedPolicy::ScenarioDefault))
        .unwrap();
    let before = session.export_replay().unwrap();
    session.replay.controllers.clear();
    let after = session.export_replay().unwrap();

    assert_eq!(before.bytes(), after.bytes());
    assert_eq!(before.sha256(), after.sha256());
    assert!(!before.diagnostics().is_empty());
    assert!(after.diagnostics().is_empty());
}

#[test]
fn operational_session_identity_is_absent_from_canonical_replay() {
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut first_request = request(
        SCENARIOS[3].0,
        AgentSeedPolicy::Explicit(AgentUInt::from_u64(91)),
    );
    first_request.session_id = SessionId::parse("session_replay_first").unwrap();
    let mut second_request = first_request.clone();
    second_request.session_id = SessionId::parse("session_replay_second").unwrap();
    let first = factory.create(first_request).unwrap();
    let second = factory.create(second_request).unwrap();

    let first_export = first.export_replay().unwrap();
    let second_export = second.export_replay().unwrap();
    assert_eq!(first_export.bytes(), second_export.bytes());
    assert_eq!(first_export.sha256(), second_export.sha256());
    assert_eq!(first_export.diagnostics(), second_export.diagnostics());
}
