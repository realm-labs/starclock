use std::sync::Arc;

use starclock_activity::{
    ActivityBattleResultContract, ActivityBattleSettlementError, ActivityBattleStartRequest,
    ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityInstanceId, ActivityMasterSeed, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivityStateDefinition, ActivityStateHash,
    ActivityTransactionState, BattleResultProjection, OneBattleFlow, ProjectionField, ProjectionId,
};

#[test]
fn four_thousand_ninety_six_invalid_commands_preserve_bytes_hash_and_rng() {
    let graph = graph();
    let identity = identity();
    let instance = ActivityInstanceId::new(7).unwrap();
    let rng = rng();
    let contract = Arc::new(
        ActivityBattleResultContract::new(
            Arc::new(
                BattleResultProjection::new(
                    ProjectionId::new(1).unwrap(),
                    vec![
                        ProjectionField::Outcome,
                        ProjectionField::FinalStateHash,
                        ProjectionField::EventDigest,
                        ProjectionField::TerminalFault,
                    ],
                )
                .unwrap(),
            ),
            vec![],
            vec![],
        )
        .unwrap(),
    );
    let mut state = ActivityTransactionState::new(
        ActivityStateDefinition::new(vec![], vec![], vec![]).unwrap(),
        graph.entry(),
    );
    let expected = state.state_hash(identity, &graph, instance, &rng);
    let bytes = state.canonical_state_bytes(identity, &graph, instance, &rng);
    let snapshots = rng.snapshots();
    for index in 0..4_096_u32 {
        let (hash, error) = if index % 2 == 0 {
            (
                ActivityStateHash::new([index.wrapping_add(1) as u8; 32]).unwrap(),
                ActivityBattleSettlementError::StaleState,
            )
        } else {
            (
                expected,
                ActivityBattleSettlementError::MissingPendingBattle,
            )
        };
        assert_eq!(
            state.start_pending_battle(
                &graph,
                &rng,
                ActivityBattleStartRequest::new(hash, identity, instance, Arc::clone(&contract),),
            ),
            Err(error)
        );
        assert_eq!(
            state.canonical_state_bytes(identity, &graph, instance, &rng),
            bytes
        );
        assert_eq!(rng.snapshots(), snapshots);
    }
}

#[test]
fn perturbing_each_rng_stream_leaves_every_other_next_draw_identical() {
    for perturbed_label in ActivityRngLabel::ALL {
        let mut baseline = rng();
        let mut perturbed = rng();
        for purpose in 1..=257_u16 {
            perturbed
                .choose_index(perturbed_label, purpose, 31)
                .unwrap();
        }
        for label in ActivityRngLabel::ALL {
            if label == perturbed_label {
                continue;
            }
            let expected = baseline.choose_index(label, 911, 97).unwrap().unwrap();
            let actual = perturbed.choose_index(label, 911, 97).unwrap().unwrap();
            assert_eq!(actual, expected, "{perturbed_label:?} shifted {label:?}");
        }
        let baseline_draws = baseline
            .snapshots()
            .iter()
            .find(|snapshot| snapshot.label() == perturbed_label)
            .unwrap()
            .draw_count();
        let perturbed_draws = perturbed
            .snapshots()
            .iter()
            .find(|snapshot| snapshot.label() == perturbed_label)
            .unwrap()
            .draw_count();
        assert_eq!(baseline_draws, 0);
        assert_eq!(perturbed_draws, 257);
    }
}

fn graph() -> starclock_activity::ActivityGraphDefinition {
    OneBattleFlow::new(
        starclock_activity::SectionId::new(10).unwrap(),
        starclock_activity::NodeId::new(20).unwrap(),
        starclock_activity::NodeId::new(21).unwrap(),
        starclock_activity::NodeId::new(22).unwrap(),
        starclock_activity::NodeId::new(23).unwrap(),
    )
    .unwrap()
    .into_graph()
}

fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([0xa1; 32]).unwrap(),
        ActivityConfigDigest::new([0xa2; 32]).unwrap(),
    )
}

fn rng() -> ActivityRngStreams {
    let graph = graph();
    let identity = identity();
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(5),
        identity.id(),
        identity.definition_digest(),
        identity.config_digest(),
        graph.digest(),
        ActivityInstanceId::new(7).unwrap(),
        Some(starclock_activity::SectionId::new(10).unwrap()),
        Some(starclock_activity::NodeId::new(20).unwrap()),
        Some(starclock_activity::AttemptId::new(1).unwrap()),
        1,
    ))
}
