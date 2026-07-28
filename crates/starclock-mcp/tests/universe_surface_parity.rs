use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use rmcp::{ServiceExt, model::CallToolRequestParams};
use serde_json::{Map, Value, json};
use starclock_agent_api::{
    error::AgentError,
    schema::SessionId,
    session::{
        AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry, OperationalClock,
        SessionIdSource,
    },
};
use starclock_mcp::server::StarclockMcp;

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
            "session_mcp_parity_{}",
            self.0.fetch_add(1, Ordering::Relaxed) + 1
        ))
        .map_err(|error| panic!("test session ID is valid: {error:?}"))
    }
}

fn arguments(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

fn selected(observation: &Value) -> &Value {
    let actions = observation["legal_actions"].as_array().unwrap();
    if let Some(engage) = actions
        .iter()
        .find(|action| action["kind"] == "engage_battle")
    {
        return engage;
    }
    actions
        .iter()
        .max_by(|left, right| {
            let priority = |value: &Value| {
                value["priority"]
                    .as_str()
                    .map_or(0, |text| text.parse::<i64>().unwrap())
            };
            let option =
                |value: &Value| value["option_id"].as_str().unwrap().parse::<u64>().unwrap();
            priority(left)
                .cmp(&priority(right))
                .then_with(|| option(right).cmp(&option(left)))
        })
        .unwrap()
}

#[tokio::test]
async fn mcp_activity_surface_matches_agent_replay_and_fresh_verification() {
    let battle_factory = AgentSessionFactory::load_production().unwrap();
    let activity_factory =
        starclock_agent_api::activity_session::ActivityAgentSessionFactory::load_production()
            .unwrap();
    let clock: Arc<dyn OperationalClock> = Arc::new(TestClock);
    let ids: Arc<dyn SessionIdSource> = Arc::new(TestIds::default());
    let server = StarclockMcp::new(
        AgentSessionRegistry::new(battle_factory.clone(), Arc::clone(&clock), Arc::clone(&ids)),
        battle_factory,
        starclock_agent_api::activity_session::registry::ActivityAgentSessionRegistry::new(
            activity_factory.clone(),
            clock,
            ids,
        ),
        activity_factory,
        AgentSessionOwner::new("local", "parity").unwrap(),
    );
    let (server_transport, client_transport) = tokio::io::duplex(256 * 1024);
    let task = tokio::spawn(async move {
        server
            .serve(server_transport)
            .await
            .unwrap()
            .waiting()
            .await
            .unwrap();
    });
    let client = ().serve(client_transport).await.unwrap();

    let created = client
        .call_tool(
            CallToolRequestParams::new("starclock_create_universe").with_arguments(arguments(
                json!({
                    "schema_revision":"agent-api-v1",
                    "world":"1",
                    "difficulty_index":"0",
                    "seed":"10"
                }),
            )),
        )
        .await
        .unwrap();
    let mut observation = created.structured_content.unwrap()["observation"].clone();
    let session_id = observation["session_id"].as_str().unwrap().to_owned();
    for sequence in 0_u64..1_000 {
        if observation["status"] == "completed" {
            break;
        }
        let action = selected(&observation);
        let played = client
            .call_tool(
                CallToolRequestParams::new("starclock_play_activity_action").with_arguments(
                    arguments(json!({
                        "schema_revision":"agent-api-v1",
                        "session_id":session_id,
                        "boundary_id":observation["boundary_id"],
                        "expected_state_hash":observation["state_hash"],
                        "action_token":action["token"],
                        "idempotency_key":format!("mcp_parity_{sequence}")
                    })),
                ),
            )
            .await
            .unwrap();
        assert_eq!(played.is_error, Some(false));
        observation = played.structured_content.unwrap()["response"]["observation"].clone();
    }
    assert_eq!(observation["status"], "completed");
    assert_eq!(
        observation["state_hash"],
        "7c59f6648b8c7301081a0d26d548b73a7cf86bbe95fc9a863e0d40807bbbddb6"
    );

    let exported = client
        .call_tool(
            CallToolRequestParams::new("starclock_export_activity_replay").with_arguments(
                arguments(json!({
                    "schema_revision":"agent-api-v1",
                    "session_id":session_id
                })),
            ),
        )
        .await
        .unwrap();
    let export = exported.structured_content.unwrap();
    assert_eq!(export["complete"], true);
    assert_eq!(
        export["sha256"],
        "6babfd2e4c695b6d5def1e442b481351dcd607b8e138ad00a0ba13a70301efc7"
    );

    let verified = client
        .call_tool(
            CallToolRequestParams::new("starclock_verify_activity_replay").with_arguments(
                arguments(json!({
                    "schema_revision":"agent-api-v1",
                    "world":"1",
                    "difficulty_index":"0",
                    "seed":"10",
                    "replay_hex":export["replay_hex"]
                })),
            ),
        )
        .await
        .unwrap();
    assert_eq!(verified.is_error, Some(false));
    let verification = verified.structured_content.unwrap();
    assert_eq!(verification["final_state_hash"], observation["state_hash"]);
    assert_eq!(verification["nested_battles"], "6");

    client.cancel().await.unwrap();
    task.await.unwrap();
}
