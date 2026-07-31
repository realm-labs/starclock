use std::sync::Arc;

use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionId, ActivityDefinitionIdentity, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityExternalOutcomeId,
    ActivityGraphDefinition, ActivityHandlerBundle, ActivityHandlerFault, ActivityHandlerFaultKind,
    ActivityHandlerId, ActivityHandlerInput, ActivityHandlerOutput, ActivityHandlerRegistration,
    ActivityHandlerRegistry, ActivityInstanceId, ActivityInteractionBinding,
    ActivityInteractionRandomPolicy, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomPolicies, ActivityRngLabel, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateDefinition, ActivityStateSource,
    ActivityStateVisibility, ActivityTerminalOutcome, ActivityValue, BuildDigest, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram, LoadoutLockScope,
    NodeId, OpaqueParticipantBuild, ParticipantId, ParticipantLock, ParticipantLockEntry,
    ParticipantPolicy, ParticipantSourceKind, ParticipantUniquenessScope, SectionId,
    SlotCarryPolicy, SlotResetPoint,
};
use starclock_combat::{CombatantSpecDigest, UnitDefinitionId};

const SUCCESS_OUTCOME: u64 = 1;
const HANDLER_FAULT_OUTCOME: u64 = 2;
const OPERATION_FAULT_OUTCOME: u64 = 3;

#[test]
fn stale_and_faulting_random_interactions_preserve_exact_state_and_rng() {
    let definition = definition();
    for (outcome, expected) in [
        (
            HANDLER_FAULT_OUTCOME,
            GraphActivityCommandError::HandlerFault(ActivityHandlerFaultKind::InvalidState),
        ),
        (
            OPERATION_FAULT_OUTCOME,
            GraphActivityCommandError::InteractionFault(
                starclock_activity::ActivityFault::SlotBounds(slot(1)),
            ),
        ),
    ] {
        let mut activity = start(Arc::clone(&definition), outcome);
        let before_bytes = activity.canonical_state_bytes();
        let before_rng = activity.debug_view().rng().to_vec();
        let view = activity.player_view();
        let decision = view.decision().expect("external interaction decision");
        assert_eq!(
            activity.submit_external_outcome(
                starclock_activity::ActivityStateHash::new([0x55; 32]).unwrap(),
                decision.id(),
                external(outcome),
            ),
            Err(GraphActivityCommandError::StaleStateHash)
        );
        assert_eq!(activity.canonical_state_bytes(), before_bytes);
        assert_eq!(activity.debug_view().rng(), before_rng);

        assert_eq!(
            activity.submit_external_outcome(
                activity.state_hash(),
                decision.id(),
                external(outcome),
            ),
            Err(expected)
        );
        assert_eq!(activity.canonical_state_bytes(), before_bytes);
        assert_eq!(activity.debug_view().rng(), before_rng);
    }
}

#[test]
fn accepted_random_interaction_commits_draw_effect_and_transition_together() {
    let definition = definition();
    let mut activity = start(Arc::clone(&definition), SUCCESS_OUTCOME);
    let before = activity.canonical_state_bytes();
    let before_draws = occurrence_draws(&activity);
    let view = activity.player_view();
    let decision = view.decision().expect("external interaction decision");
    activity
        .submit_external_outcome(view.state_hash(), decision.id(), external(SUCCESS_OUTCOME))
        .expect("accepted interaction");
    assert_ne!(activity.canonical_state_bytes(), before);
    assert_eq!(occurrence_draws(&activity), before_draws + 1);
    assert_eq!(
        activity.current_node(),
        node(2),
        "authored transition commits with handler operations"
    );
}

fn handler(input: ActivityHandlerInput<'_>) -> Result<ActivityHandlerOutput, ActivityHandlerFault> {
    match input.payload() {
        [0] => Ok(ActivityHandlerOutput::new(vec![
            ActivityOperation::AddToSlot {
                slot: slot(1),
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(
                    i64::from(input.random_index().unwrap_or(0)) + 1,
                )),
            },
        ])),
        [1] => Err(ActivityHandlerFault::new(
            ActivityHandlerFaultKind::InvalidState,
        )),
        [2] => Ok(ActivityHandlerOutput::new(vec![
            ActivityOperation::AddToSlot {
                slot: slot(1),
                delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(100)),
            },
        ])),
        _ => Err(ActivityHandlerFault::new(
            ActivityHandlerFaultKind::InvalidPayload,
        )),
    }
}

fn definition() -> Arc<GraphActivityDefinition> {
    let graph = ActivityGraphDefinition::new(
        node(1),
        vec![
            ActivityNodeDefinition::new(node(1), section(1), ActivityNodeKind::ExternalOutcome, 1)
                .unwrap(),
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
                edge(1),
                node(1),
                node(2),
                ActivityEdgeCondition::OptionSelected,
                0,
                1,
            )
            .unwrap(),
        ],
        2,
    )
    .unwrap();
    let options = [
        SUCCESS_OUTCOME,
        HANDLER_FAULT_OUTCOME,
        OPERATION_FAULT_OUTCOME,
    ]
    .into_iter()
    .enumerate()
    .map(|(priority, value)| {
        ActivityOptionDefinition::new(
            option(value),
            priority as i32,
            ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true))),
            vec![ActivityOperation::Traverse(edge(1))],
        )
    })
    .collect::<Vec<_>>();
    let program = GraphActivityNodeProgram::new(
        node(1),
        ActivityProgramDefinition::new(
            program(1),
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::ExternalOutcome,
                options: options.into_boxed_slice(),
            }],
        )
        .unwrap(),
    );
    let state = ActivityStateDefinition::new(
        vec![
            ActivitySlotDefinition::new_with_policy(
                slot(1),
                ActivityScope::Activity,
                ActivityValue::BoundedInteger(0),
                Some((0, 10)),
                None,
                vec![SlotResetPoint::ActivityStart],
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
    let registration = ActivityHandlerRegistration::new(
        ActivityHandlerId::new(1).unwrap(),
        "test.random-interaction",
        "v1",
        [1; 32],
        "one-labeled-draw",
        "test",
        handler,
    );
    let bundle = ActivityHandlerBundle::new("test", "v1", vec![], vec![registration]).unwrap();
    let registry = ActivityHandlerRegistry::compose(vec![bundle]).unwrap();
    let bindings = [
        (SUCCESS_OUTCOME, 0),
        (HANDLER_FAULT_OUTCOME, 1),
        (OPERATION_FAULT_OUTCOME, 2),
    ]
    .into_iter()
    .map(|(outcome, payload)| {
        ActivityInteractionBinding::new(
            node(1),
            external(outcome),
            ActivityHandlerId::new(1).unwrap(),
            vec![payload],
            "test.interaction",
        )
        .unwrap()
        .with_random_policy(
            ActivityInteractionRandomPolicy::new(ActivityRngLabel::Occurrence, outcome as u16, 7)
                .unwrap(),
        )
    })
    .collect();
    Arc::new(
        GraphActivityDefinition::new(
            ActivityDefinitionIdentity::new(
                ActivityDefinitionId::new(1).unwrap(),
                ActivityDefinitionDigest::new([2; 32]).unwrap(),
                ActivityConfigDigest::new([3; 32]).unwrap(),
            ),
            graph,
            state,
            Arc::new(participants()),
            vec![program],
            None,
            ActivityRandomPolicies::default(),
        )
        .and_then(|definition| definition.with_interactions(registry, bindings))
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
        CombatantSpecDigest::new([4; 32]).unwrap(),
        BuildDigest::new([5; 32]).unwrap(),
        "test",
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

fn occurrence_draws(activity: &GraphActivity) -> u64 {
    activity
        .debug_view()
        .rng()
        .iter()
        .find(|stream| stream.label() == ActivityRngLabel::Occurrence)
        .expect("Occurrence RNG stream")
        .draw_count()
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

fn edge(value: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(value).unwrap()
}

fn program(value: u32) -> ActivityProgramId {
    ActivityProgramId::new(value).unwrap()
}

fn option(value: u64) -> ActivityOptionId {
    ActivityOptionId::new(value).unwrap()
}

fn external(value: u64) -> ActivityExternalOutcomeId {
    ActivityExternalOutcomeId::new(value).unwrap()
}
