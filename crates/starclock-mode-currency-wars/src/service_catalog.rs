use std::collections::BTreeSet;

use crate::{CurrencyWarsDecimal, CurrencyWarsEquipmentCategory, CurrencyWarsEquipmentId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsItemId(u32);

impl CurrencyWarsItemId {
    #[must_use]
    pub const fn new(raw: u32) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsItemDefinition {
    pub id: CurrencyWarsItemId,
    pub stable_key: Box<str>,
    pub priority: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSpecialGood {
    pub id: u32,
    pub stable_key: Box<str>,
    pub group_id: u16,
    pub quality: u8,
    pub acquisition: CurrencyWarsSpecialGoodAcquisition,
    pub config_path: Box<str>,
    pub parameters: Box<[CurrencyWarsDecimal]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsSpecialGoodAcquisition {
    ShopPurchase { price: u32 },
    CyreneThreeStar,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsConsumableKind {
    RemoveEquipment,
    RerollEquipment,
    UpgradeEquipment,
    CopyRole,
    GainRecommendedEquipment,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsConsumableDefinition {
    pub item: CurrencyWarsItemId,
    pub stable_key: Box<str>,
    pub kind: CurrencyWarsConsumableKind,
    pub consume: bool,
    pub stack: bool,
    pub parameters: Box<[u32]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsManagedFunction {
    pub stable_key: Box<str>,
    pub function_id: Box<str>,
    pub unlock_id: u32,
    pub hidden_while_locked: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsRewardKind {
    DefaultCurrency,
    Refresh,
    Experience,
    Item {
        item: CurrencyWarsItemId,
        count: u32,
    },
    Orb(u32),
    RandomRole {
        rarity: u8,
        star: u8,
    },
    SpecificAvatar {
        avatar_id: u32,
        star: u8,
    },
    RandomEquipmentByCategory(u32),
    RandomEquipmentByFunction(u32),
    SpecificAvatarWithEquipment {
        avatar_id: u32,
        star: u8,
        equipment: Box<[CurrencyWarsEquipmentId]>,
    },
    SpecificAvatarWithRandomEquipment {
        avatar_id: u32,
        star: u8,
        category_selector: u32,
        count: u8,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRewardDefinition {
    pub id: u32,
    pub stable_key: Box<str>,
    pub budget_cost: Option<u32>,
    pub scalar_parameter: Option<u32>,
    pub kind: CurrencyWarsRewardKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRewardPoolCandidate {
    pub reward_id: u32,
    pub maximum: u16,
    pub weight: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRewardPool {
    pub id: u32,
    pub stable_key: Box<str>,
    pub total_value: u32,
    pub candidates: Box<[CurrencyWarsRewardPoolCandidate]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentRecipe {
    pub id: u32,
    pub stable_key: Box<str>,
    pub season_id: u16,
    pub output: CurrencyWarsEquipmentId,
    pub inputs: Box<[CurrencyWarsEquipmentId]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEquipmentUpgrade {
    pub source: CurrencyWarsEquipmentId,
    pub output: CurrencyWarsEquipmentId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrencyWarsForgeTarget {
    Equipment,
    Role { rarity: u8, star: u8 },
    Expert { minimum: u8, maximum: u8 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsForgeService {
    pub item: CurrencyWarsItemId,
    pub stable_key: Box<str>,
    pub category: CurrencyWarsEquipmentCategory,
    pub offer_count: u8,
    pub target: CurrencyWarsForgeTarget,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsServiceConstant {
    pub name: Box<str>,
    pub value: CurrencyWarsServiceConstantValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsServiceConstantValue {
    Integer(u32),
    IntegerArray(Box<[u32]>),
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsProvenEmptyServiceFamily {
    GambleGroups,
    GambleUnits,
    CurseChests,
    HexStates,
    HexEligibility,
    Curios,
    CurioGroups,
    CurioStates,
    CurioLifecycle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsServiceCatalog {
    items: Box<[CurrencyWarsItemDefinition]>,
    special_goods: Box<[CurrencyWarsSpecialGood]>,
    season_items: BTreeSet<CurrencyWarsItemId>,
    consumables: Box<[CurrencyWarsConsumableDefinition]>,
    managed_functions: Box<[CurrencyWarsManagedFunction]>,
    rewards: Box<[CurrencyWarsRewardDefinition]>,
    reward_pools: Box<[CurrencyWarsRewardPool]>,
    recipes: Box<[CurrencyWarsEquipmentRecipe]>,
    upgrades: Box<[CurrencyWarsEquipmentUpgrade]>,
    forge_services: Box<[CurrencyWarsForgeService]>,
    constants: Box<[CurrencyWarsServiceConstant]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsServiceCatalogParts {
    pub items: Vec<CurrencyWarsItemDefinition>,
    pub special_goods: Vec<CurrencyWarsSpecialGood>,
    pub season_items: BTreeSet<CurrencyWarsItemId>,
    pub consumables: Vec<CurrencyWarsConsumableDefinition>,
    pub managed_functions: Vec<CurrencyWarsManagedFunction>,
    pub rewards: Vec<CurrencyWarsRewardDefinition>,
    pub reward_pools: Vec<CurrencyWarsRewardPool>,
    pub recipes: Vec<CurrencyWarsEquipmentRecipe>,
    pub upgrades: Vec<CurrencyWarsEquipmentUpgrade>,
    pub forge_services: Vec<CurrencyWarsForgeService>,
    pub constants: Vec<CurrencyWarsServiceConstant>,
    pub gamble_group_count: usize,
    pub gamble_unit_count: usize,
    pub curse_chest_count: usize,
    pub hex_state_count: usize,
    pub hex_eligibility_count: usize,
    pub curio_count: usize,
    pub curio_group_count: usize,
    pub curio_state_count: usize,
    pub curio_lifecycle_count: usize,
}

impl CurrencyWarsServiceCatalog {
    pub fn new(
        mut parts: CurrencyWarsServiceCatalogParts,
    ) -> Result<Self, CurrencyWarsServiceCatalogError> {
        parts.items.sort_by_key(|value| value.id);
        parts.special_goods.sort_by_key(|value| value.id);
        parts.consumables.sort_by_key(|value| value.item);
        parts
            .managed_functions
            .sort_by(|left, right| left.function_id.cmp(&right.function_id));
        parts.rewards.sort_by_key(|value| value.id);
        parts.reward_pools.sort_by_key(|value| value.id);
        parts.recipes.sort_by_key(|value| value.id);
        parts.upgrades.sort_by_key(|value| value.source);
        parts.forge_services.sort_by_key(|value| value.item);
        parts
            .constants
            .sort_by(|left, right| left.name.cmp(&right.name));
        validate(&parts)?;
        Ok(Self {
            items: parts.items.into_boxed_slice(),
            special_goods: parts.special_goods.into_boxed_slice(),
            season_items: parts.season_items,
            consumables: parts.consumables.into_boxed_slice(),
            managed_functions: parts.managed_functions.into_boxed_slice(),
            rewards: parts.rewards.into_boxed_slice(),
            reward_pools: parts.reward_pools.into_boxed_slice(),
            recipes: parts.recipes.into_boxed_slice(),
            upgrades: parts.upgrades.into_boxed_slice(),
            forge_services: parts.forge_services.into_boxed_slice(),
            constants: parts.constants.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn items(&self) -> &[CurrencyWarsItemDefinition] {
        &self.items
    }
    #[must_use]
    pub fn special_goods(&self) -> &[CurrencyWarsSpecialGood] {
        &self.special_goods
    }
    #[must_use]
    pub const fn season_items(&self) -> &BTreeSet<CurrencyWarsItemId> {
        &self.season_items
    }
    #[must_use]
    pub fn consumables(&self) -> &[CurrencyWarsConsumableDefinition] {
        &self.consumables
    }
    #[must_use]
    pub fn managed_functions(&self) -> &[CurrencyWarsManagedFunction] {
        &self.managed_functions
    }
    #[must_use]
    pub fn rewards(&self) -> &[CurrencyWarsRewardDefinition] {
        &self.rewards
    }
    #[must_use]
    pub fn reward_pools(&self) -> &[CurrencyWarsRewardPool] {
        &self.reward_pools
    }
    #[must_use]
    pub fn recipes(&self) -> &[CurrencyWarsEquipmentRecipe] {
        &self.recipes
    }
    #[must_use]
    pub fn upgrades(&self) -> &[CurrencyWarsEquipmentUpgrade] {
        &self.upgrades
    }
    #[must_use]
    pub fn forge_services(&self) -> &[CurrencyWarsForgeService] {
        &self.forge_services
    }
    #[must_use]
    pub fn constants(&self) -> &[CurrencyWarsServiceConstant] {
        &self.constants
    }

    #[must_use]
    pub fn constant(&self, name: &str) -> Option<&CurrencyWarsServiceConstantValue> {
        self.constants
            .binary_search_by(|value| value.name.as_ref().cmp(name))
            .ok()
            .map(|index| &self.constants[index].value)
    }

    #[must_use]
    pub const fn proven_empty_families() -> [CurrencyWarsProvenEmptyServiceFamily; 9] {
        [
            CurrencyWarsProvenEmptyServiceFamily::GambleGroups,
            CurrencyWarsProvenEmptyServiceFamily::GambleUnits,
            CurrencyWarsProvenEmptyServiceFamily::CurseChests,
            CurrencyWarsProvenEmptyServiceFamily::HexStates,
            CurrencyWarsProvenEmptyServiceFamily::HexEligibility,
            CurrencyWarsProvenEmptyServiceFamily::Curios,
            CurrencyWarsProvenEmptyServiceFamily::CurioGroups,
            CurrencyWarsProvenEmptyServiceFamily::CurioStates,
            CurrencyWarsProvenEmptyServiceFamily::CurioLifecycle,
        ]
    }

    #[must_use]
    pub fn item(&self, id: CurrencyWarsItemId) -> Option<&CurrencyWarsItemDefinition> {
        self.items
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.items[index])
    }

    #[must_use]
    pub fn special_good(&self, id: u32) -> Option<&CurrencyWarsSpecialGood> {
        self.special_goods
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.special_goods[index])
    }

    #[must_use]
    pub fn season_contains(&self, item: CurrencyWarsItemId) -> bool {
        self.season_items.contains(&item)
    }

    #[must_use]
    pub fn consumable(
        &self,
        item: CurrencyWarsItemId,
    ) -> Option<&CurrencyWarsConsumableDefinition> {
        self.consumables
            .binary_search_by_key(&item, |value| value.item)
            .ok()
            .map(|index| &self.consumables[index])
    }

    #[must_use]
    pub fn reward(&self, id: u32) -> Option<&CurrencyWarsRewardDefinition> {
        self.rewards
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.rewards[index])
    }

    #[must_use]
    pub fn reward_pool(&self, id: u32) -> Option<&CurrencyWarsRewardPool> {
        self.reward_pools
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.reward_pools[index])
    }

    #[must_use]
    pub fn recipe(&self, id: u32) -> Option<&CurrencyWarsEquipmentRecipe> {
        self.recipes
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.recipes[index])
    }

    #[must_use]
    pub fn upgrade(&self, source: CurrencyWarsEquipmentId) -> Option<CurrencyWarsEquipmentId> {
        self.upgrades
            .binary_search_by_key(&source, |value| value.source)
            .ok()
            .map(|index| self.upgrades[index].output)
    }

    #[must_use]
    pub fn forge_service(&self, item: CurrencyWarsItemId) -> Option<&CurrencyWarsForgeService> {
        self.forge_services
            .binary_search_by_key(&item, |value| value.item)
            .ok()
            .map(|index| &self.forge_services[index])
    }
}

fn validate(
    parts: &CurrencyWarsServiceCatalogParts,
) -> Result<(), CurrencyWarsServiceCatalogError> {
    let required_counts = [
        parts.items.len(),
        parts.special_goods.len(),
        parts.season_items.len(),
        parts.consumables.len(),
        parts.managed_functions.len(),
        parts.rewards.len(),
        parts.reward_pools.len(),
        parts.recipes.len(),
        parts.upgrades.len(),
        parts.forge_services.len(),
        parts.constants.len(),
    ];
    if required_counts.contains(&0) {
        return Err(error(
            "Currency Wars service catalog has an empty required family",
        ));
    }
    let empty_counts = [
        parts.gamble_group_count,
        parts.gamble_unit_count,
        parts.curse_chest_count,
        parts.hex_state_count,
        parts.hex_eligibility_count,
        parts.curio_count,
        parts.curio_group_count,
        parts.curio_state_count,
        parts.curio_lifecycle_count,
    ];
    if empty_counts.iter().any(|count| *count != 0) {
        return Err(error(
            "Currency Wars proven-empty service family is not empty",
        ));
    }
    unique(parts.items.iter().map(|value| value.id), parts.items.len())?;
    unique(
        parts.special_goods.iter().map(|value| value.id),
        parts.special_goods.len(),
    )?;
    unique(
        parts
            .special_goods
            .iter()
            .map(|value| value.config_path.as_ref()),
        parts.special_goods.len(),
    )?;
    unique(
        parts.consumables.iter().map(|value| value.item),
        parts.consumables.len(),
    )?;
    unique(
        parts
            .managed_functions
            .iter()
            .map(|value| value.function_id.as_ref()),
        parts.managed_functions.len(),
    )?;
    unique(
        parts.rewards.iter().map(|value| value.id),
        parts.rewards.len(),
    )?;
    unique(
        parts.reward_pools.iter().map(|value| value.id),
        parts.reward_pools.len(),
    )?;
    unique(
        parts.recipes.iter().map(|value| value.id),
        parts.recipes.len(),
    )?;
    unique(
        parts.upgrades.iter().map(|value| value.source),
        parts.upgrades.len(),
    )?;
    unique(
        parts.forge_services.iter().map(|value| value.item),
        parts.forge_services.len(),
    )?;
    unique(
        parts.constants.iter().map(|value| value.name.as_ref()),
        parts.constants.len(),
    )?;
    if parts.special_goods.iter().any(|good| {
        good.id == 0 || good.group_id == 0 || good.quality == 0 || good.config_path.is_empty()
    }) || parts
        .managed_functions
        .iter()
        .any(|function| function.function_id.is_empty() || function.unlock_id == 0)
        || parts.constants.iter().any(|constant| {
            constant.name.is_empty()
                || matches!(
                    &constant.value,
                    CurrencyWarsServiceConstantValue::IntegerArray(values) if values.is_empty()
                )
        })
    {
        return Err(error("Currency Wars service definition is invalid"));
    }
    for pool in &parts.reward_pools {
        if pool.total_value == 0
            || pool.candidates.is_empty()
            || pool.candidates.iter().any(|candidate| {
                candidate.maximum == 0
                    || candidate.weight == 0
                    || !parts.rewards.iter().any(|reward| {
                        reward.id == candidate.reward_id
                            && reward.budget_cost.is_some_and(|cost| cost != 0)
                    })
            })
        {
            return Err(error("Currency Wars reward pool is invalid"));
        }
    }
    if parts.recipes.iter().any(|recipe| recipe.inputs.len() != 2)
        || parts
            .season_items
            .iter()
            .any(|item| !parts.items.iter().any(|value| value.id == *item))
    {
        return Err(error(
            "Currency Wars service relationship closure is invalid",
        ));
    }
    Ok(())
}

fn unique<T: Ord>(
    values: impl Iterator<Item = T>,
    expected: usize,
) -> Result<(), CurrencyWarsServiceCatalogError> {
    if values.collect::<BTreeSet<_>>().len() != expected {
        return Err(error("Currency Wars service identity is duplicated"));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsServiceCatalogError {
    message: Box<str>,
}
impl std::fmt::Display for CurrencyWarsServiceCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for CurrencyWarsServiceCatalogError {}
fn error(message: &'static str) -> CurrencyWarsServiceCatalogError {
    CurrencyWarsServiceCatalogError {
        message: message.into(),
    }
}

#[must_use]
pub fn equipment_category_from_selector(selector: u32) -> Option<CurrencyWarsEquipmentCategory> {
    match selector {
        1 => Some(CurrencyWarsEquipmentCategory::Basic),
        2 => Some(CurrencyWarsEquipmentCategory::Craftable),
        3 => Some(CurrencyWarsEquipmentCategory::Emblem),
        4 => Some(CurrencyWarsEquipmentCategory::Crown),
        6 => Some(CurrencyWarsEquipmentCategory::Artifacts),
        7 => Some(CurrencyWarsEquipmentCategory::Radiant),
        8 => Some(CurrencyWarsEquipmentCategory::Support),
        10 => Some(CurrencyWarsEquipmentCategory::Material),
        11 => Some(CurrencyWarsEquipmentCategory::TraitSpecial),
        _ => None,
    }
}
