use super::{SOURCE_CATEGORIES, coverage_snapshot, validate_categories, validate_rule_fixture_ids};
use crate::swarm_disaster_entry::SwarmDisasterRuntimeFactory;
use crate::swarm_disaster_entry::tests::BUNDLE;

#[test]
fn exact_6963_23_23_runtime_coverage_is_stable() {
    let factory = factory();
    let snapshot = coverage_snapshot(&factory.content, factory.semantic_fixtures.digest()).unwrap();
    assert_eq!(snapshot.categories.len(), 42);
    assert_eq!(
        snapshot
            .categories
            .iter()
            .map(|(_, count)| count)
            .sum::<u32>(),
        6_963
    );
    assert_eq!(snapshot.rules.len(), 23);
    assert_eq!(snapshot.fixture_ids.len(), 23);
}

#[test]
fn all_42_source_categories_have_exact_nonzero_denominators() {
    let factory = factory();
    let snapshot = coverage_snapshot(&factory.content, factory.semantic_fixtures.digest()).unwrap();
    assert_eq!(snapshot.categories.len(), SOURCE_CATEGORIES.len());
    assert!(snapshot.categories.iter().zip(SOURCE_CATEGORIES).all(
        |((category, count), expected)| category.as_ref() == expected.0 && *count == expected.1
    ));
    assert!(snapshot.categories.iter().all(|(_, count)| *count > 0));
}

#[test]
fn missing_extra_or_duplicate_source_categories_fail_closed() {
    let factory = factory();
    let snapshot = coverage_snapshot(&factory.content, factory.semantic_fixtures.digest()).unwrap();
    let mut missing = snapshot.categories.to_vec();
    missing.pop();
    assert!(validate_categories(&missing).is_err());

    let mut duplicate = snapshot.categories.to_vec();
    duplicate[1] = duplicate[0].clone();
    assert!(validate_categories(&duplicate).is_err());
}

#[test]
fn orphan_duplicate_or_mismatched_rule_fixture_ids_fail_closed() {
    let factory = factory();
    let snapshot = coverage_snapshot(&factory.content, factory.semantic_fixtures.digest()).unwrap();
    let mut rules = snapshot.rules.to_vec();
    rules[0].fixture_keys[0] = "swarm-disaster.fixture.unknown".into();
    assert!(validate_rule_fixture_ids(&rules, &snapshot.fixture_ids).is_err());

    let mut fixtures = snapshot.fixture_ids.to_vec();
    fixtures[1] = fixtures[0].clone();
    assert!(validate_rule_fixture_ids(&snapshot.rules, &fixtures).is_err());
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}
