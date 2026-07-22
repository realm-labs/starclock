//! Authoritative ephemeral session and in-memory registry contracts.
//!
//! Sessions compose deterministic Goal 01 libraries while operational identity,
//! time, ownership, expiry, quotas and idempotency remain outside domain state.

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
    observation::VisibilityPolicy,
    schema::{ActionToken, AgentHash, AgentUInt, ScenarioId, SessionId},
};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "ephemeral authoritative sessions and registry";
pub const MAX_ACCEPTED_COMMANDS_PER_SETTLEMENT: usize = 4_096;

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
    pub controllers: Box<[AcceptedCommandRecord]>,
}

/// Incremental accepted-command recorder owned by exactly one session.
#[derive(Default)]
struct AgentReplayRecorder {
    trace: Vec<BattleTraceEntry>,
    controllers: Vec<AcceptedCommandRecord>,
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
    pub fn offered_actions(&self) -> &[OfferedAction] {
        self.offered.as_ref().map_or(&[], OfferedActionSet::actions)
    }

    #[must_use]
    pub fn controller_records(&self) -> &[AcceptedCommandRecord] {
        &self.replay.controllers
    }

    pub fn play_action(&mut self, token: &ActionToken) -> Result<AgentSettlement, AgentError> {
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
            controllers: self.replay.controllers[controller_start..]
                .to_vec()
                .into_boxed_slice(),
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
        let before = session.replay_command_count();
        let settlement = session.play_action(&token).unwrap();
        assert!(settlement.accepted_commands.to_u64() >= 1);
        assert!(settlement.emitted_events.to_u64() >= 1);
        assert_eq!(
            settlement.controllers[0].controller,
            AgentControllerKind::ExternalPlayer
        );
        assert_eq!(
            session.replay_command_count() - before,
            usize::try_from(settlement.accepted_commands.to_u64()).unwrap()
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
}
