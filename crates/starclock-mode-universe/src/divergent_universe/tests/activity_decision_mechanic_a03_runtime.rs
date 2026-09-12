#[test]
fn activity_decision_a03_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A03,
        programs: 64,
        shapes: 286,
        occurrences: 463,
        operation_types: 15,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0412801-opt0412801-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0419504-act0419504-json",
        prior_partitions: &[super::DivergentUniverseActivityDecisionPartition::A02],
    });
}

#[test]
fn all_activity_decision_a03_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A03,
        64,
        2,
        68,
        108,
    );
}

#[test]
fn activity_decision_a03_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A03,
        69,
        109,
        0x73,
    );
}
