//! Dispatcher/transaction tests. Room nodes are test probes, not room gameplay.

use std::cell::Cell;
use std::collections::BTreeSet;
use std::sync::Arc;

use crate::divergent_universe::{
    DivergentUniverseBaselineFixture,
    domain_deck::{DomainCardId, DomainDeck, DomainDeckError, DomainDeckSlots},
    economy::DivergentUniverseCurrencyKind,
    tests::{currency_balance, instance},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies,
    ActivityRngLabel, ActivityScope, ActivitySlotDefinition, ActivitySlotId,
    ActivityStateDefinition, ActivityStateSource, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityValue, GraphActivity, GraphActivityDefinition, GraphActivityNodeProgram,
    LogicalScopeAddress, LogicalScopeClassDefinition, LogicalScopeClassId, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, SectionId, SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_domain_decks::{DomainCardKind, DomainDeckDefinition},
};

const ARRIVAL: ActivitySlotId = slot(104);
const SLOTS: DomainDeckSlots = DomainDeckSlots {
    draw: slot(100),
    discard: slot(101),
    selected: slot(102),
    accepted: slot(103),
};

fn kind_key(kind: DomainCardKind) -> u32 {
    match kind {
        DomainCardKind::Battle => 1,
        DomainCardKind::Elite => 2,
        DomainCardKind::Encounter => 3,
        DomainCardKind::Event => 4,
        DomainCardKind::Coin => 5,
        DomainCardKind::Shop => 6,
        DomainCardKind::Reward => 7,
        DomainCardKind::Adventure => 8,
        DomainCardKind::Reforge => 9,
    }
}

fn graph(
    fixture: &DivergentUniverseBaselineFixture,
    deck: &DomainDeck,
    authored: &DomainDeckDefinition,
    reject_room: bool,
) -> Arc<GraphActivityDefinition> {
    let base = fixture.flow(DivergentUniverseRunFamily::Ordinary).unwrap();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(deck.slot_definitions().unwrap());
    slots.push(
        ActivitySlotDefinition::new_with_policy(
            ARRIVAL,
            ActivityScope::Activity,
            ActivityValue::BoundedInteger(0),
            Some((0, 9)),
            None,
            vec![SlotResetPoint::ActivityStart],
            SlotCarryPolicy::CarryExact,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(104).unwrap(),
        )
        .unwrap(),
    );
    let kinds = authored
        .cards
        .iter()
        .map(|card| kind_key(card.kind))
        .collect::<BTreeSet<_>>();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut programs = Vec::new();
    let mut offers = Vec::new();
    let run = LogicalScopeClassId::new(1).unwrap();
    let plane = LogicalScopeClassId::new(2).unwrap();
    let domain = LogicalScopeClassId::new(3).unwrap();
    let run_address = LogicalScopeAddress::new(run, 1).unwrap();
    let mut bindings = Vec::new();
    for round in 0..6 {
        let section = SectionId::new(round / 2 + 1).unwrap();
        let prepare = node(2 * round + 1);
        let draw = node(2 * round + 2);
        let next = node(2 * round + 3);
        let enter_draw = edge(2 * round + 1);
        let path = vec![
            run_address,
            LogicalScopeAddress::new(plane, u64::from(round / 2 + 1)).unwrap(),
            LogicalScopeAddress::new(domain, u64::from(round % 2 + 1)).unwrap(),
        ];
        for physical in [prepare, draw] {
            bindings.push(LogicalScopeNodeBinding::new(physical, path.clone()).unwrap());
        }
        nodes.push(
            ActivityNodeDefinition::new(prepare, section, ActivityNodeKind::Choice, 1).unwrap(),
        );
        nodes
            .push(ActivityNodeDefinition::new(draw, section, ActivityNodeKind::Choice, 1).unwrap());
        edges.push(
            ActivityEdgeDefinition::new(
                enter_draw,
                prepare,
                draw,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        );
        programs.push(program(prepare, deck.prepare_program(enter_draw)));
        let destinations = authored
            .cards
            .iter()
            .map(|card| {
                (
                    DomainCardId::new(card.instance.get()).unwrap(),
                    edge(100 + round * 10 + kind_key(card.kind)),
                )
            })
            .collect::<Vec<_>>();
        programs.push(program(
            draw,
            deck.offer_program_by_card(&destinations).unwrap(),
        ));
        offers.push(deck.random_offer(draw).unwrap());
        for kind in kinds.iter().copied() {
            let room = node(100 + round * 10 + kind);
            let enter_room = edge(100 + round * 10 + kind);
            let leave_room = edge(500 + round * 10 + kind);
            bindings.push(LogicalScopeNodeBinding::new(room, path.clone()).unwrap());
            nodes.push(
                ActivityNodeDefinition::new(room, section, ActivityNodeKind::Choice, 1).unwrap(),
            );
            edges.push(
                ActivityEdgeDefinition::new(
                    enter_room,
                    draw,
                    room,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap(),
            );
            edges.push(
                ActivityEdgeDefinition::new(
                    leave_room,
                    room,
                    next,
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap(),
            );
            let mut operations = vec![ActivityOperation::SetSlot {
                slot: ARRIVAL,
                value: literal(ActivityValue::BoundedInteger(i64::from(kind))),
            }];
            if reject_room {
                operations.push(ActivityOperation::Require(condition(false)));
            }
            operations.push(ActivityOperation::Offer {
                kind: ActivityDecisionKind::Service,
                options: vec![ActivityOptionDefinition::new(
                    ActivityOptionId::new(1).unwrap(),
                    0,
                    condition(true),
                    vec![ActivityOperation::Traverse(leave_room)],
                )]
                .into_boxed_slice(),
            });
            programs.push(program(room, operations));
        }
    }
    nodes.push(
        ActivityNodeDefinition::new(
            node(13),
            SectionId::new(3).unwrap(),
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
            1,
        )
        .unwrap(),
    );
    bindings.push(LogicalScopeNodeBinding::new(node(13), vec![run_address]).unwrap());
    let scopes = LogicalScopeDefinitions::new(
        vec![
            LogicalScopeClassDefinition::new(run, None, 1).unwrap(),
            LogicalScopeClassDefinition::new(plane, Some(run), 3).unwrap(),
            LogicalScopeClassDefinition::new(domain, Some(plane), 6).unwrap(),
        ],
        bindings,
    )
    .unwrap();
    Arc::new(
        GraphActivityDefinition::new(
            base.definition().identity(),
            ActivityGraphDefinition::new(node(1), nodes, edges, 19).unwrap(),
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(scopes),
            Arc::clone(base.definition().participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), offers),
        )
        .unwrap(),
    )
}

fn program(node: NodeId, operations: Vec<ActivityOperation>) -> GraphActivityNodeProgram {
    GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(ActivityProgramId::new(node.get()).unwrap(), operations)
            .unwrap(),
    )
}
fn start(graph: Arc<GraphActivityDefinition>) -> GraphActivity {
    GraphActivity::start(graph, instance(24103), ActivityMasterSeed::from_u64(24103))
        .unwrap()
        .into_activity()
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
fn condition(value: bool) -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(value)))
}
const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn node(raw: u32) -> NodeId {
    NodeId::new(raw).unwrap()
}
fn edge(raw: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(raw).unwrap()
}

#[test]
fn domain_deck_dispatch_requires_exact_instance_bindings_and_canonicalizes_order() {
    let deck = DomainDeck::new(
        (1..=3).map(|id| DomainCardId::new(id).unwrap()).collect(),
        3,
        24103,
        SLOTS,
    )
    .unwrap();
    let destinations = (1..=3)
        .map(|id| (DomainCardId::new(id).unwrap(), edge(1)))
        .collect::<Vec<_>>();
    let expected = deck.offer_program_by_card(&destinations).unwrap();
    assert_eq!(expected, deck.offer_program(edge(1)));
    let mut reversed = destinations.clone();
    reversed.reverse();
    assert_eq!(deck.offer_program_by_card(&reversed).unwrap(), expected);
    for invalid in [
        vec![],
        destinations[..2].to_vec(),
        vec![destinations[0], destinations[0], destinations[2]],
        vec![
            destinations[0],
            destinations[1],
            (DomainCardId::new(99).unwrap(), edge(1)),
        ],
        vec![
            destinations[0],
            destinations[1],
            destinations[2],
            destinations[2],
        ],
    ] {
        assert_eq!(
            deck.offer_program_by_card(&invalid),
            Err(DomainDeckError::InvalidDestinations)
        );
    }
}

#[test]
fn domain_deck_dispatch_authored_decks_branch_and_commit_actual_curio_entry_effects() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let curios = factory.curio_runtime().unwrap();
    let grant = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9071").unwrap();
    let currency = fixture
        .flow(DivergentUniverseRunFamily::Ordinary)
        .unwrap()
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    for authored in factory.decision_catalog().domain_decks() {
        let deck = factory
            .compile_domain_deck(&authored.key, 3, SLOTS)
            .unwrap();
        let graph = graph(&fixture, &deck, authored, false);
        let mut activity = start(Arc::clone(&graph));
        let mut replay = start(graph);
        for run in [&mut activity, &mut replay] {
            curios
                .acquire_accepted_state(run, run.state_hash(), &grant)
                .unwrap();
        }
        let initial = currency_balance(&activity, currency);
        for round in 0..6 {
            let offered = activity.player_view().decision().unwrap().clone();
            let chosen =
                offered.options()[usize::try_from(round).unwrap() % offered.options().len()].id();
            let card = authored
                .cards
                .iter()
                .find(|card| card.instance.get() == chosen.get())
                .unwrap();
            let before = deck.observe(&activity).unwrap();
            let rng = activity.debug_view().rng().to_vec();
            for run in [&mut activity, &mut replay] {
                let hash = run.state_hash();
                let decision = run.player_view().decision().unwrap().id();
                let result = deck
                    .choose_with_generated_entry(
                        run,
                        hash,
                        decision,
                        chosen,
                        |view, _, selected| {
                            assert_eq!(selected.get(), card.instance.get());
                            Ok((
                                curios.domain_entry_operations(view).unwrap(),
                                card.instance.get(),
                            ))
                        },
                    )
                    .unwrap();
                assert_eq!(*result.value(), chosen.get());
            }
            assert_eq!(
                activity.current_node(),
                node(100 + round * 10 + kind_key(card.kind))
            );
            assert_eq!(
                activity.canonical_state_bytes(),
                replay.canonical_state_bytes()
            );
            assert_eq!(activity.debug_view().rng(), rng);
            let arrived = activity.player_view();
            assert_eq!(
                arrived
                    .slots()
                    .iter()
                    .find(|slot| slot.id() == ARRIVAL)
                    .unwrap()
                    .value(),
                &ActivityValue::BoundedInteger(i64::from(kind_key(card.kind)))
            );
            let after = deck.observe(&activity).unwrap();
            assert!(after.hand.is_empty());
            assert_eq!(after.draw.len() + after.discard.len(), authored.cards.len());
            assert_eq!(
                after.discard.len(),
                before.discard.len() + before.hand.len()
            );
            assert_eq!(after.selected.unwrap().get(), chosen.get());
            assert!(before.hand.iter().all(|card| after.discard.contains(card)));
            assert_eq!(
                currency_balance(&activity, currency),
                initial + 60 * i64::from((round + 1).min(3))
            );
            let owned = curios.owned(&activity).unwrap();
            let remaining = 3 - (round + 1).min(3);
            assert_eq!(
                owned
                    .iter()
                    .find(|held| held.state() == &grant)
                    .map(|held| held.charges()),
                (remaining > 0).then(|| u16::try_from(remaining).unwrap())
            );
            for run in [&mut activity, &mut replay] {
                let hash = run.state_hash();
                let offer = run.player_view().decision().unwrap().clone();
                run.choose_option(hash, offer.id(), offer.options()[0].id())
                    .unwrap();
            }
            assert_eq!(
                activity.canonical_state_bytes(),
                replay.canonical_state_bytes()
            );
        }
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
    }
}

#[test]
fn domain_deck_dispatch_late_failure_restores_curios_currency_hand_and_generated_rng() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let authored = &factory.decision_catalog().domain_decks()[0];
    let deck = factory
        .compile_domain_deck(&authored.key, 3, SLOTS)
        .unwrap();
    let mut activity = start(graph(&fixture, &deck, authored, true));
    let curios = factory.curio_runtime().unwrap();
    let grant = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9071").unwrap();
    let hash = activity.state_hash();
    curios
        .acquire_accepted_state(&mut activity, hash, &grant)
        .unwrap();
    let before = activity.canonical_state_bytes();
    let rng = activity.debug_view().rng().to_vec();
    let offered = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    assert!(
        deck.choose_with_generated_entry(
            &mut activity,
            hash,
            offered.id(),
            offered.options()[0].id(),
            |view, rng, _| {
                rng.choose_index(ActivityRngLabel::Reward, 24103, 5)
                    .unwrap();
                Ok((curios.domain_entry_operations(view).unwrap(), 99))
            }
        )
        .is_err()
    );
    assert_eq!(activity.canonical_state_bytes(), before);
    assert_eq!(activity.debug_view().rng(), rng);
}

#[test]
fn domain_deck_dispatch_rejects_invalid_commands_and_protected_or_boundary_entry_effects() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let authored = &fixture.factory().decision_catalog().domain_decks()[0];
    let deck = fixture
        .factory()
        .compile_domain_deck(&authored.key, 3, SLOTS)
        .unwrap();
    let mut activity = start(graph(&fixture, &deck, authored, false));
    let stale = activity.state_hash();
    let changed = ActivityProgramDefinition::new(
        ActivityProgramId::new(24104).unwrap(),
        vec![ActivityOperation::SetSlot {
            slot: ARRIVAL,
            value: literal(ActivityValue::BoundedInteger(1)),
        }],
    )
    .unwrap();
    activity.apply_boundary_program(stale, &changed).unwrap();
    let offered = activity.player_view().decision().unwrap().clone();
    let hash = activity.state_hash();
    let selected = offered.options()[0].id();
    let before = activity.canonical_state_bytes();
    let calls = Cell::new(0);
    assert!(
        activity
            .choose_option(hash, offered.id(), selected)
            .is_err()
    );
    for (expected, option) in [
        (hash, ActivityOptionId::new(999).unwrap()),
        (stale, selected),
    ] {
        assert!(
            deck.choose_with_generated_entry(
                &mut activity,
                expected,
                offered.id(),
                option,
                |_, _, _| {
                    calls.set(calls.get() + 1);
                    Ok((Vec::new(), ()))
                }
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    assert_eq!(calls.get(), 0);
    let mut invalid = [SLOTS.draw, SLOTS.discard, SLOTS.selected, SLOTS.accepted]
        .into_iter()
        .map(|slot| ActivityOperation::SetSlot {
            slot,
            value: literal(ActivityValue::Boolean(false)),
        })
        .collect::<Vec<_>>();
    invalid.extend([
        ActivityOperation::Traverse(edge(101)),
        ActivityOperation::Relocate(node(13)),
        ActivityOperation::Terminal(ActivityTerminalOutcome::Completed),
        ActivityOperation::Offer {
            kind: ActivityDecisionKind::Reward,
            options: vec![ActivityOptionDefinition::new(
                ActivityOptionId::new(1).unwrap(),
                0,
                condition(true),
                Vec::new(),
            )]
            .into_boxed_slice(),
        },
        ActivityOperation::Conditional {
            condition: condition(false),
            if_true: vec![ActivityOperation::InsertOrderedId {
                slot: SLOTS.discard,
                id: 99,
            }]
            .into_boxed_slice(),
            if_false: Box::new([]),
        },
    ]);
    for operation in invalid {
        let rng = activity.debug_view().rng().to_vec();
        assert!(
            deck.choose_with_generated_entry(
                &mut activity,
                hash,
                offered.id(),
                selected,
                |_, rng, _| {
                    rng.choose_index(ActivityRngLabel::Reward, 24103, 5)
                        .unwrap();
                    Ok((vec![operation], ()))
                }
            )
            .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(activity.debug_view().rng(), rng);
    }
}
