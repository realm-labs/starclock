//! Additive Universe Activity MCP DTOs and facade delegation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starclock_agent_api::{
    activity_session::{
        PlayActivityActionRequest,
        registry::{
            RegistryCreateActivitySessionRequest, RegistryCreateCurrencyWarsSessionRequest,
            RegistryCreateGoldAndGearsSessionRequest, RegistryCreateSwarmDisasterSessionRequest,
        },
    },
    currency_wars_activity_session::AgentCurrencyWarsGambit,
    error::AgentError,
    schema::{ActionToken, AgentHash, AgentUInt, EventCursor, IdempotencyKey},
    session::AgentSessionOwner,
};

use crate::{
    server::StarclockMcp,
    tools::{
        MAX_REPLAY_IMPORT_BYTES, decode_hex_bounded, encode_hex, invalid_request, json_output,
        parse_session,
    },
};

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CreateUniverseInput {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub world: Option<String>,
    #[serde(default)]
    pub difficulty_index: Option<String>,
    #[serde(default)]
    pub route_id: Option<String>,
    #[serde(default)]
    pub difficulty_id: Option<String>,
    #[serde(default)]
    pub gambit: Option<String>,
    pub seed: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ActivitySessionInput {
    pub session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ObserveActivityInput {
    pub session_id: String,
    #[serde(default)]
    pub event_cursor: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PlayActivityActionInput {
    pub session_id: String,
    pub boundary_id: String,
    pub expected_state_hash: String,
    pub action_token: String,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct VerifyActivityReplayInput {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub world: Option<String>,
    #[serde(default)]
    pub difficulty_index: Option<String>,
    pub seed: String,
    pub replay_hex: String,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct ActivityObservationOutput {
    pub observation: Value,
    pub event_cursor: String,
    pub events: Value,
    pub events_truncated: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct ActivityActionOutput {
    pub response: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct ActivityReplayExportOutput {
    pub session_id: String,
    pub encoding: String,
    pub replay_hex: String,
    pub sha256: String,
    pub action_count: String,
    pub complete: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct CloseActivityOutput {
    pub session_id: String,
    pub closed: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct VerifyActivityReplayOutput {
    pub action_count: String,
    pub nested_battles: String,
    pub final_state_hash: String,
    pub terminal: Value,
}

impl StarclockMcp {
    pub(crate) fn create_universe_output(
        &self,
        owner: &AgentSessionOwner,
        input: CreateUniverseInput,
    ) -> Result<ActivityObservationOutput, AgentError> {
        let seed = uint(&input.seed, "The seed is invalid.")?;
        let observation = match activity_mode(input.mode.as_deref())? {
            ActivityMode::Standard => {
                validate_currency_entry(
                    input.route_id.as_deref(),
                    input.difficulty_id.as_deref(),
                    input.gambit.as_deref(),
                    false,
                )?;
                self.activity_registry.create(
                    owner,
                    RegistryCreateActivitySessionRequest {
                        world: required_uint(input.world.as_deref(), "The world is invalid.")?,
                        difficulty_index: required_uint(
                            input.difficulty_index.as_deref(),
                            "The difficulty index is invalid.",
                        )?,
                        seed,
                    },
                )?
            }
            ActivityMode::GoldAndGears => {
                validate_gold_entry(input.world.as_deref(), input.difficulty_index.as_deref())?;
                validate_currency_entry(
                    input.route_id.as_deref(),
                    input.difficulty_id.as_deref(),
                    input.gambit.as_deref(),
                    false,
                )?;
                self.activity_registry.create_gold_and_gears(
                    owner,
                    RegistryCreateGoldAndGearsSessionRequest { seed },
                )?
            }
            ActivityMode::SwarmDisaster => {
                validate_swarm_entry(input.world.as_deref(), input.difficulty_index.as_deref())?;
                validate_currency_entry(
                    input.route_id.as_deref(),
                    input.difficulty_id.as_deref(),
                    input.gambit.as_deref(),
                    false,
                )?;
                self.activity_registry.create_swarm_disaster(
                    owner,
                    RegistryCreateSwarmDisasterSessionRequest { seed },
                )?
            }
            ActivityMode::CurrencyWars => {
                if input.world.is_some() || input.difficulty_index.is_some() {
                    return Err(invalid_request(
                        "Currency Wars does not accept Universe entry fields.",
                    ));
                }
                validate_currency_entry(
                    input.route_id.as_deref(),
                    input.difficulty_id.as_deref(),
                    input.gambit.as_deref(),
                    true,
                )?;
                self.activity_registry.create_currency_wars(
                    owner,
                    RegistryCreateCurrencyWarsSessionRequest {
                        route_id: required_uint(
                            input.route_id.as_deref(),
                            "The Currency Wars route ID is invalid.",
                        )?,
                        difficulty_id: required_uint(
                            input.difficulty_id.as_deref(),
                            "The Currency Wars difficulty ID is invalid.",
                        )?,
                        gambit: currency_wars_gambit(input.gambit.as_deref())?,
                        seed,
                    },
                )?
            }
        };
        Ok(ActivityObservationOutput {
            observation: json_output(observation)?,
            event_cursor: "event_0".into(),
            events: json_output(Vec::<Value>::new())?,
            events_truncated: false,
        })
    }

    pub(crate) fn observe_activity_output(
        &self,
        owner: &AgentSessionOwner,
        input: ObserveActivityInput,
    ) -> Result<ActivityObservationOutput, AgentError> {
        let page = self.activity_registry.observe_with_events(
            owner,
            &parse_session(&input.session_id)?,
            &EventCursor::parse(input.event_cursor.as_deref().unwrap_or("event_0"))
                .map_err(|_| invalid_request("The Activity event cursor is invalid."))?,
        )?;
        Ok(ActivityObservationOutput {
            observation: json_output(page.observation)?,
            event_cursor: page.event_cursor.as_str().into(),
            events: json_output(page.events)?,
            events_truncated: page.events_truncated,
        })
    }

    pub(crate) fn play_activity_action_output(
        &self,
        owner: &AgentSessionOwner,
        input: PlayActivityActionInput,
    ) -> Result<ActivityActionOutput, AgentError> {
        let response = self.activity_registry.apply_action(
            owner,
            PlayActivityActionRequest {
                session_id: parse_session(&input.session_id)?,
                boundary_id: uint(&input.boundary_id, "The boundary ID is invalid.")?,
                expected_state_hash: AgentHash::parse(&input.expected_state_hash)
                    .map_err(|_| invalid_request("The expected state hash is invalid."))?,
                action_token: ActionToken::parse(&input.action_token)
                    .map_err(|_| invalid_request("The Activity action token is invalid."))?,
                idempotency_key: IdempotencyKey::parse(&input.idempotency_key)
                    .map_err(|_| invalid_request("The idempotency key is invalid."))?,
            },
        )?;
        Ok(ActivityActionOutput {
            response: json_output(response)?,
        })
    }

    pub(crate) fn export_activity_replay_output(
        &self,
        owner: &AgentSessionOwner,
        input: ActivitySessionInput,
    ) -> Result<ActivityReplayExportOutput, AgentError> {
        let session_id = parse_session(&input.session_id)?;
        let export = self.activity_registry.export_replay(owner, &session_id)?;
        Ok(ActivityReplayExportOutput {
            session_id: session_id.as_str().into(),
            encoding: "lowercase_hex".into(),
            replay_hex: encode_hex(export.bytes()),
            sha256: export.sha256().as_str().into(),
            action_count: export.action_count().as_str().into(),
            complete: export.complete(),
        })
    }

    pub(crate) fn close_activity_output(
        &self,
        owner: &AgentSessionOwner,
        input: ActivitySessionInput,
    ) -> Result<CloseActivityOutput, AgentError> {
        let session_id = parse_session(&input.session_id)?;
        self.activity_registry.close(owner, &session_id)?;
        Ok(CloseActivityOutput {
            session_id: session_id.as_str().into(),
            closed: true,
        })
    }

    pub(crate) fn verify_activity_replay_output(
        &self,
        input: VerifyActivityReplayInput,
    ) -> Result<VerifyActivityReplayOutput, AgentError> {
        let seed = uint(&input.seed, "The seed is invalid.")?;
        let replay = decode_hex_bounded(&input.replay_hex, MAX_REPLAY_IMPORT_BYTES)?;
        let verification = match activity_mode(input.mode.as_deref())? {
            ActivityMode::Standard => self.activity_factory.verify_replay(
                &required_uint(input.world.as_deref(), "The world is invalid.")?,
                &required_uint(
                    input.difficulty_index.as_deref(),
                    "The difficulty index is invalid.",
                )?,
                &seed,
                &replay,
            )?,
            ActivityMode::GoldAndGears => {
                validate_gold_entry(input.world.as_deref(), input.difficulty_index.as_deref())?;
                self.activity_registry
                    .verify_gold_and_gears_replay(&seed, &replay)?
            }
            ActivityMode::SwarmDisaster => {
                validate_swarm_entry(input.world.as_deref(), input.difficulty_index.as_deref())?;
                self.activity_registry
                    .verify_swarm_disaster_replay(&seed, &replay)?
            }
            ActivityMode::CurrencyWars => {
                return Err(invalid_request(
                    "Currency Wars Agent replay verification is not yet available.",
                ));
            }
        };
        Ok(VerifyActivityReplayOutput {
            action_count: verification.action_count.as_str().into(),
            nested_battles: verification.nested_battles.as_str().into(),
            final_state_hash: verification.final_state_hash.as_str().into(),
            terminal: json_output(verification.terminal)?,
        })
    }
}

fn uint(value: &str, message: &'static str) -> Result<AgentUInt, AgentError> {
    AgentUInt::parse(value).map_err(|_| invalid_request(message))
}

fn required_uint(value: Option<&str>, message: &'static str) -> Result<AgentUInt, AgentError> {
    uint(value.ok_or_else(|| invalid_request(message))?, message)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ActivityMode {
    Standard,
    GoldAndGears,
    SwarmDisaster,
    CurrencyWars,
}

fn activity_mode(value: Option<&str>) -> Result<ActivityMode, AgentError> {
    match value {
        None | Some("standard") => Ok(ActivityMode::Standard),
        Some("gold-and-gears") => Ok(ActivityMode::GoldAndGears),
        Some("swarm-disaster") => Ok(ActivityMode::SwarmDisaster),
        Some("currency-wars") => Ok(ActivityMode::CurrencyWars),
        Some(_) => Err(invalid_request("The Universe mode is invalid.")),
    }
}

fn validate_currency_entry(
    route_id: Option<&str>,
    difficulty_id: Option<&str>,
    gambit: Option<&str>,
    required: bool,
) -> Result<(), AgentError> {
    if required {
        if route_id.is_none() || difficulty_id.is_none() || gambit.is_none() {
            return Err(invalid_request(
                "Currency Wars requires route_id, difficulty_id and gambit.",
            ));
        }
    } else if route_id.is_some() || difficulty_id.is_some() || gambit.is_some() {
        return Err(invalid_request(
            "Currency Wars entry fields require mode currency-wars.",
        ));
    }
    Ok(())
}

fn currency_wars_gambit(value: Option<&str>) -> Result<AgentCurrencyWarsGambit, AgentError> {
    match value {
        Some("standard") => Ok(AgentCurrencyWarsGambit::Standard),
        Some("overclock") => Ok(AgentCurrencyWarsGambit::Overclock),
        _ => Err(invalid_request(
            "The Currency Wars Gambit must be standard or overclock.",
        )),
    }
}

fn validate_gold_entry(
    world: Option<&str>,
    difficulty_index: Option<&str>,
) -> Result<(), AgentError> {
    if world.is_some_and(|value| value != "401")
        || difficulty_index.is_some_and(|value| value != "0")
    {
        return Err(invalid_request(
            "The Gold and Gears fixed entry is incompatible.",
        ));
    }
    Ok(())
}

fn validate_swarm_entry(
    world: Option<&str>,
    difficulty_index: Option<&str>,
) -> Result<(), AgentError> {
    if world.is_some_and(|value| value != "201")
        || difficulty_index.is_some_and(|value| value != "0")
    {
        return Err(invalid_request(
            "The Swarm Disaster fixed entry is incompatible.",
        ));
    }
    Ok(())
}
