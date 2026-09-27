//! Independent Leave must preserve settled reward and pending exit on failure.

use super::{FAMILIES, compile};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseLogicalScopeKind,
    adventure_room::{AdventureEarnedChests, LEAVE_ADVENTURE},
    tests::{currency_balance, instance, shop_room::program},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDefinitionDigest, ActivityDefinitionIdentity,
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression,
    ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionId, ActivityRandomPolicies, ActivitySlotId,
    ActivityTerminalOutcome, ActivityValue, GraphActivity, GraphActivityDefinition,
    LogicalScopeAddress, LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId,
};
use std::sync::Arc;

#[test]
fn adventure_leave_late_entry_rejection_restores_settled_count_credit_doors_offer_and_rng() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    let room = &profile.adventures[0];
    let context = room.context();
    let next = context.successor();
    let end = NodeId::new(9_000_000).unwrap();
    let tail = ActivityEdgeId::new(9_000_000).unwrap();
    let mut nodes = room.fragment().nodes.clone();
    nodes.push(
        ActivityNodeDefinition::new(next, context.section, ActivityNodeKind::Choice, 1).unwrap(),
    );
    nodes.push(
        ActivityNodeDefinition::new(
            end,
            context.section,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .unwrap(),
    );
    let mut edges = room.fragment().edges.clone();
    edges.push(
        ActivityEdgeDefinition::new(
            context.exit_edge(),
            room.menu_node(),
            next,
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
    edges.push(
        ActivityEdgeDefinition::new(tail, next, end, ActivityEdgeCondition::Always, 0, 1).unwrap(),
    );
    let graph = ActivityGraphDefinition::new(context.entry_node(), nodes, edges, 4).unwrap();
    let base = profile.flow.definition();
    let original_scopes = base.state_definition().logical_scopes();
    let mut bindings = original_scopes
        .bindings()
        .iter()
        .filter(|binding| {
            room.fragment()
                .nodes
                .iter()
                .any(|node| node.id() == binding.node())
        })
        .cloned()
        .collect::<Vec<_>>();
    for node in [next, end] {
        bindings.push(
            LogicalScopeNodeBinding::new(
                node,
                vec![
                    LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                        .unwrap(),
                ],
            )
            .unwrap(),
        );
    }
    let scopes =
        LogicalScopeDefinitions::new(original_scopes.classes().to_vec(), bindings).unwrap();
    let mut programs = base
        .programs()
        .iter()
        .filter(|record| {
            room.fragment()
                .nodes
                .iter()
                .any(|node| node.id() == record.node())
        })
        .cloned()
        .collect::<Vec<_>>();
    programs.push(program(
        next,
        vec![
            ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Literal(
                ActivityValue::Boolean(false),
            ))),
            ActivityOperation::Traverse(tail),
        ],
    ));
    let mut digest = CanonicalDigestBuilder::new();
    digest.update(b"du.test.adventure.isolated-source-context.failing-next-entry");
    digest.update(base.identity().config_digest().bytes());
    digest.update(room.configuration_digest());
    digest.update(graph.digest().bytes());
    let hash = digest.finalize();
    let identity = ActivityDefinitionIdentity::new(
        base.identity().id(),
        ActivityDefinitionDigest::new(hash).unwrap(),
        ActivityConfigDigest::new(hash).unwrap(),
    );
    let interactions = base.interactions().unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            graph,
            base.state_definition().clone().with_logical_scopes(scopes),
            Arc::clone(base.participants()),
            programs,
            None,
            ActivityRandomPolicies::default(),
        )
        .unwrap()
        .with_interactions(
            interactions.registry().as_ref().clone(),
            room.interaction_bindings().to_vec(),
        )
        .unwrap(),
    );
    let bound = room.bind(Arc::clone(&definition)).unwrap();
    let mut activity = GraphActivity::start(
        definition,
        instance(22650),
        ActivityMasterSeed::from_u64(22650),
    )
    .unwrap()
    .into_activity();
    let hash = activity.state_hash();
    let decision = activity.player_view().decision().unwrap().id();
    bound
        .submit(&mut activity, hash, decision, AdventureEarnedChests::Three)
        .unwrap();
    assert_eq!(currency_balance(&activity, profile.wallet), 300);
    let before = activity.canonical_state_bytes();
    let hash = activity.state_hash();
    let debug = activity.debug_view();
    let decision = activity.player_view().decision().unwrap().id();
    for _ in 0..3 {
        assert!(
            bound
                .leave(
                    &mut activity,
                    hash,
                    decision,
                    ActivityOptionId::new(LEAVE_ADVENTURE).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.state_hash(), hash);
        assert_eq!(activity.debug_view(), debug);
    }
}

#[test]
fn adventure_profile_rejects_empty_repeated_duplicate_or_wrong_slot_attachments() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0]);
    let factory = source.factory();
    assert!(
        factory
            .bind_position_adventure_rooms(profile.unbound.clone(), &[])
            .is_err()
    );
    assert!(
        factory
            .bind_position_adventure_rooms(profile.flow.clone(), &profile.adventures)
            .is_err()
    );
    let room = &profile.adventures[0];
    assert!(
        factory
            .bind_position_adventure_rooms(profile.unbound.clone(), &[room.clone(), room.clone()])
            .is_err()
    );
    let other = factory
        .adventure_room_compiler(ActivitySlotId::new(74).unwrap())
        .unwrap()
        .compile(room.context())
        .unwrap();
    assert_ne!(room.configuration_digest(), other.configuration_digest());
    assert!(
        factory
            .bind_position_adventure_rooms(profile.unbound.clone(), &[other])
            .is_err()
    );
}
