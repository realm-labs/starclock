#[test]
fn activity_decision_a09_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A09,
        programs: 64,
        shapes: 191,
        occurrences: 353,
        operation_types: 11,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0625902-opt0625902-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-230-roguenpc413301-json",
        prior_partitions: &[
            super::DivergentUniverseActivityDecisionPartition::A02,
            super::DivergentUniverseActivityDecisionPartition::A03,
            super::DivergentUniverseActivityDecisionPartition::A04,
            super::DivergentUniverseActivityDecisionPartition::A05,
            super::DivergentUniverseActivityDecisionPartition::A06,
            super::DivergentUniverseActivityDecisionPartition::A07,
            super::DivergentUniverseActivityDecisionPartition::A08,
        ],
    });
}

#[test]
fn all_activity_decision_a09_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A09,
        64,
        2,
        80,
        120,
    );
}

#[test]
fn activity_decision_a09_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A09,
        81,
        121,
        0x79,
    );
}
