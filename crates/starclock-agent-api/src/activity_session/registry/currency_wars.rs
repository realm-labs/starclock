//! Currency Wars extension over the shared Activity registry state.

use crate::currency_wars_activity_session::CreateCurrencyWarsActivitySessionRequest;

use super::*;

impl ActivityAgentSessionRegistry {
    pub fn new_with_all_modes(
        factory: ActivityAgentSessionFactory,
        gold_factory: GoldAndGearsActivityAgentSessionFactory,
        swarm_factory: SwarmDisasterActivityAgentSessionFactory,
        currency_wars_factory: CurrencyWarsActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(
            factory,
            Some(gold_factory),
            Some(swarm_factory),
            Some(currency_wars_factory),
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }

    pub fn new_with_currency_wars(
        factory: ActivityAgentSessionFactory,
        currency_wars_factory: CurrencyWarsActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(
            factory,
            None,
            None,
            Some(currency_wars_factory),
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }

    pub fn create_currency_wars(
        &self,
        owner: &AgentSessionOwner,
        request: RegistryCreateCurrencyWarsSessionRequest,
    ) -> Result<AgentActivityObservation, AgentError> {
        let _create = lock(&self.inner.create_lane)?;
        let now = self.read_now()?;
        self.sweep_expired(now)?;
        self.ensure_quota(owner)?;
        let session_id = self.inner.id_source.next_session_id()?;
        let factory = self
            .inner
            .currency_wars_factory
            .as_ref()
            .ok_or_else(currency_wars_not_configured)?;
        let session = factory.create(CreateCurrencyWarsActivitySessionRequest {
            session_id: session_id.clone(),
            route_id: request.route_id,
            difficulty_id: request.difficulty_id,
            gambit: request.gambit,
            seed: request.seed,
        })?;
        let observation = session.observe()?;
        let entry = Arc::new(SessionEntry {
            owner: owner.clone(),
            lane: Mutex::new(SessionLane {
                created_at: now,
                last_accessed_at: now,
                state: SessionLaneState::Active {
                    session: Box::new(HostedActivitySession::CurrencyWars(session)),
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

    pub fn currency_wars_manifest(&self) -> Result<AgentCurrencyWarsManifest, AgentError> {
        self.inner
            .currency_wars_factory
            .as_ref()
            .map(CurrencyWarsActivityAgentSessionFactory::manifest)
            .ok_or_else(currency_wars_not_configured)
    }
}

pub(super) fn currency_wars_not_configured() -> AgentError {
    agent_error(
        AgentErrorCode::ConfigurationRejected,
        "Currency Wars Activity sessions are not configured.",
    )
}
