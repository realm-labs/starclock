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

const SCENARIO: &str = "scenario.standard.basic-single-wave";
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
#[ignore = "exhaustive current-state TCP conformance and load trace"]
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

    let primary = run_basic_trace(discovery, "primary").await;
    assert_trace(&primary);

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
        assert_trace(&trace);
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
                "world":"1", "difficulty_index":"0", "seed":"1"
            }),
        )
        .await;
    let observation = &created["observation"];
    let session_id = observation["session_id"].as_str().unwrap().to_owned();
    let input = json!({
        "session_id":session_id,
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
                "session_id":session_id
            }),
        )
        .await;
    assert_ne!(exported["action_count"], "0");
    assert_eq!(exported["complete"], true);
    let verified = client
        .tool(
            "starclock_verify_activity_replay",
            json!({
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
                "session_id":session_id
            }),
        )
        .await;
    assert_eq!(observed["observation"]["session_id"], session_id);
    let closed = client
        .tool(
            "starclock_close_activity",
            json!({
                "session_id":session_id
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
        .tool("starclock_create_battle", json!({"scenario_id":SCENARIO}))
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
        .tool("starclock_export_replay", json!({"session_id":session_id}))
        .await;
    let replay_hex = exported["replay_hex"].clone();
    let command_count = exported["command_count"].clone();
    let verified = client
        .tool(
            "starclock_verify_replay",
            json!({
                                "scenario_id":SCENARIO,
                "replay_hex":replay_hex
            }),
        )
        .await;
    assert_eq!(verified["phase"], "won");
    assert_eq!(verified["command_count"], command_count);
    let closed = client
        .tool("starclock_close_battle", json!({"session_id":session_id}))
        .await;
    assert_eq!(closed["closed"], true);
    TransportTrace {
        state_hashes,
        replay_hex,
        command_count,
        final_hash: verified["final_state_hash"].clone(),
    }
}

fn assert_trace(actual: &TransportTrace) {
    const CURRENT_COMBAT_STATE_HASHES: [&str; 17] = [
        "08bd3cbddc356df065f1a0a0014c3300bb5a930b0a2aafd713afcbdb6d881fca",
        "9f7c1c9909677305a886392cdb8237dc53e57b2e8e5ff638785ec574d21f7dd0",
        "e2aa6c3104bfefb58bb50bd8739990b3aea31c998f8587ef6306d5caaa07a5c5",
        "bc79607eb27370708954cb17b8d4beb44786dbe938cc08be26b7ef5bbb9a67b8",
        "83860b5093b6f772e71f0634de4561e31e701b6cac3a331bc3e22b50dea90743",
        "7d8c52276e234176f0ad0af2bfc023c7029965d8c319b2c41206eebe645190c3",
        "57ab6e8dc752f945e3644e9670fc856f7a5e6fba0de481026be6b65b776a83e5",
        "e9c0c20acb36135739bcf9e9366cfa00c4166ee303ca34e9b471eaa76a2d6ace",
        "e645bfe65e71374f98ba683418efa8ee5f7e0637e729be7defebb293b531fe8a",
        "cc46077296883febb7d20bd826b0da6f7e5b372f55bb7dcc163d52fd42ea0c4d",
        "dd7252d288fc16fcd0fb26062df7328dc217727c72f2c39efa8609945669fa06",
        "e499f8f8db0a03b79ade449681b5ee6a044d0127b4226fc9de17497246046bde",
        "136e6cf4a4f2a9c90c7445e119e468c50f16193148cbd5a7af81e200998fceed",
        "46f1c2b1e35b5315529fe537fff119ccc65f2533853858745b4d79a7c82ef373",
        "e986ffa9de1e4f72d33048f0331f5036886afdf6febd5090f6acf0ff266145b6",
        "20e3a9bb5cefdf51cb1c05bf5b92092b236027a163c967755f1665d13ac77b86",
        "c3a887357ed05ed76e51512f9813635cbd7bea223bde32ca10570b530ef44342",
    ];
    assert_eq!(
        Value::Array(actual.state_hashes.clone()),
        serde_json::to_value(CURRENT_COMBAT_STATE_HASHES).unwrap(),
        "the transport trace follows the current declared combat state codec"
    );
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
