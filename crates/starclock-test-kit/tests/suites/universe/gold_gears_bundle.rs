use starclock_mode_universe::gold_gears_catalog::{
    GoldAndGearsBundleLoadError, validate_gold_and_gears_bundle,
};

const BUNDLE: &[u8] = include_bytes!("../../../../../config/gold-and-gears-generated/config.sora");

#[test]
fn exact_goal08_bundle_loads_through_the_public_generated_free_boundary() {
    let summary = validate_gold_and_gears_bundle(BUNDLE).unwrap();
    assert_eq!(summary.table_count(), 52);
    assert_eq!(summary.row_count(), 29_140);
    assert_eq!(summary.source_obligations(), 7_913);
    assert_eq!(summary.mechanic_rules(), 1_224);
    assert_eq!(summary.semantic_fixtures(), 18);
    assert_eq!(summary.policy_boundaries(), 16);
}

#[test]
fn a_different_or_tampered_bundle_is_rejected_before_lowering() {
    let mut tampered = BUNDLE.to_vec();
    let last = tampered.len() - 1;
    tampered[last] ^= 1;
    assert_eq!(
        validate_gold_and_gears_bundle(&tampered),
        Err(GoldAndGearsBundleLoadError::BundleDigest)
    );
    assert_eq!(
        validate_gold_and_gears_bundle(include_bytes!(
            "../../../../../config/universe-generated/config.sora"
        )),
        Err(GoldAndGearsBundleLoadError::BundleDigest)
    );
}
