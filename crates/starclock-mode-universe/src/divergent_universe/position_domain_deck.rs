//! Immutable source-position hand dispatch, distinct from ordinary room exits.

use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    battle_room::BattleRoomError,
    domain_deck::DomainDeckObservation,
    domain_route::{CompiledDomainRoute, DomainRouteError},
};
use starclock_activity::{
    ActivityDecisionId, ActivityOptionId, ActivityStateHash, GraphActivity,
    GraphActivityCommandError, GraphActivityRuntimeError,
};
use std::sync::Arc;

impl DivergentUniverseRuntimeFactory {
    /// Attaches the exact compiled source deck to an already validated immutable
    /// battle/service/event profile. No live Activity is changed. The owner's
    /// profile payload must already bind the deck, width and all other programs.
    /// Changed catalogs, graph, programs, scopes, slots or random policies reject;
    /// a second attachment rejects rather than silently replacing a deck.
    pub fn bind_position_domain_route(
        &self,
        mut flow: DivergentUniverseFlowInstance,
        route: &CompiledDomainRoute,
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        let rooms = flow
            .position_battles
            .as_deref()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        let definition = flow.definition();
        let slots = route
            .deck
            .slot_definitions()
            .map_err(DomainRouteError::Deck)
            .map_err(BattleRoomError::Route)?;
        if rooms.domain_deck.is_some()
            || flow.component_digest != self.bundle_identity().component_digest().bytes()
            || rooms
                .rooms
                .iter()
                .any(|room| room.decisions != self.decision_catalog().digest())
            || rooms
                .services
                .iter()
                .any(|room| !room.matches_factory(self))
            || rooms
                .occurrences
                .iter()
                .any(|room| !room.matches_factory(self))
            || !route.deck.matches_authored_source(self)
            || route.rooms.is_empty()
            || route.rooms.iter().any(|context| {
                let layer = context
                    .plane_ordinal
                    .checked_sub(1)
                    .and_then(|index| usize::try_from(index).ok())
                    .and_then(|index| flow.layers().get(index));
                context.area != *flow.area()
                    || layer != Some(&context.layer)
                    || !self.room_context_matches(context)
            })
            || route.graph.digest() != definition.graph().digest()
            || route.programs.len() != definition.programs().len()
            || definition
                .programs()
                .iter()
                .any(|program| !route.programs.contains(program))
            || &route.logical_scopes != definition.state_definition().logical_scopes()
            || slots
                .iter()
                .any(|slot| !definition.state_definition().slots().contains(slot))
            || route.random_offers.len() != definition.random_offers().len()
            || definition
                .random_offers()
                .iter()
                .any(|offer| !route.random_offers.contains(offer))
            || route.random_offers.iter().any(|offer| {
                !route
                    .deck
                    .random_offer(offer.node())
                    .is_ok_and(|expected| &expected == offer)
            })
            || !definition.random_checkpoints().is_empty()
            || definition.bootstrap().is_some()
            || definition.interactions().is_some()
        {
            return Err(BattleRoomError::InvalidDefinition);
        }
        let rooms = flow
            .position_battles
            .as_mut()
            .ok_or(BattleRoomError::UnsupportedEntry)?;
        Arc::make_mut(rooms).domain_deck = Some(route.deck.clone());
        Ok(flow)
    }
}

impl DivergentUniverseFlowInstance {
    /// Non-mutating authenticated piles for a bound position profile, including
    /// fixed rooms with no hand. Legacy entry-only deck selection is separate.
    /// Foreign whole definitions and malformed partitions return typed errors.
    pub fn position_domain_deck(
        &self,
        activity: &GraphActivity,
    ) -> Result<Option<DomainDeckObservation>, GraphActivityCommandError> {
        let Some(rooms) = &self.position_battles else {
            return Ok(None);
        };
        if !rooms.matches(activity) {
            return Err(invalid());
        }
        rooms
            .domain_deck
            .as_ref()
            .map(|deck| deck.observe(activity))
            .transpose()
    }

    /// True only for the bound deck's current sampled Route hand, never an event
    /// Leave or ordinary room exit. No RNG or accepted operations are produced.
    pub fn has_position_domain_hand(
        &self,
        activity: &GraphActivity,
    ) -> Result<bool, GraphActivityCommandError> {
        Ok(self
            .position_domain_deck(activity)?
            .is_some_and(|deck| !deck.hand.is_empty()))
    }

    /// Consumes only an actually offered card through existing whole-hand discard,
    /// selected-instance recording and traversal in one shared transaction.
    /// Curio entry lifetimes already belong to the compiled target entry program;
    /// they are not generated again here. Rejection restores canonical state/RNG.
    pub fn choose_position_domain_card(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if !self.has_position_domain_hand(activity)? {
            return Err(invalid());
        }
        let deck = self
            .position_battles
            .as_deref()
            .and_then(|rooms| rooms.domain_deck.as_ref())
            .ok_or_else(invalid)?;
        deck.choose(activity, expected, decision, option)
    }
}

fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
