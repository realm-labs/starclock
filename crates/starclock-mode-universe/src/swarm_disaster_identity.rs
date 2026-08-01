//! Private component-aware identity for Swarm Disaster catalog composition.

use crate::{
    digest::Encoder,
    error::UniverseCatalogLoadError,
    swarm_disaster_catalog::SwarmDisasterBundleSummary,
    swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_handler_bundle::{
        SWARM_DISASTER_HANDLER_BUNDLE_REVISION, swarm_disaster_activity_handler_registry,
    },
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

pub(crate) const SWARM_DISASTER_CATALOG_REVISION: &str = "swarm-disaster-v4.4-runtime-v1";
pub(crate) const SWARM_DISASTER_PROFILE_REVISION: &str = "swarm-disaster-profile-v1";
pub(crate) const SWARM_DISASTER_CONTENT_REVISION: &str = "swarm-disaster-content-v1";
pub(crate) const UNIVERSE_SHARED_CONTENT_REVISION: &str = "universe-shared-content-v1";
pub(crate) const SWARM_DISASTER_COMBAT_RULE_REGISTRY_REVISION: &str =
    "swarm-disaster-rule-registry-v1";
pub(crate) const SWARM_DISASTER_ENCOUNTER_OVERLAY_REVISION: &str =
    "swarm-disaster-encounter-overlay-v1";

const STANDARD_UNIVERSE_SHARED_CONTENT_DIGEST: [u8; 32] = [
    0x5e, 0x52, 0x34, 0xee, 0x39, 0x77, 0xf7, 0x94, 0xae, 0x9b, 0x1b, 0x83, 0x33, 0x72, 0xf5, 0x1c,
    0x38, 0x40, 0x8c, 0x20, 0x51, 0x05, 0xc4, 0x64, 0xf1, 0x18, 0x27, 0xe9, 0xe9, 0xae, 0x6a, 0x75,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SwarmDisasterCatalogIdentity {
    bundle: [u8; 32],
    shared_content: [u8; 32],
    profile: [u8; 32],
    activity_handlers: [u8; 32],
    composition: [u8; 32],
}

impl SwarmDisasterCatalogIdentity {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, UniverseCatalogLoadError> {
        let structural = SwarmDisasterStructuralCatalog::load(bytes)?;
        let unique = SwarmDisasterUniqueCatalog::load(bytes)?;
        let content = SwarmDisasterContentCatalog::load(bytes, &structural, &unique)?;
        debug_assert_eq!(structural.bundle_summary(), unique.bundle_summary());
        debug_assert_eq!(structural.bundle_summary(), content.bundle_summary());
        Ok(Self::from_validated_bundle(structural.bundle_summary()))
    }

    fn from_validated_bundle(bundle: SwarmDisasterBundleSummary) -> Self {
        let activity_handlers = swarm_disaster_activity_handler_registry().digest().bytes();
        let profile = profile_digest(bundle.bundle_digest());
        let composition = composition_digest(
            bundle.bundle_digest(),
            STANDARD_UNIVERSE_SHARED_CONTENT_DIGEST,
            profile,
            activity_handlers,
        );
        Self {
            bundle: bundle.bundle_digest(),
            shared_content: STANDARD_UNIVERSE_SHARED_CONTENT_DIGEST,
            profile,
            activity_handlers,
            composition,
        }
    }

    pub(crate) const fn bundle_digest(&self) -> [u8; 32] {
        self.bundle
    }

    pub(crate) const fn shared_content_digest(&self) -> [u8; 32] {
        self.shared_content
    }

    pub(crate) const fn profile_digest(&self) -> [u8; 32] {
        self.profile
    }

    pub(crate) const fn activity_handler_registry_digest(&self) -> [u8; 32] {
        self.activity_handlers
    }

    pub(crate) const fn composition_digest(&self) -> [u8; 32] {
        self.composition
    }
}

fn profile_digest(bundle: [u8; 32]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.profile-identity.v1");
    encoder.text(SWARM_DISASTER_PROFILE_REVISION);
    encoder.digest(bundle);
    encoder.finish()
}

fn composition_digest(
    bundle: [u8; 32],
    shared_content: [u8; 32],
    profile: [u8; 32],
    activity_handlers: [u8; 32],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.catalog-composition.v1");
    for revision in [
        SWARM_DISASTER_CATALOG_REVISION,
        SWARM_DISASTER_PROFILE_REVISION,
        SWARM_DISASTER_CONTENT_REVISION,
        UNIVERSE_SHARED_CONTENT_REVISION,
        SWARM_DISASTER_HANDLER_BUNDLE_REVISION,
        SWARM_DISASTER_COMBAT_RULE_REGISTRY_REVISION,
        SWARM_DISASTER_ENCOUNTER_OVERLAY_REVISION,
    ] {
        encoder.text(revision);
    }
    encoder.digest(bundle);
    encoder.digest(shared_content);
    encoder.digest(profile);
    encoder.digest(activity_handlers);
    encoder.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &[u8] = include_bytes!("../../../config/swarm-disaster-generated/config.sora");

    #[test]
    fn separates_bundle_shared_profile_registry_and_composition_domains() {
        let identity = SwarmDisasterCatalogIdentity::load(BUNDLE).unwrap();
        let digests = [
            identity.bundle_digest(),
            identity.shared_content_digest(),
            identity.profile_digest(),
            identity.activity_handler_registry_digest(),
            identity.composition_digest(),
        ];
        for (index, digest) in digests.iter().enumerate() {
            assert!(!digests[..index].contains(digest));
        }
        assert_eq!(
            identity.composition_digest(),
            [
                0x8f, 0x29, 0x97, 0x49, 0xc3, 0xd7, 0x23, 0xe1, 0x73, 0x89, 0x96, 0xeb, 0xfc, 0x11,
                0xc5, 0x72, 0x9c, 0xe2, 0xac, 0xd5, 0xf4, 0xe4, 0xd8, 0xe9, 0xa8, 0x1b, 0x71, 0xc8,
                0xe3, 0x00, 0x76, 0x18,
            ]
        );
    }
}
