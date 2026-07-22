//! Release-mode Streamable HTTP adapter and multi-session memory harness.

use std::{
    cell::Cell,
    hint::black_box,
    sync::Arc,
    time::{Duration, Instant},
};

use allocation_counter::{AllocationInfo, measure};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{HeaderValue, Method, Request, header::CONTENT_TYPE},
};
use serde_json::{Value, json};
use starclock_mcp::{
    authorization::{
        AccessTokenSignatureVerifier, AuthorizationClock, AuthorizationPolicy, SUPPORTED_SCOPES,
        SignatureVerificationError, SignedTokenClaims,
    },
    http::{
        LoopbackHttpConfig, MAX_HTTP_RESPONSE_BYTES, MCP_HTTP_PATH,
        PROTECTED_RESOURCE_METADATA_PATH, authorized_loopback_router,
    },
};
use tower::ServiceExt;

const WORKLOAD_REVISION: &str = "g02-mcp-http-adapter-v1";
const AUTHORITY: &str = "127.0.0.1:43125";
const ORIGIN: &str = "http://127.0.0.1:43125";
const TOKEN: &str = "benchmark-tenant:benchmark-principal";
const SCENARIO: &str = "scenario.standard-v1.basic-single-wave";
const INITIAL_HASH: &str = "b9d33065f40e044f3326921cd7b3dca7e3341d004dcad8818c4b7265d0440292";
const FIRST_ACTION_HASH: &str = "956e35db53515806c3e5c2a96da078880462b190144094c73e6f313cffa8103a";
const OBSERVATIONS: usize = 256;
const SESSIONS: usize = 16;

fn main() {
    assert!(
        std::env::args().len() == 1,
        "g02_http_benchmark takes no arguments"
    );
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("benchmark runtime builds");
    let rows = [
        measure_observe(&runtime),
        measure_actions(&runtime),
        measure_resident_sessions(&runtime),
    ];
    println!(
        "{{\"schema_revision\":\"starclock.mcp-http-benchmark-report.v1\",\"workload_revision\":\"{}\",\"rows\":[{}]}}",
        WORKLOAD_REVISION,
        rows.iter().map(Row::json).collect::<Vec<_>>().join(",")
    );
}

struct Row {
    id: &'static str,
    operations: usize,
    sessions: usize,
    elapsed: Duration,
    allocations: AllocationInfo,
    payload_bytes: usize,
    final_hash: &'static str,
}

impl Row {
    fn json(&self) -> String {
        let elapsed_ns = u64::try_from(self.elapsed.as_nanos()).unwrap_or(u64::MAX);
        let operations = u64::try_from(self.operations).expect("operation count fits u64");
        let sessions = u64::try_from(self.sessions).expect("session count fits u64");
        let throughput = operations
            .saturating_mul(1_000_000_000)
            .checked_div(elapsed_ns)
            .unwrap_or(0);
        let latency = elapsed_ns.checked_div(operations).unwrap_or(0);
        let peak = self.allocations.bytes_max;
        let retained = nonnegative(self.allocations.bytes_current);
        format!(
            concat!(
                "{{\"id\":\"{}\",\"operations\":{},\"sessions\":{},",
                "\"elapsed_ns\":{},\"latency_ns_per_operation\":{},",
                "\"operations_per_second\":{},\"allocation_count\":{},",
                "\"allocation_bytes\":{},\"peak_live_bytes\":{},",
                "\"retained_bytes\":{},\"peak_live_bytes_per_session\":{},",
                "\"retained_bytes_per_session\":{},\"payload_bytes\":{},",
                "\"final_hash\":\"{}\"}}"
            ),
            self.id,
            operations,
            sessions,
            elapsed_ns,
            latency,
            throughput,
            self.allocations.count_total,
            self.allocations.bytes_total,
            peak,
            retained,
            divide_ceil(peak, sessions),
            divide_ceil(retained, sessions),
            self.payload_bytes,
            self.final_hash,
        )
    }
}

fn measure_observe(runtime: &tokio::runtime::Runtime) -> Row {
    let mut client = runtime.block_on(BenchClient::new());
    let created = runtime.block_on(client.tool(
        "starclock_create_battle",
        json!({"schema_revision":"agent-api-v1", "scenario_id":SCENARIO}),
    ));
    let session_id = created["observation"]["session_id"]
        .as_str()
        .unwrap()
        .to_owned();
    let payload = Cell::new(0_usize);
    let start = Instant::now();
    let allocations = measure(|| {
        runtime.block_on(async {
            for _ in 0..OBSERVATIONS {
                let (observed, bytes) = client
                    .tool_measured(
                        "starclock_observe_battle",
                        json!({"schema_revision":"agent-api-v1", "session_id":session_id}),
                    )
                    .await;
                assert_eq!(observed["observation"]["state_hash"], INITIAL_HASH);
                payload.set(payload.get().saturating_add(bytes));
                black_box(observed);
            }
        });
    });
    Row {
        id: "http-observe-256-v1",
        operations: OBSERVATIONS,
        sessions: 1,
        elapsed: start.elapsed(),
        allocations,
        payload_bytes: payload.get(),
        final_hash: INITIAL_HASH,
    }
}

fn measure_actions(runtime: &tokio::runtime::Runtime) -> Row {
    let mut client = runtime.block_on(BenchClient::new());
    let mut arguments = Vec::with_capacity(SESSIONS);
    for index in 0..SESSIONS {
        let created = runtime.block_on(client.tool(
            "starclock_create_battle",
            json!({"schema_revision":"agent-api-v1", "scenario_id":SCENARIO}),
        ));
        let observation = &created["observation"];
        let action = observation["legal_actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|action| action["kind"] == "use_ability")
            .or_else(|| {
                observation["legal_actions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|action| action["kind"] == "pass_interrupt")
            })
            .unwrap();
        arguments.push(json!({
            "schema_revision":"agent-api-v1",
            "session_id":observation["session_id"],
            "decision_id":observation["decision_id"],
            "expected_state_hash":observation["state_hash"],
            "action_token":action["token"],
            "idempotency_key":format!("benchmark_action_{index}")
        }));
    }
    let payload = Cell::new(0_usize);
    let start = Instant::now();
    let allocations = measure(|| {
        runtime.block_on(async {
            for input in &arguments {
                let (played, bytes) = client
                    .tool_measured("starclock_play_action", input.clone())
                    .await;
                assert_eq!(
                    played["response"]["observation"]["state_hash"],
                    FIRST_ACTION_HASH
                );
                payload.set(payload.get().saturating_add(bytes));
                black_box(played);
            }
        });
    });
    Row {
        id: "http-action-16-v1",
        operations: SESSIONS,
        sessions: SESSIONS,
        elapsed: start.elapsed(),
        allocations,
        payload_bytes: payload.get(),
        final_hash: FIRST_ACTION_HASH,
    }
}

fn measure_resident_sessions(runtime: &tokio::runtime::Runtime) -> Row {
    let mut client = runtime.block_on(BenchClient::new());
    let payload = Cell::new(0_usize);
    let start = Instant::now();
    let allocations = measure(|| {
        runtime.block_on(async {
            for _ in 0..SESSIONS {
                let (created, bytes) = client
                    .tool_measured(
                        "starclock_create_battle",
                        json!({"schema_revision":"agent-api-v1", "scenario_id":SCENARIO}),
                    )
                    .await;
                assert_eq!(created["observation"]["state_hash"], INITIAL_HASH);
                payload.set(payload.get().saturating_add(bytes));
                black_box(created);
            }
        });
    });
    Row {
        id: "http-resident-sessions-16-v1",
        operations: SESSIONS,
        sessions: SESSIONS,
        elapsed: start.elapsed(),
        allocations,
        payload_bytes: payload.get(),
        final_hash: INITIAL_HASH,
    }
}

struct BenchClient {
    router: Router,
    transport_session: HeaderValue,
    next_id: u64,
}

impl BenchClient {
    async fn new() -> Self {
        let router = router();
        let response = router
            .clone()
            .oneshot(request(
                json!({
                    "jsonrpc":"2.0", "id":1, "method":"initialize",
                    "params":{
                        "protocolVersion":"2025-11-25", "capabilities":{},
                        "clientInfo":{"name":"starclock-http-benchmark","version":"1"}
                    }
                }),
                None,
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let transport_session = response.headers()["mcp-session-id"].clone();
        let _ = decode(response).await;
        let notification = router
            .clone()
            .oneshot(request(
                json!({"jsonrpc":"2.0", "method":"notifications/initialized", "params":{}}),
                Some(&transport_session),
            ))
            .await
            .unwrap();
        assert!(notification.status().is_success());
        Self {
            router,
            transport_session,
            next_id: 2,
        }
    }

    async fn tool(&mut self, name: &str, arguments: Value) -> Value {
        self.tool_measured(name, arguments).await.0
    }

    async fn tool_measured(&mut self, name: &str, arguments: Value) -> (Value, usize) {
        let id = self.next_id;
        self.next_id += 1;
        let response = self
            .router
            .clone()
            .oneshot(request(
                json!({
                    "jsonrpc":"2.0", "id":id, "method":"tools/call",
                    "params":{"name":name, "arguments":arguments}
                }),
                Some(&self.transport_session),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        let (body, bytes) = decode(response).await;
        let result = &body["result"];
        assert_ne!(result["isError"], true, "{result}");
        (result["structuredContent"].clone(), bytes)
    }
}

fn router() -> Router {
    let config = LoopbackHttpConfig::new(AUTHORITY.parse().unwrap(), vec![ORIGIN.into()]).unwrap();
    authorized_loopback_router(&config, policy()).unwrap()
}

fn request(body: Value, session: Option<&HeaderValue>) -> Request<Body> {
    let mut request = Request::builder()
        .method(Method::POST)
        .uri(MCP_HTTP_PATH)
        .header("host", AUTHORITY)
        .header("origin", ORIGIN)
        .header("accept", "application/json, text/event-stream")
        .header(CONTENT_TYPE, "application/json")
        .header("MCP-Protocol-Version", "2025-11-25")
        .header("authorization", format!("Bearer {TOKEN}"))
        .body(Body::from(body.to_string()))
        .unwrap();
    if let Some(session) = session {
        request
            .headers_mut()
            .insert("mcp-session-id", session.clone());
    }
    request
}

async fn decode(response: axum::response::Response) -> (Value, usize) {
    let bytes = to_bytes(response.into_body(), MAX_HTTP_RESPONSE_BYTES)
        .await
        .unwrap();
    let len = bytes.len();
    if let Ok(value) = serde_json::from_slice(&bytes) {
        return (value, len);
    }
    let text = String::from_utf8_lossy(&bytes);
    let data = text
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .find(|line| line.starts_with('{'))
        .unwrap();
    (serde_json::from_str(data).unwrap(), len)
}

#[derive(Clone, Copy)]
struct Verifier;

impl AccessTokenSignatureVerifier for Verifier {
    fn verify_signature_and_decode(
        &self,
        bearer_token: &str,
    ) -> Result<SignedTokenClaims, SignatureVerificationError> {
        let (tenant, principal) = bearer_token
            .split_once(':')
            .ok_or(SignatureVerificationError::Invalid)?;
        SignedTokenClaims::new(
            "https://auth.example".into(),
            vec![format!("http://{AUTHORITY}{MCP_HTTP_PATH}")],
            2_000,
            Some(900),
            tenant.into(),
            principal.into(),
            SUPPORTED_SCOPES.iter().map(ToString::to_string).collect(),
        )
        .map_err(|_| SignatureVerificationError::Invalid)
    }
}

struct Clock;

impl AuthorizationClock for Clock {
    fn now_seconds(&self) -> u64 {
        1_000
    }
}

fn policy() -> AuthorizationPolicy {
    AuthorizationPolicy::new(
        "https://auth.example".into(),
        format!("http://{AUTHORITY}{MCP_HTTP_PATH}"),
        format!("http://{AUTHORITY}{PROTECTED_RESOURCE_METADATA_PATH}"),
        Arc::new(Verifier),
        Arc::new(Clock),
    )
    .unwrap()
}

fn nonnegative(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

fn divide_ceil(value: u64, divisor: u64) -> u64 {
    value.saturating_add(divisor.saturating_sub(1)) / divisor
}
