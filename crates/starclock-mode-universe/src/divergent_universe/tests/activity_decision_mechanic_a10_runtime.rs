#[test]
fn activity_decision_a10_closes_all_exact_source_shapes() {
    assert_activity_decision_partition_closure(ActivityDecisionPartitionExpectations {
        partition: super::DivergentUniverseActivityDecisionPartition::A10,
        programs: 64,
        shapes: 128,
        occurrences: 128,
        operation_types: 2,
        first_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-230-roguenpc413401-json",
        last_mechanic: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-310-roguenpc614301-json",
        prior_partitions: &[
            super::DivergentUniverseActivityDecisionPartition::A02,
            super::DivergentUniverseActivityDecisionPartition::A03,
            super::DivergentUniverseActivityDecisionPartition::A04,
            super::DivergentUniverseActivityDecisionPartition::A05,
            super::DivergentUniverseActivityDecisionPartition::A06,
            super::DivergentUniverseActivityDecisionPartition::A07,
            super::DivergentUniverseActivityDecisionPartition::A08,
            super::DivergentUniverseActivityDecisionPartition::A09,
        ],
    });
}

#[test]
fn all_activity_decision_a10_mechanics_commit_once_without_rng() {
    assert_all_activity_decision_mechanics_commit_once_without_rng(
        super::DivergentUniverseActivityDecisionPartition::A10,
        64,
        1,
        82,
        122,
    );
}

#[test]
fn activity_decision_a10_rejections_are_atomic_and_reconstruct_fresh() {
    assert_activity_decision_rejections_are_atomic_and_reconstruct_fresh(
        super::DivergentUniverseActivityDecisionPartition::A10,
        83,
        123,
        0x7a,
    );
}
