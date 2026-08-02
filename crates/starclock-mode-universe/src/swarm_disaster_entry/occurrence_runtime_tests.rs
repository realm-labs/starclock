use starclock_activity::{
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams,
};

use crate::swarm_disaster_entry::{
    SwarmDisasterRuntimeFactory, SwarmDisasterRuntimeInstance,
    tests::{BUNDLE, participants, policy, released_entry},
};

#[test]
fn frozen_occurrence_catalog_closes_every_pool_variant_choice_and_random_policy() {
    let factory = factory();
    assert_eq!(factory.occurrences.denominators(), (75, 57, 308, 60));
    let instance = instance(&factory);
    assert_eq!(instance.occurrence_count(), 75);
}

#[test]
fn weighted_pool_selection_is_canonical_and_uses_only_occurrence_rng() {
    let instance = instance(&factory());
    let mut rng = activity_rng(&instance, 41);
    let before = rng.snapshots();
    let selected = instance
        .select_occurrence(
            "occurrence",
            &[
                ("swarm-disaster.occurrence.10".into(), 1),
                ("swarm-disaster.occurrence.2".into(), 3),
            ],
            &mut rng,
        )
        .unwrap();
    assert_eq!(selected.as_deref(), Some("swarm-disaster.occurrence.2"));
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Occurrence, 1);

    let before = rng.snapshots();
    assert!(
        instance
            .select_occurrence(
                "the-swarm",
                &[("swarm-disaster.occurrence.2".into(), 1)],
                &mut rng,
            )
            .is_err()
    );
    assert!(
        instance
            .select_occurrence(
                "occurrence",
                &[("swarm-disaster.occurrence.2".into(), 0)],
                &mut rng,
            )
            .is_err()
    );
    assert!(
        instance
            .select_occurrence(
                "occurrence",
                &[
                    ("swarm-disaster.occurrence.2".into(), 1),
                    ("swarm-disaster.occurrence.2".into(), 1),
                ],
                &mut rng,
            )
            .is_err()
    );
    assert_eq!(before, rng.snapshots());
}

#[test]
fn occurrence_variant_and_choice_closure_preserves_authored_order() {
    let instance = instance(&factory());
    assert_eq!(
        instance
            .occurrence_variant_keys("swarm-disaster.occurrence.2")
            .unwrap()
            .iter()
            .map(Box::as_ref)
            .collect::<Vec<_>>(),
        ["swarm-disaster.occurrence-variant.110101"]
    );
    assert_eq!(
        instance
            .occurrence_choice_keys("swarm-disaster.occurrence-variant.110101")
            .unwrap()
            .iter()
            .map(Box::as_ref)
            .collect::<Vec<_>>(),
        [
            "swarm-disaster.occurrence-choice.110101.01",
            "swarm-disaster.occurrence-choice.110101.02",
            "swarm-disaster.occurrence-choice.110101.03",
            "swarm-disaster.occurrence-choice.110101.04",
        ]
    );
    assert!(instance.occurrence_variant_keys("missing").is_err());
    assert!(instance.occurrence_choice_keys("missing").is_err());
}

#[test]
fn seeded_random_outcomes_are_stable_fail_closed_and_do_not_draw_for_empty_work() {
    let instance = instance(&factory());
    let mut rng = activity_rng(&instance, 43);
    let before = rng.snapshots();
    let selected = instance
        .select_occurrence_outcome_candidates(
            "swarm-disaster.occurrence-choice.110101.01",
            &[30, 10, 20],
            2,
            &mut rng,
        )
        .unwrap();
    assert_eq!(selected.as_ref(), &[10, 30]);
    assert_only_label_advanced(&before, &rng.snapshots(), ActivityRngLabel::Occurrence, 2);

    let before = rng.snapshots();
    assert!(
        instance
            .select_occurrence_outcome_candidates(
                "swarm-disaster.occurrence-choice.110101.02",
                &[1],
                1,
                &mut rng,
            )
            .is_err()
    );
    assert!(
        instance
            .select_occurrence_outcome_candidates(
                "swarm-disaster.occurrence-choice.110101.01",
                &[1, 1],
                1,
                &mut rng,
            )
            .is_err()
    );
    assert!(
        instance
            .select_occurrence_outcome_candidates(
                "swarm-disaster.occurrence-choice.110101.01",
                &[1, 2],
                0,
                &mut rng,
            )
            .unwrap()
            .is_empty()
    );
    assert_eq!(before, rng.snapshots());
}

fn factory() -> SwarmDisasterRuntimeFactory {
    SwarmDisasterRuntimeFactory::load_candidate(BUNDLE).unwrap()
}

fn instance(factory: &SwarmDisasterRuntimeFactory) -> SwarmDisasterRuntimeInstance {
    factory
        .compile_entry(released_entry(
            "swarm-disaster.area.201",
            "universe.path.preservation",
            "swarm-disaster.audience-die.1",
            participants(policy()),
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
