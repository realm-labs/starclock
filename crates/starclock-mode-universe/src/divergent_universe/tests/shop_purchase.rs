//! Trusted isolated purchase fixtures, not original shop offers or full runs.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBaselineFixture, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseCurioRuntimeError, DivergentUniverseCurrencyKind,
    DivergentUniverseLogicalScopeKind,
    shop_purchase::{
        ShopItemId, ShopPurchaseError, ShopPurchaseRuntime, ShopReward, ShopStockItem,
    },
    state::{BLESSINGS_SLOT, CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT},
    tests::{battle_room::base, currency_balance, instance, reward_draws},
};
use starclock_activity::{
    ActivityCondition, ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId,
    ActivityExpression, ActivityGraphDefinition, ActivityMasterSeed, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRandomPolicies, ActivitySlotDefinition,
    ActivitySlotId, ActivityStateDefinition, ActivityStateHash, ActivityTerminalOutcome,
    ActivityTransactionEventKind, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityDefinition, GraphActivityNodeProgram, LogicalScopeAddress,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingId,
    divergent_universe_catalog::DivergentUniverseRunFamily,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
};

const RECEIPT: u64 = 0x2263_0001;
const PURCHASED: ActivitySlotId = ActivitySlotId::new(70).unwrap();
fn id(raw: u16) -> ShopItemId {
    ShopItemId::new(raw).unwrap()
}
fn curio(raw: u32) -> DivergentUniverseCurioStateId {
    DivergentUniverseCurioStateId::new(format!("divergent-universe.curio-state.{raw}")).unwrap()
}
fn item(raw: u16, reward: ShopReward, price: u64) -> ShopStockItem {
    ShopStockItem {
        id: id(raw),
        reward,
        price,
    }
}
fn stock(fixture: &DivergentUniverseBaselineFixture) -> Vec<ShopStockItem> {
    vec![
        item(
            1,
            ShopReward::Blessing(
                fixture.factory().blessing_runtime().unwrap().blessings()[0]
                    .id()
                    .clone(),
            ),
            50,
        ),
        item(2, ShopReward::Curio(curio(9043)), 70), // authored wax: one mandatory Blessing
        item(3, ShopReward::Curio(curio(9053)), 40), // 40% post-payment fragment grant
    ]
}
fn literal(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn value(activity: &GraphActivity, slot: ActivitySlotId) -> ActivityValue {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|entry| entry.id() == slot)
        .unwrap()
        .value()
        .clone()
}
fn mutate(activity: &mut GraphActivity, operations: Vec<ActivityOperation>) {
    activity
        .apply_boundary_program(
            activity.state_hash(),
            &ActivityProgramDefinition::new(ActivityProgramId::new(26310).unwrap(), operations)
                .unwrap(),
        )
        .unwrap();
}
fn buy(
    runtime: &ShopPurchaseRuntime,
    activity: &mut GraphActivity,
    item: ShopItemId,
) -> Result<(), ShopPurchaseError> {
    runtime
        .purchase_accepted(activity, activity.state_hash(), item)
        .map(|_| ())
}
fn fixture_activity(
    fixture: &DivergentUniverseBaselineFixture,
    family: DivergentUniverseRunFamily,
    runtime: &ShopPurchaseRuntime,
    override_slot: Option<ActivitySlotDefinition>,
) -> GraphActivity {
    let base = base(fixture, family);
    let nodes = [100, 101, 102, 103].map(|raw| NodeId::new(raw).unwrap());
    let section = SectionId::new(1).unwrap();
    let graph = ActivityGraphDefinition::new(
        nodes[0],
        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                ActivityNodeDefinition::new(
                    *node,
                    section,
                    if index == 3 {
                        ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed)
                    } else {
                        ActivityNodeKind::Choice
                    },
                    1,
                )
                .unwrap()
            })
            .collect(),
        (0..3)
            .map(|index| {
                ActivityEdgeDefinition::new(
                    ActivityEdgeId::new(nodes[index].get()).unwrap(),
                    nodes[index],
                    nodes[index + 1],
                    ActivityEdgeCondition::Always,
                    0,
                    1,
                )
                .unwrap()
            })
            .collect(),
        4,
    )
    .unwrap();
    let scopes = LogicalScopeDefinitions::new(
        base.definition()
            .state_definition()
            .logical_scopes()
            .classes()
            .to_vec(),
        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                LogicalScopeNodeBinding::new(
                    *node,
                    vec![
                        LogicalScopeAddress::new(
                            DivergentUniverseLogicalScopeKind::Run.class_id(),
                            1,
                        )
                        .unwrap(),
                        LogicalScopeAddress::new(
                            DivergentUniverseLogicalScopeKind::Plane.class_id(),
                            1,
                        )
                        .unwrap(),
                        LogicalScopeAddress::new(
                            DivergentUniverseLogicalScopeKind::Node.class_id(),
                            if index < 2 { 1 } else { 2 },
                        )
                        .unwrap(),
                    ],
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    let key = base
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    let programs = nodes[..3]
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let mut operations = if index == 0 {
                vec![ActivityOperation::SetCounter {
                    slot: CURRENCIES_SLOT,
                    key,
                    value: literal(250),
                }]
            } else {
                Vec::new()
            };
            operations.push(ActivityOperation::Offer {
                kind: ActivityDecisionKind::Choice,
                options: vec![ActivityOptionDefinition::new(
                    ActivityOptionId::new(1).unwrap(),
                    0,
                    ActivityCondition::Boolean(ActivityExpression::Literal(
                        ActivityValue::Boolean(true),
                    )),
                    vec![ActivityOperation::Traverse(
                        ActivityEdgeId::new(node.get()).unwrap(),
                    )],
                )]
                .into(),
            });
            GraphActivityNodeProgram::new(
                *node,
                ActivityProgramDefinition::new(
                    ActivityProgramId::new(node.get()).unwrap(),
                    operations,
                )
                .unwrap(),
            )
        })
        .collect();
    let mut slots = base.definition().state_definition().slots().to_vec();
    slots.push(override_slot.unwrap_or_else(|| runtime.slot_definition().clone()));
    let mut digest = CanonicalDigestBuilder::new();
    digest.update(
        b"du.test.shop.trusted-fixed-stock.two-menus-same-room.next-room.trusted-250-fragments",
    );
    digest.update(base.definition().identity().config_digest().bytes());
    digest.update(runtime.configuration_digest());
    digest.update(graph.digest().bytes());
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
    GraphActivity::start(
        definition,
        instance(26310),
        ActivityMasterSeed::from_u64(26310),
    )
    .unwrap()
    .into_activity()
}
fn move_next(activity: &mut GraphActivity) {
    let offer = activity.player_view().decision().unwrap().clone();
    activity
        .choose_option(activity.state_hash(), offer.id(), offer.options()[0].id())
        .unwrap();
}

#[test]
fn shop_purchase_stock_identity_is_canonical_and_rejects_invalid_owners_and_prices() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let factory = fixture.factory();
    let runtime = factory
        .shop_purchase_runtime(stock(&fixture), PURCHASED)
        .unwrap();
    let mut reversed = stock(&fixture);
    reversed.reverse();
    assert_eq!(
        runtime.configuration_digest(),
        factory
            .shop_purchase_runtime(reversed, PURCHASED)
            .unwrap()
            .configuration_digest()
    );
    for (items, slot) in [
        (Vec::new(), PURCHASED),
        (vec![stock(&fixture)[0].clone(); 65], PURCHASED),
        (stock(&fixture), ActivitySlotId::new(69).unwrap()),
        (
            vec![stock(&fixture)[0].clone(), stock(&fixture)[0].clone()],
            PURCHASED,
        ),
        (vec![item(1, ShopReward::Curio(curio(9043)), 0)], PURCHASED),
        (
            vec![
                item(1, ShopReward::Curio(curio(9043)), 1),
                item(2, ShopReward::Curio(curio(9043)), 1),
            ],
            PURCHASED,
        ),
        (
            vec![
                stock(&fixture)[0].clone(),
                ShopStockItem {
                    id: id(2),
                    ..stock(&fixture)[0].clone()
                },
            ],
            PURCHASED,
        ),
        (
            vec![item(1, ShopReward::Curio(curio(9043)), u64::MAX)],
            PURCHASED,
        ),
        (vec![item(1, ShopReward::Curio(curio(99999)), 1)], PURCHASED),
        (
            vec![item(
                1,
                ShopReward::Blessing(
                    DivergentUniverseBlessingId::new("divergent-universe.blessing.unknown")
                        .unwrap(),
                ),
                1,
            )],
            PURCHASED,
        ),
    ] {
        assert!(factory.shop_purchase_runtime(items, slot).is_err());
    }
    assert!(ShopItemId::new(0).is_err());
    assert!(ShopItemId::new(65).is_err());
    let curios = factory.curio_runtime().unwrap();
    for state in curios.states().iter().filter(|state| {
        state.curio().is_none()
            || state.evolution_owner().is_some()
            || state.category() == DivergentUniverseCurioCategory::Negative
    }) {
        assert!(
            factory
                .shop_purchase_runtime(
                    vec![item(1, ShopReward::Curio(state.id().clone()), 1)],
                    PURCHASED
                )
                .is_err()
        );
    }
    let mut changed = stock(&fixture);
    let pair = curios
        .states()
        .iter()
        .find_map(|first| {
            let owner = first.curio()?;
            if first.evolution_owner().is_some()
                || first.category() == DivergentUniverseCurioCategory::Negative
            {
                return None;
            }
            let second = curios.states().iter().find(|second| {
                second.id() != first.id()
                    && second.curio() == Some(owner)
                    && second.evolution_owner().is_none()
                    && second.category() != DivergentUniverseCurioCategory::Negative
            })?;
            Some([first.id().clone(), second.id().clone()])
        })
        .unwrap();
    assert!(
        factory
            .shop_purchase_runtime(
                vec![
                    item(1, ShopReward::Curio(pair[0].clone()), 1),
                    item(2, ShopReward::Curio(pair[1].clone()), 2),
                ],
                PURCHASED
            )
            .is_err()
    );
    changed[0].price += 1;
    assert_ne!(
        runtime.configuration_digest(),
        factory
            .shop_purchase_runtime(changed, PURCHASED)
            .unwrap()
            .configuration_digest()
    );
    assert_ne!(
        runtime.configuration_digest(),
        factory
            .shop_purchase_runtime(stock(&fixture), ActivitySlotId::new(71).unwrap())
            .unwrap()
            .configuration_digest()
    );
}

#[test]
fn shop_purchase_executes_payment_full_rewards_and_sold_out_state_in_both_families() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let fresh_fixture = DivergentUniverseBaselineFixture::production().unwrap();
    for family in [
        DivergentUniverseRunFamily::Ordinary,
        DivergentUniverseRunFamily::Cyclical,
    ] {
        let runtime = fixture
            .factory()
            .shop_purchase_runtime(stock(&fixture), PURCHASED)
            .unwrap();
        let fresh = fresh_fixture
            .factory()
            .shop_purchase_runtime(stock(&fresh_fixture), PURCHASED)
            .unwrap();
        let mut activity = fixture_activity(&fixture, family, &runtime, None);
        let mut reconstructed = fixture_activity(&fresh_fixture, family, &fresh, None);
        let wallet = base(&fixture, family)
            .economy()
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .key();
        let blessings = fixture.factory().blessing_runtime().unwrap();
        let before = reward_draws(&activity);
        let result = runtime
            .purchase_accepted(&mut activity, reconstructed.state_hash(), id(1))
            .unwrap();
        let payment = result.events().iter().position(|event| matches!(event.kind(), ActivityTransactionEventKind::CounterChanged { slot, .. } if *slot == CURRENCIES_SLOT)).unwrap();
        let reward = result.events().iter().position(|event| matches!(event.kind(), ActivityTransactionEventKind::SlotChanged(slot) if *slot == BLESSINGS_SLOT)).unwrap();
        let sold_out = result.events().iter().position(|event| matches!(event.kind(), ActivityTransactionEventKind::CounterChanged { slot, .. } if *slot == PURCHASED)).unwrap();
        let receipt = result.events().iter().position(|event| matches!(event.kind(), ActivityTransactionEventKind::CounterChanged { slot, .. } if *slot == SERVICE_RECEIPTS_SLOT)).unwrap();
        assert!(payment < reward && reward < sold_out && sold_out < receipt);
        assert_eq!(result.state_hash(), activity.state_hash());
        buy(&fresh, &mut reconstructed, id(1)).unwrap();
        assert_eq!(currency_balance(&activity, wallet), 200);
        assert_eq!(reward_draws(&activity), before);
        assert_eq!(blessings.owned(&activity).unwrap().len(), 1);
        assert_eq!(blessings.owned(&activity).unwrap()[0].level(), 1);
        for (selected, balance) in [(id(2), 130), (id(3), 126)] {
            buy(&runtime, &mut activity, selected).unwrap();
            buy(&fresh, &mut reconstructed, selected).unwrap();
            assert_eq!(currency_balance(&activity, wallet), balance);
            assert_eq!(
                activity.canonical_state_bytes(),
                reconstructed.canonical_state_bytes()
            );
        }
        assert_eq!(reward_draws(&activity), before + 1);
        assert_eq!(blessings.owned(&activity).unwrap().len(), 2);
        assert_eq!(
            fixture
                .factory()
                .curio_runtime()
                .unwrap()
                .owned(&activity)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            value(&activity, PURCHASED),
            ActivityValue::BoundedCounterMap(vec![(1, 1), (2, 1), (3, 1)].into())
        );
        let snapshot = activity.canonical_state_bytes();
        assert!(matches!(
            buy(&runtime, &mut activity, id(2)),
            Err(ShopPurchaseError::SoldOut)
        ));
        assert_eq!(activity.canonical_state_bytes(), snapshot);
        move_next(&mut activity);
        move_next(&mut reconstructed); // physical move within logical room
        assert_eq!(
            value(&activity, PURCHASED),
            ActivityValue::BoundedCounterMap(vec![(1, 1), (2, 1), (3, 1)].into())
        );
        move_next(&mut activity);
        move_next(&mut reconstructed); // new logical room
        assert_eq!(
            value(&activity, PURCHASED),
            ActivityValue::BoundedCounterMap(Box::new([]))
        );
        assert_eq!(
            value(&activity, SERVICE_RECEIPTS_SLOT),
            ActivityValue::BoundedCounterMap(vec![(RECEIPT, 3)].into())
        );
        let hash = activity.state_hash();
        fixture
            .factory()
            .curio_runtime()
            .unwrap()
            .destroy_accepted(&mut activity, hash, &curio(9043))
            .unwrap();
        let hash = reconstructed.state_hash();
        fresh_fixture
            .factory()
            .curio_runtime()
            .unwrap()
            .destroy_accepted(&mut reconstructed, hash, &curio(9043))
            .unwrap();
        assert!(matches!(
            buy(&runtime, &mut activity, id(2)),
            Err(ShopPurchaseError::Curio(
                DivergentUniverseCurioRuntimeError::AlreadyOwned
            ))
        ));
        assert_eq!(
            activity.canonical_state_bytes(),
            reconstructed.canonical_state_bytes()
        );
        move_next(&mut activity);
        let snapshot = activity.canonical_state_bytes();
        assert!(matches!(
            buy(&runtime, &mut activity, id(2)),
            Err(ShopPurchaseError::ActivityCompleted)
        ));
        assert_eq!(activity.canonical_state_bytes(), snapshot);
    }
}

#[test]
fn shop_purchase_stale_hidden_insufficient_and_late_reward_failure_are_atomic() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture
        .factory()
        .shop_purchase_runtime(stock(&fixture), PURCHASED)
        .unwrap();
    let mut activity = fixture_activity(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &runtime,
        None,
    );
    let snapshot = activity.canonical_state_bytes();
    assert!(matches!(
        runtime.purchase_accepted(
            &mut activity,
            ActivityStateHash::new([0x55; 32]).unwrap(),
            id(2)
        ),
        Err(ShopPurchaseError::Activity(
            GraphActivityCommandError::StaleStateHash
        ))
    ));
    assert!(matches!(
        buy(&runtime, &mut activity, id(64)),
        Err(ShopPurchaseError::UnknownItem)
    ));
    assert_eq!(activity.canonical_state_bytes(), snapshot);
    let wallet = base(&fixture, DivergentUniverseRunFamily::Ordinary)
        .economy()
        .currency(DivergentUniverseCurrencyKind::CosmicFragment)
        .key();
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: CURRENCIES_SLOT,
            key: wallet,
            value: literal(69),
        }],
    );
    let snapshot = activity.canonical_state_bytes();
    assert!(matches!(
        buy(&runtime, &mut activity, id(2)),
        Err(ShopPurchaseError::InsufficientFunds)
    ));
    assert_eq!(activity.canonical_state_bytes(), snapshot);
    mutate(
        &mut activity,
        vec![
            ActivityOperation::SetCounter {
                slot: CURRENCIES_SLOT,
                key: wallet,
                value: literal(70),
            },
            ActivityOperation::SetCounter {
                slot: SERVICE_RECEIPTS_SLOT,
                key: RECEIPT,
                value: literal(i64::MAX),
            },
        ],
    );
    let snapshot = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    for _ in 0..2 {
        assert!(buy(&runtime, &mut activity, id(2)).is_err());
        assert_eq!(activity.canonical_state_bytes(), snapshot);
        assert_eq!(reward_draws(&activity), draws);
    }
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: SERVICE_RECEIPTS_SLOT,
            values: Box::new([]),
        }],
    );
    buy(&runtime, &mut activity, id(2)).unwrap();
    assert_eq!(currency_balance(&activity, wallet), 0);
    assert_eq!(reward_draws(&activity), draws + 1);
}

#[test]
fn shop_purchase_wrong_slot_corrupt_stock_and_existing_reward_leave_bytes_unchanged() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture
        .factory()
        .shop_purchase_runtime(stock(&fixture), PURCHASED)
        .unwrap();
    let other = fixture
        .factory()
        .shop_purchase_runtime(stock(&fixture), ActivitySlotId::new(71).unwrap())
        .unwrap();
    let mut wrong = fixture_activity(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &runtime,
        Some(other.slot_definition().clone()),
    );
    let snapshot = wrong.canonical_state_bytes();
    assert!(matches!(
        buy(&runtime, &mut wrong, id(1)),
        Err(ShopPurchaseError::DefinitionMismatch)
    ));
    assert_eq!(wrong.canonical_state_bytes(), snapshot);
    let mut activity = fixture_activity(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &runtime,
        None,
    );
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounter {
            slot: PURCHASED,
            key: 64,
            value: literal(1),
        }],
    );
    let snapshot = activity.canonical_state_bytes();
    assert!(matches!(
        buy(&runtime, &mut activity, id(1)),
        Err(ShopPurchaseError::InvalidState)
    ));
    assert_eq!(activity.canonical_state_bytes(), snapshot);
    mutate(
        &mut activity,
        vec![ActivityOperation::SetCounterMap {
            slot: PURCHASED,
            values: Box::new([]),
        }],
    );
    let ShopReward::Blessing(blessing) = &runtime.items()[0].reward else {
        panic!("first stock")
    };
    let hash = activity.state_hash();
    fixture
        .factory()
        .blessing_runtime()
        .unwrap()
        .acquire_accepted_identity(&mut activity, hash, blessing)
        .unwrap();
    let snapshot = activity.canonical_state_bytes();
    assert!(matches!(
        buy(&runtime, &mut activity, id(1)),
        Err(ShopPurchaseError::Blessing(_))
    ));
    assert_eq!(activity.canonical_state_bytes(), snapshot);
}

#[test]
fn shop_purchase_pending_blessing_offer_cannot_skip_a_bought_curios_mandatory_reward() {
    let fixture = DivergentUniverseBaselineFixture::production().unwrap();
    let runtime = fixture
        .factory()
        .shop_purchase_runtime(stock(&fixture), PURCHASED)
        .unwrap();
    let mut activity = fixture_activity(
        &fixture,
        DivergentUniverseRunFamily::Ordinary,
        &runtime,
        None,
    );
    let blessings = fixture.factory().blessing_runtime().unwrap();
    let group = blessings
        .groups()
        .iter()
        .find(|group| !group.candidates().is_empty())
        .unwrap();
    let hash = activity.state_hash();
    blessings
        .begin_group_offer(&mut activity, hash, group.id())
        .unwrap();
    let snapshot = activity.canonical_state_bytes();
    let draws = reward_draws(&activity);
    assert!(matches!(
        buy(&runtime, &mut activity, id(1)),
        Err(ShopPurchaseError::Blessing(
            DivergentUniverseBlessingRuntimeError::OfferAlreadyActive
        ))
    ));
    for _ in 0..2 {
        assert!(matches!(
            buy(&runtime, &mut activity, id(2)),
            Err(ShopPurchaseError::Curio(_))
        ));
        assert_eq!(activity.canonical_state_bytes(), snapshot);
        assert_eq!(reward_draws(&activity), draws);
    }
}
