use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionId,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityExpression, ActivityGraphDefinition, ActivityInstanceId, ActivityMasterSeed,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityRngContext,
    ActivityRngLabel, ActivityRngStreams, ActivityScope, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue, NodeId, SectionId,
    SlotCarryPolicy,
};

#[test]
fn labeled_rng_streams_are_golden_reproducible_and_perturbation_isolated() {
    let mut left = rng();
    let mut right = rng();
    assert_eq!(left.snapshots(), right.snapshots());
    assert_eq!(
        left.choose_index(ActivityRngLabel::Graph, 1, 0).unwrap(),
        None
    );
    assert_eq!(left.snapshots()[0].draw_count(), 0);
    let occurrence_before = left.snapshots()[ActivityRngLabel::Occurrence as usize];
    assert_eq!(
        left.choose_weighted(ActivityRngLabel::Occurrence, 4, &[0, 0])
            .unwrap(),
        None
    );
    assert_eq!(
        left.choose_weighted(ActivityRngLabel::Occurrence, 0, &[1]),
        Err(starclock_activity::ActivityRngError::InvalidPurpose)
    );
    assert_eq!(
        left.snapshots()[ActivityRngLabel::Occurrence as usize],
        occurrence_before
    );

    let graph = left
        .choose_index(ActivityRngLabel::Graph, 1, 17)
        .unwrap()
        .unwrap();
    assert_eq!(
        graph,
        right
            .choose_index(ActivityRngLabel::Graph, 1, 17)
            .unwrap()
            .unwrap()
    );
    assert_eq!(graph.raw(), 8_809_253_053_890_565_554);

    let _ = left.choose_index(ActivityRngLabel::Reward, 2, 9).unwrap();
    let next_left = left.choose_index(ActivityRngLabel::Graph, 3, 17).unwrap();
    let next_right = right.choose_index(ActivityRngLabel::Graph, 3, 17).unwrap();
    assert_eq!(next_left, next_right);
    assert_eq!(
        left.choose_weighted(ActivityRngLabel::Occurrence, 4, &[0, 2, 0, 5])
            .unwrap()
            .unwrap()
            .0,
        3
    );
}

#[test]
fn weighted_without_replacement_is_unique_bounded_and_draw_exact() {
    let mut left = rng();
    let mut right = rng();
    let selected = left
        .choose_weighted_without_replacement(ActivityRngLabel::Reward, 9, &[1, 2, 3, 4], 3)
        .unwrap();
    assert_eq!(
        selected,
        right
            .choose_weighted_without_replacement(ActivityRngLabel::Reward, 9, &[1, 2, 3, 4], 3,)
            .unwrap()
    );
    assert_eq!(selected.len(), 3);
    let mut unique = selected.to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), 3);
    let reward = left
        .snapshots()
        .iter()
        .find(|stream| stream.label() == ActivityRngLabel::Reward)
        .copied()
        .unwrap();
    assert_eq!(reward.draw_count(), 3);

    let before = left.snapshots();
    assert!(
        left.choose_weighted_without_replacement(ActivityRngLabel::Reward, 9, &[0, 0], 2)
            .unwrap()
            .is_empty()
    );
    assert_eq!(left.snapshots(), before);
}

#[test]
fn canonical_v2_state_bytes_and_hash_cover_commands_values_options_and_rng() {
    let graph = graph();
    let identity = identity();
    let instance = instance();
    let mut state = state();
    let mut rng = rng();
    let initial_bytes = state.canonical_state_bytes(identity, &graph, instance, &rng);
    assert_eq!(&initial_bytes[..8], b"SCAS\x02\0\0\0");
    let initial = state.state_hash(identity, &graph, instance, &rng);
    assert_eq!(
        initial.bytes(),
        [
            89, 78, 108, 132, 216, 12, 57, 224, 126, 242, 189, 212, 44, 119, 252, 133, 162, 199,
            248, 187, 137, 83, 224, 87, 81, 7, 171, 109, 240, 208, 202, 235,
        ]
    );

    assert_eq!(
        state.apply_program(
            &ActivityProgramDefinition::new(program_id(2), vec![]).unwrap(),
            starclock_activity::ActivityCause::new(2, program_id(2), node(1)).unwrap(),
            &graph
        ),
        ActivityTransactionOutcome::Rejected(
            starclock_activity::ActivityTransactionRejection::CauseMismatch
        )
    );
    assert_eq!(
        state.canonical_state_bytes(identity, &graph, instance, &rng),
        initial_bytes
    );

    let program = ActivityProgramDefinition::new(
        program_id(1),
        vec![
            ActivityOperation::SetSlot {
                slot: slot(1),
                value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
            },
            ActivityOperation::Offer {
                kind: starclock_activity::ActivityDecisionKind::Choice,
                options: vec![ActivityOptionDefinition::new(
                    option(1),
                    0,
                    boolean(true),
                    vec![ActivityOperation::Terminal(
                        ActivityTerminalOutcome::Completed,
                    )],
                )]
                .into_boxed_slice(),
            },
        ],
    )
    .unwrap();
    assert!(matches!(
        state.apply_program(
            &program,
            starclock_activity::ActivityCause::new(1, program_id(1), node(1)).unwrap(),
            &graph
        ),
        ActivityTransactionOutcome::Committed(_)
    ));
    let after_command = state.state_hash(identity, &graph, instance, &rng);
    assert_ne!(after_command, initial);
    let _ = rng.choose_index(ActivityRngLabel::Graph, 1, 2).unwrap();
    assert_ne!(
        state.state_hash(identity, &graph, instance, &rng),
        after_command
    );
}

#[test]
fn player_view_is_visibility_filtered_while_debug_view_is_bounded_and_complete() {
    let graph = graph();
    let identity = identity();
    let instance = instance();
    let state = state();
    let rng = rng();
    let player = state.player_view(identity, &graph, instance, &rng);
    let debug = state.debug_view(identity, &graph, instance, &rng);
    assert_eq!(player.slots().len(), 1);
    assert_eq!(player.slots()[0].id(), slot(1));
    assert_eq!(debug.all_slots().len(), 2);
    assert_eq!(debug.rng().len(), 8);
    assert_eq!(debug.player().state_hash(), player.state_hash());
}

fn state() -> ActivityTransactionState {
    let slots = vec![
        ActivitySlotDefinition::new_with_policy(
            slot(1),
            ActivityScope::Activity,
            ActivityValue::Boolean(false),
            None,
            None,
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            source(1),
        )
        .unwrap(),
        ActivitySlotDefinition::new_with_policy(
            slot(2),
            ActivityScope::Activity,
            ActivityValue::BoundedInteger(3),
            Some((0, 9)),
            None,
            vec![],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Private,
            source(2),
        )
        .unwrap(),
    ];
    ActivityTransactionState::new(
        ActivityStateDefinition::new(slots, vec![], vec![]).unwrap(),
        node(1),
    )
}

fn graph() -> ActivityGraphDefinition {
    ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 1).unwrap(),
            ActivityNodeDefinition::new(
                node(2),
                section(1),
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        ],
        vec![
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(1).unwrap(),
                node(1),
                node(2),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        ],
        2,
    )
    .unwrap()
}

fn rng() -> ActivityRngStreams {
    ActivityRngStreams::new(ActivityRngContext::new(
        ActivityMasterSeed::from_u64(0x5eed),
        identity().id(),
        identity().definition_digest(),
        identity().config_digest(),
        graph().digest(),
        instance(),
        Some(section(1)),
        Some(node(1)),
        None,
        0,
    ))
}
fn identity() -> ActivityDefinitionIdentity {
    ActivityDefinitionIdentity::new(
        ActivityDefinitionId::new(1).unwrap(),
        ActivityDefinitionDigest::new([0x11; 32]).unwrap(),
        ActivityConfigDigest::new([0x22; 32]).unwrap(),
    )
}
fn instance() -> ActivityInstanceId {
    ActivityInstanceId::new(7).unwrap()
}
fn boolean(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(value)))
}
fn source(value: u64) -> ActivityStateSource {
    ActivityStateSource::new(value).unwrap()
}
fn program_id(value: u32) -> ActivityProgramId {
    ActivityProgramId::new(value).unwrap()
}
fn option(value: u64) -> ActivityOptionId {
    ActivityOptionId::new(value).unwrap()
}
fn slot(value: u32) -> ActivitySlotId {
    ActivitySlotId::new(value).unwrap()
}
fn node(value: u32) -> NodeId {
    NodeId::new(value).unwrap()
}
fn section(value: u32) -> SectionId {
    SectionId::new(value).unwrap()
}
