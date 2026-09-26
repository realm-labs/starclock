//! Bound public menu at a real reviewed source context; isolated fixture graph.
//! Initial funds/holdings and stock are trusted test policy, not full-run evidence.

#[path = "shop_room_authentication.rs"]
mod authentication;
#[path = "shop_room_availability.rs"]
mod availability;
#[path = "shop_profile.rs"]
mod profile;

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseCurrencyKind,
    DivergentUniverseLogicalScopeKind,
    domain_deck::DomainDeckSlots,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram, DomainRouteError},
    shop_purchase::{
        ShopItemId, ShopReward, ShopStockItem,
        room::{BoundShopRoom, CompiledShopRoom, LEAVE_SHOP, ShopRoomSlots},
    },
    state::{
        BLESSINGS_SLOT, CURIO_ACTIVATIONS_SLOT, CURIO_CHARGES_SLOT, CURIO_STATES_SLOT,
        CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT,
    },
    tests::{battle_room::base, currency_balance, instance, reward_draws, shop_purchase::stock},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivitySlotId,
    ActivityStateDefinition, ActivityStateHash, ActivityTerminalOutcome, ActivityValue,
    GraphActivity, GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram,
    LogicalScopeAddress, LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::DivergentUniverseCurioStateId,
    divergent_universe_domain_decks::DomainCardKind,
};

const ROOT: u32 = 2_000_000;
const RECEIPT: u64 = 0x2263_0001;
const SLOTS: ShopRoomSlots = ShopRoomSlots {
    purchased: slot(70),
    accepted: slot(71),
};
const DECK: DomainDeckSlots = DomainDeckSlots {
    draw: slot(66),
    discard: slot(67),
    selected: slot(68),
    accepted: slot(69),
};
const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).unwrap()
}
fn literal(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn yes() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}
fn program(node: NodeId, operations: Vec<ActivityOperation>) -> GraphActivityNodeProgram {
    GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(ActivityProgramId::new(node.get()).unwrap(), operations)
            .unwrap(),
    )
}
fn probe(context: &DomainRoomContext) -> Result<DomainRoomProgram, DomainRouteError> {
    let node = context.entry_node();
    Ok(DomainRoomProgram {
        exit_node: node,
        nodes: vec![
            ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, 1)
                .unwrap(),
        ],
        edges: Vec::new(),
        random_offers: Vec::new(),
        programs: vec![program(
            node,
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::Checkpoint,
                options: vec![ActivityOptionDefinition::new(
                    ActivityOptionId::new(1).unwrap(),
                    0,
                    yes(),
                    vec![ActivityOperation::Traverse(context.exit_edge())],
                )]
                .into(),
            }],
        )],
    })
}
#[derive(Default)]
struct Setup {
    budget: u64,
    all_blessings: bool,
    owned: Option<(DivergentUniverseCurioStateId, i64)>,
    receipt_overflow: bool,
    failing_exit: bool,
}
struct Fixture {
    room: CompiledShopRoom,
    definition: Arc<GraphActivityDefinition>,
    wallet: u64,
}
impl Fixture {
    fn start(&self) -> GraphActivity {
        GraphActivity::start(
            Arc::clone(&self.definition),
            instance(26311),
            ActivityMasterSeed::from_u64(26311),
        )
        .unwrap()
        .into_activity()
    }
    fn bound(&self) -> BoundShopRoom {
        self.room.bind(Arc::clone(&self.definition)).unwrap()
    }
}
fn compile(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    items: Vec<ShopStockItem>,
    setup: Setup,
) -> Fixture {
    let factory = fixture.factory();
    let base = base(fixture, family);
    let compiler = factory.shop_room_compiler(items, SLOTS).unwrap();
    let mut selected = None;
    let route = factory
        .compile_curio_domain_route(
            base.area(),
            &factory.decision_catalog().domain_decks()[0].key,
            3,
            DECK,
            |context| {
                if context.composition == DomainRoomComposition::Card(DomainCardKind::Shop) {
                    let room = compiler.compile(context).unwrap();
                    let fragment = room.fragment().clone();
                    if selected.is_none() {
                        selected = Some(room);
                    }
                    Ok(fragment)
                } else {
                    probe(context)
                }
            },
        )
        .unwrap();
    let room = selected.unwrap();
    let context = room.context();
    let root = NodeId::new(ROOT).unwrap();
    let enter = ActivityEdgeId::new(ROOT).unwrap();
    let next = context.successor();
    let mut nodes = room.fragment().nodes.clone();
    nodes.push(
        ActivityNodeDefinition::new(
            root,
            SectionId::new(1).unwrap(),
            ActivityNodeKind::Choice,
            1,
        )
        .unwrap(),
    );
    nodes.push(
        ActivityNodeDefinition::new(
            next,
            context.section,
            if setup.failing_exit {
                ActivityNodeKind::Choice
            } else {
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed)
            },
            1,
        )
        .unwrap(),
    );
    let mut edges = room.fragment().edges.clone();
    edges.push(
        ActivityEdgeDefinition::new(
            enter,
            root,
            context.entry_node(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .unwrap(),
    );
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
    if setup.failing_exit {
        let end = NodeId::new(ROOT + 1).unwrap();
        nodes.push(
            ActivityNodeDefinition::new(
                end,
                context.section,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
                1,
            )
            .unwrap(),
        );
        edges.push(
            ActivityEdgeDefinition::new(
                ActivityEdgeId::new(ROOT + 1).unwrap(),
                next,
                end,
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .unwrap(),
        );
    }
    let maximum = nodes.iter().map(|node| node.maximum_visits()).sum();
    let graph = ActivityGraphDefinition::new(root, nodes, edges, maximum).unwrap();
    let mut bindings = route
        .logical_scopes
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
    bindings.push(
        LogicalScopeNodeBinding::new(
            next,
            vec![
                LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                    .unwrap(),
                LogicalScopeAddress::new(
                    DivergentUniverseLogicalScopeKind::Plane.class_id(),
                    u64::from(context.plane_ordinal),
                )
                .unwrap(),
                LogicalScopeAddress::new(
                    DivergentUniverseLogicalScopeKind::Node.class_id(),
                    u64::from(context.position_ordinal) + 1,
                )
                .unwrap(),
            ],
        )
        .unwrap(),
    );
    if setup.failing_exit {
        bindings.push(
            LogicalScopeNodeBinding::new(
                NodeId::new(ROOT + 1).unwrap(),
                vec![
                    LogicalScopeAddress::new(DivergentUniverseLogicalScopeKind::Run.class_id(), 1)
                        .unwrap(),
                ],
            )
            .unwrap(),
        );
    }
    let scopes =
        LogicalScopeDefinitions::new(route.logical_scopes.classes().to_vec(), bindings).unwrap();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.extend_from_slice(room.slot_definitions());
    let wallet = base
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let mut initialize = vec![ActivityOperation::SetCounter {
        slot: CURRENCIES_SLOT,
        key: wallet,
        value: literal(i64::try_from(setup.budget).unwrap()),
    }];
    if setup.all_blessings {
        let blessings = factory.blessing_runtime().unwrap();
        let mut values = blessings
            .blessings()
            .iter()
            .map(|blessing| (blessing.state_key(), 1))
            .collect::<Vec<_>>();
        values.sort_unstable();
        initialize.push(ActivityOperation::SetCounterMap {
            slot: BLESSINGS_SLOT,
            values: values.into(),
        });
    }
    if let Some((id, status)) = &setup.owned {
        let state = factory
            .curio_runtime()
            .unwrap()
            .state(id)
            .unwrap()
            .state_key();
        for (slot, amount) in [
            (CURIO_STATES_SLOT, *status),
            (CURIO_CHARGES_SLOT, 0),
            (CURIO_ACTIVATIONS_SLOT, 0),
        ] {
            initialize.push(ActivityOperation::SetCounter {
                slot,
                key: state,
                value: literal(amount),
            });
        }
    }
    if setup.receipt_overflow {
        initialize.push(ActivityOperation::SetCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            value: literal(i64::MAX),
        });
    }
    initialize.push(ActivityOperation::Traverse(enter));
    let mut programs = room
        .fragment()
        .programs
        .iter()
        .map(|program| {
            if program.node() == context.entry_node() {
                route
                    .programs
                    .iter()
                    .find(|candidate| candidate.node() == program.node())
                    .unwrap()
                    .clone()
            } else {
                program.clone()
            }
        })
        .collect::<Vec<_>>();
    programs.push(program(root, initialize));
    if setup.failing_exit {
        programs.push(program(
            next,
            vec![ActivityOperation::Require(ActivityCondition::Not(
                Box::new(yes()),
            ))],
        ));
    }
    let mut digest = CanonicalDigestBuilder::new();
    digest.update(b"du.test.shop-room.isolated-source-context.trusted-funds-and-inventory.explicit-next-entry-failure");
    digest.update(base.definition().identity().config_digest().bytes());
    digest.update(room.configuration_digest());
    digest.update(graph.digest().bytes());
    digest.update(setup.budget.to_le_bytes());
    digest.update([
        u8::from(setup.all_blessings),
        u8::from(setup.receipt_overflow),
        u8::from(setup.failing_exit),
    ]);
    if let Some((id, status)) = setup.owned {
        let key = factory
            .curio_runtime()
            .unwrap()
            .state(&id)
            .unwrap()
            .state_key();
        digest.update([1]);
        digest.update(key.to_le_bytes());
        digest.update(status.to_le_bytes());
    } else {
        digest.update([0]);
    }
    let digest = digest.finalize();
    let definition = Arc::new(
        GraphActivityDefinition::new(
            ActivityDefinitionIdentity::new(
                base.definition().identity().id(),
                ActivityDefinitionDigest::new(digest).unwrap(),
                ActivityConfigDigest::new(digest).unwrap(),
            ),
            graph,
            ActivityStateDefinition::new(slots, Vec::new(), Vec::new())
                .unwrap()
                .with_logical_scopes(scopes),
            Arc::clone(base.definition().participants()),
            programs,
            None,
            ActivityRandomPolicies::new(Vec::new(), Vec::new()),
        )
        .unwrap(),
    );
    Fixture {
        room,
        definition,
        wallet,
    }
}
fn choose(
    room: &BoundShopRoom,
    activity: &mut GraphActivity,
    raw: u64,
) -> Result<(), GraphActivityCommandError> {
    let offer = activity.player_view().decision().unwrap().clone();
    room.choose(
        activity,
        activity.state_hash(),
        offer.id(),
        ActivityOptionId::new(raw).unwrap(),
    )
    .map(|_| ())
}
fn options(activity: &GraphActivity) -> Vec<u64> {
    activity
        .player_view()
        .decision()
        .unwrap()
        .options()
        .iter()
        .map(|option| option.id().get())
        .collect()
}
fn mutate(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    activity
        .apply_boundary_program(
            activity.state_hash(),
            &ActivityProgramDefinition::new(ActivityProgramId::new(26312).unwrap(), operations)
                .unwrap(),
        )
        .unwrap();
}

#[test]
fn shop_room_public_purchases_regenerate_finite_menus_and_reconstruct_both_families() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_source = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let fixture = compile(
            &source,
            family,
            stock(&source),
            Setup {
                budget: 250,
                ..Setup::default()
            },
        );
        let fresh = compile(
            &fresh_source,
            family,
            stock(&fresh_source),
            Setup {
                budget: 250,
                ..Setup::default()
            },
        );
        let room = fixture.bound();
        let other = fresh.bound();
        let mut activity = fixture.start();
        let mut reconstructed = fresh.start();
        assert!(room.offered(&activity));
        assert!(room.offered(&reconstructed));
        assert_eq!(options(&activity), [1, 2, 3, LEAVE_SHOP]);
        let draws = reward_draws(&activity);
        let before = activity.canonical_state_bytes();
        for _ in 0..2 {
            assert!(room.offered(&activity));
            assert_eq!(activity.canonical_state_bytes(), before);
        }
        for (id, balance, remaining) in [
            (1, 200, vec![2, 3, LEAVE_SHOP]),
            (2, 130, vec![3, LEAVE_SHOP]),
            (3, 126, vec![LEAVE_SHOP]),
        ] {
            choose(&room, &mut activity, id).unwrap();
            choose(&other, &mut reconstructed, id).unwrap();
            assert_eq!(currency_balance(&activity, fixture.wallet), balance);
            assert_eq!(options(&activity), remaining);
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
            assert!(room.offered(&activity));
        }
        assert_eq!(reward_draws(&activity), draws + 1);
        assert_eq!(
            source
                .factory()
                .curio_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            source
                .factory()
                .blessing_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .len(),
            2
        );
        choose(&room, &mut activity, LEAVE_SHOP).unwrap();
        choose(&other, &mut reconstructed, LEAVE_SHOP).unwrap();
        assert!(!room.offered(&activity));
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
fn shop_room_availability_filters_funds_holdings_and_mandatory_reward_exhaustion() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let wax = DivergentUniverseCurioStateId::new("divergent-universe.curio-state.9043").unwrap();
    for (setup, expected) in [
        (Setup::default(), vec![LEAVE_SHOP]),
        (
            Setup {
                budget: 49,
                ..Setup::default()
            },
            vec![3, LEAVE_SHOP],
        ),
        (
            Setup {
                budget: 250,
                all_blessings: true,
                ..Setup::default()
            },
            vec![3, LEAVE_SHOP],
        ),
        (
            Setup {
                budget: 250,
                owned: Some((wax.clone(), 1)),
                ..Setup::default()
            },
            vec![1, 3, LEAVE_SHOP],
        ),
        (
            Setup {
                budget: 250,
                owned: Some((wax.clone(), 2)),
                ..Setup::default()
            },
            vec![1, 3, LEAVE_SHOP],
        ),
    ] {
        let fixture = compile(
            &source,
            DivergentUniverseRunFamily::Ordinary,
            stock(&source),
            setup,
        );
        let mut activity = fixture.start();
        let room = fixture.bound();
        assert_eq!(options(&activity), expected);
        let before = activity.canonical_state_bytes();
        if !expected.contains(&2) {
            assert!(choose(&room, &mut activity, 2).is_err());
        }
        assert_eq!(activity.canonical_state_bytes(), before);
        choose(&room, &mut activity, LEAVE_SHOP).unwrap();
    }
}

#[test]
fn shop_room_stale_hidden_raw_and_foreign_choices_reject_without_rng_or_state_change() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fixture = compile(
        &source,
        DivergentUniverseRunFamily::Ordinary,
        stock(&source),
        Setup {
            budget: 250,
            ..Setup::default()
        },
    );
    let room = fixture.bound();
    let mut activity = fixture.start();
    let before = activity.canonical_state_bytes();
    let offer = activity.player_view().decision().unwrap().clone();
    for raw in [1, LEAVE_SHOP] {
        assert!(
            activity
                .choose_option(
                    activity.state_hash(),
                    offer.id(),
                    ActivityOptionId::new(raw).unwrap()
                )
                .is_err()
        );
        assert_eq!(activity.canonical_state_bytes(), before);
    }
    assert_eq!(
        room.choose(
            &mut activity,
            ActivityStateHash::new([0x66; 32]).unwrap(),
            offer.id(),
            ActivityOptionId::new(1).unwrap()
        )
        .map(|_| ()),
        Err(GraphActivityCommandError::StaleStateHash)
    );
    assert!(choose(&room, &mut activity, 64).is_err());
    assert_eq!(activity.canonical_state_bytes(), before);
    let foreign = compile(
        &source,
        DivergentUniverseRunFamily::Cyclical,
        stock(&source),
        Setup {
            budget: 250,
            ..Setup::default()
        },
    );
    assert!(!foreign.bound().offered(&activity));
    let expected = activity.state_hash();
    assert_eq!(
        foreign
            .bound()
            .choose(
                &mut activity,
                expected,
                offer.id(),
                ActivityOptionId::new(1).unwrap()
            )
            .map(|_| ()),
        Err(GraphActivityCommandError::DecisionNotOffered)
    );
}

#[test]
fn shop_room_receipt_failure_keeps_offer_and_next_entry_failure_restores_scope_and_stock() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let fixture = compile(
        &source,
        DivergentUniverseRunFamily::Ordinary,
        stock(&source),
        Setup {
            budget: 250,
            receipt_overflow: true,
            failing_exit: true,
            ..Setup::default()
        },
    );
    let room = fixture.bound();
    let mut activity = fixture.start();
    let before = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for _ in 0..2 {
        assert!(choose(&room, &mut activity, 2).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
        assert_eq!(reward_draws(&activity), draws);
    }
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: SERVICE_RECEIPTS_SLOT,
            values: Box::new([]),
        }],
    );
    choose(&room, &mut activity, 2).unwrap();
    assert_eq!(reward_draws(&activity), draws + 1);
    assert_eq!(currency_balance(&activity, fixture.wallet), 180);
    let before = activity.canonical_state_bytes();
    for _ in 0..2 {
        assert!(choose(&room, &mut activity, LEAVE_SHOP).is_err());
        assert_eq!(activity.canonical_state_bytes(), before);
        assert!(room.offered(&activity));
    }
}

#[test]
fn shop_room_all_sixty_four_items_are_buyable_once_and_leave_is_always_available() {
    let source = DivergentUniverseBaselineFixture::production().unwrap();
    let blessings = source.factory().blessing_runtime().unwrap();
    let items = blessings.blessings()[..64]
        .iter()
        .enumerate()
        .map(|(index, blessing)| ShopStockItem {
            id: ShopItemId::new(u16::try_from(index + 1).unwrap()).unwrap(),
            reward: ShopReward::Blessing(blessing.id().clone()),
            price: 1,
        })
        .collect();
    let fixture = compile(
        &source,
        DivergentUniverseRunFamily::Ordinary,
        items,
        Setup {
            budget: 250,
            ..Setup::default()
        },
    );
    let room = fixture.bound();
    let mut activity = fixture.start();
    let draws = reward_draws(&activity);
    for id in 1..=64 {
        assert_eq!(options(&activity).len(), usize::try_from(66 - id).unwrap());
        choose(&room, &mut activity, id).unwrap();
    }
    assert_eq!(options(&activity), [LEAVE_SHOP]);
    assert_eq!(reward_draws(&activity), draws);
    assert_eq!(currency_balance(&activity, fixture.wallet), 186);
    choose(&room, &mut activity, LEAVE_SHOP).unwrap();
}
