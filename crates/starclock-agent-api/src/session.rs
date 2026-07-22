//! Authoritative ephemeral session and in-memory registry contracts.
//!
//! Sessions compose deterministic Goal 01 libraries while operational identity,
//! time, ownership, expiry, quotas and idempotency remain outside domain state.

use serde::{Deserialize, Serialize};
use starclock_combat::{Battle, BattlePhase, BattleSpecDigest, EncounterId};
use starclock_data::standard_v1::StandardV1Catalog;
use starclock_replay::battle::BattleTraceEntry;

use crate::{
    error::{AgentError, AgentErrorCode},
    observation::VisibilityPolicy,
    schema::{AgentHash, AgentUInt, ScenarioId, SessionId},
};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "ephemeral authoritative sessions and registry";

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
        Ok(AgentSession {
            id: request.session_id,
            scenario: request.scenario_id,
            visibility: request.visibility_policy,
            encounter: instantiated.encounter(),
            spec_digest: instantiated.spec_digest(),
            master_seed: instantiated.master_seed(),
            battle: instantiated.into_battle(),
            replay: AgentReplayRecorder::default(),
        })
    }
}

/// Incremental accepted-command recorder owned by exactly one session.
#[derive(Default)]
struct AgentReplayRecorder {
    trace: Vec<BattleTraceEntry>,
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
    fn factory_creates_only_frozen_production_scenarios_with_empty_replays() {
        let factory = AgentSessionFactory::load_production().unwrap();
        let default_seeds = [104_729, 419_431, 314_159, 524_287, 209_759, 629_137];
        for ((scenario, _, encounter), expected_seed) in SCENARIOS.into_iter().zip(default_seeds) {
            let session = factory
                .create(request(scenario, AgentSeedPolicy::ScenarioDefault))
                .unwrap();
            assert_eq!(session.scenario_id().as_str(), scenario);
            assert_eq!(session.encounter().get(), encounter);
            assert_eq!(session.phase(), BattlePhase::Initializing);
            assert_eq!(session.replay_command_count(), 0);
            assert_eq!(session.master_seed().to_u64(), expected_seed);
            assert_eq!(session.visibility_policy(), VisibilityPolicy::PlayerVisible);
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
}
