//! Additive Standard Universe Activity MCP DTOs and facade delegation.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starclock_agent_api::{
    activity_session::{PlayActivityActionRequest, registry::RegistryCreateActivitySessionRequest},
    error::AgentError,
    schema::{ActionToken, AgentHash, AgentUInt, IdempotencyKey},
    session::AgentSessionOwner,
};

use crate::{
    server::StarclockMcp,
    tools::{
        MAX_REPLAY_IMPORT_BYTES, decode_hex_bounded, encode_hex, invalid_request, json_output,
        parse_revision, parse_session, schema_revision,
    },
};

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct CreateUniverseInput {
    pub schema_revision: String,
    pub world: String,
    pub difficulty_index: String,
    pub seed: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct ActivitySessionInput {
    pub schema_revision: String,
    pub session_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct PlayActivityActionInput {
    pub schema_revision: String,
    pub session_id: String,
    pub boundary_id: String,
    pub expected_state_hash: String,
    pub action_token: String,
    pub idempotency_key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub(crate) struct VerifyActivityReplayInput {
    pub schema_revision: String,
    pub world: String,
    pub difficulty_index: String,
    pub seed: String,
    pub replay_hex: String,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct ActivityObservationOutput {
    pub observation: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct ActivityActionOutput {
    pub response: Value,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct ActivityReplayExportOutput {
    pub schema_revision: String,
    pub session_id: String,
    pub encoding: String,
    pub replay_hex: String,
    pub sha256: String,
    pub action_count: String,
    pub complete: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct CloseActivityOutput {
    pub schema_revision: String,
    pub session_id: String,
    pub closed: bool,
}

#[derive(Debug, JsonSchema, Serialize)]
pub(crate) struct VerifyActivityReplayOutput {
    pub schema_revision: String,
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
        parse_revision(&input.schema_revision)?;
        let observation = self.activity_registry.create(
            owner,
            RegistryCreateActivitySessionRequest {
                world: uint(&input.world, "The world is invalid.")?,
                difficulty_index: uint(
                    &input.difficulty_index,
                    "The difficulty index is invalid.",
                )?,
                seed: uint(&input.seed, "The seed is invalid.")?,
            },
        )?;
        Ok(ActivityObservationOutput {
            observation: json_output(observation)?,
        })
    }

    pub(crate) fn observe_activity_output(
        &self,
        owner: &AgentSessionOwner,
        input: ActivitySessionInput,
    ) -> Result<ActivityObservationOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let observation = self
            .activity_registry
            .observe(owner, &parse_session(&input.session_id)?)?;
        Ok(ActivityObservationOutput {
            observation: json_output(observation)?,
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
                schema_revision: parse_revision(&input.schema_revision)?,
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
        parse_revision(&input.schema_revision)?;
        let session_id = parse_session(&input.session_id)?;
        let export = self.activity_registry.export_replay(owner, &session_id)?;
        Ok(ActivityReplayExportOutput {
            schema_revision: schema_revision(),
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
        parse_revision(&input.schema_revision)?;
        let session_id = parse_session(&input.session_id)?;
        self.activity_registry.close(owner, &session_id)?;
        Ok(CloseActivityOutput {
            schema_revision: schema_revision(),
            session_id: session_id.as_str().into(),
            closed: true,
        })
    }

    pub(crate) fn verify_activity_replay_output(
        &self,
        input: VerifyActivityReplayInput,
    ) -> Result<VerifyActivityReplayOutput, AgentError> {
        parse_revision(&input.schema_revision)?;
        let verification = self.activity_factory.verify_replay(
            &uint(&input.world, "The world is invalid.")?,
            &uint(&input.difficulty_index, "The difficulty index is invalid.")?,
            &uint(&input.seed, "The seed is invalid.")?,
            &decode_hex_bounded(&input.replay_hex, MAX_REPLAY_IMPORT_BYTES)?,
        )?;
        Ok(VerifyActivityReplayOutput {
            schema_revision: schema_revision(),
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
