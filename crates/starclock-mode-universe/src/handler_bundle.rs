//! Standard Universe contribution to the composed Activity handler registry.

use starclock_activity::ActivityHandlerBundle;

pub const STANDARD_UNIVERSE_HANDLER_BUNDLE_ID: &str = "starclock.mode.standard-universe";
pub const STANDARD_UNIVERSE_HANDLER_BUNDLE_REVISION: &str =
    "standard-universe-activity-handlers-v1";

pub fn activity_handler_bundle() -> ActivityHandlerBundle {
    ActivityHandlerBundle::new(
        STANDARD_UNIVERSE_HANDLER_BUNDLE_ID,
        STANDARD_UNIVERSE_HANDLER_BUNDLE_REVISION,
        vec!["starclock.activity.core"],
        Vec::new(),
    )
    .expect("the static Standard Universe Activity handler bundle is valid")
}
