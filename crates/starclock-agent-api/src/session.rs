//! Authoritative ephemeral session and in-memory registry contracts.
//!
//! Sessions compose deterministic Goal 01 libraries while operational identity,
//! time, ownership, expiry, quotas and idempotency remain outside domain state.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starclock_ai::EnemyController;
use starclock_combat::{
    Battle, BattlePhase, BattleSpecDigest, Command, DecisionKind, DecisionOwner, EncounterId,
    TeamSide, UnitDefinitionId, catalog::encounter::AiTransitionTiming, rng::types::RngSeed,
    rule::model::ConditionExpr,
};
use starclock_data::standard_v1::{
    CATALOG_REVISION, CONFIG_DIGEST, RULES_REVISION, SCENARIOS, StandardV1Catalog,
};
use starclock_replay::{
    battle::{BattleTraceEntry, battle_record_count, encode_battle_trace, verify_battle_replay},
    digest::{ConfigBundleDigest, ControllerDigest, EntrySpecDigest},
    format::{ControllerIdentity, ReplayEntry, ReplayHeader, ReplayIdentity, decode_replay},
};

use crate::{
    action::{ActionBindingError, OfferedAction, OfferedActionSet},
    error::{AgentError, AgentErrorCode},
    observation::{
        AgentBattleStatus, AgentEventPage, AgentEventSummary, AgentObservation, VisibilityPolicy,
        project_event_summary, project_player_visible,
    },
    schema::{
        ActionToken, AgentHash, AgentSchemaRevision, AgentUInt, EventCursor, IdempotencyKey,
        ScenarioId, SessionId,
    },
};

mod registry;
pub use registry::{
    AgentSessionOwner, AgentSessionRegistry, IDLE_TTL_SECONDS, MAX_GLOBAL_SESSIONS,
    MAX_SESSIONS_PER_PRINCIPAL, MAX_SESSIONS_PER_TENANT, MAXIMUM_LIFETIME_SECONDS,
    OperationalClock, RegistryCreateSessionRequest, SessionIdSource,
};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "ephemeral authoritative sessions and registry";
pub const MAX_ACCEPTED_COMMANDS_PER_SETTLEMENT: usize = 4_096;
pub const MAX_IDEMPOTENCY_ENTRIES: usize = 1_024;
pub const MAX_CACHED_RESPONSE_BYTES: usize = 512 * 1_024;
pub const MAX_RETAINED_EVENT_SUMMARIES: usize = 8_192;
pub const AGENT_REPLAY_CONTROLLER_REVISION: &str = "agent-standard-session-v1";
const AGENT_REPLAY_CONTROLLER_DESCRIPTOR: &[u8] =
    b"agent-standard-session-v1\0agent-api-v1\0external-player\0authored-enemy\0system-automatic";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSeedPolicy {
    ScenarioDefault,
    Explicit(AgentUInt),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateSessionRequest {
    pub session_id: SessionId,
    pub scenario_id: ScenarioId,
    pub seed: AgentSeedPolicy,
    pub visibility_policy: VisibilityPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayActionRequest {
    pub schema_revision: AgentSchemaRevision,
    pub session_id: SessionId,
    pub decision_id: AgentUInt,
    pub expected_state_hash: AgentHash,
    pub action_token: ActionToken,
    pub idempotency_key: IdempotencyKey,
}

/// Validated shared production catalog factory; mutable battles are never shared.
#[derive(Clone)]
pub struct AgentSessionFactory {
    standard: StandardV1Catalog,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentScenarioSummary {
    pub scenario_id: ScenarioId,
    pub scenario_definition_id: AgentUInt,
    pub encounter_definition_id: AgentUInt,
    pub default_seed: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentCatalogManifest {
    pub catalog_revision: Box<str>,
    pub config_digest: AgentHash,
    pub game_version: Box<str>,
    pub snapshot_date: Box<str>,
    pub data_revision: Box<str>,
    pub rules_revision: Box<str>,
    pub numeric_policy_revision: Box<str>,
    pub rng_algorithm_revision: Box<str>,
    pub state_hash_revision: Box<str>,
    pub replay_format_version: Box<str>,
    pub coverage_manifest_sha256: Box<str>,
    pub identity_count: AgentUInt,
    pub enabled_identity_count: AgentUInt,
    pub ability_count: AgentUInt,
    pub hit_plan_count: AgentUInt,
    pub character_count: AgentUInt,
    pub light_cone_count: AgentUInt,
    pub effect_count: AgentUInt,
    pub ai_graph_count: AgentUInt,
    pub enemy_count: AgentUInt,
    pub encounter_count: AgentUInt,
    pub standard_profile_count: AgentUInt,
    pub standard_scenario_count: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentCharacterSummary {
    pub form_id: AgentUInt,
    pub stat_row_count: AgentUInt,
    pub ability_count: AgentUInt,
    pub resource_count: AgentUInt,
    pub ability_parameter_count: AgentUInt,
    pub trace_count: AgentUInt,
    pub eidolon_count: AgentUInt,
}

impl AgentSessionFactory {
    pub fn load_production() -> Result<Self, AgentError> {
        let standard = StandardV1Catalog::load().map_err(|_| {
            agent_error(
                AgentErrorCode::ConfigurationRejected,
                "The frozen production Standard catalog could not be loaded.",
            )
        })?;
        Ok(Self { standard })
    }

    pub fn create(&self, request: CreateSessionRequest) -> Result<AgentSession, AgentError> {
        let CreateSessionRequest {
            session_id,
            scenario_id,
            seed,
            visibility_policy,
        } = request;
        self.create_with_id_source(scenario_id, seed, visibility_policy, || Ok(session_id))
    }

    pub fn list_scenarios(&self) -> Result<Box<[AgentScenarioSummary]>, AgentError> {
        SCENARIOS
            .into_iter()
            .map(|(scenario, definition, encounter)| {
                let scenario_id = ScenarioId::parse(scenario).map_err(|_| replay_header_error())?;
                let default_seed = self
                    .standard
                    .instantiate(scenario, None)
                    .map_err(|_| replay_header_error())?
                    .master_seed();
                Ok(AgentScenarioSummary {
                    scenario_id,
                    scenario_definition_id: AgentUInt::from_u64(u64::from(definition)),
                    encounter_definition_id: AgentUInt::from_u64(u64::from(encounter)),
                    default_seed: AgentUInt::from_u64(default_seed),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn catalog_manifest(&self) -> Result<AgentCatalogManifest, AgentError> {
        let manifest = self.standard.manifest();
        let summary = self.standard.summary();
        Ok(AgentCatalogManifest {
            catalog_revision: CATALOG_REVISION.into(),
            config_digest: AgentHash::from_bytes(CONFIG_DIGEST),
            game_version: manifest.game_version.as_str().into(),
            snapshot_date: manifest.snapshot_date.as_str().into(),
            data_revision: manifest.data_revision.as_str().into(),
            rules_revision: manifest.required_rules_revision.as_str().into(),
            numeric_policy_revision: manifest.numeric_policy_revision.as_str().into(),
            rng_algorithm_revision: manifest.rng_algorithm_revision.as_str().into(),
            state_hash_revision: manifest.state_hash_revision.as_str().into(),
            replay_format_version: manifest.replay_format_version.as_str().into(),
            coverage_manifest_sha256: manifest.coverage_manifest_sha256.as_str().into(),
            identity_count: agent_count(summary.identity_count)?,
            enabled_identity_count: agent_count(summary.enabled_identity_count)?,
            ability_count: agent_count(summary.ability_count)?,
            hit_plan_count: agent_count(summary.hit_plan_count)?,
            character_count: agent_count(summary.character_count)?,
            light_cone_count: agent_count(summary.light_cone_count)?,
            effect_count: agent_count(summary.effect_count)?,
            ai_graph_count: agent_count(summary.ai_graph_count)?,
            enemy_count: agent_count(summary.enemy_count)?,
            encounter_count: agent_count(summary.encounter_count)?,
            standard_profile_count: agent_count(summary.standard_profile_count)?,
            standard_scenario_count: agent_count(summary.standard_scenario_count)?,
        })
    }

    pub fn character_summary(
        &self,
        form_id: &AgentUInt,
    ) -> Result<Option<AgentCharacterSummary>, AgentError> {
        let Ok(raw) = u32::try_from(form_id.to_u64()) else {
            return Ok(None);
        };
        let Some(id) = UnitDefinitionId::new(raw) else {
            return Ok(None);
        };
        self.standard
            .character(id)
            .map(|character| {
                Ok(AgentCharacterSummary {
                    form_id: form_id.clone(),
                    stat_row_count: agent_count(character.stat_row_count())?,
                    ability_count: agent_count(character.ability_count())?,
                    resource_count: agent_count(character.resource_count())?,
                    ability_parameter_count: agent_count(character.ability_parameter_count())?,
                    trace_count: agent_count(character.trace_count())?,
                    eidolon_count: agent_count(character.eidolon_count())?,
                })
            })
            .transpose()
    }

    pub fn verify_replay(
        &self,
        scenario_id: &ScenarioId,
        seed: &AgentSeedPolicy,
        bytes: &[u8],
    ) -> Result<AgentReplayVerification, AgentError> {
        let seed_override = match seed {
            AgentSeedPolicy::ScenarioDefault => None,
            AgentSeedPolicy::Explicit(value) => Some(value.to_u64()),
        };
        let instantiated = self
            .standard
            .instantiate(scenario_id.as_str(), seed_override)
            .map_err(|_| {
                agent_error(
                    AgentErrorCode::ConfigurationRejected,
                    "The replay's frozen Standard scenario could not be reconstructed.",
                )
            })?;
        let decoded = decode_replay(bytes).map_err(|_| replay_diverged_error())?;
        let expected = build_replay_header(
            instantiated.master_seed(),
            instantiated.encounter(),
            instantiated.spec_digest(),
            decoded.header().record_count(),
        )?;
        if decoded.header() != &expected {
            return Err(replay_diverged_error());
        }
        let report = verify_battle_replay(bytes, instantiated.into_battle())
            .map_err(|_| replay_diverged_error())?;
        Ok(AgentReplayVerification {
            command_count: AgentUInt::from_u64(u64::from(report.command_count())),
            final_state_hash: AgentHash::from_bytes(report.final_hash().bytes()),
            phase: replay_phase(report.phase())?,
        })
    }

    fn create_with_id_source(
        &self,
        scenario_id: ScenarioId,
        seed: AgentSeedPolicy,
        visibility_policy: VisibilityPolicy,
        allocate_id: impl FnOnce() -> Result<SessionId, AgentError>,
    ) -> Result<AgentSession, AgentError> {
        if visibility_policy != VisibilityPolicy::PlayerVisible {
            return Err(agent_error(
                AgentErrorCode::UnauthorizedPolicy,
                "Debug visibility requires a separately authorized creation path.",
            ));
        }
        let seed_override = match &seed {
            AgentSeedPolicy::ScenarioDefault => None,
            AgentSeedPolicy::Explicit(value) => Some(value.to_u64()),
        };
        let instantiated = self
            .standard
            .instantiate(scenario_id.as_str(), seed_override)
            .map_err(|_| {
                agent_error(
                    AgentErrorCode::ConfigurationRejected,
                    "The requested frozen Standard scenario is unknown or incompatible.",
                )
            })?;
        let session_id = allocate_id()?;
        let master_seed = instantiated.master_seed();
        let enemy_seed = controller_seed(scenario_id.as_str(), master_seed);
        let mut session = AgentSession {
            id: session_id,
            scenario: scenario_id,
            visibility: visibility_policy,
            encounter: instantiated.encounter(),
            spec_digest: instantiated.spec_digest(),
            master_seed,
            battle: instantiated.into_battle(),
            standard: self.standard.clone(),
            enemy: EnemyController::new(enemy_seed),
            offered: None,
            replay: AgentReplayRecorder::default(),
            idempotency: BTreeMap::new(),
        };
        session.settle_to_player(MAX_ACCEPTED_COMMANDS_PER_SETTLEMENT)?;
        Ok(session)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentControllerKind {
    ExternalPlayer,
    AuthoredEnemy,
    SystemAutomatic,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcceptedCommandRecord {
    pub sequence: AgentUInt,
    pub decision_id: AgentUInt,
    pub controller: AgentControllerKind,
    pub resulting_state_hash: AgentHash,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentSettlement {
    pub accepted_commands: AgentUInt,
    pub emitted_events: AgentUInt,
    pub resolver_operations: AgentUInt,
    pub controllers: Box<[AcceptedCommandRecord]>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentSettlementSummary {
    pub accepted_commands: AgentUInt,
    pub emitted_events: AgentUInt,
    pub resolver_operations: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActionResponse {
    pub schema_revision: AgentSchemaRevision,
    pub session_id: SessionId,
    pub committed: bool,
    pub idempotent_replay: bool,
    pub accepted_action_token: ActionToken,
    pub settlement: AgentSettlementSummary,
    pub observation: AgentObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentReplayExport {
    bytes: Box<[u8]>,
    diagnostics: Box<[AcceptedCommandRecord]>,
    sha256: AgentHash,
}

impl AgentReplayExport {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[AcceptedCommandRecord] {
        &self.diagnostics
    }

    #[must_use]
    pub fn sha256(&self) -> &AgentHash {
        &self.sha256
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentReplayVerification {
    pub command_count: AgentUInt,
    pub final_state_hash: AgentHash,
    pub phase: crate::observation::AgentBattlePhase,
}

struct CachedActionResponse {
    request: PlayActionRequest,
    response: AgentActionResponse,
    canonical_json: Box<[u8]>,
}

/// Incremental accepted-command recorder owned by exactly one session.
#[derive(Default)]
struct AgentReplayRecorder {
    trace: Vec<BattleTraceEntry>,
    controllers: Vec<AcceptedCommandRecord>,
    events: VecDeque<AgentEventSummary>,
}

impl AgentReplayRecorder {
    fn retain_event(&mut self, event: AgentEventSummary) {
        if self.events.len() == MAX_RETAINED_EVENT_SUMMARIES {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    fn latest_cursor(&self) -> EventCursor {
        let id = self
            .events
            .back()
            .map_or(0, |event| event.event_id.to_u64());
        EventCursor::parse(&format!("event_{id}"))
            .expect("event IDs always form a valid opaque cursor")
    }

    fn page_after(&self, cursor: &EventCursor) -> Result<AgentEventPage, AgentError> {
        let requested = cursor_id(cursor)?;
        let latest = self
            .events
            .back()
            .map_or(0, |event| event.event_id.to_u64());
        if requested > latest {
            return Err(agent_error(
                AgentErrorCode::InvalidRequest,
                "The event cursor is ahead of the retained battle history.",
            ));
        }
        if let Some(oldest) = self.events.front().map(|event| event.event_id.to_u64())
            && requested.saturating_add(1) < oldest
        {
            return Err(agent_error(
                AgentErrorCode::EventCursorExpired,
                "The event cursor precedes the retained summary window.",
            ));
        }
        let mut visible = self
            .events
            .iter()
            .filter(|event| event.event_id.to_u64() > requested);
        let events = visible
            .by_ref()
            .take(crate::observation::MAX_EVENTS_PER_PAGE)
            .cloned()
            .collect::<Vec<_>>();
        let truncated = visible.next().is_some();
        let next = events
            .last()
            .map_or(requested, |event| event.event_id.to_u64());
        Ok(AgentEventPage {
            events: events.into_boxed_slice(),
            next_cursor: EventCursor::parse(&format!("event_{next}"))
                .expect("event IDs always form a valid opaque cursor"),
            truncated,
        })
    }
}

/// One isolated authoritative battle and its complete incremental replay facts.
pub struct AgentSession {
    id: SessionId,
    scenario: ScenarioId,
    visibility: VisibilityPolicy,
    encounter: EncounterId,
    spec_digest: BattleSpecDigest,
    master_seed: u64,
    battle: Battle,
    standard: StandardV1Catalog,
    enemy: EnemyController,
    offered: Option<OfferedActionSet>,
    replay: AgentReplayRecorder,
    idempotency: BTreeMap<IdempotencyKey, CachedActionResponse>,
}

impl AgentSession {
    #[must_use]
    pub fn session_id(&self) -> &SessionId {
        &self.id
    }

    #[must_use]
    pub fn scenario_id(&self) -> &ScenarioId {
        &self.scenario
    }

    #[must_use]
    pub const fn visibility_policy(&self) -> VisibilityPolicy {
        self.visibility
    }

    #[must_use]
    pub const fn phase(&self) -> BattlePhase {
        self.battle.view().phase()
    }

    #[must_use]
    pub fn state_hash(&self) -> AgentHash {
        AgentHash::from_bytes(self.battle.state_hash().bytes())
    }

    #[must_use]
    pub fn master_seed(&self) -> AgentUInt {
        AgentUInt::from_u64(self.master_seed)
    }

    #[must_use]
    pub const fn encounter(&self) -> EncounterId {
        self.encounter
    }

    #[must_use]
    pub const fn spec_digest(&self) -> BattleSpecDigest {
        self.spec_digest
    }

    #[must_use]
    pub fn replay_command_count(&self) -> usize {
        self.replay.trace.len()
    }

    #[must_use]
    pub const fn rng_draw_count(&self) -> u64 {
        self.battle.view().rng_draw_count()
    }

    #[must_use]
    pub fn offered_actions(&self) -> &[OfferedAction] {
        self.offered.as_ref().map_or(&[], OfferedActionSet::actions)
    }

    #[must_use]
    pub fn controller_records(&self) -> &[AcceptedCommandRecord] {
        &self.replay.controllers
    }

    pub fn export_replay(&self) -> Result<AgentReplayExport, AgentError> {
        let header = self.replay_header()?;
        let bytes = encode_battle_trace(&header, &self.replay.trace).map_err(|_| {
            agent_error(
                AgentErrorCode::AdapterFailure,
                "The canonical battle replay could not be encoded.",
            )
        })?;
        let sha256 = AgentHash::from_bytes(Sha256::digest(&bytes).into());
        Ok(AgentReplayExport {
            bytes: bytes.into_boxed_slice(),
            diagnostics: self.replay.controllers.clone().into_boxed_slice(),
            sha256,
        })
    }

    pub fn verify_replay(&self, bytes: &[u8]) -> Result<AgentReplayVerification, AgentError> {
        AgentSessionFactory {
            standard: self.standard.clone(),
        }
        .verify_replay(
            &self.scenario,
            &AgentSeedPolicy::Explicit(AgentUInt::from_u64(self.master_seed)),
            bytes,
        )
    }

    fn replay_header(&self) -> Result<ReplayHeader, AgentError> {
        build_replay_header(
            self.master_seed,
            self.encounter,
            self.spec_digest,
            battle_record_count(self.replay.trace.len()).map_err(|_| replay_header_error())?,
        )
    }

    /// Applies one preconditioned action or returns the byte-identical cached response.
    pub fn apply_action(
        &mut self,
        request: PlayActionRequest,
    ) -> Result<AgentActionResponse, AgentError> {
        if request.session_id != self.id {
            return Err(agent_error(
                AgentErrorCode::SessionNotOwned,
                "The action request does not belong to this session.",
            ));
        }
        if let Some(cached) = self.idempotency.get(&request.idempotency_key) {
            if cached.request == request {
                debug_assert_eq!(
                    serde_json::to_vec(&cached.response).expect("cached response serializes"),
                    cached.canonical_json.as_ref()
                );
                return Ok(cached.response.clone());
            }
            return Err(agent_error(
                AgentErrorCode::IdempotencyConflict,
                "The idempotency key is already bound to a different request.",
            ));
        }
        if self.idempotency.len() == MAX_IDEMPOTENCY_ENTRIES {
            return Err(agent_error(
                AgentErrorCode::SessionQuotaExceeded,
                "The session idempotency cache reached its fixed entry limit.",
            ));
        }
        let current_decision = self
            .offered
            .as_ref()
            .map(OfferedActionSet::decision_id)
            .ok_or_else(|| {
                agent_error(
                    AgentErrorCode::StaleDecision,
                    "The session has no current external decision.",
                )
            })?;
        if request.decision_id != current_decision {
            return Err(agent_error(
                AgentErrorCode::StaleDecision,
                "The requested decision is no longer current.",
            ));
        }
        if request.expected_state_hash != self.state_hash() {
            return Err(agent_error(
                AgentErrorCode::StaleStateHash,
                "The expected state hash does not match the current battle state.",
            ));
        }

        let event_cursor = self.replay.latest_cursor();
        let settlement = self.play_token(&request.action_token)?;
        let response = AgentActionResponse {
            schema_revision: AgentSchemaRevision::V1,
            session_id: self.id.clone(),
            committed: true,
            idempotent_replay: false,
            accepted_action_token: request.action_token.clone(),
            settlement: AgentSettlementSummary {
                accepted_commands: settlement.accepted_commands,
                emitted_events: settlement.emitted_events,
                resolver_operations: settlement.resolver_operations,
            },
            observation: self.observation_after(&event_cursor)?,
        };
        let canonical_json = serde_json::to_vec(&response).map_err(|_| {
            committed_error(
                AgentErrorCode::AdapterFailure,
                "The committed action response could not be serialized.",
            )
        })?;
        if canonical_json.len() > MAX_CACHED_RESPONSE_BYTES {
            return Err(committed_error(
                AgentErrorCode::ObservationTooLarge,
                "The committed action response exceeds its fixed cache limit.",
            ));
        }
        self.idempotency.insert(
            request.idempotency_key.clone(),
            CachedActionResponse {
                request,
                response: response.clone(),
                canonical_json: canonical_json.into_boxed_slice(),
            },
        );
        Ok(response)
    }

    fn play_token(&mut self, token: &ActionToken) -> Result<AgentSettlement, AgentError> {
        let offered = self.offered.as_ref().ok_or_else(|| {
            agent_error(
                AgentErrorCode::CombatRejected,
                "The session is not awaiting an external player decision.",
            )
        })?;
        let decision = offered.decision_id();
        let selected = offered.select(&decision, token).map_err(binding_error)?;
        let command = selected.into_command();
        let command_start = self.replay.trace.len();
        let controller_start = self.replay.controllers.len();
        let mut events = self.apply_recorded(command, AgentControllerKind::ExternalPlayer)?;
        self.offered = None;
        events = events
            .checked_add(self.settle_to_player(MAX_ACCEPTED_COMMANDS_PER_SETTLEMENT - 1)?)
            .ok_or_else(settlement_budget_error)?;
        Ok(AgentSettlement {
            accepted_commands: AgentUInt::from_u64(
                u64::try_from(self.replay.trace.len() - command_start)
                    .expect("settlement command bound fits u64"),
            ),
            emitted_events: AgentUInt::from_u64(events),
            resolver_operations: AgentUInt::from_u64(0),
            controllers: self.replay.controllers[controller_start..]
                .to_vec()
                .into_boxed_slice(),
        })
    }

    pub fn observe(&self, after: &EventCursor) -> Result<AgentObservation, AgentError> {
        self.observation_after(after)
    }

    fn observation_after(&self, after: &EventCursor) -> Result<AgentObservation, AgentError> {
        let view = self.battle.view();
        let page = self.replay.page_after(after)?;
        let status = match view.phase() {
            BattlePhase::AwaitingCommand => AgentBattleStatus::AwaitingPlayer,
            BattlePhase::Won => AgentBattleStatus::Won,
            BattlePhase::Lost => AgentBattleStatus::Lost,
            BattlePhase::Faulted => AgentBattleStatus::Faulted,
            BattlePhase::Initializing | BattlePhase::Resolving => {
                return Err(agent_error(
                    AgentErrorCode::AdapterFailure,
                    "The session observation boundary is not stable.",
                ));
            }
        };
        Ok(AgentObservation {
            schema_revision: AgentSchemaRevision::V1,
            session_id: self.id.clone(),
            scenario_id: self.scenario.clone(),
            catalog_digest: AgentHash::from_bytes(view.identity().catalog_digest().bytes()),
            decision_id: self.offered.as_ref().map(OfferedActionSet::decision_id),
            state_hash: self.state_hash(),
            event_cursor: page.next_cursor,
            visibility_policy: self.visibility,
            status,
            battle: project_player_visible(view).map_err(|_| {
                agent_error(
                    AgentErrorCode::AdapterFailure,
                    "The stable battle could not be projected.",
                )
            })?,
            legal_actions: self.offered.as_ref().map_or_else(
                || Vec::new().into_boxed_slice(),
                |offered| offered.actions().to_vec().into_boxed_slice(),
            ),
            events: page.events,
            events_truncated: page.truncated,
        })
    }

    fn settle_to_player(&mut self, command_budget: usize) -> Result<u64, AgentError> {
        let start = self.replay.trace.len();
        let mut emitted_events = 0_u64;
        loop {
            if self.battle.view().phase().is_terminal() {
                self.offered = None;
                return Ok(emitted_events);
            }
            if self.replay.trace.len() - start == command_budget {
                return Err(settlement_budget_error());
            }
            let decision = self.battle.decision().cloned().ok_or_else(|| {
                agent_error(
                    AgentErrorCode::BattleFaulted,
                    "A nonterminal battle exposed no decision boundary.",
                )
            })?;
            match decision.owner() {
                DecisionOwner::Team(TeamSide::Player) => {
                    self.offered =
                        Some(OfferedActionSet::bind(&self.id, &decision).map_err(binding_error)?);
                    return Ok(emitted_events);
                }
                DecisionOwner::System => {
                    let command = system_command(&decision)?;
                    emitted_events = emitted_events
                        .checked_add(
                            self.apply_recorded(command, AgentControllerKind::SystemAutomatic)?,
                        )
                        .ok_or_else(settlement_budget_error)?;
                }
                DecisionOwner::Team(TeamSide::Enemy) => {
                    let (command, actor, graph_id) = self.authored_enemy_command(&decision)?;
                    emitted_events = emitted_events
                        .checked_add(
                            self.apply_recorded(command, AgentControllerKind::AuthoredEnemy)?,
                        )
                        .ok_or_else(settlement_budget_error)?;
                    let graph = self.standard.ai_graph(graph_id).ok_or_else(ai_error)?;
                    self.enemy
                        .settle(graph, actor, AiTransitionTiming::AfterAction, |condition| {
                            static_condition(condition).unwrap_or(false)
                        })
                        .map_err(|_| ai_error())?;
                }
            }
        }
    }

    fn authored_enemy_command(
        &mut self,
        decision: &starclock_combat::DecisionPoint,
    ) -> Result<
        (
            Command,
            starclock_combat::UnitId,
            starclock_combat::AiGraphId,
        ),
        AgentError,
    > {
        let actor = decision
            .legal_commands()
            .iter()
            .find_map(command_actor)
            .ok_or_else(ai_error)?;
        let (graph_id, initial_state, _) = self
            .battle
            .view()
            .units_by_id()
            .find(|unit| unit.id() == actor)
            .and_then(|unit| unit.enemy_ai_state())
            .ok_or_else(ai_error)?;
        let graph = self.standard.ai_graph(graph_id).ok_or_else(ai_error)?;
        if graph.states().iter().any(|state| {
            state
                .candidates()
                .iter()
                .any(|candidate| static_condition(candidate.condition()).is_none())
                || state
                    .transitions()
                    .iter()
                    .any(|transition| static_condition(transition.condition()).is_none())
        }) {
            return Err(ai_error());
        }
        let selected = self
            .enemy
            .decide(graph, initial_state, actor, decision, |condition| {
                static_condition(condition).unwrap_or(false)
            })
            .map_err(|_| ai_error())?;
        Ok((selected.command().clone(), actor, graph_id))
    }

    fn apply_recorded(
        &mut self,
        command: Command,
        controller: AgentControllerKind,
    ) -> Result<u64, AgentError> {
        let decision_id = command.decision().get();
        let resolution = self.battle.apply(command.clone()).map_err(|_| {
            agent_error(
                AgentErrorCode::CombatRejected,
                "An exact offered command was rejected by the battle boundary.",
            )
        })?;
        let state_hash = resolution.state_hash();
        let event_count = u64::try_from(resolution.events().len())
            .expect("resolution event collection length fits u64");
        for event in resolution.events() {
            self.replay.retain_event(project_event_summary(event));
        }
        self.replay
            .trace
            .push(BattleTraceEntry::new(command, state_hash));
        self.replay.controllers.push(AcceptedCommandRecord {
            sequence: AgentUInt::from_u64(
                u64::try_from(self.replay.trace.len()).expect("replay bound fits u64"),
            ),
            decision_id: AgentUInt::from_u64(decision_id),
            controller,
            resulting_state_hash: AgentHash::from_bytes(state_hash.bytes()),
        });
        Ok(event_count)
    }
}

fn command_actor(command: &Command) -> Option<starclock_combat::UnitId> {
    match command {
        Command::UseAbility { actor, .. } | Command::UseInterrupt { actor, .. } => Some(*actor),
        Command::StartBattle { .. }
        | Command::PassInterruptWindow { .. }
        | Command::Concede { .. } => None,
    }
}

fn cursor_id(cursor: &EventCursor) -> Result<u64, AgentError> {
    let value = cursor.as_str().strip_prefix("event_").ok_or_else(|| {
        agent_error(
            AgentErrorCode::InvalidRequest,
            "The event cursor has an invalid opaque representation.",
        )
    })?;
    AgentUInt::parse(value).map_or_else(
        |_| {
            Err(agent_error(
                AgentErrorCode::InvalidRequest,
                "The event cursor has an invalid opaque representation.",
            ))
        },
        |value| Ok(value.to_u64()),
    )
}

fn agent_count(value: usize) -> Result<AgentUInt, AgentError> {
    u64::try_from(value)
        .map(AgentUInt::from_u64)
        .map_err(|_| replay_header_error())
}

fn build_replay_header(
    master_seed: u64,
    encounter: EncounterId,
    spec_digest: BattleSpecDigest,
    record_count: u32,
) -> Result<ReplayHeader, AgentError> {
    let identity = ReplayIdentity::new(
        "4.4",
        RULES_REVISION,
        CATALOG_REVISION,
        ConfigBundleDigest::new(CONFIG_DIGEST),
        starclock_combat::NUMERIC_POLICY_REVISION,
        starclock_combat::rng::RNG_ALGORITHM_REVISION,
        starclock_combat::STATE_HASH_REVISION,
    )
    .map_err(|_| replay_header_error())?;
    let controller = ControllerIdentity::new(
        AGENT_REPLAY_CONTROLLER_REVISION,
        ControllerDigest::new(Sha256::digest(AGENT_REPLAY_CONTROLLER_DESCRIPTOR).into()),
    )
    .map_err(|_| replay_header_error())?;
    ReplayHeader::new(
        identity,
        controller,
        master_seed,
        ReplayEntry::Battle {
            definition_id: encounter.get(),
            spec_digest: EntrySpecDigest::new(spec_digest.bytes()),
        },
        record_count,
    )
    .map_err(|_| replay_header_error())
}

fn replay_phase(phase: BattlePhase) -> Result<crate::observation::AgentBattlePhase, AgentError> {
    match phase {
        BattlePhase::AwaitingCommand => Ok(crate::observation::AgentBattlePhase::AwaitingCommand),
        BattlePhase::Won => Ok(crate::observation::AgentBattlePhase::Won),
        BattlePhase::Lost => Ok(crate::observation::AgentBattlePhase::Lost),
        BattlePhase::Faulted => Ok(crate::observation::AgentBattlePhase::Faulted),
        BattlePhase::Initializing | BattlePhase::Resolving => Err(agent_error(
            AgentErrorCode::ReplayDiverged,
            "The verified replay ended outside a stable external boundary.",
        )),
    }
}

fn replay_header_error() -> AgentError {
    agent_error(
        AgentErrorCode::AdapterFailure,
        "The frozen replay compatibility header could not be constructed.",
    )
}

fn replay_diverged_error() -> AgentError {
    agent_error(
        AgentErrorCode::ReplayDiverged,
        "The canonical replay did not verify against a fresh battle.",
    )
}

fn system_command(decision: &starclock_combat::DecisionPoint) -> Result<Command, AgentError> {
    let selected = match decision.kind() {
        DecisionKind::BattleStart => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::StartBattle { .. })),
        DecisionKind::InterruptWindow => decision
            .legal_commands()
            .iter()
            .find(|command| matches!(command, Command::PassInterruptWindow { .. })),
        DecisionKind::NormalAction | DecisionKind::BattleChoice => None,
    };
    selected.cloned().ok_or_else(|| {
        agent_error(
            AgentErrorCode::CombatRejected,
            "The system decision has no supported exact automatic command.",
        )
    })
}

fn static_condition(condition: &ConditionExpr) -> Option<bool> {
    match condition {
        ConditionExpr::Literal(value) => Some(*value),
        ConditionExpr::Not(value) => static_condition(value).map(|value| !value),
        ConditionExpr::All(values) => values
            .iter()
            .map(static_condition)
            .try_fold(true, |left, right| right.map(|right| left && right)),
        ConditionExpr::Any(values) => values
            .iter()
            .map(static_condition)
            .try_fold(false, |left, right| right.map(|right| left || right)),
        ConditionExpr::Compare { .. }
        | ConditionExpr::EventKind(_)
        | ConditionExpr::SourceTag(_)
        | ConditionExpr::SelectorCardinality { .. }
        | ConditionExpr::LifePresence { .. }
        | ConditionExpr::EffectExists { .. }
        | ConditionExpr::HasWeakness { .. }
        | ConditionExpr::IsBroken(_) => None,
    }
}

fn controller_seed(scenario: &str, master_seed: u64) -> RngSeed {
    let mut digest = Sha256::new();
    digest.update(b"starclock-agent-enemy-controller-v1\0");
    digest.update((scenario.len() as u64).to_be_bytes());
    digest.update(scenario.as_bytes());
    digest.update(master_seed.to_be_bytes());
    RngSeed::new(digest.finalize().into())
}

fn binding_error(_error: ActionBindingError) -> AgentError {
    agent_error(
        AgentErrorCode::InvalidActionToken,
        "The offered action binding is invalid for the current decision.",
    )
}

fn ai_error() -> AgentError {
    agent_error(
        AgentErrorCode::CombatRejected,
        "The authored enemy controller could not select a supported exact command.",
    )
}

fn settlement_budget_error() -> AgentError {
    agent_error(
        AgentErrorCode::SettlementBudgetExceeded,
        "The synchronous decision settlement exceeded its accepted-command budget.",
    )
}

fn agent_error(code: AgentErrorCode, message: &'static str) -> AgentError {
    AgentError::new(code, message, false, false).expect("static session error is bounded")
}

fn committed_error(code: AgentErrorCode, message: &'static str) -> AgentError {
    AgentError::new(code, message, false, true).expect("static session error is bounded")
}

#[cfg(test)]
mod tests;
