//! Authoritative incremental Standard Universe Activity sessions.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityDecisionKind, ActivityExternalOutcomeId, ActivityTerminalOutcome,
};
use starclock_mode_universe::{
    nested_battle_executor::UniverseNestedBattleExecutor,
    runtime::StandardUniverseActivity,
    universe_replay::{
        MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS, StandardUniverseReplayAction,
        StandardUniverseTraceEntry,
    },
    universe_replay_v2::{
        encode_standard_universe_trace_parts_v2, standard_universe_header_v2,
        verify_standard_universe_replay_v2,
    },
};
use starclock_replay::{
    activity::{ControllerDecisionKind, ControllerDiagnostic, ControllerOptionScore},
    format_v2::ReplayHeaderV2,
};

use crate::{
    activity_action::{
        ActivityActionBindingError, BoundActivityAction, OfferedActivityAction,
        OfferedActivityActionSet,
    },
    activity_observation::{
        ActivityObservationContext, AgentActivityObservation, project_activity_observation,
    },
    activity_runtime::{ActivityRuntimeError, ActivityRuntimeFactory, BATTLE_EXECUTOR_REVISION},
    error::{AgentError, AgentErrorCode},
    schema::{ActionToken, AgentHash, AgentSchemaRevision, AgentUInt, IdempotencyKey, SessionId},
    session::{MAX_CACHED_RESPONSE_BYTES, MAX_IDEMPOTENCY_ENTRIES},
};

pub mod registry;

pub const RESPONSIBILITY: &str = "authoritative Activity sessions and replay export";
pub const ACTIVITY_AGENT_CONTROLLER_REVISION: &str = "agent-activity-session-v1";
pub const MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT: usize = 16;
pub const DEFAULT_TECHNIQUE_POINTS: u16 = 5;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentUniverseWorldSummary {
    pub world: AgentUInt,
    pub stable_key: Box<str>,
    pub name_en: Box<str>,
    pub name_zh_cn: Box<str>,
    pub difficulty_count: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentUniverseManifest {
    pub schema_revision: AgentSchemaRevision,
    pub game_version: Box<str>,
    pub snapshot_date: Box<str>,
    pub catalog_revision: Box<str>,
    pub profile_revision: Box<str>,
    pub activity_interface_revision: Box<str>,
    pub battle_executor_revision: Box<str>,
    pub worlds: Box<[AgentUniverseWorldSummary]>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateActivitySessionRequest {
    pub session_id: SessionId,
    pub world: AgentUInt,
    pub difficulty_index: AgentUInt,
    pub seed: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlayActivityActionRequest {
    pub schema_revision: AgentSchemaRevision,
    pub session_id: SessionId,
    pub boundary_id: AgentUInt,
    pub expected_state_hash: AgentHash,
    pub action_token: ActionToken,
    pub idempotency_key: IdempotencyKey,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivitySettlementSummary {
    pub accepted_activity_actions: AgentUInt,
    pub nested_battles: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityActionResponse {
    pub schema_revision: AgentSchemaRevision,
    pub session_id: SessionId,
    pub committed: bool,
    pub idempotent_replay: bool,
    pub accepted_action_token: ActionToken,
    pub settlement: AgentActivitySettlementSummary,
    pub observation: AgentActivityObservation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AgentActivityReplayExport {
    bytes: Box<[u8]>,
    sha256: AgentHash,
    action_count: AgentUInt,
    complete: bool,
}

impl AgentActivityReplayExport {
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    #[must_use]
    pub const fn sha256(&self) -> &AgentHash {
        &self.sha256
    }
    #[must_use]
    pub const fn action_count(&self) -> &AgentUInt {
        &self.action_count
    }
    #[must_use]
    pub const fn complete(&self) -> bool {
        self.complete
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentActivityReplayVerification {
    pub action_count: AgentUInt,
    pub nested_battles: AgentUInt,
    pub final_state_hash: AgentHash,
    pub terminal: AgentActivityTerminalOutcome,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityTerminalOutcome {
    Completed,
    Failed,
    Abandoned,
    Faulted,
}

#[derive(Clone)]
pub struct ActivityAgentSessionFactory {
    runtime: ActivityRuntimeFactory,
}

impl ActivityAgentSessionFactory {
    pub fn load_production() -> Result<Self, AgentError> {
        Ok(Self {
            runtime: ActivityRuntimeFactory::load().map_err(runtime_error)?,
        })
    }

    pub fn create(
        &self,
        request: CreateActivitySessionRequest,
    ) -> Result<ActivityAgentSession, AgentError> {
        let world = u32::try_from(request.world.to_u64()).map_err(|_| invalid_request())?;
        let difficulty_index =
            usize::try_from(request.difficulty_index.to_u64()).map_err(|_| invalid_request())?;
        let seed = request.seed.to_u64();
        let runtime = self
            .runtime
            .start(world, difficulty_index, seed, controller_digest())
            .map_err(runtime_error)?;
        let (profile, activity, combat_catalog, components, compatibility) = runtime.into_parts();
        let replay_header = standard_universe_header_v2(
            compatibility.clone(),
            components.clone(),
            seed,
            &activity,
            &profile,
        )
        .map_err(|_| adapter_error(false))?;
        let mut session = ActivityAgentSession {
            id: request.session_id,
            profile,
            world,
            difficulty_index,
            seed,
            activity,
            replay_header,
            battle_executor: UniverseNestedBattleExecutor::new(combat_catalog),
            trace: Vec::new(),
            offered: None,
            idempotency: BTreeMap::new(),
            closed: false,
        };
        session.refresh_offer()?;
        Ok(session)
    }

    pub fn manifest(&self) -> AgentUniverseManifest {
        let catalog = self.runtime.catalog();
        let identity = catalog.identity();
        AgentUniverseManifest {
            schema_revision: AgentSchemaRevision::V1,
            game_version: identity.game_version().into(),
            snapshot_date: identity.snapshot_date().into(),
            catalog_revision: identity.catalog_revision().into(),
            profile_revision: identity.profile_revision().into(),
            activity_interface_revision:
                crate::activity_observation::ACTIVITY_AGENT_INTERFACE_REVISION.into(),
            battle_executor_revision: BATTLE_EXECUTOR_REVISION.into(),
            worlds: catalog
                .worlds()
                .iter()
                .map(|world| AgentUniverseWorldSummary {
                    world: AgentUInt::from_u64(u64::from(world.number())),
                    stable_key: world.stable_key().into(),
                    name_en: world.text().name_en().into(),
                    name_zh_cn: world.text().name_zh_cn().into(),
                    difficulty_count: AgentUInt::from_u64(world.difficulties().len() as u64),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn verify_replay(
        &self,
        world: &AgentUInt,
        difficulty_index: &AgentUInt,
        seed: &AgentUInt,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let world = u32::try_from(world.to_u64()).map_err(|_| invalid_request())?;
        let difficulty_index =
            usize::try_from(difficulty_index.to_u64()).map_err(|_| invalid_request())?;
        let runtime = self
            .runtime
            .start(world, difficulty_index, seed.to_u64(), controller_digest())
            .map_err(runtime_error)?;
        let (profile, activity, combat_catalog, components, compatibility) = runtime.into_parts();
        let report = verify_standard_universe_replay_v2(
            bytes,
            activity,
            combat_catalog,
            &components,
            &compatibility,
            &profile,
        )
        .map_err(|error| replay_error_with_reason(&format!("{error:?}")))?;
        Ok(AgentActivityReplayVerification {
            action_count: AgentUInt::from_u64(u64::from(report.action_count())),
            nested_battles: AgentUInt::from_u64(u64::from(report.battle_count())),
            final_state_hash: AgentHash::from_bytes(report.final_state_hash().bytes()),
            terminal: terminal(report.terminal()),
        })
    }
}

struct CachedActivityResponse {
    request: PlayActivityActionRequest,
    response: AgentActivityActionResponse,
    canonical_json: Box<[u8]>,
}

pub struct ActivityAgentSession {
    id: SessionId,
    profile: Box<str>,
    world: u32,
    difficulty_index: usize,
    seed: u64,
    activity: StandardUniverseActivity,
    replay_header: ReplayHeaderV2,
    battle_executor: UniverseNestedBattleExecutor,
    trace: Vec<StandardUniverseTraceEntry>,
    offered: Option<OfferedActivityActionSet>,
    idempotency: BTreeMap<IdempotencyKey, CachedActivityResponse>,
    closed: bool,
}

impl ActivityAgentSession {
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.id
    }
    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile
    }
    #[must_use]
    pub const fn world(&self) -> u32 {
        self.world
    }
    #[must_use]
    pub const fn difficulty_index(&self) -> usize {
        self.difficulty_index
    }
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }
    #[must_use]
    pub fn state_hash(&self) -> AgentHash {
        AgentHash::from_bytes(self.activity.view().state_hash().bytes())
    }
    #[must_use]
    pub fn terminal(&self) -> Option<ActivityTerminalOutcome> {
        self.activity.view().terminal()
    }
    #[must_use]
    pub fn offered_actions(&self) -> &[OfferedActivityAction] {
        self.offered
            .as_ref()
            .map_or(&[], OfferedActivityActionSet::actions)
    }
    #[must_use]
    pub fn replay_action_count(&self) -> usize {
        self.trace.len()
    }

    pub fn observe(&self) -> Result<AgentActivityObservation, AgentError> {
        let view = self.activity.view();
        let offered = self
            .offered
            .as_ref()
            .map(|value| (value.boundary(), value.actions()));
        project_activity_observation(
            &view,
            ActivityObservationContext {
                session: &self.id,
                profile: &self.profile,
                world: self.world,
                difficulty_index: self.difficulty_index,
                offered,
                closed: self.closed,
            },
        )
        .map_err(|_| adapter_error(false))
    }

    pub fn apply_action(
        &mut self,
        request: PlayActivityActionRequest,
    ) -> Result<AgentActivityActionResponse, AgentError> {
        if request.session_id != self.id {
            return Err(agent_error(
                AgentErrorCode::SessionNotOwned,
                "The Activity action does not belong to this session.",
                false,
            ));
        }
        if self.closed || self.activity.view().terminal().is_some() {
            return Err(agent_error(
                AgentErrorCode::SessionClosed,
                "The Activity session has already settled or closed.",
                false,
            ));
        }
        if let Some(cached) = self.idempotency.get(&request.idempotency_key) {
            if cached.request == request {
                debug_assert_eq!(
                    serde_json::to_vec(&cached.response).expect("cached response serializes"),
                    cached.canonical_json.as_ref(),
                );
                return Ok(cached.response.clone());
            }
            return Err(agent_error(
                AgentErrorCode::IdempotencyConflict,
                "The Activity idempotency key is bound to another request.",
                false,
            ));
        }
        if self.idempotency.len() == MAX_IDEMPOTENCY_ENTRIES {
            return Err(agent_error(
                AgentErrorCode::SessionQuotaExceeded,
                "The Activity idempotency cache reached its fixed limit.",
                false,
            ));
        }
        let offered = self.offered.as_ref().ok_or_else(stale_boundary)?;
        if request.boundary_id.to_u64() != offered.boundary() {
            return Err(stale_boundary());
        }
        if request.expected_state_hash != AgentHash::from_bytes(offered.state_hash().bytes()) {
            return Err(agent_error(
                AgentErrorCode::StaleStateHash,
                "The expected hash does not match the current Activity state.",
                false,
            ));
        }
        let selected = offered
            .select(&request.boundary_id, &request.action_token)
            .map_err(action_binding_error)?;
        self.offered = None;
        let start = self.trace.len();
        self.apply_selected(selected.into_action())?;
        let nested_battles = self.settle_automatic_battles()?;
        if self.trace.len() - start > MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT {
            return Err(agent_error(
                AgentErrorCode::SettlementBudgetExceeded,
                "The Activity settlement exceeded its accepted-action budget.",
                true,
            ));
        }
        self.refresh_offer()?;
        let response = AgentActivityActionResponse {
            schema_revision: AgentSchemaRevision::V1,
            session_id: self.id.clone(),
            committed: true,
            idempotent_replay: false,
            accepted_action_token: request.action_token.clone(),
            settlement: AgentActivitySettlementSummary {
                accepted_activity_actions: AgentUInt::from_u64(
                    u64::try_from(self.trace.len() - start).expect("settlement bound fits u64"),
                ),
                nested_battles: AgentUInt::from_u64(nested_battles),
            },
            observation: self.observe()?,
        };
        let canonical_json = serde_json::to_vec(&response).map_err(|_| adapter_error(true))?;
        if canonical_json.len() > MAX_CACHED_RESPONSE_BYTES {
            return Err(agent_error(
                AgentErrorCode::ObservationTooLarge,
                "The committed Activity response exceeds its cache limit.",
                true,
            ));
        }
        self.idempotency.insert(
            request.idempotency_key.clone(),
            CachedActivityResponse {
                request,
                response: response.clone(),
                canonical_json: canonical_json.into_boxed_slice(),
            },
        );
        Ok(response)
    }

    pub fn export_replay(&self) -> Result<AgentActivityReplayExport, AgentError> {
        let bytes = encode_standard_universe_trace_parts_v2(
            &self.replay_header,
            &self.trace,
            self.battle_executor.reports(),
        )
        .map_err(|_| adapter_error(false))?;
        Ok(AgentActivityReplayExport {
            sha256: AgentHash::from_bytes(Sha256::digest(&bytes).into()),
            bytes: bytes.into_boxed_slice(),
            action_count: AgentUInt::from_u64(
                u64::try_from(self.trace.len()).expect("replay action bound fits u64"),
            ),
            complete: self.activity.view().terminal().is_some(),
        })
    }

    pub fn verify_replay(
        &self,
        factory: &ActivityAgentSessionFactory,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        factory.verify_replay(
            &AgentUInt::from_u64(u64::from(self.world)),
            &AgentUInt::from_u64(self.difficulty_index as u64),
            &AgentUInt::from_u64(self.seed),
            bytes,
        )
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.offered = None;
    }

    fn apply_selected(&mut self, action: BoundActivityAction) -> Result<(), AgentError> {
        let expected = self.activity.view().state_hash();
        let (replay_action, diagnostic) = match action {
            BoundActivityAction::Decision {
                decision,
                kind,
                option,
            } => {
                let diagnostic = self.external_decision_diagnostic(decision, kind, option)?;
                match kind {
                    ActivityDecisionKind::Encounter => self
                        .activity
                        .engage_encounter(expected, decision, option, DEFAULT_TECHNIQUE_POINTS)
                        .map(|_| ())
                        .map_err(|_| activity_rejected())?,
                    ActivityDecisionKind::ExternalOutcome => self
                        .activity
                        .submit_external_outcome(
                            expected,
                            decision,
                            ActivityExternalOutcomeId::new(option.get())
                                .expect("offered option IDs are non-zero"),
                        )
                        .map(|_| ())
                        .map_err(|_| activity_rejected())?,
                    _ => self
                        .activity
                        .choose_option(expected, decision, option)
                        .map(|_| ())
                        .map_err(|_| activity_rejected())?,
                }
                (
                    StandardUniverseReplayAction::Decision {
                        decision,
                        kind,
                        option,
                        technique_points: DEFAULT_TECHNIQUE_POINTS,
                    },
                    Some(diagnostic),
                )
            }
            BoundActivityAction::Preparation { option } => {
                self.activity
                    .choose_preparation_option(expected, option)
                    .map_err(|_| activity_rejected())?;
                (StandardUniverseReplayAction::Preparation { option }, None)
            }
        };
        self.push_trace(replay_action, diagnostic)
    }

    fn settle_automatic_battles(&mut self) -> Result<u64, AgentError> {
        let mut battles = 0_u64;
        while self.activity.view().pending_battle().is_some() {
            if self.trace.len() >= MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS as usize {
                return Err(settlement_budget_error());
            }
            let settled = self
                .battle_executor
                .execute_pending_activity_battle(&mut self.activity)
                .map_err(|_| nested_battle_error())?;
            self.push_trace(
                StandardUniverseReplayAction::Battle {
                    result: Box::new(settled.result().clone()),
                },
                None,
            )?;
            battles = battles.checked_add(1).ok_or_else(settlement_budget_error)?;
            if battles as usize == MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT {
                return Err(settlement_budget_error());
            }
        }
        Ok(battles)
    }

    fn push_trace(
        &mut self,
        action: StandardUniverseReplayAction,
        diagnostic: Option<ControllerDiagnostic>,
    ) -> Result<(), AgentError> {
        if self.trace.len() >= MAX_STANDARD_UNIVERSE_REPLAY_ACTIONS as usize {
            return Err(settlement_budget_error());
        }
        self.trace.push(StandardUniverseTraceEntry::new(
            action,
            self.activity.view().state_hash(),
            diagnostic,
        ));
        Ok(())
    }

    fn external_decision_diagnostic(
        &self,
        decision: starclock_activity::ActivityDecisionId,
        kind: ActivityDecisionKind,
        option: starclock_activity::ActivityOptionId,
    ) -> Result<ControllerDiagnostic, AgentError> {
        let view = self.activity.view();
        let offered = view
            .decision()
            .filter(|value| value.id() == decision && value.kind() == kind)
            .ok_or_else(activity_rejected)?;
        let selected = offered
            .options()
            .iter()
            .position(|candidate| candidate.id() == option)
            .ok_or_else(activity_rejected)?;
        ControllerDiagnostic::new(
            ControllerDecisionKind::Activity,
            self.trace.len() as u64,
            u32::try_from(selected).map_err(|_| settlement_budget_error())?,
            None,
            offered
                .options()
                .iter()
                .enumerate()
                .map(|(ordinal, _)| {
                    Ok(ControllerOptionScore::new(
                        u32::try_from(ordinal).map_err(|_| settlement_budget_error())?,
                        i64::from(ordinal == selected),
                    ))
                })
                .collect::<Result<Vec<_>, AgentError>>()?,
        )
        .map_err(|_| adapter_error(false))
    }

    fn refresh_offer(&mut self) -> Result<(), AgentError> {
        let view = self.activity.view();
        if self.closed || view.terminal().is_some() {
            self.offered = None;
            return Ok(());
        }
        if view.pending_battle().is_some() {
            return Err(adapter_error(false));
        }
        self.offered =
            Some(OfferedActivityActionSet::bind(&self.id, &view).map_err(action_binding_error)?);
        Ok(())
    }
}

fn controller_digest() -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"agent-activity-session-v1\0external-player\0");
    hash.update(BATTLE_EXECUTOR_REVISION.as_bytes());
    hash.finalize().into()
}

fn action_binding_error(error: ActivityActionBindingError) -> AgentError {
    match error {
        ActivityActionBindingError::StaleBoundary => stale_boundary(),
        ActivityActionBindingError::InvalidActionToken => agent_error(
            AgentErrorCode::InvalidActionToken,
            "The Activity token is not in the current exact offer.",
            false,
        ),
        _ => adapter_error(false),
    }
}

fn runtime_error(error: ActivityRuntimeError) -> AgentError {
    match error {
        ActivityRuntimeError::Runtime(
            starclock_mode_universe::production_runtime::StandardUniverseRuntimeFactoryError::UnknownEntry
            | starclock_mode_universe::production_runtime::StandardUniverseRuntimeFactoryError::InvalidSeed,
        ) => invalid_request(),
        ActivityRuntimeError::Runtime(
            starclock_mode_universe::production_runtime::StandardUniverseRuntimeFactoryError::Configuration
            | starclock_mode_universe::production_runtime::StandardUniverseRuntimeFactoryError::Start,
        ) => agent_error(
            AgentErrorCode::ConfigurationRejected,
            "The Standard Universe Activity could not be constructed.",
            false,
        ),
    }
}

fn nested_battle_error() -> AgentError {
    agent_error(
        AgentErrorCode::CombatRejected,
        "The automatic nested battle failed and the Activity boundary was restored.",
        true,
    )
}

fn terminal(value: ActivityTerminalOutcome) -> AgentActivityTerminalOutcome {
    match value {
        ActivityTerminalOutcome::Completed => AgentActivityTerminalOutcome::Completed,
        ActivityTerminalOutcome::Failed => AgentActivityTerminalOutcome::Failed,
        ActivityTerminalOutcome::Abandoned => AgentActivityTerminalOutcome::Abandoned,
        ActivityTerminalOutcome::Faulted => AgentActivityTerminalOutcome::Faulted,
    }
}

fn invalid_request() -> AgentError {
    agent_error(
        AgentErrorCode::InvalidRequest,
        "The Standard Universe world, difficulty or seed is invalid.",
        false,
    )
}

fn stale_boundary() -> AgentError {
    agent_error(
        AgentErrorCode::StaleDecision,
        "The requested Activity boundary is no longer current.",
        false,
    )
}

fn activity_rejected() -> AgentError {
    agent_error(
        AgentErrorCode::CombatRejected,
        "The exact offered Activity action was rejected by the runtime.",
        false,
    )
}

fn replay_error() -> AgentError {
    agent_error(
        AgentErrorCode::ReplayDiverged,
        "The Standard Universe Activity replay diverged.",
        false,
    )
}

fn replay_error_with_reason(reason: &str) -> AgentError {
    let mut error = replay_error();
    let bounded = if reason.len() <= 512 {
        reason
    } else {
        "replay verifier returned an oversized diagnostic"
    };
    error
        .insert_detail("reason", bounded)
        .expect("bounded replay diagnostic is valid");
    error
}

fn settlement_budget_error() -> AgentError {
    agent_error(
        AgentErrorCode::SettlementBudgetExceeded,
        "The Activity settlement exceeded its accepted-action budget.",
        true,
    )
}

fn adapter_error(committed: bool) -> AgentError {
    agent_error(
        AgentErrorCode::AdapterFailure,
        "The stable Activity boundary could not be projected or encoded.",
        committed,
    )
}

fn agent_error(code: AgentErrorCode, message: &'static str, committed: bool) -> AgentError {
    AgentError::new(code, message, false, committed)
        .expect("static Activity session error is bounded")
}
