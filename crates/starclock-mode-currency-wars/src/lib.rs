//! Currency Wars catalogs, deterministic run operations and Activity profile.
//!
//! This crate owns mode terminology and validation. Cross-battle mutation is
//! committed by `starclock-activity`; individual battles remain owned by
//! `starclock-combat`.

#![forbid(unsafe_code)]

mod catalog;
mod economy;
mod flow;
mod runtime;
#[cfg(test)]
mod runtime_tests;

pub use catalog::{
    CurrencyWarsBond, CurrencyWarsBondId, CurrencyWarsBondLevel, CurrencyWarsCatalog,
    CurrencyWarsCatalogError, CurrencyWarsCatalogParts, CurrencyWarsDifficulty, CurrencyWarsGambit,
    CurrencyWarsInvestment, CurrencyWarsInvestmentId, CurrencyWarsInvestmentKind, CurrencyWarsNode,
    CurrencyWarsNodeId, CurrencyWarsNodeKind, CurrencyWarsOfferLevel, CurrencyWarsPolicy,
    CurrencyWarsPositionKind, CurrencyWarsPriceRule, CurrencyWarsRole, CurrencyWarsRoleId,
    CurrencyWarsRoute, CurrencyWarsRouteId, CurrencyWarsStarRule, CurrencyWarsTeamLevel,
};
pub use economy::{
    CurrencyWarsDeployment, CurrencyWarsEconomyError, CurrencyWarsPosition, CurrencyWarsRoleState,
    CurrencyWarsRoster, advance_team_level,
};
pub use flow::{CurrencyWarsFlow, CurrencyWarsFlowError};
pub use runtime::{
    CURRENCY_WARS_ACTION_VALUE_REMAINING_KEY, CURRENCY_WARS_SQUAD_HP_LOSS_KEY, CurrencyWarsRun,
    CurrencyWarsRunDefinition, CurrencyWarsRunSetup, CurrencyWarsRuntimeError,
};
