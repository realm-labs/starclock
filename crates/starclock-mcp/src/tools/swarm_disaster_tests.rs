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
    error::AgentError,
    gold_gears_activity_session::GoldAndGearsActivityAgentSessionFactory,
    schema::SessionId,
    session::{
        AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry, OperationalClock,
        SessionIdSource,
    },
    swarm_disaster_activity_session::SwarmDisasterActivityAgentSessionFactory,
};

use crate::server::StarclockMcp;

const FINAL_STATE: &str = "058921eb765ac41314587c791c68f3267cb6a376ef71add9ce01a901d5645840";
const REPLAY_SHA256: &str = "3a7eb3742a6be456f8f8d3c526ea314fbf09b44d6719925d261ebbefc8997a67";

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
            "swarm_mcp_{}",
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
async fn swarm_disaster_uses_authorized_activity_tools_resources_and_replay() {
    let factory = AgentSessionFactory::load_production().expect("battle factory");
    let standard = ActivityAgentSessionFactory::load_production().expect("Standard factory");
    let gold = GoldAndGearsActivityAgentSessionFactory::load_production().expect("Gold factory");
    let swarm = SwarmDisasterActivityAgentSessionFactory::load_production().expect("Swarm factory");
    let clock = Arc::new(TestClock);
    let ids = Arc::new(TestIds::default());
    let battle_registry = AgentSessionRegistry::new(factory.clone(), clock.clone(), ids.clone());
    let activity_registry =
        ActivityAgentSessionRegistry::new_with_modes(standard.clone(), gold, swarm, clock, ids);
    let server = StarclockMcp::new(
        battle_registry,
        factory,
        activity_registry,
        standard,
        AgentSessionOwner::new("local", "swarm-test").expect("owner"),
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

    let unknown_mode = client
        .call_tool(
            CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                json!({
                    "schema_revision":"agent-api-v1", "mode":"swarm", "seed":"20001"
                }),
            )),
        )
        .await
        .expect("unknown mode response");
    assert_eq!(unknown_mode.is_error, Some(true));
    assert!(
        unknown_mode.structured_content.expect("unknown mode error")["message"]
            .as_str()
            .expect("error message")
            .contains("mode")
    );

    let incompatible_entry = client
        .call_tool(
            CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                json!({
                    "schema_revision":"agent-api-v1", "mode":"swarm-disaster",
                    "world":"401", "difficulty_index":"0", "seed":"20001"
                }),
            )),
        )
        .await
        .expect("incompatible entry response");
    assert_eq!(incompatible_entry.is_error, Some(true));
    assert!(
        incompatible_entry
            .structured_content
            .expect("incompatible entry error")["message"]
            .as_str()
            .expect("error message")
            .contains("incompatible")
    );

    let manifest = client
        .read_resource(ReadResourceRequestParams::new(
            "starclock://universe/swarm-disaster/manifest",
        ))
        .await
        .expect("manifest");
    let manifest = serde_json::to_string(&manifest).expect("manifest JSON");
    assert!(manifest.contains("swarm-disaster.profile.v1"));
    assert!(manifest.contains("SyntheticBalanceIndependentNotObservedNumericParity"));
    let rules = client
        .read_resource(ReadResourceRequestParams::new(
            "starclock://rules/swarm-disaster",
        ))
        .await
        .expect("rules");
    assert!(
        serde_json::to_string(&rules)
            .expect("rules JSON")
            .contains("authoritative_real_combat_settlement")
    );

    let created = client
        .call_tool(
            CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                json!({
                    "schema_revision":"agent-api-v1", "mode":"swarm-disaster",
                    "world":"201", "difficulty_index":"0", "seed":"20001"
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
    let mut external_actions = 0_u64;
    let mut nested_battles = 0_u64;

    while observation["status"] != "completed" {
        let action = selected_action(&observation).clone();
        let input = json!({
            "schema_revision":"agent-api-v1", "session_id":session_id,
            "boundary_id":observation["boundary_id"],
            "expected_state_hash":observation["state_hash"], "action_token":action["token"],
            "idempotency_key":format!("swarm_mcp_action_{external_actions}")
        });
        let played = client
            .call_tool(
                CallToolRequestParams::new("starclock_play_activity_action")
                    .with_arguments(arguments(input.clone())),
            )
            .await
            .expect("play");
        assert_eq!(played.is_error, Some(false));
        if external_actions == 0 {
            let repeated = client
                .call_tool(
                    CallToolRequestParams::new("starclock_play_activity_action")
                        .with_arguments(arguments(input)),
                )
                .await
                .expect("retry");
            assert_eq!(repeated.structured_content, played.structured_content);
        }
        let response = &played.structured_content.expect("content")["response"];
        nested_battles += response["settlement"]["nested_battles"]
            .as_str()
            .expect("battle count")
            .parse::<u64>()
            .expect("integer");
        observation = response["observation"].clone();
        external_actions += 1;
    }
    assert_eq!(external_actions, 27);
    assert_eq!(nested_battles, 11);
    assert_eq!(observation["state_hash"], FINAL_STATE);

    let exported = client
        .call_tool(
            CallToolRequestParams::new("starclock_export_activity_replay").with_arguments(
                arguments(json!({
                    "schema_revision":"agent-api-v1", "session_id":session_id
                })),
            ),
        )
        .await
        .expect("export");
    let export = exported.structured_content.expect("content");
    assert_eq!(export["action_count"], "48");
    assert_eq!(export["sha256"], REPLAY_SHA256);
    assert_eq!(
        export["replay_hex"].as_str().expect("hex").len(),
        81_086 * 2
    );

    let verified = client
        .call_tool(
            CallToolRequestParams::new("starclock_verify_activity_replay").with_arguments(
                arguments(json!({
                    "schema_revision":"agent-api-v1", "mode":"swarm-disaster",
                    "world":"201", "difficulty_index":"0", "seed":"20001",
                    "replay_hex":export["replay_hex"]
                })),
            ),
        )
        .await
        .expect("verify");
    let verification = verified.structured_content.expect("content");
    assert_eq!(verification["action_count"], "48");
    assert_eq!(verification["nested_battles"], "12");
    assert_eq!(verification["final_state_hash"], FINAL_STATE);

    let closed = client
        .call_tool(
            CallToolRequestParams::new("starclock_close_activity").with_arguments(arguments(
                json!({
                    "schema_revision":"agent-api-v1", "session_id":session_id
                }),
            )),
        )
        .await
        .expect("close");
    assert_eq!(closed.structured_content.expect("content")["closed"], true);
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
