//! Gold and Gears sessions over the shared Activity agent vocabulary.

use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starclock_activity::{ActivityInstanceId, ActivityPlayerView, ActivityStateHash};
use starclock_mode_universe::{
    gold_gears_entry::{
        GoldAndGearsCommandFamily, GoldAndGearsControllerIdentity, GoldAndGearsOfferedCommand,
        GoldAndGearsRuntimeFactory, GoldAndGearsSeededRunError, GoldAndGearsSeededRunRequest,
        baseline_fixture::{GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY, GoldAndGearsBaselineFixture},
        encode_gold_and_gears_replay, gold_and_gears_replay_header,
        incremental_run::GoldAndGearsIncrementalRun,
        record_incremental_gold_and_gears_run, verify_gold_and_gears_replay,
    },
    gold_gears_identity::GoldAndGearsCatalogIdentity,
};
use starclock_replay::{component::ConfigurationComponentSet, format::ReplayHeader};

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
        AgentActivityActionResponse, AgentActivityReplayExport, AgentActivityReplayVerification,
        AgentActivitySettlementSummary, PlayActivityActionRequest,
    },
    error::{AgentError, AgentErrorCode},
    schema::{ActionToken, AgentHash, AgentSInt, AgentUInt, SessionId},
    session::{MAX_CACHED_RESPONSE_BYTES, MAX_IDEMPOTENCY_ENTRIES},
};

const GOLD_BUNDLE: &[u8] = include_bytes!("../../../config/gold-and-gears-generated/config.sora");
const PROFILE: &str = starclock_mode_universe::gold_gears_entry::GOLD_AND_GEARS_REPLAY_PROFILE;
const AREA: u32 = 401;
const DIFFICULTY_INDEX: usize = 0;
const MAX_ACTIVITY_ACTIONS_PER_SETTLEMENT: usize = 16;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateGoldAndGearsActivitySessionRequest {
    pub session_id: SessionId,
    pub seed: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentGoldAndGearsManifest {
    pub profile_id: Box<str>,
    pub fixture_accuracy: Box<str>,
    pub area: AgentUInt,
    pub path: Box<str>,
    pub custom_dice: Box<str>,
    pub component_root: AgentHash,
}

#[derive(Clone)]
pub struct GoldAndGearsActivityAgentSessionFactory {
    fixture: Arc<GoldAndGearsBaselineFixture>,
    components: ConfigurationComponentSet,
}

impl GoldAndGearsActivityAgentSessionFactory {
    pub fn load_production() -> Result<Self, AgentError> {
        let identity =
            GoldAndGearsCatalogIdentity::load(GOLD_BUNDLE).map_err(|_| configuration_error())?;
        let runtime = GoldAndGearsRuntimeFactory::load_candidate(GOLD_BUNDLE)
            .map_err(|_| configuration_error())?;
        let fixture = runtime
            .compile_synthetic_baseline_fixture(&identity)
            .map_err(|_| configuration_error())?;
        let components = fixture
            .components_for_controller(GoldAndGearsControllerIdentity {
                id: "agent-activity-controller",
                digest: controller_digest(),
            })
            .map_err(|_| configuration_error())?;
        Ok(Self {
            fixture: Arc::new(fixture),
            components,
        })
    }

    pub fn create(
        &self,
        request: CreateGoldAndGearsActivitySessionRequest,
    ) -> Result<GoldAndGearsActivityAgentSession, AgentError> {
        let run_request = run_request(request.seed.to_u64(), &self.fixture)?;
        let replay_header = gold_and_gears_replay_header(
            self.components.clone(),
            run_request,
            self.fixture.roster(),
        )
        .map_err(|_| adapter_error(false))?;
        let mut run = GoldAndGearsIncrementalRun::start(self.fixture.instance(), run_request);
        run.settle_automatic(self.fixture.instance(), self.fixture.roster())
            .map_err(|error| run_error(error, true))?;
        let mut session = GoldAndGearsActivityAgentSession {
            id: request.session_id,
            seed: request.seed.to_u64(),
            fixture: Arc::clone(&self.fixture),
            replay_header,
            run,
            offered: None,
            idempotency: BTreeMap::new(),
            closed: false,
        };
        session.refresh_offer()?;
        Ok(session)
    }

    #[must_use]
    pub fn manifest(&self) -> AgentGoldAndGearsManifest {
        AgentGoldAndGearsManifest {
            profile_id: PROFILE.into(),
            fixture_accuracy: GOLD_AND_GEARS_BASELINE_FIXTURE_ACCURACY.into(),
            area: AgentUInt::from_u64(u64::from(AREA)),
            path: self.fixture.path().into(),
            custom_dice: self.fixture.custom_dice().into(),
            component_root: AgentHash::from_bytes(self.components.root().bytes()),
        }
    }

    pub fn verify_replay(
        &self,
        seed: &AgentUInt,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let request = run_request(seed.to_u64(), &self.fixture)?;
        let report = verify_gold_and_gears_replay(
            bytes,
            self.fixture.instance(),
            request,
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

#[cfg(test)]
pub(crate) fn production_factory_for_tests() -> GoldAndGearsActivityAgentSessionFactory {
    static FACTORY: std::sync::OnceLock<GoldAndGearsActivityAgentSessionFactory> =
        std::sync::OnceLock::new();
    FACTORY
        .get_or_init(|| {
            GoldAndGearsActivityAgentSessionFactory::load_production()
                .expect("production factory loads")
        })
        .clone()
}

struct CachedGoldResponse {
    request: PlayActivityActionRequest,
    response: AgentActivityActionResponse,
    canonical_json: Box<[u8]>,
}

pub struct GoldAndGearsActivityAgentSession {
    id: SessionId,
    seed: u64,
    fixture: Arc<GoldAndGearsBaselineFixture>,
    replay_header: ReplayHeader,
    run: GoldAndGearsIncrementalRun,
    offered: Option<GoldOfferedActionSet>,
    idempotency: BTreeMap<crate::schema::IdempotencyKey, CachedGoldResponse>,
    closed: bool,
}

impl GoldAndGearsActivityAgentSession {
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.id
    }

    #[must_use]
    pub const fn profile_id(&self) -> &'static str {
        PROFILE
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
            .map_or(&[], GoldOfferedActionSet::actions)
    }

    #[must_use]
    pub fn replay_action_count(&self) -> usize {
        self.run.action_count()
    }

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
                profile: PROFILE,
                world: AREA,
                difficulty_index: DIFFICULTY_INDEX,
                offered,
                decision_kind: self
                    .offered
                    .as_ref()
                    .map(GoldOfferedActionSet::decision_kind),
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
        let settlement = self
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
                nested_battles: AgentUInt::from_u64(u64::from(settlement.nested_battles())),
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
            CachedGoldResponse {
                request,
                response: response.clone(),
                canonical_json: canonical_json.into_boxed_slice(),
            },
        );
        Ok(response)
    }

    pub fn export_replay(&self) -> Result<AgentActivityReplayExport, AgentError> {
        let recorded = record_incremental_gold_and_gears_run(self.fixture.instance(), &self.run)
            .map_err(|error| replay_error(&format!("{error:?}")))?;
        let bytes = encode_gold_and_gears_replay(&self.replay_header, &recorded)
            .map_err(|error| replay_error(&format!("{error:?}")))?;
        Ok(AgentActivityReplayExport::new(
            bytes,
            self.run.action_count(),
            true,
        ))
    }

    pub fn verify_replay(
        &self,
        factory: &GoldAndGearsActivityAgentSessionFactory,
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
            GoldOfferedActionSet::bind(&self.id, &view, &commands).map_err(action_binding_error)?,
        );
        Ok(())
    }
}

struct GoldOfferedActionSet {
    boundary: u64,
    state_hash: ActivityStateHash,
    decision_kind: AgentActivityDecisionKind,
    public: Box<[OfferedActivityAction]>,
    private: Box<[(ActionToken, GoldAndGearsOfferedCommand)]>,
}

impl GoldOfferedActionSet {
    fn bind(
        session: &SessionId,
        view: &ActivityPlayerView,
        commands: &[GoldAndGearsOfferedCommand],
    ) -> Result<Self, ActivityActionBindingError> {
        if commands.is_empty() {
            return Err(ActivityActionBindingError::NoOffer);
        }
        if commands.len() > MAX_OFFERED_ACTIVITY_ACTIONS {
            return Err(ActivityActionBindingError::TooManyActions);
        }
        let family = commands[0].family();
        if commands.iter().any(|command| command.family() != family) {
            return Err(ActivityActionBindingError::DuplicateOption);
        }
        let mut commands = commands.to_vec();
        commands.sort_by_key(GoldAndGearsOfferedCommand::id);
        if commands.windows(2).any(|pair| pair[0].id() == pair[1].id()) {
            return Err(ActivityActionBindingError::DuplicateOption);
        }
        let boundary = view.command_sequence();
        let mut public = Vec::with_capacity(commands.len());
        let mut private = Vec::with_capacity(commands.len());
        for (ordinal, command) in commands.into_iter().enumerate() {
            let token =
                activity_action_token(session, view.state_hash(), boundary, command.id(), ordinal)?;
            public.push(OfferedActivityAction {
                token: token.clone(),
                kind: action_kind(command.family()),
                label: format!(
                    "Select {:?} option {}.",
                    command.family(),
                    command.id().get()
                )
                .into_boxed_str(),
                option_id: AgentUInt::from_u64(command.id().get()),
                priority: Some(AgentSInt::from_i64(i64::from(command.authored_priority()))),
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
    ) -> Result<GoldAndGearsOfferedCommand, ActivityActionBindingError> {
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

const fn action_kind(family: GoldAndGearsCommandFamily) -> AgentActivityActionKind {
    match family {
        GoldAndGearsCommandFamily::BossSelection => AgentActivityActionKind::EngageEncounter,
        GoldAndGearsCommandFamily::AdventureOutcome => {
            AgentActivityActionKind::SubmitExternalOutcome
        }
        _ => AgentActivityActionKind::SelectOption,
    }
}

const fn decision_kind(family: GoldAndGearsCommandFamily) -> AgentActivityDecisionKind {
    match family {
        GoldAndGearsCommandFamily::Route => AgentActivityDecisionKind::Route,
        GoldAndGearsCommandFamily::BossSelection => AgentActivityDecisionKind::Encounter,
        GoldAndGearsCommandFamily::Reward => AgentActivityDecisionKind::Reward,
        GoldAndGearsCommandFamily::Service => AgentActivityDecisionKind::Service,
        GoldAndGearsCommandFamily::AdventureOutcome => AgentActivityDecisionKind::ExternalOutcome,
        _ => AgentActivityDecisionKind::Choice,
    }
}

fn run_request(
    seed: u64,
    fixture: &GoldAndGearsBaselineFixture,
) -> Result<GoldAndGearsSeededRunRequest, AgentError> {
    Ok(GoldAndGearsSeededRunRequest::new(
        seed,
        fixture.activity_identity(),
        ActivityInstanceId::new(1).ok_or_else(invalid_request)?,
    ))
}

fn controller_digest() -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(b"agent-activity-session-v1\0external-player\0gold-and-gears\0");
    hash.update(b"gold-and-gears-nested-battle-execution");
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

fn run_error(error: GoldAndGearsSeededRunError, committed: bool) -> AgentError {
    match error {
        GoldAndGearsSeededRunError::StepBudgetExceeded => settlement_budget_error(committed),
        GoldAndGearsSeededRunError::CommandNotOffered => agent_error(
            AgentErrorCode::InvalidActionToken,
            "The Activity command is not in the current exact offer.",
            committed,
        ),
        _ => {
            let mut result = agent_error(
                AgentErrorCode::CombatRejected,
                "The Gold and Gears Activity command or settlement failed.",
                committed,
            );
            let reason = format!("{error:?}");
            let bounded = if reason.len() <= 512 {
                reason.as_str()
            } else {
                "Gold runtime returned an oversized diagnostic"
            };
            result
                .insert_detail("reason", bounded)
                .expect("bounded Gold runtime error is valid");
            result
        }
    }
}

fn configuration_error() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "The Gold and Gears Activity could not be constructed.",
        false,
    )
}

fn invalid_request() -> AgentError {
    agent_error(
        AgentErrorCode::InvalidRequest,
        "The Gold and Gears seed is invalid.",
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
        "The Gold and Gears Activity replay diverged.",
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
