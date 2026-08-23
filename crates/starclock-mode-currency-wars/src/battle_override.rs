use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{AbilityId, ActionValue, Energy, Hp, Ratio, Rounding, Scalar};

use crate::{
    CurrencyWarsDeployment, CurrencyWarsEmpowermentCatalogError, CurrencyWarsPosition,
    CurrencyWarsPositionKind, CurrencyWarsRole, CurrencyWarsRoleId, CurrencyWarsRoleState,
    CurrencyWarsStarState, CurrencyWarsStarStateOwner,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBattleEventKind {
    Assist,
    Servant,
    DummyCharacter,
    CountdownWarning,
    TraitAssist,
}

impl CurrencyWarsBattleEventKind {
    pub(crate) const fn canonical_tag(self) -> u8 {
        match self {
            Self::Assist => 0,
            Self::Servant => 1,
            Self::DummyCharacter => 2,
            Self::CountdownWarning => 3,
            Self::TraitAssist => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBattleEventTeam {
    Player,
    Neutral,
}

impl CurrencyWarsBattleEventTeam {
    pub(crate) const fn canonical_tag(self) -> u8 {
        match self {
            Self::Player => 0,
            Self::Neutral => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CurrencyWarsBattleOverrideEnvironment {
    pub season_id: u16,
    pub lethal_rescue_action_value: ActionValue,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsBattleEventPropertyKind {
    AllDamageTypeAddedRatio,
    AttackAddedRatio,
    AttackDelta,
    BaseAttack,
    BaseDefence,
    BaseHp,
    CriticalChance,
    CriticalDamage,
    FireAddedRatio,
    FirePenetration,
    MaximumEnergy,
    StatusProbability,
}

impl CurrencyWarsBattleEventPropertyKind {
    pub(crate) const fn canonical_tag(self) -> u8 {
        match self {
            Self::AllDamageTypeAddedRatio => 0,
            Self::AttackAddedRatio => 1,
            Self::AttackDelta => 2,
            Self::BaseAttack => 3,
            Self::BaseDefence => 4,
            Self::BaseHp => 5,
            Self::CriticalChance => 6,
            Self::CriticalDamage => 7,
            Self::FireAddedRatio => 8,
            Self::FirePenetration => 9,
            Self::MaximumEnergy => 10,
            Self::StatusProbability => 11,
        }
    }
}

/// Exact authored decimal retained without forcing it into the six-decimal
/// combat domain before a declared operation boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsDecimal {
    significand: i64,
    decimal_places: u8,
}

impl CurrencyWarsDecimal {
    pub const fn new(significand: i64, decimal_places: u8) -> Option<Self> {
        if decimal_places <= 18 {
            Some(Self {
                significand,
                decimal_places,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn significand(self) -> i64 {
        self.significand
    }

    #[must_use]
    pub const fn decimal_places(self) -> u8 {
        self.decimal_places
    }

    pub(crate) fn scalar(self) -> Result<Scalar, CurrencyWarsEmpowermentCatalogError> {
        let numerator = i128::from(self.significand)
            .checked_mul(1_000_000)
            .ok_or_else(|| fail("Currency Wars decimal conversion overflow"))?;
        let denominator = 10_i128
            .checked_pow(u32::from(self.decimal_places))
            .ok_or_else(|| fail("Currency Wars decimal scale overflow"))?;
        round_nearest_ties_even(numerator, denominator)
            .and_then(|value| i64::try_from(value).ok())
            .map(Scalar::from_scaled)
            .ok_or_else(|| fail("Currency Wars decimal is outside Scalar range"))
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsBattleEventProperty {
    pub kind: CurrencyWarsBattleEventPropertyKind,
    pub value: CurrencyWarsDecimal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBackBattleEvent {
    pub event_id: u32,
    pub kind: CurrencyWarsBattleEventKind,
    pub team: CurrencyWarsBattleEventTeam,
    pub abilities: Box<[Box<str>]>,
    pub speed: Option<CurrencyWarsDecimal>,
    pub hard_level: bool,
    pub values: Box<[CurrencyWarsDecimal]>,
    pub properties: Box<[CurrencyWarsBattleEventProperty]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsSpecialResourceKind {
    EnergyBar,
    MaximumEnergy,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsFrontSpecialResource {
    pub role: CurrencyWarsRoleId,
    pub star: u8,
    pub kind: CurrencyWarsSpecialResourceKind,
    pub maximum: CurrencyWarsDecimal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRoleGlobalModifier {
    pub role: CurrencyWarsRoleId,
    pub saved_value: Option<Box<str>>,
    pub values: Box<[CurrencyWarsDecimal]>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsSkillParameterOperator {
    Add,
    Multiply,
    Set,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CurrencyWarsSkillParameterEdit {
    pub index: u8,
    pub operator: CurrencyWarsSkillParameterOperator,
    pub value: CurrencyWarsDecimal,
}

impl CurrencyWarsSkillParameterEdit {
    /// Applies the exact authored edit at the combat Scalar boundary. Values
    /// with more than six decimals use nearest-ties-even explicitly.
    pub fn apply(self, current: Scalar) -> Result<Scalar, CurrencyWarsEmpowermentCatalogError> {
        match self.operator {
            CurrencyWarsSkillParameterOperator::Add => current
                .checked_add(self.value.scalar()?)
                .map_err(|_| fail("Currency Wars parameter addition overflow")),
            CurrencyWarsSkillParameterOperator::Multiply => current
                .checked_mul(self.value.scalar()?, Rounding::NearestTiesEven)
                .map_err(|_| fail("Currency Wars parameter multiplication overflow")),
            CurrencyWarsSkillParameterOperator::Set => self.value.scalar(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsRankSkillOverride {
    pub rank_id: u32,
    pub skill_id: u32,
    pub edits: Box<[CurrencyWarsSkillParameterEdit]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsCyreneSkillOverride {
    pub provider_role: CurrencyWarsRoleId,
    pub role: CurrencyWarsRoleId,
    pub skill_id: u32,
    pub multiple_value_key: Box<str>,
    pub edits: Box<[CurrencyWarsSkillParameterEdit]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsSummonBattleEventOverride {
    pub season_id: u16,
    pub battle_event_id: u32,
    pub front_config: Option<Box<str>>,
    pub back_config: Option<Box<str>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CurrencyWarsLethalRescueHpPolicy {
    FullMaximumHp,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrencyWarsBattleOverrideDefinition {
    AutomaticTechnique {
        eligible_position: CurrencyWarsPositionKind,
    },
    DefeatEnergyScaling {
        regular_energy_ratio: Ratio,
    },
    LethalDamageRescue {
        hp_policy: CurrencyWarsLethalRescueHpPolicy,
    },
    BackBattleEvent(CurrencyWarsBackBattleEvent),
    FrontSpecialResource(CurrencyWarsFrontSpecialResource),
    RoleGlobalModifier(CurrencyWarsRoleGlobalModifier),
    RankSkillOverride(CurrencyWarsRankSkillOverride),
    SummonBattleEventOverride(CurrencyWarsSummonBattleEventOverride),
    CyreneSkillOverride(CurrencyWarsCyreneSkillOverride),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleOverride {
    pub stable_key: Box<str>,
    pub source_id: Box<str>,
    pub definition: CurrencyWarsBattleOverrideDefinition,
    pub teardown: Box<str>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleOverrideRoleBuild {
    pub role: CurrencyWarsRoleId,
    pub technique_ability: AbilityId,
    pub eidolon: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsAutomaticTechnique {
    pub position: CurrencyWarsPosition,
    pub role_state: CurrencyWarsRoleState,
    pub ability: AbilityId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsActiveSpecialResource {
    pub position: CurrencyWarsPosition,
    pub role_state: CurrencyWarsRoleState,
    pub kind: CurrencyWarsSpecialResourceKind,
    pub maximum: CurrencyWarsDecimal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrencyWarsBattleOverrideSnapshot {
    pub automatic_techniques: Box<[CurrencyWarsAutomaticTechnique]>,
    pub defeat_energy_ratio: Ratio,
    pub lethal_rescue_hp_policy: CurrencyWarsLethalRescueHpPolicy,
    pub lethal_rescue_action_value: ActionValue,
    pub back_battle_events: Box<[CurrencyWarsBackBattleEvent]>,
    pub external_battle_event_ids: Box<[u32]>,
    pub special_resources: Box<[CurrencyWarsActiveSpecialResource]>,
    pub role_global_modifiers: Box<[CurrencyWarsRoleGlobalModifier]>,
    pub rank_skill_overrides: Box<[CurrencyWarsRankSkillOverride]>,
    pub summon_battle_event_overrides: Box<[CurrencyWarsSummonBattleEventOverride]>,
    pub cyrene_skill_overrides: Box<[CurrencyWarsCyreneSkillOverride]>,
}

impl CurrencyWarsBattleOverrideSnapshot {
    pub fn scale_defeat_energy(
        &self,
        regular: Energy,
    ) -> Result<Energy, CurrencyWarsEmpowermentCatalogError> {
        self.defeat_energy_ratio
            .checked_apply(Scalar::from_scaled(regular.scaled()), Rounding::Floor)
            .and_then(|value| Energy::from_scaled(value.scaled()))
            .map_err(|_| fail("Currency Wars defeat Energy scaling failed"))
    }

    pub fn resolve_lethal_damage(
        &self,
        maximum_hp: Hp,
        remaining_action_value: ActionValue,
    ) -> Result<CurrencyWarsLethalRescueResolution, CurrencyWarsEmpowermentCatalogError> {
        let deducted = remaining_action_value
            .scaled()
            .min(self.lethal_rescue_action_value.scaled());
        let remaining = remaining_action_value
            .scaled()
            .checked_sub(deducted)
            .and_then(|value| ActionValue::from_scaled(value).ok())
            .ok_or_else(|| fail("Currency Wars lethal-rescue countdown deduction failed"))?;
        let restored_hp = match self.lethal_rescue_hp_policy {
            CurrencyWarsLethalRescueHpPolicy::FullMaximumHp => maximum_hp,
        };
        Ok(CurrencyWarsLethalRescueResolution {
            restored_hp,
            deducted_action_value: ActionValue::from_scaled(deducted)
                .map_err(|_| fail("Currency Wars lethal-rescue deduction is invalid"))?,
            remaining_action_value: remaining,
            countdown_expired: remaining == ActionValue::ZERO,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CurrencyWarsLethalRescueResolution {
    pub restored_hp: Hp,
    pub deducted_action_value: ActionValue,
    pub remaining_action_value: ActionValue,
    pub countdown_expired: bool,
}

pub(crate) fn validate_battle_overrides(
    definitions: &[CurrencyWarsBattleOverride],
) -> Result<(), CurrencyWarsEmpowermentCatalogError> {
    let automatic = definitions
        .iter()
        .filter(|value| {
            matches!(
                value.definition,
                CurrencyWarsBattleOverrideDefinition::AutomaticTechnique { .. }
            )
        })
        .count();
    let energy = definitions
        .iter()
        .filter(|value| {
            matches!(
                value.definition,
                CurrencyWarsBattleOverrideDefinition::DefeatEnergyScaling { .. }
            )
        })
        .count();
    let rescue = definitions
        .iter()
        .filter(|value| {
            matches!(
                value.definition,
                CurrencyWarsBattleOverrideDefinition::LethalDamageRescue { .. }
            )
        })
        .count();
    if automatic != 1 || energy != 1 || rescue != 1 {
        return Err(fail("Currency Wars global battle overrides are incomplete"));
    }
    Ok(())
}

pub(crate) fn resolve_battle_overrides(
    definitions: &[CurrencyWarsBattleOverride],
    deployment: &CurrencyWarsDeployment,
    roles: &[CurrencyWarsRole],
    builds: &[CurrencyWarsBattleOverrideRoleBuild],
    battle_event_ids: &[u32],
    star_states: &[CurrencyWarsStarState],
    environment: CurrencyWarsBattleOverrideEnvironment,
) -> Result<CurrencyWarsBattleOverrideSnapshot, CurrencyWarsEmpowermentCatalogError> {
    let build_by_role = builds
        .iter()
        .map(|build| (build.role, build))
        .collect::<BTreeMap<_, _>>();
    let deployed = deployment
        .positions()
        .values()
        .map(|state| state.role())
        .collect::<BTreeSet<_>>();
    let mut active_ranks = BTreeSet::new();
    for role_id in &deployed {
        let role = roles
            .binary_search_by_key(role_id, |role| role.id)
            .ok()
            .map(|index| &roles[index])
            .ok_or_else(|| fail("Currency Wars battle-override role is missing"))?;
        let build = build_by_role
            .get(role_id)
            .ok_or_else(|| fail("Currency Wars battle-override Build is missing"))?;
        if build.eidolon > 6 {
            return Err(fail("Currency Wars battle-override Eidolon is invalid"));
        }
        active_ranks.extend(
            role.backend_rank_ids
                .iter()
                .take(usize::from(build.eidolon))
                .copied(),
        );
    }

    let automatic_position = definitions
        .iter()
        .find_map(|value| match value.definition {
            CurrencyWarsBattleOverrideDefinition::AutomaticTechnique { eligible_position } => {
                Some(eligible_position)
            }
            _ => None,
        })
        .ok_or_else(|| fail("Currency Wars automatic Technique rule is missing"))?;
    let automatic_techniques = deployment
        .positions()
        .iter()
        .filter(|(position, _)| position.kind() == automatic_position)
        .map(|(position, state)| {
            let build = build_by_role
                .get(&state.role())
                .ok_or_else(|| fail("Currency Wars automatic Technique Build is missing"))?;
            Ok(CurrencyWarsAutomaticTechnique {
                position: *position,
                role_state: *state,
                ability: build.technique_ability,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    let defeat_energy_ratio = definitions
        .iter()
        .find_map(|value| match value.definition {
            CurrencyWarsBattleOverrideDefinition::DefeatEnergyScaling {
                regular_energy_ratio,
            } => Some(regular_energy_ratio),
            _ => None,
        })
        .ok_or_else(|| fail("Currency Wars defeat Energy rule is missing"))?;
    let lethal_rescue_hp_policy = definitions
        .iter()
        .find_map(|value| match value.definition {
            CurrencyWarsBattleOverrideDefinition::LethalDamageRescue { hp_policy } => {
                Some(hp_policy)
            }
            _ => None,
        })
        .ok_or_else(|| fail("Currency Wars lethal-rescue rule is missing"))?;

    let selected_events = deployment
        .positions()
        .values()
        .filter_map(|state| {
            let owner = CurrencyWarsStarStateOwner::Role(state.role());
            star_states
                .binary_search_by_key(&(owner, state.star()), |value| (value.owner, value.star))
                .ok()
                .and_then(|index| star_states[index].battle_event_id)
        })
        .chain(battle_event_ids.iter().copied())
        .collect::<BTreeSet<_>>();
    let back_battle_events = definitions
        .iter()
        .filter_map(|value| match &value.definition {
            CurrencyWarsBattleOverrideDefinition::BackBattleEvent(event)
                if selected_events.contains(&event.event_id) =>
            {
                Some(event.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let resolved_events = back_battle_events
        .iter()
        .map(|event| event.event_id)
        .collect::<BTreeSet<_>>();
    let external_battle_event_ids = selected_events
        .difference(&resolved_events)
        .copied()
        .collect::<Vec<_>>();
    let special_resources = deployment
        .positions()
        .iter()
        .filter_map(|(position, state)| {
            (position.kind() == CurrencyWarsPositionKind::Front)
                .then(|| {
                    definitions.iter().find_map(|value| match value.definition {
                        CurrencyWarsBattleOverrideDefinition::FrontSpecialResource(resource)
                            if resource.role == state.role() && resource.star == state.star() =>
                        {
                            Some(CurrencyWarsActiveSpecialResource {
                                position: *position,
                                role_state: *state,
                                kind: resource.kind,
                                maximum: resource.maximum,
                            })
                        }
                        _ => None,
                    })
                })
                .flatten()
        })
        .collect::<Vec<_>>();
    let role_global_modifiers = definitions
        .iter()
        .filter_map(|value| match &value.definition {
            CurrencyWarsBattleOverrideDefinition::RoleGlobalModifier(modifier)
                if deployed.contains(&modifier.role) =>
            {
                Some(modifier.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let rank_skill_overrides = definitions
        .iter()
        .filter_map(|value| match &value.definition {
            CurrencyWarsBattleOverrideDefinition::RankSkillOverride(override_)
                if active_ranks.contains(&override_.rank_id) =>
            {
                Some(override_.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let summon_battle_event_overrides = definitions
        .iter()
        .filter_map(|value| match &value.definition {
            CurrencyWarsBattleOverrideDefinition::SummonBattleEventOverride(override_)
                if override_.season_id == environment.season_id =>
            {
                Some(override_.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    let cyrene_skill_overrides = definitions
        .iter()
        .filter_map(|value| match &value.definition {
            CurrencyWarsBattleOverrideDefinition::CyreneSkillOverride(override_)
                if deployed.contains(&override_.provider_role)
                    && deployed.contains(&override_.role) =>
            {
                Some(override_.clone())
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    Ok(CurrencyWarsBattleOverrideSnapshot {
        automatic_techniques: automatic_techniques.into_boxed_slice(),
        defeat_energy_ratio,
        lethal_rescue_hp_policy,
        lethal_rescue_action_value: environment.lethal_rescue_action_value,
        back_battle_events: back_battle_events.into_boxed_slice(),
        external_battle_event_ids: external_battle_event_ids.into_boxed_slice(),
        special_resources: special_resources.into_boxed_slice(),
        role_global_modifiers: role_global_modifiers.into_boxed_slice(),
        rank_skill_overrides: rank_skill_overrides.into_boxed_slice(),
        summon_battle_event_overrides: summon_battle_event_overrides.into_boxed_slice(),
        cyrene_skill_overrides: cyrene_skill_overrides.into_boxed_slice(),
    })
}

fn round_nearest_ties_even(numerator: i128, denominator: i128) -> Option<i128> {
    if denominator <= 0 {
        return None;
    }
    let sign = if numerator < 0 { -1 } else { 1 };
    let absolute = numerator.checked_abs()?;
    let quotient = absolute.checked_div(denominator)?;
    let remainder = absolute.checked_rem(denominator)?;
    let doubled = remainder.checked_mul(2)?;
    let rounded = if doubled > denominator || (doubled == denominator && quotient % 2 != 0) {
        quotient.checked_add(1)?
    } else {
        quotient
    };
    rounded.checked_mul(sign)
}

fn fail(message: &'static str) -> CurrencyWarsEmpowermentCatalogError {
    CurrencyWarsEmpowermentCatalogError::new(message)
}
