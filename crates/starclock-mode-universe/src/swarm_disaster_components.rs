//! Exact component-set composition for Swarm Disaster runtime instances.

use starclock_replay::{
    component::{
        ConfigurationComponentIdentity, ConfigurationComponentKind, ConfigurationComponentSet,
    },
    digest::ComponentDigest,
};

use crate::{
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_handler_bundle::swarm_disaster_activity_handler_registry,
    swarm_disaster_identity::{
        SWARM_DISASTER_COMBAT_RULE_REGISTRY_REVISION, SWARM_DISASTER_CONTENT_REVISION,
        SWARM_DISASTER_ENCOUNTER_OVERLAY_REVISION, SWARM_DISASTER_PROFILE_REVISION,
        SwarmDisasterCatalogIdentity, UNIVERSE_SHARED_CONTENT_REVISION,
    },
};

/// Builds the ten canonically ordered components consumed by one Swarm Disaster run.
///
/// Tuple arguments carry `(revision, digest)` for shared catalogs and
/// `(id, revision, digest)` for the caller-selected controller. Candidate and
/// generated types remain behind the bundle byte boundary.
pub fn swarm_disaster_component_set(
    bundle: &[u8],
    combat_catalog: (&str, [u8; 32]),
    build_catalog: (&str, [u8; 32]),
    activity_definition_digest: [u8; 32],
    combat_rule_registry_digest: [u8; 32],
    encounter_overlay_digest: [u8; 32],
    controller: (&str, &str, [u8; 32]),
) -> Result<ConfigurationComponentSet, UniverseCatalogLoadError> {
    let identity = SwarmDisasterCatalogIdentity::load(bundle)?;
    let handlers = swarm_disaster_activity_handler_registry();
    debug_assert_eq!(
        handlers.digest().bytes(),
        identity.activity_handler_registry_digest()
    );
    debug_assert_ne!(identity.composition_digest(), identity.bundle_digest());
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
            "swarm-disaster-activity",
            starclock_activity::ACTIVITY_STATE_HASH_REVISION,
            activity_definition_digest,
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "swarm-disaster-profile",
            SWARM_DISASTER_PROFILE_REVISION,
            identity.profile_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "swarm-disaster-content",
            SWARM_DISASTER_CONTENT_REVISION,
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
            "swarm-disaster-activity-handlers",
            starclock_activity::ACTIVITY_HANDLER_REGISTRY_REVISION,
            identity.activity_handler_registry_digest(),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "swarm-disaster-combat-rules",
            SWARM_DISASTER_COMBAT_RULE_REGISTRY_REVISION,
            combat_rule_registry_digest,
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "swarm-disaster-encounter-overlay",
            SWARM_DISASTER_ENCOUNTER_OVERLAY_REVISION,
            encounter_overlay_digest,
        )?,
        component(
            ConfigurationComponentKind::Controller,
            controller.0,
            controller.1,
            controller.2,
        )?,
    ])
    .map_err(|error| {
        UniverseCatalogLoadError::new(
            UniverseCatalogLoadErrorKind::InvalidDefinition,
            error.to_string(),
        )
    })
}

fn component(
    kind: ConfigurationComponentKind,
    id: &str,
    revision: &str,
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, UniverseCatalogLoadError> {
    ConfigurationComponentIdentity::new(kind, id, revision, ComponentDigest::new(digest)).map_err(
        |error| {
            UniverseCatalogLoadError::new(
                UniverseCatalogLoadErrorKind::InvalidDefinition,
                error.to_string(),
            )
        },
    )
}
