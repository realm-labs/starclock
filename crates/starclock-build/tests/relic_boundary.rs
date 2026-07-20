use starclock_build::{
    digest::COMBATANT_BUILD_DIGEST_REVISION,
    relic_boundary::{
        DeferredRelicBoundary, RELIC_BOUNDARY_REVISION, RelicBoundaryError, RelicSetFamily,
        RelicSlot,
    },
    spec::{CombatantBuildSpec, PromotionStage},
};
use starclock_combat::{UnitDefinitionId, UnitLevel};

#[test]
fn deferred_boundary_is_explicit_versioned_and_empty() {
    let spec = CombatantBuildSpec::new(
        UnitDefinitionId::new(1).unwrap(),
        UnitLevel::new(80).unwrap(),
        PromotionStage::new(6).unwrap(),
    );
    let boundary = spec.relic_boundary();

    assert_eq!(boundary, DeferredRelicBoundary::EMPTY);
    assert_eq!(boundary.revision(), RELIC_BOUNDARY_REVISION);
    assert_eq!(boundary.piece_count(), 0);
    assert_eq!(
        COMBATANT_BUILD_DIGEST_REVISION,
        "starclock-combatant-build-v2"
    );
    assert_eq!(boundary.verify_revision(RELIC_BOUNDARY_REVISION), Ok(()));
    assert_eq!(
        boundary.verify_revision("relic-planar-future-v2"),
        Err(RelicBoundaryError::IncompatibleRevision)
    );
}

#[test]
fn slot_family_contract_is_closed_without_importing_definitions() {
    let cavern = [
        RelicSlot::Head,
        RelicSlot::Hands,
        RelicSlot::Body,
        RelicSlot::Feet,
    ];
    let planar = [RelicSlot::PlanarSphere, RelicSlot::LinkRope];

    assert!(
        cavern
            .into_iter()
            .all(|slot| slot.family() == RelicSetFamily::Cavern)
    );
    assert!(
        planar
            .into_iter()
            .all(|slot| slot.family() == RelicSetFamily::Planar)
    );
}
