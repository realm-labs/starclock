use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
};

use serde_json::{Value, json};

const EXPECTED_TOOLS: [&str; 7] = [
    "starclock_close_battle",
    "starclock_create_battle",
    "starclock_export_replay",
    "starclock_list_scenarios",
    "starclock_observe_battle",
    "starclock_play_action",
    "starclock_verify_replay",
];
const BASIC_SCENARIO: &str = "scenario.standard-v1.basic-single-wave";
const BASIC_FINAL_HASH: &str = "5021cdd6019e0a100ad35e36ffb69fdb4860600db472c77fb8b33a9571b507ec";

fn spawn_server() -> std::process::Child {
    Command::new(env!("CARGO_BIN_EXE_starclock"))
        .args(["mcp", "serve", "--transport", "stdio"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("stdio MCP server launches")
}

struct StdioClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    stderr: ChildStderr,
    next_id: u64,
    response_frames: usize,
}

impl StdioClient {
    fn launch() -> Self {
        let mut child = spawn_server();
        Self {
            stdin: child.stdin.take().unwrap(),
            stdout: BufReader::new(child.stdout.take().unwrap()),
            stderr: child.stderr.take().unwrap(),
            child,
            next_id: 1,
            response_frames: 0,
        }
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        let encoded = serde_json::to_vec(&request).unwrap();
        assert!(
            encoded.len() <= starclock_mcp::stdio::MAX_STDIO_FRAME_BYTES,
            "test request exceeded the production stdio frame limit"
        );
        self.stdin.write_all(&encoded).unwrap();
        self.stdin.write_all(b"\n").unwrap();
        self.stdin.flush().unwrap();

        let mut frame = String::new();
        assert_ne!(self.stdout.read_line(&mut frame).unwrap(), 0, "server EOF");
        assert!(
            frame.starts_with('{') && frame.ends_with("}\n"),
            "{frame:?}"
        );
        let response: Value = serde_json::from_str(&frame).expect("stdout frame is JSON");
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], id);
        self.response_frames += 1;
        response
    }

    fn notify(&mut self, method: &str, params: Value) {
        let notification = json!({"jsonrpc": "2.0", "method": method, "params": params});
        let encoded = serde_json::to_vec(&notification).unwrap();
        assert!(encoded.len() <= starclock_mcp::stdio::MAX_STDIO_FRAME_BYTES);
        self.stdin.write_all(&encoded).unwrap();
        self.stdin.write_all(b"\n").unwrap();
        self.stdin.flush().unwrap();
    }

    fn result(&mut self, method: &str, params: Value) -> Value {
        let response = self.request(method, params);
        assert!(response.get("error").is_none(), "{response}");
        response["result"].clone()
    }

    fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let result = self.result("tools/call", json!({"name": name, "arguments": arguments}));
        assert_ne!(result["isError"], true, "{result}");
        result["structuredContent"].clone()
    }

    fn shutdown(self) -> (ExitStatus, String, String, usize) {
        let Self {
            mut child,
            stdin,
            mut stdout,
            mut stderr,
            response_frames,
            ..
        } = self;
        drop(stdin);
        let mut trailing_stdout = String::new();
        stdout.read_to_string(&mut trailing_stdout).unwrap();
        let status = child.wait().unwrap();
        let mut stderr_text = String::new();
        stderr.read_to_string(&mut stderr_text).unwrap();
        (status, trailing_stdout, stderr_text, response_frames)
    }
}

fn observation_from_action(action: &Value) -> Value {
    action["response"]["observation"].clone()
}

fn first_scripted_action(observation: &Value) -> &Value {
    observation["legal_actions"]
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
        .expect("frozen public script has an ability or interrupt pass")
}

#[test]
fn independent_stdio_client_proves_discovery_play_errors_cancellation_replay_and_shutdown() {
    let mut client = StdioClient::launch();
    let initialized = client.result(
        "initialize",
        json!({
            "protocolVersion": "2025-11-25",
            "capabilities": {},
            "clientInfo": {"name": "starclock-conformance", "version": "1"}
        }),
    );
    assert_eq!(initialized["protocolVersion"], "2025-11-25");
    assert_eq!(initialized["serverInfo"]["name"], "starclock-mcp");
    client.notify("notifications/initialized", json!({}));

    let tools = client.result("tools/list", json!({}));
    let mut tool_names = tools["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect::<Vec<_>>();
    tool_names.sort_unstable();
    assert_eq!(tool_names, EXPECTED_TOOLS);
    assert!(tools["tools"].as_array().unwrap().iter().all(|tool| {
        tool["inputSchema"]["type"] == "object" && tool["outputSchema"]["type"] == "object"
    }));

    let resources = client.result("resources/list", json!({}));
    let resource_uris = resources["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| resource["uri"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        resource_uris,
        [
            "starclock://catalog/manifest",
            "starclock://rules/core-combat"
        ]
    );
    let templates = client.result("resources/templates/list", json!({}));
    let template_uris = templates["resourceTemplates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|template| template["uriTemplate"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        template_uris,
        [
            "starclock://scenario/{scenario_id}",
            "starclock://character/{form_id}"
        ]
    );
    let prompt_list = client.result("prompts/list", json!({}));
    assert_eq!(prompt_list["prompts"][0]["name"], "starclock_battle_loop");
    let prompt = client.result(
        "prompts/get",
        json!({"name": "starclock_battle_loop", "arguments": {}}),
    );
    assert!(
        prompt.to_string().contains("opaque"),
        "unexpected prompt wire value: {prompt}"
    );
    for uri in [
        "starclock://catalog/manifest",
        "starclock://scenario/scenario.standard-v1.basic-single-wave",
    ] {
        let resource = client.result("resources/read", json!({"uri": uri}));
        let text = resource["contents"][0]["text"].as_str().unwrap();
        assert!(text.len() <= 16 * 1024);
        assert_eq!(
            serde_json::from_str::<Value>(text).unwrap()["inert_data"],
            true
        );
    }

    let scenarios = client.tool("starclock_list_scenarios", json!({}));
    assert_eq!(scenarios["scenarios"].as_array().unwrap().len(), 6);
    let created = client.tool(
        "starclock_create_battle",
        json!({"schema_revision": "agent-api-v1", "scenario_id": BASIC_SCENARIO}),
    );
    let mut observation = created["observation"].clone();
    let mut transport_hashes = vec![observation["state_hash"].clone()];
    let session_id = observation["session_id"].as_str().unwrap().to_owned();
    let stale_decision = observation["decision_id"].clone();
    let stale_hash = observation["state_hash"].clone();
    let stale_token = first_scripted_action(&observation)["token"].clone();

    let first = first_scripted_action(&observation);
    let first_result = client.tool(
        "starclock_play_action",
        json!({
            "schema_revision": "agent-api-v1",
            "session_id": session_id,
            "decision_id": observation["decision_id"],
            "expected_state_hash": observation["state_hash"],
            "action_token": first["token"],
            "idempotency_key": "stdio_action_0"
        }),
    );
    observation = observation_from_action(&first_result);
    transport_hashes.push(observation["state_hash"].clone());
    let post_first_hash = observation["state_hash"].clone();

    let stale = client.result(
        "tools/call",
        json!({
            "name": "starclock_play_action",
            "arguments": {
                "schema_revision": "agent-api-v1",
                "session_id": session_id,
                "decision_id": stale_decision,
                "expected_state_hash": stale_hash,
                "action_token": stale_token,
                "idempotency_key": "stdio_stale_action"
            }
        }),
    );
    assert_eq!(stale["isError"], true);
    assert!(matches!(
        stale["structuredContent"]["code"].as_str(),
        Some("stale_decision" | "stale_state_hash")
    ));

    let malformed = client.request(
        "tools/call",
        json!({
            "name": "starclock_play_action",
            "arguments": {"schema_revision": "agent-api-v1", "session_id": session_id}
        }),
    );
    assert_eq!(malformed["result"]["isError"], true, "{malformed}");
    assert!(
        malformed["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("missing field"),
        "{malformed}"
    );
    client.notify(
        "notifications/cancelled",
        json!({"requestId": 9_999_999, "reason": "conformance continuity probe"}),
    );
    let observed = client.tool(
        "starclock_observe_battle",
        json!({"schema_revision": "agent-api-v1", "session_id": session_id}),
    );
    assert_eq!(observed["observation"]["state_hash"], post_first_hash);
    observation = observed["observation"].clone();

    let mut step = 1_u64;
    while observation["status"] == "awaiting_player" {
        assert!(step < 64, "basic scenario exceeded scripted bound");
        let action = first_scripted_action(&observation);
        let played = client.tool(
            "starclock_play_action",
            json!({
                "schema_revision": "agent-api-v1",
                "session_id": session_id,
                "decision_id": observation["decision_id"],
                "expected_state_hash": observation["state_hash"],
                "action_token": action["token"],
                "idempotency_key": format!("stdio_action_{step}")
            }),
        );
        observation = observation_from_action(&played);
        transport_hashes.push(observation["state_hash"].clone());
        step += 1;
    }
    assert_eq!(step, 8);
    assert_eq!(observation["status"], "won");
    assert_eq!(observation["state_hash"], BASIC_FINAL_HASH);
    let frozen_trace: Value = serde_json::from_str(include_str!(
        "../../../evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json"
    ))
    .unwrap();
    assert_eq!(Value::Array(transport_hashes), frozen_trace["state_hashes"]);

    let exported = client.tool(
        "starclock_export_replay",
        json!({"schema_revision": "agent-api-v1", "session_id": session_id}),
    );
    assert_eq!(exported["command_count"], "9");
    assert_eq!(exported["replay_hex"], frozen_trace["replay_hex"]);
    let replay_hex = exported["replay_hex"].clone();
    let closed = client.tool(
        "starclock_close_battle",
        json!({"schema_revision": "agent-api-v1", "session_id": session_id}),
    );
    assert_eq!(closed["closed"], true);
    let verified = client.tool(
        "starclock_verify_replay",
        json!({
            "schema_revision": "agent-api-v1",
            "scenario_id": BASIC_SCENARIO,
            "replay_hex": replay_hex
        }),
    );
    assert_eq!(verified["command_count"], "9");
    assert_eq!(verified["phase"], "won");
    assert_eq!(verified["final_state_hash"], BASIC_FINAL_HASH);

    let (status, trailing_stdout, stderr, response_frames) = client.shutdown();
    assert!(status.success(), "{status:?}: {stderr}");
    assert!(trailing_stdout.is_empty(), "{trailing_stdout}");
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(response_frames, 24);
}

#[test]
fn stdio_stdout_contains_only_json_rpc_frames() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(
            concat!(
                r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"stdio-contract","version":"1"}}}"#,
                "\n",
                r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
                "\n",
                r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
                "\n"
            )
            .as_bytes(),
        )
        .unwrap();
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "stderr: {output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let lines = stdout.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2, "stdout: {stdout}");
    assert!(lines.iter().all(|line| {
        line.starts_with('{')
            && line.ends_with('}')
            && line.contains(r#""jsonrpc":"2.0""#)
            && !line.contains("MCP service error")
    }));
    assert!(stdout.contains(r#""id":1"#));
    assert!(stdout.contains(r#""id":2"#));
    assert!(stdout.contains("starclock_play_action"));
}

#[test]
fn oversized_stdio_frame_stops_before_json_decode_and_diagnostics_use_stderr() {
    let mut child = spawn_server();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(&vec![b'x'; starclock_mcp::stdio::MAX_STDIO_FRAME_BYTES + 1])
        .unwrap();
    stdin.write_all(b"\n").unwrap();
    drop(stdin);

    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(8), "{output:?}");
    assert!(output.stdout.is_empty(), "stdout: {output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("MCP service error"), "stderr: {stderr}");
    assert!(!stderr.contains(&"x".repeat(64)));
}
