#[test]
fn activity_decision_a08_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A08,
        programs: 64,
        shapes: 262,
        occurrences: 769,
        operation_types: 9,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0623001-opt0623001-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0625902-act0625902-json",
        prior_partitions: &[
            super::DivergentUniverseActivityDecisionPartition::A02,
            super::DivergentUniverseActivityDecisionPartition::A03,
            super::DivergentUniverseActivityDecisionPartition::A04,
            super::DivergentUniverseActivityDecisionPartition::A05,
            super::DivergentUniverseActivityDecisionPartition::A06,
            super::DivergentUniverseActivityDecisionPartition::A07,
        ],
    });
}

#[test]
fn all_activity_decision_a08_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A08,
        64,
        2,
        78,
        118,
    );
}

#[test]
fn activity_decision_a08_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A08,
        79,
        119,
        0x78,
    );
}
