use std::{collections::BTreeMap, sync::Arc};

use starclock_activity::{ActivityRngLabel, GraphActivityCommandError};

use super::service::{
    CurrencyWarsAppliedReward, CurrencyWarsRewardPoolResolution, add_item, encode_item_inventory,
    grant_role, invalid_generated_boundary, validate_service_roster,
};
use super::{
    CurrencyWarsRun, CurrencyWarsRuntimeError, DEPLOYMENT, EQUIPMENT_INVENTORY, EXPERIENCE,
    FREE_REFRESHES, GOLD, INVESTMENTS, ITEM_INVENTORY, ROSTER, TEAM_LEVEL, add_equipment_inventory,
    advance_team_level, bond_operations, debug_error, encode_equipment_inventory, error,
    program_id, set_counter_map, set_integer, set_ordered_ids,
};
use crate::{
    CurrencyWarsCatalog, CurrencyWarsEquipmentId, CurrencyWarsRewardKind, CurrencyWarsRewardPool,
    CurrencyWarsRoleId, CurrencyWarsRuntimeEquipment, equipment_category_from_selector,
};

const REWARD_POOL_PURPOSE: u16 = 30;
const REWARD_ROLE_PURPOSE: u16 = 31;
const DIRECT_REWARD_ROLE_PURPOSE: u16 = 35;
const DIRECT_REWARD_EQUIPMENT_PURPOSE: u16 = 36;

impl CurrencyWarsRun {
    pub fn resolve_reward_pool(
        &mut self,
        pool_id: u32,
    ) -> Result<CurrencyWarsRewardPoolResolution, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let pool = self
            .definition
            .catalog
            .service_catalog()
            .reward_pool(pool_id)
            .cloned()
            .ok_or_else(|| error("Currency Wars reward pool is missing"))?;
        if !pool.candidates.iter().any(|candidate| {
            self.definition
                .catalog
                .service_catalog()
                .reward(candidate.reward_id)
                .and_then(|reward| reward.budget_cost)
                .is_some_and(|cost| cost <= pool.total_value)
        }) {
            return Ok(CurrencyWarsRewardPoolResolution {
                selected_reward_ids: Box::new([]),
                remaining_value: pool.total_value,
            });
        }
        let catalog = Arc::clone(&self.definition.catalog);
        let mut items = self.item_inventory()?;
        let mut roster = self.roster()?;
        let mut deployment = self.deployment()?;
        let loadout = self.equipment_loadout()?;
        let bond_context = self.bond_context()?;
        let team_level = self.team_level();
        let back_capacity = self.back_capacity();
        let mut gold = self.gold();
        let mut free_refreshes = self.free_refreshes();
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(136), move |rng| {
                let (selected_reward_ids, remaining_value) = select_pool(&pool, &catalog, rng)?;
                for reward_id in &selected_reward_ids {
                    let reward = catalog
                        .service_catalog()
                        .reward(*reward_id)
                        .ok_or_else(invalid_generated_boundary)?;
                    match &reward.kind {
                        CurrencyWarsRewardKind::DefaultCurrency => {
                            gold = gold
                                .checked_add(reward.scalar_parameter.unwrap_or(1))
                                .ok_or_else(invalid_generated_boundary)?;
                        }
                        CurrencyWarsRewardKind::Refresh => {
                            free_refreshes = free_refreshes
                                .checked_add(reward.scalar_parameter.unwrap_or(1))
                                .ok_or_else(invalid_generated_boundary)?;
                        }
                        CurrencyWarsRewardKind::Item { item, count } => {
                            add_item(&mut items, *item, *count)
                                .map_err(|_| invalid_generated_boundary())?;
                        }
                        CurrencyWarsRewardKind::RandomRole { rarity, star } => {
                            let candidates = catalog
                                .roles()
                                .iter()
                                .filter(|role| {
                                    role.rarity == *rarity
                                        && *star <= role.maximum_star
                                        && catalog.role_available(role.id)
                                })
                                .map(|role| role.id)
                                .collect::<Vec<_>>();
                            let draw = rng
                                .choose_index(
                                    ActivityRngLabel::Reward,
                                    REWARD_ROLE_PURPOSE,
                                    u32::try_from(candidates.len())
                                        .map_err(|_| invalid_generated_boundary())?,
                                )
                                .map_err(GraphActivityCommandError::Rng)?
                                .ok_or_else(invalid_generated_boundary)?;
                            let role = candidates[draw.value() as usize];
                            grant_role(&catalog, &mut roster, role, *star)
                                .map_err(|_| invalid_generated_boundary())?;
                            deployment = deployment.reconcile_acquisition(&roster, role);
                            validate_service_roster(
                                &catalog,
                                &roster,
                                &deployment,
                                team_level,
                                back_capacity,
                            )
                            .map_err(|_| invalid_generated_boundary())?;
                        }
                        _ => return Err(invalid_generated_boundary()),
                    }
                }
                let bonds = catalog
                    .bond_catalog()
                    .resolve(&deployment, &loadout, &bond_context);
                let mut operations = vec![
                    set_integer(GOLD, i64::from(gold)),
                    set_integer(FREE_REFRESHES, i64::from(free_refreshes)),
                    set_counter_map(ITEM_INVENTORY, encode_item_inventory(&items)),
                    set_counter_map(ROSTER, roster.encoded()),
                    set_counter_map(DEPLOYMENT, deployment.encoded()),
                ];
                operations.extend(bond_operations(&bonds));
                Ok((
                    operations,
                    CurrencyWarsRewardPoolResolution {
                        selected_reward_ids: selected_reward_ids.into_boxed_slice(),
                        remaining_value,
                    },
                ))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }

    pub fn apply_reward(
        &mut self,
        reward_id: u32,
    ) -> Result<CurrencyWarsAppliedReward, CurrencyWarsRuntimeError> {
        self.require_active_decision()?;
        let reward = self
            .definition
            .catalog
            .service_catalog()
            .reward(reward_id)
            .cloned()
            .ok_or_else(|| error("Currency Wars reward is missing"))?;
        let catalog = Arc::clone(&self.definition.catalog);
        if reward_has_no_legal_result(&catalog, &reward.kind) {
            return Ok(CurrencyWarsAppliedReward::NoLegalResult);
        }
        let mut items = self.item_inventory()?;
        let mut equipment = self.equipment_inventory()?;
        let mut roster = self.roster()?;
        let mut deployment = self.deployment()?;
        let loadout = self.equipment_loadout()?;
        let bond_context = self.bond_context()?;
        let team_level = self.team_level();
        let back_capacity = self.back_capacity();
        let mut investments = self.ordered_ids(INVESTMENTS)?.into_vec();
        let mut gold = self.gold();
        let mut free_refreshes = self.free_refreshes();
        let mut level = self.team_level();
        let mut experience = self.experience();
        let resolution = self
            .activity
            .apply_generated_boundary(self.state_hash(), program_id(142), move |rng| {
                let applied = match reward.kind.clone() {
                    CurrencyWarsRewardKind::DefaultCurrency => {
                        let amount = reward.scalar_parameter.unwrap_or(1);
                        gold = gold
                            .checked_add(amount)
                            .ok_or_else(invalid_generated_boundary)?;
                        CurrencyWarsAppliedReward::Gold(amount)
                    }
                    CurrencyWarsRewardKind::Refresh => {
                        let count = reward.scalar_parameter.unwrap_or(1);
                        free_refreshes = free_refreshes
                            .checked_add(count)
                            .ok_or_else(invalid_generated_boundary)?;
                        CurrencyWarsAppliedReward::FreeRefresh(count)
                    }
                    CurrencyWarsRewardKind::Experience => {
                        let amount = reward.scalar_parameter.unwrap_or(1);
                        let total = experience
                            .checked_add(amount)
                            .ok_or_else(invalid_generated_boundary)?;
                        (level, experience) = advance_team_level(&catalog, level, total)
                            .map_err(|_| invalid_generated_boundary())?;
                        CurrencyWarsAppliedReward::Experience(amount)
                    }
                    CurrencyWarsRewardKind::Item { item, count } => {
                        add_item(&mut items, item, count)
                            .map_err(|_| invalid_generated_boundary())?;
                        CurrencyWarsAppliedReward::Item { item, count }
                    }
                    CurrencyWarsRewardKind::Orb(source_id) => {
                        let investment = catalog
                            .cross_investment_catalog()
                            .orbs()
                            .iter()
                            .find(|definition| {
                                definition
                                    .source_id
                                    .split('.')
                                    .next()
                                    .and_then(|value| value.parse::<u32>().ok())
                                    == Some(source_id)
                            })
                            .map(|definition| definition.investment)
                            .ok_or_else(invalid_generated_boundary)?;
                        if investments.binary_search(&investment.get()).is_ok() {
                            return Err(invalid_generated_boundary());
                        }
                        investments.push(investment.get());
                        investments.sort_unstable();
                        CurrencyWarsAppliedReward::Investment(investment)
                    }
                    CurrencyWarsRewardKind::RandomRole { rarity, star } => {
                        let candidates = catalog
                            .roles()
                            .iter()
                            .filter(|role| {
                                role.rarity == rarity
                                    && star <= role.maximum_star
                                    && catalog.role_available(role.id)
                            })
                            .map(|role| role.id)
                            .collect::<Vec<_>>();
                        let role = choose_role(rng, &candidates)?;
                        grant_role(&catalog, &mut roster, role, star)
                            .map_err(|_| invalid_generated_boundary())?;
                        deployment = deployment.reconcile_acquisition(&roster, role);
                        validate_service_roster(
                            &catalog,
                            &roster,
                            &deployment,
                            team_level,
                            back_capacity,
                        )
                        .map_err(|_| invalid_generated_boundary())?;
                        CurrencyWarsAppliedReward::Role { role, star }
                    }
                    CurrencyWarsRewardKind::SpecificAvatar { avatar_id, star } => {
                        let role = resolve_avatar_role(&catalog, avatar_id, star)
                            .ok_or_else(invalid_generated_boundary)?;
                        grant_role(&catalog, &mut roster, role, star)
                            .map_err(|_| invalid_generated_boundary())?;
                        deployment = deployment.reconcile_acquisition(&roster, role);
                        validate_service_roster(
                            &catalog,
                            &roster,
                            &deployment,
                            team_level,
                            back_capacity,
                        )
                        .map_err(|_| invalid_generated_boundary())?;
                        CurrencyWarsAppliedReward::Role { role, star }
                    }
                    CurrencyWarsRewardKind::RandomEquipmentByCategory(selector) => {
                        let category = equipment_category_from_selector(selector)
                            .ok_or_else(invalid_generated_boundary)?;
                        let candidates =
                            equipment_candidates(&catalog, |value| value.category == category);
                        let selected = choose_equipment(rng, &candidates, 1)?;
                        add_selected_equipment(&mut equipment, &selected)?;
                        CurrencyWarsAppliedReward::Equipment(selected.into_boxed_slice())
                    }
                    CurrencyWarsRewardKind::RandomEquipmentByFunction(tag) => {
                        let candidates =
                            equipment_candidates(&catalog, |value| value.tags.contains(&tag));
                        let selected = choose_equipment(rng, &candidates, 1)?;
                        add_selected_equipment(&mut equipment, &selected)?;
                        CurrencyWarsAppliedReward::Equipment(selected.into_boxed_slice())
                    }
                    CurrencyWarsRewardKind::SpecificAvatarWithEquipment {
                        avatar_id,
                        star,
                        equipment: selected,
                    } => {
                        let role = resolve_avatar_role(&catalog, avatar_id, star)
                            .ok_or_else(invalid_generated_boundary)?;
                        grant_role(&catalog, &mut roster, role, star)
                            .map_err(|_| invalid_generated_boundary())?;
                        deployment = deployment.reconcile_acquisition(&roster, role);
                        validate_service_roster(
                            &catalog,
                            &roster,
                            &deployment,
                            team_level,
                            back_capacity,
                        )
                        .map_err(|_| invalid_generated_boundary())?;
                        add_selected_equipment(&mut equipment, &selected)?;
                        CurrencyWarsAppliedReward::RoleWithEquipment {
                            role,
                            star,
                            equipment: selected.clone(),
                        }
                    }
                    CurrencyWarsRewardKind::SpecificAvatarWithRandomEquipment {
                        avatar_id,
                        star,
                        category_selector,
                        count,
                    } => {
                        let role = resolve_avatar_role(&catalog, avatar_id, star)
                            .ok_or_else(invalid_generated_boundary)?;
                        grant_role(&catalog, &mut roster, role, star)
                            .map_err(|_| invalid_generated_boundary())?;
                        deployment = deployment.reconcile_acquisition(&roster, role);
                        validate_service_roster(
                            &catalog,
                            &roster,
                            &deployment,
                            team_level,
                            back_capacity,
                        )
                        .map_err(|_| invalid_generated_boundary())?;
                        let category = equipment_category_from_selector(category_selector)
                            .ok_or_else(invalid_generated_boundary)?;
                        let candidates =
                            equipment_candidates(&catalog, |value| value.category == category);
                        let selected = choose_equipment(rng, &candidates, u16::from(count))?;
                        add_selected_equipment(&mut equipment, &selected)?;
                        CurrencyWarsAppliedReward::RoleWithEquipment {
                            role,
                            star,
                            equipment: selected.into_boxed_slice(),
                        }
                    }
                };
                let bonds = catalog
                    .bond_catalog()
                    .resolve(&deployment, &loadout, &bond_context);
                let mut operations = vec![
                    set_integer(GOLD, i64::from(gold)),
                    set_integer(FREE_REFRESHES, i64::from(free_refreshes)),
                    set_integer(TEAM_LEVEL, i64::from(level)),
                    set_integer(EXPERIENCE, i64::from(experience)),
                    set_counter_map(ITEM_INVENTORY, encode_item_inventory(&items)),
                    set_counter_map(EQUIPMENT_INVENTORY, encode_equipment_inventory(&equipment)),
                    set_counter_map(ROSTER, roster.encoded()),
                    set_counter_map(DEPLOYMENT, deployment.encoded()),
                    set_ordered_ids(INVESTMENTS, investments.into_boxed_slice()),
                ];
                operations.extend(bond_operations(&bonds));
                Ok((operations, applied))
            })
            .map_err(debug_error)?;
        Ok(resolution.into_value())
    }
}

fn select_pool(
    pool: &CurrencyWarsRewardPool,
    catalog: &CurrencyWarsCatalog,
    rng: &mut starclock_activity::ActivityRngStreams,
) -> Result<(Vec<u32>, u32), GraphActivityCommandError> {
    let mut remaining = pool.total_value;
    let mut counts = vec![0_u16; pool.candidates.len()];
    let mut selected = Vec::new();
    loop {
        let weights = pool
            .candidates
            .iter()
            .enumerate()
            .map(|(index, candidate)| {
                let cost = catalog
                    .service_catalog()
                    .reward(candidate.reward_id)
                    .and_then(|reward| reward.budget_cost);
                if counts[index] < candidate.maximum && cost.is_some_and(|cost| cost <= remaining) {
                    candidate.weight
                } else {
                    0
                }
            })
            .collect::<Vec<_>>();
        let Some((index, _)) = rng
            .choose_weighted(ActivityRngLabel::Reward, REWARD_POOL_PURPOSE, &weights)
            .map_err(GraphActivityCommandError::Rng)?
        else {
            break;
        };
        let index = index as usize;
        let candidate = pool.candidates[index];
        let cost = catalog
            .service_catalog()
            .reward(candidate.reward_id)
            .and_then(|reward| reward.budget_cost)
            .ok_or_else(invalid_generated_boundary)?;
        counts[index] = counts[index]
            .checked_add(1)
            .ok_or_else(invalid_generated_boundary)?;
        remaining = remaining
            .checked_sub(cost)
            .ok_or_else(invalid_generated_boundary)?;
        selected.push(candidate.reward_id);
    }
    Ok((selected, remaining))
}

fn resolve_avatar_role(
    catalog: &CurrencyWarsCatalog,
    avatar_id: u32,
    star: u8,
) -> Option<CurrencyWarsRoleId> {
    catalog
        .roles()
        .iter()
        .filter(|role| role.avatar_id == avatar_id && star <= role.maximum_star)
        .map(|role| role.id)
        .min()
}

fn reward_has_no_legal_result(
    catalog: &CurrencyWarsCatalog,
    reward: &CurrencyWarsRewardKind,
) -> bool {
    match reward {
        CurrencyWarsRewardKind::RandomRole { rarity, star } => !catalog
            .roles()
            .iter()
            .any(|role| role.rarity == *rarity && *star <= role.maximum_star),
        CurrencyWarsRewardKind::RandomEquipmentByCategory(selector) => {
            let Some(category) = equipment_category_from_selector(*selector) else {
                return true;
            };
            equipment_candidates(catalog, |value| value.category == category).is_empty()
        }
        CurrencyWarsRewardKind::RandomEquipmentByFunction(tag) => {
            equipment_candidates(catalog, |value| value.tags.contains(tag)).is_empty()
        }
        CurrencyWarsRewardKind::SpecificAvatarWithRandomEquipment {
            category_selector,
            count,
            ..
        } => {
            let Some(category) = equipment_category_from_selector(*category_selector) else {
                return true;
            };
            equipment_candidates(catalog, |value| value.category == category).len()
                < usize::from(*count)
        }
        _ => false,
    }
}

fn choose_role(
    rng: &mut starclock_activity::ActivityRngStreams,
    candidates: &[CurrencyWarsRoleId],
) -> Result<CurrencyWarsRoleId, GraphActivityCommandError> {
    let draw = rng
        .choose_index(
            ActivityRngLabel::Reward,
            DIRECT_REWARD_ROLE_PURPOSE,
            u32::try_from(candidates.len()).map_err(|_| invalid_generated_boundary())?,
        )
        .map_err(GraphActivityCommandError::Rng)?
        .ok_or_else(invalid_generated_boundary)?;
    Ok(candidates[draw.value() as usize])
}

fn equipment_candidates(
    catalog: &CurrencyWarsCatalog,
    predicate: impl Fn(&CurrencyWarsRuntimeEquipment) -> bool,
) -> Vec<CurrencyWarsEquipmentId> {
    catalog
        .build_catalog()
        .equipment()
        .iter()
        .filter_map(|definition| definition.runtime.as_ref())
        .filter(|definition| predicate(definition))
        .map(|definition| definition.id)
        .collect()
}

fn choose_equipment(
    rng: &mut starclock_activity::ActivityRngStreams,
    candidates: &[CurrencyWarsEquipmentId],
    count: u16,
) -> Result<Vec<CurrencyWarsEquipmentId>, GraphActivityCommandError> {
    if candidates.len() < usize::from(count) {
        return Err(invalid_generated_boundary());
    }
    let indices = rng
        .choose_weighted_without_replacement(
            ActivityRngLabel::Reward,
            DIRECT_REWARD_EQUIPMENT_PURPOSE,
            &vec![1; candidates.len()],
            count,
        )
        .map_err(GraphActivityCommandError::Rng)?;
    let mut selected = indices
        .iter()
        .map(|index| candidates[*index as usize])
        .collect::<Vec<_>>();
    selected.sort_unstable();
    Ok(selected)
}

fn add_selected_equipment(
    inventory: &mut BTreeMap<CurrencyWarsEquipmentId, u32>,
    selected: &[CurrencyWarsEquipmentId],
) -> Result<(), GraphActivityCommandError> {
    for value in selected {
        add_equipment_inventory(inventory, *value).map_err(|_| invalid_generated_boundary())?;
    }
    Ok(())
}
