use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityGraphDefinition,
    ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomOffer, ActivityRandomPolicies, ActivityRngLabel,
    ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue,
    BuildDigest, GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram,
    LoadoutLockScope, NodeId, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    SectionId, SlotCarryPolicy,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

#[test]
fn candidate_filter_and_selected_marker_are_atomic_deterministic_and_reroll_safe() {
    let definition = definition();
    let mut left = start(Arc::clone(&definition), 1);
    let right = start(definition, 1);

    assert_eq!(left.state_hash(), right.state_hash());
    assert_eq!(offered(&left), offered(&right));
    assert_eq!(markers(&left), markers(&right));
    assert_offer_and_marker(&left);

    let before = left.state_hash();
    left.reroll_random_offer(before).unwrap();
    assert_ne!(left.state_hash(), before);
    assert_offer_and_marker(&left);
    let view = left.player_view();
    let decision = view.decision().unwrap();
    left.choose_option(view.state_hash(), decision.id(), decision.options()[0].id())
        .unwrap();
    assert!(markers(&left).contains(&(99, 1)));
}

fn assert_offer_and_marker(activity: &GraphActivity) {
    let visible = offered(activity);
    assert_eq!(visible.len(), 1);
    assert!(visible.iter().all(|id| (2..=4).contains(&id.get())));
    let marked = markers(activity);
    assert_eq!(marked.len(), 1);
    assert_eq!(marked[0].1, 1);
    assert!(visible.iter().any(|option| option.get() == marked[0].0));
}

fn offered(activity: &GraphActivity) -> Vec<ActivityOptionId> {
    activity
        .player_view()
        .decision()
        .expect("random offer")
        .options()
        .iter()
        .map(|option| option.id())
        .collect()
}

fn markers(activity: &GraphActivity) -> Vec<(u64, i64)> {
    activity
        .debug_view()
        .all_slots()
        .iter()
        .find(|value| value.id() == slot(2))
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(values) => Some(values.to_vec()),
            _ => None,
        })
        .expect("private marker map")
}

fn definition() -> Arc<GraphActivityDefinition> {
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::Choice, 8).unwrap(),
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
                8,
            )
            .unwrap(),
        ],
        16,
    )
    .unwrap();
    let state = ActivityStateDefinition::new(
        vec![
            counter_slot(1, ActivityStateVisibility::Private),
            counter_slot(2, ActivityStateVisibility::Private),
        ],
        vec![],
        vec![],
    )
    .unwrap();
    let options = (1_u64..=5)
        .map(|raw| {
            ActivityOptionDefinition::new(
                option(raw),
                raw as i32,
                always(),
                vec![
                    ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap()),
                    ActivityOperation::Terminal(ActivityTerminalOutcome::Completed),
                ],
            )
        })
        .collect::<Vec<_>>();
    let program_definition = ActivityProgramDefinition::new(
        program(1),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Reward,
            options: options.into_boxed_slice(),
        }],
    )
    .unwrap();
    let offer = ActivityRandomOffer::new(
        node(1),
        ActivityRngLabel::Reward,
        101,
        3,
        (1_u64..=5).map(|raw| (option(raw), 1)).collect(),
        Some((slot(1), 2)),
    )
    .unwrap()
    .with_maximum_options_reduction(always(), 1)
    .unwrap()
    .with_maximum_options_reduction(always(), 1)
    .unwrap()
    .with_conditional_candidate_filter(always(), vec![option(2), option(3), option(4)])
    .unwrap()
    .with_selected_option_marker(always(), slot(2), ActivityRngLabel::Reward, 102, 1)
    .unwrap()
    .with_selection_prefix(vec![ActivityOperation::AddCounter {
        slot: slot(2),
        key: 99,
        delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
    }])
    .unwrap();
    Arc::new(
        GraphActivityDefinition::new(
            ActivityDefinitionIdentity::new(
                ActivityDefinitionId::new(1).unwrap(),
                ActivityDefinitionDigest::new([1; 32]).unwrap(),
                ActivityConfigDigest::new([2; 32]).unwrap(),
            ),
            graph,
            state,
            Arc::new(participants()),
            vec![GraphActivityNodeProgram::new(node(1), program_definition)],
            None,
            ActivityRandomPolicies::new(Vec::new(), vec![offer]),
        )
        .unwrap(),
    )
}

fn counter_slot(raw: u32, visibility: ActivityStateVisibility) -> ActivitySlotDefinition {
    ActivitySlotDefinition::new_with_policy(
        slot(raw),
        ActivityScope::Activity,
        ActivityValue::BoundedCounterMap(Box::new([])),
        Some((0, 2)),
        Some(8),
        vec![],
        SlotCarryPolicy::CarryExact,
        visibility,
        ActivityStateSource::new(u64::from(raw)).unwrap(),
    )
    .unwrap()
}

fn start(definition: Arc<GraphActivityDefinition>, instance: u64) -> GraphActivity {
    GraphActivity::start(
        definition,
        ActivityInstanceId::new(instance).unwrap(),
        ActivityMasterSeed::from_u64(7),
    )
    .unwrap()
    .into_activity()
}

fn participants() -> ParticipantLock {
    let policy = ParticipantPolicy::new(
        1,
        1,
        1,
        ParticipantUniquenessScope::Activity,
        LoadoutLockScope::Activity,
    )
    .unwrap();
    let build = OpaqueParticipantBuild::new(
        CombatantSpecDigest::new([3; 32]).unwrap(),
        BuildDigest::new([4; 32]).unwrap(),
        ParticipantSourceKind::Synthetic,
    )
    .unwrap();
    ParticipantLock::seal(
        policy,
        vec![
            ParticipantLockEntry::new(
                ParticipantId::new(1).unwrap(),
                0,
                0,
                UnitDefinitionId::new(1).unwrap(),
                build,
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn always() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}

fn node(raw: u32) -> NodeId {
    NodeId::new(raw).unwrap()
}
fn section(raw: u32) -> SectionId {
    SectionId::new(raw).unwrap()
}
fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn option(raw: u64) -> ActivityOptionId {
    ActivityOptionId::new(raw).unwrap()
}
fn program(raw: u32) -> ActivityProgramId {
    ActivityProgramId::new(raw).unwrap()
}
