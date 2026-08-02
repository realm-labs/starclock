//! Gold and Gears compatibility constructor over the shared Activity registry.

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
            Some(gold_factory),
            None,
            clock,
            id_source,
            FROZEN_LIMITS,
        )
    }
}
