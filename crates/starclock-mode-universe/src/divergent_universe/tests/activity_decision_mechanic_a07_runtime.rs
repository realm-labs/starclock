#[test]
fn activity_decision_a07_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A07,
        programs: 64,
        shapes: 255,
        occurrences: 669,
        operation_types: 10,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0620701-opt0620701-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0623001-act0623001-json",
        prior_partitions: &[
            super::DivergentUniverseActivityDecisionPartition::A02,
            super::DivergentUniverseActivityDecisionPartition::A03,
            super::DivergentUniverseActivityDecisionPartition::A04,
            super::DivergentUniverseActivityDecisionPartition::A05,
            super::DivergentUniverseActivityDecisionPartition::A06,
        ],
    });
}

#[test]
fn all_activity_decision_a07_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A07,
        64,
        2,
        76,
        116,
    );
}

#[test]
fn activity_decision_a07_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A07,
        77,
        117,
        0x77,
    );
}
