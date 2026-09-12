#[test]
fn activity_decision_a11_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A11,
        programs: 62,
        shapes: 124,
        occurrences: 124,
        operation_types: 2,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-310-roguenpc614401-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-330-roguenpc627001-json",
        prior_partitions: &[
            super::DivergentUniverseActivityDecisionPartition::A02,
            super::DivergentUniverseActivityDecisionPartition::A03,
            super::DivergentUniverseActivityDecisionPartition::A04,
            super::DivergentUniverseActivityDecisionPartition::A05,
            super::DivergentUniverseActivityDecisionPartition::A06,
            super::DivergentUniverseActivityDecisionPartition::A07,
            super::DivergentUniverseActivityDecisionPartition::A08,
            super::DivergentUniverseActivityDecisionPartition::A09,
            super::DivergentUniverseActivityDecisionPartition::A10,
        ],
    });
}

#[test]
fn all_activity_decision_a11_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A11,
        62,
        1,
        84,
        124,
    );
}

#[test]
fn activity_decision_a11_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A11,
        85,
        125,
        0x7b,
    );
}
