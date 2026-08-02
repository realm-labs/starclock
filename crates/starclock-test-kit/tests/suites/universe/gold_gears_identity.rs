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

const BUNDLE: &[u8] = include_bytes!("../../../../../config/gold-and-gears-generated/config.sora");

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
    assert_ne!(identity.bundle_digest(), identity.shared_content_digest());
    assert_eq!(
        identity.profile_digest(),
        [
            238, 19, 209, 46, 114, 118, 173, 4, 47, 152, 201, 200, 241, 238, 51, 26, 68, 55, 67,
            81, 186, 215, 37, 251, 195, 122, 212, 169, 146, 147, 206, 35,
        ]
    );
    assert_eq!(
        identity.activity_handler_registry_digest(),
        [
            228, 29, 48, 246, 158, 223, 147, 254, 101, 27, 52, 193, 90, 202, 190, 227, 232, 34,
            229, 65, 216, 202, 100, 150, 89, 3, 25, 50, 17, 12, 231, 91,
        ]
    );
    assert_eq!(
        identity.composition_digest(),
        [
            90, 126, 185, 138, 212, 35, 180, 153, 119, 33, 109, 159, 31, 111, 237, 170, 122, 233,
            176, 214, 169, 183, 233, 248, 85, 180, 30, 230, 162, 249, 90, 211,
        ]
    );
}

#[test]
fn component_set_has_exact_ten_component_closure_and_stable_order() {
    let identity = identity();
    let components = gold_and_gears_component_set(
        &identity,
        [0x11; 32],
        [0x22; 32],
        [0x33; 32],
        [0x44; 32],
        [0x55; 32],
        ("baseline-controller", [0x66; 32]),
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
            46, 214, 208, 103, 251, 126, 224, 139, 82, 46, 87, 18, 14, 218, 229, 185, 96, 102, 77,
            73, 186, 75, 241, 34, 141, 125, 72, 63, 65, 137, 130, 36,
        ]
    );

    let changed_controller = gold_and_gears_component_set(
        &identity,
        [0x11; 32],
        [0x22; 32],
        [0x33; 32],
        [0x44; 32],
        [0x55; 32],
        ("baseline-controller", [0x67; 32]),
    )
    .unwrap();
    assert_ne!(components.root(), changed_controller.root());
}
