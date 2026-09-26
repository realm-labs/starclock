//! Actual source-card admission, explicit stock/funds and three real Boss proxies.
//! Other payloads are probes; this does not establish released complete-run parity.

#[path = "shop_profile_authentication.rs"]
mod authentication;

use std::sync::Arc;

use super::{DECK, LEAVE_SHOP, ROOT, SLOTS, literal, mutate, probe, program, stock};
use crate::baseline_controller::{
    ActivityBaselineHints, ActivityOptionHint, ActivityScoreComponents,
};
use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBaselinePolicy,
    DivergentUniverseBaselineRunner, DivergentUniverseBaselineStep, DivergentUniverseCurrencyKind,
    DivergentUniverseFlowInstance, DivergentUniverseLogicalScopeKind,
    battle_room::BattleRoomSelection,
    domain_route::DomainRoomComposition,
    shop_purchase::{ShopStockItem, room::CompiledShopRoom},
    state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT},
    tests::{battle_room::base, currency_balance, instance, reward_draws},
};
use starclock_activity::{
    ActivityConfigDigest, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionId, ActivityRandomPolicies, ActivityStateDefinition,
    ActivityTerminalOutcome, ActivityValue, GraphActivity, GraphActivityDefinition,
    LogicalScopeAddress, LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
};

const RECEIPT: u64 = 0x2263_0001;
const FAMILIES: [DivergentUniverseRunFamily; 2] = [
    DivergentUniverseRunFamily::Ordinary,
    DivergentUniverseRunFamily::Cyclical,
];
struct Profile {
    flow: DivergentUniverseFlowInstance,
    unbound: DivergentUniverseFlowInstance,
    rooms: Vec<CompiledShopRoom>,
    wallet: u64,
}

fn compile(
    source: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    budget: u64,
) -> Profile {
    compile_with_stock(source, family, budget, stock(source))
}

fn compile_with_stock(
    source: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    budget: u64,
    items: Vec<ShopStockItem>,
) -> Profile {
    let factory = source.factory();
    let base = base(source, family);
    let pool = factory.decision_catalog().encounter_pool();
    let battle = factory
        .battle_room_compiler(BattleRoomSelection {
            group: pool.encounter_group.clone(),
            stage: pool.candidate_stages[0].clone(),
            domain: BattleRewardDomain::Boss,
        })
        .unwrap();
    let shop = factory.shop_room_compiler(items, SLOTS).unwrap();
    let mut battles = Vec::new();
    let mut rooms = Vec::new();
    let deck = &factory.decision_catalog().domain_decks()[0].key;
    let mut route = factory
        .compile_curio_domain_route(base.area(), deck, 3, DECK, |context| {
            if context.composition == DomainRoomComposition::Fixed(FixedDomainKind::Boss) {
                let room = battle.compile(context).unwrap();
                let fragment = room.fragment().clone();
                battles.push(room);
                Ok(fragment)
            } else if context.composition == DomainRoomComposition::Card(DomainCardKind::Shop) {
                let room = shop.compile(context).unwrap();
                let fragment = room.fragment().clone();
                rooms.push(room);
                Ok(fragment)
            } else {
                probe(context)
            }
        })
        .unwrap();
    assert!(!rooms.is_empty());
    assert_eq!(battles.len(), 3);
    let root = NodeId::new(ROOT).unwrap();
    let edge = ActivityEdgeId::new(ROOT).unwrap();
    let mut nodes = route.graph.nodes().to_vec();
    nodes.push(
        ActivityNodeDefinition::new(
            root,
            SectionId::new(1).unwrap(),
            ActivityNodeKind::Choice,
            1,
        )
        .unwrap(),
    );
    let mut edges = route.graph.edges().to_vec();
    edges.push(
        ActivityEdgeDefinition::new(
            edge,
            root,
            route.graph.entry(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
    route.graph =
        ActivityGraphDefinition::new(root, nodes, edges, route.graph.maximum_total_visits() + 1)
            .unwrap();
    let mut bindings = route.logical_scopes.bindings().to_vec();
    bindings.push(
        LogicalScopeNodeBinding::new(
            root,
            vec![
                LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                    .unwrap(),
            ],
        )
        .unwrap(),
    );
    route.logical_scopes =
        LogicalScopeDefinitions::new(route.logical_scopes.classes().to_vec(), bindings).unwrap();
    let wallet = base
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    route.programs.push(program(
        root,
        vec![
            ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet,
                value: literal(i64::try_from(budget).unwrap()),
            },
            ActivityOperation::Traverse(edge),
        ],
    ));
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend(route.deck.slot_definitions().unwrap());
    slots.extend_from_slice(rooms[0].slot_definitions());
    let mut owner = CanonicalDigestBuilder::new();
    owner.update(b"du.test.shop-profile.reviewed-shop-cards.explicit-stock-and-funds.three-boss-proxies.other-payloads-probes");
    owner.update(u64::try_from(deck.len()).unwrap().to_le_bytes());
    owner.update(deck.as_bytes());
    owner.update(3_u16.to_le_bytes());
    owner.update(budget.to_le_bytes());
    for slot in [DECK.draw, DECK.discard, DECK.selected, DECK.accepted] {
        owner.update(slot.get().to_le_bytes());
    }
    owner.update(u32::try_from(rooms.len()).unwrap().to_le_bytes());
    for room in &rooms {
        owner.update(room.configuration_digest());
    }
    let payload = ActivityConfigDigest::new(owner.finalize()).unwrap();
    let identity = factory
        .battle_room_identity(&base, &route.graph, &battles, payload)
        .unwrap();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            identity,
            route.graph.clone(),
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(route.logical_scopes.clone()),
            Arc::clone(base.definition().participants()),
            route.programs.clone(),
            None,
            ActivityRandomPolicies::new(Vec::new(), route.random_offers.clone()),
        )
        .unwrap(),
    );
    let flow = factory
        .bind_battle_rooms(base, definition, &battles, payload)
        .unwrap();
    let flow = factory.bind_position_domain_route(flow, &route).unwrap();
    let unbound = flow.clone();
    let flow = factory.bind_position_shop_rooms(flow, &rooms).unwrap();
    Profile {
        flow,
        unbound,
        rooms,
        wallet,
    }
}

fn start(flow: &DivergentUniverseFlowInstance) -> GraphActivity {
    flow.start(instance(26314), ActivityMasterSeed::from_u64(26314))
        .unwrap()
        .into_activity()
}
fn policy(source: &DivergentUniverseBaselineFixture) -> DivergentUniverseBaselinePolicy {
    let original = source.policy().unwrap();
    // Prefer a Shop only when its actual card instance is in the sampled hand.
    let hints = source.factory().decision_catalog().domain_decks()[0]
        .cards
        .iter()
        .filter(|card| card.kind == DomainCardKind::Shop)
        .map(|card| {
            ActivityOptionHint::new(
                ActivityOptionId::new(card.instance.get()).unwrap(),
                ActivityScoreComponents::new(10_000, 0, 0, 0, 0).unwrap(),
            )
        })
        .collect();
    DivergentUniverseBaselinePolicy::new(
        ActivityBaselineHints::new(hints).unwrap(),
        original.encounter_group().clone(),
        original.encounter_stage(),
        128,
    )
    .unwrap()
}
fn advance(
    source: &DivergentUniverseBaselineFixture,
    flow: &DivergentUniverseFlowInstance,
    activity: &mut GraphActivity,
) -> DivergentUniverseBaselineStep {
    DivergentUniverseBaselineRunner::default()
        .advance(
            source.factory(),
            flow,
            activity,
            source.core(),
            &policy(source),
        )
        .unwrap()
}
fn ready(source: &DivergentUniverseBaselineFixture, profile: &Profile) -> GraphActivity {
    let mut activity = start(&profile.flow);
    for _ in 0..128 {
        if profile.flow.offered_shop(&activity) {
            return activity;
        }
        advance(source, &profile.flow, &mut activity);
    }
    panic!("the pinned source-card trace must visit a shop");
}
fn receipts(activity: &GraphActivity) -> i64 {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == SERVICE_RECEIPTS_SLOT)
        .unwrap()
        .value();
    let ActivityValue::BoundedCounterMap(values) = value else {
        panic!("receipt map");
    };
    values
        .iter()
        .find(|(key, _)| *key == RECEIPT)
        .map_or(0, |(_, amount)| *amount)
}

#[test]
fn shop_controller_reconstructs_both_families_after_actual_card_choices_and_three_boss_battles() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        let profile = compile(&source, family, 250);
        let fresh = compile(&fresh_source, family, 250);
        let mut activity = start(&profile.flow);
        let mut reconstructed = start(&fresh.flow);
        let mut purchases = Vec::new();
        let mut battles_at_first_purchase = None;
        let mut leaves = 0;
        for _ in 0..128 {
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            if activity.player_view().terminal().is_some() {
                break;
            }
            let shop = profile.flow.offered_shop(&activity);
            assert_eq!(shop, fresh.flow.offered_shop(&reconstructed));
            let before = activity.canonical_state_bytes();
            assert_eq!(profile.flow.offered_shop(&activity), shop);
            assert_eq!(activity.canonical_state_bytes(), before);
            let old_receipts = receipts(&activity);
            let old_balance = currency_balance(&activity, profile.wallet);
            let draws = reward_draws(&activity);
            if shop {
                let view = activity.player_view();
                let room = profile
                    .rooms
                    .iter()
                    .find(|room| room.menu_node() == view.current_node())
                    .unwrap();
                let deck = profile
                    .flow
                    .position_domain_deck(&activity)
                    .unwrap()
                    .unwrap();
                let selected = deck.selected.unwrap();
                let card = source.factory().decision_catalog().domain_decks()[0]
                    .cards
                    .iter()
                    .find(|card| card.instance.get() == selected.get())
                    .unwrap();
                assert_eq!(card.kind, DomainCardKind::Shop);
                assert_eq!(card.preset_source, room.context().preset_source);
                assert!(deck.hand.is_empty());
            }
            let step = advance(&source, &profile.flow, &mut activity);
            let fresh_step = advance(&fresh_source, &fresh.flow, &mut reconstructed);
            assert_eq!(step, fresh_step);
            if shop {
                let DivergentUniverseBaselineStep::ActivityDecision { decision, .. } = step else {
                    panic!("shop decision");
                };
                let selected = decision.option().get();
                if selected == LEAVE_SHOP {
                    leaves += 1;
                    assert_eq!(receipts(&activity), old_receipts);
                } else {
                    battles_at_first_purchase
                        .get_or_insert(activity.player_view().completed_battle_count());
                    purchases.push(selected);
                    assert_eq!(receipts(&activity), old_receipts + 1);
                    let expected = match selected {
                        1 => old_balance - 50,
                        2 => old_balance - 70,
                        3 => (old_balance - 40) + (old_balance - 40) * 40 / 100,
                        _ => panic!("only offered stock"),
                    };
                    assert_eq!(currency_balance(&activity, profile.wallet), expected);
                    assert_eq!(reward_draws(&activity), draws + u64::from(selected == 2));
                    assert!(profile.flow.offered_shop(&activity));
                }
            }
        }
        assert_eq!(purchases, [1, 2, 3]);
        assert!(leaves > 0);
        assert_eq!(receipts(&activity), 3);
        assert_eq!(activity.player_view().completed_battle_count(), 3);
        assert!(battles_at_first_purchase.is_some_and(|count| count < 3));
        assert_eq!(
            activity.player_view().terminal(),
            Some(ActivityTerminalOutcome::Completed)
        );
        assert_eq!(
            activity.canonical_state_bytes(),
            reconstructed.canonical_state_bytes()
        );
    }
}

#[test]
fn shop_controller_late_receipt_failure_restores_state_and_reward_rng() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let profile = compile(&source, FAMILIES[0], 250);
    let mut activity = ready(&source, &profile);
    advance(&source, &profile.flow, &mut activity);
    assert_eq!(receipts(&activity), 1);
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            value: literal(i64::MAX),
        }],
    );
    let before = activity.canonical_state_bytes();
    let balance = currency_balance(&activity, profile.wallet);
    let draws = reward_draws(&activity);
    for _ in 0..2 {
        assert!(
            DivergentUniverseBaselineRunner::default()
                .advance(
                    source.factory(),
                    &profile.flow,
                    &mut activity,
                    source.core(),
                    &policy(&source),
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
    }
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            value: literal(1),
        }],
    );
    advance(&source, &profile.flow, &mut activity);
    assert_eq!(receipts(&activity), 2);
    assert_eq!(currency_balance(&activity, profile.wallet), balance - 70);
    assert_eq!(reward_draws(&activity), draws + 1);
}

#[test]
fn shop_controller_unaffordable_stock_leaves_the_first_shop_in_both_families() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in FAMILIES {
        // Battles before the first sampled Shop can earn Fragments. Explicit
        // high test prices keep the current budget insufficient after earnings.
        let mut items = stock(&source);
        for item in &mut items {
            item.price = 10_000;
        }
        let empty = compile_with_stock(&source, family, 0, items);
        let mut activity = ready(&source, &empty);
        assert!(currency_balance(&activity, empty.wallet) < 10_000);
        let offer = activity.player_view().decision().unwrap().clone();
        assert_eq!(offer.options().len(), 1);
        assert_eq!(offer.options()[0].id().get(), LEAVE_SHOP);
        assert_eq!(receipts(&activity), 0);
        let step = advance(&source, &empty.flow, &mut activity);
        let DivergentUniverseBaselineStep::ActivityDecision { decision, .. } = step else {
            panic!("leave decision");
        };
        assert_eq!(decision.option().get(), LEAVE_SHOP);
        assert_eq!(receipts(&activity), 0);
        assert!(!empty.flow.offered_shop(&activity));
    }
}
