use crate::{
    CurrencyWarsEntryKind, CurrencyWarsFlowCatalog, CurrencyWarsGambit, CurrencyWarsRankBoundary,
    CurrencyWarsRankProgressionKey, CurrencyWarsRouteId, CurrencyWarsUnlockCondition,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsRouteMembershipPolicy {
    SharedCompleteRouteSet,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsCarryResetPolicy {
    CarryRunAndParticipantStateResetNodeOffers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEntryState {
    player_level: u32,
    completed_standard_gambit: bool,
    highest_standard_rank: u8,
}

impl CurrencyWarsEntryState {
    #[must_use]
    pub const fn new(
        player_level: u32,
        completed_standard_gambit: bool,
        highest_standard_rank: u8,
    ) -> Self {
        Self {
            player_level,
            completed_standard_gambit,
            highest_standard_rank,
        }
    }

    #[must_use]
    pub const fn player_level(self) -> u32 {
        self.player_level
    }

    #[must_use]
    pub const fn completed_standard_gambit(self) -> bool {
        self.completed_standard_gambit
    }

    #[must_use]
    pub const fn highest_standard_rank(self) -> u8 {
        self.highest_standard_rank
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEntryResolution {
    pub route: CurrencyWarsRouteId,
    pub difficulty: u32,
    pub gambit: CurrencyWarsGambit,
    pub season: u16,
    pub division_level: u8,
}

impl CurrencyWarsFlowCatalog {
    #[must_use]
    pub const fn route_membership_policy(&self) -> CurrencyWarsRouteMembershipPolicy {
        CurrencyWarsRouteMembershipPolicy::SharedCompleteRouteSet
    }

    #[must_use]
    pub const fn carry_reset_policy(&self) -> CurrencyWarsCarryResetPolicy {
        CurrencyWarsCarryResetPolicy::CarryRunAndParticipantStateResetNodeOffers
    }

    pub fn resolve_entry(
        &self,
        route: CurrencyWarsRouteId,
        difficulty: u32,
        gambit: CurrencyWarsGambit,
        state: CurrencyWarsEntryState,
    ) -> Result<CurrencyWarsEntryResolution, CurrencyWarsEntryError> {
        validate_profile_unlock(self, state)?;
        validate_gambit_unlock(self, gambit, state)?;
        if !self.profile().gambits.contains(&gambit)
            || !self.area_group().routes.contains(&route)
            || self.route(route).is_none()
        {
            return Err(error(
                "Currency Wars route or Gambit is not in the active profile",
            ));
        }
        let module = self
            .modules()
            .iter()
            .find(|module| module.stable_key == self.profile().module_id)
            .ok_or_else(|| error("Currency Wars active module is missing"))?;
        let difficulty_definition = self
            .difficulties()
            .iter()
            .find(|candidate| candidate.source_id == difficulty)
            .ok_or_else(|| error("Currency Wars difficulty is not in the catalog"))?;
        if difficulty_definition.season_id != module.season_id {
            return Err(error(
                "Currency Wars difficulty is not in the active season",
            ));
        }
        let maximum = rank_maximum(self, module.season_id, state.highest_standard_rank, gambit)?;
        if difficulty_definition.division_level > maximum {
            return Err(error(
                "Currency Wars difficulty exceeds the highest Standard rank",
            ));
        }
        let route_definition = self
            .route(route)
            .expect("Currency Wars route membership was validated");
        if !route_definition.difficulty_ids.is_empty()
            && !route_definition.difficulty_ids.contains(&difficulty)
        {
            return Err(error("Currency Wars route does not admit the difficulty"));
        }
        Ok(CurrencyWarsEntryResolution {
            route,
            difficulty,
            gambit,
            season: module.season_id,
            division_level: difficulty_definition.division_level,
        })
    }
}

fn validate_profile_unlock(
    catalog: &CurrencyWarsFlowCatalog,
    state: CurrencyWarsEntryState,
) -> Result<(), CurrencyWarsEntryError> {
    for entry in catalog.entries().iter().filter(|entry| {
        catalog
            .profile()
            .entry_ids
            .iter()
            .any(|id| id == &entry.stable_key)
            && entry.kind == CurrencyWarsEntryKind::GuideData
    }) {
        for unlock in &entry.unlocks {
            if let CurrencyWarsUnlockCondition::PlayerLevel(minimum) = unlock
                && state.player_level < *minimum
            {
                return Err(error("Currency Wars player-level entry is locked"));
            }
        }
    }
    Ok(())
}

fn validate_gambit_unlock(
    catalog: &CurrencyWarsFlowCatalog,
    gambit: CurrencyWarsGambit,
    state: CurrencyWarsEntryState,
) -> Result<(), CurrencyWarsEntryError> {
    let definition = catalog
        .gambits()
        .iter()
        .find(|definition| definition.gambit == gambit)
        .ok_or_else(|| error("Currency Wars Gambit definition is missing"))?;
    if definition.unlocks.iter().any(|unlock| {
        matches!(
            unlock,
            CurrencyWarsUnlockCondition::CompleteOneStandardGambit
        ) && !state.completed_standard_gambit
    }) {
        return Err(error(
            "Currency Wars Overclock requires a completed Standard Gambit",
        ));
    }
    Ok(())
}

fn rank_maximum(
    catalog: &CurrencyWarsFlowCatalog,
    season: u16,
    highest_rank: u8,
    gambit: CurrencyWarsGambit,
) -> Result<u8, CurrencyWarsEntryError> {
    let boundary = catalog
        .rank_progression()
        .iter()
        .find_map(
            |progression| match (&progression.key, &progression.boundary) {
                (
                    CurrencyWarsRankProgressionKey::Division { season: row, level },
                    CurrencyWarsRankBoundary::GambitDifficulty {
                        maximum_standard,
                        maximum_overclock,
                        ..
                    },
                ) if *row == season && *level == highest_rank => {
                    Some((*maximum_standard, *maximum_overclock))
                }
                _ => None,
            },
        )
        .ok_or_else(|| error("Currency Wars highest Standard rank is not configured"))?;
    Ok(match gambit {
        CurrencyWarsGambit::Standard => boundary.0,
        CurrencyWarsGambit::Overclock => boundary.1,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEntryError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsEntryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsEntryError {}

fn error(message: &str) -> CurrencyWarsEntryError {
    CurrencyWarsEntryError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::{CurrencyWarsEntryState, CurrencyWarsRouteMembershipPolicy};
    use crate::{CurrencyWarsGambit, catalog::tests_support};

    #[test]
    fn standard_and_overclock_entry_apply_authored_unlocks_and_rank_caps() {
        let catalog = tests_support::catalog();
        let flow = catalog.flow_catalog();
        let route = catalog.routes()[0].id;

        assert_eq!(
            flow.route_membership_policy(),
            CurrencyWarsRouteMembershipPolicy::SharedCompleteRouteSet,
        );
        assert!(
            flow.resolve_entry(
                route,
                1,
                CurrencyWarsGambit::Standard,
                CurrencyWarsEntryState::new(21, false, 1),
            )
            .is_ok(),
        );
        assert_eq!(
            flow.resolve_entry(
                route,
                1,
                CurrencyWarsGambit::Standard,
                CurrencyWarsEntryState::new(20, false, 1),
            )
            .unwrap_err()
            .to_string(),
            "Currency Wars player-level entry is locked",
        );
        assert_eq!(
            flow.resolve_entry(
                route,
                1,
                CurrencyWarsGambit::Overclock,
                CurrencyWarsEntryState::new(21, false, 1),
            )
            .unwrap_err()
            .to_string(),
            "Currency Wars Overclock requires a completed Standard Gambit",
        );
        assert!(
            flow.resolve_entry(
                route,
                1,
                CurrencyWarsGambit::Overclock,
                CurrencyWarsEntryState::new(21, true, 1),
            )
            .is_ok(),
        );
        assert_eq!(
            flow.resolve_entry(
                route,
                2,
                CurrencyWarsGambit::Standard,
                CurrencyWarsEntryState::new(21, false, 1),
            )
            .unwrap_err()
            .to_string(),
            "Currency Wars difficulty exceeds the highest Standard rank",
        );
    }
}
