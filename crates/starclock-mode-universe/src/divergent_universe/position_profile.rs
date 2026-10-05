//! Owned native source-position composition; no caller programs or test probes.
#[path = "position_profile_rooms.rs"]
mod rooms;

use crate::digest::Encoder;
use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
    adventure_room::AdventureRoomError,
    battle_room::{BattleRoomError, BattleRoomSelection, BattleRoomSequenceLength},
    blank_room::BlankRoomError,
    coin_room::CoinRoomError,
    domain_route::DomainRouteError,
    occurrence_room::OccurrenceRoomError,
    respite_room::{RespiteEnhancementPolicy, RespiteRoomError},
    reward_occurrence_room::RewardOccurrenceRoomError,
    shop_purchase::room::ShopRoomError,
    weighted_curio::{WeightedCurioSlotLimit, room::WeightedCurioRoomError},
};
use rooms::{DECK, ProfileRooms};
use starclock_activity::{
    ActivityConfigDigest, ActivityHandlerRegistry, ActivityRandomPolicies, ActivitySlotDefinition,
    ActivityStateDefinition, GraphActivityDefinition, core_activity_handler_bundle,
};
use starclock_data::{
    divergent_universe_decisions::{reward_occurrences::RewardOccurrenceId, shop::ShopStockId},
    divergent_universe_service_catalog::{
        DivergentUniverseOccurrenceVariantId, DivergentUniverseWorkbenchId,
    },
};
use std::sync::Arc;

/// Explicit current inputs, not recovered source room pools or slot selectors.
/// Each battle role selects its own candidate/domain. Sequence counts are
/// independent of preset level; Event repeats are an explicit bounded policy.
/// Compilation rejects unknown selections/missing native payloads as a whole.
#[derive(Clone, Debug)]
pub struct PositionProfileRecipe {
    pub deck: Box<str>,
    pub hand_width: u16,
    pub combat: BattleRoomSelection,
    pub encounter: BattleRoomSelection,
    pub elite: BattleRoomSelection,
    pub boss: BattleRoomSelection,
    pub battle_sequence: BattleRoomSequenceLength,
    pub conversion_sequence: BattleRoomSequenceLength,
    pub event: DivergentUniverseOccurrenceVariantId,
    pub following_events: Vec<DivergentUniverseOccurrenceVariantId>,
    pub shop: ShopStockId,
    pub reward: RewardOccurrenceId,
    pub workbench: DivergentUniverseWorkbenchId,
    pub respite: RespiteEnhancementPolicy,
    pub equipment_capacity: WeightedCurioSlotLimit,
}

/// Composition policy only. Each native room retains its own independently
/// replaceable accuracy contract; this does not assert complete room parity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PositionProfileAccuracy {
    VersionedProjectPolicyExplicitNativeRoomRecipeNoMissingPayloadFallback,
}

#[derive(Debug)]
pub enum PositionProfileError {
    InvalidSlots,
    InvalidDefinition,
    Battle(BattleRoomError),
    Route(DomainRouteError),
    Occurrence(OccurrenceRoomError),
    Shop(ShopRoomError),
    Reward(RewardOccurrenceRoomError),
    Respite(RespiteRoomError),
    Equipment(WeightedCurioRoomError),
    Coin(CoinRoomError),
    Adventure(AdventureRoomError),
    Blank(BlankRoomError),
}
impl std::fmt::Display for PositionProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "Divergent Universe native position profile: {self:?}"
        )
    }
}
impl std::error::Error for PositionProfileError {}

impl PositionProfileRecipe {
    #[must_use]
    pub const fn accuracy(&self) -> PositionProfileAccuracy {
        PositionProfileAccuracy::VersionedProjectPolicyExplicitNativeRoomRecipeNoMissingPayloadFallback
    }
}

impl DivergentUniverseRuntimeFactory {
    /// Build and authenticate a native fragment at every fixed/card alternative
    /// in the exact current area/deck. All immutable inputs come from the caller's
    /// typed recipe and current catalogs; there is no arbitrary program callback.
    /// The base must be a plain mapped battle-route entry with no attached entry
    /// services/profile. Missing data, conflicting slots or failed binding return
    /// a typed error and no flow, without live state/RNG/cache mutation.
    /// This does not change default admission or original room/encounter parity.
    pub fn compile_position_profile(
        &self,
        base: DivergentUniverseFlowInstance,
        recipe: &PositionProfileRecipe,
    ) -> Result<DivergentUniverseFlowInstance, PositionProfileError> {
        let mut rooms = ProfileRooms::new(self, recipe)?;
        let mut failure = None;
        let route = self.compile_curio_domain_route(
            base.area(),
            &recipe.deck,
            recipe.hand_width,
            DECK,
            |context| match rooms.compile(context, recipe) {
                Ok(fragment) => Ok(fragment),
                Err(error) => {
                    failure = Some(error);
                    Err(DomainRouteError::MissingRoomProgram(
                        context.preset_source.clone(),
                    ))
                }
            },
        );
        let route = route.map_err(|error| failure.unwrap_or(PositionProfileError::Route(error)))?;
        let mut slots = base.definition().state_definition().slots().to_vec();
        add_slots(
            &mut slots,
            route
                .deck
                .slot_definitions()
                .map_err(DomainRouteError::Deck)
                .map_err(PositionProfileError::Route)?,
        )?;
        rooms.add_slots(&mut slots)?;
        let mut owner = Encoder::new(b"starclock.du.explicit-native-position-recipe.current");
        owner.digest(self.decision_catalog().digest());
        owner.text(&recipe.deck);
        owner.u32(u32::from(recipe.hand_width));
        for (node, digest) in rooms.configuration_digests() {
            owner.u32(node.get());
            owner.digest(digest);
        }
        let payload = ActivityConfigDigest::new(owner.finish())
            .ok_or(PositionProfileError::InvalidDefinition)?;
        let identity = self
            .position_room_identity_with_occurrences(
                &base,
                &route.graph,
                &rooms.battles,
                &[],
                &rooms.occurrences,
                payload,
            )
            .map_err(PositionProfileError::Battle)?;
        let definition = GraphActivityDefinition::new(
            identity,
            route.graph.clone(),
            ActivityStateDefinition::new(
                slots,
                base.definition().state_definition().inventories().to_vec(),
                base.definition().state_definition().modifiers().to_vec(),
            )
            .map_err(|_| PositionProfileError::InvalidSlots)?
            .with_logical_scopes(route.logical_scopes.clone()),
            Arc::clone(base.definition().participants()),
            route.programs.clone(),
            None,
            ActivityRandomPolicies::new(Vec::new(), route.random_offers.clone()),
        )
        .map_err(|_| PositionProfileError::InvalidDefinition)?;
        let definition = if rooms.adventures.is_empty() {
            definition
        } else {
            definition
                .with_interactions(
                    ActivityHandlerRegistry::compose(vec![core_activity_handler_bundle()])
                        .map_err(|_| PositionProfileError::InvalidDefinition)?,
                    rooms
                        .adventures
                        .iter()
                        .flat_map(|room| room.interaction_bindings().iter().cloned())
                        .collect(),
                )
                .map_err(|_| PositionProfileError::InvalidDefinition)?
        };
        let flow = self
            .bind_position_rooms_with_occurrences(
                base,
                Arc::new(definition),
                &rooms.battles,
                &[],
                &rooms.occurrences,
                payload,
            )
            .map_err(PositionProfileError::Battle)?;
        let mut flow = self
            .bind_position_domain_route(flow, &route)
            .map_err(PositionProfileError::Battle)?;
        if !rooms.equipment.is_empty() {
            flow = self
                .bind_position_weighted_curio_rooms(flow, &rooms.equipment)
                .map_err(PositionProfileError::Battle)?;
        }
        if !rooms.shops.is_empty() {
            flow = self
                .bind_position_shop_rooms(flow, &rooms.shops)
                .map_err(PositionProfileError::Battle)?;
        }
        if !rooms.coins.is_empty() {
            flow = self
                .bind_position_coin_rooms(flow, &rooms.coins)
                .map_err(PositionProfileError::Battle)?;
        }
        if !rooms.adventures.is_empty() {
            flow = self
                .bind_position_adventure_rooms(flow, &rooms.adventures)
                .map_err(PositionProfileError::Battle)?;
        }
        if !rooms.respites.is_empty() {
            flow = self
                .bind_position_respite_rooms(flow, &rooms.respites)
                .map_err(PositionProfileError::Battle)?;
        }
        if !rooms.blanks.is_empty() {
            flow = self
                .bind_position_blank_rooms(flow, &rooms.blanks)
                .map_err(PositionProfileError::Battle)?;
        }
        Ok(flow)
    }
}

fn add_slots(
    slots: &mut Vec<ActivitySlotDefinition>,
    declarations: impl IntoIterator<Item = ActivitySlotDefinition>,
) -> Result<(), PositionProfileError> {
    for declaration in declarations {
        if let Some(existing) = slots.iter().find(|slot| slot.id() == declaration.id()) {
            if existing != &declaration {
                return Err(PositionProfileError::InvalidSlots);
            }
        } else {
            slots.push(declaration);
        }
    }
    Ok(())
}
