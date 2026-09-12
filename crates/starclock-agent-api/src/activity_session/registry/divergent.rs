//! Divergent Universe extension over the shared Activity registry state.

use crate::divergent_universe_activity_session::{
    AgentDivergentUniverseRunFamily, CreateDivergentUniverseActivitySessionRequest,
};

use super::*;

impl ActivityAgentSessionRegistry {
    pub fn new_with_all_modes_including_divergent_universe(
        factory: ActivityAgentSessionFactory,
        gold_factory: GoldAndGearsActivityAgentSessionFactory,
        swarm_factory: SwarmDisasterActivityAgentSessionFactory,
        currency_wars_factory: CurrencyWarsActivityAgentSessionFactory,
        divergent_universe_factory: DivergentUniverseActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(
            factory,
            ActivityModeFactories {
                gold: Some(gold_factory),
                swarm: Some(swarm_factory),
                currency_wars: Some(currency_wars_factory),
                divergent_universe: Some(divergent_universe_factory),
            },
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }

    pub fn new_with_divergent_universe(
        factory: ActivityAgentSessionFactory,
        divergent_universe_factory: DivergentUniverseActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(
            factory,
            ActivityModeFactories {
                divergent_universe: Some(divergent_universe_factory),
                ..ActivityModeFactories::default()
            },
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }

    pub fn create_divergent_universe(
        &self,
        owner: &AgentSessionOwner,
        request: RegistryCreateDivergentUniverseSessionRequest,
    ) -> Result<AgentActivityObservation, AgentError> {
        let _create = lock(&self.inner.create_lane)?;
        let now = self.read_now()?;
        self.sweep_expired(now)?;
        self.ensure_quota(owner)?;
        let session_id = self.inner.id_source.next_session_id()?;
        let factory = self
            .inner
            .divergent_universe_factory
            .as_ref()
            .ok_or_else(divergent_universe_not_configured)?;
        let session = factory.create(CreateDivergentUniverseActivitySessionRequest {
            session_id: session_id.clone(),
            family: request.family,
            seed: request.seed,
            tawot_forge_level: request.tawot_forge_level,
        })?;
        let observation = session.observe()?;
        let entry = Arc::new(SessionEntry {
            owner: owner.clone(),
            lane: Mutex::new(SessionLane {
                created_at: now,
                last_accessed_at: now,
                state: SessionLaneState::Active {
                    session: Box::new(HostedActivitySession::DivergentUniverse(session)),
                    events: ActivityEventRecorder::default(),
                },
            }),
        });
        let mut state = lock(&self.inner.state)?;
        if state.active.contains_key(&session_id) || state.terminal.contains_key(&session_id) {
            return Err(adapter_error(
                "The injected session ID source produced a duplicate identity.",
            ));
        }
        state.active.insert(session_id, entry);
        Ok(observation)
    }

    pub fn verify_divergent_universe_replay(
        &self,
        seed: &AgentUInt,
        family: AgentDivergentUniverseRunFamily,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        self.inner
            .divergent_universe_factory
            .as_ref()
            .ok_or_else(divergent_universe_not_configured)?
            .verify_replay(seed, family, bytes)
    }

    pub fn divergent_universe_manifest(
        &self,
    ) -> Result<AgentDivergentUniverseManifest, AgentError> {
        self.inner
            .divergent_universe_factory
            .as_ref()
            .ok_or_else(divergent_universe_not_configured)?
            .manifest()
    }
}

pub(super) fn divergent_universe_not_configured() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "Divergent Universe Activity sessions are not configured.",
    )
}
