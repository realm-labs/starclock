//! Closed native room routing and immutable contributions for an owned recipe.
use super::{PositionProfileError, PositionProfileRecipe, add_slots};
use crate::divergent_universe::{
    DivergentUniverseRuntimeFactory,
    adventure_room::{AdventureRoomCompiler, CompiledAdventureRoom},
    battle_room::{BattleRoomCompiler, CompiledBattleRoom},
    blank_room::{BlankRoomCompiler, CompiledBlankRoom},
    coin_room::{CoinRoomCompiler, CompiledCoinRoom},
    domain_deck::DomainDeckSlots,
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRoomProgram},
    occurrence_room::{CompiledOccurrenceRoom, OccurrenceRoomCompiler},
    respite_room::{CompiledRespiteRoom, RespiteRoomCompiler},
    reward_occurrence_room::RewardOccurrenceRoomCompiler,
    shop_purchase::room::{CompiledShopRoom, ShopRoomCompiler, ShopRoomSlots},
    weighted_curio::room::{
        CompiledWeightedCurioRoom, WeightedCurioRoomCompiler, WeightedCurioRoomSlots,
    },
};
use starclock_activity::{ActivitySlotDefinition, ActivitySlotId, NodeId};
use starclock_data::{
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
};

pub(super) const DECK: DomainDeckSlots = DomainDeckSlots {
    draw: slot(80),
    discard: slot(81),
    selected: slot(82),
    accepted: slot(83),
};
const fn slot(value: u32) -> ActivitySlotId {
    match ActivitySlotId::new(value) {
        Some(slot) => slot,
        None => panic!("fixed host address is nonzero"),
    }
}

pub(super) struct ProfileRooms {
    combat: BattleRoomCompiler,
    encounter: BattleRoomCompiler,
    elite: BattleRoomCompiler,
    boss: BattleRoomCompiler,
    event: OccurrenceRoomCompiler,
    shop: ShopRoomCompiler,
    reward: RewardOccurrenceRoomCompiler,
    respite: RespiteRoomCompiler,
    weighted: WeightedCurioRoomCompiler,
    coin: CoinRoomCompiler,
    adventure: AdventureRoomCompiler,
    blank: BlankRoomCompiler,
    pub(super) battles: Vec<CompiledBattleRoom>,
    pub(super) occurrences: Vec<CompiledOccurrenceRoom>,
    pub(super) shops: Vec<CompiledShopRoom>,
    pub(super) respites: Vec<CompiledRespiteRoom>,
    pub(super) equipment: Vec<CompiledWeightedCurioRoom>,
    pub(super) coins: Vec<CompiledCoinRoom>,
    pub(super) adventures: Vec<CompiledAdventureRoom>,
    pub(super) blanks: Vec<CompiledBlankRoom>,
}

impl ProfileRooms {
    pub(super) fn new(
        factory: &DivergentUniverseRuntimeFactory,
        recipe: &PositionProfileRecipe,
    ) -> Result<Self, PositionProfileError> {
        Ok(Self {
            combat: factory
                .battle_room_compiler(recipe.combat.clone())
                .map_err(PositionProfileError::Battle)?,
            encounter: factory
                .battle_room_compiler(recipe.encounter.clone())
                .map_err(PositionProfileError::Battle)?,
            elite: factory
                .battle_room_compiler(recipe.elite.clone())
                .map_err(PositionProfileError::Battle)?,
            boss: factory
                .battle_room_compiler(recipe.boss.clone())
                .map_err(PositionProfileError::Battle)?,
            event: factory
                .occurrence_room_compiler(&recipe.event)
                .map_err(PositionProfileError::Occurrence)?,
            shop: factory
                .authored_shop_room_compiler(
                    &recipe.shop,
                    ShopRoomSlots {
                        purchased: slot(110),
                        accepted: slot(111),
                    },
                )
                .map_err(PositionProfileError::Shop)?,
            reward: factory
                .authored_reward_occurrence_room_compiler(&recipe.reward)
                .map_err(PositionProfileError::Reward)?,
            respite: factory
                .respite_room_compiler(&recipe.workbench, recipe.respite)
                .map_err(PositionProfileError::Respite)?,
            weighted: factory
                .weighted_curio_room_compiler(
                    recipe.equipment_capacity,
                    WeightedCurioRoomSlots {
                        changes: slot(90),
                        accepted: slot(91),
                    },
                )
                .map_err(PositionProfileError::Equipment)?,
            coin: factory
                .coin_room_compiler(slot(112))
                .map_err(PositionProfileError::Coin)?,
            adventure: factory
                .adventure_room_compiler(slot(113))
                .map_err(PositionProfileError::Adventure)?,
            blank: factory.blank_room_compiler(),
            battles: Vec::new(),
            occurrences: Vec::new(),
            shops: Vec::new(),
            respites: Vec::new(),
            equipment: Vec::new(),
            coins: Vec::new(),
            adventures: Vec::new(),
            blanks: Vec::new(),
        })
    }

    pub(super) fn compile(
        &mut self,
        context: &DomainRoomContext,
        recipe: &PositionProfileRecipe,
    ) -> Result<DomainRoomProgram, PositionProfileError> {
        match context.composition {
            DomainRoomComposition::Fixed(FixedDomainKind::Battle)
            | DomainRoomComposition::Card(DomainCardKind::Battle) => {
                let room = self
                    .combat
                    .compile_sequence(context, recipe.battle_sequence)
                    .map_err(PositionProfileError::Battle)?;
                let fragment = room.fragment().clone();
                self.battles.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Fixed(FixedDomainKind::Boss) => {
                let room = self
                    .boss
                    .compile_sequence(context, recipe.battle_sequence)
                    .map_err(PositionProfileError::Battle)?;
                let fragment = room.fragment().clone();
                self.battles.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Elite) => {
                let room = self
                    .elite
                    .compile_sequence(context, recipe.battle_sequence)
                    .map_err(PositionProfileError::Battle)?;
                let fragment = room.fragment().clone();
                self.battles.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Encounter) => {
                let room = self
                    .encounter
                    .compile_sequence(context, recipe.battle_sequence)
                    .map_err(PositionProfileError::Battle)?;
                let fragment = room.fragment().clone();
                self.battles.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Fixed(FixedDomainKind::Conversion) => {
                let room = self
                    .combat
                    .compile_conversion_sequence(context, recipe.conversion_sequence)
                    .map_err(PositionProfileError::Battle)?;
                let fragment = room.fragment().clone();
                self.battles.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Event) => {
                let room = self
                    .event
                    .compile_sequence(context, &recipe.following_events)
                    .map_err(PositionProfileError::Occurrence)?;
                let fragment = room.fragment().clone();
                self.occurrences.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Reward) => {
                let room = self
                    .reward
                    .compile(context)
                    .map_err(PositionProfileError::Reward)?;
                let fragment = room.fragment().clone();
                self.occurrences.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Shop) => {
                let room = self
                    .shop
                    .compile(context)
                    .map_err(PositionProfileError::Shop)?;
                let fragment = room.fragment().clone();
                self.shops.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Reforge) => {
                let room = self
                    .weighted
                    .compile(context)
                    .map_err(PositionProfileError::Equipment)?;
                let fragment = room.fragment().clone();
                self.equipment.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Coin)
            | DomainRoomComposition::Fixed(FixedDomainKind::Coin) => {
                let room = self
                    .coin
                    .compile(context)
                    .map_err(PositionProfileError::Coin)?;
                let fragment = room.fragment().clone();
                self.coins.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Card(DomainCardKind::Adventure) => {
                let room = self
                    .adventure
                    .compile(context)
                    .map_err(PositionProfileError::Adventure)?;
                let fragment = room.fragment().clone();
                self.adventures.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Fixed(FixedDomainKind::Respite) => {
                let room = self
                    .respite
                    .compile(context)
                    .map_err(PositionProfileError::Respite)?;
                let fragment = room.fragment().clone();
                self.respites.push(room);
                Ok(fragment)
            }
            DomainRoomComposition::Fixed(FixedDomainKind::Blank) => {
                let room = self
                    .blank
                    .compile(context)
                    .map_err(PositionProfileError::Blank)?;
                let fragment = room.fragment().clone();
                self.blanks.push(room);
                Ok(fragment)
            }
        }
    }

    pub(super) fn add_slots(
        &self,
        slots: &mut Vec<ActivitySlotDefinition>,
    ) -> Result<(), PositionProfileError> {
        for room in &self.shops {
            add_slots(slots, room.slot_definitions().iter().cloned())?;
        }
        for room in &self.respites {
            add_slots(slots, room.slot_definitions().iter().cloned())?;
        }
        for room in &self.equipment {
            add_slots(slots, room.slot_definitions().iter().cloned())?;
        }
        for room in &self.coins {
            add_slots(slots, std::iter::once(room.slot_definition().clone()))?;
        }
        for room in &self.adventures {
            add_slots(slots, std::iter::once(room.slot_definition().clone()))?;
        }
        Ok(())
    }

    pub(super) fn configuration_digests(&self) -> Vec<(NodeId, [u8; 32])> {
        let mut values = Vec::new();
        for room in &self.battles {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.occurrences {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.shops {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.respites {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.equipment {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.coins {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.adventures {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        for room in &self.blanks {
            values.push((room.context().entry_node(), room.configuration_digest()));
        }
        values.sort_by_key(|(node, _)| *node);
        values
    }
}
