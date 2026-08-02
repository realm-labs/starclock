//! Exact component-set composition for Gold and Gears runtime instances.

use starclock_replay::{
    component::{
        ComponentIdentityError, ConfigurationComponentIdentity, ConfigurationComponentKind,
        ConfigurationComponentSet,
    },
    digest::ComponentDigest,
};

use crate::{
    gold_gears_handler_bundle::gold_and_gears_activity_handler_registry,
    gold_gears_identity::GoldAndGearsCatalogIdentity,
};

/// Builds the ten canonically ordered components consumed by one mode run.
///
/// Arguments carry the shared catalog digests and caller-selected controller identity.
pub fn gold_and_gears_component_set(
    identity: &GoldAndGearsCatalogIdentity,
    combat_catalog: [u8; 32],
    build_catalog: [u8; 32],
    activity_definition_digest: [u8; 32],
    combat_rule_registry_digest: [u8; 32],
    encounter_overlay_digest: [u8; 32],
    controller: (&str, [u8; 32]),
) -> Result<ConfigurationComponentSet, ComponentIdentityError> {
    let handlers = gold_and_gears_activity_handler_registry();
    debug_assert_eq!(
        handlers.digest().bytes(),
        identity.activity_handler_registry_digest()
    );
    ConfigurationComponentSet::new(vec![
        component(
            ConfigurationComponentKind::CombatCatalog,
            "combat-catalog",
            combat_catalog,
        )?,
        component(
            ConfigurationComponentKind::BuildCatalog,
            "build-catalog",
            build_catalog,
        )?,
        component(
            ConfigurationComponentKind::ActivityCore,
            "gold-and-gears-activity",
            activity_definition_digest,
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "gold-and-gears-profile",
            identity.profile_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "gold-and-gears-content",
            identity.bundle_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "universe-shared-content",
            identity.shared_content_digest(),
        )?,
        component(
            ConfigurationComponentKind::ActivityHandlerRegistry,
            "gold-and-gears-activity-handlers",
            identity.activity_handler_registry_digest(),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "gold-and-gears-combat-rules",
            combat_rule_registry_digest,
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "gold-and-gears-encounter-overlay",
            encounter_overlay_digest,
        )?,
        component(
            ConfigurationComponentKind::Controller,
            controller.0,
            controller.1,
        )?,
    ])
}

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, ComponentIdentityError> {
    ConfigurationComponentIdentity::new(kind, id, ComponentDigest::new(digest))
}
