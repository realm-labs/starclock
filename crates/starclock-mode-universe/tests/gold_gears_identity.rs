use starclock_activity::{ActivityHandlerRegistry, core_activity_handler_bundle};
use starclock_mode_universe::{
    gold_gears_components::gold_and_gears_component_set,
    gold_gears_handler_bundle::{
        GOLD_AND_GEARS_HANDLER_BUNDLE_ID, gold_and_gears_activity_handler_bundle,
    },
    gold_gears_identity::GoldAndGearsCatalogIdentity,
    handler_bundle::{STANDARD_UNIVERSE_HANDLER_BUNDLE_ID, activity_handler_bundle},
};
use starclock_replay::component::ConfigurationComponentKind;

const BUNDLE: &[u8] = include_bytes!("../../../config/gold-and-gears-generated/config.sora");

fn identity() -> GoldAndGearsCatalogIdentity {
    GoldAndGearsCatalogIdentity::load(BUNDLE).unwrap()
}

#[test]
fn mode_registry_composes_only_core_and_gold_contributions() {
    let gold = ActivityHandlerRegistry::compose(vec![
        gold_and_gears_activity_handler_bundle(),
        core_activity_handler_bundle(),
    ])
    .unwrap();
    let standard = ActivityHandlerRegistry::compose(vec![
        activity_handler_bundle(),
        core_activity_handler_bundle(),
    ])
    .unwrap();
    assert_eq!(gold.bundles().len(), 2);
    assert_eq!(gold.bundles()[1].id(), GOLD_AND_GEARS_HANDLER_BUNDLE_ID);
    assert_eq!(gold.bundles()[1].registrations().len(), 0);
    assert_eq!(
        standard.bundles()[1].id(),
        STANDARD_UNIVERSE_HANDLER_BUNDLE_ID
    );
    assert_ne!(gold.digest(), standard.digest());
    assert_eq!(
        gold.digest().bytes(),
        identity().activity_handler_registry_digest()
    );
}

#[test]
fn catalog_identity_separates_gold_and_shared_content() {
    let identity = identity();
    assert_eq!(identity.game_version(), "4.4");
    assert_eq!(identity.snapshot_date(), "2026-07-22");
    assert_eq!(
        identity.catalog_revision(),
        "gold-and-gears-v4.4-runtime-v1"
    );
    assert_eq!(identity.profile_revision(), "gold-gears-profile-v1");
    assert_eq!(identity.content_revision(), "gold-gears-content-v1");
    assert_eq!(
        identity.shared_content_revision(),
        "universe-shared-content-v1"
    );
    assert_ne!(identity.bundle_digest(), identity.shared_content_digest());
    assert_eq!(
        identity.profile_digest(),
        [
            0xb9, 0xda, 0x9f, 0x15, 0x9d, 0x3b, 0x80, 0xfe, 0x47, 0xea, 0x2b, 0xfe, 0x70, 0x2c,
            0x66, 0xc8, 0x8d, 0xc9, 0xba, 0x2e, 0x53, 0xbb, 0x44, 0xb2, 0x6b, 0x90, 0x38, 0xc7,
            0x0d, 0xf9, 0x90, 0x3b,
        ]
    );
    assert_eq!(
        identity.activity_handler_registry_digest(),
        [
            0x83, 0xc5, 0x42, 0xb5, 0x14, 0x2a, 0x9b, 0x62, 0x7d, 0xcc, 0x5a, 0xdd, 0x7b, 0xc3,
            0x82, 0xcb, 0x51, 0xf7, 0x61, 0x85, 0xda, 0x73, 0x6a, 0x11, 0x04, 0xf1, 0xf9, 0x41,
            0x9f, 0x93, 0x20, 0x5d,
        ]
    );
    assert_eq!(
        identity.composition_digest(),
        [
            0x8e, 0x53, 0xd5, 0xef, 0x10, 0x3f, 0x55, 0x4b, 0x96, 0x02, 0xde, 0x63, 0x4c, 0xdc,
            0xf2, 0xfc, 0x3c, 0xcf, 0x49, 0x37, 0x00, 0x00, 0x1e, 0x2b, 0xb8, 0x77, 0x56, 0x7d,
            0xec, 0xd3, 0x13, 0x40,
        ]
    );
}

#[test]
fn component_set_has_exact_ten_component_closure_and_stable_order() {
    let identity = identity();
    let components = gold_and_gears_component_set(
        &identity,
        ("combat-v1", [0x11; 32]),
        ("build-v1", [0x22; 32]),
        [0x33; 32],
        [0x44; 32],
        [0x55; 32],
        ("baseline-controller", "baseline-v1", [0x66; 32]),
    )
    .unwrap();
    assert_eq!(components.components().len(), 10);
    assert_eq!(
        components
            .components()
            .iter()
            .map(|component| (component.kind(), component.id()))
            .collect::<Vec<_>>(),
        [
            (ConfigurationComponentKind::CombatCatalog, "combat-catalog"),
            (ConfigurationComponentKind::BuildCatalog, "build-catalog"),
            (
                ConfigurationComponentKind::ActivityCore,
                "gold-and-gears-activity"
            ),
            (
                ConfigurationComponentKind::ModeProfile,
                "gold-and-gears-profile"
            ),
            (
                ConfigurationComponentKind::ModeContent,
                "gold-and-gears-content"
            ),
            (
                ConfigurationComponentKind::ModeContent,
                "universe-shared-content"
            ),
            (
                ConfigurationComponentKind::ActivityHandlerRegistry,
                "gold-and-gears-activity-handlers"
            ),
            (
                ConfigurationComponentKind::CombatRuleRegistry,
                "gold-and-gears-combat-rules"
            ),
            (
                ConfigurationComponentKind::EncounterOverlay,
                "gold-and-gears-encounter-overlay"
            ),
            (
                ConfigurationComponentKind::Controller,
                "baseline-controller"
            ),
        ]
    );
    assert_eq!(
        components.root().bytes(),
        [
            0x93, 0xc5, 0x0f, 0x43, 0x0c, 0xf8, 0x95, 0x0b, 0xb4, 0x0f, 0xc1, 0x80, 0xd3, 0x55,
            0xad, 0xbc, 0x67, 0x19, 0xd8, 0xf5, 0x6a, 0xe4, 0x34, 0xda, 0x4f, 0x8a, 0xac, 0x30,
            0x68, 0x50, 0x9b, 0x18,
        ]
    );

    let changed_controller = gold_and_gears_component_set(
        &identity,
        ("combat-v1", [0x11; 32]),
        ("build-v1", [0x22; 32]),
        [0x33; 32],
        [0x44; 32],
        [0x55; 32],
        ("baseline-controller", "baseline-v1", [0x67; 32]),
    )
    .unwrap();
    assert_ne!(components.root(), changed_controller.root());
}
