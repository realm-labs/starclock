use starclock_activity::{
    ActivityOperation, ActivityPlayerView, ActivityRngLabel, ActivityRngStreams, ActivityValue,
    GraphActivityCommandError, GraphActivityRuntimeError,
};

use super::{
    CURRENT_CHAPTER, CURRENT_SECTION, CurrencyWarsRun, CurrencyWarsRunDefinition,
    CurrencyWarsRuntimeError, DEPLOYMENT, EXPERIENCE, FREE_REFRESHES, GOLD, LOCKED_SHOP_OFFERS,
    ROSTER, SHOP_LOCKED, SHOP_OFFERS, TEAM_LEVEL, add_integer, advance_team_level, bond_operations,
    debug_error, error, program_id, set_counter_map, set_integer, set_ordered_ids, set_value, slot,
};
use crate::{
    CurrencyWarsRoleId, CurrencyWarsRoleState, CurrencyWarsRoster, CurrencyWarsRunPosition,
};

const SHOP_RARITY_PURPOSE: u16 = 1;
const SHOP_ROLE_PURPOSE: u16 = 2;
const SHOP_SLOT_SHIFT: u32 = 32;

struct ShopGenerationContext {
    rarity_weights: [u32; 5],
    groups: [Vec<(CurrencyWarsRoleId, u32)>; 5],
    width: u8,
    role_weight: u64,
    locked: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsShopOffer {
    slot: u8,
    role: CurrencyWarsRoleId,
}

impl CurrencyWarsShopOffer {
    pub fn new(slot: u8, role: CurrencyWarsRoleId) -> Result<Self, CurrencyWarsRuntimeError> {
        if slot == 0 {
            return Err(error("Currency Wars shop slot is zero"));
        }
        Ok(Self { slot, role })
    }

    #[must_use]
    pub const fn slot(self) -> u8 {
        self.slot
    }

    #[must_use]
    pub const fn role(self) -> CurrencyWarsRoleId {
        self.role
    }

    #[must_use]
    pub const fn encode(self) -> u64 {
        (self.slot as u64) << SHOP_SLOT_SHIFT | self.role.get() as u64
    }

    fn decode(raw: u64) -> Result<Self, CurrencyWarsRuntimeError> {
        let slot = u8::try_from(raw >> SHOP_SLOT_SHIFT)
            .map_err(|_| error("Currency Wars shop slot is invalid"))?;
        let role = u32::try_from(raw & u64::from(u32::MAX))
            .ok()
            .and_then(CurrencyWarsRoleId::new)
            .ok_or_else(|| error("Currency Wars shop role is invalid"))?;
        Self::new(slot, role)
    }
}

impl CurrencyWarsRun {
    pub fn refresh_shop(
        &mut self,
    ) -> Result<Box<[CurrencyWarsShopOffer]>, CurrencyWarsRuntimeError> {
        if self.free_refreshes() > 0 {
            return self.generate_shop(0, true, true);
        }
        let cost = self.definition.catalog.refresh_cost();
        if self.gold() < cost {
            return Err(error("Currency Wars refresh requires more Gold"));
        }
        self.generate_shop(cost, true, false)
    }

    pub fn current_shop_offers(
        &self,
    ) -> Result<Box<[CurrencyWarsShopOffer]>, CurrencyWarsRuntimeError> {
        self.shop_offers().map(Vec::into_boxed_slice)
    }

    pub fn buy_shop_offer(
        &mut self,
        offer: CurrencyWarsShopOffer,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let mut offers = self.shop_offers()?;
        let index = offers
            .binary_search(&offer)
            .map_err(|_| error("Currency Wars shop offer is not current"))?;
        let definition = self
            .definition
            .catalog
            .role(offer.role)
            .ok_or_else(|| error("Currency Wars role is missing"))?;
        let cost = self
            .definition
            .catalog
            .price(definition.rarity)
            .and_then(|price| price.buy(1))
            .ok_or_else(|| error("Currency Wars role buy price is missing"))?;
        if self.gold() < cost {
            return Err(error("Currency Wars role purchase requires more Gold"));
        }
        let roster = self
            .roster()?
            .acquire(&self.definition.catalog, offer.role)
            .map_err(debug_error)?;
        let deployment = self
            .deployment()?
            .reconcile_acquisition(&roster, offer.role);
        deployment
            .validate_with_back_capacity(
                &self.definition.catalog,
                &roster,
                self.team_level(),
                self.back_capacity(),
            )
            .map_err(debug_error)?;
        offers.remove(index);
        if roster.reached_maximum_star(&self.definition.catalog, offer.role) {
            offers.retain(|candidate| candidate.role != offer.role);
        }
        let encoded = encode_offers(&offers);
        let snapshot = self.bond_snapshot_for(&deployment, &self.equipment_loadout()?)?;
        let mut operations = vec![
            add_integer(GOLD, -i64::from(cost)),
            set_counter_map(ROSTER, roster.encoded()),
            set_counter_map(DEPLOYMENT, deployment.encoded()),
            set_ordered_ids(SHOP_OFFERS, encoded.clone()),
        ];
        operations.extend(bond_operations(&snapshot));
        if self.shop_locked()? {
            operations.push(set_ordered_ids(LOCKED_SHOP_OFFERS, encoded));
        }
        self.apply_state(101, operations)
    }

    pub fn sell_role(
        &mut self,
        state: CurrencyWarsRoleState,
    ) -> Result<(), CurrencyWarsRuntimeError> {
        let definition = self
            .definition
            .catalog
            .role(state.role())
            .ok_or_else(|| error("Currency Wars role is missing"))?;
        let price = self
            .definition
            .catalog
            .price(definition.rarity)
            .and_then(|rule| rule.sell(state.star()))
            .ok_or_else(|| error("Currency Wars role sell price is missing"))?;
        let roster = self.roster()?.sell(state).map_err(debug_error)?;
        let deployment = self.deployment()?.reconcile_roster(&roster);
        self.apply_roster_state(102, &roster, &deployment, i64::from(price))
    }

    pub fn buy_experience(&mut self) -> Result<(), CurrencyWarsRuntimeError> {
        let current = self
            .definition
            .catalog
            .team_level(self.team_level())
            .ok_or_else(|| error("Currency Wars team level is missing"))?;
        if current.experience_to_next.is_none() {
            return Err(error("Currency Wars team level is already maximum"));
        }
        let cost = self.definition.catalog.direct_experience_cost();
        let affix_loss = self
            .definition
            .enemy_affixes
            .growing_pains_gold_loss(current.level)
            .map_err(debug_error)?;
        let total_cost = cost
            .checked_add(affix_loss)
            .ok_or_else(|| error("Currency Wars experience purchase cost overflow"))?;
        if self.gold() < total_cost {
            return Err(error("Currency Wars level purchase requires more Gold"));
        }
        let total = self
            .experience()
            .checked_add(self.definition.catalog.direct_experience_gain())
            .ok_or_else(|| error("Currency Wars experience overflow"))?;
        let (level, experience) =
            advance_team_level(&self.definition.catalog, self.team_level(), total)
                .map_err(debug_error)?;
        self.apply_state(
            105,
            vec![
                add_integer(GOLD, -i64::from(total_cost)),
                set_integer(TEAM_LEVEL, i64::from(level)),
                set_integer(EXPERIENCE, i64::from(experience)),
            ],
        )
    }

    pub fn set_shop_locked(&mut self, locked: bool) -> Result<(), CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let mut operations = vec![set_value(SHOP_LOCKED, ActivityValue::Boolean(locked))];
        operations.push(set_ordered_ids(
            LOCKED_SHOP_OFFERS,
            if locked {
                encode_offers(&self.shop_offers()?)
            } else {
                Box::new([])
            },
        ));
        self.apply_state(108, operations)
    }

    #[must_use]
    pub fn is_shop_locked(&self) -> bool {
        self.shop_locked().unwrap_or_default()
    }

    pub(super) fn synchronize_current_node_shop(&mut self) -> Result<(), CurrencyWarsRuntimeError> {
        if self
            .definition
            .flow
            .route_index(self.activity.current_node())
            .is_none()
            || self.activity.player_view().decision().is_none()
        {
            return Ok(());
        }
        if self.shop_locked()? {
            let offers = self.locked_shop_offers()?;
            self.apply_state(
                109,
                vec![set_ordered_ids(SHOP_OFFERS, encode_offers(&offers))],
            )
        } else {
            self.generate_shop(0, false, false).map(drop)
        }
    }

    fn generate_shop(
        &mut self,
        cost: u32,
        require_full_offer: bool,
        consume_free_refresh: bool,
    ) -> Result<Box<[CurrencyWarsShopOffer]>, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let context = shop_generation_context(&self.definition, &self.activity.player_view())?
            .ok_or_else(|| error("Currency Wars shop generation is not active"))?;
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(100), move |rng| {
                generate_shop_operations(
                    context,
                    cost,
                    require_full_offer,
                    consume_free_refresh,
                    rng,
                )
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    pub(super) fn require_active_decision(&self) -> Result<(), CurrencyWarsRuntimeError> {
        let view = self.activity.player_view();
        if view.terminal().is_some() || view.decision().is_none() {
            return Err(error(
                "Currency Wars economy command is not currently offered",
            ));
        }
        Ok(())
    }

    fn shop_offers(&self) -> Result<Vec<CurrencyWarsShopOffer>, CurrencyWarsRuntimeError> {
        decode_offer_slot(self.value(SHOP_OFFERS)?)
    }

    fn locked_shop_offers(&self) -> Result<Vec<CurrencyWarsShopOffer>, CurrencyWarsRuntimeError> {
        decode_offer_slot(self.value(LOCKED_SHOP_OFFERS)?)
    }

    fn shop_locked(&self) -> Result<bool, CurrencyWarsRuntimeError> {
        match self.value(SHOP_LOCKED)? {
            ActivityValue::Boolean(value) => Ok(value),
            _ => Err(error("Currency Wars shop-lock slot has the wrong type")),
        }
    }
}

pub(super) fn settlement_shop_operations(
    definition: &CurrencyWarsRunDefinition,
    view: &ActivityPlayerView,
    rng: &mut ActivityRngStreams,
) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
    if definition.flow.route_index(view.current_node()).is_none() || view.decision().is_none() {
        return Ok(Vec::new());
    }
    if view_boolean(view, SHOP_LOCKED).map_err(|_| invalid_generated_boundary())? {
        let locked =
            view_ordered_ids(view, LOCKED_SHOP_OFFERS).map_err(|_| invalid_generated_boundary())?;
        return Ok(vec![set_ordered_ids(SHOP_OFFERS, locked)]);
    }
    let context = shop_generation_context(definition, view)
        .map_err(|_| invalid_generated_boundary())?
        .ok_or_else(invalid_generated_boundary)?;
    generate_shop_operations(context, 0, false, false, rng).map(|(operations, _)| operations)
}

fn shop_generation_context(
    definition: &CurrencyWarsRunDefinition,
    view: &ActivityPlayerView,
) -> Result<Option<ShopGenerationContext>, CurrencyWarsRuntimeError> {
    if definition.flow.route_index(view.current_node()).is_none() || view.decision().is_none() {
        return Ok(None);
    }
    let team_level = u8::try_from(view_integer(view, TEAM_LEVEL)?)
        .map_err(|_| error("Currency Wars team level is invalid"))?;
    let position = CurrencyWarsRunPosition::new(
        u8::try_from(view_integer(view, CURRENT_CHAPTER)?)
            .map_err(|_| error("Currency Wars current chapter is invalid"))?,
        u8::try_from(view_integer(view, CURRENT_SECTION)?)
            .map_err(|_| error("Currency Wars current section is invalid"))?,
    )
    .map_err(debug_error)?;
    let roster = CurrencyWarsRoster::new(
        &definition.catalog,
        view_counter_map(view, ROSTER)?
            .into_iter()
            .map(|(state, count)| {
                Ok((
                    CurrencyWarsRoleState::decode(state).map_err(debug_error)?,
                    u32::try_from(count).map_err(debug_error)?,
                ))
            })
            .collect::<Result<Vec<_>, CurrencyWarsRuntimeError>>()?,
    )
    .map_err(debug_error)?;
    let offer = definition
        .catalog
        .offer(team_level)
        .ok_or_else(|| error("Currency Wars offer level is missing"))?;
    let mut groups: [Vec<(CurrencyWarsRoleId, u32)>; 5] = std::array::from_fn(|_| Vec::new());
    for role in &offer.candidates {
        if !definition.catalog.role_available(*role) {
            continue;
        }
        let role_definition = definition
            .catalog
            .role(*role)
            .ok_or_else(|| error("Currency Wars offer role is missing"))?;
        if !definition
            .catalog
            .progression_catalog()
            .role_cost_available(definition.gambit, role_definition.rarity, position)
            || roster.reached_maximum_star(&definition.catalog, *role)
        {
            continue;
        }
        let authored = definition
            .catalog
            .copies_per_role(role_definition.rarity)
            .ok_or_else(|| error("Currency Wars offer rarity is invalid"))?;
        let owned = roster
            .base_copy_count(&definition.catalog, *role)
            .map_err(debug_error)?;
        let remaining = authored
            .checked_sub(owned)
            .ok_or_else(|| error("Currency Wars roster exceeds the authored role pool"))?;
        if remaining > 0 {
            groups[usize::from(role_definition.rarity - 1)].push((*role, remaining));
        }
    }
    for group in &mut groups {
        group.sort_by_key(|candidate| candidate.0);
    }
    Ok(Some(ShopGenerationContext {
        rarity_weights: offer.rarity_weights,
        groups,
        width: definition.catalog.cards_per_refresh(),
        role_weight: u64::from(definition.catalog.role_offer_initial_weight()),
        locked: view_boolean(view, SHOP_LOCKED)?,
    }))
}

fn generate_shop_operations(
    mut context: ShopGenerationContext,
    cost: u32,
    require_full_offer: bool,
    consume_free_refresh: bool,
    rng: &mut ActivityRngStreams,
) -> Result<(Vec<ActivityOperation>, Box<[CurrencyWarsShopOffer]>), GraphActivityCommandError> {
    let mut selected = Vec::with_capacity(usize::from(context.width));
    for slot_index in 1..=context.width {
        let rarity_weights = context
            .rarity_weights
            .iter()
            .enumerate()
            .map(|(index, weight)| {
                u64::from(*weight)
                    * u64::from(
                        context.groups[index]
                            .iter()
                            .any(|candidate| candidate.1 > 0),
                    )
            })
            .collect::<Vec<_>>();
        let rarity = rng
            .choose_weighted(ActivityRngLabel::Shop, SHOP_RARITY_PURPOSE, &rarity_weights)
            .map_err(GraphActivityCommandError::Rng)?;
        let Some((rarity, _)) = rarity else {
            if require_full_offer {
                return Err(invalid_generated_boundary());
            }
            break;
        };
        let candidate_weights = context.groups[rarity as usize]
            .iter()
            .map(|candidate| u64::from(candidate.1).saturating_mul(context.role_weight))
            .collect::<Vec<_>>();
        let (candidate, _) = rng
            .choose_weighted(
                ActivityRngLabel::Shop,
                SHOP_ROLE_PURPOSE,
                &candidate_weights,
            )
            .map_err(GraphActivityCommandError::Rng)?
            .ok_or_else(invalid_generated_boundary)?;
        let chosen = &mut context.groups[rarity as usize][candidate as usize];
        chosen.1 = chosen
            .1
            .checked_sub(1)
            .ok_or_else(invalid_generated_boundary)?;
        selected.push(
            CurrencyWarsShopOffer::new(slot_index, chosen.0)
                .map_err(|_| invalid_generated_boundary())?,
        );
    }
    let encoded = encode_offers(&selected);
    let mut operations = Vec::with_capacity(if context.locked { 4 } else { 3 });
    if cost > 0 {
        operations.push(add_integer(GOLD, -i64::from(cost)));
    }
    if consume_free_refresh {
        operations.push(add_integer(FREE_REFRESHES, -1));
    }
    operations.push(set_ordered_ids(SHOP_OFFERS, encoded.clone()));
    if context.locked {
        operations.push(set_ordered_ids(LOCKED_SHOP_OFFERS, encoded));
    }
    Ok((operations, selected.into_boxed_slice()))
}

fn view_value(
    view: &ActivityPlayerView,
    raw: u32,
) -> Result<ActivityValue, CurrencyWarsRuntimeError> {
    view.slots()
        .iter()
        .find(|entry| entry.id() == slot(raw))
        .map(|entry| entry.value().clone())
        .ok_or_else(|| error("Currency Wars state slot is missing"))
}

fn view_integer(view: &ActivityPlayerView, raw: u32) -> Result<i64, CurrencyWarsRuntimeError> {
    match view_value(view, raw)? {
        ActivityValue::BoundedInteger(value) => Ok(value),
        _ => Err(error("Currency Wars integer slot has the wrong type")),
    }
}

fn view_counter_map(
    view: &ActivityPlayerView,
    raw: u32,
) -> Result<Vec<(u64, i64)>, CurrencyWarsRuntimeError> {
    match view_value(view, raw)? {
        ActivityValue::BoundedCounterMap(values) => Ok(values.into_vec()),
        _ => Err(error("Currency Wars counter slot has the wrong type")),
    }
}

fn view_boolean(view: &ActivityPlayerView, raw: u32) -> Result<bool, CurrencyWarsRuntimeError> {
    match view_value(view, raw)? {
        ActivityValue::Boolean(value) => Ok(value),
        _ => Err(error("Currency Wars Boolean slot has the wrong type")),
    }
}

fn view_ordered_ids(
    view: &ActivityPlayerView,
    raw: u32,
) -> Result<Box<[u64]>, CurrencyWarsRuntimeError> {
    match view_value(view, raw)? {
        ActivityValue::OrderedIdSet(values) => Ok(values),
        _ => Err(error("Currency Wars ordered-ID slot has the wrong type")),
    }
}

fn decode_offer_slot(
    value: ActivityValue,
) -> Result<Vec<CurrencyWarsShopOffer>, CurrencyWarsRuntimeError> {
    match value {
        ActivityValue::OrderedIdSet(values) => values
            .iter()
            .map(|raw| CurrencyWarsShopOffer::decode(*raw))
            .collect(),
        _ => Err(error("Currency Wars shop-offer slot has the wrong type")),
    }
}

fn encode_offers(offers: &[CurrencyWarsShopOffer]) -> Box<[u64]> {
    offers
        .iter()
        .map(|offer| offer.encode())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn invalid_generated_boundary() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
