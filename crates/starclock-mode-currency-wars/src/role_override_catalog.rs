use std::collections::BTreeSet;

use crate::{
    CurrencyWarsMechanicActivityProgram, CurrencyWarsMechanicProgram,
    CurrencyWarsMechanicProgramDisposition, CurrencyWarsPositionKind, CurrencyWarsRoleId,
    CurrencyWarsStarState, CurrencyWarsStarStateOwner,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsOverrideConfigurationKind {
    Character,
    Servant,
    SummonBattleEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsCharacterOverrideBinding {
    RoleStar {
        role: CurrencyWarsRoleId,
        star_levels: Box<[u8]>,
    },
    ServantStar {
        role: CurrencyWarsRoleId,
        servant_id: u32,
        star_levels: Box<[u8]>,
    },
    SummonBattleEvent {
        season_id: u16,
        unit_id: u32,
        position: CurrencyWarsPositionKind,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOverrideSkillAbilityBinding {
    pub skill: Box<str>,
    pub ability_names: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOverrideSkillBinding {
    pub name: Box<str>,
    pub skill_type: Box<str>,
    pub use_type: Box<str>,
    pub target_type: Box<str>,
    pub entry_ability: Box<str>,
    pub prepare_ability: Box<str>,
    pub actual_attacker: Box<str>,
    pub child_skills: Box<[Box<str>]>,
    pub insertable: bool,
    pub insert_priority: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsOverrideDynamicSource {
    pub additive: bool,
    pub value_kind: Box<str>,
    pub key: Box<str>,
    pub source_kind: Box<str>,
    pub trigger_key: Box<str>,
    pub index: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCharacterOverrideProgram {
    pub stable_key: Box<str>,
    pub source_path: Box<str>,
    pub source_sha256: Box<str>,
    pub configuration_kind: CurrencyWarsOverrideConfigurationKind,
    pub parent_config_path: Box<str>,
    pub bindings: Box<[CurrencyWarsCharacterOverrideBinding]>,
    pub ability_names: Box<[Box<str>]>,
    pub skill_ability_bindings: Box<[CurrencyWarsOverrideSkillAbilityBinding]>,
    pub replaced_skills: Box<[Box<str>]>,
    pub skill_bindings: Box<[CurrencyWarsOverrideSkillBinding]>,
    pub dynamic_sources: Box<[CurrencyWarsOverrideDynamicSource]>,
    pub mechanical_shape_sha256: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCharacterOverridePolicy {
    pub policy_id: Box<str>,
    pub source_path: Box<str>,
    pub selected_behavior: Box<str>,
    pub replacement_condition: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoleOverrideCatalog {
    programs: Box<[CurrencyWarsCharacterOverrideProgram]>,
}

impl CurrencyWarsRoleOverrideCatalog {
    pub fn from_mechanic_programs(
        mechanics: &[CurrencyWarsMechanicProgram],
    ) -> Result<Self, CurrencyWarsRoleOverrideCatalogError> {
        let mut programs = mechanics
            .iter()
            .filter_map(|mechanic| match &mechanic.disposition {
                CurrencyWarsMechanicProgramDisposition::ExecutedActivity(
                    CurrencyWarsMechanicActivityProgram::CharacterOverride(program),
                ) => Some(program.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();
        programs.sort_by(|left, right| left.source_path.cmp(&right.source_path));
        validate(&programs)?;
        Ok(Self {
            programs: programs.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn programs(&self) -> &[CurrencyWarsCharacterOverrideProgram] {
        &self.programs
    }

    #[must_use]
    pub fn by_source_path(
        &self,
        source_path: &str,
    ) -> Option<&CurrencyWarsCharacterOverrideProgram> {
        self.programs
            .binary_search_by(|program| program.source_path.as_ref().cmp(source_path))
            .ok()
            .map(|index| &self.programs[index])
    }

    pub fn for_star_state(
        &self,
        state: &CurrencyWarsStarState,
    ) -> Result<Option<&CurrencyWarsCharacterOverrideProgram>, CurrencyWarsRoleOverrideCatalogError>
    {
        let Some(source_path) = state.config_path.as_deref() else {
            return Ok(None);
        };
        let Some(program) = self.by_source_path(source_path) else {
            return Ok(None);
        };
        let bound = program
            .bindings
            .iter()
            .any(|binding| match (binding, state.owner) {
                (
                    CurrencyWarsCharacterOverrideBinding::RoleStar { role, star_levels },
                    CurrencyWarsStarStateOwner::Role(owner),
                ) => *role == owner && star_levels.contains(&state.star),
                (
                    CurrencyWarsCharacterOverrideBinding::ServantStar {
                        role,
                        servant_id,
                        star_levels,
                    },
                    CurrencyWarsStarStateOwner::Servant {
                        avatar_id,
                        servant_id: owner_servant,
                    },
                ) => {
                    role.get() == avatar_id
                        && *servant_id == owner_servant
                        && star_levels.contains(&state.star)
                }
                _ => false,
            });
        if !bound {
            return Err(error(
                "Currency Wars star override binding does not match its owner",
            ));
        }
        Ok(Some(program))
    }

    #[must_use]
    pub fn policy_for_star_state(
        &self,
        state: &CurrencyWarsStarState,
    ) -> Option<CurrencyWarsCharacterOverridePolicy> {
        let source_path = state.config_path.as_deref()?;
        self.by_source_path(source_path).is_none().then(|| {
            CurrencyWarsCharacterOverridePolicy {
                policy_id: "currency-wars.character-override-base-build-policy.v1".into(),
                source_path: source_path.into(),
                selected_behavior: "UseCompiledSharedBaseBuildAndAuthoredStarMetadata".into(),
                replacement_condition:
                    "Replace when the released referenced character configuration is available in the pinned source closure."
                        .into(),
            }
        })
    }

    pub fn summon_battle_events(
        &self,
        season_id: u16,
    ) -> impl Iterator<Item = &CurrencyWarsCharacterOverrideProgram> {
        self.programs.iter().filter(move |program| {
            program.bindings.iter().any(|binding| {
                matches!(binding,
                    CurrencyWarsCharacterOverrideBinding::SummonBattleEvent {
                        season_id: binding_season,
                        ..
                    } if *binding_season == season_id)
            })
        })
    }
}

#[cfg(test)]
impl CurrencyWarsRoleOverrideCatalog {
    pub(crate) fn test_fixture(role: CurrencyWarsRoleId) -> Self {
        Self {
            programs: Box::new([CurrencyWarsCharacterOverrideProgram {
                stable_key: "override.fixture".into(),
                source_path: "fixture/override.json".into(),
                source_sha256: "00".repeat(32).into(),
                configuration_kind: CurrencyWarsOverrideConfigurationKind::Character,
                parent_config_path: "fixture/parent.json".into(),
                bindings: Box::new([CurrencyWarsCharacterOverrideBinding::RoleStar {
                    role,
                    star_levels: Box::new([1]),
                }]),
                ability_names: Box::new(["fixture_ability".into()]),
                skill_ability_bindings: Box::new([]),
                replaced_skills: Box::new([]),
                skill_bindings: Box::new([CurrencyWarsOverrideSkillBinding {
                    name: "Skill01".into(),
                    skill_type: "Basic".into(),
                    use_type: "SelectEntity".into(),
                    target_type: "EnemySelect".into(),
                    entry_ability: "fixture_ability".into(),
                    prepare_ability: "".into(),
                    actual_attacker: "".into(),
                    child_skills: Box::new([]),
                    insertable: false,
                    insert_priority: "".into(),
                }]),
                dynamic_sources: Box::new([]),
                mechanical_shape_sha256: "11".repeat(32).into(),
            }]),
        }
    }
}

fn validate(
    programs: &[CurrencyWarsCharacterOverrideProgram],
) -> Result<(), CurrencyWarsRoleOverrideCatalogError> {
    if programs.is_empty()
        || programs
            .windows(2)
            .any(|pair| pair[0].source_path >= pair[1].source_path)
        || programs.iter().any(|program| {
            program.stable_key.is_empty()
                || program.source_path.is_empty()
                || !valid_digest(&program.source_sha256)
                || !valid_digest(&program.mechanical_shape_sha256)
                || program.bindings.is_empty()
                || program.skill_bindings.is_empty()
                || !valid_bindings(&program.bindings)
        })
    {
        return Err(error("Currency Wars role override catalog is invalid"));
    }
    Ok(())
}

fn valid_bindings(bindings: &[CurrencyWarsCharacterOverrideBinding]) -> bool {
    let mut keys = BTreeSet::new();
    bindings.iter().all(|binding| match binding {
        CurrencyWarsCharacterOverrideBinding::RoleStar { role, star_levels } => {
            valid_stars(star_levels) && keys.insert((0, role.get(), 0, 0))
        }
        CurrencyWarsCharacterOverrideBinding::ServantStar {
            role,
            servant_id,
            star_levels,
        } => {
            *servant_id != 0
                && valid_stars(star_levels)
                && keys.insert((1, role.get(), *servant_id, 0))
        }
        CurrencyWarsCharacterOverrideBinding::SummonBattleEvent {
            season_id,
            unit_id,
            position,
        } => {
            *season_id != 0
                && *unit_id != 0
                && keys.insert((2, u32::from(*season_id), *unit_id, position_tag(*position)))
        }
    })
}

fn valid_stars(stars: &[u8]) -> bool {
    !stars.is_empty()
        && stars.iter().all(|star| *star > 0)
        && stars.windows(2).all(|pair| pair[0] < pair[1])
}

const fn position_tag(position: CurrencyWarsPositionKind) -> u32 {
    match position {
        CurrencyWarsPositionKind::Front => 0,
        CurrencyWarsPositionKind::Back => 1,
    }
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoleOverrideCatalogError {
    message: Box<str>,
}

impl std::fmt::Display for CurrencyWarsRoleOverrideCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for CurrencyWarsRoleOverrideCatalogError {}

fn error(message: &'static str) -> CurrencyWarsRoleOverrideCatalogError {
    CurrencyWarsRoleOverrideCatalogError {
        message: message.into(),
    }
}
