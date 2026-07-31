use starclock_activity::{ActivityHandlerRegistry, core_activity_handler_bundle};
use starclock_mode_universe::handler_bundle::{
    STANDARD_UNIVERSE_HANDLER_BUNDLE_ID, activity_handler_bundle,
};

#[test]
fn standard_universe_bundle_composes_without_central_registry_edits() {
    let registry = ActivityHandlerRegistry::compose(vec![
        activity_handler_bundle(),
        core_activity_handler_bundle(),
    ])
    .unwrap();
    assert_eq!(registry.bundles().len(), 2);
    assert_eq!(
        registry.bundles()[1].id(),
        STANDARD_UNIVERSE_HANDLER_BUNDLE_ID
    );
    assert_ne!(registry.digest().bytes(), [0; 32]);
}
