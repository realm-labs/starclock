//! Legend of the Galactic Baseballer profiles and deterministic preparation rules.
//!
//! The crate owns edition-specific catalogs, synthesis validation and score
//! settlement. Battles and cross-battle mutation remain in Combat and Activity.

#![forbid(unsafe_code)]

mod catalog;
mod flow;
mod inventory;
mod progression;
mod progression_catalog;
mod runtime;
mod score;

pub use catalog::{
    BaseballerAdventureStrategyId, BaseballerCatalog, BaseballerCatalogError, BaseballerEquipment,
    BaseballerEquipmentId, BaseballerEquipmentKind, BaseballerPeriodRank, BaseballerProfile,
    BaseballerProfileId, BaseballerProfileKind, BaseballerRecipe, BaseballerRecipeInput,
    BaseballerRecipeInputKind, BaseballerRecipeTier, BaseballerRuntimePolicy,
    BaseballerShopUpgradeId, BaseballerStage, BaseballerStageId, BaseballerStagePeriod,
    BaseballerStagePeriodId, BaseballerSynthesisError,
};
pub use flow::{BaseballerFlowError, BaseballerStageFlow};
pub use inventory::{
    BaseballerInventoryBindings, BaseballerInventoryError, BaseballerInventoryOptions,
};
pub use progression::{
    BaseballerProgression, BaseballerProgressionDefinition, BaseballerProgressionError,
    BaseballerProgressionSeed, BaseballerProgressionSnapshot, BaseballerUnresolvedMazeBuff,
};
pub use progression_catalog::{
    BaseballerAdventureStrategy, BaseballerAdventureStrategyKind, BaseballerRuntimeCatalogContent,
    BaseballerShopUpgrade, BaseballerShopUpgradeKind, BaseballerTeamBonus,
};
pub use runtime::{
    BASEBALLER_SCORE_KEY, BaseballerRun, BaseballerRunDefinition, BaseballerRuntimeError,
};
pub use score::{BaseballerRating, BaseballerScoreRule, BaseballerSettlement};
