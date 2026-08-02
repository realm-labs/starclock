//! Private component-aware identity for Swarm Disaster catalog composition.

use crate::{
    digest::Encoder, error::UniverseCatalogLoadError,
    swarm_disaster_catalog::SwarmDisasterBundleSummary,
    swarm_disaster_content::SwarmDisasterContentCatalog,
    swarm_disaster_handler_bundle::swarm_disaster_activity_handler_registry,
    swarm_disaster_structural::SwarmDisasterStructuralCatalog,
    swarm_disaster_unique::SwarmDisasterUniqueCatalog,
};

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
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.profile-identity");
    encoder.digest(bundle);
    encoder.finish()
}

fn composition_digest(
    bundle: [u8; 32],
    shared_content: [u8; 32],
    profile: [u8; 32],
    activity_handlers: [u8; 32],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.catalog-composition");
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
                114, 175, 143, 1, 20, 219, 162, 250, 14, 252, 123, 200, 150, 94, 222, 210, 182,
                149, 50, 229, 106, 230, 64, 117, 33, 169, 253, 105, 220, 209, 202, 35,
            ]
        );
    }
}
