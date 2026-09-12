use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, ReadResourceRequestParams},
};
use serde_json::{Map, Value, json};
use starclock_agent_api::{
    activity_session::{ActivityAgentSessionFactory, registry::ActivityAgentSessionRegistry},
    divergent_universe_activity_session::DivergentUniverseActivityAgentSessionFactory,
    error::AgentError,
    schema::SessionId,
    session::{
        AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry, OperationalClock,
        SessionIdSource,
    },
};

use crate::server::StarclockMcp;

struct TestClock;
impl OperationalClock for TestClock {
    fn now_seconds(&self) -> u64 {
        0
    }
}

#[derive(Default)]
struct TestIds(AtomicU64);
impl SessionIdSource for TestIds {
    fn next_session_id(&self) -> Result<SessionId, AgentError> {
        SessionId::parse(&format!(
            "divergent_mcp_{}",
            self.0.fetch_add(1, Ordering::Relaxed) + 1
        ))
        .map_err(|_| {
            AgentError::new(
                starclock_agent_api::error::AgentErrorCode::AdapterFailure,
                "The test session ID is invalid.",
                false,
                false,
            )
            .expect("static test error")
        })
    }
}

#[tokio::test]
async fn divergent_universe_mcp_is_bounded_idempotent_cancellable_and_replayable() {
    run_public_session(None).await;
}

#[tokio::test]
async fn divergent_universe_mcp_tawot_configuration_and_encoded_replay() {
    run_public_session(Some("5")).await;
}

async fn run_public_session(tawot_forge_level: Option<&str>) {
    let factory = AgentSessionFactory::load_production().expect("battle factory");
    let standard = ActivityAgentSessionFactory::load_production().expect("Standard factory");
    let divergent =
        DivergentUniverseActivityAgentSessionFactory::load_production().expect("DU factory");
    let clock = Arc::new(TestClock);
    let ids = Arc::new(TestIds::default());
    let battle_registry = AgentSessionRegistry::new(factory.clone(), clock.clone(), ids.clone());
    let activity_registry = ActivityAgentSessionRegistry::new_with_divergent_universe(
        standard.clone(),
        divergent,
        clock,
        ids,
    );
    let server = StarclockMcp::new(
        battle_registry,
        factory,
        activity_registry,
        standard,
        AgentSessionOwner::new("local", "divergent-test").expect("owner"),
    );
    let (server_transport, client_transport) = tokio::io::duplex(512 * 1024);
    let task = tokio::spawn(async move {
        server
            .serve(server_transport)
            .await
            .expect("serve")
            .waiting()
            .await
            .expect("server wait");
    });
    let client = ().serve(client_transport).await.expect("client");
    let tools = client.list_all_tools().await.unwrap();
    let create = tools
        .iter()
        .find(|tool| tool.name == "starclock_create_universe")
        .unwrap();
    let schema = serde_json::to_value(&create.input_schema).unwrap();
    assert!(schema["properties"].get("tawot_forge_level").is_some());
    assert!(
        !schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|field| field == "tawot_forge_level")
    );

    let manifest = client
        .read_resource(ReadResourceRequestParams::new(
            "starclock://universe/divergent-universe/manifest",
        ))
        .await
        .expect("manifest");
    let manifest = serde_json::to_string(&manifest).expect("manifest JSON");
    assert!(manifest.contains("divergent-universe"));
    assert!(manifest.contains("source_obligations"));
    assert!(!manifest.contains("generated_row"));
    let rules = client
        .read_resource(ReadResourceRequestParams::new(
            "starclock://rules/divergent-universe",
        ))
        .await
        .expect("rules");
    assert!(
        serde_json::to_string(&rules)
            .expect("rules JSON")
            .contains("authoritative_real_combat_settlement")
    );

    for (mode, level) in [
        ("standard", "2"),
        ("gold-and-gears", "2"),
        ("swarm-disaster", "2"),
        ("divergent-universe", "1"),
        ("divergent-universe", "6"),
        ("divergent-universe", "02"),
    ] {
        let rejected = client
            .call_tool(
                CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                    json!({
                        "mode":mode, "seed":"22301", "tawot_forge_level":level
                    }),
                )),
            )
            .await
            .unwrap();
        assert_eq!(rejected.is_error, Some(true), "{mode} {level}");
    }
    let created = client
        .call_tool(
            CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                json!({
                    "mode":"divergent-universe", "family":"ordinary", "seed":"22301",
                    "tawot_forge_level":tawot_forge_level
                }),
            )),
        )
        .await
        .expect("create");
    assert_eq!(created.is_error, Some(false));
    let mut observation = created.structured_content.expect("content")["observation"].clone();
    let session_id = observation["session_id"]
        .as_str()
        .expect("session")
        .to_owned();
    let mut action_index = 0_u64;
    while observation["status"] != "completed" {
        let action = if action_index == 1 && tawot_forge_level.is_none() {
            observation["legal_actions"]
                .as_array()
                .expect("offered actions")
                .iter()
                .find(|action| action["option_id"] == "2")
                .expect("non-default Curio choice")
        } else {
            selected_action(&observation)
        }
        .clone();
        let input = json!({
            "session_id":session_id,
            "boundary_id":observation["boundary_id"],
            "expected_state_hash":observation["state_hash"],
            "action_token":action["token"],
            "idempotency_key":format!("du_mcp_action_{action_index}")
        });
        let played = client
            .call_tool(
                CallToolRequestParams::new("starclock_play_activity_action")
                    .with_arguments(arguments(input.clone())),
            )
            .await
            .expect("play");
        assert_eq!(played.is_error, Some(false));
        if action_index == 0 {
            let repeated = client
                .call_tool(
                    CallToolRequestParams::new("starclock_play_activity_action")
                        .with_arguments(arguments(input)),
                )
                .await
                .expect("retry");
            assert_eq!(repeated.structured_content, played.structured_content);
        }
        observation =
            played.structured_content.expect("content")["response"]["observation"].clone();
        action_index += 1;
    }
    let expected_actions = if tawot_forge_level.is_some() { 16 } else { 10 };
    assert_eq!(action_index, expected_actions);

    let observed = client
        .call_tool(
            CallToolRequestParams::new("starclock_observe_activity").with_arguments(arguments(
                json!({"session_id":session_id, "event_cursor":"event_0"}),
            )),
        )
        .await
        .expect("observe events");
    let observed = observed.structured_content.expect("content");
    assert_eq!(
        observed["events"].as_array().expect("events").len(),
        expected_actions as usize
    );
    assert_eq!(observed["events_truncated"], false);
    assert_eq!(
        observed["event_cursor"],
        format!("event_{expected_actions}")
    );

    let exported = client
        .call_tool(
            CallToolRequestParams::new("starclock_export_activity_replay")
                .with_arguments(arguments(json!({"session_id":session_id}))),
        )
        .await
        .expect("export");
    let export = exported.structured_content.expect("content");
    assert_eq!(export["action_count"], expected_actions.to_string());
    assert_eq!(export["complete"], true);
    let verified = client
        .call_tool(
            CallToolRequestParams::new("starclock_verify_activity_replay").with_arguments(
                arguments(json!({
                    "mode":"divergent-universe", "family":"ordinary", "seed":"22301",
                    "replay_hex":export["replay_hex"]
                })),
            ),
        )
        .await
        .expect("verify");
    assert_eq!(verified.is_error, Some(false));
    assert_eq!(
        verified.structured_content.expect("verification")["nested_battles"],
        "3"
    );

    let closed = client
        .call_tool(
            CallToolRequestParams::new("starclock_close_activity")
                .with_arguments(arguments(json!({"session_id":session_id}))),
        )
        .await
        .expect("close");
    assert_eq!(closed.structured_content.expect("content")["closed"], true);
    let after_close = client
        .call_tool(
            CallToolRequestParams::new("starclock_observe_activity")
                .with_arguments(arguments(json!({"session_id":session_id}))),
        )
        .await
        .expect("closed response");
    assert_eq!(after_close.is_error, Some(true));

    client.cancel().await.expect("cancel");
    task.await.expect("task");
}

fn selected_action(observation: &Value) -> &Value {
    observation["legal_actions"]
        .as_array()
        .expect("actions")
        .iter()
        .max_by(|left, right| {
            priority(left)
                .cmp(&priority(right))
                .then_with(|| option(right).cmp(&option(left)))
        })
        .expect("one offered action")
}

fn priority(action: &Value) -> i64 {
    action["priority"]
        .as_str()
        .map_or(0, |value| value.parse().expect("priority"))
}

fn option(action: &Value) -> u64 {
    action["option_id"]
        .as_str()
        .expect("option")
        .parse()
        .expect("integer")
}

fn arguments(value: Value) -> Map<String, Value> {
    value.as_object().expect("arguments").clone()
}
