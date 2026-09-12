//! Gold and Gears constructor over the shared Activity registry.

use super::*;

impl ActivityAgentSessionRegistry {
    pub fn new_with_gold_and_gears(
        factory: ActivityAgentSessionFactory,
        gold_factory: GoldAndGearsActivityAgentSessionFactory,
        clock: Arc<dyn OperationalClock>,
        id_source: Arc<dyn SessionIdSource>,
    ) -> Self {
        Self::with_limits(
            factory,
            ActivityModeFactories {
                gold: Some(gold_factory),
                ..ActivityModeFactories::default()
            },
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }
}
