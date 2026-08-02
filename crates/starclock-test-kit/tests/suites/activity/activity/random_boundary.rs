use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityGraphDefinition,
    ActivityInstanceId, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition, ActivityStateHash,
    ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue,
    BuildDigest, GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram,
    LoadoutLockScope, NodeId, OpaqueParticipantBuild, ParticipantId, ParticipantLock,
    ParticipantLockEntry, ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope,
    SectionId, SlotCarryPolicy,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

#[test]
fn random_option_boundary_is_atomic_replayable_and_without_replacement() {
    let definition = definition();
    let mut left = start(Arc::clone(&definition), 1);
    let mut right = start(definition, 1);
    let initial = left.state_hash();
    assert_eq!(initial, right.state_hash());
    let candidates = candidates();
    let prefix = [ActivityOperation::AddCounter {
        slot: slot(1),
        key: 99,
        delta: integer(1),
    }];
    let left_result = left
        .apply_random_option_boundary(
            initial,
            program(2),
            ActivityRngLabel::Reward,
            101,
            102,
            1,
            3,
            &prefix,
            &candidates,
        )
        .unwrap();
    let right_result = right
        .apply_random_option_boundary(
            initial,
            program(2),
            ActivityRngLabel::Reward,
            101,
            102,
            1,
            3,
            &prefix,
            &candidates,
        )
        .unwrap();
    assert_eq!(left_result, right_result);
    assert_eq!(left.state_hash(), right.state_hash());
    assert!((1..=3).contains(&left_result.selected_options().len()));
    let mut unique = left_result.selected_options().to_vec();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), left_result.selected_options().len());
    let values = visible_counters(&left);
    assert!(values.contains(&(99, 1)));
    for option in left_result.selected_options() {
        assert!(values.contains(&(option.get(), 1)));
    }

    let accepted = left.state_hash();
    let draws = left.debug_view().rng().to_vec();
    assert!(
        left.apply_random_option_boundary(
            ActivityStateHash::new([0xaa; 32]).unwrap(),
            program(2),
            ActivityRngLabel::Reward,
            101,
            102,
            1,
            3,
            &prefix,
            &candidates,
        )
        .is_err()
    );
    assert_eq!(left.state_hash(), accepted);
    assert_eq!(left.debug_view().rng(), draws);
}

#[test]
fn zero_selection_random_boundary_commits_prefix_without_rng_draws() {
    let definition = definition();
    let mut activity = start(definition, 2);
    let initial = activity.state_hash();
    let draws = activity.debug_view().rng().to_vec();
    let prefix = [ActivityOperation::AddCounter {
        slot: slot(1),
        key: 77,
        delta: integer(1),
    }];

    let result = activity
        .apply_random_option_boundary(
            initial,
            program(2),
            ActivityRngLabel::Reward,
            101,
            102,
            0,
            0,
            &prefix,
            &candidates(),
        )
        .unwrap();

    assert!(result.selected_options().is_empty());
    assert_eq!(activity.debug_view().rng(), draws);
    assert!(visible_counters(&activity).contains(&(77, 1)));
}

fn candidates() -> Vec<(ActivityOptionDefinition, u64)> {
    (1_u64..=5)
        .map(|raw| {
            (
                ActivityOptionDefinition::new(
                    option(raw),
                    raw as i32,
                    always(),
                    vec![ActivityOperation::AddCounter {
                        slot: slot(1),
                        key: raw,
                        delta: integer(1),
                    }],
                ),
                raw,
            )
        })
        .collect()
}

fn visible_counters(activity: &GraphActivity) -> Vec<(u64, i64)> {
    let view = activity.player_view();
    view.slots()
        .iter()
        .find(|value| value.id() == slot(1))
        .and_then(|value| match value.value() {
            ActivityValue::BoundedCounterMap(values) => Some(values.to_vec()),
            _ => None,
        })
        .expect("visible counter map")
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
            ActivitySlotDefinition::new_with_policy(
                slot(1),
                ActivityScope::Activity,
                ActivityValue::BoundedCounterMap(Box::new([])),
                Some((0, 1_000)),
                Some(128),
                vec![],
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(1).unwrap(),
            )
            .unwrap(),
        ],
        vec![],
        vec![],
    )
    .unwrap();
    let choice = ActivityOptionDefinition::new(
        option(100),
        0,
        always(),
        vec![
            ActivityOperation::Traverse(ActivityEdgeId::new(1).unwrap()),
            ActivityOperation::Terminal(ActivityTerminalOutcome::Completed),
        ],
    );
    let program_definition = ActivityProgramDefinition::new(
        program(1),
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Choice,
            options: vec![choice].into_boxed_slice(),
        }],
    )
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
            ActivityRandomPolicies::default(),
        )
        .unwrap(),
    )
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

fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
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
