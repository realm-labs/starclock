use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams,
};

use crate::error::UniverseCatalogLoadErrorKind;

use super::OccurrenceRuleRuntimeCatalog;
use crate::swarm_disaster_entry::{SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance};

#[test]
fn exact_sora_rule_binds_and_contract_drift_fails_closed() {
    let factory = factory();
    let instance = instance(&factory);
    assert_eq!(
        hex(instance.occurrence_rule_runtime_digest()),
        "65dcb9286df7a0457737396024b04c6a1e5f3b91eedc4f3859f5eebf75cf79d2"
    );

    let mut input = factory
        .content
        .mechanic_rule_runtime_input("occurrence-choice")
        .unwrap();
    input.domain = "CrossBattle".into();
    assert_eq!(
        OccurrenceRuleRuntimeCatalog::compile(input)
            .unwrap_err()
            .kind(),
        UniverseCatalogLoadErrorKind::InvalidReference
    );
}

#[test]
fn exact_fixture_variant_and_choice_route_through_the_existing_catalog() {
    let instance = instance(&factory());
    let choices = instance
        .occurrence_choice_keys("swarm-disaster.occurrence-variant.110301")
        .unwrap();
    assert!(
        choices
            .iter()
            .any(|key| key.as_ref() == "swarm-disaster.occurrence-choice.110301.04")
    );
}

#[test]
fn pool_and_outcome_draws_share_only_the_occurrence_stream() {
    let instance = instance(&factory());
    let mut rng = activity_rng(&instance, 41);
    let before = rng.snapshots();
    assert_eq!(
        instance
            .select_occurrence(
                "occurrence",
                &[
                    ("swarm-disaster.occurrence.10".into(), 1),
                    ("swarm-disaster.occurrence.2".into(), 3),
                ],
                &mut rng,
            )
            .unwrap()
            .as_deref(),
        Some("swarm-disaster.occurrence.2")
    );
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Occurrence, 1);

    let mut rng = activity_rng(&instance, 43);
    assert_eq!(
        instance
            .select_occurrence_outcome_candidates(
                "swarm-disaster.occurrence-choice.110101.01",
                &[30, 10, 20],
                2,
                &mut rng,
            )
            .unwrap()
            .as_ref(),
        &[10, 30]
    );
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(super::super::tests::BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(super::super::tests::released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            super::super::tests::participants(super::super::tests::policy()),
        ))
        .unwrap()
}

fn activity_rng(instance: &SwarmDisasterRuntimeInstance, seed: u64) -> ActivityRngStreams {
    let identity = ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(20).unwrap(),
        ActivityDefinitionDigest::new([0x20; 32]).unwrap(),
        ActivityConfigDigest::new([0x53; 32]).unwrap(),
    );
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(seed),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        instance.graph_definition().digest(),
        ActivityInstanceId::new(1).unwrap(),
        None,
        Some(instance.graph_definition().entry()),
        None,
        0,
    ))
}

fn assert_only_label_advanced(
    before: &[starclock_activity::ActivityRngStreamSnapshot],
    after: &[starclock_activity::ActivityRngStreamSnapshot],
    label: ActivityRngLabel,
    draws: u64,
) {
    for (before, after) in before.iter().zip(after) {
        let expected = if after.label() == label { draws } else { 0 };
        assert_eq!(after.draw_count(), before.draw_count() + expected);
    }
}

fn hex(bytes: [u8; 32]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
