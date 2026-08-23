#[cfg(test)]
use crate::{
    CurrencyWarsBackBattleEvent, CurrencyWarsBattleEventKind, CurrencyWarsBattleEventTeam,
    CurrencyWarsBattleOverrideDefinition, CurrencyWarsCyreneSkillOverride,
    CurrencyWarsFrontSpecialResource, CurrencyWarsLethalRescueHpPolicy,
    CurrencyWarsRankSkillOverride, CurrencyWarsRoleGlobalModifier, CurrencyWarsRoleId,
    CurrencyWarsSkillParameterEdit, CurrencyWarsSkillParameterOperator,
    CurrencyWarsSpecialResourceKind, CurrencyWarsSummonBattleEventOverride,
};
use crate::{
    CurrencyWarsBattleOverride, CurrencyWarsBattleOverrideRoleBuild,
    CurrencyWarsBattleOverrideSnapshot, CurrencyWarsDecimal, CurrencyWarsDeployment,
    CurrencyWarsPosition, CurrencyWarsPositionKind, CurrencyWarsRole, CurrencyWarsRoleState,
    CurrencyWarsStarState, CurrencyWarsStarStateOwner,
    battle_override::{
        CurrencyWarsBattleOverrideEnvironment, resolve_battle_overrides, validate_battle_overrides,
    },
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCharacterEmpowerment {
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub avatar_id: Option<u32>,
    pub skill_id: Option<u32>,
    pub position: CurrencyWarsPositionKind,
    pub activation: Box<str>,
    pub effect_ids: Box<[Box<str>]>,
    pub category_tags: Box<[Box<str>]>,
    pub skill_level: Option<u8>,
    pub cooldown: Option<i32>,
    pub initial_cooldown: Option<i32>,
    pub sp_multiple_ratio: Option<Box<str>>,
    pub delay_ratio: Option<Box<str>>,
    pub parameter_values: Box<[CurrencyWarsDecimal]>,
    pub teardown: Box<str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEmpowermentCatalog {
    empowerments: Box<[CurrencyWarsCharacterEmpowerment]>,
    battle_overrides: Box<[CurrencyWarsBattleOverride]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActiveEmpowermentSkill {
    pub skill_id: u32,
    pub levels: Box<[u8]>,
    pub stable_keys: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActiveCharacterEmpowerment {
    pub position: CurrencyWarsPosition,
    pub role_state: CurrencyWarsRoleState,
    pub display_stable_key: Box<str>,
    pub category_tags: Box<[Box<str>]>,
    pub skills: Box<[CurrencyWarsActiveEmpowermentSkill]>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrencyWarsEmpowermentSnapshot {
    active: Box<[CurrencyWarsActiveCharacterEmpowerment]>,
}

impl CurrencyWarsEmpowermentSnapshot {
    #[must_use]
    pub fn active(&self) -> &[CurrencyWarsActiveCharacterEmpowerment] {
        &self.active
    }
}

impl CurrencyWarsEmpowermentCatalog {
    pub fn new(
        mut empowerments: Vec<CurrencyWarsCharacterEmpowerment>,
        mut battle_overrides: Vec<CurrencyWarsBattleOverride>,
    ) -> Result<Self, CurrencyWarsEmpowermentCatalogError> {
        empowerments.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        battle_overrides.sort_by(|left, right| left.stable_key.cmp(&right.stable_key));
        if empowerments.is_empty()
            || battle_overrides.is_empty()
            || empowerments
                .windows(2)
                .any(|pair| pair[0].stable_key == pair[1].stable_key)
            || battle_overrides
                .windows(2)
                .any(|pair| pair[0].stable_key == pair[1].stable_key)
        {
            return Err(error(
                "Currency Wars Empowerment catalog is empty or duplicated",
            ));
        }
        validate_battle_overrides(&battle_overrides)?;
        Ok(Self {
            empowerments: empowerments.into_boxed_slice(),
            battle_overrides: battle_overrides.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn empowerments(&self) -> &[CurrencyWarsCharacterEmpowerment] {
        &self.empowerments
    }
    #[must_use]
    pub fn battle_overrides(&self) -> &[CurrencyWarsBattleOverride] {
        &self.battle_overrides
    }

    pub(crate) fn skill_row(
        &self,
        position: CurrencyWarsPositionKind,
        skill_id: u32,
        level: u8,
    ) -> Option<&CurrencyWarsCharacterEmpowerment> {
        self.empowerments.iter().find(|row| {
            row.position == position
                && row.skill_id == Some(skill_id)
                && row.skill_level == Some(level)
        })
    }

    pub(crate) fn maximum_skill_level(
        &self,
        position: CurrencyWarsPositionKind,
        skill_id: u32,
    ) -> Option<u8> {
        self.empowerments
            .iter()
            .filter(|row| row.position == position && row.skill_id == Some(skill_id))
            .filter_map(|row| row.skill_level)
            .max()
    }

    pub(crate) fn battle_override_snapshot(
        &self,
        deployment: &CurrencyWarsDeployment,
        roles: &[CurrencyWarsRole],
        builds: &[CurrencyWarsBattleOverrideRoleBuild],
        battle_event_ids: &[u32],
        star_states: &[CurrencyWarsStarState],
        environment: CurrencyWarsBattleOverrideEnvironment,
    ) -> Result<CurrencyWarsBattleOverrideSnapshot, CurrencyWarsEmpowermentCatalogError> {
        resolve_battle_overrides(
            &self.battle_overrides,
            deployment,
            roles,
            builds,
            battle_event_ids,
            star_states,
            environment,
        )
    }

    pub(crate) fn resolve(
        &self,
        deployment: &CurrencyWarsDeployment,
        roles: &[CurrencyWarsRole],
        star_states: &[CurrencyWarsStarState],
    ) -> Result<CurrencyWarsEmpowermentSnapshot, CurrencyWarsEmpowermentCatalogError> {
        let mut active = Vec::new();
        for (position, role_state) in deployment.positions() {
            let role = roles
                .binary_search_by_key(&role_state.role(), |role| role.id)
                .ok()
                .map(|index| &roles[index])
                .ok_or_else(|| error("Currency Wars Empowerment role is missing"))?;
            if !role.positions.contains(&position.kind()) {
                continue;
            }
            let display = self
                .empowerments
                .iter()
                .find(|value| {
                    value.avatar_id == Some(role.id.get()) && value.position == position.kind()
                })
                .ok_or_else(|| error("Currency Wars Empowerment display is missing"))?;
            let owner = CurrencyWarsStarStateOwner::Role(role_state.role());
            let star_state = star_states
                .binary_search_by_key(&(owner, role_state.star()), |value| {
                    (value.owner, value.star)
                })
                .ok()
                .map(|index| &star_states[index])
                .ok_or_else(|| error("Currency Wars Empowerment star state is missing"))?;
            let skill_ids = match position.kind() {
                CurrencyWarsPositionKind::Front => &star_state.front_execution_skill_ids,
                CurrencyWarsPositionKind::Back => &star_state.back_execution_skill_ids,
            };
            let skills = skill_ids
                .iter()
                .map(|skill_id| self.skill_family(position.kind(), *skill_id))
                .collect::<Result<Vec<_>, _>>()?;
            active.push(CurrencyWarsActiveCharacterEmpowerment {
                position: *position,
                role_state: *role_state,
                display_stable_key: display.stable_key.clone(),
                category_tags: display.category_tags.clone(),
                skills: skills.into_boxed_slice(),
            });
        }
        Ok(CurrencyWarsEmpowermentSnapshot {
            active: active.into_boxed_slice(),
        })
    }

    fn skill_family(
        &self,
        position: CurrencyWarsPositionKind,
        skill_id: u32,
    ) -> Result<CurrencyWarsActiveEmpowermentSkill, CurrencyWarsEmpowermentCatalogError> {
        let mut rows = self
            .empowerments
            .iter()
            .filter(|value| value.position == position && value.skill_id == Some(skill_id))
            .collect::<Vec<_>>();
        rows.sort_by_key(|row| row.skill_level);
        if rows.is_empty() {
            return Err(error("Currency Wars Empowerment skill family is missing"));
        }
        let levels = rows
            .iter()
            .map(|row| row.skill_level.expect("skill rows have a level"))
            .collect::<Vec<_>>();
        if levels.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(error("Currency Wars Empowerment skill levels are invalid"));
        }
        Ok(CurrencyWarsActiveEmpowermentSkill {
            skill_id,
            levels: levels.into_boxed_slice(),
            stable_keys: rows
                .into_iter()
                .map(|row| row.stable_key.clone())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
    }
}

#[cfg(test)]
impl CurrencyWarsEmpowermentCatalog {
    pub(crate) fn test_fixture(avatar_id: u32) -> Self {
        Self::new(
            vec![
                CurrencyWarsCharacterEmpowerment {
                    stable_key: "empowerment.fixture".into(),
                    source_id: "fixture".into(),
                    avatar_id: Some(avatar_id),
                    skill_id: None,
                    position: CurrencyWarsPositionKind::Front,
                    activation: "fixture".into(),
                    effect_ids: Box::new([]),
                    category_tags: Box::new([]),
                    skill_level: None,
                    cooldown: None,
                    initial_cooldown: None,
                    sp_multiple_ratio: None,
                    delay_ratio: None,
                    parameter_values: Box::new([]),
                    teardown: "fixture".into(),
                },
                CurrencyWarsCharacterEmpowerment {
                    stable_key: "empowerment.fixture.skill.1".into(),
                    source_id: "1:1".into(),
                    avatar_id: None,
                    skill_id: Some(1),
                    position: CurrencyWarsPositionKind::Front,
                    activation: "fixture".into(),
                    effect_ids: Box::new([]),
                    category_tags: Box::new([]),
                    skill_level: Some(1),
                    cooldown: None,
                    initial_cooldown: None,
                    sp_multiple_ratio: None,
                    delay_ratio: None,
                    parameter_values: Box::new([
                        CurrencyWarsDecimal::new(1, 0).unwrap(),
                        CurrencyWarsDecimal::new(2, 0).unwrap(),
                    ]),
                    teardown: "fixture".into(),
                },
            ],
            vec![
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.automatic".into(),
                    source_id: "automatic".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::AutomaticTechnique {
                        eligible_position: CurrencyWarsPositionKind::Front,
                    },
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.energy".into(),
                    source_id: "energy".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::DefeatEnergyScaling {
                        regular_energy_ratio: starclock_combat::Ratio::from_scaled(500_000),
                    },
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.rescue".into(),
                    source_id: "rescue".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::LethalDamageRescue {
                        hp_policy: CurrencyWarsLethalRescueHpPolicy::FullMaximumHp,
                    },
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.event.12".into(),
                    source_id: "12".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::BackBattleEvent(
                        CurrencyWarsBackBattleEvent {
                            event_id: 12,
                            kind: CurrencyWarsBattleEventKind::Assist,
                            team: CurrencyWarsBattleEventTeam::Neutral,
                            abilities: Box::new(["fixture.ability.12".into()]),
                            speed: None,
                            hard_level: true,
                            values: Box::new([]),
                            properties: Box::new([]),
                        },
                    ),
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.event.22".into(),
                    source_id: "22".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::BackBattleEvent(
                        CurrencyWarsBackBattleEvent {
                            event_id: 22,
                            kind: CurrencyWarsBattleEventKind::TraitAssist,
                            team: CurrencyWarsBattleEventTeam::Neutral,
                            abilities: Box::new([]),
                            speed: None,
                            hard_level: true,
                            values: Box::new([]),
                            properties: Box::new([]),
                        },
                    ),
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.resource".into(),
                    source_id: "1001.1".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::FrontSpecialResource(
                        CurrencyWarsFrontSpecialResource {
                            role: CurrencyWarsRoleId::new(avatar_id).unwrap(),
                            star: 1,
                            kind: CurrencyWarsSpecialResourceKind::EnergyBar,
                            maximum: CurrencyWarsDecimal::new(12, 0).unwrap(),
                        },
                    ),
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.global".into(),
                    source_id: "1001".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::RoleGlobalModifier(
                        CurrencyWarsRoleGlobalModifier {
                            role: CurrencyWarsRoleId::new(avatar_id).unwrap(),
                            saved_value: Some("fixture.saved".into()),
                            values: Box::new([CurrencyWarsDecimal::new(30, 0).unwrap()]),
                        },
                    ),
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.rank".into(),
                    source_id: "100101.1".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::RankSkillOverride(
                        CurrencyWarsRankSkillOverride {
                            rank_id: 100_101,
                            skill_id: 1,
                            edits: Box::new([CurrencyWarsSkillParameterEdit {
                                index: 1,
                                operator: CurrencyWarsSkillParameterOperator::Add,
                                value: CurrencyWarsDecimal::new(1, 0).unwrap(),
                            }]),
                        },
                    ),
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.summon".into(),
                    source_id: "1.1".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::SummonBattleEventOverride(
                        CurrencyWarsSummonBattleEventOverride {
                            season_id: 1,
                            battle_event_id: 1,
                            front_config: Some("fixture.front.json".into()),
                            back_config: None,
                        },
                    ),
                    teardown: "fixture".into(),
                },
                CurrencyWarsBattleOverride {
                    stable_key: "battle-override.fixture.cyrene".into(),
                    source_id: "1001.1.fixture".into(),
                    definition: CurrencyWarsBattleOverrideDefinition::CyreneSkillOverride(
                        CurrencyWarsCyreneSkillOverride {
                            provider_role: CurrencyWarsRoleId::new(avatar_id).unwrap(),
                            role: CurrencyWarsRoleId::new(avatar_id).unwrap(),
                            skill_id: 1,
                            multiple_value_key: "fixture.cyrene".into(),
                            edits: Box::new([CurrencyWarsSkillParameterEdit {
                                index: 1,
                                operator: CurrencyWarsSkillParameterOperator::Multiply,
                                value: CurrencyWarsDecimal::new(12, 1).unwrap(),
                            }]),
                        },
                    ),
                    teardown: "fixture".into(),
                },
            ],
        )
        .expect("test Empowerment catalog is valid")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsEmpowermentCatalogError {
    message: Box<str>,
}
impl std::fmt::Display for CurrencyWarsEmpowermentCatalogError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for CurrencyWarsEmpowermentCatalogError {}
impl CurrencyWarsEmpowermentCatalogError {
    pub(crate) fn new(message: &'static str) -> Self {
        Self {
            message: message.into(),
        }
    }
}
fn error(message: &'static str) -> CurrencyWarsEmpowermentCatalogError {
    CurrencyWarsEmpowermentCatalogError::new(message)
}
