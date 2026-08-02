//! Swarm Disaster sessions over the shared Activity agent vocabulary.

use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityDecisionKind, ActivityInstanceId, ActivityPlayerView, ActivityStateHash,
    GraphActivityCommand, GraphActivityCommandKind,
};
use starclock_mode_universe::swarm_disaster_entry::{
    SwarmDisasterControllerIdentity, SwarmDisasterRuntimeFactory,
    baseline_fixture::{
        SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION,
        SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY, SWARM_DISASTER_BASELINE_FIXTURE_REVISION,
        SWARM_DISASTER_BASELINE_PROFILE, SwarmDisasterBaselineFixture,
    },
    incremental_run::{SwarmDisasterIncrementalOffer, SwarmDisasterIncrementalRun},
    replay::{SwarmReplayError, encode_incremental_swarm_replay, verify_complete_swarm_replay},
};
use starclock_replay::component::ConfigurationComponentSet;

use crate::{
    activity_action::{
        ActivityActionBindingError, AgentActivityActionKind, MAX_OFFERED_ACTIVITY_ACTIONS,
        OfferedActivityAction, activity_action_token,
    },
    activity_observation::{
        ActivityObservationContext, AgentActivityDecisionKind, AgentActivityObservation,
        project_activity_observation,
    },
    activity_session::{
        ACTIVITY_AGENT_CONTROLLER_REVISION, AgentActivityActionResponse, AgentActivityReplayExport,
        AgentActivityReplayVerification, AgentActivitySettlementSummary, PlayActivityActionRequest,
    },
    error::{AgentError, AgentErrorCode},
    schema::{ActionToken, AgentHash, AgentSInt, AgentUInt, SessionId},
    session::{MAX_CACHED_RESPONSE_BYTES, MAX_IDEMPOTENCY_ENTRIES},
};

const SWARM_BUNDLE: &[u8] = include_bytes!("../../../config/swarm-disaster-generated/config.sora");
const AREA: u32 = 201;
const DIFFICULTY_INDEX: usize = 0;
const MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT: usize = 16;

/// Request for one fixed-fixture Swarm Activity session.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateSwarmDisasterActivitySessionRequest {
    pub session_id: SessionId,
    pub seed: AgentUInt,
}

/// Immutable mode and configuration identity advertised to agent clients.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentSwarmDisasterManifest {
    pub profile_id: Box<str>,
    pub fixture_revision: Box<str>,
    pub fixture_accuracy: Box<str>,
    pub activity_interface_revision: Box<str>,
    pub controller_revision: Box<str>,
    pub battle_executor_revision: Box<str>,
    pub area: AgentUInt,
    pub path: Box<str>,
    pub audience_die: Box<str>,
    pub component_root: AgentHash,
}

/// Immutable production fixture shared by Swarm agent sessions.
#[derive(Clone)]
pub struct SwarmDisasterActivityAgentSessionFactory {
    fixture: Arc<SwarmDisasterBaselineFixture>,
    components: ConfigurationComponentSet,
}

impl SwarmDisasterActivityAgentSessionFactory {
    /// Loads and validates the production Sora bundle and fixed fixture.
    pub fn load_production() -> Result<Self, AgentError> {
        let runtime = SwarmDisasterRuntimeFactory::load_candidate(SWARM_BUNDLE)
            .map_err(|_| configuration_error())?;
        let fixture = runtime
            .compile_synthetic_baseline_fixture()
            .map_err(|_| configuration_error())?;
        let components = fixture
            .components_for_controller(SwarmDisasterControllerIdentity {
                id: "agent-activity-controller",
                revision: ACTIVITY_AGENT_CONTROLLER_REVISION,
                digest: controller_digest(),
            })
            .map_err(|_| configuration_error())?;
        Ok(Self {
            fixture: Arc::new(fixture),
            components,
        })
    }

    /// Creates a fresh authoritative session and settles system-owned setup.
    pub fn create(
        &self,
        request: CreateSwarmDisasterActivitySessionRequest,
    ) -> Result<SwarmDisasterActivityAgentSession, AgentError> {
        let activity_instance = activity_instance()?;
        let mut run = SwarmDisasterIncrementalRun::start(
            self.fixture.instance(),
            request.seed.to_u64(),
            self.fixture.activity_identity(),
            activity_instance,
        );
        run.settle_automatic(self.fixture.instance(), self.fixture.roster())
            .map_err(|error| run_error(error, true))?;
        let mut session = SwarmDisasterActivityAgentSession {
            id: request.session_id,
            seed: request.seed.to_u64(),
            fixture: Arc::clone(&self.fixture),
            components: self.components.clone(),
            run,
            offered: None,
            idempotency: BTreeMap::new(),
            closed: false,
        };
        session.refresh_offer()?;
        Ok(session)
    }

    /// Returns the exact immutable adapter manifest.
    #[must_use]
    pub fn manifest(&self) -> AgentSwarmDisasterManifest {
        AgentSwarmDisasterManifest {
            profile_id: SWARM_DISASTER_BASELINE_PROFILE.into(),
            fixture_revision: SWARM_DISASTER_BASELINE_FIXTURE_REVISION.into(),
            fixture_accuracy: SWARM_DISASTER_BASELINE_FIXTURE_ACCURACY.into(),
            activity_interface_revision:
                crate::activity_observation::ACTIVITY_AGENT_INTERFACE_REVISION.into(),
            controller_revision: ACTIVITY_AGENT_CONTROLLER_REVISION.into(),
            battle_executor_revision: SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION.into(),
            area: AgentUInt::from_u64(u64::from(AREA)),
            path: self.fixture.path().into(),
            audience_die: self.fixture.audience_die().into(),
            component_root: AgentHash::from_bytes(self.components.root().bytes()),
        }
    }

    /// Freshly verifies a canonical replay without touching a live session.
    pub fn verify_replay(
        &self,
        seed: &AgentUInt,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let report = verify_complete_swarm_replay(
            bytes,
            self.fixture.instance(),
            seed.to_u64(),
            self.fixture.activity_identity(),
            activity_instance()?,
            self.fixture.roster(),
            &self.components,
        )
        .map_err(|error| replay_error(&format!("{error:?}")))?;
        Ok(AgentActivityReplayVerification {
            action_count: AgentUInt::from_u64(u64::from(report.action_count())),
            nested_battles: AgentUInt::from_u64(u64::from(report.battle_count())),
            final_state_hash: AgentHash::from_bytes(report.final_state_hash().bytes()),
            terminal: super::activity_session::terminal(report.terminal()),
        })
    }
}

struct CachedSwarmResponse {
    request: PlayActivityActionRequest,
    response: AgentActivityActionResponse,
    canonical_json: Box<[u8]>,
}

/// One authoritative Swarm session over the shared `agent-activity-v1` DTOs.
pub struct SwarmDisasterActivityAgentSession {
    id: SessionId,
    seed: u64,
    fixture: Arc<SwarmDisasterBaselineFixture>,
    components: ConfigurationComponentSet,
    run: SwarmDisasterIncrementalRun,
    offered: Option<SwarmOfferedActionSet>,
    idempotency: BTreeMap<crate::schema::IdempotencyKey, CachedSwarmResponse>,
    closed: bool,
}

impl SwarmDisasterActivityAgentSession {
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.id
    }

    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        SWARM_DISASTER_BASELINE_PROFILE
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub fn state_hash(&self) -> AgentHash {
        AgentHash::from_bytes(self.run.state_hash(self.fixture.instance()).bytes())
    }

    #[must_use]
    pub const fn terminal(&self) -> Option<starclock_activity::ActivityTerminalOutcome> {
        self.run.terminal()
    }

    #[must_use]
    pub fn offered_actions(&self) -> &[OfferedActivityAction] {
        self.offered
            .as_ref()
            .map_or(&[], SwarmOfferedActionSet::actions)
    }

    #[must_use]
    pub fn replay_action_count(&self) -> usize {
        self.run.action_count()
    }

    /// Projects the current player-visible state and exact offer.
    pub fn observe(&self) -> Result<AgentActivityObservation, AgentError> {
        let view = self.run.player_view(self.fixture.instance());
        let offered = self
            .offered
            .as_ref()
            .map(|value| (value.boundary(), value.actions()));
        project_activity_observation(
            &view,
            ActivityObservationContext {
                session: &self.id,
                profile: SWARM_DISASTER_BASELINE_PROFILE,
                world: AREA,
                difficulty_index: DIFFICULTY_INDEX,
                offered,
                decision_kind: self
                    .offered
                    .as_ref()
                    .map(SwarmOfferedActionSet::decision_kind),
                closed: self.closed,
            },
        )
        .map_err(|_| adapter_error(false))
    }

    /// Applies one opaque action with ownership, state, boundary and
    /// idempotency checks before the mode executor sees a command.
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
        if self.closed || self.run.terminal().is_some() {
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
        let start = self.run.action_count();
        self.run
            .apply_offered_command(self.fixture.instance(), &selected)
            .map_err(|error| run_error(error, false))?;
        let (_, nested_battles) = self
            .run
            .settle_automatic(self.fixture.instance(), self.fixture.roster())
            .map_err(|error| run_error(error, true))?;
        let accepted = self.run.action_count() - start;
        if accepted > MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT {
            return Err(settlement_budget_error(true));
        }
        self.refresh_offer()?;
        let response = AgentActivityActionResponse {
            session_id: self.id.clone(),
            committed: true,
            idempotent_replay: false,
            accepted_action_token: request.action_token.clone(),
            settlement: AgentActivitySettlementSummary {
                accepted_activity_actions: AgentUInt::from_u64(
                    u64::try_from(accepted).expect("the settlement budget fits u64"),
                ),
                nested_battles: AgentUInt::from_u64(u64::from(nested_battles)),
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
            CachedSwarmResponse {
                request,
                response: response.clone(),
                canonical_json: canonical_json.into_boxed_slice(),
            },
        );
        Ok(response)
    }

    /// Exports the live terminal transcript through the canonical Swarm
    /// replay encoder.
    pub fn export_replay(&self) -> Result<AgentActivityReplayExport, AgentError> {
        let bytes = encode_incremental_swarm_replay(
            self.fixture.instance(),
            &self.run,
            self.fixture.roster(),
            self.components.clone(),
        )
        .map_err(|error| replay_error(&format!("{error:?}")))?;
        Ok(AgentActivityReplayExport::new(
            bytes,
            self.run.action_count(),
            true,
        ))
    }

    pub fn verify_replay(
        &self,
        factory: &SwarmDisasterActivityAgentSessionFactory,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        factory.verify_replay(&AgentUInt::from_u64(self.seed), bytes)
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.offered = None;
    }

    fn refresh_offer(&mut self) -> Result<(), AgentError> {
        if self.closed || self.run.terminal().is_some() {
            self.offered = None;
            return Ok(());
        }
        let view = self.run.player_view(self.fixture.instance());
        let commands = self
            .run
            .offered_commands(self.fixture.instance())
            .map_err(|error| run_error(error, false))?;
        self.offered = Some(
            SwarmOfferedActionSet::bind(&self.id, &view, &commands)
                .map_err(action_binding_error)?,
        );
        Ok(())
    }
}

struct SwarmOfferedActionSet {
    boundary: u64,
    state_hash: ActivityStateHash,
    decision_kind: AgentActivityDecisionKind,
    public: Box<[OfferedActivityAction]>,
    private: Box<[(ActionToken, GraphActivityCommand)]>,
}

impl SwarmOfferedActionSet {
    fn bind(
        session: &SessionId,
        view: &ActivityPlayerView,
        commands: &[SwarmDisasterIncrementalOffer],
    ) -> Result<Self, ActivityActionBindingError> {
        if commands.is_empty() {
            return Err(ActivityActionBindingError::NoOffer);
        }
        if commands.len() > MAX_OFFERED_ACTIVITY_ACTIONS {
            return Err(ActivityActionBindingError::TooManyActions);
        }
        let family = commands[0].2;
        if commands.iter().any(|command| command.2 != family) {
            return Err(ActivityActionBindingError::DuplicateOption);
        }
        let mut commands = commands.to_vec();
        commands.sort_by_key(|(command, _, _)| option_id(command).map(|value| value.get()));
        if commands.windows(2).any(|pair| {
            option_id(&pair[0].0).is_some() && option_id(&pair[0].0) == option_id(&pair[1].0)
        }) {
            return Err(ActivityActionBindingError::DuplicateOption);
        }
        let boundary = view.command_sequence();
        let mut public = Vec::with_capacity(commands.len());
        let mut private = Vec::with_capacity(commands.len());
        for (ordinal, (command, priority, kind)) in commands.into_iter().enumerate() {
            let option =
                option_id(&command).ok_or(ActivityActionBindingError::InvalidActionToken)?;
            let token =
                activity_action_token(session, view.state_hash(), boundary, option, ordinal)?;
            public.push(OfferedActivityAction {
                token: token.clone(),
                kind: action_kind(kind),
                label: format!("Select {kind:?} option {}.", option.get()).into_boxed_str(),
                option_id: AgentUInt::from_u64(option.get()),
                priority: Some(AgentSInt::from_i64(i64::from(priority))),
                participant_id: None,
                technique_point_cost: None,
            });
            private.push((token, command));
        }
        Ok(Self {
            boundary,
            state_hash: view.state_hash(),
            decision_kind: decision_kind(family),
            public: public.into_boxed_slice(),
            private: private.into_boxed_slice(),
        })
    }

    const fn boundary(&self) -> u64 {
        self.boundary
    }
    const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
    const fn decision_kind(&self) -> AgentActivityDecisionKind {
        self.decision_kind
    }
    fn actions(&self) -> &[OfferedActivityAction] {
        &self.public
    }
    fn select(
        &self,
        boundary: &AgentUInt,
        token: &ActionToken,
    ) -> Result<GraphActivityCommand, ActivityActionBindingError> {
        if boundary.to_u64() != self.boundary {
            return Err(ActivityActionBindingError::StaleBoundary);
        }
        self.private
            .iter()
            .find(|(candidate, _)| candidate == token)
            .map(|(_, command)| command.clone())
            .ok_or(ActivityActionBindingError::InvalidActionToken)
    }
}

fn option_id(command: &GraphActivityCommand) -> Option<starclock_activity::ActivityOptionId> {
    match command.kind() {
        GraphActivityCommandKind::ChooseOption { option } => Some(*option),
        _ => None,
    }
}

const fn action_kind(kind: ActivityDecisionKind) -> AgentActivityActionKind {
    match kind {
        ActivityDecisionKind::Encounter => AgentActivityActionKind::EngageEncounter,
        ActivityDecisionKind::ExternalOutcome => AgentActivityActionKind::SubmitExternalOutcome,
        _ => AgentActivityActionKind::SelectOption,
    }
}

const fn decision_kind(kind: ActivityDecisionKind) -> AgentActivityDecisionKind {
    match kind {
        ActivityDecisionKind::Choice => AgentActivityDecisionKind::Choice,
        ActivityDecisionKind::Route => AgentActivityDecisionKind::Route,
        ActivityDecisionKind::Encounter => AgentActivityDecisionKind::Encounter,
        ActivityDecisionKind::Preparation => AgentActivityDecisionKind::Preparation,
        ActivityDecisionKind::Reward => AgentActivityDecisionKind::Reward,
        ActivityDecisionKind::Shop => AgentActivityDecisionKind::Shop,
        ActivityDecisionKind::Service => AgentActivityDecisionKind::Service,
        ActivityDecisionKind::Roster => AgentActivityDecisionKind::Roster,
        ActivityDecisionKind::ExternalOutcome => AgentActivityDecisionKind::ExternalOutcome,
        ActivityDecisionKind::BattleReady => AgentActivityDecisionKind::BattleReady,
        ActivityDecisionKind::Checkpoint => AgentActivityDecisionKind::Checkpoint,
        ActivityDecisionKind::Abandon => AgentActivityDecisionKind::Abandon,
    }
}

fn activity_instance() -> Result<ActivityInstanceId, AgentError> {
    ActivityInstanceId::new(1).ok_or_else(invalid_request)
}

fn controller_digest() -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"agent-activity-session-v1\0external-player\0swarm-disaster\0");
    hash.update(SWARM_DISASTER_BASELINE_BATTLE_EXECUTION_REVISION.as_bytes());
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

fn run_error(error: SwarmReplayError, committed: bool) -> AgentError {
    let mut result = agent_error(
        AgentErrorCode::CombatRejected,
        "The Swarm Disaster Activity command or settlement failed.",
        committed,
    );
    let reason = format!("{error:?}");
    let bounded = if reason.len() <= 512 {
        reason.as_str()
    } else {
        "Swarm runtime returned an oversized diagnostic"
    };
    result
        .insert_detail("reason", bounded)
        .expect("bounded Swarm runtime error is valid");
    result
}

fn configuration_error() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "The Swarm Disaster Activity could not be constructed.",
        false,
    )
}

fn invalid_request() -> AgentError {
    agent_error(
        AgentErrorCode::InvalidRequest,
        "The Swarm Disaster seed is invalid.",
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

fn replay_error(reason: &str) -> AgentError {
    let mut error = agent_error(
        AgentErrorCode::ReplayDiverged,
        "The Swarm Disaster Activity replay diverged.",
        false,
    );
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

fn settlement_budget_error(committed: bool) -> AgentError {
    agent_error(
        AgentErrorCode::SettlementBudgetExceeded,
        "The Activity settlement exceeded its accepted-action budget.",
        committed,
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

#[cfg(test)]
mod tests;
