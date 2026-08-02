use super::{
    GoldAndGearsEntryError, GoldAndGearsRuntimeFactory,
    runtime_coverage::{category_contract, validate_exact_ids, validate_exact_rule_bindings},
};

#[test]
fn production_factory_closes_all_three_frozen_denominators_exactly_once() {
    let factory = factory();
    let summary = factory.runtime_coverage_summary();
    assert_eq!(summary.source_categories(), 42);
    assert_eq!(summary.source_runtime_slices(), 44);
    assert_eq!(summary.source_obligations(), 7_913);
    assert_eq!(summary.mechanic_rules(), 1_224);
    assert_eq!(summary.semantic_fixtures(), 18);
    assert_eq!(summary.native_handlers(), 0);
}

#[test]
fn all_42_source_categories_have_exact_owner_and_disposition_slices() {
    let summary = factory().runtime_coverage_summary();
    assert_eq!(summary.source_categories(), 42);
    assert_eq!(summary.source_runtime_slices(), 44);
    assert_eq!(summary.integrated_obligations(), 7_181);
    assert_eq!(summary.shared_integrated_obligations(), 706);
    assert_eq!(summary.external_outcomes(), 8);
    assert_eq!(summary.metadata_obligations(), 18);
}

#[test]
fn exact_id_validator_rejects_missing_duplicate_and_orphan_runtime_rows() {
    assert_eq!(
        validate_exact_ids(&["a", "b"], &["a"], 2),
        Err(GoldAndGearsEntryError::InvalidRuntimeCoverage)
    );
    assert_eq!(
        validate_exact_ids(&["a", "b"], &["a", "a"], 2),
        Err(GoldAndGearsEntryError::InvalidRuntimeCoverage)
    );
    assert_eq!(
        validate_exact_ids(&["a", "b"], &["a", "c"], 2),
        Err(GoldAndGearsEntryError::InvalidRuntimeCoverage)
    );
    assert_eq!(validate_exact_ids(&["a", "b"], &["a", "b"], 2), Ok(()));
    assert_eq!(
        validate_exact_rule_bindings(&[("a", "owner-a")], &[("a", "owner-b")], 1),
        Err(GoldAndGearsEntryError::InvalidRuntimeCoverage)
    );
}

#[test]
fn source_contract_rejects_unknown_categories_and_mixed_count_drift() {
    assert!(category_contract("unknown-category", 1).is_none());
    assert!(category_contract("beacons", 5).is_none());
    assert!(category_contract("curios", 79).is_none());
    assert!(category_contract("occurrences", 61).is_none());
    assert_eq!(category_contract("curios", 80).unwrap().len(), 2);
    assert_eq!(category_contract("occurrences", 62).unwrap().len(), 2);
}

fn factory() -> &'static GoldAndGearsRuntimeFactory {
    super::tests::shared_factory()
}
