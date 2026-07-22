use std::{
    sync::{
        Arc, Barrier,
        atomic::{AtomicU64, Ordering},
    },
    thread,
};

use serde_json::Value;
use starclock_agent_api::{
    action::AgentActionKind,
    error::{AgentError, AgentErrorCode},
    observation::{AgentBattleStatus, VisibilityPolicy},
    schema::{
        ActionToken, AgentHash, AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey,
        ScenarioId, SessionId,
    },
    session::{
        AgentSeedPolicy, AgentSession, AgentSessionFactory, AgentSessionOwner,
        AgentSessionRegistry, CreateSessionRequest, OperationalClock, PlayActionRequest,
        RegistryCreateSessionRequest, SessionIdSource,
    },
};

fn corpus() -> Value {
    serde_json::from_str(include_str!(
        "../../../evidence/agent-control-mcp-v1/security/hardening-corpus.json"
    ))
    .unwrap()
}

fn session(factory: &AgentSessionFactory, id: &str, scenario: &str) -> AgentSession {
    factory
        .create(CreateSessionRequest {
            session_id: SessionId::parse(id).unwrap(),
            scenario_id: ScenarioId::parse(scenario).unwrap(),
            seed: AgentSeedPolicy::ScenarioDefault,
            visibility_policy: VisibilityPolicy::PlayerVisible,
        })
        .unwrap()
}

fn action_request(session: &AgentSession, key: &str) -> PlayActionRequest {
    let observation = session
        .observe(&EventCursor::parse("event_0").unwrap())
        .unwrap();
    let action = observation
        .legal_actions
        .iter()
        .find(|action| action.kind != AgentActionKind::Concede)
        .unwrap();
    PlayActionRequest {
        schema_revision: AgentSchemaRevision::V1,
        session_id: observation.session_id,
        decision_id: observation.decision_id.unwrap(),
        expected_state_hash: observation.state_hash,
        action_token: action.token.clone(),
        idempotency_key: IdempotencyKey::parse(key).unwrap(),
    }
}

fn snapshot(session: &AgentSession) -> (AgentHash, usize, u64) {
    (
        session.state_hash(),
        session.replay_command_count(),
        session.rng_draw_count(),
    )
}

#[test]
fn malformed_request_and_token_corpus_is_total_and_bounded() {
    let corpus = corpus();
    for raw in corpus["malformed_requests"].as_array().unwrap() {
        assert!(
            serde_json::from_str::<PlayActionRequest>(raw.as_str().unwrap()).is_err(),
            "malformed request unexpectedly decoded: {raw}"
        );
    }
    for case in corpus["tokens"].as_array().unwrap() {
        assert_eq!(
            ActionToken::parse(case["value"].as_str().unwrap()).is_ok(),
            case["valid"].as_bool().unwrap(),
            "token corpus mismatch: {case}"
        );
    }
}

#[test]
fn conflicting_idempotency_and_cursor_corpus_never_mutates() {
    let corpus = corpus();
    let factory = AgentSessionFactory::load_production().unwrap();
    for (index, mutation) in corpus["idempotency_mutations"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut session = session(&factory, &format!("session_conflict_{index}"), scenario());
        let request = action_request(&session, &format!("conflict_key_{index}"));
        session.apply_action(request.clone()).unwrap();
        let committed = snapshot(&session);
        let mut conflict = request;
        match mutation["field"].as_str().unwrap() {
            "action_token" => {
                conflict.action_token = ActionToken::parse("different_token").unwrap()
            }
            "expected_state_hash" => {
                conflict.expected_state_hash = AgentHash::from_bytes([0x44; 32])
            }
            "decision_id" => {
                conflict.decision_id = AgentUInt::from_u64(conflict.decision_id.to_u64() + 1)
            }
            "session_id" => conflict.session_id = SessionId::parse("session_different").unwrap(),
            value => panic!("unknown idempotency mutation {value}"),
        }
        let expected = match mutation["error"].as_str().unwrap() {
            "idempotency_conflict" => AgentErrorCode::IdempotencyConflict,
            "session_not_owned" => AgentErrorCode::SessionNotOwned,
            value => panic!("unknown idempotency error {value}"),
        };
        assert_eq!(session.apply_action(conflict).unwrap_err().code, expected);
        assert_eq!(snapshot(&session), committed);
    }

    let session = session(&factory, "session_cursor_corpus", scenario());
    let before = snapshot(&session);
    for case in corpus["cursors"].as_array().unwrap() {
        let parsed = EventCursor::parse(case["value"].as_str().unwrap());
        if case["parse"].as_bool().unwrap() {
            assert_eq!(
                session.observe(&parsed.unwrap()).unwrap_err().code,
                AgentErrorCode::InvalidRequest,
                "cursor corpus mismatch: {case}"
            );
        } else {
            assert!(parsed.is_err(), "cursor unexpectedly parsed: {case}");
        }
        assert_eq!(snapshot(&session), before);
    }
}

#[test]
fn corrupted_replay_corpus_fails_without_touching_live_state() {
    let corpus = corpus();
    let factory = AgentSessionFactory::load_production().unwrap();
    let mut session = session(&factory, "session_replay_corpus", scenario());
    let request = action_request(&session, "replay_corpus_action");
    session.apply_action(request).unwrap();
    let replay = session.export_replay().unwrap().bytes().to_vec();
    let before = snapshot(&session);
    for mutation in corpus["replay_mutations"].as_array().unwrap() {
        let mut corrupted = replay.clone();
        let position = usize::try_from(mutation["position"].as_u64().unwrap()).unwrap();
        match mutation["kind"].as_str().unwrap() {
            "truncate" => corrupted.truncate(position * corrupted.len() / 100),
            "trailing" => corrupted.push(0),
            "flip_percent" => {
                let index = (position * (corrupted.len() - 1) / 100).min(corrupted.len() - 1);
                corrupted[index] ^= 0x80;
            }
            value => panic!("unknown replay mutation {value}"),
        }
        assert!(
            session.verify_replay(&corrupted).is_err(),
            "mutation passed: {mutation}"
        );
        assert_eq!(snapshot(&session), before);
    }
}

#[test]
fn every_settlement_corpus_path_stays_within_all_three_budgets() {
    let corpus = corpus();
    let settlement = &corpus["settlement"];
    let factory = AgentSessionFactory::load_production().unwrap();
    for (scenario_index, scenario) in settlement["scenario_ids"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let mut session = session(
            &factory,
            &format!("session_settlement_{scenario_index}"),
            scenario.as_str().unwrap(),
        );
        let mut steps = 0_u64;
        loop {
            let observation = session
                .observe(&EventCursor::parse("event_0").unwrap())
                .unwrap();
            if observation.status != AgentBattleStatus::AwaitingPlayer {
                assert!(session.phase().is_terminal());
                break;
            }
            let action = observation
                .legal_actions
                .iter()
                .find(|action| action.kind == AgentActionKind::UseAbility)
                .or_else(|| {
                    observation
                        .legal_actions
                        .iter()
                        .find(|action| action.kind == AgentActionKind::PassInterrupt)
                })
                .unwrap();
            let response = session
                .apply_action(PlayActionRequest {
                    schema_revision: AgentSchemaRevision::V1,
                    session_id: observation.session_id,
                    decision_id: observation.decision_id.unwrap(),
                    expected_state_hash: observation.state_hash,
                    action_token: action.token.clone(),
                    idempotency_key: IdempotencyKey::parse(&format!(
                        "settlement_{scenario_index}_{steps}"
                    ))
                    .unwrap(),
                })
                .unwrap();
            assert!(
                response.settlement.accepted_commands.to_u64()
                    <= settlement["maximum_accepted_commands"].as_u64().unwrap()
            );
            assert!(
                response.settlement.emitted_events.to_u64()
                    <= settlement["maximum_emitted_events"].as_u64().unwrap()
            );
            assert!(
                response.settlement.resolver_operations.to_u64()
                    <= settlement["maximum_resolver_operations"].as_u64().unwrap()
            );
            steps += 1;
            assert!(steps <= settlement["maximum_external_steps"].as_u64().unwrap());
        }
    }
}

#[derive(Default)]
struct CorpusClock;

impl OperationalClock for CorpusClock {
    fn now_seconds(&self) -> u64 {
        0
    }
}

#[derive(Default)]
struct CorpusIds(AtomicU64);

impl SessionIdSource for CorpusIds {
    fn next_session_id(&self) -> Result<SessionId, AgentError> {
        Ok(SessionId::parse(&format!(
            "session_race_corpus_{}",
            self.0.fetch_add(1, Ordering::Relaxed)
        ))
        .expect("corpus session ID is canonical"))
    }
}

#[test]
fn seeded_race_corpus_allows_exactly_one_commit_per_round() {
    let corpus = corpus();
    let rounds = corpus["races"]["rounds"].as_u64().unwrap();
    assert_eq!(corpus["races"]["contenders"], 2);
    let registry = AgentSessionRegistry::new(
        AgentSessionFactory::load_production().unwrap(),
        Arc::new(CorpusClock),
        Arc::new(CorpusIds::default()),
    );
    let owner = AgentSessionOwner::new("corpus_tenant", "corpus_principal").unwrap();
    for round in 0..rounds {
        let observation = registry
            .create(
                &owner,
                RegistryCreateSessionRequest {
                    scenario_id: ScenarioId::parse(scenario()).unwrap(),
                    seed: AgentSeedPolicy::ScenarioDefault,
                    visibility_policy: VisibilityPolicy::PlayerVisible,
                },
            )
            .unwrap();
        let action = observation
            .legal_actions
            .iter()
            .find(|action| action.kind != AgentActionKind::Concede)
            .unwrap();
        let base = PlayActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: observation.session_id.clone(),
            decision_id: observation.decision_id.clone().unwrap(),
            expected_state_hash: observation.state_hash,
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse(&format!("race_{round}_0")).unwrap(),
        };
        let barrier = Arc::new(Barrier::new(3));
        let mut handles = Vec::new();
        for contender in 0..2 {
            let registry = registry.clone();
            let owner = owner.clone();
            let barrier = Arc::clone(&barrier);
            let mut request = base.clone();
            request.idempotency_key =
                IdempotencyKey::parse(&format!("race_{round}_{contender}")).unwrap();
            handles.push(thread::spawn(move || {
                barrier.wait();
                registry.apply_action(&owner, request)
            }));
        }
        barrier.wait();
        let results = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter_map(|result| result.as_ref().err())
                .filter(|error| error.code == AgentErrorCode::StaleDecision)
                .count(),
            1
        );
        registry.close(&owner, &observation.session_id).unwrap();
    }
}

fn scenario() -> &'static str {
    "scenario.standard-v1.basic-single-wave"
}
