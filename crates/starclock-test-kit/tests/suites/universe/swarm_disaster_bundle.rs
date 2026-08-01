use starclock_mode_universe::{
    error::UniverseCatalogLoadErrorKind, swarm_disaster_catalog::validate_swarm_disaster_bundle,
};

const BUNDLE: &[u8] = include_bytes!("../../../../../config/swarm-disaster-generated/config.sora");

#[test]
fn exact_goal09_bundle_loads_through_the_generated_type_free_boundary() {
    validate_swarm_disaster_bundle(BUNDLE).unwrap();
}

#[test]
fn a_different_or_tampered_bundle_is_rejected_before_lowering() {
    let mut tampered = BUNDLE.to_vec();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert_eq!(
        validate_swarm_disaster_bundle(&tampered)
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidEmbeddedData
    );
    assert_eq!(
        validate_swarm_disaster_bundle(include_bytes!(
            "../../../../../config/universe-generated/config.sora"
        ))
        .unwrap_err()
        .kind(),
        UniverseCatalogLoadErrorKind::InvalidEmbeddedData
    );
}
