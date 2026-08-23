use std::collections::BTreeMap;

use starclock_activity::{ActivityRngLabel, GraphActivityCommandError, GraphActivityRuntimeError};

use super::{
    CurrencyWarsRun, CurrencyWarsRuntimeError, DEPLOYMENT, EQUIPMENT_INVENTORY, EQUIPMENT_LOADOUT,
    FORGE_ITEM, FORGE_OFFERS, FREE_REFRESHES, GOLD, ITEM_INVENTORY, ROSTER,
    SPECIAL_GOOD_ACTIVATIONS, SPECIAL_GOOD_OFFER, SPECIAL_GOOD_PURCHASED, TREASURE_TO_TRASH_PLANE,
    add_equipment_inventory, bond_operations, debug_error, encode_equipment_inventory, error,
    program_id, remove_equipment_inventory, set_counter_map, set_integer, set_ordered_ids,
};
use crate::{
    CurrencyWarsCatalog, CurrencyWarsConsumableDefinition, CurrencyWarsConsumableKind,
    CurrencyWarsDeployment, CurrencyWarsEquipmentCategory, CurrencyWarsEquipmentId,
    CurrencyWarsEquipmentLoadout, CurrencyWarsForgeTarget, CurrencyWarsInvestmentId,
    CurrencyWarsItemId, CurrencyWarsRoleId, CurrencyWarsRoster, CurrencyWarsSpecialGoodAcquisition,
};

const REROLL_EQUIPMENT_PURPOSE: u16 = 32;
const RECOMMENDED_EQUIPMENT_PURPOSE: u16 = 33;
const FORGE_OFFER_PURPOSE: u16 = 34;
const TREASURE_TO_TRASH_PURPOSE: u16 = 35;
const FORGE_ROLE_BIT: u64 = 1 << 63;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsForgeOffer {
    Equipment(CurrencyWarsEquipmentId),
    Role { role: CurrencyWarsRoleId, star: u8 },
}

impl CurrencyWarsForgeOffer {
    fn encode(self) -> u64 {
        match self {
            Self::Equipment(equipment) => u64::from(equipment.get()),
            Self::Role { role, star } => {
                FORGE_ROLE_BIT | (u64::from(role.get()) << 8) | u64::from(star)
            }
        }
    }

    fn decode(raw: u64) -> Result<Self, CurrencyWarsRuntimeError> {
        if raw & FORGE_ROLE_BIT == 0 {
            return u32::try_from(raw)
                .ok()
                .and_then(CurrencyWarsEquipmentId::new)
                .map(Self::Equipment)
                .ok_or_else(|| error("Currency Wars forge equipment offer is invalid"));
        }
        let role = u32::try_from((raw & !FORGE_ROLE_BIT) >> 8)
            .ok()
            .and_then(CurrencyWarsRoleId::new)
            .ok_or_else(|| error("Currency Wars forge role offer is invalid"))?;
        let star = u8::try_from(raw & 0xff)
            .map_err(|_| error("Currency Wars forge role star is invalid"))?;
        if star == 0 {
            return Err(error("Currency Wars forge role star is zero"));
        }
        Ok(Self::Role { role, star })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRewardPoolResolution {
    pub selected_reward_ids: Box<[u32]>,
    pub remaining_value: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSpecialGoodActivation {
    pub id: u32,
    pub activation_count: u32,
    pub price_paid: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsAppliedReward {
    NoLegalResult,
    Gold(u32),
    FreeRefresh(u32),
    Experience(u32),
    Item {
        item: CurrencyWarsItemId,
        count: u32,
    },
    Investment(CurrencyWarsInvestmentId),
    Role {
        role: CurrencyWarsRoleId,
        star: u8,
    },
    Equipment(Box<[CurrencyWarsEquipmentId]>),
    RoleWithEquipment {
        role: CurrencyWarsRoleId,
        star: u8,
        equipment: Box<[CurrencyWarsEquipmentId]>,
    },
}

impl CurrencyWarsRun {
    pub fn item_inventory(
        &self,
    ) -> Result<BTreeMap<CurrencyWarsItemId, u32>, CurrencyWarsRuntimeError> {
        decode_item_inventory(self.counter_map(ITEM_INVENTORY)?)
    }

    #[must_use]
    pub fn free_refreshes(&self) -> u32 {
        u32::try_from(self.integer(FREE_REFRESHES)).unwrap_or_default()
    }

    pub fn current_special_good_offer(&self) -> Result<Option<u32>, CurrencyWarsRuntimeError> {
        let offers = self.ordered_ids(SPECIAL_GOOD_OFFER)?;
        match offers.as_ref() {
            [] => Ok(None),
            [raw] => u32::try_from(*raw)
                .map(Some)
                .map_err(|_| error("Currency Wars special-good offer is invalid")),
            _ => Err(error("Currency Wars special-good offer is not singular")),
        }
    }

    /// Installs one catalog-backed offer chosen by an owning mode program.
    pub fn offer_special_good(&mut self, id: u32) -> Result<(), CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let good = self
            .definition
            .catalog
            .service_catalog()
            .special_good(id)
            .ok_or_else(|| error("Currency Wars special good is missing"))?;
        if !matches!(
            good.acquisition,
            CurrencyWarsSpecialGoodAcquisition::ShopPurchase { .. }
        ) {
            return Err(error("Currency Wars special good is not a shop purchase"));
        }
        if self.current_special_good_offer()?.is_some() {
            return Err(error("Currency Wars special-good offer is already active"));
        }
        if !self.ordered_ids(SPECIAL_GOOD_PURCHASED)?.is_empty() {
            return Err(error(
                "Currency Wars special-good purchase limit was reached for this node",
            ));
        }
        self.apply_state(
            143,
            vec![set_ordered_ids(
                SPECIAL_GOOD_OFFER,
                Box::new([u64::from(id)]),
            )],
        )
    }

    pub fn purchase_special_good(
        &mut self,
        id: u32,
    ) -> Result<CurrencyWarsSpecialGoodActivation, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        if self.current_special_good_offer()? != Some(id) {
            return Err(error("Currency Wars special good is not currently offered"));
        }
        if !self.ordered_ids(SPECIAL_GOOD_PURCHASED)?.is_empty() {
            return Err(error(
                "Currency Wars special-good purchase limit was reached for this node",
            ));
        }
        let good = self
            .definition
            .catalog
            .service_catalog()
            .special_good(id)
            .ok_or_else(|| error("Currency Wars special good is missing"))?;
        let CurrencyWarsSpecialGoodAcquisition::ShopPurchase { price } = good.acquisition else {
            return Err(error("Currency Wars special good is not a shop purchase"));
        };
        let gold = self
            .gold()
            .checked_sub(price)
            .ok_or_else(|| error("Currency Wars special good is unaffordable"))?;
        let mut activations =
            decode_special_good_activations(self.counter_map(SPECIAL_GOOD_ACTIVATIONS)?)?;
        let count = activations.entry(id).or_default();
        *count = count
            .checked_add(1)
            .ok_or_else(|| error("Currency Wars special-good activation count overflow"))?;
        let activation = CurrencyWarsSpecialGoodActivation {
            id,
            activation_count: *count,
            price_paid: price,
        };
        self.apply_state(
            144,
            vec![
                set_integer(GOLD, i64::from(gold)),
                set_ordered_ids(SPECIAL_GOOD_OFFER, Box::new([])),
                set_ordered_ids(SPECIAL_GOOD_PURCHASED, Box::new([u64::from(id)])),
                set_counter_map(
                    SPECIAL_GOOD_ACTIVATIONS,
                    encode_special_good_activations(&activations),
                ),
            ],
        )?;
        Ok(activation)
    }

    pub fn special_good_activations(&self) -> Result<BTreeMap<u32, u32>, CurrencyWarsRuntimeError> {
        decode_special_good_activations(self.counter_map(SPECIAL_GOOD_ACTIVATIONS)?)
    }

    /// Activates the complete source-authored three-star Cyrene reward set once.
    pub fn activate_cyrene_three_star_goods(
        &mut self,
    ) -> Result<Box<[u32]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let ids = self
            .definition
            .catalog
            .service_catalog()
            .special_goods()
            .iter()
            .filter(|good| good.acquisition == CurrencyWarsSpecialGoodAcquisition::CyreneThreeStar)
            .map(|good| good.id)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return Err(error("Currency Wars three-star Cyrene reward set is empty"));
        }
        let mut activations = self.special_good_activations()?;
        if ids.iter().any(|id| activations.contains_key(id)) {
            return Err(error(
                "Currency Wars three-star Cyrene reward set is already active",
            ));
        }
        for id in &ids {
            activations.insert(*id, 1);
        }
        self.apply_state(
            145,
            vec![set_counter_map(
                SPECIAL_GOOD_ACTIVATIONS,
                encode_special_good_activations(&activations),
            )],
        )?;
        Ok(ids.into_boxed_slice())
    }

    pub fn receive_item(
        &mut self,
        item: CurrencyWarsItemId,
        count: u32,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        if count == 0 {
            return Err(error("Currency Wars received item count is zero"));
        }
        self.definition
            .catalog
            .service_catalog()
            .item(item)
            .ok_or_else(|| error("Currency Wars received item is missing"))?;
        let mut inventory = self.item_inventory()?;
        add_item(&mut inventory, item, count)?;
        self.apply_state(
            130,
            vec![set_counter_map(
                ITEM_INVENTORY,
                encode_item_inventory(&inventory),
            )],
        )
    }

    pub fn craft_equipment(&mut self, recipe_id: u32) -> Result<(), CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let recipe = self
            .definition
            .catalog
            .service_catalog()
            .recipe(recipe_id)
            .cloned()
            .ok_or_else(|| error("Currency Wars equipment recipe is missing"))?;
        if recipe.season_id != self.current_service_season()? {
            return Err(error(
                "Currency Wars equipment recipe is not in the current season",
            ));
        }
        let output_definition = self
            .definition
            .catalog
            .build_catalog()
            .runtime_equipment(recipe.output)
            .ok_or_else(|| error("Currency Wars recipe output equipment is missing"))?;
        let mut inventory = self.equipment_inventory()?;
        for input in &recipe.inputs {
            remove_equipment_inventory(&mut inventory, *input)?;
        }
        let chance = self.definition.enemy_affixes.treasure_to_trash_chance();
        let plane = self
            .current_plane()
            .ok_or_else(|| error("Currency Wars craft has no current plane"))?;
        let first_advanced_synthesis = chance.is_some()
            && output_definition.category == CurrencyWarsEquipmentCategory::Craftable
            && self.integer(TREASURE_TO_TRASH_PLANE) != i64::from(plane);
        if !first_advanced_synthesis {
            add_equipment_inventory(&mut inventory, recipe.output)?;
            return self.apply_equipment_state(131, &inventory, &self.equipment_loadout()?);
        }
        let chance = chance.expect("first advanced synthesis requires an Affix chance");
        if !(0..=1_000_000).contains(&chance.scaled()) {
            return Err(error("Currency Wars Treasure to Trash chance is invalid"));
        }
        let trash = self
            .definition
            .catalog
            .build_catalog()
            .equipment()
            .iter()
            .filter_map(|definition| definition.runtime.as_ref())
            .find(|definition| definition.category == CurrencyWarsEquipmentCategory::Trash)
            .map(|definition| definition.id)
            .ok_or_else(|| error("Currency Wars Trash Bag equipment is missing"))?;
        self.activity
            .apply_generated_boundary(self.state_hash(), program_id(140), move |rng| {
                let weights = [
                    u64::try_from(chance.scaled()).map_err(|_| invalid_generated_boundary())?,
                    u64::try_from(1_000_000 - chance.scaled())
                        .map_err(|_| invalid_generated_boundary())?,
                ];
                let (selected, _) = rng
                    .choose_weighted(
                        ActivityRngLabel::Reward,
                        TREASURE_TO_TRASH_PURPOSE,
                        &weights,
                    )
                    .map_err(GraphActivityCommandError::Rng)?
                    .ok_or_else(invalid_generated_boundary)?;
                let output = if selected == 0 { trash } else { recipe.output };
                add_equipment_inventory(&mut inventory, output)
                    .map_err(|_| invalid_generated_boundary())?;
                Ok((
                    vec![
                        set_counter_map(
                            EQUIPMENT_INVENTORY,
                            encode_equipment_inventory(&inventory),
                        ),
                        set_integer(TREASURE_TO_TRASH_PLANE, i64::from(plane)),
                    ],
                    (),
                ))
            })
            .map_err(debug_error)?;
        Ok(())
    }

    pub fn upgrade_inventory_equipment(
        &mut self,
        source: CurrencyWarsEquipmentId,
    ) -> Result<CurrencyWarsEquipmentId, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let output = self
            .definition
            .catalog
            .service_catalog()
            .upgrade(source)
            .ok_or_else(|| error("Currency Wars equipment has no authored upgrade"))?;
        self.definition
            .catalog
            .build_catalog()
            .runtime_equipment(output)
            .ok_or_else(|| error("Currency Wars upgraded equipment is missing"))?;
        let mut inventory = self.equipment_inventory()?;
        remove_equipment_inventory(&mut inventory, source)?;
        add_equipment_inventory(&mut inventory, output)?;
        self.apply_equipment_state(132, &inventory, &self.equipment_loadout()?)?;
        Ok(output)
    }

    pub fn use_remove_equipment(
        &mut self,
        item: CurrencyWarsItemId,
        role: CurrencyWarsRoleId,
    ) -> Result<Box<[CurrencyWarsEquipmentId]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let consumable =
            self.require_consumable(item, CurrencyWarsConsumableKind::RemoveEquipment)?;
        let mut items = self.item_inventory()?;
        consume_item(&mut items, item, consumable.consume)?;
        let mut equipment = self.equipment_inventory()?;
        let mut loadout = self.equipment_loadout()?;
        let removed = loadout
            .for_role(role)
            .map(|(_, equipment)| equipment)
            .collect::<Vec<_>>();
        if removed.is_empty() {
            return Err(error("Currency Wars role has no equipment to remove"));
        }
        loadout.remove_role(role);
        for value in &removed {
            add_equipment_inventory(&mut equipment, *value)?;
        }
        self.apply_service_equipment_state(133, &items, &equipment, &loadout)?;
        Ok(removed.into_boxed_slice())
    }

    pub fn use_upgrade_equipment(
        &mut self,
        item: CurrencyWarsItemId,
        source: CurrencyWarsEquipmentId,
    ) -> Result<CurrencyWarsEquipmentId, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let consumable =
            self.require_consumable(item, CurrencyWarsConsumableKind::UpgradeEquipment)?;
        let output = self
            .definition
            .catalog
            .service_catalog()
            .upgrade(source)
            .ok_or_else(|| error("Currency Wars equipment has no authored upgrade"))?;
        let mut items = self.item_inventory()?;
        consume_item(&mut items, item, consumable.consume)?;
        let mut equipment = self.equipment_inventory()?;
        remove_equipment_inventory(&mut equipment, source)?;
        add_equipment_inventory(&mut equipment, output)?;
        self.apply_service_equipment_state(134, &items, &equipment, &self.equipment_loadout()?)?;
        Ok(output)
    }

    pub fn use_copy_role(
        &mut self,
        item: CurrencyWarsItemId,
        role: CurrencyWarsRoleId,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let consumable = self.require_consumable(item, CurrencyWarsConsumableKind::CopyRole)?;
        let [maximum_rarity, star] = consumable.parameters.as_ref() else {
            return Err(error("Currency Wars role-copy parameters are malformed"));
        };
        let definition = self
            .definition
            .catalog
            .role(role)
            .ok_or_else(|| error("Currency Wars copied role is missing"))?;
        if u32::from(definition.rarity) > *maximum_rarity {
            return Err(error("Currency Wars copied role exceeds the rarity limit"));
        }
        let star = u8::try_from(*star).map_err(debug_error)?;
        let mut items = self.item_inventory()?;
        consume_item(&mut items, item, consumable.consume)?;
        let mut roster = self.roster()?;
        grant_role(&self.definition.catalog, &mut roster, role, star)?;
        let deployment = self.deployment()?.reconcile_acquisition(&roster, role);
        validate_service_roster(
            &self.definition.catalog,
            &roster,
            &deployment,
            self.team_level(),
            self.back_capacity(),
        )?;
        self.apply_service_roster_state(135, &items, &roster, &deployment)
    }

    pub fn use_reroll_equipment(
        &mut self,
        item: CurrencyWarsItemId,
        source: CurrencyWarsEquipmentId,
    ) -> Result<CurrencyWarsEquipmentId, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let consumable =
            self.require_consumable(item, CurrencyWarsConsumableKind::RerollEquipment)?;
        let source_definition = self
            .definition
            .catalog
            .build_catalog()
            .runtime_equipment(source)
            .ok_or_else(|| error("Currency Wars rerolled equipment is missing"))?;
        let candidates = self
            .definition
            .catalog
            .build_catalog()
            .equipment()
            .iter()
            .filter_map(|definition| definition.runtime.as_ref())
            .filter(|definition| {
                definition.id != source && definition.category == source_definition.category
            })
            .map(|definition| definition.id)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return Err(error("Currency Wars equipment reroll has no candidate"));
        }
        let mut items = self.item_inventory()?;
        consume_item(&mut items, item, consumable.consume)?;
        let mut equipment = self.equipment_inventory()?;
        remove_equipment_inventory(&mut equipment, source)?;
        let loadout = self.equipment_loadout()?;
        let bonds = self.bond_snapshot_for(&self.deployment()?, &loadout)?;
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(137), move |rng| {
                let draw = rng
                    .choose_index(
                        ActivityRngLabel::Reward,
                        REROLL_EQUIPMENT_PURPOSE,
                        u32::try_from(candidates.len())
                            .map_err(|_| invalid_generated_boundary())?,
                    )
                    .map_err(GraphActivityCommandError::Rng)?
                    .ok_or_else(invalid_generated_boundary)?;
                let output = candidates[draw.value() as usize];
                add_equipment_inventory(&mut equipment, output)
                    .map_err(|_| invalid_generated_boundary())?;
                let mut operations = vec![
                    set_counter_map(ITEM_INVENTORY, encode_item_inventory(&items)),
                    set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(&equipment)),
                ];
                operations.extend(bond_operations(&bonds));
                Ok((operations, output))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    pub fn use_recommended_equipment(
        &mut self,
        item: CurrencyWarsItemId,
        role: CurrencyWarsRoleId,
    ) -> Result<Box<[CurrencyWarsEquipmentId]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let consumable =
            self.require_consumable(item, CurrencyWarsConsumableKind::GainRecommendedEquipment)?;
        let [raw_count] = consumable.parameters.as_ref() else {
            return Err(error(
                "Currency Wars recommended-equipment parameters are malformed",
            ));
        };
        if !self.roster()?.owns_role(role) {
            return Err(error(
                "Currency Wars recommended-equipment role is not owned",
            ));
        }
        let count = u16::try_from(*raw_count).map_err(debug_error)?;
        let candidates = self
            .definition
            .catalog
            .build_catalog()
            .recommended_for_role(role)
            .collect::<Vec<_>>();
        if candidates.len() < usize::from(count) {
            return Err(error(
                "Currency Wars recommended-equipment pool is too small",
            ));
        }
        let mut items = self.item_inventory()?;
        consume_item(&mut items, item, consumable.consume)?;
        let mut equipment = self.equipment_inventory()?;
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(138), move |rng| {
                let indices = rng
                    .choose_weighted_without_replacement(
                        ActivityRngLabel::Reward,
                        RECOMMENDED_EQUIPMENT_PURPOSE,
                        &vec![1; candidates.len()],
                        count,
                    )
                    .map_err(GraphActivityCommandError::Rng)?;
                let mut selected = indices
                    .iter()
                    .map(|index| candidates[*index as usize])
                    .collect::<Vec<_>>();
                selected.sort_unstable();
                for value in &selected {
                    add_equipment_inventory(&mut equipment, *value)
                        .map_err(|_| invalid_generated_boundary())?;
                }
                Ok((
                    vec![
                        set_counter_map(ITEM_INVENTORY, encode_item_inventory(&items)),
                        set_counter_map(
                            EQUIPMENT_INVENTORY,
                            encode_equipment_inventory(&equipment),
                        ),
                    ],
                    selected.into_boxed_slice(),
                ))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    pub fn open_forge(
        &mut self,
        item: CurrencyWarsItemId,
    ) -> Result<Box<[CurrencyWarsForgeOffer]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        if !self.current_forge_offers()?.is_empty() {
            return Err(error("Currency Wars forge offer is already active"));
        }
        if !self.item_inventory()?.contains_key(&item) {
            return Err(error("Currency Wars forge item is not owned"));
        }
        let service = self
            .definition
            .catalog
            .service_catalog()
            .forge_service(item)
            .cloned()
            .ok_or_else(|| error("Currency Wars forge service is missing"))?;
        let candidates =
            forge_candidates(&self.definition.catalog, service.category, service.target);
        if candidates.len() < usize::from(service.offer_count) {
            return Err(error("Currency Wars forge has too few candidates"));
        }
        let offer_count = u16::from(service.offer_count);
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(139), move |rng| {
                let indices = rng
                    .choose_weighted_without_replacement(
                        ActivityRngLabel::Reward,
                        FORGE_OFFER_PURPOSE,
                        &vec![1; candidates.len()],
                        offer_count,
                    )
                    .map_err(GraphActivityCommandError::Rng)?;
                let mut offers = indices
                    .iter()
                    .map(|index| candidates[*index as usize])
                    .collect::<Vec<_>>();
                offers.sort_unstable();
                Ok((
                    vec![
                        set_ordered_ids(
                            FORGE_OFFERS,
                            offers.iter().map(|offer| offer.encode()).collect(),
                        ),
                        set_integer(FORGE_ITEM, i64::from(item.get())),
                    ],
                    offers.into_boxed_slice(),
                ))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    pub fn current_forge_offers(
        &self,
    ) -> Result<Box<[CurrencyWarsForgeOffer]>, CurrencyWarsRuntimeError> {
        self.ordered_ids(FORGE_OFFERS)?
            .iter()
            .map(|raw| CurrencyWarsForgeOffer::decode(*raw))
            .collect::<Result<Vec<_>, _>>()
            .map(Vec::into_boxed_slice)
    }

    pub fn choose_forge_offer(
        &mut self,
        offer: CurrencyWarsForgeOffer,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        if self.current_forge_offers()?.binary_search(&offer).is_err() {
            return Err(error("Currency Wars forge offer is not active"));
        }
        let item = u32::try_from(self.integer(FORGE_ITEM))
            .ok()
            .and_then(CurrencyWarsItemId::new)
            .ok_or_else(|| error("Currency Wars forge item state is invalid"))?;
        let mut items = self.item_inventory()?;
        consume_item(&mut items, item, true)?;
        match offer {
            CurrencyWarsForgeOffer::Equipment(value) => {
                let mut equipment = self.equipment_inventory()?;
                add_equipment_inventory(&mut equipment, value)?;
                let loadout = self.equipment_loadout()?;
                let bonds = self.bond_snapshot_for(&self.deployment()?, &loadout)?;
                let mut operations = vec![
                    set_counter_map(ITEM_INVENTORY, encode_item_inventory(&items)),
                    set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(&equipment)),
                    set_ordered_ids(FORGE_OFFERS, Box::new([])),
                    set_integer(FORGE_ITEM, 0),
                ];
                operations.extend(bond_operations(&bonds));
                self.apply_state(140, operations)
            }
            CurrencyWarsForgeOffer::Role { role, star } => {
                let mut roster = self.roster()?;
                grant_role(&self.definition.catalog, &mut roster, role, star)?;
                let deployment = self.deployment()?.reconcile_acquisition(&roster, role);
                validate_service_roster(
                    &self.definition.catalog,
                    &roster,
                    &deployment,
                    self.team_level(),
                    self.back_capacity(),
                )?;
                let mut equipment = self.equipment_inventory()?;
                let mut loadout = self.equipment_loadout()?;
                let removed_roles = loadout
                    .slots()
                    .keys()
                    .map(|slot| slot.role())
                    .filter(|role| !roster.owns_role(*role))
                    .collect::<std::collections::BTreeSet<_>>();
                for removed_role in removed_roles {
                    let removed = loadout
                        .for_role(removed_role)
                        .map(|(_, equipment)| equipment)
                        .collect::<Vec<_>>();
                    loadout.remove_role(removed_role);
                    for value in removed {
                        add_equipment_inventory(&mut equipment, value)?;
                    }
                }
                let bonds = self.bond_snapshot_for(&deployment, &loadout)?;
                let mut operations = vec![
                    set_counter_map(ITEM_INVENTORY, encode_item_inventory(&items)),
                    set_counter_map(ROSTER, roster.encoded()),
                    set_counter_map(DEPLOYMENT, deployment.encoded()),
                    set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(&equipment)),
                    set_counter_map(EQUIPMENT_LOADOUT, loadout.encoded()),
                    set_ordered_ids(FORGE_OFFERS, Box::new([])),
                    set_integer(FORGE_ITEM, 0),
                ];
                operations.extend(bond_operations(&bonds));
                self.apply_state(141, operations)
            }
        }
    }

    fn current_service_season(&self) -> Result<u16, CurrencyWarsRuntimeError> {
        self.definition
            .catalog
            .difficulties()
            .iter()
            .find(|value| value.source_id == self.definition.difficulty)
            .map(|value| value.season_id)
            .ok_or_else(|| error("Currency Wars service difficulty is missing"))
    }

    fn require_consumable(
        &self,
        item: CurrencyWarsItemId,
        kind: CurrencyWarsConsumableKind,
    ) -> Result<CurrencyWarsConsumableDefinition, CurrencyWarsRuntimeError> {
        let consumable = self
            .definition
            .catalog
            .service_catalog()
            .consumable(item)
            .cloned()
            .ok_or_else(|| error("Currency Wars consumable is missing"))?;
        if consumable.kind != kind {
            return Err(error("Currency Wars consumable has the wrong function"));
        }
        Ok(consumable)
    }

    fn apply_service_equipment_state(
        &mut self,
        id: u32,
        items: &BTreeMap<CurrencyWarsItemId, u32>,
        equipment: &BTreeMap<CurrencyWarsEquipmentId, u32>,
        loadout: &CurrencyWarsEquipmentLoadout,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let bonds = self.bond_snapshot_for(&self.deployment()?, loadout)?;
        let mut operations = vec![
            set_counter_map(ITEM_INVENTORY, encode_item_inventory(items)),
            set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(equipment)),
            set_counter_map(EQUIPMENT_LOADOUT, loadout.encoded()),
        ];
        operations.extend(bond_operations(&bonds));
        self.apply_state(id, operations)
    }

    fn apply_service_roster_state(
        &mut self,
        id: u32,
        items: &BTreeMap<CurrencyWarsItemId, u32>,
        roster: &CurrencyWarsRoster,
        deployment: &CurrencyWarsDeployment,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let mut equipment = self.equipment_inventory()?;
        let mut loadout = self.equipment_loadout()?;
        let removed_roles = loadout
            .slots()
            .keys()
            .map(|slot| slot.role())
            .filter(|role| !roster.owns_role(*role))
            .collect::<std::collections::BTreeSet<_>>();
        for role in removed_roles {
            let removed = loadout
                .for_role(role)
                .map(|(_, equipment)| equipment)
                .collect::<Vec<_>>();
            loadout.remove_role(role);
            for value in removed {
                add_equipment_inventory(&mut equipment, value)?;
            }
        }
        let bonds = self.bond_snapshot_for(deployment, &loadout)?;
        let mut operations = vec![
            set_counter_map(ITEM_INVENTORY, encode_item_inventory(items)),
            set_counter_map(ROSTER, roster.encoded()),
            set_counter_map(DEPLOYMENT, deployment.encoded()),
            set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(&equipment)),
            set_counter_map(EQUIPMENT_LOADOUT, loadout.encoded()),
        ];
        operations.extend(bond_operations(&bonds));
        self.apply_state(id, operations)
    }
}

fn forge_candidates(
    catalog: &CurrencyWarsCatalog,
    category: CurrencyWarsEquipmentCategory,
    target: CurrencyWarsForgeTarget,
) -> Vec<CurrencyWarsForgeOffer> {
    match target {
        CurrencyWarsForgeTarget::Equipment => catalog
            .build_catalog()
            .equipment()
            .iter()
            .filter_map(|definition| definition.runtime.as_ref())
            .filter(|definition| definition.category == category)
            .map(|definition| CurrencyWarsForgeOffer::Equipment(definition.id))
            .collect(),
        CurrencyWarsForgeTarget::Role { rarity, star } => catalog
            .roles()
            .iter()
            .filter(|role| role.rarity == rarity && catalog.role_available(role.id))
            .map(|role| CurrencyWarsForgeOffer::Role {
                role: role.id,
                star,
            })
            .collect(),
        CurrencyWarsForgeTarget::Expert { minimum, maximum } => catalog
            .roles()
            .iter()
            .filter(|role| {
                role.rarity >= minimum && role.rarity <= maximum && catalog.role_available(role.id)
            })
            .map(|role| CurrencyWarsForgeOffer::Role {
                role: role.id,
                star: 1,
            })
            .collect(),
    }
}

pub(super) fn grant_role(
    catalog: &CurrencyWarsCatalog,
    roster: &mut CurrencyWarsRoster,
    role: CurrencyWarsRoleId,
    star: u8,
) -> Result<(), CurrencyWarsRuntimeError> {
    if !catalog.role_available(role) {
        return Err(error(
            "Currency Wars reward role is excluded by the active season or module",
        ));
    }
    let copies = catalog
        .star_copy_count(role, star)
        .ok_or_else(|| error("Currency Wars reward role star is missing"))?;
    for _ in 0..copies {
        *roster = roster.acquire(catalog, role).map_err(debug_error)?;
    }
    Ok(())
}

pub(super) fn validate_service_roster(
    catalog: &CurrencyWarsCatalog,
    roster: &CurrencyWarsRoster,
    deployment: &CurrencyWarsDeployment,
    team_level: u8,
    back_capacity: u8,
) -> Result<(), CurrencyWarsRuntimeError> {
    deployment
        .validate_service_overflow(catalog, roster, team_level, back_capacity)
        .map_err(debug_error)
}

fn decode_item_inventory(
    values: Vec<(u64, i64)>,
) -> Result<BTreeMap<CurrencyWarsItemId, u32>, CurrencyWarsRuntimeError> {
    values
        .into_iter()
        .map(|(raw_id, raw_count)| {
            let id = u32::try_from(raw_id)
                .ok()
                .and_then(CurrencyWarsItemId::new)
                .ok_or_else(|| error("Currency Wars item inventory ID is invalid"))?;
            let count = u32::try_from(raw_count)
                .map_err(|_| error("Currency Wars item inventory count is invalid"))?;
            Ok((id, count))
        })
        .collect()
}

pub(super) fn encode_item_inventory(
    inventory: &BTreeMap<CurrencyWarsItemId, u32>,
) -> Box<[(u64, i64)]> {
    inventory
        .iter()
        .map(|(id, count)| (u64::from(id.get()), i64::from(*count)))
        .collect()
}

fn decode_special_good_activations(
    values: Vec<(u64, i64)>,
) -> Result<BTreeMap<u32, u32>, CurrencyWarsRuntimeError> {
    values
        .into_iter()
        .map(|(raw_id, raw_count)| {
            let id = u32::try_from(raw_id)
                .map_err(|_| error("Currency Wars special-good activation ID is invalid"))?;
            let count = u32::try_from(raw_count)
                .map_err(|_| error("Currency Wars special-good activation count is invalid"))?;
            Ok((id, count))
        })
        .collect()
}

fn encode_special_good_activations(inventory: &BTreeMap<u32, u32>) -> Box<[(u64, i64)]> {
    inventory
        .iter()
        .map(|(id, count)| (u64::from(*id), i64::from(*count)))
        .collect()
}

pub(super) fn add_item(
    inventory: &mut BTreeMap<CurrencyWarsItemId, u32>,
    item: CurrencyWarsItemId,
    count: u32,
) -> Result<(), CurrencyWarsRuntimeError> {
    let value = inventory.entry(item).or_default();
    *value = value
        .checked_add(count)
        .ok_or_else(|| error("Currency Wars item count overflow"))?;
    Ok(())
}

fn consume_item(
    inventory: &mut BTreeMap<CurrencyWarsItemId, u32>,
    item: CurrencyWarsItemId,
    consume: bool,
) -> Result<(), CurrencyWarsRuntimeError> {
    let count = inventory
        .get_mut(&item)
        .ok_or_else(|| error("Currency Wars consumable is not owned"))?;
    if consume {
        *count = count
            .checked_sub(1)
            .ok_or_else(|| error("Currency Wars item count underflow"))?;
        if *count == 0 {
            inventory.remove(&item);
        }
    }
    Ok(())
}

pub(super) fn invalid_generated_boundary() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
