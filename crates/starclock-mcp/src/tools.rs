//! Frozen tool names, input schemas and registry/application delegation.

use core::str::FromStr;

use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult, tool,
    tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starclock_agent_api::{
    error::{AgentError, AgentErrorCode},
    observation::VisibilityPolicy,
    schema::{
        ActionToken, AgentHash, AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey,
        ScenarioId, SessionId,
    },
    session::{AgentSeedPolicy, PlayActionRequest, RegistryCreateSessionRequest},
};

use crate::{error::structured_result, server::StarclockMcp};

pub const MAX_REPLAY_IMPORT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EmptyInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateBattleInput {
    pub schema_revision: String,
    pub scenario_id: String,
    pub seed: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SessionInput {
    pub schema_revision: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ObserveBattleInput {
    pub schema_revision: String,
    pub session_id: String,
    pub event_cursor: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlayActionInput {
    pub schema_revision: String,
    pub session_id: String,
    pub decision_id: String,
    pub expected_state_hash: String,
    pub action_token: String,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct VerifyReplayInput {
    pub schema_revision: String,
    pub scenario_id: String,
    pub seed: Option<String>,
    pub replay_hex: String,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct ScenarioSummaryOutput {
    pub scenario_id: String,
    pub scenario_definition_id: String,
    pub encounter_definition_id: String,
    pub default_seed: String,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct ListScenariosOutput {
    pub schema_revision: String,
    pub scenarios: Vec<ScenarioSummaryOutput>,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct ObservationOutput {
    #[schemars(with = "ObservationSchema")]
    pub observation: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct ActionOutput {
    #[schemars(with = "ActionResponseSchema")]
    pub response: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct ReplayExportOutput {
    pub schema_revision: String,
    pub session_id: String,
    pub encoding: String,
    pub replay_hex: String,
    pub sha256: String,
    pub command_count: String,
    #[schemars(with = "Vec<AcceptedCommandSchema>")]
    pub diagnostics: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct CloseBattleOutput {
    pub schema_revision: String,
    pub session_id: String,
    pub closed: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
pub struct VerifyReplayOutput {
    pub schema_revision: String,
    pub command_count: String,
    pub final_state_hash: String,
    pub phase: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ActionKindSchema {
    UseAbility,
    UseInterrupt,
    PassInterrupt,
    Concede,
    BattleChoice,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum TeamSideSchema {
    Player,
    Enemy,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum LifeStateSchema {
    Alive,
    Defeated,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum PresenceStateSchema {
    Present,
    Untargetable,
    Linked,
    Reserved,
    Departed,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum EffectCategorySchema {
    Buff,
    Debuff,
    Control,
    Dot,
    Mark,
    Field,
    Shield,
    NeutralState,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum BattlePhaseSchema {
    AwaitingCommand,
    Won,
    Lost,
    Faulted,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum BattleStatusSchema {
    AwaitingPlayer,
    Won,
    Lost,
    Faulted,
    Closed,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
#[serde(rename_all = "snake_case")]
enum ControllerKindSchema {
    ExternalPlayer,
    AuthoredEnemy,
    SystemAutomatic,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct OfferedActionSchema {
    token: String,
    kind: ActionKindSchema,
    label: String,
    actor_unit_id: Option<String>,
    primary_target_unit_id: Option<String>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct WaveSchema {
    number: String,
    total: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct TeamSchema {
    side: TeamSideSchema,
    skill_points: String,
    maximum_skill_points: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct UnitSchema {
    unit_id: String,
    side: TeamSideSchema,
    formation: String,
    life: LifeStateSchema,
    presence: PresenceStateSchema,
    current_hp: String,
    maximum_hp: String,
    current_energy_scaled: String,
    maximum_energy_scaled: String,
    weakness_broken: bool,
    public_intent: Option<String>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct EffectSchema {
    effect_id: String,
    target_unit_id: String,
    category: EffectCategorySchema,
    stacks: String,
    remaining: Option<String>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct TimelineSchema {
    actor_id: String,
    owner_unit_id: String,
    active: bool,
    action_gauge_scaled: String,
    speed_scaled: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct BattleSchema {
    phase: BattlePhaseSchema,
    committed_revision: String,
    rng_draw_count: String,
    wave: WaveSchema,
    teams: Vec<TeamSchema>,
    units: Vec<UnitSchema>,
    effects: Vec<EffectSchema>,
    timeline: Vec<TimelineSchema>,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct EventSchema {
    event_id: String,
    kind: String,
    summary: String,
    root_command_id: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct ObservationSchema {
    schema_revision: String,
    session_id: String,
    scenario_id: String,
    catalog_digest: String,
    decision_id: Option<String>,
    state_hash: String,
    event_cursor: String,
    visibility_policy: String,
    status: BattleStatusSchema,
    battle: BattleSchema,
    legal_actions: Vec<OfferedActionSchema>,
    events: Vec<EventSchema>,
    events_truncated: bool,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct SettlementSchema {
    accepted_commands: String,
    emitted_events: String,
    resolver_operations: String,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct ActionResponseSchema {
    schema_revision: String,
    session_id: String,
    committed: bool,
    idempotent_replay: bool,
    accepted_action_token: String,
    settlement: SettlementSchema,
    observation: ObservationSchema,
}

#[allow(dead_code)]
#[derive(JsonSchema)]
struct AcceptedCommandSchema {
    sequence: String,
    decision_id: String,
    controller: ControllerKindSchema,
    resulting_state_hash: String,
}

#[tool_router(router = tool_router)]
impl StarclockMcp {
    pub(crate) fn registered_tool_router() -> rmcp::handler::server::router::tool::ToolRouter<Self>
    {
        Self::tool_router()
    }

    #[tool(
        name = "starclock_list_scenarios",
        description = "List the six frozen production Standard battle scenarios.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<ListScenariosOutput>()
    )]
    async fn list_scenarios(
        &self,
        Parameters(_input): Parameters<EmptyInput>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(self.list_scenarios_output())
    }

    #[tool(
        name = "starclock_create_battle",
        description = "Create one owned ephemeral Standard battle with default or exact seed.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<ObservationOutput>()
    )]
    async fn create_battle(
        &self,
        Parameters(input): Parameters<CreateBattleInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(
            self.owner_for_context(&context)
                .and_then(|owner| self.create_battle_output(&owner, input)),
        )
    }

    #[tool(
        name = "starclock_observe_battle",
        description = "Read the current bounded player-visible observation after an optional event cursor.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<ObservationOutput>()
    )]
    async fn observe_battle(
        &self,
        Parameters(input): Parameters<ObserveBattleInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(
            self.owner_for_context(&context)
                .and_then(|owner| self.observe_battle_output(&owner, input)),
        )
    }

    #[tool(
        name = "starclock_play_action",
        description = "Submit one currently offered opaque action with exact decision, hash and idempotency preconditions.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<ActionOutput>()
    )]
    async fn play_action(
        &self,
        Parameters(input): Parameters<PlayActionInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(
            self.owner_for_context(&context)
                .and_then(|owner| self.play_action_output(&owner, input)),
        )
    }

    #[tool(
        name = "starclock_export_replay",
        description = "Export the complete canonical replay and nonauthoritative controller diagnostics as lowercase hex.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<ReplayExportOutput>()
    )]
    async fn export_replay(
        &self,
        Parameters(input): Parameters<SessionInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(
            self.owner_for_context(&context)
                .and_then(|owner| self.export_replay_output(&owner, input)),
        )
    }

    #[tool(
        name = "starclock_close_battle",
        description = "Close an owned battle session and release its active quota capacity.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<CloseBattleOutput>()
    )]
    async fn close_battle(
        &self,
        Parameters(input): Parameters<SessionInput>,
        context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(
            self.owner_for_context(&context)
                .and_then(|owner| self.close_battle_output(&owner, input)),
        )
    }

    #[tool(
        name = "starclock_verify_replay",
        description = "Verify a bounded canonical replay against a fresh exact Standard scenario without a model.",
        output_schema = rmcp::handler::server::tool::schema_for_type::<VerifyReplayOutput>()
    )]
    async fn verify_replay(
        &self,
        Parameters(input): Parameters<VerifyReplayInput>,
    ) -> Result<CallToolResult, McpError> {
        structured_result(self.verify_replay_output(input))
    }

    fn list_scenarios_output(&self) -> Result<ListScenariosOutput, AgentError> {
        let scenarios = self
            .factory
            .list_scenarios()?
            .iter()
            .map(|scenario| ScenarioSummaryOutput {
                scenario_id: scenario.scenario_id.as_str().into(),
                scenario_definition_id: scenario.scenario_definition_id.as_str().into(),
                encounter_definition_id: scenario.encounter_definition_id.as_str().into(),
                default_seed: scenario.default_seed.as_str().into(),
            })
            .collect();
        Ok(ListScenariosOutput {
            schema_revision: schema_revision(),
            scenarios,
        })
    }

    fn create_battle_output(
        &self,
        owner: &starclock_agent_api::session::AgentSessionOwner,
        input: CreateBattleInput,
    ) -> Result<ObservationOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let observation = self.registry.create(
            owner,
            RegistryCreateSessionRequest {
                scenario_id: parse_scenario(&input.scenario_id)?,
                seed: parse_seed(input.seed.as_deref())?,
                visibility_policy: VisibilityPolicy::PlayerVisible,
            },
        )?;
        json_output(observation).map(|observation| ObservationOutput { observation })
    }

    fn observe_battle_output(
        &self,
        owner: &starclock_agent_api::session::AgentSessionOwner,
        input: ObserveBattleInput,
    ) -> Result<ObservationOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let session_id = parse_session(&input.session_id)?;
        let cursor = EventCursor::parse(input.event_cursor.as_deref().unwrap_or("event_0"))
            .map_err(|_| invalid_request("The event cursor is invalid."))?;
        let observation = self.registry.observe(owner, &session_id, &cursor)?;
        json_output(observation).map(|observation| ObservationOutput { observation })
    }

    fn play_action_output(
        &self,
        owner: &starclock_agent_api::session::AgentSessionOwner,
        input: PlayActionInput,
    ) -> Result<ActionOutput, AgentError> {
        let revision = parse_revision(&input.schema_revision)?;
        let response = self.registry.apply_action(
            owner,
            PlayActionRequest {
                schema_revision: revision,
                session_id: parse_session(&input.session_id)?,
                decision_id: AgentUInt::parse(&input.decision_id)
                    .map_err(|_| invalid_request("The decision ID is invalid."))?,
                expected_state_hash: AgentHash::parse(&input.expected_state_hash)
                    .map_err(|_| invalid_request("The expected state hash is invalid."))?,
                action_token: ActionToken::parse(&input.action_token)
                    .map_err(|_| invalid_request("The action token is invalid."))?,
                idempotency_key: IdempotencyKey::parse(&input.idempotency_key)
                    .map_err(|_| invalid_request("The idempotency key is invalid."))?,
            },
        )?;
        json_output(response).map(|response| ActionOutput { response })
    }

    fn export_replay_output(
        &self,
        owner: &starclock_agent_api::session::AgentSessionOwner,
        input: SessionInput,
    ) -> Result<ReplayExportOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let session_id = parse_session(&input.session_id)?;
        let export = self.registry.export_replay(owner, &session_id)?;
        Ok(ReplayExportOutput {
            schema_revision: schema_revision(),
            session_id: session_id.as_str().into(),
            encoding: "lowercase_hex".into(),
            replay_hex: encode_hex(export.bytes()),
            sha256: export.sha256().as_str().into(),
            command_count: export.diagnostics().len().to_string(),
            diagnostics: json_output(export.diagnostics())?,
        })
    }

    fn close_battle_output(
        &self,
        owner: &starclock_agent_api::session::AgentSessionOwner,
        input: SessionInput,
    ) -> Result<CloseBattleOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let session_id = parse_session(&input.session_id)?;
        self.registry.close(owner, &session_id)?;
        Ok(CloseBattleOutput {
            schema_revision: schema_revision(),
            session_id: session_id.as_str().into(),
            closed: true,
        })
    }

    fn verify_replay_output(
        &self,
        input: VerifyReplayInput,
    ) -> Result<VerifyReplayOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let bytes = decode_hex(&input.replay_hex)?;
        let verification = self.factory.verify_replay(
            &parse_scenario(&input.scenario_id)?,
            &parse_seed(input.seed.as_deref())?,
            &bytes,
        )?;
        let phase = serde_json::to_value(verification.phase)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .ok_or_else(adapter_error)?;
        Ok(VerifyReplayOutput {
            schema_revision: schema_revision(),
            command_count: verification.command_count.as_str().into(),
            final_state_hash: verification.final_state_hash.as_str().into(),
            phase,
        })
    }
}

fn parse_revision(value: &str) -> Result<AgentSchemaRevision, AgentError> {
    AgentSchemaRevision::from_str(value).map_err(|_| unknown_revision())
}

fn parse_session(value: &str) -> Result<SessionId, AgentError> {
    SessionId::parse(value).map_err(|_| invalid_request("The session ID is invalid."))
}

fn parse_scenario(value: &str) -> Result<ScenarioId, AgentError> {
    ScenarioId::parse(value).map_err(|_| invalid_request("The scenario ID is invalid."))
}

fn parse_seed(value: Option<&str>) -> Result<AgentSeedPolicy, AgentError> {
    value.map_or(Ok(AgentSeedPolicy::ScenarioDefault), |value| {
        AgentUInt::parse(value)
            .map(AgentSeedPolicy::Explicit)
            .map_err(|_| invalid_request("The exact seed is invalid."))
    })
}

fn json_output(value: impl Serialize) -> Result<Value, AgentError> {
    serde_json::to_value(value).map_err(|_| adapter_error())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn decode_hex(value: &str) -> Result<Vec<u8>, AgentError> {
    decode_hex_bounded(value, MAX_REPLAY_IMPORT_BYTES)
}

fn decode_hex_bounded(value: &str, maximum_bytes: usize) -> Result<Vec<u8>, AgentError> {
    if value.len() > maximum_bytes.saturating_mul(2) {
        return Err(request_too_large());
    }
    if !value.len().is_multiple_of(2) {
        return Err(invalid_request("The replay hex length is invalid."));
    }
    let mut decoded = Vec::with_capacity(value.len() / 2);
    for pair in value.as_bytes().chunks_exact(2) {
        let high =
            hex_nibble(pair[0]).ok_or_else(|| invalid_request("The replay hex is invalid."))?;
        let low =
            hex_nibble(pair[1]).ok_or_else(|| invalid_request("The replay hex is invalid."))?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn schema_revision() -> String {
    AgentSchemaRevision::V1.as_str().into()
}

fn invalid_request(message: &'static str) -> AgentError {
    AgentError::new(AgentErrorCode::InvalidRequest, message, false, false)
        .expect("static MCP validation error is bounded")
}

fn unknown_revision() -> AgentError {
    AgentError::new(
        AgentErrorCode::UnknownRevision,
        "The agent schema revision is unknown.",
        false,
        false,
    )
    .expect("static MCP revision error is bounded")
}

fn request_too_large() -> AgentError {
    AgentError::new(
        AgentErrorCode::RequestTooLarge,
        "The replay exceeds the MCP import limit.",
        false,
        false,
    )
    .expect("static MCP request limit error is bounded")
}

fn adapter_error() -> AgentError {
    AgentError::new(
        AgentErrorCode::AdapterFailure,
        "The MCP adapter could not serialize a bounded application value.",
        false,
        false,
    )
    .expect("static MCP adapter error is bounded")
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    };

    use rmcp::{
        ServerHandler, ServiceExt,
        model::{CallToolRequestParams, GetPromptRequestParams, ReadResourceRequestParams},
    };
    use serde_json::{Map, json};
    use starclock_agent_api::session::{OperationalClock, SessionIdSource};

    use super::*;

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
                "session_mcp_{}",
                self.0.fetch_add(1, Ordering::Relaxed) + 1
            ))
            .map_err(|_| adapter_error())
        }
    }

    fn arguments(value: Value) -> Map<String, Value> {
        value.as_object().unwrap().clone()
    }

    #[test]
    fn exact_revision_and_bounded_lowercase_hex_fail_closed() {
        assert_eq!(
            parse_revision("agent-api-v2").unwrap_err().code,
            AgentErrorCode::UnknownRevision
        );
        assert_eq!(decode_hex("00ff").unwrap(), [0, 255]);
        assert_eq!(
            decode_hex("00FF").unwrap_err().code,
            AgentErrorCode::InvalidRequest
        );
        assert_eq!(
            decode_hex("0").unwrap_err().code,
            AgentErrorCode::InvalidRequest
        );
        assert_eq!(
            decode_hex_bounded("0000", 1).unwrap_err().code,
            AgentErrorCode::RequestTooLarge
        );
    }

    #[tokio::test]
    async fn seven_tools_discover_and_complete_create_play_export_verify_close() {
        let factory = starclock_agent_api::session::AgentSessionFactory::load_production().unwrap();
        let registry = starclock_agent_api::session::AgentSessionRegistry::new(
            factory.clone(),
            Arc::new(TestClock),
            Arc::new(TestIds::default()),
        );
        let server = StarclockMcp::new(
            registry,
            factory,
            starclock_agent_api::session::AgentSessionOwner::new("local", "test").unwrap(),
        );
        let info = server.get_info();
        assert!(info.capabilities.tools.is_some());
        let resource_capability = info.capabilities.resources.unwrap();
        assert_eq!(resource_capability.subscribe, None);
        assert_eq!(resource_capability.list_changed, None);
        assert_eq!(info.capabilities.prompts.unwrap().list_changed, None);
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
        let tools = client.list_all_tools().await.unwrap();
        let mut names: Vec<_> = tools.iter().map(|tool| tool.name.as_ref()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "starclock_close_battle",
                "starclock_create_battle",
                "starclock_export_replay",
                "starclock_list_scenarios",
                "starclock_observe_battle",
                "starclock_play_action",
                "starclock_verify_replay",
            ]
        );
        assert!(tools.iter().all(|tool| tool.output_schema.is_some()));
        for (tool_name, required_fragments) in [
            (
                "starclock_create_battle",
                ["observation", "legal_actions", "battle"],
            ),
            (
                "starclock_play_action",
                ["response", "settlement", "observation"],
            ),
            (
                "starclock_export_replay",
                ["diagnostics", "controller", "resulting_state_hash"],
            ),
        ] {
            let tool = tools.iter().find(|tool| tool.name == tool_name).unwrap();
            let schema = serde_json::to_string(tool.output_schema.as_ref().unwrap()).unwrap();
            assert!(
                required_fragments
                    .into_iter()
                    .all(|fragment| schema.contains(fragment)),
                "{tool_name} has an incomplete nested output schema: {schema}"
            );
        }

        let resources = client.list_all_resources().await.unwrap();
        assert_eq!(
            resources
                .iter()
                .map(|resource| resource.uri.as_str())
                .collect::<Vec<_>>(),
            [
                "starclock://catalog/manifest",
                "starclock://rules/core-combat"
            ]
        );
        let templates = client.list_all_resource_templates().await.unwrap();
        assert_eq!(
            templates
                .iter()
                .map(|template| template.uri_template.as_str())
                .collect::<Vec<_>>(),
            [
                "starclock://scenario/{scenario_id}",
                "starclock://character/{form_id}"
            ]
        );
        let scenario_resource = client
            .read_resource(ReadResourceRequestParams::new(
                "starclock://scenario/scenario.standard-v1.basic-single-wave",
            ))
            .await
            .unwrap();
        let scenario_json = serde_json::to_string(&scenario_resource).unwrap();
        assert!(scenario_json.contains("scenario_definition_id"));
        assert!(scenario_json.contains("inert_data"));

        let prompts = client.list_all_prompts().await.unwrap();
        assert_eq!(
            prompts
                .iter()
                .map(|prompt| prompt.name.as_str())
                .collect::<Vec<_>>(),
            ["starclock_battle_loop"]
        );
        let prompt = client
            .get_prompt(GetPromptRequestParams::new("starclock_battle_loop"))
            .await
            .unwrap();
        assert_eq!(prompt.messages.len(), 1);
        assert!(
            serde_json::to_string(&prompt)
                .unwrap()
                .contains("grants no authorization")
        );

        let listed = client
            .call_tool(
                CallToolRequestParams::new("starclock_list_scenarios").with_arguments(Map::new()),
            )
            .await
            .unwrap();
        assert_eq!(
            listed.structured_content.as_ref().unwrap()["scenarios"]
                .as_array()
                .unwrap()
                .len(),
            6
        );
        let created = client
            .call_tool(
                CallToolRequestParams::new("starclock_create_battle").with_arguments(arguments(
                    json!({
                        "schema_revision":"agent-api-v1",
                        "scenario_id":"scenario.standard-v1.basic-single-wave"
                    }),
                )),
            )
            .await
            .unwrap();
        let observation = &created.structured_content.as_ref().unwrap()["observation"];
        let session_id = observation["session_id"].as_str().unwrap().to_owned();
        let action = observation["legal_actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|action| action["kind"] != "concede")
            .unwrap();
        let played = client
            .call_tool(
                CallToolRequestParams::new("starclock_play_action").with_arguments(arguments(
                    json!({
                        "schema_revision":"agent-api-v1",
                        "session_id":session_id,
                        "decision_id":observation["decision_id"],
                        "expected_state_hash":observation["state_hash"],
                        "action_token":action["token"],
                        "idempotency_key":"mcp_action_1"
                    }),
                )),
            )
            .await
            .unwrap();
        assert_eq!(played.is_error, Some(false));
        let observed = client
            .call_tool(
                CallToolRequestParams::new("starclock_observe_battle").with_arguments(arguments(
                    json!({
                        "schema_revision":"agent-api-v1", "session_id":session_id
                    }),
                )),
            )
            .await
            .unwrap();
        assert_eq!(observed.is_error, Some(false));
        let exported = client
            .call_tool(
                CallToolRequestParams::new("starclock_export_replay").with_arguments(arguments(
                    json!({
                        "schema_revision":"agent-api-v1", "session_id":session_id
                    }),
                )),
            )
            .await
            .unwrap();
        let replay_hex = exported.structured_content.as_ref().unwrap()["replay_hex"]
            .as_str()
            .unwrap()
            .to_owned();
        let final_state_hash =
            played.structured_content.as_ref().unwrap()["response"]["observation"]["state_hash"]
                .as_str()
                .unwrap()
                .to_owned();
        let closed = client
            .call_tool(
                CallToolRequestParams::new("starclock_close_battle").with_arguments(arguments(
                    json!({
                        "schema_revision":"agent-api-v1", "session_id":session_id
                    }),
                )),
            )
            .await
            .unwrap();
        assert_eq!(closed.structured_content.as_ref().unwrap()["closed"], true);

        let verified = client
            .call_tool(
                CallToolRequestParams::new("starclock_verify_replay").with_arguments(arguments(
                    json!({
                        "schema_revision":"agent-api-v1",
                        "scenario_id":"scenario.standard-v1.basic-single-wave",
                        "replay_hex":replay_hex
                    }),
                )),
            )
            .await;
        let verified = verified.unwrap();
        assert_eq!(verified.is_error, Some(false));
        assert_eq!(
            verified.structured_content.as_ref().unwrap()["final_state_hash"],
            final_state_hash
        );

        client.cancel().await.unwrap();
        task.await.unwrap();
    }
}
