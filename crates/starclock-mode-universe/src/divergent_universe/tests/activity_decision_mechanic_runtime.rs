#[test]
fn activity_decision_a02_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A02,
        programs: 64,
        shapes: 295,
        occurrences: 526,
        operation_types: 17,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-maze-mazerogue-roguetourn-roguetourn-goup-waitdialogue-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0412801-act0412801-json",
        prior_partitions: &[],
    });
}

#[test]
fn all_activity_decision_a02_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A02,
        64,
        3,
        66,
        106,
    );
}

#[test]
fn activity_decision_a02_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A02,
        67,
        107,
        0x72,
    );
}
