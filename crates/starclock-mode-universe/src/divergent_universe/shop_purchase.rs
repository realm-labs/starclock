//! Trusted fixed-stock purchases; not original shop admission or player offers.

#[path = "shop_room.rs"]
pub mod room;

use std::{collections::BTreeSet, num::NonZeroU16, slice::from_ref};

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseBlessingRuntime, DivergentUniverseBlessingRuntimeError,
    DivergentUniverseCurioRuntime, DivergentUniverseCurioRuntimeError,
    DivergentUniverseCurrencyKind, DivergentUniverseCurrencyRuntime, DivergentUniverseEconomyError,
    DivergentUniverseLogicalScopeKind, DivergentUniverseRuntimeFactory,
    economy::compile as compile_economy,
    state::{CURRENCIES_SLOT, SERVICE_RECEIPTS_SLOT},
};
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityPlayerView, ActivityProgramId,
    ActivityRngStreams, ActivityScope, ActivitySlotDefinition, ActivitySlotId, ActivityStateHash,
    ActivityStateSource, ActivityStateVisibility, ActivityTransactionEvent, ActivityValue,
    GraphActivity, GraphActivityCommandError, GraphActivityRuntimeError, SlotCarryPolicy,
    SlotResetPoint,
};
use starclock_data::{
    divergent_universe_blessing_catalog::DivergentUniverseBlessingId,
    divergent_universe_curio_catalog::{
        DivergentUniverseCurioCategory, DivergentUniverseCurioStateId,
    },
};

const LIMIT: u16 = 64;
const PROGRAM: u32 = 22_631;
const RECEIPT: u64 = 0x2263_0001;

/// Immutable owning-stock address, not an upstream numeric item locator.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ShopItemId(NonZeroU16);
impl ShopItemId {
    pub fn new(raw: u16) -> Result<Self, ShopPurchaseError> {
        if raw > LIMIT {
            return Err(ShopPurchaseError::InvalidStock);
        }
        NonZeroU16::new(raw)
            .map(Self)
            .ok_or(ShopPurchaseError::InvalidStock)
    }
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0.get()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShopReward {
    Blessing(DivergentUniverseBlessingId),
    Curio(DivergentUniverseCurioStateId),
}
/// Caller-selected current reward and positive fixed Cosmic Fragment price.
/// Admission, price, absence of discounts/refresh and base-level Blessings are
/// explicit replacement policy, never recovered original shop facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShopStockItem {
    pub id: ShopItemId,
    pub reward: ShopReward,
    pub price: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShopPurchaseAccuracy {
    VersionedProjectPolicyExplicitFixedStockAndPriceBaseRewardsPrePaymentEligibility,
}
#[derive(Debug)]
pub enum ShopPurchaseError {
    InvalidStock,
    InvalidState,
    DefinitionMismatch,
    UnknownItem,
    SoldOut,
    InsufficientFunds,
    ActivityCompleted,
    BattlePending,
    Economy(DivergentUniverseEconomyError),
    Curio(DivergentUniverseCurioRuntimeError),
    Blessing(DivergentUniverseBlessingRuntimeError),
    Activity(GraphActivityCommandError),
}
impl std::fmt::Display for ShopPurchaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Divergent Universe shop purchase: {self:?}")
    }
}
impl std::error::Error for ShopPurchaseError {}

/// Immutable selected inventory plus the exact current acquisition runtimes.
/// Mutable sold-out state belongs to the shared logical-room slot, not this value.
#[derive(Clone, Debug)]
pub struct ShopPurchaseRuntime {
    items: Box<[ShopStockItem]>,
    purchased: ActivitySlotDefinition,
    currency: DivergentUniverseCurrencyRuntime,
    blessings: DivergentUniverseBlessingRuntime,
    curios: DivergentUniverseCurioRuntime,
    component: [u8; 32],
    decisions: [u8; 32],
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShopPurchaseResolution {
    item: ShopItemId,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}
impl ShopPurchaseResolution {
    #[must_use]
    pub const fn item(&self) -> ShopItemId {
        self.item
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

impl DivergentUniverseRuntimeFactory {
    /// Compile 1..=64 explicitly selected, owner-unique ordinary rewards. The
    /// host slot is >=70 and logical-room scoped. Invalid/unbound/evolution-only
    /// or negative Curios, duplicate items/owners and invalid prices reject.
    /// Caller binds the returned digest into its profile; no NPC/pool is inferred.
    pub fn shop_purchase_runtime(
        &self,
        mut items: Vec<ShopStockItem>,
        purchased: ActivitySlotId,
    ) -> Result<ShopPurchaseRuntime, ShopPurchaseError> {
        if items.is_empty() || items.len() > usize::from(LIMIT) || purchased.get() < 70 {
            return Err(ShopPurchaseError::InvalidStock);
        }
        items.sort_by_key(|item| item.id);
        let blessings = self
            .blessing_runtime()
            .map_err(ShopPurchaseError::Blessing)?;
        let curios = self.curio_runtime().map_err(ShopPurchaseError::Curio)?;
        let mut blessing_ids = BTreeSet::new();
        let mut curio_owners = BTreeSet::new();
        let mut item_ids = BTreeSet::new();
        for item in &items {
            if !item_ids.insert(item.id) || item.price == 0 || i64::try_from(item.price).is_err() {
                return Err(ShopPurchaseError::InvalidStock);
            }
            match &item.reward {
                ShopReward::Blessing(id) => {
                    if !blessings
                        .blessings()
                        .iter()
                        .any(|blessing| blessing.id() == id)
                    {
                        return Err(ShopPurchaseError::Blessing(
                            DivergentUniverseBlessingRuntimeError::UnknownBlessing,
                        ));
                    }
                    if !blessing_ids.insert(id.clone()) {
                        return Err(ShopPurchaseError::InvalidStock);
                    }
                }
                ShopReward::Curio(id) => {
                    let state = curios.state(id).map_err(ShopPurchaseError::Curio)?;
                    let owner = state.curio().ok_or(ShopPurchaseError::InvalidStock)?;
                    if state.evolution_owner().is_some()
                        || state.category() == DivergentUniverseCurioCategory::Negative
                        || !curio_owners.insert(owner.clone())
                    {
                        return Err(ShopPurchaseError::InvalidStock);
                    }
                }
            }
        }
        let definition = ActivitySlotDefinition::new_with_policy(
            purchased,
            ActivityScope::Node,
            ActivityValue::BoundedCounterMap(Box::new([])),
            Some((1, 1)),
            Some(u32::from(LIMIT)),
            vec![SlotResetPoint::NodeStart],
            SlotCarryPolicy::Reset,
            ActivityStateVisibility::Player,
            ActivityStateSource::new(u64::from(purchased.get()))
                .ok_or(ShopPurchaseError::InvalidStock)?,
        )
        .map_err(|_| ShopPurchaseError::InvalidStock)?
        .with_logical_scope(DivergentUniverseLogicalScopeKind::Node.class_id());
        Ok(ShopPurchaseRuntime {
            items: items.into_boxed_slice(),
            purchased: definition,
            currency: compile_economy(
                self.bundle.service_catalog(),
                self.bundle.progression_catalog(),
                self.bundle.curio_catalog(),
                self.decision_catalog(),
            )
            .map_err(ShopPurchaseError::Economy)?
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .clone(),
            blessings,
            curios,
            component: self.bundle_identity().component_digest().bytes(),
            decisions: self.decision_catalog().digest(),
        })
    }
}
impl ShopPurchaseRuntime {
    #[must_use]
    pub fn items(&self) -> &[ShopStockItem] {
        &self.items
    }
    #[must_use]
    pub const fn slot_definition(&self) -> &ActivitySlotDefinition {
        &self.purchased
    }
    #[must_use]
    pub const fn accuracy(&self) -> ShopPurchaseAccuracy {
        ShopPurchaseAccuracy::VersionedProjectPolicyExplicitFixedStockAndPriceBaseRewardsPrePaymentEligibility
    }
    /// Exact current inputs, canonical stock, fixed-price/snapshot policy and host
    /// address. Owning profiles additionally bind placement and their whole graph.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut digest = CanonicalDigestBuilder::new();
        digest.update(b"starclock.du.shop.explicit-fixed-stock-positive-fragment-price.no-discount-no-refresh.one-per-logical-room.ordinary-curios.base-blessings.pre-payment-reward-eligibility.debit-before-acquisition.last-run-receipt");
        digest.update(self.component);
        digest.update(self.decisions);
        digest.update(self.purchased.id().get().to_le_bytes());
        digest.update(PROGRAM.to_le_bytes());
        digest.update(LIMIT.to_le_bytes());
        digest.update(RECEIPT.to_le_bytes());
        digest.update(
            u16::try_from(self.items.len())
                .expect("bounded stock")
                .to_le_bytes(),
        );
        for item in &self.items {
            digest.update(item.id.get().to_le_bytes());
            digest.update(item.price.to_le_bytes());
            let (tag, id) = match &item.reward {
                ShopReward::Blessing(id) => (0, id.as_str()),
                ShopReward::Curio(id) => (1, id.as_str()),
            };
            digest.update([tag]);
            digest.update(
                u64::try_from(id.len())
                    .expect("bounded content identity")
                    .to_le_bytes(),
            );
            digest.update(id.as_bytes());
        }
        digest.finalize()
    }
    /// Trusted owning-service purchase, not a public player action. The caller
    /// authenticates active shop/offer/profile and binds this stock's digest.
    /// This validates the exact host declaration, stock state, funds and current
    /// reward ownership. Payment, mandatory rewards, sold-out mark and Run receipt
    /// commit together; rejected/late-failing commands restore bytes/events/RNG.
    /// Eligibility/expansion planning sees the pre-payment view, while authored
    /// fragment-grant expressions execute after the debit and inventory insertion.
    pub fn purchase_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        item: ShopItemId,
    ) -> Result<ShopPurchaseResolution, ShopPurchaseError> {
        if expected != activity.state_hash() {
            return Err(ShopPurchaseError::Activity(
                GraphActivityCommandError::StaleStateHash,
            ));
        }
        if !activity
            .definition()
            .state_definition()
            .slots()
            .contains(&self.purchased)
        {
            return Err(ShopPurchaseError::DefinitionMismatch);
        }
        let view = activity.player_view();
        if view.terminal().is_some() {
            return Err(ShopPurchaseError::ActivityCompleted);
        }
        if view.pending_battle().is_some() {
            return Err(ShopPurchaseError::BattlePending);
        }
        let mut generation_error = None;
        let result = activity
            .apply_generated_boundary(
                expected,
                ActivityProgramId::new(PROGRAM).expect("nonzero mode-owned shop program"),
                |rng| {
                    self.purchase_operations(&view, item, rng)
                        .map(|ops| (ops, ()))
                        .map_err(|error| {
                            generation_error = Some(error);
                            GraphActivityCommandError::Runtime(
                                GraphActivityRuntimeError::InvalidBoundaryProgram,
                            )
                        })
                },
            )
            .map_err(|error| generation_error.unwrap_or(ShopPurchaseError::Activity(error)))?;
        Ok(ShopPurchaseResolution {
            item,
            events: result.events().into(),
            state_hash: activity.state_hash(),
        })
    }
    /// One state-only plan for the owning generated-choice boundary. It does not
    /// authorize selection, mutate live state or skip acquisition/expansion effects.
    pub(in crate::divergent_universe) fn purchase_operations(
        &self,
        view: &ActivityPlayerView,
        selected: ShopItemId,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, ShopPurchaseError> {
        if view.terminal().is_some() {
            return Err(ShopPurchaseError::ActivityCompleted);
        }
        if view.pending_battle().is_some() {
            return Err(ShopPurchaseError::BattlePending);
        }
        if !view.logical_scopes().iter().any(|scope| {
            scope.address().class() == DivergentUniverseLogicalScopeKind::Node.class_id()
        }) {
            return Err(ShopPurchaseError::InvalidState);
        }
        let item = self
            .items
            .iter()
            .find(|item| item.id == selected)
            .ok_or(ShopPurchaseError::UnknownItem)?;
        let purchased = counter_map(view, self.purchased.id())?;
        if purchased.iter().any(|(key, value)| {
            *value != 1
                || !self
                    .items
                    .iter()
                    .any(|item| u64::from(item.id.get()) == *key)
        }) {
            return Err(ShopPurchaseError::InvalidState);
        }
        if purchased
            .iter()
            .any(|(key, _)| *key == u64::from(selected.get()))
        {
            return Err(ShopPurchaseError::SoldOut);
        }
        let price = i64::try_from(item.price).map_err(|_| ShopPurchaseError::InvalidStock)?;
        let balance = counter_map(view, CURRENCIES_SLOT)?
            .iter()
            .find(|(key, _)| *key == self.currency.key())
            .map_or(0, |(_, balance)| *balance);
        if balance < price {
            return Err(ShopPurchaseError::InsufficientFunds);
        }
        let acquisition = match &item.reward {
            ShopReward::Blessing(id) => self
                .blessings
                .acquisition_operations(view, from_ref(id), rng)
                .map_err(ShopPurchaseError::Blessing)?,
            ShopReward::Curio(id) => self
                .curios
                .acquisition_operations(view, from_ref(id), rng)
                .map_err(ShopPurchaseError::Curio)?,
        };
        let mut operations = vec![
            self.currency
                .spend_operation(item.price)
                .map_err(|_| ShopPurchaseError::InvalidStock)?,
        ];
        operations.extend(acquisition);
        operations.push(ActivityOperation::SetCounter {
            slot: self.purchased.id(),
            key: u64::from(selected.get()),
            value: literal(1),
        });
        operations.push(ActivityOperation::AddCounter {
            slot: SERVICE_RECEIPTS_SLOT,
            key: RECEIPT,
            delta: literal(1),
        });
        Ok(operations)
    }
}
fn literal(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn counter_map(
    view: &ActivityPlayerView,
    id: ActivitySlotId,
) -> Result<&[(u64, i64)], ShopPurchaseError> {
    match view
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .map(|slot| slot.value())
    {
        Some(ActivityValue::BoundedCounterMap(values)) => Ok(values),
        _ => Err(ShopPurchaseError::InvalidState),
    }
}
