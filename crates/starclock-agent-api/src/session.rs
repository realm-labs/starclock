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
    TeamSide, catalog::encounter::AiTransitionTiming, rng::types::RngSeed,
    rule::model::ConditionExpr,
};
use starclock_data::standard_v1::StandardV1Catalog;
use starclock_replay::battle::BattleTraceEntry;

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

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "ephemeral authoritative sessions and registry";
pub const MAX_ACCEPTED_COMMANDS_PER_SETTLEMENT: usize = 4_096;
pub const MAX_IDEMPOTENCY_ENTRIES: usize = 1_024;
pub const MAX_CACHED_RESPONSE_BYTES: usize = 512 * 1_024;
pub const MAX_RETAINED_EVENT_SUMMARIES: usize = 8_192;

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
        if request.visibility_policy != VisibilityPolicy::PlayerVisible {
            return Err(agent_error(
                AgentErrorCode::UnauthorizedPolicy,
                "Debug visibility requires a separately authorized creation path.",
            ));
        }
        let seed_override = match &request.seed {
            AgentSeedPolicy::ScenarioDefault => None,
            AgentSeedPolicy::Explicit(value) => Some(value.to_u64()),
        };
        let instantiated = self
            .standard
            .instantiate(request.scenario_id.as_str(), seed_override)
            .map_err(|_| {
                agent_error(
                    AgentErrorCode::ConfigurationRejected,
                    "The requested frozen Standard scenario is unknown or incompatible.",
                )
            })?;
        let master_seed = instantiated.master_seed();
        let enemy_seed = controller_seed(request.scenario_id.as_str(), master_seed);
        let mut session = AgentSession {
            id: request.session_id,
            scenario: request.scenario_id,
            visibility: request.visibility_policy,
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
mod tests {
    use super::*;
    use crate::observation::VisibilityPolicy;
    use starclock_data::standard_v1::SCENARIOS;

    fn request(scenario: &str, seed: AgentSeedPolicy) -> CreateSessionRequest {
        CreateSessionRequest {
            session_id: SessionId::parse("session_test").unwrap(),
            scenario_id: ScenarioId::parse(scenario).unwrap(),
            seed,
            visibility_policy: VisibilityPolicy::PlayerVisible,
        }
    }

    fn play_request(session: &AgentSession, token: ActionToken, key: &str) -> PlayActionRequest {
        PlayActionRequest {
            schema_revision: AgentSchemaRevision::V1,
            session_id: session.session_id().clone(),
            decision_id: session.offered.as_ref().unwrap().decision_id(),
            expected_state_hash: session.state_hash(),
            action_token: token,
            idempotency_key: IdempotencyKey::parse(key).unwrap(),
        }
    }

    #[test]
    fn factory_creates_only_frozen_scenarios_and_settles_internal_start() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let default_seeds = [104_729, 419_431, 314_159, 524_287, 209_759, 629_137];
        for ((scenario, _, encounter), expected_seed) in SCENARIOS.into_iter().zip(default_seeds) {
            let session = factory
                .create(request(scenario, AgentSeedPolicy::ScenarioDefault))
                .unwrap();
            assert_eq!(session.scenario_id().as_str(), scenario);
            assert_eq!(session.encounter().get(), encounter);
            assert_eq!(session.phase(), BattlePhase::AwaitingCommand);
            assert_eq!(session.replay_command_count(), 1);
            assert_eq!(session.master_seed().to_u64(), expected_seed);
            assert_eq!(session.visibility_policy(), VisibilityPolicy::PlayerVisible);
            assert!(!session.offered_actions().is_empty());
            assert_eq!(
                session.controller_records()[0].controller,
                AgentControllerKind::SystemAutomatic
            );
        }
    }

    #[test]
    fn explicit_seed_is_exact_reproducible_and_operational_identity_is_inert() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let scenario = SCENARIOS[0].0;
        let mut first_request =
            request(scenario, AgentSeedPolicy::Explicit(AgentUInt::from_u64(7)));
        first_request.session_id = SessionId::parse("session_first").unwrap();
        let mut second_request = first_request.clone();
        second_request.session_id = SessionId::parse("session_second").unwrap();
        let first = factory.create(first_request).unwrap();
        let second = factory.create(second_request).unwrap();
        assert_eq!(first.master_seed().as_str(), "7");
        assert_eq!(first.state_hash(), second.state_hash());
        assert_eq!(first.spec_digest(), second.spec_digest());
    }

    #[test]
    fn unknown_scenario_and_unauthorized_debug_fail_before_session_creation() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let unknown = factory
            .create(request(
                "scenario.standard-v1.not-authored",
                AgentSeedPolicy::ScenarioDefault,
            ))
            .err()
            .expect("unknown scenario must be rejected");
        assert_eq!(unknown.code, AgentErrorCode::ConfigurationRejected);

        let mut debug = request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault);
        debug.visibility_policy = VisibilityPolicy::OmniscientDebug;
        let unauthorized = factory
            .create(debug)
            .err()
            .expect("debug creation must require separate authorization");
        assert_eq!(unauthorized.code, AgentErrorCode::UnauthorizedPolicy);
    }

    #[test]
    fn external_action_settles_and_records_every_controller_boundary() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let mut session = factory
            .create(request(SCENARIOS[4].0, AgentSeedPolicy::ScenarioDefault))
            .unwrap();
        let token = session
            .offered_actions()
            .iter()
            .find(|action| action.kind != crate::action::AgentActionKind::Concede)
            .unwrap()
            .token
            .clone();
        let request = play_request(&session, token, "external_action_1");
        let before = session.replay_command_count();
        let controller_before = session.controller_records().len();
        let response = session.apply_action(request).unwrap();
        assert!(response.settlement.accepted_commands.to_u64() >= 1);
        assert!(response.settlement.emitted_events.to_u64() >= 1);
        assert_eq!(
            session.controller_records()[controller_before].controller,
            AgentControllerKind::ExternalPlayer
        );
        assert_eq!(
            session.replay_command_count() - before,
            usize::try_from(response.settlement.accepted_commands.to_u64()).unwrap()
        );
        assert!(
            session
                .controller_records()
                .iter()
                .enumerate()
                .all(|(index, record)| {
                    record.sequence.to_u64() == u64::try_from(index + 1).unwrap()
                })
        );
        assert!(session.phase().is_terminal() || !session.offered_actions().is_empty());
    }

    #[test]
    fn response_loss_retry_returns_identical_bytes_without_a_second_commit() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let mut session = factory
            .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
            .unwrap();
        let token = session
            .offered_actions()
            .iter()
            .find(|action| action.kind != crate::action::AgentActionKind::Concede)
            .unwrap()
            .token
            .clone();
        let request = play_request(&session, token, "lost_response_1");
        let first = session.apply_action(request.clone()).unwrap();
        let first_bytes = serde_json::to_vec(&first).unwrap();
        let committed_snapshot = (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
            session.controller_records().len(),
        );

        let retry = session.apply_action(request).unwrap();
        assert_eq!(serde_json::to_vec(&retry).unwrap(), first_bytes);
        assert_eq!(
            (
                session.state_hash(),
                session.replay_command_count(),
                session.rng_draw_count(),
                session.controller_records().len(),
            ),
            committed_snapshot
        );
        assert_eq!(session.idempotency.len(), 1);
    }

    #[test]
    fn stale_forged_conflicting_and_racing_equivalent_requests_are_inert() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let mut session = factory
            .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
            .unwrap();
        let token = session.offered_actions()[0].token.clone();
        let mut stale_hash = play_request(&session, token.clone(), "stale_hash");
        stale_hash.expected_state_hash = AgentHash::from_bytes([0x77; 32]);
        let before = (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
        );
        assert_eq!(
            session.apply_action(stale_hash).unwrap_err().code,
            AgentErrorCode::StaleStateHash
        );

        let mut forged = play_request(&session, token.clone(), "forged");
        forged.action_token = ActionToken::parse("a_forged").unwrap();
        assert_eq!(
            session.apply_action(forged).unwrap_err().code,
            AgentErrorCode::InvalidActionToken
        );
        assert_eq!(
            (
                session.state_hash(),
                session.replay_command_count(),
                session.rng_draw_count(),
            ),
            before
        );

        let committed_request = play_request(&session, token, "one_commit");
        let mut racing_request = committed_request.clone();
        racing_request.idempotency_key = IdempotencyKey::parse("racing_loser").unwrap();
        session.apply_action(committed_request.clone()).unwrap();
        let after_commit = (
            session.state_hash(),
            session.replay_command_count(),
            session.rng_draw_count(),
        );
        let mut conflict = committed_request;
        conflict.action_token = ActionToken::parse("a_conflict").unwrap();
        assert_eq!(
            session.apply_action(conflict).unwrap_err().code,
            AgentErrorCode::IdempotencyConflict
        );
        assert_eq!(
            session.apply_action(racing_request).unwrap_err().code,
            AgentErrorCode::StaleDecision
        );
        assert_eq!(
            (
                session.state_hash(),
                session.replay_command_count(),
                session.rng_draw_count(),
            ),
            after_commit
        );
    }

    #[test]
    fn retained_events_page_exclusively_and_reject_expired_or_future_cursors() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let mut session = factory
            .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
            .unwrap();
        session.replay.events.clear();
        for id in 1..=u64::try_from(MAX_RETAINED_EVENT_SUMMARIES + 1).unwrap() {
            session.replay.retain_event(AgentEventSummary {
                event_id: AgentUInt::from_u64(id),
                kind: "test".into(),
                summary: "Retained test event.".into(),
                root_command_id: AgentUInt::from_u64(1),
            });
        }
        let expired = EventCursor::parse("event_0").unwrap();
        assert_eq!(
            session.observe(&expired).unwrap_err().code,
            AgentErrorCode::EventCursorExpired
        );
        let first_retained = session
            .observe(&EventCursor::parse("event_1").unwrap())
            .unwrap();
        assert_eq!(
            first_retained.events.len(),
            crate::observation::MAX_EVENTS_PER_PAGE
        );
        assert_eq!(first_retained.events[0].event_id.as_str(), "2");
        assert!(first_retained.events_truncated);
        assert_eq!(first_retained.event_cursor.as_str(), "event_257");
        assert_eq!(
            session.replay.trace.len(),
            1,
            "summary eviction cannot erase replay facts"
        );
        assert_eq!(
            session
                .observe(&EventCursor::parse("event_999999").unwrap())
                .unwrap_err()
                .code,
            AgentErrorCode::InvalidRequest
        );
        assert_eq!(
            session
                .observe(&EventCursor::parse("cursor_wrong_family").unwrap())
                .unwrap_err()
                .code,
            AgentErrorCode::InvalidRequest
        );
    }

    #[test]
    fn terminal_action_returns_terminal_observation_and_complete_trace() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let mut session = factory
            .create(request(SCENARIOS[0].0, AgentSeedPolicy::ScenarioDefault))
            .unwrap();
        let mut preparation = 0;
        while !session
            .offered_actions()
            .iter()
            .any(|action| action.kind == crate::action::AgentActionKind::Concede)
        {
            let token = session.offered_actions()[0].token.clone();
            let request = play_request(&session, token, &format!("prepare_{preparation}"));
            session.apply_action(request).unwrap();
            preparation += 1;
            assert!(preparation < 8);
        }
        let concede = session
            .offered_actions()
            .iter()
            .find(|action| action.kind == crate::action::AgentActionKind::Concede)
            .unwrap()
            .token
            .clone();
        let trace_before = session.replay.trace.len();
        let request = play_request(&session, concede, "terminal_concede");
        let response = session.apply_action(request).unwrap();
        assert_eq!(response.observation.status, AgentBattleStatus::Lost);
        assert_eq!(response.observation.decision_id, None);
        assert!(response.observation.legal_actions.is_empty());
        assert!(!response.observation.events.is_empty());
        assert_eq!(
            response.observation.event_cursor.as_str(),
            session.replay.latest_cursor().as_str()
        );
        assert_eq!(session.replay.trace.len(), trace_before + 1);
        assert_eq!(session.replay.controllers.len(), session.replay.trace.len());
    }
}
