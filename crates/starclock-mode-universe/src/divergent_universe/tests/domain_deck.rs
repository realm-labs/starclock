use std::sync::Arc;

use super::{entry, instance};
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    domain_deck::{DomainCardId, DomainDeck, DomainDeckSlots},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityRandomOffer,
    ActivityRandomPolicies, ActivityRngLabel, ActivityScope, ActivitySlotDefinition,
    ActivitySlotId, ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility,
    ActivityTerminalOutcome, ActivityValue, GraphActivity, GraphActivityDefinition,
    GraphActivityNodeProgram, NodeId, SectionId, SlotCarryPolicy, SlotResetPoint,
};

fn slots() -> DomainDeckSlots {
    DomainDeckSlots {
        draw: ActivitySlotId::new(1).unwrap(),
        discard: ActivitySlotId::new(2).unwrap(),
        selected: ActivitySlotId::new(3).unwrap(),
        accepted: ActivitySlotId::new(4).unwrap(),
    }
}
fn deck() -> DomainDeck {
    DomainDeck::new(
        (1..=5).map(|id| DomainCardId::new(id).unwrap()).collect(),
        3,
        24_101,
        slots(),
    )
    .unwrap()
}

fn setup(deck: &DomainDeck, reject_after_prefix: bool) -> Arc<GraphActivityDefinition> {
    setup_with(
        deck,
        reject_after_prefix,
        false,
        deck.slot_definitions().unwrap(),
    )
}

fn setup_with(
    deck: &DomainDeck,
    reject_after_prefix: bool,
    fixed_rooms: bool,
    definitions: Vec<ActivitySlotDefinition>,
) -> Arc<GraphActivityDefinition> {
    let base = DivergentUniverseRuntimeFactory::production()
        .unwrap()
        .compile(entry("401", "3011"))
        .unwrap();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut programs = Vec::new();
    let mut offers = Vec::new();
    for round in 0..6 {
        let section = SectionId::new(round / 2 + 1).unwrap();
        let prepare = NodeId::new(2 * round + 1).unwrap();
        let draw = NodeId::new(2 * round + 2).unwrap();
        let next = NodeId::new(2 * round + 3).unwrap();
        let fixed = NodeId::new(14 + round).unwrap();
        let prepare_edge = ActivityEdgeId::new(2 * round + 1).unwrap();
        let choose_edge = ActivityEdgeId::new(2 * round + 2).unwrap();
        for node in [prepare, draw] {
            nodes.push(
                ActivityNodeDefinition::new(node, section, ActivityNodeKind::Choice, 1).unwrap(),
            );
        }
        edges.push(
            ActivityEdgeDefinition::new(
                prepare_edge,
                prepare,
                draw,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        );
        edges.push(
            ActivityEdgeDefinition::new(
                choose_edge,
                draw,
                if fixed_rooms { fixed } else { next },
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        );
        let mut offered = deck.offer_program(choose_edge);
        if reject_after_prefix {
            let ActivityOperation::Offer { options, .. } = &mut offered[0] else {
                panic!("sole offer")
            };
            *options = options
                .iter()
                .map(|option| {
                    let mut operations = option.operations().to_vec();
                    operations.insert(
                        1,
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Literal(ActivityValue::Boolean(false)),
                        )),
                    );
                    ActivityOptionDefinition::new(
                        option.id(),
                        option.priority(),
                        option.enabled().clone(),
                        operations,
                    )
                })
                .collect();
        }
        programs.push(GraphActivityNodeProgram::new(
            prepare,
            ActivityProgramDefinition::new(
                ActivityProgramId::new(prepare.get()).unwrap(),
                deck.prepare_program(prepare_edge),
            )
            .unwrap(),
        ));
        programs.push(GraphActivityNodeProgram::new(
            draw,
            ActivityProgramDefinition::new(ActivityProgramId::new(draw.get()).unwrap(), offered)
                .unwrap(),
        ));
        offers.push(deck.random_offer(draw).unwrap());
        if fixed_rooms {
            nodes.push(
                ActivityNodeDefinition::new(fixed, section, ActivityNodeKind::Choice, 1).unwrap(),
            );
            let edge = ActivityEdgeId::new(100 + round).unwrap();
            edges.push(
                ActivityEdgeDefinition::new(edge, fixed, next, ActivityEdgeCondition::Always, 0, 1)
                    .unwrap(),
            );
            let option = ActivityOptionId::new(100).unwrap();
            programs.push(GraphActivityNodeProgram::new(
                fixed,
                ActivityProgramDefinition::new(
                    ActivityProgramId::new(fixed.get()).unwrap(),
                    vec![ActivityOperation::Offer {
                        kind: ActivityDecisionKind::Reward,
                        options: vec![ActivityOptionDefinition::new(
                            option,
                            0,
                            ActivityCondition::Boolean(ActivityExpression::Literal(
                                ActivityValue::Boolean(true),
                            )),
                            vec![ActivityOperation::Traverse(edge)],
                        )]
                        .into_boxed_slice(),
                    }],
                )
                .unwrap(),
            ));
            offers.push(
                ActivityRandomOffer::new(
                    fixed,
                    ActivityRngLabel::Reward,
                    24_102,
                    1,
                    vec![(option, 1)],
                    None,
                )
                .unwrap(),
            );
        }
    }
    nodes.push(
        ActivityNodeDefinition::new(
            NodeId::new(13).unwrap(),
            SectionId::new(3).unwrap(),
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .unwrap(),
    );
    Arc::new(
        GraphActivityDefinition::new(
            base.definition().identity(),
            ActivityGraphDefinition::new(
                NodeId::new(1).unwrap(),
                nodes,
                edges,
                if fixed_rooms { 19 } else { 13 },
            )
            .unwrap(),
            ActivityStateDefinition::new(definitions, Vec::new(), Vec::new()).unwrap(),
            Arc::clone(base.definition().participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), offers),
        )
        .unwrap(),
    )
}

fn start(definition: Arc<GraphActivityDefinition>, seed: u64) -> GraphActivity {
    GraphActivity::start(
        definition,
        instance(24_101),
        ActivityMasterSeed::from_u64(seed),
    )
    .unwrap()
    .into_activity()
}
fn pile(activity: &GraphActivity, slot: ActivitySlotId) -> Vec<u64> {
    let view = activity.player_view();
    let ActivityValue::OrderedIdSet(values) = view
        .slots()
        .iter()
        .find(|value| value.id() == slot)
        .unwrap()
        .value()
    else {
        panic!("set slot")
    };
    values.to_vec()
}

#[test]
fn domain_deck_discards_the_whole_offer_preserves_short_hands_and_refills_across_sections() {
    let deck = deck();
    let definition = setup(&deck, false);
    for seed in [0, 1, 24_101, u64::MAX] {
        let mut activity = start(Arc::clone(&definition), seed);
        let mut replay = start(Arc::clone(&definition), seed);
        let mut pair = Vec::new();
        for round in 0..6 {
            let decision = activity.player_view().decision().unwrap().clone();
            assert_eq!(decision.options().len(), if round % 2 == 0 { 3 } else { 2 });
            let hand = decision
                .options()
                .iter()
                .map(|option| option.id().get())
                .collect::<Vec<_>>();
            let observation = deck.observe(&activity).unwrap();
            let before_observation = activity.canonical_state_bytes();
            assert_eq!(deck.observe(&activity).unwrap(), observation);
            assert_eq!(activity.canonical_state_bytes(), before_observation);
            assert_eq!(
                observation
                    .hand
                    .iter()
                    .map(|card| card.get())
                    .collect::<Vec<_>>(),
                hand
            );
            assert_eq!(observation.draw.len(), if round % 2 == 0 { 2 } else { 0 });
            assert_eq!(
                observation.draw.len() + observation.hand.len() + observation.discard.len(),
                5
            );
            assert!(hand.iter().all(|card| !pair.contains(card)));
            pair.extend(&hand);
            let selected = decision.options()[0].id();
            let hash = activity.state_hash();
            deck.choose(&mut activity, hash, decision.id(), selected)
                .unwrap();
            let replay_hash = replay.state_hash();
            let replay_decision = replay.player_view().decision().unwrap().id();
            deck.choose(&mut replay, replay_hash, replay_decision, selected)
                .unwrap();
            assert_eq!(
                activity.canonical_state_bytes(),
                replay.canonical_state_bytes()
            );
            if round % 2 == 0 {
                let mut expected = hand;
                expected.sort_unstable();
                assert_eq!(pile(&activity, slots().discard), expected);
                assert_eq!(pile(&activity, slots().draw).len(), 2);
            } else {
                pair.sort_unstable();
                assert_eq!(pair, vec![1, 2, 3, 4, 5]);
                pair.clear();
                if round < 5 {
                    assert!(pile(&activity, slots().discard).is_empty());
                    assert_eq!(pile(&activity, slots().draw), vec![1, 2, 3, 4, 5]);
                }
            }
        }
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }
}

#[test]
fn domain_deck_rejected_stale_unoffered_raw_and_late_commands_preserve_piles_offer_and_rng() {
    let deck = deck();
    for late in [false, true] {
        let mut activity = start(setup(&deck, late), 7);
        let before = activity.canonical_state_bytes();
        let decision = activity.player_view().decision().unwrap().clone();
        let selected = decision.options()[0].id();
        let hash = activity.state_hash();
        assert!(
            activity
                .choose_option(hash, decision.id(), selected)
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert!(
            deck.choose(
                &mut activity,
                hash,
                decision.id(),
                ActivityOptionId::new(99).unwrap()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        if late {
            assert!(
                deck.choose(&mut activity, hash, decision.id(), selected)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), before);
        } else {
            deck.choose(&mut activity, hash, decision.id(), selected)
                .unwrap();
            let committed = activity.canonical_state_bytes();
            assert!(
                deck.choose(&mut activity, hash, decision.id(), selected)
                    .is_err()
            );
            assert_eq!(activity.canonical_state_bytes(), committed);
        }
    }
}

#[test]
fn domain_deck_invalid_definitions_and_foreign_random_policy_reject() {
    assert!(DomainCardId::new(0).is_none());
    for cards in [vec![], vec![DomainCardId::new(1).unwrap(); 2]] {
        assert!(DomainDeck::new(cards, 3, 24_101, slots()).is_err());
    }
    for width in [0, 6] {
        assert!(
            DomainDeck::new(vec![DomainCardId::new(1).unwrap()], width, 24_101, slots()).is_err()
        );
    }
    let mut aliases = slots();
    aliases.discard = aliases.draw;
    assert!(DomainDeck::new(vec![DomainCardId::new(1).unwrap()], 3, 24_101, aliases).is_err());
    let mut activity = start(setup(&deck(), false), 9);
    let different = DomainDeck::new(
        (1..=5).map(|id| DomainCardId::new(id).unwrap()).collect(),
        2,
        24_101,
        slots(),
    )
    .unwrap();
    let before = activity.canonical_state_bytes();
    let view = activity.player_view();
    let decision = view.decision().unwrap();
    let hash = activity.state_hash();
    assert!(
        different
            .choose(
                &mut activity,
                hash,
                decision.id(),
                decision.options()[0].id()
            )
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert!(different.observe(&activity).is_err());
    assert!(DomainDeck::new(vec![DomainCardId::new(1).unwrap()], 1, 0, slots()).is_err());
    assert!(
        DomainDeck::new(
            (1..=257).map(|id| DomainCardId::new(id).unwrap()).collect(),
            3,
            24_101,
            slots()
        )
        .is_err()
    );
}

#[test]
fn domain_deck_fixed_room_rewards_preserve_piles_and_do_not_draw_domain_rng() {
    let deck = deck();
    let mut fixed = start(
        setup_with(&deck, false, true, deck.slot_definitions().unwrap()),
        42,
    );
    for round in 0..6 {
        let draw_rng = fixed
            .debug_view()
            .rng()
            .iter()
            .find(|stream| stream.label() == ActivityRngLabel::Graph)
            .copied()
            .unwrap();
        let offered = fixed.player_view().decision().unwrap().clone();
        let selected = offered.options()[0].id();
        let hash = fixed.state_hash();
        deck.choose(&mut fixed, hash, offered.id(), selected)
            .unwrap();
        assert_eq!(
            fixed
                .debug_view()
                .rng()
                .iter()
                .find(|stream| stream.label() == ActivityRngLabel::Graph)
                .copied()
                .unwrap(),
            draw_rng
        );
        let observation = deck.observe(&fixed).unwrap();
        assert!(observation.hand.is_empty());
        assert_eq!(observation.draw.len() + observation.discard.len(), 5);
        assert_eq!(observation.selected.unwrap().get(), selected.get());
        assert_eq!(observation.draw.len(), if round % 2 == 0 { 2 } else { 0 });
        assert_eq!(
            observation.discard.len(),
            if round % 2 == 0 { 3 } else { 5 }
        );
        let before = fixed.canonical_state_bytes();
        let reward = fixed.player_view().decision().unwrap().clone();
        let hash = fixed.state_hash();
        assert!(
            deck.choose(&mut fixed, hash, reward.id(), reward.options()[0].id())
                .is_err()
        );
        assert_eq!(fixed.canonical_state_bytes(), before);
        fixed
            .choose_option(hash, reward.id(), reward.options()[0].id())
            .unwrap();
        let next = deck.observe(&fixed).unwrap();
        assert_eq!(next.draw.len() + next.hand.len() + next.discard.len(), 5);
        assert_eq!(next.selected, observation.selected);
        if round < 5 {
            assert_eq!(next.hand.len(), if round % 2 == 0 { 2 } else { 3 });
        }
    }
    assert_eq!(
        fixed.player_view().terminal(),
        Some(ActivityTerminalOutcome::Completed)
    );
}

#[test]
fn domain_deck_wrong_acceptance_scope_rejects_before_mutation() {
    let deck = deck();
    let mut definitions = deck.slot_definitions().unwrap();
    definitions[3] = ActivitySlotDefinition::new_with_policy(
        slots().accepted,
        ActivityScope::Activity,
        ActivityValue::Boolean(false),
        None,
        None,
        vec![SlotResetPoint::ActivityStart],
        SlotCarryPolicy::CarryExact,
        ActivityStateVisibility::Player,
        ActivityStateSource::new(u64::from(slots().accepted.get())).unwrap(),
    )
    .unwrap();
    let mut activity = start(setup_with(&deck, false, false, definitions), 7);
    let before = activity.canonical_state_bytes();
    let offered = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert!(deck.observe(&activity).is_err());
    assert!(
        deck.choose(&mut activity, hash, offered.id(), offered.options()[0].id())
            .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
}

#[test]
fn domain_deck_corrupt_partition_rejects_without_repairing_or_advancing() {
    let deck = deck();
    for (draw, discard) in [
        (vec![1, 2, 3, 4, 5], vec![1]),
        (vec![1, 2, 3, 4], vec![]),
        (vec![1, 2, 3, 4, 99], vec![]),
    ] {
        let mut activity = start(setup(&deck, false), 19);
        let hash = activity.state_hash();
        activity
            .apply_generated_boundary(hash, ActivityProgramId::new(100).unwrap(), |_| {
                Ok((
                    vec![
                        ActivityOperation::SetOrderedIdSet {
                            slot: slots().draw,
                            values: draw.into_boxed_slice(),
                        },
                        ActivityOperation::SetOrderedIdSet {
                            slot: slots().discard,
                            values: discard.into_boxed_slice(),
                        },
                    ],
                    (),
                ))
            })
            .unwrap();
        let before = activity.canonical_state_bytes();
        let decision = activity.player_view().decision().unwrap().clone();
        let hash = activity.state_hash();
        assert!(deck.observe(&activity).is_err());
        assert!(
            deck.choose(
                &mut activity,
                hash,
                decision.id(),
                decision.options()[0].id()
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
}

#[test]
fn authored_domain_decks_execute_real_sora_instances_with_reconstruction() {
    let factory = DivergentUniverseRuntimeFactory::production().unwrap();
    assert!(
        factory
            .compile_domain_deck("du.domain-deck.unknown", 3, slots())
            .is_err()
    );
    for authored in factory.decision_catalog().domain_decks() {
        let deck = factory
            .compile_domain_deck(&authored.key, 3, slots())
            .unwrap();
        let graph = setup(&deck, false);
        let mut activity = start(Arc::clone(&graph), 321);
        let mut replay = start(graph, 321);
        for _ in 0..6 {
            let offered = activity.player_view().decision().unwrap().clone();
            let observed = deck.observe(&activity).unwrap();
            assert_eq!(
                observed.draw.len() + observed.hand.len() + observed.discard.len(),
                authored.cards.len()
            );
            for card in observed.hand.iter() {
                assert!(
                    authored
                        .cards
                        .iter()
                        .any(|row| row.instance.get() == card.get())
                );
            }
            let selected = offered.options()[0].id();
            let hash = activity.state_hash();
            deck.choose(&mut activity, hash, offered.id(), selected)
                .unwrap();
            let hash = replay.state_hash();
            let decision = replay.player_view().decision().unwrap().id();
            deck.choose(&mut replay, hash, decision, selected).unwrap();
            assert_eq!(
                activity.canonical_state_bytes(),
                replay.canonical_state_bytes()
            );
            assert_eq!(
                deck.observe(&activity).unwrap().selected.unwrap().get(),
                selected.get()
            );
        }
    }
}
