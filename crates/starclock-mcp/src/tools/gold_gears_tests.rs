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
};

use crate::server::StarclockMcp;

const FINAL_STATE: &str = "fe3c463ffeb94dabbb93d8d7347d53683573e0d3bd966b97df66c60d4c6fd1d7";
const REPLAY_SHA256: &str = "9ee780dec457ae17705ba22a13b4599d25288b64805681fd73b35bfc43509ecb";

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
            "gold_mcp_{}",
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
async fn gold_and_gears_uses_authorized_activity_tools_resources_and_replay() {
    let factory = AgentSessionFactory::load_production().expect("battle factory");
    let standard = ActivityAgentSessionFactory::load_production().expect("Standard factory");
    let gold = GoldAndGearsActivityAgentSessionFactory::load_production().expect("Gold factory");
    let clock = Arc::new(TestClock);
    let ids = Arc::new(TestIds::default());
    let battle_registry = AgentSessionRegistry::new(factory.clone(), clock.clone(), ids.clone());
    let activity_registry =
        ActivityAgentSessionRegistry::new_with_gold_and_gears(standard.clone(), gold, clock, ids);
    let server = StarclockMcp::new(
        battle_registry,
        factory,
        activity_registry,
        standard,
        AgentSessionOwner::new("local", "gold-test").expect("owner"),
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
                                        "mode":"golden-gears",
                    "seed":"14001"
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
                                        "mode":"gold-and-gears",
                    "world":"1",
                    "difficulty_index":"0",
                    "seed":"14001"
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
            "starclock://universe/gold-and-gears/manifest",
        ))
        .await
        .expect("manifest");
    let manifest = serde_json::to_string(&manifest).expect("manifest JSON");
    assert!(manifest.contains("gold-gears.profile.v1"));
    assert!(manifest.contains("SyntheticBalanceIndependentNotObservedNumericParity"));

    let created = client
        .call_tool(
            CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                json!({
                                        "mode":"gold-and-gears",
                    "seed":"14001"
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
                        "session_id":session_id,
            "boundary_id":observation["boundary_id"],
            "expected_state_hash":observation["state_hash"],
            "action_token":action["token"],
            "idempotency_key":format!("gold_mcp_action_{external_actions}")
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
    assert_eq!(external_actions, 42);
    assert_eq!(nested_battles, 17);
    assert_eq!(observation["state_hash"], FINAL_STATE);

    let observed = client
        .call_tool(
            CallToolRequestParams::new("starclock_observe_activity")
                .with_arguments(arguments(json!({"session_id":session_id}))),
        )
        .await
        .expect("observe");
    assert_eq!(
        observed.structured_content.expect("content")["observation"]["state_hash"],
        FINAL_STATE
    );

    let exported = client
        .call_tool(
            CallToolRequestParams::new("starclock_export_activity_replay").with_arguments(
                arguments(json!({
                    "session_id":session_id
                })),
            ),
        )
        .await
        .expect("export");
    let export = exported.structured_content.expect("content");
    assert_eq!(export["action_count"], "62");
    assert_eq!(export["sha256"], REPLAY_SHA256);
    assert_eq!(
        export["replay_hex"].as_str().expect("hex").len(),
        107_261 * 2
    );

    let verified = client
        .call_tool(
            CallToolRequestParams::new("starclock_verify_activity_replay").with_arguments(
                arguments(json!({
                                        "mode":"gold-and-gears",
                    "seed":"14001",
                    "replay_hex":export["replay_hex"]
                })),
            ),
        )
        .await
        .expect("verify");
    let verification = verified.structured_content.expect("content");
    assert_eq!(verification["action_count"], "62");
    assert_eq!(verification["nested_battles"], "17");
    assert_eq!(verification["final_state_hash"], FINAL_STATE);

    let closed = client
        .call_tool(
            CallToolRequestParams::new("starclock_close_activity")
                .with_arguments(arguments(json!({"session_id":session_id}))),
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
