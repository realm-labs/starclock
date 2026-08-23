use std::collections::BTreeSet;
use std::num::NonZeroU32;

use crate::{CurrencyWarsDecimal, CurrencyWarsGambit, CurrencyWarsInvestmentId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsInvestmentOfferFamily {
    Augment,
    Enhancement,
    Orb,
    Portal,
    Projection,
    Talent,
}

impl CurrencyWarsInvestmentOfferFamily {
    #[must_use]
    pub const fn bit(self) -> u8 {
        match self {
            Self::Augment => 1,
            Self::Enhancement => 2,
            Self::Orb => 4,
            Self::Portal => 8,
            Self::Projection => 16,
            Self::Talent => 32,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsInvestmentOfferSpec {
    pub families: Box<[CurrencyWarsInvestmentOfferFamily]>,
    pub augment_quality: Option<CurrencyWarsAugmentQuality>,
    pub width: u8,
    pub rerolls: u8,
}

impl CurrencyWarsInvestmentOfferSpec {
    pub fn new(
        mut families: Vec<CurrencyWarsInvestmentOfferFamily>,
        augment_quality: Option<CurrencyWarsAugmentQuality>,
        width: u8,
        rerolls: u8,
    ) -> Result<Self, CurrencyWarsAugmentCatalogError> {
        families.sort_unstable();
        families.dedup();
        if families.is_empty() || width == 0 {
            return Err(error(
                "Currency Wars investment offer specification is empty",
            ));
        }
        if augment_quality.is_some()
            && families
                .binary_search(&CurrencyWarsInvestmentOfferFamily::Augment)
                .is_err()
        {
            return Err(error(
                "Currency Wars investment quality requires the Augment family",
            ));
        }
        Ok(Self {
            families: families.into_boxed_slice(),
            augment_quality,
            width,
            rerolls,
        })
    }

    #[must_use]
    pub fn contains(&self, family: CurrencyWarsInvestmentOfferFamily) -> bool {
        self.families.binary_search(&family).is_ok()
    }

    #[must_use]
    pub fn family_mask(&self) -> u8 {
        self.families
            .iter()
            .fold(0, |mask, family| mask | family.bit())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsAugmentQuality {
    Silver,
    Gold,
    Prismatic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAugmentLifecycle {
    pub saved_values: Box<[Box<str>]>,
    pub overclock_effective: bool,
    pub effect_parameters: Box<[CurrencyWarsDecimal]>,
    pub description_parameters: Box<[CurrencyWarsDecimal]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAugmentRemark {
    pub en: Box<str>,
    pub zh_cn: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsInvestmentMazeBuff {
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub series: u16,
    pub level: u8,
    pub maximum_level: u8,
    pub binding_type: Box<str>,
    pub binding_key: Box<str>,
    pub maze_buff_type: Box<str>,
    pub parameters: Box<[CurrencyWarsDecimal]>,
    pub modifier: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAugmentMonsterRule {
    pub quality: CurrencyWarsAugmentQuality,
    pub division_level: Option<u8>,
    pub enemy_difficulty_level_add: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAugmentDefinition {
    pub investment: CurrencyWarsInvestmentId,
    pub source_id: u32,
    pub stable_key: Box<str>,
    pub category_id: u16,
    pub quality: CurrencyWarsAugmentQuality,
    pub chapter_limits: Box<[u8]>,
    pub season_ids: Box<[u16]>,
    pub effect_ids: Box<[Box<str>]>,
    pub config_path: Box<str>,
    pub lifecycle: CurrencyWarsAugmentLifecycle,
    pub remark: Option<CurrencyWarsAugmentRemark>,
    pub banned_module_ids: Box<[u32]>,
}

impl CurrencyWarsAugmentDefinition {
    #[must_use]
    pub fn eligible(
        &self,
        season_id: u16,
        plane: u8,
        gambit: CurrencyWarsGambit,
        module_id: u32,
    ) -> bool {
        self.season_ids.binary_search(&season_id).is_ok()
            && (self.chapter_limits.is_empty() || self.chapter_limits.binary_search(&plane).is_ok())
            && (gambit == CurrencyWarsGambit::Standard || self.lifecycle.overclock_effective)
            && self.banned_module_ids.binary_search(&module_id).is_err()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsSelectedEnhancementId(NonZeroU32);

impl CurrencyWarsSelectedEnhancementId {
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

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsEnhancementSelectCondition {
    Always,
    Permanent,
    MaximumStar,
}

impl CurrencyWarsEnhancementSelectCondition {
    #[must_use]
    pub const fn eligible(self, maximum_star: bool) -> bool {
        match self {
            Self::Always | Self::Permanent => true,
            Self::MaximumStar => maximum_star,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSelectedEnhancement {
    pub id: CurrencyWarsSelectedEnhancementId,
    pub stable_key: Box<str>,
    pub trait_effect_id: u32,
    pub gold_cost: Option<u32>,
    pub condition: CurrencyWarsEnhancementSelectCondition,
    pub parameters: Box<[CurrencyWarsDecimal]>,
    pub effects: Box<[CurrencyWarsDecimal]>,
    pub effect_ids: Box<[Box<str>]>,
}

impl CurrencyWarsSelectedEnhancement {
    #[must_use]
    pub const fn eligible(&self, maximum_star: bool) -> bool {
        self.condition.eligible(maximum_star)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEnhancement {
    pub investment: CurrencyWarsInvestmentId,
    pub id: CurrencyWarsSelectedEnhancementId,
    pub stable_key: Box<str>,
    pub trait_effect_id: u32,
    pub gold_cost: Option<u32>,
    pub condition: CurrencyWarsEnhancementSelectCondition,
    pub effects: Box<[CurrencyWarsDecimal]>,
    pub effect_ids: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAugmentCatalog {
    augments: Box<[CurrencyWarsAugmentDefinition]>,
    selected_enhancements: Box<[CurrencyWarsSelectedEnhancement]>,
    enhancements: Box<[CurrencyWarsEnhancement]>,
    maze_buffs: Box<[CurrencyWarsInvestmentMazeBuff]>,
    monster_rules: Box<[CurrencyWarsAugmentMonsterRule]>,
}

impl CurrencyWarsAugmentCatalog {
    pub fn new(
        mut augments: Vec<CurrencyWarsAugmentDefinition>,
        mut selected_enhancements: Vec<CurrencyWarsSelectedEnhancement>,
        mut enhancements: Vec<CurrencyWarsEnhancement>,
        mut maze_buffs: Vec<CurrencyWarsInvestmentMazeBuff>,
        mut monster_rules: Vec<CurrencyWarsAugmentMonsterRule>,
    ) -> Result<Self, CurrencyWarsAugmentCatalogError> {
        augments.sort_by_key(|value| value.investment);
        selected_enhancements.sort_by_key(|value| value.id);
        enhancements.sort_by_key(|value| value.investment);
        maze_buffs.sort_by_key(|value| (value.source_id, value.level));
        monster_rules.sort_by_key(|value| (value.quality, value.division_level));
        validate_augments(&augments)?;
        validate_selected_enhancements(&selected_enhancements)?;
        validate_enhancements(&enhancements)?;
        Ok(Self {
            augments: augments.into_boxed_slice(),
            selected_enhancements: selected_enhancements.into_boxed_slice(),
            enhancements: enhancements.into_boxed_slice(),
            maze_buffs: maze_buffs.into_boxed_slice(),
            monster_rules: monster_rules.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn augments(&self) -> &[CurrencyWarsAugmentDefinition] {
        &self.augments
    }

    #[must_use]
    pub fn selected_enhancements(&self) -> &[CurrencyWarsSelectedEnhancement] {
        &self.selected_enhancements
    }

    #[must_use]
    pub fn enhancements(&self) -> &[CurrencyWarsEnhancement] {
        &self.enhancements
    }

    #[must_use]
    pub fn maze_buffs(&self) -> &[CurrencyWarsInvestmentMazeBuff] {
        &self.maze_buffs
    }

    #[must_use]
    pub fn monster_rules(&self) -> &[CurrencyWarsAugmentMonsterRule] {
        &self.monster_rules
    }

    #[must_use]
    pub fn enhancement(&self, id: CurrencyWarsInvestmentId) -> Option<&CurrencyWarsEnhancement> {
        self.enhancements
            .binary_search_by_key(&id, |value| value.investment)
            .ok()
            .map(|index| &self.enhancements[index])
    }

    #[must_use]
    pub fn enemy_difficulty_add(&self, quality: CurrencyWarsAugmentQuality, division: u8) -> u8 {
        self.monster_rules
            .iter()
            .find(|value| value.quality == quality && value.division_level == Some(division))
            .and_then(|value| value.enemy_difficulty_level_add)
            .unwrap_or_default()
    }

    #[must_use]
    pub fn augment(
        &self,
        investment: CurrencyWarsInvestmentId,
    ) -> Option<&CurrencyWarsAugmentDefinition> {
        self.augments
            .binary_search_by_key(&investment, |value| value.investment)
            .ok()
            .map(|index| &self.augments[index])
    }

    #[must_use]
    pub fn selected_enhancement(
        &self,
        id: CurrencyWarsSelectedEnhancementId,
    ) -> Option<&CurrencyWarsSelectedEnhancement> {
        self.selected_enhancements
            .binary_search_by_key(&id, |value| value.id)
            .ok()
            .map(|index| &self.selected_enhancements[index])
    }
}

fn validate_augments(
    augments: &[CurrencyWarsAugmentDefinition],
) -> Result<(), CurrencyWarsAugmentCatalogError> {
    let mut investments = BTreeSet::new();
    let mut source_ids = BTreeSet::new();
    for augment in augments {
        if !investments.insert(augment.investment)
            || !source_ids.insert(augment.source_id)
            || augment.category_id == 0
            || augment.season_ids.is_empty()
            || augment.season_ids.windows(2).any(|pair| pair[0] >= pair[1])
            || augment
                .chapter_limits
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || augment
                .chapter_limits
                .iter()
                .any(|plane| !(1..=3).contains(plane))
            || augment
                .banned_module_ids
                .windows(2)
                .any(|pair| pair[0] >= pair[1])
            || augment.config_path.is_empty()
        {
            return Err(error("Currency Wars Augment definition is invalid"));
        }
    }
    Ok(())
}

fn validate_selected_enhancements(
    definitions: &[CurrencyWarsSelectedEnhancement],
) -> Result<(), CurrencyWarsAugmentCatalogError> {
    let mut ids = BTreeSet::new();
    for definition in definitions {
        if !ids.insert(definition.id)
            || definition.trait_effect_id == 0
            || definition.effect_ids.is_empty()
        {
            return Err(error(
                "Currency Wars selected Enhancement definition is invalid",
            ));
        }
    }
    Ok(())
}

fn validate_enhancements(
    definitions: &[CurrencyWarsEnhancement],
) -> Result<(), CurrencyWarsAugmentCatalogError> {
    let investments = definitions
        .iter()
        .map(|definition| definition.investment)
        .collect::<BTreeSet<_>>();
    let ids = definitions
        .iter()
        .map(|definition| definition.id)
        .collect::<BTreeSet<_>>();
    if investments.len() != definitions.len()
        || ids.len() != definitions.len()
        || definitions
            .iter()
            .any(|definition| definition.trait_effect_id == 0 || definition.effect_ids.is_empty())
    {
        return Err(error("Currency Wars Enhancement definition is invalid"));
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAugmentCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsAugmentCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsAugmentCatalogError {}

fn error(message: &'static str) -> CurrencyWarsAugmentCatalogError {
    CurrencyWarsAugmentCatalogError {
        message: message.into(),
    }
}
