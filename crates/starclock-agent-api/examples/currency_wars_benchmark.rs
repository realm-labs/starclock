//! Goal 21 release-mode Currency Wars performance workloads.

use std::{
    hint::black_box,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use starclock_agent_api::{
    activity_observation::{AgentActivityObservation, AgentActivityStatus},
    activity_session::{AgentActivityReplayExport, PlayActivityActionRequest},
    currency_wars_activity_session::{
        AgentCurrencyWarsGambit, CreateCurrencyWarsActivitySessionRequest,
        CurrencyWarsActivityAgentSession, CurrencyWarsActivityAgentSessionFactory,
    },
    schema::{AgentUInt, IdempotencyKey, SessionId},
};

const MATRIX_ITERATIONS: usize = 97;
const WARM_ITERATIONS: usize = 10_000;
const TRIGGER_ITERATIONS: usize = 100;
const CONCURRENT_ITERATIONS: usize = 16;
const INVALID_ITERATIONS: usize = 4_096;
const SEED: u64 = 31_000_501;

#[derive(Serialize)]
struct Report {
    allocation_measurement_authoritative: bool,
    rows: Vec<Row>,
}

#[derive(Serialize)]
struct Row {
    id: &'static str,
    iterations: usize,
    elapsed_ns: u64,
    operations_per_second: u64,
    allocation_count: u64,
    allocation_bytes: u64,
    peak_live_bytes: u64,
    retained_bytes: u64,
    external_actions: u64,
    nested_battles: u64,
    replay_bytes: usize,
    final_digest: String,
}

fn main() {
    assert!(
        std::env::args().len() == 1,
        "currency_wars_benchmark takes no arguments"
    );
    let (catalog, factory) = catalog_load();
    let factory = Arc::new(factory);
    let matrix = factory_start_matrix(&factory);
    let (complete, replay) = complete_run(&factory);
    let replay_row = replay_verify(&factory, &replay);
    let trigger = trigger_heavy(&factory);
    let warm = warm_session_start(&factory);
    let concurrent = concurrent_sessions(&factory);
    let invalid = invalid_replay(&factory);
    println!(
        "{}",
        serde_json::to_string(&Report {
            allocation_measurement_authoritative: false,
            rows: vec![
                catalog, matrix, complete, replay_row, trigger, warm, concurrent, invalid
            ],
        })
        .expect("benchmark report serializes")
    );
}

fn catalog_load() -> (Row, CurrencyWarsActivityAgentSessionFactory) {
    let mut factory = None;
    let start = Instant::now();
    let allocations = measure(|| {
        factory =
            Some(CurrencyWarsActivityAgentSessionFactory::load_production().expect("factory"));
    });
    let factory = factory.expect("measured factory");
    let digest = digest_json(&factory.manifest());
    (
        row(
            "catalog-load-and-lower",
            1,
            start.elapsed(),
            allocations,
            0,
            0,
            0,
            digest,
        ),
        factory,
    )
}

fn factory_start_matrix(factory: &CurrencyWarsActivityAgentSessionFactory) -> Row {
    let mut hash = Sha256::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for ordinal in 0..MATRIX_ITERATIONS {
            let session = create(
                factory,
                &format!("cw_matrix_{ordinal}"),
                SEED + ordinal as u64,
            );
            hash.update(session.state_hash().as_str().as_bytes());
        }
    });
    row(
        "factory-start-all-matrix-entries",
        MATRIX_ITERATIONS,
        start.elapsed(),
        allocations,
        0,
        0,
        0,
        hex(hash.finalize()),
    )
}

fn complete_run(
    factory: &CurrencyWarsActivityAgentSessionFactory,
) -> (Row, AgentActivityReplayExport) {
    let mut completed = None;
    let start = Instant::now();
    let allocations = measure(|| {
        let mut session = create(factory, "cw_complete", SEED);
        let actions = play_to_terminal(&mut session);
        let replay = session.export_replay().expect("terminal replay");
        completed = Some((actions, session.nested_battle_count(), replay));
    });
    let (actions, battles, replay) = completed.expect("measured complete run");
    let digest = replay.sha256().as_str().to_owned();
    (
        row(
            "complete-run",
            1,
            start.elapsed(),
            allocations,
            actions,
            battles as u64,
            replay.bytes().len(),
            digest,
        ),
        replay,
    )
}

fn replay_verify(
    factory: &CurrencyWarsActivityAgentSessionFactory,
    replay: &AgentActivityReplayExport,
) -> Row {
    let mut digest = None;
    let start = Instant::now();
    let allocations = measure(|| {
        digest = Some(
            factory
                .verify_replay(replay.bytes())
                .expect("fresh replay")
                .final_state_hash,
        );
    });
    row(
        "fresh-replay",
        1,
        start.elapsed(),
        allocations,
        replay.action_count().to_u64(),
        7,
        replay.bytes().len(),
        digest.expect("digest").as_str().to_owned(),
    )
}

fn trigger_heavy(factory: &CurrencyWarsActivityAgentSessionFactory) -> Row {
    let mut hash = Sha256::new();
    let mut actions = 0_u64;
    let mut battles = 0_u64;
    let start = Instant::now();
    let allocations = measure(|| {
        let mut session = create(factory, "cw_trigger", SEED);
        while actions < TRIGGER_ITERATIONS as u64 {
            let observation = session.observe().expect("observation");
            if observation.status != AgentActivityStatus::AwaitingAction {
                session = create(factory, &format!("cw_trigger_{actions}"), SEED + actions);
                continue;
            }
            let response = play(&mut session, &observation, actions);
            actions += 1;
            battles += response.settlement.nested_battles.to_u64();
            hash.update(response.observation.state_hash.as_str().as_bytes());
        }
    });
    row(
        "trigger-heavy-investment-bond-battle",
        TRIGGER_ITERATIONS,
        start.elapsed(),
        allocations,
        actions,
        battles,
        0,
        hex(hash.finalize()),
    )
}

fn warm_session_start(factory: &CurrencyWarsActivityAgentSessionFactory) -> Row {
    let mut hash = Sha256::new();
    let start = Instant::now();
    let allocations = measure(|| {
        for ordinal in 0..WARM_ITERATIONS {
            let session = create(factory, &format!("cw_warm_{ordinal}"), SEED);
            hash.update(session.state_hash().as_str().as_bytes());
        }
    });
    row(
        "warm-shared-catalog-session-start",
        WARM_ITERATIONS,
        start.elapsed(),
        allocations,
        0,
        0,
        0,
        hex(hash.finalize()),
    )
}

fn concurrent_sessions(factory: &Arc<CurrencyWarsActivityAgentSessionFactory>) -> Row {
    let start = Instant::now();
    let mut digests = Vec::new();
    let allocations = measure(|| {
        let handles = (0..CONCURRENT_ITERATIONS)
            .map(|ordinal| {
                let factory = Arc::clone(factory);
                thread::spawn(move || {
                    let mut session = create(&factory, &format!("cw_concurrent_{ordinal}"), SEED);
                    let actions = play_to_terminal(&mut session);
                    (ordinal, actions, session.state_hash().as_str().to_owned())
                })
            })
            .collect::<Vec<_>>();
        digests = handles
            .into_iter()
            .map(|handle| handle.join().expect("session"))
            .collect();
    });
    digests.sort_by_key(|value| value.0);
    let actions = digests.iter().map(|value| value.1).sum();
    row(
        "concurrent-shared-catalog-sessions",
        CONCURRENT_ITERATIONS,
        start.elapsed(),
        allocations,
        actions,
        112,
        0,
        digest_text(digests.iter().map(|value| value.2.as_str())),
    )
}

fn invalid_replay(factory: &CurrencyWarsActivityAgentSessionFactory) -> Row {
    let malformed = b"SCRP\0currency-wars";
    let start = Instant::now();
    let allocations = measure(|| {
        for _ in 0..INVALID_ITERATIONS {
            black_box(
                factory
                    .verify_replay(malformed)
                    .expect_err("malformed replay rejected"),
            );
        }
    });
    row(
        "invalid-command-and-replay-corruption",
        INVALID_ITERATIONS,
        start.elapsed(),
        allocations,
        0,
        0,
        0,
        hex(Sha256::digest(malformed)),
    )
}

fn create(
    factory: &CurrencyWarsActivityAgentSessionFactory,
    id: &str,
    seed: u64,
) -> CurrencyWarsActivityAgentSession {
    factory
        .create(CreateCurrencyWarsActivitySessionRequest {
            session_id: SessionId::parse(id).expect("session ID"),
            route_id: AgentUInt::from_u64(801),
            difficulty_id: AgentUInt::from_u64(1),
            gambit: AgentCurrencyWarsGambit::Standard,
            seed: AgentUInt::from_u64(seed),
        })
        .expect("session")
}

fn play_to_terminal(session: &mut CurrencyWarsActivityAgentSession) -> u64 {
    let mut actions = 0;
    loop {
        let observation = session.observe().expect("observation");
        if observation.status != AgentActivityStatus::AwaitingAction {
            return actions;
        }
        play(session, &observation, actions);
        actions += 1;
    }
}

fn play(
    session: &mut CurrencyWarsActivityAgentSession,
    observation: &AgentActivityObservation,
    ordinal: u64,
) -> starclock_agent_api::activity_session::AgentActivityActionResponse {
    let action = observation.legal_actions.first().expect("offered action");
    session
        .apply_action(PlayActivityActionRequest {
            session_id: observation.session_id.clone(),
            boundary_id: observation.boundary_id.clone().expect("boundary"),
            expected_state_hash: observation.state_hash.clone(),
            action_token: action.token.clone(),
            idempotency_key: IdempotencyKey::parse(&format!("cw_action_{ordinal}")).expect("key"),
        })
        .expect("action")
}

fn row(
    id: &'static str,
    iterations: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    actions: u64,
    battles: u64,
    replay_bytes: usize,
    digest: String,
) -> Row {
    let elapsed_ns = u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX);
    Row {
        id,
        iterations,
        elapsed_ns,
        operations_per_second: if elapsed_ns == 0 {
            0
        } else {
            u64::try_from((iterations as u128 * 1_000_000_000) / u128::from(elapsed_ns))
                .unwrap_or(u64::MAX)
        },
        allocation_count: allocations.count_total as u64,
        allocation_bytes: allocations.bytes_total as u64,
        peak_live_bytes: allocations.bytes_max as u64,
        retained_bytes: allocations.bytes_current as u64,
        external_actions: actions,
        nested_battles: battles,
        replay_bytes,
        final_digest: digest,
    }
}

fn digest_json(value: &impl Serialize) -> String {
    hex(Sha256::digest(serde_json::to_vec(value).expect("JSON")))
}
fn digest_text<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let mut hash = Sha256::new();
    for value in values {
        hash.update(value.as_bytes());
    }
    hex(hash.finalize())
}
fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
