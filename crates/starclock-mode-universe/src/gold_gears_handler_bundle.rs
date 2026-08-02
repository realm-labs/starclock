//! Gold and Gears contribution to the immutable Activity handler registry.

use starclock_activity::{
    ActivityHandlerBundle, ActivityHandlerRegistry, core_activity_handler_bundle,
};

pub const GOLD_AND_GEARS_HANDLER_BUNDLE_ID: &str = "starclock.mode.gold-and-gears";

/// Returns the static mode bundle.
///
/// P1 establishes registry identity before executable handler partitions are
/// admitted. Later handlers are added here without changing composition order.
#[must_use]
pub fn gold_and_gears_activity_handler_bundle() -> ActivityHandlerBundle {
    ActivityHandlerBundle::new(
        GOLD_AND_GEARS_HANDLER_BUNDLE_ID,
        vec!["starclock.activity.core"],
        Vec::new(),
    )
    .expect("the static Gold and Gears Activity handler bundle is valid")
}

pub(crate) fn gold_and_gears_activity_handler_registry() -> ActivityHandlerRegistry {
    ActivityHandlerRegistry::compose(vec![
        core_activity_handler_bundle(),
        gold_and_gears_activity_handler_bundle(),
    ])
    .expect("the static Gold and Gears Activity handler registry composes")
}
