//! Standard Universe contribution to the composed Activity handler registry.

use starclock_activity::{
    ActivityHandlerBundle, ActivityHandlerFault, ActivityHandlerFaultKind, ActivityHandlerId,
    ActivityHandlerInput, ActivityHandlerOutput, ActivityHandlerRegistration,
    ActivityHandlerRegistry, core_activity_handler_bundle,
};

use crate::occurrence_interaction::{
    OCCURRENCE_INTERACTION_HANDLER_ID, execute as execute_occurrence_interaction,
};
use crate::service_interaction::{
    SERVICE_INTERACTION_HANDLER_ID, execute as execute_service_interaction,
};

pub const STANDARD_UNIVERSE_HANDLER_BUNDLE_ID: &str = "starclock.mode.standard-universe";
pub const STANDARD_UNIVERSE_HANDLER_BUNDLE_REVISION: &str =
    "standard-universe-activity-handlers-v3";
pub const STANDARD_UNIVERSE_EXTERNAL_INTERACTION_HANDLER_ID: u32 = 1;

fn deferred_external_interaction(
    input: ActivityHandlerInput<'_>,
) -> Result<ActivityHandlerOutput, ActivityHandlerFault> {
    if input.payload().is_empty() || !input.payload().iter().all(|byte| byte.is_ascii_graphic()) {
        return Err(ActivityHandlerFault::new(
            ActivityHandlerFaultKind::InvalidPayload,
        ));
    }
    Ok(ActivityHandlerOutput::new(Vec::new()))
}

pub fn activity_handler_bundle() -> ActivityHandlerBundle {
    ActivityHandlerBundle::new(
        STANDARD_UNIVERSE_HANDLER_BUNDLE_ID,
        STANDARD_UNIVERSE_HANDLER_BUNDLE_REVISION,
        vec!["starclock.activity.core"],
        vec![
            ActivityHandlerRegistration::new(
                ActivityHandlerId::new(STANDARD_UNIVERSE_EXTERNAL_INTERACTION_HANDLER_ID)
                    .expect("static handler ID is non-zero"),
                "standard-universe.external-interaction",
                "v1",
                [0x51; 32],
                "validated-content-id-no-rng",
                "starclock.mode.standard-universe",
                deferred_external_interaction,
            ),
            ActivityHandlerRegistration::new(
                ActivityHandlerId::new(OCCURRENCE_INTERACTION_HANDLER_ID)
                    .expect("static handler ID is non-zero"),
                "standard-universe.occurrence-choice",
                "v2",
                [0x62; 32],
                "canonical-choice-plan-labeled-activity-rng",
                "starclock.mode.standard-universe",
                execute_occurrence_interaction,
            ),
            ActivityHandlerRegistration::new(
                ActivityHandlerId::new(SERVICE_INTERACTION_HANDLER_ID)
                    .expect("static handler ID is non-zero"),
                "standard-universe.service-selection",
                "v1",
                [0x73; 32],
                "canonical-concrete-offer-no-untracked-rng",
                "starclock.mode.standard-universe",
                execute_service_interaction,
            ),
        ],
    )
    .expect("the static Standard Universe Activity handler bundle is valid")
}

pub(crate) fn activity_handler_registry() -> ActivityHandlerRegistry {
    ActivityHandlerRegistry::compose(vec![
        core_activity_handler_bundle(),
        activity_handler_bundle(),
    ])
    .expect("the static Standard Universe Activity handler registry composes")
}
