//! Domain-card draw/discard programs over the shared Activity transaction engine.
//!
//! This compiler does not admit a mask, initial deck, domain payload or room
//! selector. The caller must supply proven/policy-authored card instances and
//! bind these programs into the same authoritative Activity graph as the rooms.

use std::collections::BTreeSet;
use std::num::NonZeroU64;

#[path = "domain_deck_dispatch.rs"]
mod dispatch;

use super::DivergentUniverseRuntimeFactory;

use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId,
    ActivityExpression, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityPlayerView, ActivityRandomOffer, ActivityRngLabel, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateHash, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityRuntimeError, NodeId, SlotCarryPolicy, SlotResetPoint,
};

/// Stable instance identity. Copies of one domain definition have different IDs.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DomainCardId(NonZeroU64);

impl DomainCardId {
    #[must_use]
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

/// Four distinct slots allocated by the owning Activity profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainDeckSlots {
    pub draw: ActivitySlotId,
    pub discard: ActivitySlotId,
    pub selected: ActivitySlotId,
    pub accepted: ActivitySlotId,
}

/// Immutable base-deck compiler. All mutable piles reside in Activity slots;
/// the current hand is the shared engine's authenticated pending Route offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainDeck {
    cards: Box<[DomainCardId]>,
    initial_draw: Box<[u64]>,
    width: u16,
    purpose: u16,
    slots: DomainDeckSlots,
}

/// Player-visible piles. The authenticated hand is reserved and is not part of
/// `draw`, even though the internal unsettled partition still contains its IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainDeckObservation {
    pub draw: Box<[DomainCardId]>,
    pub hand: Box<[DomainCardId]>,
    pub discard: Box<[DomainCardId]>,
    pub selected: Option<DomainCardId>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainDeckError {
    UnknownAuthoredDeck,
    InvalidCards,
    InvalidWidth,
    InvalidSlots,
    InvalidRandomPolicy,
    InvalidDestinations,
}

impl std::fmt::Display for DomainDeckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid domain-deck definition: {self:?}")
    }
}
impl std::error::Error for DomainDeckError {}

impl DivergentUniverseRuntimeFactory {
    /// Compiles an explicitly selected Sora deck. Selection does not admit the
    /// source mask to a released offer pool or implement its special effects.
    /// The owning graph must bind this factory's decision digest and preserve
    /// the card-instance-to-room mapping in `decision_catalog().domain_decks()`.
    /// Width remains a caller-selected 1–5 policy; unknown keys fail closed.
    pub fn compile_domain_deck(
        &self,
        key: &str,
        width: u16,
        slots: DomainDeckSlots,
    ) -> Result<DomainDeck, DomainDeckError> {
        let definition = self
            .decision_catalog()
            .domain_decks()
            .iter()
            .find(|deck| deck.key.as_ref() == key)
            .ok_or(DomainDeckError::UnknownAuthoredDeck)?;
        DomainDeck::new(
            definition
                .cards
                .iter()
                .map(|card| {
                    DomainCardId::new(card.instance.get())
                        .expect("validated nonzero authored card instance")
                })
                .collect(),
            width,
            24_101,
            slots,
        )
    }
}

impl DomainDeck {
    /// Projects reserved cards out of the available draw pile without sampling
    /// or mutation. During fixed rooms there is no hand. Unknown IDs or broken
    /// partition conservation reject instead of fabricating an empty deck.
    pub fn observe(
        &self,
        activity: &GraphActivity,
    ) -> Result<DomainDeckObservation, GraphActivityCommandError> {
        self.validate_slots(activity)?;
        let view = activity.player_view();
        let mut draw = pile(&view, self.slots.draw)?;
        let discard = pile(&view, self.slots.discard)?;
        if !draw.is_disjoint(&discard)
            || draw.union(&discard).copied().collect::<Vec<_>>() != self.card_keys().as_ref()
        {
            return Err(invalid());
        }
        let policy = self
            .random_offer(activity.current_node())
            .map_err(|_| invalid())?;
        let current_policy = activity.definition().random_offers().iter().find(|offer| {
            offer.node() == activity.current_node()
                && offer.label() == ActivityRngLabel::Graph
                && offer.purpose() == self.purpose
        });
        if current_policy.is_some_and(|actual| actual != &policy) {
            return Err(invalid());
        }
        let hand = if current_policy.is_some() {
            let offered = view
                .decision()
                .filter(|d| d.kind() == ActivityDecisionKind::Route)
                .ok_or_else(invalid)?;
            if offered.options().len() != usize::from(self.width).min(draw.len()) {
                return Err(invalid());
            }
            offered
                .options()
                .iter()
                .map(|option| {
                    if !draw.remove(&option.id().get()) {
                        return Err(invalid());
                    }
                    DomainCardId::new(option.id().get()).ok_or_else(invalid)
                })
                .collect::<Result<Box<[_]>, _>>()?
        } else {
            Box::new([])
        };
        let selected = match view
            .slots()
            .iter()
            .find(|slot| slot.id() == self.slots.selected)
            .map(|slot| slot.value())
        {
            Some(ActivityValue::OptionalId(None)) => None,
            Some(ActivityValue::OptionalId(Some(id))) => {
                let card = DomainCardId::new(*id).ok_or_else(invalid)?;
                if self.cards.binary_search(&card).is_err() {
                    return Err(invalid());
                }
                Some(card)
            }
            _ => return Err(invalid()),
        };
        Ok(DomainDeckObservation {
            draw: draw
                .into_iter()
                .map(|id| DomainCardId::new(id).expect("validated card partition"))
                .collect(),
            hand,
            discard: discard
                .into_iter()
                .map(|id| DomainCardId::new(id).expect("validated card partition"))
                .collect(),
            selected,
        })
    }

    /// Compiles 1–256 distinct instances and a draw width of 1–5. Stable numeric
    /// order, uniform sampling without replacement and short final hands are
    /// explicit base policies; this does not claim original hidden RNG parity.
    pub fn new(
        mut cards: Vec<DomainCardId>,
        width: u16,
        purpose: u16,
        slots: DomainDeckSlots,
    ) -> Result<Self, DomainDeckError> {
        cards.sort_unstable();
        if cards.is_empty() || cards.len() > 256 || cards.windows(2).any(|p| p[0] == p[1]) {
            return Err(DomainDeckError::InvalidCards);
        }
        if !(1..=5).contains(&width) {
            return Err(DomainDeckError::InvalidWidth);
        }
        if purpose == 0 {
            return Err(DomainDeckError::InvalidRandomPolicy);
        }
        let mut ids = [slots.draw, slots.discard, slots.selected, slots.accepted];
        ids.sort_unstable();
        if ids.windows(2).any(|p| p[0] == p[1]) {
            return Err(DomainDeckError::InvalidSlots);
        }
        Ok(Self {
            initial_draw: cards.iter().map(|card| card.get()).collect(),
            cards: cards.into_boxed_slice(),
            width,
            purpose,
            slots,
        })
    }

    /// Declares run-carried draw/discard/selection slots and a node-reset guard.
    /// The caller must include these exact declarations in its state definition.
    pub fn slot_definitions(&self) -> Result<Vec<ActivitySlotDefinition>, DomainDeckError> {
        [
            (
                self.slots.draw,
                ActivityValue::OrderedIdSet(self.initial_draw.clone()),
                false,
                Some(256),
            ),
            (
                self.slots.discard,
                ActivityValue::OrderedIdSet(Box::new([])),
                false,
                Some(256),
            ),
            (
                self.slots.selected,
                ActivityValue::OptionalId(None),
                false,
                None,
            ),
            (
                self.slots.accepted,
                ActivityValue::Boolean(false),
                true,
                None,
            ),
        ]
        .into_iter()
        .map(|(id, value, node, limit)| {
            ActivitySlotDefinition::new_with_policy(
                id,
                if node {
                    ActivityScope::Node
                } else {
                    ActivityScope::Activity
                },
                value,
                None,
                limit,
                vec![if node {
                    SlotResetPoint::NodeStart
                } else {
                    SlotResetPoint::ActivityStart
                }],
                if node {
                    SlotCarryPolicy::Reset
                } else {
                    SlotCarryPolicy::CarryExact
                },
                ActivityStateVisibility::Player,
                ActivityStateSource::new(u64::from(id.get()))
                    .ok_or(DomainDeckError::InvalidSlots)?,
            )
            .map_err(|_| DomainDeckError::InvalidSlots)
        })
        .collect()
    }

    /// Automatic preparation must precede a separate draw node. Refill only an
    /// exhausted draw pile; never silently top up a short hand or restore cards
    /// removed by an unsupported mutation. Fixed rooms need not call this program.
    pub fn prepare_program(&self, next: ActivityEdgeId) -> Vec<ActivityOperation> {
        let mut complete = self
            .cards
            .iter()
            .map(|card| contains(self.slots.discard, card.get()))
            .collect::<Vec<_>>();
        complete.push(ActivityCondition::Equal(
            ActivityExpression::OrderedIdSetCount(self.slots.discard),
            integer(i64::try_from(self.cards.len()).expect("at most 256 cards")),
        ));
        vec![ActivityOperation::Conditional {
            condition: ActivityCondition::Equal(
                ActivityExpression::OrderedIdSetCount(self.slots.draw),
                integer(0),
            ),
            if_true: vec![
                ActivityOperation::Require(ActivityCondition::All(complete.into_boxed_slice())),
                ActivityOperation::SetOrderedIdSet {
                    slot: self.slots.draw,
                    values: self.card_keys(),
                },
                ActivityOperation::SetOrderedIdSet {
                    slot: self.slots.discard,
                    values: Box::new([]),
                },
                ActivityOperation::Traverse(next),
            ]
            .into_boxed_slice(),
            if_false: vec![ActivityOperation::Traverse(next)].into_boxed_slice(),
        }]
    }

    /// Builds the sole Offer at a draw node. Raw option execution cannot bypass
    /// discard settlement. The next edge enters the caller's actual room executor.
    pub fn offer_program(&self, next: ActivityEdgeId) -> Vec<ActivityOperation> {
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Route,
            options: self
                .cards
                .iter()
                .map(|card| self.card_option(*card, next))
                .collect(),
        }]
    }

    fn card_option(&self, card: DomainCardId, next: ActivityEdgeId) -> ActivityOptionDefinition {
        ActivityOptionDefinition::new(
            option(card),
            0,
            contains(self.slots.draw, card.get()),
            vec![
                ActivityOperation::Require(ActivityCondition::All(
                    vec![
                        ActivityCondition::Boolean(ActivityExpression::Slot(self.slots.accepted)),
                        ActivityCondition::Equal(
                            ActivityExpression::Slot(self.slots.selected),
                            ActivityExpression::Literal(ActivityValue::OptionalId(Some(
                                card.get(),
                            ))),
                        ),
                    ]
                    .into_boxed_slice(),
                )),
                ActivityOperation::Traverse(next),
            ],
        )
    }

    /// Shared Activity owns Graph RNG, offer persistence and failed-command rollback.
    pub fn random_offer(&self, node: NodeId) -> Result<ActivityRandomOffer, DomainDeckError> {
        ActivityRandomOffer::new(
            node,
            ActivityRngLabel::Graph,
            self.purpose,
            self.width,
            self.cards.iter().map(|card| (option(*card), 1)).collect(),
            None,
        )
        .map_err(|_| DomainDeckError::InvalidRandomPolicy)
    }

    /// Atomically settles every displayed card to discard, records the chosen
    /// instance, then executes its already-offered traversal. Rejection—including
    /// downstream graph failures—restores the exact offer, state, events and RNG.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        self.choose_with_generated_entry(activity, expected, decision, selected, |_, _, _| {
            Ok((Vec::new(), ()))
        })?;
        Ok(())
    }

    fn selection_operations(
        &self,
        view: &ActivityPlayerView,
        selected: ActivityOptionId,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let mut draw = pile(view, self.slots.draw)?;
        let mut discard = pile(view, self.slots.discard)?;
        if !draw.is_disjoint(&discard)
            || draw.union(&discard).copied().collect::<Vec<_>>() != self.card_keys().as_ref()
        {
            return Err(invalid());
        }
        let offered = view
            .decision()
            .filter(|d| d.kind() == ActivityDecisionKind::Route)
            .ok_or_else(invalid)?;
        if offered.options().is_empty()
            || offered.options().len() != usize::from(self.width).min(draw.len())
            || !offered.options().iter().any(|o| o.id() == selected)
        {
            return Err(invalid());
        }
        for card in offered.options() {
            if !draw.remove(&card.id().get()) || !discard.insert(card.id().get()) {
                return Err(invalid());
            }
        }
        Ok(vec![
            ActivityOperation::SetOrderedIdSet {
                slot: self.slots.draw,
                values: draw.into_iter().collect(),
            },
            ActivityOperation::SetOrderedIdSet {
                slot: self.slots.discard,
                values: discard.into_iter().collect(),
            },
            ActivityOperation::SetSlot {
                slot: self.slots.selected,
                value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(selected.get()))),
            },
            ActivityOperation::SetSlot {
                slot: self.slots.accepted,
                value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
            },
        ])
    }

    fn card_keys(&self) -> Box<[u64]> {
        self.cards.iter().map(|card| card.get()).collect()
    }

    /// An accepted entry choice initializes the partition in the shared graph.
    /// Until that choice, observation/selection correctly reject the empty deck.
    pub(super) fn deferred_initialization(mut self) -> Self {
        self.initial_draw = Box::new([]);
        self
    }

    fn validate_slots(&self, activity: &GraphActivity) -> Result<(), GraphActivityCommandError> {
        let expected = self.slot_definitions().map_err(|_| invalid())?;
        let actual = activity.definition().state_definition().slots();
        if expected.iter().any(|slot| !actual.contains(slot)) {
            return Err(invalid());
        }
        Ok(())
    }
}

fn pile(
    view: &ActivityPlayerView,
    id: ActivitySlotId,
) -> Result<BTreeSet<u64>, GraphActivityCommandError> {
    match view
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .map(|slot| slot.value())
    {
        Some(ActivityValue::OrderedIdSet(values)) => Ok(values.iter().copied().collect()),
        _ => Err(invalid()),
    }
}
fn option(card: DomainCardId) -> ActivityOptionId {
    ActivityOptionId::new(card.get()).expect("nonzero card instance")
}
fn contains(slot: ActivitySlotId, id: u64) -> ActivityCondition {
    ActivityCondition::OrderedIdSetContains { slot, id }
}
fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
