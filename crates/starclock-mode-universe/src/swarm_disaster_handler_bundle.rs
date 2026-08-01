//! Swarm Disaster contribution to the immutable Activity handler registry.

use starclock_activity::{
    ActivityHandlerBundle, ActivityHandlerRegistry, core_activity_handler_bundle,
};

pub(crate) const SWARM_DISASTER_HANDLER_BUNDLE_ID: &str = "starclock.mode.swarm-disaster";
pub(crate) const SWARM_DISASTER_HANDLER_BUNDLE_REVISION: &str =
    "swarm-disaster-activity-handlers-v1";

pub(crate) fn swarm_disaster_activity_handler_bundle() -> ActivityHandlerBundle {
    ActivityHandlerBundle::new(
        SWARM_DISASTER_HANDLER_BUNDLE_ID,
        SWARM_DISASTER_HANDLER_BUNDLE_REVISION,
        vec!["starclock.activity.core"],
        Vec::new(),
    )
    .expect("the static Swarm Disaster Activity handler bundle is valid")
}

pub(crate) fn swarm_disaster_activity_handler_registry() -> ActivityHandlerRegistry {
    ActivityHandlerRegistry::compose(vec![
        core_activity_handler_bundle(),
        swarm_disaster_activity_handler_bundle(),
    ])
    .expect("the static Swarm Disaster Activity handler registry composes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composes_only_core_and_empty_swarm_contributions() {
        let registry = swarm_disaster_activity_handler_registry();
        assert_eq!(registry.bundles().len(), 2);
        assert_eq!(registry.bundles()[0].id(), "starclock.activity.core");
        assert_eq!(registry.bundles()[1].id(), SWARM_DISASTER_HANDLER_BUNDLE_ID);
        assert_eq!(
            registry.bundles()[1].revision(),
            SWARM_DISASTER_HANDLER_BUNDLE_REVISION
        );
        assert_eq!(
            registry.bundles()[1].dependencies(),
            ["starclock.activity.core"]
        );
        assert!(registry.bundles()[1].registrations().is_empty());
    }
}
