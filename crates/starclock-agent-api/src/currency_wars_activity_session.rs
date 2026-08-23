//! Currency Wars sessions over the shared Activity agent vocabulary.

use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};
use starclock_activity::{
    ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, AttemptId,
};
use starclock_ai::{
    CurrencyWarsBaselineActivityAction, CurrencyWarsBaselineActivityTraceEntry,
    CurrencyWarsBaselineBattleReport, CurrencyWarsBaselineController,
    CurrencyWarsBaselineControllerError, CurrencyWarsBaselineRunReport, CurrencyWarsReplayGambit,
    CurrencyWarsReplayIdentity, CurrencyWarsReplayRequest, decode_currency_wars_replay_request,
    encode_currency_wars_replay, verify_currency_wars_replay,
};
use starclock_combat::catalog::CombatCatalog;
use starclock_data::{
    currency_wars::{CurrencyWarsCatalogIdentity, load_currency_wars_catalog_candidate},
    load_currency_wars_battle_resources,
};
use starclock_mode_currency_wars::{
    CurrencyWarsBattleAssembler, CurrencyWarsBattleResources, CurrencyWarsCatalog,
    CurrencyWarsDeployment, CurrencyWarsEntryState, CurrencyWarsGambit, CurrencyWarsPosition,
    CurrencyWarsPositionKind, CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster,
    CurrencyWarsRun, CurrencyWarsRunDefinition, CurrencyWarsRunSetup, CurrencyWarsRuntimeError,
};

use crate::{
    activity_action::{
        ActivityActionBindingError, BoundActivityAction, OfferedActivityAction,
        OfferedActivityActionSet,
    },
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

pub const RESPONSIBILITY: &str =
    "bounded Currency Wars manifests and opaque incremental Activity sessions";

const GAME_VERSION: &str = "4.4";
const PROFILE_PREFIX: &str = "currency-wars";
const DEFINITION_ID: u32 = 31;
const ACTIVITY_INSTANCE: u64 = 1;
const INITIAL_TEAM_LEVEL: u8 = 4;
const INITIAL_ROLES: [u32; 4] = [1301, 1306, 1014, 1015];
const MAX_BATTLE_CACHE_ENTRIES: usize = 16;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentCurrencyWarsGambit {
    Standard,
    Overclock,
}

impl AgentCurrencyWarsGambit {
    const fn domain(self) -> CurrencyWarsGambit {
        match self {
            Self::Standard => CurrencyWarsGambit::Standard,
            Self::Overclock => CurrencyWarsGambit::Overclock,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Overclock => "overclock",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentCurrencyWarsRouteSummary {
    pub route_id: AgentUInt,
    pub stable_key: Box<str>,
    pub node_count: AgentUInt,
    pub difficulty_ids: Box<[AgentUInt]>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentCurrencyWarsDifficultySummary {
    pub difficulty_id: AgentUInt,
    pub stable_key: Box<str>,
    pub season_id: AgentUInt,
    pub division_level: AgentUInt,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentCurrencyWarsManifest {
    pub profile_prefix: Box<str>,
    pub game_version: Box<str>,
    pub schema_fingerprint: Box<str>,
    pub configuration_digest: AgentHash,
    pub content_digest: AgentHash,
    pub gambits: Box<[AgentCurrencyWarsGambit]>,
    pub fixture_role_ids: Box<[AgentUInt]>,
    pub routes: Box<[AgentCurrencyWarsRouteSummary]>,
    pub difficulties: Box<[AgentCurrencyWarsDifficultySummary]>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CreateCurrencyWarsActivitySessionRequest {
    pub session_id: SessionId,
    pub route_id: AgentUInt,
    pub difficulty_id: AgentUInt,
    pub gambit: AgentCurrencyWarsGambit,
    pub seed: AgentUInt,
}

#[derive(Clone)]
pub struct CurrencyWarsActivityAgentSessionFactory {
    catalog: Arc<CurrencyWarsCatalog>,
    identity: CurrencyWarsCatalogIdentity,
    battle_resources: Arc<CurrencyWarsBattleResources>,
}

impl CurrencyWarsActivityAgentSessionFactory {
    pub fn load_production() -> Result<Self, AgentError> {
        let candidate =
            load_currency_wars_catalog_candidate().map_err(|_| configuration_error())?;
        let identity = candidate.identity().clone();
        let catalog = Arc::new(candidate.into_catalog());
        let battle_resources = Arc::new(
            load_currency_wars_battle_resources(&catalog).map_err(|_| configuration_error())?,
        );
        Ok(Self {
            catalog,
            identity,
            battle_resources,
        })
    }

    #[must_use]
    pub fn manifest(&self) -> AgentCurrencyWarsManifest {
        AgentCurrencyWarsManifest {
            profile_prefix: PROFILE_PREFIX.into(),
            game_version: GAME_VERSION.into(),
            schema_fingerprint: self.identity.schema_fingerprint().into(),
            configuration_digest: AgentHash::from_bytes(
                self.identity.configuration_digest().bytes(),
            ),
            content_digest: AgentHash::from_bytes(self.identity.content_digest().bytes()),
            gambits: vec![
                AgentCurrencyWarsGambit::Standard,
                AgentCurrencyWarsGambit::Overclock,
            ]
            .into_boxed_slice(),
            fixture_role_ids: INITIAL_ROLES
                .into_iter()
                .map(|role| AgentUInt::from_u64(u64::from(role)))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            routes: self
                .catalog
                .routes()
                .iter()
                .map(|route| AgentCurrencyWarsRouteSummary {
                    route_id: AgentUInt::from_u64(u64::from(route.id.get())),
                    stable_key: route.stable_key.clone(),
                    node_count: AgentUInt::from_u64(route.nodes.len() as u64),
                    difficulty_ids: route
                        .difficulty_ids
                        .iter()
                        .copied()
                        .map(|difficulty| AgentUInt::from_u64(u64::from(difficulty)))
                        .collect::<Vec<_>>()
                        .into_boxed_slice(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            difficulties: self
                .catalog
                .difficulties()
                .iter()
                .map(|difficulty| AgentCurrencyWarsDifficultySummary {
                    difficulty_id: AgentUInt::from_u64(u64::from(difficulty.source_id)),
                    stable_key: difficulty.stable_key.clone(),
                    season_id: AgentUInt::from_u64(u64::from(difficulty.season_id)),
                    division_level: AgentUInt::from_u64(u64::from(difficulty.division_level)),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        }
    }

    pub fn create(
        &self,
        request: CreateCurrencyWarsActivitySessionRequest,
    ) -> Result<CurrencyWarsActivityAgentSession, AgentError> {
        let route_id = u32::try_from(request.route_id.to_u64()).map_err(|_| invalid_request())?;
        let difficulty_id =
            u32::try_from(request.difficulty_id.to_u64()).map_err(|_| invalid_request())?;
        let route = self
            .catalog
            .routes()
            .iter()
            .find(|route| route.id.get() == route_id)
            .ok_or_else(invalid_request)?;
        let difficulty_index = self
            .catalog
            .difficulties()
            .iter()
            .position(|difficulty| difficulty.source_id == difficulty_id)
            .ok_or_else(invalid_request)?;
        let definition = Arc::new(
            CurrencyWarsRunDefinition::new(
                activity_identity(&self.identity)?,
                Arc::clone(&self.catalog),
                route.id,
                difficulty_id,
                request.gambit.domain(),
                CurrencyWarsEntryState::new(21, true, 9),
                baseline_setup(&self.catalog)?,
            )
            .map_err(|_| configuration_error())?,
        );
        let run = CurrencyWarsRun::start(
            definition,
            ActivityInstanceId::new(ACTIVITY_INSTANCE).ok_or_else(configuration_error)?,
            ActivityMasterSeed::from_u64(request.seed.to_u64()),
        )
        .map_err(|_| configuration_error())?;
        let assembler = CurrencyWarsBattleAssembler::new(
            Arc::clone(&self.battle_resources),
            MAX_BATTLE_CACHE_ENTRIES,
        )
        .map_err(|_| configuration_error())?;
        let profile = format!(
            "{PROFILE_PREFIX}/route-{route_id}/difficulty-{difficulty_id}/gambit-{}",
            request.gambit.name(),
        )
        .into_boxed_str();
        let mut session = CurrencyWarsActivityAgentSession {
            id: request.session_id,
            profile,
            route_id,
            difficulty_index,
            difficulty_id,
            gambit: request.gambit,
            seed: request.seed.to_u64(),
            replay_identity: self.replay_identity(),
            run,
            assembler,
            controller: CurrencyWarsBaselineController::default(),
            prepared_catalog: None,
            battle_reports: Vec::new(),
            activity_trace: Vec::new(),
            supply_decisions: 0,
            route_decisions: 0,
            accepted_actions: 0,
            offered: None,
            idempotency: BTreeMap::new(),
            closed: false,
        };
        session.refresh_offer()?;
        Ok(session)
    }

    pub fn verify_replay(
        &self,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        let request = decode_currency_wars_replay_request(bytes).map_err(replay_error)?;
        let mut session = self.create(CreateCurrencyWarsActivitySessionRequest {
            session_id: SessionId::parse("currency-wars-replay")
                .map_err(|_| adapter_error(false))?,
            route_id: AgentUInt::from_u64(u64::from(request.route_id())),
            difficulty_id: AgentUInt::from_u64(u64::from(request.difficulty_id())),
            gambit: match request.gambit() {
                CurrencyWarsReplayGambit::Standard => AgentCurrencyWarsGambit::Standard,
                CurrencyWarsReplayGambit::Overclock => AgentCurrencyWarsGambit::Overclock,
            },
            seed: AgentUInt::from_u64(request.seed()),
        })?;
        let report = session
            .controller
            .run_to_terminal(&mut session.run, &mut session.assembler)
            .map_err(|error| battle_error(error, false))?;
        verify_currency_wars_replay(bytes, request, self.replay_identity(), &report)
            .map_err(replay_error)?;
        Ok(AgentActivityReplayVerification {
            action_count: AgentUInt::from_u64(u64::from(report.activity_steps())),
            nested_battles: AgentUInt::from_u64(report.battles().len() as u64),
            final_state_hash: AgentHash::from_bytes(report.final_state_hash().bytes()),
            terminal: crate::activity_session::terminal(report.terminal()),
        })
    }

    fn replay_identity(&self) -> CurrencyWarsReplayIdentity {
        CurrencyWarsReplayIdentity::new(
            self.identity.schema_digest().bytes(),
            self.identity.configuration_digest().bytes(),
            self.identity.content_digest().bytes(),
            self.battle_resources.digest(),
            self.battle_resources.combat().digest().bytes(),
        )
    }
}

#[cfg(test)]
pub(crate) fn production_factory_for_tests() -> CurrencyWarsActivityAgentSessionFactory {
    static FACTORY: std::sync::OnceLock<CurrencyWarsActivityAgentSessionFactory> =
        std::sync::OnceLock::new();
    FACTORY
        .get_or_init(|| {
            CurrencyWarsActivityAgentSessionFactory::load_production()
                .expect("production Currency Wars factory loads")
        })
        .clone()
}

struct CachedCurrencyWarsResponse {
    request: PlayActivityActionRequest,
    response: AgentActivityActionResponse,
    canonical_json: Box<[u8]>,
}

pub struct CurrencyWarsActivityAgentSession {
    id: SessionId,
    profile: Box<str>,
    route_id: u32,
    difficulty_index: usize,
    difficulty_id: u32,
    gambit: AgentCurrencyWarsGambit,
    seed: u64,
    replay_identity: CurrencyWarsReplayIdentity,
    run: CurrencyWarsRun,
    assembler: CurrencyWarsBattleAssembler,
    controller: CurrencyWarsBaselineController,
    prepared_catalog: Option<Arc<CombatCatalog>>,
    battle_reports: Vec<CurrencyWarsBaselineBattleReport>,
    activity_trace: Vec<CurrencyWarsBaselineActivityTraceEntry>,
    supply_decisions: u32,
    route_decisions: u32,
    accepted_actions: usize,
    offered: Option<OfferedActivityActionSet>,
    idempotency: BTreeMap<IdempotencyKey, CachedCurrencyWarsResponse>,
    closed: bool,
}

impl CurrencyWarsActivityAgentSession {
    #[must_use]
    pub const fn session_id(&self) -> &SessionId {
        &self.id
    }

    #[must_use]
    pub fn profile_id(&self) -> &str {
        &self.profile
    }

    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    #[must_use]
    pub fn state_hash(&self) -> AgentHash {
        AgentHash::from_bytes(self.run.state_hash().bytes())
    }

    #[must_use]
    pub fn offered_actions(&self) -> &[OfferedActivityAction] {
        self.offered
            .as_ref()
            .map_or(&[], OfferedActivityActionSet::actions)
    }

    #[must_use]
    pub fn accepted_action_count(&self) -> usize {
        self.accepted_actions
    }

    #[must_use]
    pub fn nested_battle_count(&self) -> usize {
        self.battle_reports.len()
    }

    pub fn observe(&self) -> Result<AgentActivityObservation, AgentError> {
        let view = self.run.player_view();
        let offered = self
            .offered
            .as_ref()
            .map(|value| (value.boundary(), value.actions()));
        project_activity_observation(
            &view,
            ActivityObservationContext {
                session: &self.id,
                profile: &self.profile,
                world: self.route_id,
                difficulty_index: self.difficulty_index,
                offered,
                decision_kind: None,
                closed: self.closed,
            },
        )
        .map_err(|_| adapter_error(false))
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
                "The Currency Wars idempotency key is bound to another request.",
                false,
            ));
        }
        if self.idempotency.len() == MAX_IDEMPOTENCY_ENTRIES {
            return Err(agent_error(
                AgentErrorCode::SessionQuotaExceeded,
                "The Currency Wars idempotency cache reached its fixed limit.",
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
                "The expected hash does not match the current Currency Wars state.",
                false,
            ));
        }
        let selected = offered
            .select(&request.boundary_id, &request.action_token)
            .map_err(action_binding_error)?;
        let selected = selected.into_action();
        let trace_action = baseline_trace_action(selected)?;
        let nested_battles = match self.apply_selected(selected) {
            Ok(value) => value,
            Err(error) => {
                if error.committed {
                    self.offered = None;
                }
                return Err(error);
            }
        };
        self.offered = None;
        self.accepted_actions = self
            .accepted_actions
            .checked_add(1)
            .ok_or_else(|| settlement_budget_error(true))?;
        match trace_action {
            CurrencyWarsBaselineActivityAction::ContinueSupply => {
                self.supply_decisions = self
                    .supply_decisions
                    .checked_add(1)
                    .ok_or_else(|| settlement_budget_error(true))?;
            }
            CurrencyWarsBaselineActivityAction::ContinuePlane => {
                self.route_decisions = self
                    .route_decisions
                    .checked_add(1)
                    .ok_or_else(|| settlement_budget_error(true))?;
            }
            CurrencyWarsBaselineActivityAction::EngageEncounter
            | CurrencyWarsBaselineActivityAction::PrepareBattle => {}
        }
        self.activity_trace
            .push(CurrencyWarsBaselineActivityTraceEntry::new(
                trace_action,
                self.run.state_hash(),
                (trace_action == CurrencyWarsBaselineActivityAction::PrepareBattle).then(|| {
                    u32::try_from(self.battle_reports.len()).expect("battle bound fits u32")
                }),
            ));
        self.refresh_offer()?;
        let response = AgentActivityActionResponse {
            session_id: self.id.clone(),
            committed: true,
            idempotent_replay: false,
            accepted_action_token: request.action_token.clone(),
            settlement: AgentActivitySettlementSummary {
                accepted_activity_actions: AgentUInt::from_u64(1),
                nested_battles: AgentUInt::from_u64(nested_battles),
            },
            observation: self.observe()?,
        };
        let canonical_json = serde_json::to_vec(&response).map_err(|_| adapter_error(true))?;
        if canonical_json.len() > MAX_CACHED_RESPONSE_BYTES {
            return Err(agent_error(
                AgentErrorCode::ObservationTooLarge,
                "The committed Currency Wars response exceeds its cache limit.",
                true,
            ));
        }
        self.idempotency.insert(
            request.idempotency_key.clone(),
            CachedCurrencyWarsResponse {
                request,
                response: response.clone(),
                canonical_json: canonical_json.into_boxed_slice(),
            },
        );
        Ok(response)
    }

    pub fn close(&mut self) {
        self.closed = true;
        self.offered = None;
    }

    pub fn export_replay(&self) -> Result<AgentActivityReplayExport, AgentError> {
        let report = self.completed_report()?;
        let bytes =
            encode_currency_wars_replay(self.replay_request(), self.replay_identity, &report)
                .map_err(replay_error)?;
        Ok(AgentActivityReplayExport::new(
            bytes,
            self.accepted_actions,
            true,
        ))
    }

    pub fn verify_replay(
        &self,
        factory: &CurrencyWarsActivityAgentSessionFactory,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        factory.verify_replay(bytes)
    }

    fn completed_report(&self) -> Result<CurrencyWarsBaselineRunReport, AgentError> {
        let view = self.run.player_view();
        let terminal = view.terminal().ok_or_else(|| {
            agent_error(
                AgentErrorCode::ConfigurationRejected,
                "Currency Wars replay export requires a terminal session.",
                false,
            )
        })?;
        CurrencyWarsBaselineRunReport::new(
            terminal,
            view.state_hash(),
            self.supply_decisions,
            self.route_decisions,
            self.activity_trace.clone(),
            self.battle_reports.clone(),
        )
        .ok_or_else(|| adapter_error(false))
    }

    const fn replay_request(&self) -> CurrencyWarsReplayRequest {
        CurrencyWarsReplayRequest::new(
            self.route_id,
            self.difficulty_id,
            match self.gambit {
                AgentCurrencyWarsGambit::Standard => CurrencyWarsReplayGambit::Standard,
                AgentCurrencyWarsGambit::Overclock => CurrencyWarsReplayGambit::Overclock,
            },
            self.seed,
        )
    }

    fn validate_request(&self, request: &PlayActivityActionRequest) -> Result<(), AgentError> {
        if request.session_id != self.id {
            return Err(agent_error(
                AgentErrorCode::SessionNotOwned,
                "The Currency Wars action does not belong to this session.",
                false,
            ));
        }
        if self.closed || self.run.player_view().terminal().is_some() {
            return Err(agent_error(
                AgentErrorCode::SessionClosed,
                "The Currency Wars session has already settled or closed.",
                false,
            ));
        }
        Ok(())
    }

    fn apply_selected(&mut self, action: BoundActivityAction) -> Result<u64, AgentError> {
        match action {
            BoundActivityAction::Decision { kind, .. } => match kind {
                ActivityDecisionKind::Encounter => {
                    if self.prepared_catalog.is_some() {
                        return Err(adapter_error(false));
                    }
                    let attempt = u32::try_from(self.battle_reports.len())
                        .ok()
                        .and_then(|value| value.checked_add(1))
                        .and_then(AttemptId::new)
                        .ok_or_else(|| settlement_budget_error(false))?;
                    let preparation = self
                        .run
                        .engage_current_node(attempt, &mut self.assembler)
                        .map_err(|error| runtime_error(error, false))?;
                    self.prepared_catalog =
                        Some(Arc::clone(preparation.materialization().combat_catalog()));
                    Ok(0)
                }
                ActivityDecisionKind::Shop => self
                    .run
                    .continue_supply()
                    .map(|()| 0)
                    .map_err(|error| runtime_error(error, false)),
                ActivityDecisionKind::Route => self
                    .run
                    .continue_plane()
                    .map(|()| 0)
                    .map_err(|error| runtime_error(error, false)),
                _ => Err(agent_error(
                    AgentErrorCode::CombatRejected,
                    "The offered Currency Wars decision kind is unsupported.",
                    false,
                )),
            },
            BoundActivityAction::Preparation { .. } => {
                let catalog = Arc::clone(
                    self.prepared_catalog
                        .as_ref()
                        .ok_or_else(|| adapter_error(false))?,
                );
                self.run
                    .choose_prepared_battle()
                    .map_err(|error| runtime_error(error, false))?;
                let handoff = self
                    .run
                    .start_pending_battle()
                    .map_err(|error| runtime_error(error, true))?;
                let (result, report) = self
                    .controller
                    .execute_battle(catalog, &handoff)
                    .map_err(|error| battle_error(error, true))?;
                self.run
                    .submit_battle_result(result)
                    .map_err(|error| runtime_error(error, true))?;
                self.prepared_catalog = None;
                self.battle_reports.push(report);
                Ok(1)
            }
        }
    }

    fn refresh_offer(&mut self) -> Result<(), AgentError> {
        let view = self.run.player_view();
        if self.closed || view.terminal().is_some() {
            self.offered = None;
            return Ok(());
        }
        if view.pending_battle().is_some() {
            return Err(adapter_error(true));
        }
        self.offered =
            Some(OfferedActivityActionSet::bind(&self.id, &view).map_err(action_binding_error)?);
        Ok(())
    }
}

fn baseline_trace_action(
    action: BoundActivityAction,
) -> Result<CurrencyWarsBaselineActivityAction, AgentError> {
    let trace_action = match action {
        BoundActivityAction::Decision {
            kind: ActivityDecisionKind::Encounter,
            ..
        } => CurrencyWarsBaselineActivityAction::EngageEncounter,
        BoundActivityAction::Decision {
            kind: ActivityDecisionKind::Shop,
            ..
        } => CurrencyWarsBaselineActivityAction::ContinueSupply,
        BoundActivityAction::Decision {
            kind: ActivityDecisionKind::Route,
            ..
        } => CurrencyWarsBaselineActivityAction::ContinuePlane,
        BoundActivityAction::Preparation { .. } => {
            CurrencyWarsBaselineActivityAction::PrepareBattle
        }
        BoundActivityAction::Decision { .. } => {
            return Err(agent_error(
                AgentErrorCode::CombatRejected,
                "The offered Currency Wars decision kind is unsupported.",
                false,
            ));
        }
    };
    Ok(trace_action)
}

fn activity_identity(
    identity: &CurrencyWarsCatalogIdentity,
) -> Result<ActivityDefinitionIdentity, AgentError> {
    Ok(ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(DEFINITION_ID).ok_or_else(configuration_error)?,
        ActivityDefinitionDigest::new(identity.content_digest().bytes())
            .ok_or_else(configuration_error)?,
        ActivityConfigDigest::new(identity.configuration_digest().bytes())
            .ok_or_else(configuration_error)?,
    ))
}

fn baseline_setup(catalog: &CurrencyWarsCatalog) -> Result<CurrencyWarsRunSetup, AgentError> {
    let roles = INITIAL_ROLES.map(|raw| {
        CurrencyWarsRoleState::new(
            CurrencyWarsRoleId::new(raw).expect("static role ID is non-zero"),
            1,
        )
        .expect("released one-star role state is valid")
    });
    let roster = CurrencyWarsRoster::new(catalog, roles.into_iter().map(|role| (role, 1)))
        .map_err(|_| configuration_error())?;
    let deployment = CurrencyWarsDeployment::new(
        catalog,
        &roster,
        INITIAL_TEAM_LEVEL,
        (1_u8..=4).map(|index| {
            (
                CurrencyWarsPosition::new(CurrencyWarsPositionKind::Front, index)
                    .expect("front position is valid"),
                roles[usize::from(index - 1)],
            )
        }),
    )
    .map_err(|_| configuration_error())?;
    Ok(CurrencyWarsRunSetup {
        initial_gold: 0,
        initial_team_level: INITIAL_TEAM_LEVEL,
        initial_experience: 0,
        roster,
        deployment,
        enemy_affix_ids: Box::new([]),
        owned_builds: BTreeMap::new(),
    })
}

fn action_binding_error(error: ActivityActionBindingError) -> AgentError {
    match error {
        ActivityActionBindingError::StaleBoundary => stale_boundary(),
        ActivityActionBindingError::InvalidActionToken => agent_error(
            AgentErrorCode::InvalidActionToken,
            "The Currency Wars token is not in the current exact offer.",
            false,
        ),
        _ => adapter_error(false),
    }
}

fn runtime_error(error: CurrencyWarsRuntimeError, committed: bool) -> AgentError {
    diagnostic_error(
        AgentErrorCode::CombatRejected,
        "The Currency Wars Activity action was rejected.",
        &error.to_string(),
        committed,
    )
}

fn battle_error(error: CurrencyWarsBaselineControllerError, committed: bool) -> AgentError {
    diagnostic_error(
        AgentErrorCode::BattleFaulted,
        "The Currency Wars nested battle could not be settled.",
        &format!("{error:?}"),
        committed,
    )
}

fn replay_error(error: impl std::fmt::Debug) -> AgentError {
    diagnostic_error(
        AgentErrorCode::ReplayDiverged,
        "The Currency Wars replay is invalid or diverged from fresh execution.",
        &format!("{error:?}"),
        false,
    )
}

fn diagnostic_error(
    code: AgentErrorCode,
    message: &'static str,
    reason: &str,
    committed: bool,
) -> AgentError {
    let mut error = agent_error(code, message, committed);
    let bounded = if reason.len() <= 512 {
        reason
    } else {
        "Currency Wars runtime returned an oversized diagnostic"
    };
    error
        .insert_detail("reason", bounded)
        .expect("bounded Currency Wars diagnostic is valid");
    error
}

fn configuration_error() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "The Currency Wars Activity could not be constructed.",
        false,
    )
}

fn invalid_request() -> AgentError {
    agent_error(
        AgentErrorCode::InvalidRequest,
        "The Currency Wars route, difficulty, Gambit or seed is invalid.",
        false,
    )
}

fn stale_boundary() -> AgentError {
    agent_error(
        AgentErrorCode::StaleDecision,
        "The requested Currency Wars boundary is no longer current.",
        false,
    )
}

fn settlement_budget_error(committed: bool) -> AgentError {
    agent_error(
        AgentErrorCode::SettlementBudgetExceeded,
        "The Currency Wars session exceeded its accepted-action budget.",
        committed,
    )
}

fn adapter_error(committed: bool) -> AgentError {
    agent_error(
        AgentErrorCode::AdapterFailure,
        "The stable Currency Wars boundary could not be projected or encoded.",
        committed,
    )
}

fn agent_error(code: AgentErrorCode, message: &'static str, committed: bool) -> AgentError {
    AgentError::new(code, message, false, committed)
        .expect("static Currency Wars session error is bounded")
}

#[cfg(test)]
mod tests;
