//! Component-aware identity for the Gold and Gears catalog composition.

use crate::{
    digest::Encoder,
    gold_gears_catalog::{
        GoldAndGearsBundleLoadError, GoldAndGearsBundleSummary, validate_gold_and_gears_bundle,
    },
    gold_gears_content::GoldAndGearsContentCatalog,
    gold_gears_handler_bundle::{
        GOLD_AND_GEARS_HANDLER_BUNDLE_REVISION, gold_and_gears_activity_handler_registry,
    },
    gold_gears_structural::GoldAndGearsStructuralCatalog,
    gold_gears_unique::GoldAndGearsUniqueCatalog,
};

pub const GOLD_AND_GEARS_CATALOG_REVISION: &str = "gold-and-gears-v4.4-runtime-v1";
pub const GOLD_AND_GEARS_PROFILE_REVISION: &str = "gold-gears-profile-v1";
pub const GOLD_AND_GEARS_CONTENT_REVISION: &str = "gold-gears-content-v1";
pub const UNIVERSE_SHARED_CONTENT_REVISION: &str = "universe-shared-content-v1";
pub const GOLD_AND_GEARS_COMBAT_RULE_REGISTRY_REVISION: &str = "gold-and-gears-rule-registry-v1";
pub const GOLD_AND_GEARS_ENCOUNTER_OVERLAY_REVISION: &str = "gold-and-gears-encounter-overlay-v1";

const GAME_VERSION: &str = "4.4";
const SNAPSHOT_DATE: &str = "2026-07-22";
const STANDARD_UNIVERSE_SHARED_CONTENT_DIGEST: [u8; 32] = [
    0x5e, 0x52, 0x34, 0xee, 0x39, 0x77, 0xf7, 0x94, 0xae, 0x9b, 0x1b, 0x83, 0x33, 0x72, 0xf5, 0x1c,
    0x38, 0x40, 0x8c, 0x20, 0x51, 0x05, 0xc4, 0x64, 0xf1, 0x18, 0x27, 0xe9, 0xe9, 0xae, 0x6a, 0x75,
];

/// Generated-row-free identity of the frozen mode and shared catalog inputs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsCatalogIdentity {
    bundle: [u8; 32],
    shared_content: [u8; 32],
    profile: [u8; 32],
    activity_handlers: [u8; 32],
    composition: [u8; 32],
}

impl GoldAndGearsCatalogIdentity {
    /// Loads and validates the exact bundle plus the complete structural graph.
    pub fn load(bytes: &[u8]) -> Result<Self, GoldAndGearsBundleLoadError> {
        let summary = validate_gold_and_gears_bundle(bytes)?;
        let structural = GoldAndGearsStructuralCatalog::load(bytes)
            .map_err(|_| GoldAndGearsBundleLoadError::TableClosure)?;
        let unique = GoldAndGearsUniqueCatalog::load(bytes)
            .map_err(|_| GoldAndGearsBundleLoadError::TableClosure)?;
        let content = GoldAndGearsContentCatalog::load(bytes)
            .map_err(|_| GoldAndGearsBundleLoadError::TableClosure)?;
        debug_assert_eq!(structural.bundle, summary);
        debug_assert_eq!(unique.bundle, summary);
        debug_assert_eq!(content.bundle, summary);
        Ok(Self::from_validated_bundle(summary))
    }

    /// Composes identity only from a summary returned by exact bundle validation.
    #[must_use]
    pub fn from_validated_bundle(bundle: GoldAndGearsBundleSummary) -> Self {
        let activity_handlers = gold_and_gears_activity_handler_registry().digest().bytes();
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

    #[must_use]
    pub const fn game_version(&self) -> &'static str {
        GAME_VERSION
    }
    #[must_use]
    pub const fn snapshot_date(&self) -> &'static str {
        SNAPSHOT_DATE
    }
    #[must_use]
    pub const fn catalog_revision(&self) -> &'static str {
        GOLD_AND_GEARS_CATALOG_REVISION
    }
    #[must_use]
    pub const fn profile_revision(&self) -> &'static str {
        GOLD_AND_GEARS_PROFILE_REVISION
    }
    #[must_use]
    pub const fn content_revision(&self) -> &'static str {
        GOLD_AND_GEARS_CONTENT_REVISION
    }
    #[must_use]
    pub const fn shared_content_revision(&self) -> &'static str {
        UNIVERSE_SHARED_CONTENT_REVISION
    }
    #[must_use]
    pub const fn bundle_digest(&self) -> [u8; 32] {
        self.bundle
    }
    #[must_use]
    pub const fn shared_content_digest(&self) -> [u8; 32] {
        self.shared_content
    }
    #[must_use]
    pub const fn profile_digest(&self) -> [u8; 32] {
        self.profile
    }
    #[must_use]
    pub const fn activity_handler_registry_digest(&self) -> [u8; 32] {
        self.activity_handlers
    }
    #[must_use]
    pub const fn composition_digest(&self) -> [u8; 32] {
        self.composition
    }
}

fn profile_digest(bundle: [u8; 32]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.profile-identity.v1");
    encoder.text(GOLD_AND_GEARS_PROFILE_REVISION);
    encoder.digest(bundle);
    encoder.finish()
}

fn composition_digest(
    bundle: [u8; 32],
    shared_content: [u8; 32],
    profile: [u8; 32],
    activity_handlers: [u8; 32],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.catalog-composition.v1");
    for revision in [
        GOLD_AND_GEARS_CATALOG_REVISION,
        GOLD_AND_GEARS_PROFILE_REVISION,
        GOLD_AND_GEARS_CONTENT_REVISION,
        UNIVERSE_SHARED_CONTENT_REVISION,
        GOLD_AND_GEARS_HANDLER_BUNDLE_REVISION,
        GOLD_AND_GEARS_COMBAT_RULE_REGISTRY_REVISION,
        GOLD_AND_GEARS_ENCOUNTER_OVERLAY_REVISION,
    ] {
        encoder.text(revision);
    }
    encoder.digest(bundle);
    encoder.digest(shared_content);
    encoder.digest(profile);
    encoder.digest(activity_handlers);
    encoder.finish()
}
