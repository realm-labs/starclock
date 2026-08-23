use starclock_combat::{Ratio, Rounding, Scalar};

use crate::{
    CurrencyWarsDifficulty, CurrencyWarsGambit, CurrencyWarsMechanicActivityProgram,
    CurrencyWarsMechanicProgram, CurrencyWarsMechanicProgramDisposition, CurrencyWarsRoleId,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsRunPosition {
    pub chapter: u8,
    pub section: u8,
}

impl CurrencyWarsRunPosition {
    pub fn new(chapter: u8, section: u8) -> Result<Self, CurrencyWarsProgressionCatalogError> {
        if chapter == 0 || section == 0 {
            return Err(error("Currency Wars run position is zero"));
        }
        Ok(Self { chapter, section })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoleCostAvailability {
    pub stable_key: Box<str>,
    pub cost: u8,
    pub standard: CurrencyWarsRunPosition,
    pub overclock: CurrencyWarsRunPosition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSeasonProgressionRule {
    pub stable_key: Box<str>,
    pub division: u8,
    pub score_rule: u16,
    pub position: CurrencyWarsRunPosition,
    pub weekly_score: Option<u32>,
    pub experience: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsModuleRoleBan {
    pub stable_key: Box<str>,
    pub module: u32,
    pub role: CurrencyWarsRoleId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSeasonRolePool {
    pub stable_key: Box<str>,
    pub season: u16,
    pub roles: Box<[CurrencyWarsRoleId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSeasonTraitRolePool {
    pub stable_key: Box<str>,
    pub season: u16,
    pub trait_id: u32,
    pub roles: Box<[CurrencyWarsRoleId]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoleReferenceScore {
    pub stable_key: Box<str>,
    pub season: u16,
    pub role: CurrencyWarsRoleId,
    pub score: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsProgressionProgram {
    RoleCostAvailability(CurrencyWarsRoleCostAvailability),
    SeasonScoreAndExperience(CurrencyWarsSeasonProgressionRule),
    ModuleRoleBan(CurrencyWarsModuleRoleBan),
    SeasonRolePool(CurrencyWarsSeasonRolePool),
    SeasonTraitRolePool(CurrencyWarsSeasonTraitRolePool),
    RoleReferenceScore(CurrencyWarsRoleReferenceScore),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProgressionProjection<'a> {
    pub rule: &'a CurrencyWarsSeasonProgressionRule,
    pub weekly_score_modifier: Ratio,
    pub experience_modifier: Ratio,
    pub exact_weekly_score: Option<Scalar>,
    pub exact_experience: Option<Scalar>,
    /// Exact percentage applied to any Ascension Point reward owned by the
    /// surrounding settlement boundary. This catalog has no base point amount.
    pub talent_point_modifier: Ratio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProgressionModifiers {
    pub weekly_score: Ratio,
    pub experience: Ratio,
    pub talent_points: Ratio,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProgressionCatalog {
    cost_availability: Box<[CurrencyWarsRoleCostAvailability]>,
    season_rules: Box<[CurrencyWarsSeasonProgressionRule]>,
    module_role_bans: Box<[CurrencyWarsModuleRoleBan]>,
    season_role_pools: Box<[CurrencyWarsSeasonRolePool]>,
    season_trait_role_pools: Box<[CurrencyWarsSeasonTraitRolePool]>,
    role_reference_scores: Box<[CurrencyWarsRoleReferenceScore]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProgressionCatalogParts {
    pub cost_availability: Vec<CurrencyWarsRoleCostAvailability>,
    pub season_rules: Vec<CurrencyWarsSeasonProgressionRule>,
    pub module_role_bans: Vec<CurrencyWarsModuleRoleBan>,
    pub season_role_pools: Vec<CurrencyWarsSeasonRolePool>,
    pub season_trait_role_pools: Vec<CurrencyWarsSeasonTraitRolePool>,
    pub role_reference_scores: Vec<CurrencyWarsRoleReferenceScore>,
}

impl CurrencyWarsProgressionCatalog {
    pub fn new(
        mut parts: CurrencyWarsProgressionCatalogParts,
    ) -> Result<Self, CurrencyWarsProgressionCatalogError> {
        parts.cost_availability.sort_by_key(|rule| rule.cost);
        parts.season_rules.sort_by_key(|rule| {
            (
                rule.division,
                rule.score_rule,
                rule.position,
                rule.stable_key.clone(),
            )
        });
        parts
            .module_role_bans
            .sort_by_key(|rule| (rule.module, rule.role));
        parts.season_role_pools.sort_by_key(|rule| rule.season);
        parts
            .season_trait_role_pools
            .sort_by_key(|rule| (rule.season, rule.trait_id));
        parts
            .role_reference_scores
            .sort_by_key(|rule| (rule.season, rule.role));
        validate(&parts)?;
        Ok(Self {
            cost_availability: parts.cost_availability.into_boxed_slice(),
            season_rules: parts.season_rules.into_boxed_slice(),
            module_role_bans: parts.module_role_bans.into_boxed_slice(),
            season_role_pools: parts.season_role_pools.into_boxed_slice(),
            season_trait_role_pools: parts.season_trait_role_pools.into_boxed_slice(),
            role_reference_scores: parts.role_reference_scores.into_boxed_slice(),
        })
    }

    pub fn from_mechanic_programs(
        programs: &[CurrencyWarsMechanicProgram],
    ) -> Result<Self, CurrencyWarsProgressionCatalogError> {
        let mut cost_availability = Vec::new();
        let mut season_rules = Vec::new();
        let mut module_role_bans = Vec::new();
        let mut season_role_pools = Vec::new();
        let mut season_trait_role_pools = Vec::new();
        let mut role_reference_scores = Vec::new();
        for program in programs {
            let CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
                CurrencyWarsMechanicActivityProgram::Progression(progression),
            ) = &program.disposition
            else {
                continue;
            };
            match progression {
                CurrencyWarsProgressionProgram::RoleCostAvailability(rule) => {
                    cost_availability.push(rule.clone());
                }
                CurrencyWarsProgressionProgram::SeasonScoreAndExperience(rule) => {
                    season_rules.push(rule.clone());
                }
                CurrencyWarsProgressionProgram::ModuleRoleBan(rule) => {
                    module_role_bans.push(rule.clone());
                }
                CurrencyWarsProgressionProgram::SeasonRolePool(rule) => {
                    season_role_pools.push(rule.clone());
                }
                CurrencyWarsProgressionProgram::SeasonTraitRolePool(rule) => {
                    season_trait_role_pools.push(rule.clone());
                }
                CurrencyWarsProgressionProgram::RoleReferenceScore(rule) => {
                    role_reference_scores.push(rule.clone());
                }
            }
        }
        Self::new(CurrencyWarsProgressionCatalogParts {
            cost_availability,
            season_rules,
            module_role_bans,
            season_role_pools,
            season_trait_role_pools,
            role_reference_scores,
        })
    }

    #[must_use]
    pub fn cost_availability(&self) -> &[CurrencyWarsRoleCostAvailability] {
        &self.cost_availability
    }

    #[must_use]
    pub fn season_rules(&self) -> &[CurrencyWarsSeasonProgressionRule] {
        &self.season_rules
    }

    #[must_use]
    pub fn module_role_bans(&self) -> &[CurrencyWarsModuleRoleBan] {
        &self.module_role_bans
    }

    #[must_use]
    pub fn season_role_pools(&self) -> &[CurrencyWarsSeasonRolePool] {
        &self.season_role_pools
    }

    #[must_use]
    pub fn season_trait_role_pools(&self) -> &[CurrencyWarsSeasonTraitRolePool] {
        &self.season_trait_role_pools
    }

    #[must_use]
    pub fn role_reference_scores(&self) -> &[CurrencyWarsRoleReferenceScore] {
        &self.role_reference_scores
    }

    #[must_use]
    pub fn role_available(&self, season: u16, module: u32, role: CurrencyWarsRoleId) -> bool {
        self.season_role_pools
            .binary_search_by_key(&season, |pool| pool.season)
            .ok()
            .is_some_and(|index| {
                self.season_role_pools[index]
                    .roles
                    .binary_search(&role)
                    .is_ok()
            })
            && self
                .module_role_bans
                .binary_search_by_key(&(module, role), |ban| (ban.module, ban.role))
                .is_err()
    }

    #[must_use]
    pub fn trait_roles(&self, season: u16, trait_id: u32) -> Option<&[CurrencyWarsRoleId]> {
        self.season_trait_role_pools
            .binary_search_by_key(&(season, trait_id), |pool| (pool.season, pool.trait_id))
            .ok()
            .map(|index| self.season_trait_role_pools[index].roles.as_ref())
    }

    #[must_use]
    pub fn role_reference_score(&self, season: u16, role: CurrencyWarsRoleId) -> Option<u16> {
        self.role_reference_scores
            .binary_search_by_key(&(season, role), |score| (score.season, score.role))
            .ok()
            .map(|index| self.role_reference_scores[index].score)
    }

    #[must_use]
    pub fn rank_role_candidates(
        &self,
        season: u16,
        candidates: impl IntoIterator<Item = CurrencyWarsRoleId>,
    ) -> Box<[CurrencyWarsRoleId]> {
        let mut candidates = candidates.into_iter().collect::<Vec<_>>();
        candidates.sort_by_key(|role| {
            (
                std::cmp::Reverse(self.role_reference_score(season, *role).unwrap_or_default()),
                *role,
            )
        });
        candidates.dedup();
        candidates.into_boxed_slice()
    }

    #[must_use]
    pub fn role_cost_available(
        &self,
        gambit: CurrencyWarsGambit,
        cost: u8,
        position: CurrencyWarsRunPosition,
    ) -> bool {
        self.cost_availability
            .binary_search_by_key(&cost, |rule| rule.cost)
            .ok()
            .map(|index| match gambit {
                CurrencyWarsGambit::Standard => self.cost_availability[index].standard,
                CurrencyWarsGambit::Overclock => self.cost_availability[index].overclock,
            })
            .is_some_and(|required| position >= required)
    }

    pub fn project(
        &self,
        difficulty: &CurrencyWarsDifficulty,
        gambit: CurrencyWarsGambit,
        position: CurrencyWarsRunPosition,
        gambit_modifiers: CurrencyWarsProgressionModifiers,
    ) -> Result<Option<CurrencyWarsProgressionProjection<'_>>, CurrencyWarsProgressionCatalogError>
    {
        let score_rule = match gambit {
            CurrencyWarsGambit::Standard => difficulty.standard_score_rule,
            CurrencyWarsGambit::Overclock => difficulty.overclock_score_rule,
        };
        let score_rule =
            u16::try_from(score_rule).map_err(|_| error("Currency Wars score rule exceeds u16"))?;
        let key = (difficulty.division_level, score_rule, position);
        let Ok(index) = self
            .season_rules
            .binary_search_by_key(&key, |rule| (rule.division, rule.score_rule, rule.position))
        else {
            return Ok(None);
        };
        let rule = &self.season_rules[index];
        let weekly_score_modifier = compose_percent(
            difficulty.weekly_score_modifier,
            gambit_modifiers.weekly_score,
        )?;
        let experience_modifier =
            compose_percent(difficulty.experience_modifier, gambit_modifiers.experience)?;
        Ok(Some(CurrencyWarsProgressionProjection {
            rule,
            weekly_score_modifier,
            experience_modifier,
            exact_weekly_score: scale(rule.weekly_score, weekly_score_modifier)?,
            exact_experience: scale(rule.experience, experience_modifier)?,
            talent_point_modifier: gambit_modifiers.talent_points,
        }))
    }
}

fn validate(
    parts: &CurrencyWarsProgressionCatalogParts,
) -> Result<(), CurrencyWarsProgressionCatalogError> {
    if parts.cost_availability.len() != 5
        || parts
            .cost_availability
            .iter()
            .enumerate()
            .any(|(index, rule)| {
                rule.stable_key.is_empty()
                    || usize::from(rule.cost) != index + 1
                    || rule.standard.chapter == 0
                    || rule.standard.section == 0
                    || rule.overclock.chapter == 0
                    || rule.overclock.section == 0
            })
        || parts.season_rules.is_empty()
        || parts.season_rules.iter().any(|rule| {
            rule.stable_key.is_empty()
                || rule.division == 0
                || rule.score_rule == 0
                || rule.position.chapter == 0
                || rule.position.section == 0
        })
        || parts.season_rules.windows(2).any(|pair| {
            (pair[0].division, pair[0].score_rule, pair[0].position)
                >= (pair[1].division, pair[1].score_rule, pair[1].position)
        })
        || !valid_role_progression(parts)
    {
        return Err(error("Currency Wars progression catalog is invalid"));
    }
    Ok(())
}

fn valid_role_progression(parts: &CurrencyWarsProgressionCatalogParts) -> bool {
    !parts.module_role_bans.is_empty()
        && !parts.season_role_pools.is_empty()
        && !parts.season_trait_role_pools.is_empty()
        && !parts.role_reference_scores.is_empty()
        && parts
            .module_role_bans
            .iter()
            .all(|rule| !rule.stable_key.is_empty() && rule.module != 0)
        && parts
            .module_role_bans
            .windows(2)
            .all(|pair| (pair[0].module, pair[0].role) < (pair[1].module, pair[1].role))
        && parts.season_role_pools.iter().all(|pool| {
            !pool.stable_key.is_empty()
                && pool.season != 0
                && !pool.roles.is_empty()
                && pool.roles.windows(2).all(|pair| pair[0] < pair[1])
        })
        && parts
            .season_role_pools
            .windows(2)
            .all(|pair| pair[0].season < pair[1].season)
        && parts.season_trait_role_pools.iter().all(|pool| {
            !pool.stable_key.is_empty()
                && pool.season != 0
                && pool.trait_id != 0
                && !pool.roles.is_empty()
                && pool.roles.windows(2).all(|pair| pair[0] < pair[1])
                && pool.roles.iter().all(|role| {
                    parts.season_role_pools.iter().any(|season| {
                        season.season == pool.season && season.roles.binary_search(role).is_ok()
                    })
                })
        })
        && parts
            .season_trait_role_pools
            .windows(2)
            .all(|pair| (pair[0].season, pair[0].trait_id) < (pair[1].season, pair[1].trait_id))
        && parts.role_reference_scores.iter().all(|score| {
            !score.stable_key.is_empty()
                && score.season != 0
                && score.score != 0
                && parts.season_role_pools.iter().any(|pool| {
                    pool.season == score.season && pool.roles.binary_search(&score.role).is_ok()
                })
        })
        && parts
            .role_reference_scores
            .windows(2)
            .all(|pair| (pair[0].season, pair[0].role) < (pair[1].season, pair[1].role))
}

fn scale(
    value: Option<u32>,
    modifier: Ratio,
) -> Result<Option<Scalar>, CurrencyWarsProgressionCatalogError> {
    value
        .map(|value| {
            let value = i64::from(value);
            modifier
                .checked_div(Ratio::from_scaled(100_000_000), Rounding::TowardZero)
                .and_then(|modifier| {
                    Scalar::checked_from_integer(value)
                        .and_then(|value| modifier.checked_apply(value, Rounding::TowardZero))
                })
                .map_err(|_| error("Currency Wars progression projection overflowed"))
        })
        .transpose()
}

fn compose_percent(
    left: Ratio,
    right: Ratio,
) -> Result<Ratio, CurrencyWarsProgressionCatalogError> {
    left.checked_mul(right, Rounding::TowardZero)
        .and_then(|value| value.checked_div(Ratio::from_scaled(100_000_000), Rounding::TowardZero))
        .map_err(|_| error("Currency Wars progression modifier overflowed"))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsProgressionCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsProgressionCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsProgressionCatalogError {}

fn error(message: &'static str) -> CurrencyWarsProgressionCatalogError {
    CurrencyWarsProgressionCatalogError {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CurrencyWarsDifficultyEnemyScaling;

    #[test]
    fn role_cost_availability_uses_gambit_specific_inclusive_thresholds() {
        let catalog = catalog(
            CurrencyWarsRunPosition::new(2, 3).unwrap(),
            CurrencyWarsRunPosition::new(2, 1).unwrap(),
        );

        assert!(!catalog.role_cost_available(
            CurrencyWarsGambit::Standard,
            5,
            CurrencyWarsRunPosition::new(2, 2).unwrap(),
        ));
        assert!(catalog.role_cost_available(
            CurrencyWarsGambit::Standard,
            5,
            CurrencyWarsRunPosition::new(2, 3).unwrap(),
        ));
        assert!(catalog.role_cost_available(
            CurrencyWarsGambit::Overclock,
            5,
            CurrencyWarsRunPosition::new(2, 1).unwrap(),
        ));
        assert!(!catalog.role_cost_available(
            CurrencyWarsGambit::Standard,
            6,
            CurrencyWarsRunPosition::new(9, 9).unwrap(),
        ));
    }

    #[test]
    fn season_projection_preserves_optional_values_and_exact_fixed_point_products() {
        let weekly_modifier = Ratio::from_scaled(105_000_000);
        let experience_modifier = Ratio::from_scaled(180_000_000);
        let catalog = catalog(
            CurrencyWarsRunPosition::new(1, 1).unwrap(),
            CurrencyWarsRunPosition::new(1, 1).unwrap(),
        );
        let difficulty = difficulty(weekly_modifier, experience_modifier);
        let projected = catalog
            .project(
                &difficulty,
                CurrencyWarsGambit::Standard,
                CurrencyWarsRunPosition::new(1, 1).unwrap(),
                standard_modifiers(),
            )
            .unwrap()
            .unwrap();

        assert_eq!(projected.exact_weekly_score, None);
        assert_eq!(
            projected.exact_experience,
            Some(Scalar::from_scaled(45_000_000))
        );
        assert_eq!(projected.weekly_score_modifier, weekly_modifier);
        assert_eq!(projected.experience_modifier, experience_modifier);
        assert!(
            catalog
                .project(
                    &difficulty,
                    CurrencyWarsGambit::Standard,
                    CurrencyWarsRunPosition::new(1, 2).unwrap(),
                    standard_modifiers(),
                )
                .unwrap()
                .is_none()
        );
    }

    fn standard_modifiers() -> CurrencyWarsProgressionModifiers {
        CurrencyWarsProgressionModifiers {
            weekly_score: Ratio::from_scaled(100_000_000),
            experience: Ratio::from_scaled(100_000_000),
            talent_points: Ratio::from_scaled(100_000_000),
        }
    }

    #[test]
    fn role_policy_applies_season_module_trait_and_reference_score() {
        let catalog = catalog(
            CurrencyWarsRunPosition::new(1, 1).unwrap(),
            CurrencyWarsRunPosition::new(1, 1).unwrap(),
        );
        let first = CurrencyWarsRoleId::new(1).unwrap();
        let second = CurrencyWarsRoleId::new(2).unwrap();

        assert!(catalog.role_available(1, 1, first));
        assert!(!catalog.role_available(1, 2, first));
        assert!(!catalog.role_available(2, 1, first));
        assert_eq!(catalog.trait_roles(1, 1), Some([first, second].as_slice()));
        assert_eq!(
            catalog
                .rank_role_candidates(1, [first, second, first])
                .as_ref(),
            [second, first]
        );
    }

    fn catalog(
        standard: CurrencyWarsRunPosition,
        overclock: CurrencyWarsRunPosition,
    ) -> CurrencyWarsProgressionCatalog {
        let role = CurrencyWarsRoleId::new(1).unwrap();
        let second_role = CurrencyWarsRoleId::new(2).unwrap();
        CurrencyWarsProgressionCatalog::new(CurrencyWarsProgressionCatalogParts {
            cost_availability: (1..=5)
                .map(|cost| CurrencyWarsRoleCostAvailability {
                    stable_key: format!("cost.{cost}").into(),
                    cost,
                    standard,
                    overclock,
                })
                .collect(),
            season_rules: vec![CurrencyWarsSeasonProgressionRule {
                stable_key: "season.1".into(),
                division: 1,
                score_rule: 1,
                position: CurrencyWarsRunPosition::new(1, 1).unwrap(),
                weekly_score: None,
                experience: Some(25),
            }],
            module_role_bans: vec![CurrencyWarsModuleRoleBan {
                stable_key: "ban.2.1".into(),
                module: 2,
                role,
            }],
            season_role_pools: vec![CurrencyWarsSeasonRolePool {
                stable_key: "season-role.1".into(),
                season: 1,
                roles: Box::new([role, second_role]),
            }],
            season_trait_role_pools: vec![CurrencyWarsSeasonTraitRolePool {
                stable_key: "season-trait.1.1".into(),
                season: 1,
                trait_id: 1,
                roles: Box::new([role, second_role]),
            }],
            role_reference_scores: vec![
                CurrencyWarsRoleReferenceScore {
                    stable_key: "role-score.1.1".into(),
                    season: 1,
                    role,
                    score: 3,
                },
                CurrencyWarsRoleReferenceScore {
                    stable_key: "role-score.1.2".into(),
                    season: 1,
                    role: second_role,
                    score: 10,
                },
            ],
        })
        .expect("progression fixture is valid")
    }

    fn difficulty(
        weekly_score_modifier: Ratio,
        experience_modifier: Ratio,
    ) -> CurrencyWarsDifficulty {
        CurrencyWarsDifficulty {
            source_id: 1,
            stable_key: "difficulty.1".into(),
            season_id: 1,
            division_level: 1,
            progress: 1,
            standard_score_rule: 1,
            overclock_score_rule: 2,
            weekly_score_modifier,
            experience_modifier,
            enemy_scaling_refs: Box::new([]),
            enemy_scaling: CurrencyWarsDifficultyEnemyScaling {
                enemy_difficulty_level: 0,
                level_base_hp_ratio: Scalar::ONE,
                level_base_attack_ratio: Scalar::ONE,
            },
            enemy_affix_choice_counts: Box::new([]),
            binary_difficulty_rule: None,
        }
    }
}
