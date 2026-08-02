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
    swarm_disaster_identity::SwarmDisasterCatalogIdentity,
};

/// Builds the ten canonically ordered components consumed by one Swarm Disaster run.
///
/// Tuple arguments carry the shared catalog digest and
/// `(id, digest)` for the caller-selected controller. Candidate and
/// generated types remain behind the bundle byte boundary.
pub fn swarm_disaster_component_set(
    bundle: &[u8],
    combat_catalog: [u8; 32],
    build_catalog: [u8; 32],
    activity_definition_digest: [u8; 32],
    combat_rule_registry_digest: [u8; 32],
    encounter_overlay_digest: [u8; 32],
    controller: (&str, [u8; 32]),
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
            combat_catalog,
        )?,
        component(
            ConfigurationComponentKind::BuildCatalog,
            "build-catalog",
            build_catalog,
        )?,
        component(
            ConfigurationComponentKind::ActivityCore,
            "swarm-disaster-activity",
            activity_definition_digest,
        )?,
        component(
            ConfigurationComponentKind::ModeProfile,
            "swarm-disaster-profile",
            identity.profile_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "swarm-disaster-content",
            identity.bundle_digest(),
        )?,
        component(
            ConfigurationComponentKind::ModeContent,
            "universe-shared-content",
            identity.shared_content_digest(),
        )?,
        component(
            ConfigurationComponentKind::ActivityHandlerRegistry,
            "swarm-disaster-activity-handlers",
            identity.activity_handler_registry_digest(),
        )?,
        component(
            ConfigurationComponentKind::CombatRuleRegistry,
            "swarm-disaster-combat-rules",
            combat_rule_registry_digest,
        )?,
        component(
            ConfigurationComponentKind::EncounterOverlay,
            "swarm-disaster-encounter-overlay",
            encounter_overlay_digest,
        )?,
        component(
            ConfigurationComponentKind::Controller,
            controller.0,
            controller.1,
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
    digest: [u8; 32],
) -> Result<ConfigurationComponentIdentity, UniverseCatalogLoadError> {
    ConfigurationComponentIdentity::new(kind, id, ComponentDigest::new(digest)).map_err(|error| {
        UniverseCatalogLoadError::new(
            UniverseCatalogLoadErrorKind::InvalidDefinition,
            error.to_string(),
        )
    })
}
