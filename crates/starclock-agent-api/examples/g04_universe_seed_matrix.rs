use std::{collections::BTreeSet, thread};

use serde::Serialize;
use sha2::{Digest, Sha256};
use starclock_agent_api::{
    activity_action::{AgentActivityActionKind, OfferedActivityAction},
    activity_observation::AgentActivityObservation,
    activity_session::{
        ActivityAgentSessionFactory, CreateActivitySessionRequest, PlayActivityActionRequest,
    },
    error::AgentErrorCode,
    schema::{AgentSchemaRevision, AgentUInt, IdempotencyKey, SessionId},
};
use starclock_mode_universe::nested_battle_executor::UNIVERSE_NESTED_BATTLE_EXECUTOR_REVISION;

const MATRIX_REVISION: &str = "standard-universe-seed-matrix-v1";
const FIRST_SEED: u64 = 100_000;
const WORKERS: usize = 8;
const MAX_EXTERNAL_ACTIONS: u64 = 1_000;

#[derive(Clone)]
struct Entry {
    ordinal: usize,
    world: u64,
    world_key: String,
    difficulty_index: u64,
    path_index: usize,
    seed: u64,
}

#[derive(Serialize)]
struct MatrixEvidence {
    schema_revision: &'static str,
    result: &'static str,
    executor_revision: &'static str,
    coverage: Coverage,
    runs: Vec<RunEvidence>,
    failures: Vec<FailureEvidence>,
}

#[derive(Serialize)]
struct Coverage {
    worlds: usize,
    difficulties: usize,
    distinct_path_options: usize,
    complete_runs: usize,
    first_seed: u64,
    seed_rule: &'static str,
}

#[derive(Serialize)]
struct RunEvidence {
    ordinal: usize,
    world: u64,
    world_key: String,
    difficulty_index: u64,
    path_option_id: String,
    seed: u64,
    external_actions: u64,
    replay_actions: String,
    nested_battles: String,
    encoded_bytes: usize,
    final_state_hash: String,
    replay_sha256: String,
    decision_kinds: Vec<String>,
    action_kinds: Vec<String>,
    terminal: &'static str,
}

#[derive(Serialize)]
struct FailureEvidence {
    case: &'static str,
    code: AgentErrorCode,
    committed: bool,
}

fn main() {
    let factory =
        ActivityAgentSessionFactory::load_production().expect("production Activity factory");
    let manifest = factory.manifest();
    let mut entries = Vec::new();
    for world in &manifest.worlds {
        for difficulty_index in 0..world.difficulty_count.to_u64() {
            let ordinal = entries.len();
            entries.push(Entry {
                ordinal,
                world: world.world.to_u64(),
                world_key: world.stable_key.to_string(),
                difficulty_index,
                path_index: ordinal % 9,
                seed: FIRST_SEED + ordinal as u64,
            });
        }
    }
    assert_eq!(entries.len(), 33);

    let mut runs = Vec::with_capacity(entries.len());
    for chunk in entries.chunks(WORKERS) {
        let handles = chunk
            .iter()
            .cloned()
            .map(|entry| {
                let factory = factory.clone();
                thread::spawn(move || run(&factory, entry))
            })
            .collect::<Vec<_>>();
        runs.extend(
            handles
                .into_iter()
                .map(|handle| handle.join().expect("matrix worker")),
        );
    }
    runs.sort_by_key(|run| run.ordinal);
    let distinct_paths = runs
        .iter()
        .map(|run| run.path_option_id.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    assert_eq!(distinct_paths, 9);

    let evidence = MatrixEvidence {
        schema_revision: MATRIX_REVISION,
        result: "all-constructible-difficulties-complete-with-nine-path-options-covered",
        executor_revision: UNIVERSE_NESTED_BATTLE_EXECUTOR_REVISION,
        coverage: Coverage {
            worlds: manifest.worlds.len(),
            difficulties: entries.len(),
            distinct_path_options: distinct_paths,
            complete_runs: runs.len(),
            first_seed: FIRST_SEED,
            seed_rule: "seed = 100000 + canonical world/difficulty ordinal",
        },
        runs,
        failures: failure_evidence(&factory),
    };
    println!(
        "{}",
        serde_json::to_string(&evidence).expect("matrix evidence serializes")
    );
}

fn run(factory: &ActivityAgentSessionFactory, entry: Entry) -> RunEvidence {
    let mut session = factory
        .create(CreateActivitySessionRequest {
            session_id: SessionId::parse(&format!("matrix_{}", entry.ordinal)).unwrap(),
            world: AgentUInt::from_u64(entry.world),
            difficulty_index: AgentUInt::from_u64(entry.difficulty_index),
            seed: AgentUInt::from_u64(entry.seed),
        })
        .expect("matrix Activity constructs");
    let mut decision_kinds = BTreeSet::new();
    let mut action_kinds = BTreeSet::new();
    let mut path_option_id = None;
    let mut external_actions = 0_u64;
    while session.terminal().is_none() {
        assert!(external_actions < MAX_EXTERNAL_ACTIONS);
        let observation = session.observe().expect("matrix observation");
        if let Some(kind) = observation.decision_kind {
            decision_kinds.insert(enum_name(kind));
        }
        let action = if external_actions == 0 {
            let mut offered = observation.legal_actions.iter().collect::<Vec<_>>();
            offered.sort_by_key(|action| action.option_id.to_u64());
            let action = offered[entry.path_index];
            path_option_id = Some(action.option_id.as_str().to_owned());
            action
        } else {
            selected(&observation)
        };
        action_kinds.insert(enum_name(action.kind));
        let action_token = action.token.clone();
        let request = PlayActivityActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session.session_id().clone(),
            boundary_id: observation.boundary_id.clone().expect("external boundary"),
            expected_state_hash: observation.state_hash.clone(),
            action_token,
            idempotency_key: IdempotencyKey::parse(&format!(
                "matrix_{}_{}",
                entry.ordinal, external_actions
            ))
            .unwrap(),
        };
        session
            .apply_action(request)
            .expect("offered action applies");
        external_actions += 1;
    }
    let replay = session.export_replay().expect("complete replay exports");
    let verified = session
        .verify_replay(factory, replay.bytes())
        .expect("complete replay verifies fresh");
    assert_eq!(verified.final_state_hash, session.state_hash());
    assert_eq!(
        session.terminal(),
        Some(starclock_activity::ActivityTerminalOutcome::Completed)
    );
    RunEvidence {
        ordinal: entry.ordinal,
        world: entry.world,
        world_key: entry.world_key,
        difficulty_index: entry.difficulty_index,
        path_option_id: path_option_id.expect("path selected"),
        seed: entry.seed,
        external_actions,
        replay_actions: verified.action_count.as_str().into(),
        nested_battles: verified.nested_battles.as_str().into(),
        encoded_bytes: replay.bytes().len(),
        final_state_hash: verified.final_state_hash.as_str().into(),
        replay_sha256: hex(&Sha256::digest(replay.bytes())),
        decision_kinds: decision_kinds.into_iter().collect(),
        action_kinds: action_kinds.into_iter().collect(),
        terminal: "completed",
    }
}

fn selected(observation: &AgentActivityObservation) -> &OfferedActivityAction {
    if let Some(engage) = observation
        .legal_actions
        .iter()
        .find(|action| action.kind == AgentActivityActionKind::EngageBattle)
    {
        return engage;
    }
    observation
        .legal_actions
        .iter()
        .max_by(|left, right| {
            priority(left)
                .cmp(&priority(right))
                .then_with(|| right.option_id.to_u64().cmp(&left.option_id.to_u64()))
        })
        .expect("nonterminal Activity offers an action")
}

fn priority(action: &OfferedActivityAction) -> i64 {
    action.priority.as_ref().map_or(0, |value| {
        value.as_str().parse().expect("priority is exact")
    })
}

fn failure_evidence(factory: &ActivityAgentSessionFactory) -> Vec<FailureEvidence> {
    [
        ("unknown_world", 0, 0, 1),
        ("difficulty_out_of_range", 1, 99, 1),
        ("seed_overflow", 1, 0, u64::MAX),
    ]
    .into_iter()
    .map(|(case, world, difficulty, seed)| {
        let error = match factory.create(CreateActivitySessionRequest {
            session_id: SessionId::parse(&format!("failure_{case}")).unwrap(),
            world: AgentUInt::from_u64(world),
            difficulty_index: AgentUInt::from_u64(difficulty),
            seed: AgentUInt::from_u64(seed),
        }) {
            Ok(_) => panic!("invalid entry must fail before session creation"),
            Err(error) => error,
        };
        assert_eq!(error.code, AgentErrorCode::InvalidRequest);
        FailureEvidence {
            case,
            code: error.code,
            committed: error.committed,
        }
    })
    .collect()
}

fn enum_name(value: impl Serialize) -> String {
    serde_json::to_value(value)
        .expect("enum serializes")
        .as_str()
        .expect("enum is a string")
        .to_owned()
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(DIGITS[usize::from(byte >> 4)]));
        encoded.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    encoded
}
