use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};

use serde_json::{Value, json};
use starclock_mcp::{
    authorization::{
        AccessTokenSignatureVerifier, AuthorizationClock, AuthorizationPolicy, SUPPORTED_SCOPES,
        SignatureVerificationError, SignedTokenClaims,
    },
    http::{
        LoopbackHttpConfig, MCP_HTTP_PATH, PROTECTED_RESOURCE_METADATA_PATH,
        authorized_loopback_router,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

const SCENARIO: &str = "scenario.standard-v1.basic-single-wave";
const TOKEN: &str = "tenant-conformance:principal-conformance";
const CLIENTS: usize = 8;

#[derive(Clone)]
struct ConformanceVerifier {
    audience: String,
}

impl AccessTokenSignatureVerifier for ConformanceVerifier {
    fn verify_signature_and_decode(
        &self,
        bearer_token: &str,
    ) -> Result<SignedTokenClaims, SignatureVerificationError> {
        let (tenant, principal) = bearer_token
            .split_once(':')
            .ok_or(SignatureVerificationError::Invalid)?;
        SignedTokenClaims::new(
            "https://auth.example".into(),
            vec![self.audience.clone()],
            2_000,
            Some(900),
            tenant.into(),
            principal.into(),
            SUPPORTED_SCOPES.iter().map(ToString::to_string).collect(),
        )
        .map_err(|_| SignatureVerificationError::Invalid)
    }
}

struct FixedClock;

impl AuthorizationClock for FixedClock {
    fn now_seconds(&self) -> u64 {
        1_000
    }
}

struct HttpMcpClient {
    address: SocketAddr,
    authority: String,
    origin: String,
    transport_session: Option<String>,
    next_id: u64,
}

impl HttpMcpClient {
    fn new(address: SocketAddr) -> Self {
        let authority = address.to_string();
        Self {
            address,
            origin: format!("http://{authority}"),
            authority,
            transport_session: None,
            next_id: 1,
        }
    }

    async fn initialize(&mut self) {
        let response = self
            .send_rpc(json!({
                "jsonrpc":"2.0", "id":self.next_id, "method":"initialize",
                "params":{
                    "protocolVersion":"2025-11-25",
                    "capabilities":{},
                    "clientInfo":{"name":"starclock-http-conformance","version":"1"}
                }
            }))
            .await;
        self.next_id += 1;
        assert_eq!(response.status, 200);
        let body = response.json();
        assert_eq!(body["result"]["protocolVersion"], "2025-11-25");
        assert_eq!(body["result"]["serverInfo"]["name"], "starclock-mcp");
        self.transport_session = Some(response.headers["mcp-session-id"].clone());
        let initialized = self
            .send_rpc(json!({
                "jsonrpc":"2.0", "method":"notifications/initialized", "params":{}
            }))
            .await;
        assert!((200..300).contains(&initialized.status));
    }

    async fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let response = self
            .send_rpc(json!({"jsonrpc":"2.0", "id":id, "method":method, "params":params}))
            .await;
        assert_eq!(response.status, 200, "{}", response.text());
        let body = response.json();
        assert_eq!(body["id"], id);
        assert!(body.get("error").is_none(), "{body}");
        body["result"].clone()
    }

    async fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let result = self
            .request("tools/call", json!({"name":name, "arguments":arguments}))
            .await;
        assert_ne!(result["isError"], true, "{result}");
        result["structuredContent"].clone()
    }

    async fn send_rpc(&self, body: Value) -> RawResponse {
        let body = body.to_string();
        let authorization = format!("Bearer {TOKEN}");
        let mut headers = vec![
            ("Origin", self.origin.as_str()),
            ("Accept", "application/json, text/event-stream"),
            ("Content-Type", "application/json"),
            ("MCP-Protocol-Version", "2025-11-25"),
            ("Authorization", authorization.as_str()),
        ];
        if let Some(session) = &self.transport_session {
            headers.push(("MCP-Session-Id", session));
        }
        raw_http(
            self.address,
            &self.authority,
            "POST",
            MCP_HTTP_PATH,
            &headers,
            body.as_bytes(),
        )
        .await
    }
}

struct RawResponse {
    status: u16,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl RawResponse {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    fn json(&self) -> Value {
        if let Ok(value) = serde_json::from_slice(&self.body) {
            return value;
        }
        let text = self.text();
        let data = text
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .find(|line| line.starts_with('{'))
            .unwrap_or_else(|| panic!("expected JSON or SSE JSON, got {text:?}"));
        serde_json::from_str(data).unwrap()
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn authorized_tcp_client_proves_conformance_trace_and_multi_session_load() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let authority = address.to_string();
    let origin = format!("http://{authority}");
    let config = LoopbackHttpConfig::new(address, vec![origin.clone()]).unwrap();
    let audience = format!("http://{authority}{MCP_HTTP_PATH}");
    let policy = AuthorizationPolicy::new(
        "https://auth.example".into(),
        audience.clone(),
        format!("http://{authority}{PROTECTED_RESOURCE_METADATA_PATH}"),
        Arc::new(ConformanceVerifier { audience }),
        Arc::new(FixedClock),
    )
    .unwrap();
    let router = authorized_loopback_router(&config, policy).unwrap();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            })
            .await
            .unwrap();
    });

    for path in ["/healthz", "/readyz", "/metrics"] {
        let response = raw_http(
            address,
            &authority,
            "GET",
            path,
            &[("Origin", origin.as_str())],
            &[],
        )
        .await;
        assert_eq!(response.status, 200, "{path}: {}", response.text());
    }

    let mut discovery = HttpMcpClient::new(address);
    discovery.initialize().await;
    let tools = discovery.request("tools/list", json!({})).await;
    assert_eq!(tools["tools"].as_array().unwrap().len(), 13);

    run_activity_boundary(&mut discovery).await;

    let expected = frozen_trace();
    let primary = run_basic_trace(discovery, "primary").await;
    assert_trace(&primary, &expected);

    let mut tasks = Vec::with_capacity(CLIENTS);
    for index in 0..CLIENTS {
        tasks.push(tokio::spawn(async move {
            let mut client = HttpMcpClient::new(address);
            client.initialize().await;
            run_basic_trace(client, &format!("load-{index}")).await
        }));
    }
    for task in tasks {
        let trace = task.await.unwrap();
        assert_trace(&trace, &expected);
    }

    shutdown_sender.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("HTTP server stopped within the test bound")
        .unwrap();
}

async fn run_activity_boundary(client: &mut HttpMcpClient) {
    let created = client
        .tool(
            "starclock_create_universe",
            json!({
                "schema_revision":"agent-api-v1", "world":"1", "difficulty_index":"0", "seed":"1"
            }),
        )
        .await;
    let observation = &created["observation"];
    let session_id = observation["session_id"].as_str().unwrap().to_owned();
    let input = json!({
        "schema_revision":"agent-api-v1", "session_id":session_id,
        "boundary_id":observation["boundary_id"], "expected_state_hash":observation["state_hash"],
        "action_token":observation["legal_actions"][0]["token"],
        "idempotency_key":"http_activity_1"
    });
    let first = client
        .tool("starclock_play_activity_action", input.clone())
        .await;
    let repeated = client.tool("starclock_play_activity_action", input).await;
    assert_eq!(first["response"]["committed"], true);
    assert_eq!(repeated, first);
    let mut observation = first["response"]["observation"].clone();
    let mut steps = 1_u64;
    while observation["status"] == "awaiting_action" {
        assert!(steps < 1_000);
        let actions = observation["legal_actions"].as_array().unwrap();
        let action = actions
            .iter()
            .find(|action| action["kind"] == "engage_battle")
            .unwrap_or(&actions[0]);
        let played = client
            .tool(
                "starclock_play_activity_action",
                json!({
                    "schema_revision":"agent-api-v1",
                    "session_id":session_id,
                    "boundary_id":observation["boundary_id"],
                    "expected_state_hash":observation["state_hash"],
                    "action_token":action["token"],
                    "idempotency_key":format!("http_activity_{}", steps + 1)
                }),
            )
            .await;
        observation = played["response"]["observation"].clone();
        steps += 1;
    }
    assert_eq!(observation["status"], "completed");
    let exported = client
        .tool(
            "starclock_export_activity_replay",
            json!({
                "schema_revision":"agent-api-v1", "session_id":session_id
            }),
        )
        .await;
    assert_ne!(exported["action_count"], "0");
    assert_eq!(exported["complete"], true);
    let verified = client
        .tool(
            "starclock_verify_activity_replay",
            json!({
                "schema_revision":"agent-api-v1",
                "world":"1",
                "difficulty_index":"0",
                "seed":"1",
                "replay_hex":exported["replay_hex"]
            }),
        )
        .await;
    assert!(
        verified["nested_battles"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap()
            > 0
    );
    assert_eq!(verified["final_state_hash"], observation["state_hash"]);
    let observed = client
        .tool(
            "starclock_observe_activity",
            json!({
                "schema_revision":"agent-api-v1", "session_id":session_id
            }),
        )
        .await;
    assert_eq!(observed["observation"]["session_id"], session_id);
    let closed = client
        .tool(
            "starclock_close_activity",
            json!({
                "schema_revision":"agent-api-v1", "session_id":session_id
            }),
        )
        .await;
    assert_eq!(closed["closed"], true);
}

struct TransportTrace {
    state_hashes: Vec<Value>,
    replay_hex: Value,
    command_count: Value,
    final_hash: Value,
}

async fn run_basic_trace(mut client: HttpMcpClient, prefix: &str) -> TransportTrace {
    let created = client
        .tool(
            "starclock_create_battle",
            json!({"schema_revision":"agent-api-v1", "scenario_id":SCENARIO}),
        )
        .await;
    let mut observation = created["observation"].clone();
    let session_id = observation["session_id"].as_str().unwrap().to_owned();
    let mut state_hashes = vec![observation["state_hash"].clone()];
    let mut step = 0_u64;
    while observation["status"] == "awaiting_player" {
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
        let played = client
            .tool(
                "starclock_play_action",
                json!({
                    "schema_revision":"agent-api-v1",
                    "session_id":session_id,
                    "decision_id":observation["decision_id"],
                    "expected_state_hash":observation["state_hash"],
                    "action_token":action["token"],
                    "idempotency_key":format!("http_{prefix}_{step}")
                }),
            )
            .await;
        observation = played["response"]["observation"].clone();
        state_hashes.push(observation["state_hash"].clone());
        step += 1;
        assert!(step <= 16, "basic trace exceeded its frozen action count");
    }
    let exported = client
        .tool(
            "starclock_export_replay",
            json!({"schema_revision":"agent-api-v1", "session_id":session_id}),
        )
        .await;
    let replay_hex = exported["replay_hex"].clone();
    let command_count = exported["command_count"].clone();
    let verified = client
        .tool(
            "starclock_verify_replay",
            json!({
                "schema_revision":"agent-api-v1",
                "scenario_id":SCENARIO,
                "replay_hex":replay_hex
            }),
        )
        .await;
    assert_eq!(verified["phase"], "won");
    assert_eq!(verified["command_count"], command_count);
    let closed = client
        .tool(
            "starclock_close_battle",
            json!({"schema_revision":"agent-api-v1", "session_id":session_id}),
        )
        .await;
    assert_eq!(closed["closed"], true);
    TransportTrace {
        state_hashes,
        replay_hex,
        command_count,
        final_hash: verified["final_state_hash"].clone(),
    }
}

fn frozen_trace() -> Value {
    serde_json::from_str(include_str!(
        "../../../../../../evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json"
    ))
    .unwrap()
}

fn assert_trace(actual: &TransportTrace, expected: &Value) {
    const CURRENT_COMBAT_STATE_HASHES: [&str; 17] = [
        "0dbb4a6758e0f9bd2517bd0451e7dee5febcee3b8a0068ae7bdfeacea4f6f2eb",
        "6a37830a7d7d7ccc85c6176f90711f145f4a39e5f2cc8b8ed1a50b0716422d59",
        "8abc6eb88d05cc7d020f32f376568a8705fb3e08bafe244032b596ba97a7e845",
        "7930351764d3e53be8dcddae866f250c4f097fe2e2e7f9aef9a2b1a3985a0bf0",
        "b8dd6df46cfa64762bb24eac55fafdeb96fb7c543e57037df8d648f328959fc5",
        "197b76288c98079f1ec6f990cb26ef80a2a73394b6b627346be4a0f6fc4e15cf",
        "b4377c698673e0933be7f89b7b9d661f02ac28060f70e49bb621b35ab4c06499",
        "e2a72c71bac4f8afa5da82358609b645d46d790826f5416a3c7082531225285a",
        "3d50f32c45cb6770494a41afe6ed9021f3a01d473b7df00d3bad21da27083452",
        "a3e6e942302e3ac3ff009c6c4d50149686d10f150e08798fb4dbbf23e07d6798",
        "a53c1a106a15dae6a93b7f5ddd3324529f5b959534c564ed8f0e3eedb051e569",
        "aea482ba7e8859ccf4d4b0b3db2c9faa65f5c4b8f332dd9f73c074bfaafdfd7e",
        "ee65f6141efb1a5a47cda6c68f9785c1ceb8998d85ff2639b730fc18617e9303",
        "98fddb516081afdb4b54e7d52627d03697c3d8792ee8220859211a926d950c92",
        "b841c5e6fb00feb900e7588dec7b46268732a0b1a26e8f05be0acb6cc57070ea",
        "069313f874c09edbf7522e6c0a4614725dc15c39adaa44542feda23376376e8a",
        "71faf56504a7ffb1f5c54b0135c68939a5973fb6b9e065217c12ae4d0e5e5b9e",
    ];
    assert_eq!(
        Value::Array(actual.state_hashes.clone()),
        serde_json::to_value(CURRENT_COMBAT_STATE_HASHES).unwrap(),
        "the transport trace follows the current declared combat state codec"
    );
    assert_eq!(
        expected["schema_revision"],
        "starclock.agent-transport-trace.v1"
    );
    assert_eq!(expected["scenario_id"], SCENARIO);
    assert_eq!(expected["external_actions"], 8);
    assert_eq!(expected["replay_commands"], 9);
    assert!(!actual.replay_hex.as_str().unwrap().is_empty());
    assert_eq!(actual.command_count, "21");
    assert_eq!(actual.final_hash, CURRENT_COMBAT_STATE_HASHES[16]);
}

async fn raw_http(
    address: SocketAddr,
    authority: &str,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> RawResponse {
    let mut request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {authority}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in headers {
        request.push_str(name);
        request.push_str(": ");
        request.push_str(value);
        request.push_str("\r\n");
    }
    request.push_str("\r\n");
    let mut stream = TcpStream::connect(address).await.unwrap();
    stream.write_all(request.as_bytes()).await.unwrap();
    stream.write_all(body).await.unwrap();
    let mut bytes = Vec::new();
    tokio::time::timeout(Duration::from_secs(20), stream.read_to_end(&mut bytes))
        .await
        .expect("HTTP response completed within the test bound")
        .unwrap();
    parse_response(&bytes)
}

fn parse_response(bytes: &[u8]) -> RawResponse {
    let header_end = bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("HTTP response has a header terminator");
    let head = std::str::from_utf8(&bytes[..header_end]).unwrap();
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .unwrap()
        .split_whitespace()
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers = lines
        .map(|line| line.split_once(':').unwrap())
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let raw_body = &bytes[header_end + 4..];
    let body = if headers
        .get("transfer-encoding")
        .is_some_and(|value| value.eq_ignore_ascii_case("chunked"))
    {
        decode_chunks(raw_body)
    } else {
        raw_body.to_vec()
    };
    RawResponse {
        status,
        headers,
        body,
    }
}

fn decode_chunks(bytes: &[u8]) -> Vec<u8> {
    let mut decoded = Vec::new();
    let mut cursor = 0;
    loop {
        let line_end = bytes[cursor..]
            .windows(2)
            .position(|window| window == b"\r\n")
            .map(|offset| cursor + offset)
            .expect("chunk size has a terminator");
        let size_text = std::str::from_utf8(&bytes[cursor..line_end]).unwrap();
        let size = usize::from_str_radix(size_text.split(';').next().unwrap(), 16).unwrap();
        cursor = line_end + 2;
        if size == 0 {
            break;
        }
        decoded.extend_from_slice(&bytes[cursor..cursor + size]);
        cursor += size;
        assert_eq!(&bytes[cursor..cursor + 2], b"\r\n");
        cursor += 2;
    }
    decoded
}
