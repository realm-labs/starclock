use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use starclock_combat::EncounterId;

use crate::progression_catalog::{
    BaseballerAdventureStrategy, BaseballerRuntimeCatalogContent, BaseballerShopUpgrade,
    BaseballerTeamBonus, validate_shop_upgrades, validate_strategies, validate_team_bonuses,
};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU32);

        impl $name {
            #[must_use]
            pub const fn new(raw: u32) -> Option<Self> {
                match NonZeroU32::new(raw) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u32 {
                self.0.get()
            }
        }
    };
}

id_type!(BaseballerProfileId);
id_type!(BaseballerStageId);
id_type!(BaseballerStagePeriodId);
id_type!(BaseballerEquipmentId);
id_type!(BaseballerShopUpgradeId);
id_type!(BaseballerAdventureStrategyId);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerProfileKind {
    Departure,
    DemonKing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerProfile {
    pub id: BaseballerProfileId,
    pub kind: BaseballerProfileKind,
    pub stable_key: Box<str>,
    pub weapon_slots: u8,
    pub initially_unlocked_weapon_slots: u8,
    pub accessory_slots: u8,
    pub initially_unlocked_accessory_slots: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerStage {
    pub id: BaseballerStageId,
    pub profile: BaseballerProfileId,
    pub difficulty: u8,
    pub weapon_selectable: bool,
    pub initial_weapons: Box<[BaseballerEquipmentId]>,
    pub rating_thresholds: Box<[i64]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerPeriodRank {
    First,
    Second,
    Third,
    Extra,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerStagePeriod {
    pub id: BaseballerStagePeriodId,
    pub stage: BaseballerStageId,
    pub rank: BaseballerPeriodRank,
    pub encounter: EncounterId,
    pub battle_event_id: u32,
    pub wave_count: u8,
    pub countdown_by_wave: Box<[u16]>,
    pub period_score: i64,
    pub stage_score: Option<i64>,
    pub selection_weight: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerRuntimePolicy {
    pub id: Box<str>,
    pub unavailable_fact: Box<str>,
    pub known_facts: Box<str>,
    pub selected_behavior: Box<str>,
    pub rejected_alternatives: Box<[Box<str>]>,
    pub rationale: Box<str>,
    pub affected_tests: Box<[Box<str>]>,
    pub confidence: Box<str>,
    pub replacement_condition: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerEquipmentKind {
    StandardWeapon,
    LegendaryWeapon,
    Accessory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerEquipment {
    pub id: BaseballerEquipmentId,
    pub stable_key: Box<str>,
    pub kind: BaseballerEquipmentKind,
    pub maximum_level: u8,
    pub profiles: Box<[BaseballerProfileId]>,
    pub runtime_binding_exact: bool,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerRecipeInputKind {
    Equipment(BaseballerEquipmentId),
    AnyStandardWeapon,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BaseballerRecipeTier {
    Supreme,
    Twin,
    Legendary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseballerRecipeInput {
    pub order: u8,
    pub kind: BaseballerRecipeInputKind,
    pub required_level: u8,
    pub consumed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerRecipe {
    pub id: u32,
    pub profile: BaseballerProfileId,
    pub tier: BaseballerRecipeTier,
    pub output: BaseballerEquipmentId,
    pub inputs: Box<[BaseballerRecipeInput]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseballerCatalog {
    profiles: Box<[BaseballerProfile]>,
    stages: Box<[BaseballerStage]>,
    stage_periods: Box<[BaseballerStagePeriod]>,
    equipment: Box<[BaseballerEquipment]>,
    recipes: Box<[BaseballerRecipe]>,
    shop_upgrades: Box<[BaseballerShopUpgrade]>,
    strategies: Box<[BaseballerAdventureStrategy]>,
    team_bonuses: Box<[BaseballerTeamBonus]>,
}

impl BaseballerCatalog {
    pub fn new(
        profiles: Vec<BaseballerProfile>,
        stages: Vec<BaseballerStage>,
        stage_periods: Vec<BaseballerStagePeriod>,
        equipment: Vec<BaseballerEquipment>,
        recipes: Vec<BaseballerRecipe>,
    ) -> Result<Self, BaseballerCatalogError> {
        Self::new_with_shop_upgrades(profiles, stages, stage_periods, equipment, recipes, vec![])
    }

    pub fn new_with_shop_upgrades(
        profiles: Vec<BaseballerProfile>,
        stages: Vec<BaseballerStage>,
        stage_periods: Vec<BaseballerStagePeriod>,
        equipment: Vec<BaseballerEquipment>,
        recipes: Vec<BaseballerRecipe>,
        shop_upgrades: Vec<BaseballerShopUpgrade>,
    ) -> Result<Self, BaseballerCatalogError> {
        Self::new_with_runtime_content(
            profiles,
            stages,
            stage_periods,
            equipment,
            recipes,
            BaseballerRuntimeCatalogContent {
                shop_upgrades,
                ..BaseballerRuntimeCatalogContent::default()
            },
        )
    }

    pub fn new_with_runtime_content(
        mut profiles: Vec<BaseballerProfile>,
        mut stages: Vec<BaseballerStage>,
        mut stage_periods: Vec<BaseballerStagePeriod>,
        mut equipment: Vec<BaseballerEquipment>,
        mut recipes: Vec<BaseballerRecipe>,
        content: BaseballerRuntimeCatalogContent,
    ) -> Result<Self, BaseballerCatalogError> {
        let BaseballerRuntimeCatalogContent {
            mut shop_upgrades,
            mut strategies,
            mut team_bonuses,
        } = content;
        profiles.sort_by_key(|item| item.id);
        stages.sort_by_key(|item| item.id);
        stage_periods.sort_by_key(|item| item.id);
        equipment.sort_by_key(|item| item.id);
        recipes.sort_by_key(|item| item.id);
        shop_upgrades.sort_by_key(|item| item.id);
        strategies.sort_by_key(|item| item.id);
        team_bonuses.sort_by_key(|item| item.stage);
        unique(&profiles, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateProfile)?;
        unique(&stages, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateStage)?;
        unique(&stage_periods, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateStagePeriod)?;
        unique(&equipment, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateEquipment)?;
        unique(&recipes, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateRecipe)?;
        unique(&shop_upgrades, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateShopUpgrade)?;
        unique(&strategies, |item| item.id)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateStrategy)?;
        unique(&team_bonuses, |item| item.stage)
            .then_some(())
            .ok_or(BaseballerCatalogError::DuplicateTeamBonus)?;
        if profiles.iter().any(|profile| {
            profile.initially_unlocked_weapon_slots > profile.weapon_slots
                || profile.initially_unlocked_accessory_slots > profile.accessory_slots
        }) {
            return Err(BaseballerCatalogError::InvalidSlotPolicy);
        }
        for stage in &stages {
            if find(&profiles, stage.profile, |item| item.id).is_none()
                || stage.initial_weapons.iter().any(|id| {
                    find(&equipment, *id, |item| item.id).is_none_or(|item| {
                        item.kind == BaseballerEquipmentKind::Accessory
                            || !item.profiles.contains(&stage.profile)
                    })
                })
                || stage
                    .rating_thresholds
                    .windows(2)
                    .any(|pair| pair[0] >= pair[1])
            {
                return Err(BaseballerCatalogError::InvalidStageReference);
            }
            let stage_periods = stage_periods
                .iter()
                .filter(|period| period.stage == stage.id)
                .collect::<Vec<_>>();
            if stage_periods.is_empty()
                || !valid_period_ranks(stage_periods.iter().map(|period| period.rank))
            {
                return Err(BaseballerCatalogError::InvalidStagePeriod);
            }
        }
        if stage_periods.iter().any(|period| {
            find(&stages, period.stage, |stage| stage.id).is_none()
                || period.wave_count == 0
                || period.countdown_by_wave.len() != usize::from(period.wave_count)
                || period.period_score < 0
                || period.stage_score.is_some_and(|score| score < 0)
                || period.selection_weight == 0
        }) {
            return Err(BaseballerCatalogError::InvalidStagePeriod);
        }
        for item in &equipment {
            if item.maximum_level == 0
                || item.profiles.is_empty()
                || item
                    .profiles
                    .iter()
                    .any(|id| find(&profiles, *id, |profile| profile.id).is_none())
            {
                return Err(BaseballerCatalogError::InvalidEquipmentReference);
            }
        }
        validate_recipes(&profiles, &equipment, &recipes)?;
        validate_shop_upgrades(&profiles, &shop_upgrades)?;
        validate_strategies(&profiles, &strategies)?;
        validate_team_bonuses(&profiles, &stages, &team_bonuses)?;
        Ok(Self {
            profiles: profiles.into_boxed_slice(),
            stages: stages.into_boxed_slice(),
            stage_periods: stage_periods.into_boxed_slice(),
            equipment: equipment.into_boxed_slice(),
            recipes: recipes.into_boxed_slice(),
            shop_upgrades: shop_upgrades.into_boxed_slice(),
            strategies: strategies.into_boxed_slice(),
            team_bonuses: team_bonuses.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn profiles(&self) -> &[BaseballerProfile] {
        &self.profiles
    }

    #[must_use]
    pub fn stages(&self) -> &[BaseballerStage] {
        &self.stages
    }

    #[must_use]
    pub fn stage_periods(&self) -> &[BaseballerStagePeriod] {
        &self.stage_periods
    }

    #[must_use]
    pub fn periods_for_stage(&self, stage: BaseballerStageId) -> Vec<&BaseballerStagePeriod> {
        self.stage_periods
            .iter()
            .filter(|period| period.stage == stage)
            .collect()
    }

    #[must_use]
    pub fn equipment(&self) -> &[BaseballerEquipment] {
        &self.equipment
    }

    #[must_use]
    pub fn recipes(&self) -> &[BaseballerRecipe] {
        &self.recipes
    }

    #[must_use]
    pub fn shop_upgrades(&self) -> &[BaseballerShopUpgrade] {
        &self.shop_upgrades
    }

    pub fn shop_upgrades_for_profile(
        &self,
        profile: BaseballerProfileId,
    ) -> impl Iterator<Item = &BaseballerShopUpgrade> {
        self.shop_upgrades
            .iter()
            .filter(move |upgrade| upgrade.profile == profile)
    }

    #[must_use]
    pub fn shop_upgrade_by_id(
        &self,
        id: BaseballerShopUpgradeId,
    ) -> Option<&BaseballerShopUpgrade> {
        find(&self.shop_upgrades, id, |item| item.id)
    }

    #[must_use]
    pub fn strategies(&self) -> &[BaseballerAdventureStrategy] {
        &self.strategies
    }

    #[must_use]
    pub fn team_bonuses(&self) -> &[BaseballerTeamBonus] {
        &self.team_bonuses
    }

    #[must_use]
    pub fn equipment_by_id(&self, id: BaseballerEquipmentId) -> Option<&BaseballerEquipment> {
        find(&self.equipment, id, |item| item.id)
    }

    #[must_use]
    pub fn recipe_by_id(&self, id: u32) -> Option<&BaseballerRecipe> {
        self.recipes
            .binary_search_by_key(&id, |item| item.id)
            .ok()
            .map(|index| &self.recipes[index])
    }

    pub fn validate_synthesis(
        &self,
        recipe_id: u32,
        levels: &BTreeMap<BaseballerEquipmentId, u8>,
    ) -> Result<BaseballerEquipmentId, BaseballerSynthesisError> {
        let recipe = self
            .recipes
            .binary_search_by_key(&recipe_id, |item| item.id)
            .ok()
            .map(|index| &self.recipes[index])
            .ok_or(BaseballerSynthesisError::UnknownRecipe)?;
        let mut selected = BTreeSet::new();
        for input in &recipe.inputs {
            let candidate = match input.kind {
                BaseballerRecipeInputKind::Equipment(id) => levels
                    .get(&id)
                    .filter(|level| **level >= input.required_level)
                    .map(|_| id),
                BaseballerRecipeInputKind::AnyStandardWeapon => self
                    .equipment
                    .iter()
                    .filter(|item| item.kind == BaseballerEquipmentKind::StandardWeapon)
                    .filter(|item| !selected.contains(&item.id))
                    .find(|item| {
                        levels
                            .get(&item.id)
                            .is_some_and(|level| *level >= input.required_level)
                    })
                    .map(|item| item.id),
            }
            .ok_or(BaseballerSynthesisError::MissingInput)?;
            selected.insert(candidate);
        }
        Ok(recipe.output)
    }
}

fn valid_period_ranks(ranks: impl Iterator<Item = BaseballerPeriodRank>) -> bool {
    let ranks = ranks
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let expected = [
        BaseballerPeriodRank::First,
        BaseballerPeriodRank::Second,
        BaseballerPeriodRank::Third,
        BaseballerPeriodRank::Extra,
    ];
    !ranks.is_empty() && ranks.as_slice() == &expected[..ranks.len()]
}

fn validate_recipes(
    profiles: &[BaseballerProfile],
    equipment: &[BaseballerEquipment],
    recipes: &[BaseballerRecipe],
) -> Result<(), BaseballerCatalogError> {
    for recipe in recipes {
        if recipe.id == 0
            || find(profiles, recipe.profile, |item| item.id).is_none()
            || find(equipment, recipe.output, |item| item.id).is_none()
            || recipe.inputs.is_empty()
            || recipe
                .inputs
                .windows(2)
                .any(|pair| pair[0].order >= pair[1].order)
            || recipe.inputs.iter().any(|input| match input.kind {
                BaseballerRecipeInputKind::Equipment(id) => {
                    find(equipment, id, |item| item.id).is_none()
                }
                BaseballerRecipeInputKind::AnyStandardWeapon => false,
            })
        {
            return Err(BaseballerCatalogError::InvalidRecipeReference);
        }
    }
    let outputs = recipes
        .iter()
        .map(|recipe| (recipe.output, recipe.id))
        .collect::<BTreeMap<_, _>>();
    for recipe in recipes {
        validate_recipe_dependencies(
            recipe.output,
            recipes,
            &outputs,
            &mut BTreeSet::new(),
            &mut BTreeSet::new(),
        )?;
    }
    Ok(())
}

fn validate_recipe_dependencies(
    output: BaseballerEquipmentId,
    recipes: &[BaseballerRecipe],
    outputs: &BTreeMap<BaseballerEquipmentId, u32>,
    active: &mut BTreeSet<BaseballerEquipmentId>,
    complete: &mut BTreeSet<BaseballerEquipmentId>,
) -> Result<(), BaseballerCatalogError> {
    if complete.contains(&output) {
        return Ok(());
    }
    if !active.insert(output) {
        return Err(BaseballerCatalogError::CyclicRecipeGraph);
    }
    if let Some(recipe) = outputs.get(&output).and_then(|id| {
        recipes
            .binary_search_by_key(id, |item| item.id)
            .ok()
            .map(|index| &recipes[index])
    }) {
        for input in &recipe.inputs {
            if let BaseballerRecipeInputKind::Equipment(id) = input.kind {
                validate_recipe_dependencies(id, recipes, outputs, active, complete)?;
            }
        }
    }
    active.remove(&output);
    complete.insert(output);
    Ok(())
}

fn unique<T, K: Eq>(items: &[T], key: impl Fn(&T) -> K) -> bool {
    items.windows(2).all(|pair| key(&pair[0]) != key(&pair[1]))
}

fn find<T, K: Ord>(items: &[T], key: K, item_key: impl Fn(&T) -> K) -> Option<&T> {
    items
        .binary_search_by_key(&key, item_key)
        .ok()
        .map(|index| &items[index])
}

#[cfg(test)]
pub(crate) mod tests_support {
    use starclock_combat::EncounterId;

    use super::{
        BaseballerCatalog, BaseballerEquipment, BaseballerEquipmentId, BaseballerEquipmentKind,
        BaseballerPeriodRank, BaseballerProfile, BaseballerProfileId, BaseballerProfileKind,
        BaseballerRecipe, BaseballerRecipeInput, BaseballerRecipeInputKind, BaseballerRecipeTier,
        BaseballerStage, BaseballerStageId, BaseballerStagePeriod, BaseballerStagePeriodId,
    };

    pub(crate) fn catalog() -> BaseballerCatalog {
        BaseballerCatalog::new(
            vec![BaseballerProfile {
                id: profile_id(),
                kind: BaseballerProfileKind::Departure,
                stable_key: "test".into(),
                weapon_slots: 2,
                initially_unlocked_weapon_slots: 1,
                accessory_slots: 2,
                initially_unlocked_accessory_slots: 1,
            }],
            vec![BaseballerStage {
                id: stage_id(),
                profile: profile_id(),
                difficulty: 1,
                weapon_selectable: true,
                initial_weapons: Box::new([equipment_id(1)]),
                rating_thresholds: Box::new([0, 1, 2, 3, 4]),
            }],
            vec![
                BaseballerStagePeriod {
                    id: BaseballerStagePeriodId::new(1).unwrap(),
                    stage: stage_id(),
                    rank: BaseballerPeriodRank::First,
                    encounter: EncounterId::new(1).unwrap(),
                    battle_event_id: 1,
                    wave_count: 1,
                    countdown_by_wave: Box::new([1]),
                    period_score: 1,
                    stage_score: Some(1),
                    selection_weight: 1,
                },
                BaseballerStagePeriod {
                    id: BaseballerStagePeriodId::new(2).unwrap(),
                    stage: stage_id(),
                    rank: BaseballerPeriodRank::Second,
                    encounter: EncounterId::new(2).unwrap(),
                    battle_event_id: 2,
                    wave_count: 1,
                    countdown_by_wave: Box::new([1]),
                    period_score: 1,
                    stage_score: Some(1),
                    selection_weight: 1,
                },
            ],
            vec![
                test_equipment(1, BaseballerEquipmentKind::StandardWeapon),
                test_equipment(2, BaseballerEquipmentKind::StandardWeapon),
                test_equipment(3, BaseballerEquipmentKind::LegendaryWeapon),
            ],
            vec![BaseballerRecipe {
                id: 1,
                profile: profile_id(),
                tier: BaseballerRecipeTier::Twin,
                output: equipment_id(3),
                inputs: vec![
                    BaseballerRecipeInput {
                        order: 1,
                        kind: BaseballerRecipeInputKind::Equipment(equipment_id(1)),
                        required_level: 1,
                        consumed: true,
                    },
                    BaseballerRecipeInput {
                        order: 2,
                        kind: BaseballerRecipeInputKind::Equipment(equipment_id(2)),
                        required_level: 1,
                        consumed: true,
                    },
                ]
                .into_boxed_slice(),
            }],
        )
        .unwrap()
    }

    pub(crate) fn profile_id() -> BaseballerProfileId {
        BaseballerProfileId::new(1).unwrap()
    }

    pub(crate) fn stage_id() -> BaseballerStageId {
        BaseballerStageId::new(1).unwrap()
    }

    pub(crate) fn full_catalog() -> BaseballerCatalog {
        BaseballerCatalog::new(
            vec![BaseballerProfile {
                id: profile_id(),
                kind: BaseballerProfileKind::Departure,
                stable_key: "full".into(),
                weapon_slots: 1,
                initially_unlocked_weapon_slots: 1,
                accessory_slots: 1,
                initially_unlocked_accessory_slots: 1,
            }],
            vec![BaseballerStage {
                id: stage_id(),
                profile: profile_id(),
                difficulty: 1,
                weapon_selectable: true,
                initial_weapons: Box::new([equipment_id(1)]),
                rating_thresholds: Box::new([0, 1, 2, 3, 4]),
            }],
            vec![BaseballerStagePeriod {
                id: BaseballerStagePeriodId::new(1).unwrap(),
                stage: stage_id(),
                rank: BaseballerPeriodRank::First,
                encounter: EncounterId::new(1).unwrap(),
                battle_event_id: 1,
                wave_count: 1,
                countdown_by_wave: Box::new([1]),
                period_score: 1,
                stage_score: Some(1),
                selection_weight: 1,
            }],
            vec![BaseballerEquipment {
                maximum_level: 1,
                ..test_equipment(1, BaseballerEquipmentKind::StandardWeapon)
            }],
            vec![],
        )
        .unwrap()
    }

    fn equipment_id(raw: u32) -> BaseballerEquipmentId {
        BaseballerEquipmentId::new(raw).unwrap()
    }

    fn test_equipment(raw: u32, kind: BaseballerEquipmentKind) -> BaseballerEquipment {
        BaseballerEquipment {
            id: equipment_id(raw),
            stable_key: format!("equipment-{raw}").into_boxed_str(),
            kind,
            maximum_level: 8,
            profiles: Box::new([profile_id()]),
            runtime_binding_exact: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseballerCatalogError {
    DuplicateProfile,
    DuplicateStage,
    DuplicateStagePeriod,
    DuplicateEquipment,
    DuplicateRecipe,
    DuplicateShopUpgrade,
    DuplicateStrategy,
    DuplicateTeamBonus,
    InvalidSlotPolicy,
    InvalidStageReference,
    InvalidStagePeriod,
    InvalidEquipmentReference,
    InvalidRecipeReference,
    InvalidShopUpgrade,
    InvalidShopUpgradeSequence,
    InvalidStrategy,
    InvalidTeamBonus,
    CyclicRecipeGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaseballerSynthesisError {
    UnknownRecipe,
    MissingInput,
}

#[cfg(test)]
mod tests {
    use super::{
        BaseballerCatalog, BaseballerCatalogError, BaseballerEquipment, BaseballerEquipmentId,
        BaseballerEquipmentKind, BaseballerProfile, BaseballerProfileId, BaseballerProfileKind,
        BaseballerRecipe, BaseballerRecipeInput, BaseballerRecipeInputKind, BaseballerRecipeTier,
    };

    #[test]
    fn converging_recipe_dependencies_are_not_a_cycle() {
        let catalog = BaseballerCatalog::new(
            vec![profile()],
            vec![],
            vec![],
            (1..=4).map(equipment).collect(),
            vec![
                recipe(1, 2, &[1]),
                recipe(2, 3, &[1]),
                recipe(3, 4, &[2, 3]),
            ],
        );
        assert!(catalog.is_ok());
    }

    #[test]
    fn actual_recipe_cycle_is_rejected() {
        let error = BaseballerCatalog::new(
            vec![profile()],
            vec![],
            vec![],
            (1..=2).map(equipment).collect(),
            vec![recipe(1, 1, &[2]), recipe(2, 2, &[1])],
        )
        .unwrap_err();
        assert_eq!(error, BaseballerCatalogError::CyclicRecipeGraph);
    }

    fn profile() -> BaseballerProfile {
        BaseballerProfile {
            id: BaseballerProfileId::new(1).unwrap(),
            kind: BaseballerProfileKind::Departure,
            stable_key: "test".into(),
            weapon_slots: 1,
            initially_unlocked_weapon_slots: 1,
            accessory_slots: 1,
            initially_unlocked_accessory_slots: 1,
        }
    }

    fn equipment(raw: u32) -> BaseballerEquipment {
        BaseballerEquipment {
            id: BaseballerEquipmentId::new(raw).unwrap(),
            stable_key: format!("equipment-{raw}").into_boxed_str(),
            kind: BaseballerEquipmentKind::StandardWeapon,
            maximum_level: 8,
            profiles: Box::new([BaseballerProfileId::new(1).unwrap()]),
            runtime_binding_exact: true,
        }
    }

    fn recipe(id: u32, output: u32, inputs: &[u32]) -> BaseballerRecipe {
        BaseballerRecipe {
            id,
            profile: BaseballerProfileId::new(1).unwrap(),
            tier: BaseballerRecipeTier::Legendary,
            output: BaseballerEquipmentId::new(output).unwrap(),
            inputs: inputs
                .iter()
                .enumerate()
                .map(|(index, input)| BaseballerRecipeInput {
                    order: u8::try_from(index + 1).unwrap(),
                    kind: BaseballerRecipeInputKind::Equipment(
                        BaseballerEquipmentId::new(*input).unwrap(),
                    ),
                    required_level: 1,
                    consumed: true,
                })
                .collect(),
        }
    }
}
