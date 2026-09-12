//! Divergent Universe sessions over the shared Activity agent vocabulary.

use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use starclock_activity::{ActivityInstanceId, ActivityMasterSeed, GraphActivity};
use starclock_data::divergent_universe_catalog::DivergentUniverseRunFamily;
use starclock_mode_universe::divergent_universe::{
    DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE, DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE,
    DivergentUniverseBaselineError, DivergentUniverseBaselineFixture,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep, DivergentUniverseFlowInstance,
    DivergentUniverseOfferedSelection, DivergentUniverseReplayError,
    encode_divergent_universe_replay, record_divergent_universe_transcript,
    verify_divergent_universe_replay,
};
use starclock_replay::format::decode_replay;

use super::activity_session;
use crate::{
    activity_action::{ActivityActionBindingError, BoundActivityAction, OfferedActivityActionSet},
    activity_observation::{
        ActivityObservationContext, AgentActivityObservation, project_activity_observation,
    },
    activity_session::{
        AgentActivityActionResponse, AgentActivityReplayExport, AgentActivityReplayVerification,
        AgentActivitySettlementSummary, PlayActivityActionRequest,
    },
    error::{AgentError, AgentErrorCode},
    schema::{AgentHash, AgentUInt, IdempotencyKey, SessionId},
    session::{MAX_CACHED_RESPONSE_BYTES, MAX_IDEMPOTENCY_ENTRIES},
};

const ORDINARY_AREA: u32 = 401;
const CYCLICAL_AREA: u32 = 20_401;
const DIFFICULTY_INDEX: usize = 0;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentDivergentUniverseRunFamily {
    Ordinary,
    Cyclical,
}

impl AgentDivergentUniverseRunFamily {
    const fn runtime(self) -> DivergentUniverseRunFamily {
        match self {
            Self::Ordinary => DivergentUniverseRunFamily::Ordinary,
            Self::Cyclical => DivergentUniverseRunFamily::Cyclical,
        }
    }

    const fn profile(self) -> &'static str {
        match self {
            Self::Ordinary => DIVERGENT_UNIVERSE_ORDINARY_REPLAY_PROFILE,
            Self::Cyclical => DIVERGENT_UNIVERSE_CYCLICAL_REPLAY_PROFILE,
        }
    }

    const fn area(self) -> u32 {
        match self {
            Self::Ordinary => ORDINARY_AREA,
            Self::Cyclical => CYCLICAL_AREA,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateDivergentUniverseActivitySessionRequest {
    pub session_id: SessionId,
    pub family: AgentDivergentUniverseRunFamily,
    pub seed: AgentUInt,
    /// Explicit policy-bound initial service, 2..5; invalid levels reject creation.
    #[serde(default)]
    pub tawot_forge_level: Option<AgentUInt>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentDivergentUniverseFamilyManifest {
    pub family: AgentDivergentUniverseRunFamily,
    pub profile_id: Box<str>,
    pub area: AgentUInt,
    pub component_root: AgentHash,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentDivergentUniverseManifest {
    pub mode_id: Box<str>,
    pub families: Box<[AgentDivergentUniverseFamilyManifest]>,
    pub configuration_tables: AgentUInt,
    pub source_obligations: AgentUInt,
    pub mechanic_programs: AgentUInt,
}

#[derive(Clone)]
pub struct DivergentUniverseActivityAgentSessionFactory {
    fixture: Arc<DivergentUniverseBaselineFixture>,
}

impl DivergentUniverseActivityAgentSessionFactory {
    pub fn load_production() -> Result<Self, AgentError> {
        let fixture =
            DivergentUniverseBaselineFixture::production().map_err(|_| configuration_error())?;
        Ok(Self {
            fixture: Arc::new(fixture),
        })
    }

    pub fn create(
        &self,
        request: CreateDivergentUniverseActivitySessionRequest,
    ) -> Result<DivergentUniverseActivityAgentSession, AgentError> {
        let family = request.family;
        let flow = if let Some(level) = request.tawot_forge_level {
            let level = u16::try_from(level.to_u64())
                .ok()
                .filter(|level| (2..=5).contains(level))
                .ok_or_else(invalid_request)?;
            self.fixture
                .flow_with_tawot_service(family.runtime(), level)
        } else {
            self.fixture.flow(family.runtime())
        }
        .map_err(|_| configuration_error())?;
        let activity = flow
            .start(
                ActivityInstanceId::new(1).ok_or_else(invalid_request)?,
                ActivityMasterSeed::from_u64(request.seed.to_u64()),
            )
            .map_err(|_| configuration_error())?
            .into_activity();
        let mut session = DivergentUniverseActivityAgentSession {
            id: request.session_id,
            family,
            seed: request.seed.to_u64(),
            fixture: Arc::clone(&self.fixture),
            flow,
            activity,
            steps: Vec::new(),
            offered: None,
            idempotency: BTreeMap::new(),
            closed: false,
        };
        session.refresh_offer()?;
        Ok(session)
    }

    pub fn manifest(&self) -> Result<AgentDivergentUniverseManifest, AgentError> {
        let families = [
            AgentDivergentUniverseRunFamily::Ordinary,
            AgentDivergentUniverseRunFamily::Cyclical,
        ]
        .into_iter()
        .map(|family| {
            let flow = self
                .fixture
                .flow(family.runtime())
                .map_err(|_| configuration_error())?;
            let components = self
                .fixture
                .components(&flow)
                .map_err(|_| configuration_error())?;
            Ok(AgentDivergentUniverseFamilyManifest {
                family,
                profile_id: family.profile().into(),
                area: AgentUInt::from_u64(u64::from(family.area())),
                component_root: AgentHash::from_bytes(components.root().bytes()),
            })
        })
        .collect::<Result<Vec<_>, AgentError>>()?;
        Ok(AgentDivergentUniverseManifest {
            mode_id: "divergent-universe".into(),
            families: families.into_boxed_slice(),
            configuration_tables: AgentUInt::from_u64(80),
            source_obligations: AgentUInt::from_u64(6_215),
            mechanic_programs: AgentUInt::from_u64(669),
        })
    }

    pub fn verify_replay(
        &self,
        seed: &AgentUInt,
        family: AgentDivergentUniverseRunFamily,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let decoded = decode_replay(bytes).map_err(|_| replay_seed_error())?;
        if decoded.header().master_seed() != seed.to_u64() {
            return Err(replay_seed_error());
        }
        let report =
            verify_divergent_universe_replay(bytes, &self.fixture).map_err(replay_error)?;
        if report.run_family() != family.runtime() {
            return Err(replay_seed_error());
        }
        Ok(AgentActivityReplayVerification {
            action_count: AgentUInt::from_u64(u64::from(report.action_count())),
            nested_battles: AgentUInt::from_u64(u64::from(report.battle_count())),
            final_state_hash: AgentHash::from_bytes(report.final_state_hash().bytes()),
            terminal: activity_session::terminal(report.terminal()),
        })
    }
}

#[cfg(test)]
pub(crate) fn production_factory_for_tests() -> DivergentUniverseActivityAgentSessionFactory {
    static FACTORY: std::sync::OnceLock<DivergentUniverseActivityAgentSessionFactory> =
        std::sync::OnceLock::new();
    FACTORY
        .get_or_init(|| {
            DivergentUniverseActivityAgentSessionFactory::load_production()
                .expect("production factory loads")
        })
        .clone()
}

struct CachedResponse {
    request: PlayActivityActionRequest,
    response: AgentActivityActionResponse,
    canonical_json: Box<[u8]>,
}

pub struct DivergentUniverseActivityAgentSession {
    id: SessionId,
    family: AgentDivergentUniverseRunFamily,
    seed: u64,
    fixture: Arc<DivergentUniverseBaselineFixture>,
    flow: DivergentUniverseFlowInstance,
    activity: GraphActivity,
    steps: Vec<DivergentUniverseBaselineStep>,
    offered: Option<OfferedActivityActionSet>,
    idempotency: BTreeMap<IdempotencyKey, CachedResponse>,
    closed: bool,
}

impl DivergentUniverseActivityAgentSession {
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.id
    }

    #[must_use]
    pub fn state_hash(&self) -> AgentHash {
        AgentHash::from_bytes(self.activity.state_hash().bytes())
    }

    #[must_use]
    pub fn terminal(&self) -> Option<starclock_activity::ActivityTerminalOutcome> {
        self.activity.player_view().terminal()
    }

    #[must_use]
    pub fn replay_action_count(&self) -> usize {
        self.steps.len()
    }

    pub fn observe(&self) -> Result<AgentActivityObservation, AgentError> {
        let view = self.activity.player_view();
        let offered = self
            .offered
            .as_ref()
            .map(|value| (value.boundary(), value.actions()));
        let mut observation = project_activity_observation(
            &view,
            ActivityObservationContext {
                session: &self.id,
                profile: self.family.profile(),
                world: self.family.area(),
                difficulty_index: DIFFICULTY_INDEX,
                offered,
                decision_kind: None,
                closed: self.closed,
            },
        )
        .map_err(|_| adapter_error(false))?;
        if let Some(choices) = self
            .flow
            .offered_battle_domain_choices(&self.activity)
            .map_err(|_| adapter_error(false))?
        {
            for action in &mut observation.legal_actions {
                let choice = choices
                    .iter()
                    .find(|choice| u64::from(choice.ordinal) == action.option_id.to_u64())
                    .ok_or_else(|| adapter_error(false))?;
                // Presentation only: retain the exact shared opaque token and
                // option binding. Domain selection still uses the same command.
                action.label = choice.name_en.clone();
            }
        }
        Ok(observation)
    }

    pub fn apply_action(
        &mut self,
        request: PlayActivityActionRequest,
    ) -> Result<AgentActivityActionResponse, AgentError> {
        self.validate_request(&request)?;
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
            .map_err(action_binding_error)?
            .into_action();
        let BoundActivityAction::Decision {
            decision, option, ..
        } = selected
        else {
            return Err(adapter_error(false));
        };
        let step = DivergentUniverseBaselineRunner::default()
            .advance_selected(
                self.fixture.factory(),
                &self.flow,
                &mut self.activity,
                self.fixture.core(),
                &self.fixture.policy().map_err(|_| configuration_error())?,
                DivergentUniverseOfferedSelection::new(decision, option),
            )
            .map_err(run_error)?;
        // A rejected domain command keeps its authenticated offer usable.
        // Retire tokens only after a successful step, before binding the next one.
        self.offered = None;
        let nested_battles =
            usize::from(matches!(step, DivergentUniverseBaselineStep::Battle { .. }));
        self.steps.push(step);
        self.refresh_offer()?;
        let response = AgentActivityActionResponse {
            session_id: self.id.clone(),
            committed: true,
            idempotent_replay: false,
            accepted_action_token: request.action_token.clone(),
            settlement: AgentActivitySettlementSummary {
                accepted_activity_actions: AgentUInt::from_u64(1),
                nested_battles: AgentUInt::from_u64(nested_battles as u64),
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
            CachedResponse {
                request,
                response: response.clone(),
                canonical_json: canonical_json.into_boxed_slice(),
            },
        );
        Ok(response)
    }

    pub fn export_replay(&self) -> Result<AgentActivityReplayExport, AgentError> {
        let recorded = record_divergent_universe_transcript(
            &self.fixture,
            &self.flow,
            &self.activity,
            self.seed,
            self.steps.clone(),
        )
        .map_err(replay_error)?;
        let bytes = encode_divergent_universe_replay(&recorded).map_err(replay_error)?;
        Ok(AgentActivityReplayExport::new(
            bytes,
            self.steps.len(),
            true,
        ))
    }

    pub fn verify_replay(
        &self,
        factory: &DivergentUniverseActivityAgentSessionFactory,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        factory.verify_replay(&AgentUInt::from_u64(self.seed), self.family, bytes)
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.offered = None;
    }

    fn validate_request(&self, request: &PlayActivityActionRequest) -> Result<(), AgentError> {
        if request.session_id != self.id {
            return Err(agent_error(
                AgentErrorCode::SessionNotOwned,
                "The Activity action does not belong to this session.",
                false,
            ));
        }
        if self.closed || self.terminal().is_some() {
            return Err(agent_error(
                AgentErrorCode::SessionClosed,
                "The Activity session has already settled or closed.",
                false,
            ));
        }
        Ok(())
    }

    fn refresh_offer(&mut self) -> Result<(), AgentError> {
        if self.closed || self.terminal().is_some() {
            self.offered = None;
            return Ok(());
        }
        let view = self.activity.player_view();
        self.offered =
            Some(OfferedActivityActionSet::bind(&self.id, &view).map_err(action_binding_error)?);
        Ok(())
    }
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

fn run_error(error: DivergentUniverseBaselineError) -> AgentError {
    let mut result = agent_error(
        AgentErrorCode::CombatRejected,
        "The Divergent Universe Activity command or settlement failed.",
        false,
    );
    let reason = format!("{error:?}");
    result
        .insert_detail("reason", &reason[..reason.len().min(512)])
        .expect("bounded runtime error is valid");
    result
}

fn replay_error(error: DivergentUniverseReplayError) -> AgentError {
    let mut result = agent_error(
        AgentErrorCode::ReplayDiverged,
        "The Divergent Universe Activity replay diverged.",
        false,
    );
    let reason = format!("{error:?}");
    result
        .insert_detail("reason", &reason[..reason.len().min(512)])
        .expect("bounded replay error is valid");
    result
}

fn replay_seed_error() -> AgentError {
    agent_error(
        AgentErrorCode::ReplayDiverged,
        "The Divergent Universe replay seed or envelope diverged.",
        false,
    )
}

fn configuration_error() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "The Divergent Universe Activity could not be constructed.",
        false,
    )
}

fn invalid_request() -> AgentError {
    agent_error(
        AgentErrorCode::InvalidRequest,
        "The Divergent Universe request is invalid.",
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

fn adapter_error(committed: bool) -> AgentError {
    agent_error(
        AgentErrorCode::AdapterFailure,
        "The stable Activity boundary could not be projected or encoded.",
        committed,
    )
}

fn agent_error(code: AgentErrorCode, message: &'static str, committed: bool) -> AgentError {
    AgentError::new(code, message, false, committed)
        .expect("static Divergent Universe agent error is valid")
}

#[cfg(test)]
mod tests;
