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
    gold_gears_identity::{
        GOLD_AND_GEARS_COMBAT_RULE_REGISTRY_REVISION, GOLD_AND_GEARS_CONTENT_REVISION,
        GOLD_AND_GEARS_ENCOUNTER_OVERLAY_REVISION, GOLD_AND_GEARS_PROFILE_REVISION,
        GoldAndGearsCatalogIdentity, UNIVERSE_SHARED_CONTENT_REVISION,
    },
};

/// Builds the ten canonically ordered components consumed by one mode run.
///
/// Tuple arguments carry `(revision, digest)` for the two shared catalogs and
/// `(id, revision, digest)` for the caller-selected controller.
pub fn gold_and_gears_component_set(
    identity: &GoldAndGearsCatalogIdentity,
    combat_catalog: (&str, [u8; 32]),
    build_catalog: (&str, [u8; 32]),
    activity_definition_digest: [u8; 32],
    combat_rule_registry_digest: [u8; 32],
    encounter_overlay_digest: [u8; 32],
    controller: (&str, &str, [u8; 32]),
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
            combat_catalog.0,
            combat_catalog.1,
        )?,
        component(
            ConfigurationComponentKind::BuildCatalog,
            "build-catalog",
            build_catalog.0,
            build_catalog.1,
        )?,
        component(
            ConfigurationComponentKind::ActivityCore,
            "gold-and-gears-activity",
            starclock_activity::ACTIVITY_STATE_HASH_REVISION,
            activity_definition_digest,
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "gold-and-gears-profile",
            GOLD_AND_GEARS_PROFILE_REVISION,
            identity.profile_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "gold-and-gears-content",
            GOLD_AND_GEARS_CONTENT_REVISION,
            identity.bundle_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "universe-shared-content",
            UNIVERSE_SHARED_CONTENT_REVISION,
            identity.shared_content_digest(),
        )?,
        component(
            ConfigurationComponentKind::ActivityHandlerRegistry,
            "gold-and-gears-activity-handlers",
            starclock_activity::ACTIVITY_HANDLER_REGISTRY_REVISION,
            identity.activity_handler_registry_digest(),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "gold-and-gears-combat-rules",
            GOLD_AND_GEARS_COMBAT_RULE_REGISTRY_REVISION,
            combat_rule_registry_digest,
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "gold-and-gears-encounter-overlay",
            GOLD_AND_GEARS_ENCOUNTER_OVERLAY_REVISION,
            encounter_overlay_digest,
        )?,
        component(
            ConfigurationComponentKind::Controller,
            controller.0,
            controller.1,
            controller.2,
        )?,
    ])
}

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    revision: &str,
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, ComponentIdentityError> {
    ConfigurationComponentIdentity::new(kind, id, revision, ComponentDigest::new(digest))
}
