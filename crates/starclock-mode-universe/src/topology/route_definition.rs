use super::DomainRouteDefinition;
use crate::id::TopologyNodeId;
use starclock_activity::ActivityOptionId;

impl DomainRouteDefinition {
    #[must_use]
    pub const fn option(&self) -> ActivityOptionId {
        self.option
    }

    #[must_use]
    pub const fn target(&self) -> Option<TopologyNodeId> {
        self.target
    }
}
