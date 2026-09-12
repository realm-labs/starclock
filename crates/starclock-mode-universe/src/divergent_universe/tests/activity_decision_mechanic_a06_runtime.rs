#[test]
fn activity_decision_a06_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A06,
        programs: 64,
        shapes: 268,
        occurrences: 378,
        operation_types: 12,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0613801-opt0613801-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0620701-act0620701-json",
        prior_partitions: &[
            super::DivergentUniverseActivityDecisionPartition::A02,
            super::DivergentUniverseActivityDecisionPartition::A03,
            super::DivergentUniverseActivityDecisionPartition::A04,
            super::DivergentUniverseActivityDecisionPartition::A05,
        ],
    });
}

#[test]
fn all_activity_decision_a06_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A06,
        64,
        2,
        74,
        114,
    );
}

#[test]
fn activity_decision_a06_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A06,
        75,
        115,
        0x76,
    );
}
