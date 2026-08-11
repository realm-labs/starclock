use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BaseballerAdventureStrategyId, BaseballerCatalogError, BaseballerProfile, BaseballerProfileId,
    BaseballerShopUpgradeId, BaseballerStage, BaseballerStageId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerShopUpgradeKind {
    AddMazeBuff,
    InitWeaponLevel,
    AddAccessorySlot,
}

/// One exact price step in a profile-owned persistent shop upgrade.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerShopUpgrade {
    pub id: BaseballerShopUpgradeId,
    pub stable_key: Box<str>,
    pub profile: BaseballerProfileId,
    pub source_numeric_id: u32,
    pub purchase_level: u8,
    pub maximum_level: u8,
    pub kind: BaseballerShopUpgradeKind,
    pub currency_key: Box<str>,
    pub cost: i64,
    pub maze_buff_id: Option<u32>,
    pub maze_buff_parameters: Box<[Box<str>]>,
    pub shop_parameter_values: Box<[Box<str>]>,
    /// Whether this price step can be translated into authoritative runtime state.
    pub runtime_binding_exact: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerAdventureStrategyKind {
    Growth,
    Power,
    General,
    DemonKing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerAdventureStrategy {
    pub id: BaseballerAdventureStrategyId,
    pub stable_key: Box<str>,
    pub profile: BaseballerProfileId,
    pub kind: BaseballerAdventureStrategyKind,
    pub maximum_level: u8,
    pub unlock_quest_id: Option<u32>,
    pub selectable_periods: Box<[u8]>,
    pub influence_scope: Box<str>,
    pub maze_buff_id: u32,
    pub maze_buff_parameters: Box<[Box<str>]>,
    pub ability_binding: Box<str>,
    pub runtime_binding_exact: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerTeamBonus {
    pub stage: BaseballerStageId,
    pub profile: BaseballerProfileId,
    pub maze_buff_id: u32,
    pub level: u8,
    pub parameters: Box<[Box<str>]>,
    pub ability_binding: Box<str>,
    pub runtime_binding_exact: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BaseballerRuntimeCatalogContent {
    pub shop_upgrades: Vec<BaseballerShopUpgrade>,
    pub strategies: Vec<BaseballerAdventureStrategy>,
    pub team_bonuses: Vec<BaseballerTeamBonus>,
}

pub(super) fn validate_shop_upgrades(
    profiles: &[BaseballerProfile],
    upgrades: &[BaseballerShopUpgrade],
) -> Result<(), BaseballerCatalogError> {
    let mut stable_keys = BTreeSet::new();
    let mut groups = BTreeMap::<(BaseballerProfileId, u32), Vec<&BaseballerShopUpgrade>>::new();
    for upgrade in upgrades {
        let valid_binding = match upgrade.kind {
            BaseballerShopUpgradeKind::AddMazeBuff => {
                upgrade.maze_buff_id.is_some()
                    && !upgrade.maze_buff_parameters.is_empty()
                    && !upgrade.runtime_binding_exact
            }
            BaseballerShopUpgradeKind::InitWeaponLevel
            | BaseballerShopUpgradeKind::AddAccessorySlot => {
                upgrade.maze_buff_id.is_none()
                    && upgrade.maze_buff_parameters.is_empty()
                    && upgrade.runtime_binding_exact
                    && upgrade.shop_parameter_values.len() == 1
                    && upgrade.shop_parameter_values[0].as_ref() == "1"
            }
        };
        if !stable_keys.insert(upgrade.stable_key.as_ref())
            || find(profiles, upgrade.profile, |profile| profile.id).is_none()
            || upgrade.source_numeric_id == 0
            || upgrade.purchase_level == 0
            || upgrade.purchase_level > upgrade.maximum_level
            || upgrade.currency_key.is_empty()
            || upgrade.cost < 0
            || upgrade.shop_parameter_values.is_empty()
            || !valid_binding
        {
            return Err(BaseballerCatalogError::InvalidShopUpgrade);
        }
        groups
            .entry((upgrade.profile, upgrade.source_numeric_id))
            .or_default()
            .push(upgrade);
    }
    for group in groups.values_mut() {
        group.sort_by_key(|upgrade| upgrade.purchase_level);
        let first = group[0];
        if group.len() != usize::from(first.maximum_level)
            || group.iter().enumerate().any(|(index, upgrade)| {
                upgrade.purchase_level != u8::try_from(index + 1).unwrap_or(u8::MAX)
                    || upgrade.maximum_level != first.maximum_level
                    || upgrade.kind != first.kind
                    || upgrade.currency_key != first.currency_key
            })
        {
            return Err(BaseballerCatalogError::InvalidShopUpgradeSequence);
        }
    }
    Ok(())
}

pub(super) fn validate_strategies(
    profiles: &[BaseballerProfile],
    strategies: &[BaseballerAdventureStrategy],
) -> Result<(), BaseballerCatalogError> {
    let mut stable_keys = BTreeSet::new();
    for strategy in strategies {
        if !stable_keys.insert(strategy.stable_key.as_ref())
            || find(profiles, strategy.profile, |profile| profile.id).is_none()
            || strategy.maximum_level == 0
            || strategy
                .selectable_periods
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || strategy
                .selectable_periods
                .iter()
                .any(|period| !(1..=4).contains(period))
            || strategy.influence_scope.is_empty()
            || strategy.maze_buff_id == 0
            || strategy.ability_binding.is_empty()
        {
            return Err(BaseballerCatalogError::InvalidStrategy);
        }
    }
    Ok(())
}

pub(super) fn validate_team_bonuses(
    profiles: &[BaseballerProfile],
    stages: &[BaseballerStage],
    team_bonuses: &[BaseballerTeamBonus],
) -> Result<(), BaseballerCatalogError> {
    for bonus in team_bonuses {
        let stage = find(stages, bonus.stage, |stage| stage.id)
            .ok_or(BaseballerCatalogError::InvalidTeamBonus)?;
        if stage.profile != bonus.profile
            || find(profiles, bonus.profile, |profile| profile.id).is_none()
            || bonus.maze_buff_id == 0
            || bonus.level == 0
            || bonus.parameters.is_empty()
            || bonus.ability_binding.is_empty()
        {
            return Err(BaseballerCatalogError::InvalidTeamBonus);
        }
    }
    Ok(())
}

fn find<T, K: Ord>(items: &[T], key: K, item_key: impl Fn(&T) -> K) -> Option<&T> {
    items
        .binary_search_by_key(&key, item_key)
        .ok()
        .map(|index| &items[index])
}
