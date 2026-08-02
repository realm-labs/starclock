//! Swarm Disaster extension over the shared Activity registry state.

use super::*;

impl ActivityAgentSessionRegistry {
    pub fn new_with_modes(
        factory: ActivityAgentSessionFactory,
        gold_factory: GoldAndGearsActivityAgentSessionFactory,
        swarm_factory: SwarmDisasterActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(
            factory,
            Some(gold_factory),
            Some(swarm_factory),
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }

    pub fn create_swarm_disaster(
        &self,
        owner: &AgentSessionOwner,
        request: RegistryCreateSwarmDisasterSessionRequest,
    ) -> Result<AgentActivityObservation, AgentError> {
        let _create = lock(&self.inner.create_lane)?;
        let now = self.read_now()?;
        self.sweep_expired(now)?;
        self.ensure_quota(owner)?;
        let session_id = self.inner.id_source.next_session_id()?;
        let factory = self
            .inner
            .swarm_factory
            .as_ref()
            .ok_or_else(swarm_not_configured)?;
        let session = factory.create(CreateSwarmDisasterActivitySessionRequest {
            session_id: session_id.clone(),
            seed: request.seed,
        })?;
        let observation = session.observe()?;
        let entry = Arc::new(SessionEntry {
            owner: owner.clone(),
            lane: Mutex::new(SessionLane {
                created_at: now,
                last_accessed_at: now,
                state: SessionLaneState::Active(Box::new(HostedActivitySession::SwarmDisaster(
                    session,
                ))),
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

    pub fn verify_swarm_disaster_replay(
        &self,
        seed: &AgentUInt,
        bytes: &[u8],
    ) -> Result<AgentActivityReplayVerification, AgentError> {
        self.inner
            .swarm_factory
            .as_ref()
            .ok_or_else(swarm_not_configured)?
            .verify_replay(seed, bytes)
    }

    pub fn swarm_disaster_manifest(&self) -> Result<AgentSwarmDisasterManifest, AgentError> {
        self.inner
            .swarm_factory
            .as_ref()
            .map(SwarmDisasterActivityAgentSessionFactory::manifest)
            .ok_or_else(swarm_not_configured)
    }
}

pub(super) fn swarm_not_configured() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "Swarm Disaster Activity sessions are not configured.",
    )
}
