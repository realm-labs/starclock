//! Generic authored Occurrence battle bridges.
//!
//! Battle rows remain part of the Occurrence catalog. A bridge is materialized
//! only when the row carries a complete stage marker, exact enemy references
//! and an explicit post-battle reward policy marker.

use crate::{
    catalog::UniverseCatalog,
    encounter::{EncounterEnemySlot, EncounterMemberDefinition, EncounterWaveDefinition},
    id::{EncounterMemberId, EncounterWaveId, OccurrenceChoiceId},
    occurrence::{AuthoredScalarUnit, OccurrenceOperation},
    occurrence_interaction::support::exact_integer,
    path::ExactParameter,
};

const MEMBER_ID_BASE: u32 = 10_000;
const WAVE_ID_BASE: u32 = 20_000;
const STAGE_PREFIX: &str = "universe.occurrence-battle.stage.";
const LEVEL_PREFIX: &str = "universe.occurrence-battle.level.";
const WAVE_PREFIX: &str = "universe.occurrence-battle.wave.";
const ENEMY_PREFIX: &str = "enemy.";
const DEFEATED_ENEMY_BLESSING_REWARD: &str =
    "universe.occurrence-battle.reward.defeated-enemy-blessing";
const FIXED_BLESSING_REWARD_PREFIX: &str = "universe.occurrence-battle.reward.fixed-blessings.";
const CYCLE_BLESSING_REWARD_PREFIX: &str = "universe.occurrence-battle.reward.within-cycles.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceBattleReward {
    BlessingPerDefeatedEnemy,
    FixedBlessings(u8),
    BlessingsWithinCycles { cycles: u8, base: u8, bonus: u8 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OccurrenceBattleDefinition {
    choice: OccurrenceChoiceId,
    member: EncounterMemberDefinition,
    reward: OccurrenceBattleReward,
}

impl OccurrenceBattleDefinition {
    pub(crate) const fn choice(&self) -> OccurrenceChoiceId {
        self.choice
    }

    pub(crate) const fn member(&self) -> &EncounterMemberDefinition {
        &self.member
    }

    pub(crate) const fn reward(&self) -> OccurrenceBattleReward {
        self.reward
    }
}

pub(crate) fn compile(
    catalog: &UniverseCatalog,
) -> Result<Vec<OccurrenceBattleDefinition>, OccurrenceBattleCompileError> {
    let mut battles = Vec::new();
    for choice in catalog.occurrence_choices() {
        let Some(outcome) = choice
            .outcomes()
            .iter()
            .find(|outcome| outcome.operations().contains(&OccurrenceOperation::Battle))
        else {
            continue;
        };
        let stage = outcome
            .parameter_refs()
            .iter()
            .find_map(|value| value.strip_prefix(STAGE_PREFIX));
        let waves = parse_waves(outcome.parameter_refs())?;
        let reward = parse_reward(outcome.parameter_refs())?;
        if stage.is_none() && waves.is_empty() && reward.is_none() {
            continue;
        }
        let stage = stage.ok_or(OccurrenceBattleCompileError::IncompleteBridge)?;
        let reward = reward.ok_or(OccurrenceBattleCompileError::IncompleteBridge)?;
        let authored_level = outcome.parameter_refs().iter().find_map(|reference| {
            reference
                .strip_prefix(LEVEL_PREFIX)
                .and_then(|value| value.parse::<u32>().ok())
        });
        let legacy_level = outcome
            .numeric_literals()
            .first()
            .copied()
            .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
            .map(exact_integer)
            .transpose()
            .map_err(|_| OccurrenceBattleCompileError::InvalidLevel)?
            .and_then(|value| u32::try_from(value).ok());
        let level = authored_level
            .or(legacy_level)
            .filter(|value| (1..=80).contains(value))
            .ok_or(OccurrenceBattleCompileError::InvalidLevel)?;
        let member_id = EncounterMemberId::new(
            MEMBER_ID_BASE
                .checked_add(choice.id().get())
                .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?,
        )
        .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?;
        let first_enemy = waves
            .first()
            .and_then(|wave| wave.first())
            .copied()
            .ok_or(OccurrenceBattleCompileError::InvalidEnemyCount)?;
        let waves = waves
            .iter()
            .enumerate()
            .map(|(wave_index, enemies)| {
                let wave_id = wave_id(choice.id(), wave_index)?;
                let slots = enemies
                    .iter()
                    .enumerate()
                    .map(|(index, enemy)| {
                        EncounterEnemySlot::new(
                            &format!(
                                "occurrence-battle-wave-{}-slot-{}",
                                wave_index + 1,
                                index + 1
                            ),
                            enemy,
                            enemy,
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                Ok(EncounterWaveDefinition::new(wave_id, slots))
            })
            .collect::<Result<Vec<_>, OccurrenceBattleCompileError>>()?
            .into_boxed_slice();
        let member = EncounterMemberDefinition::new(
            member_id,
            choice.stable_key(),
            first_enemy,
            stage,
            ExactParameter::new(1, 0),
            level,
            2,
            Box::new([]),
            None,
            waves,
        );
        battles.push(OccurrenceBattleDefinition {
            choice: choice.id(),
            member,
            reward,
        });
    }
    battles.sort_unstable_by_key(OccurrenceBattleDefinition::choice);
    if battles
        .windows(2)
        .any(|pair| pair[0].choice == pair[1].choice)
    {
        return Err(OccurrenceBattleCompileError::DuplicateChoice);
    }
    Ok(battles)
}

fn parse_waves(references: &[Box<str>]) -> Result<Vec<Vec<&str>>, OccurrenceBattleCompileError> {
    let has_markers = references
        .iter()
        .any(|value| value.starts_with(WAVE_PREFIX));
    let mut waves = Vec::<Vec<&str>>::new();
    if !has_markers {
        let enemies = references
            .iter()
            .filter(|value| value.starts_with(ENEMY_PREFIX))
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        if !enemies.is_empty() {
            waves.push(enemies);
        }
    } else {
        for reference in references {
            if reference.starts_with(WAVE_PREFIX) {
                waves.push(Vec::new());
            } else if reference.starts_with(ENEMY_PREFIX) {
                waves
                    .last_mut()
                    .ok_or(OccurrenceBattleCompileError::IncompleteBridge)?
                    .push(reference);
            }
        }
    }
    if waves.iter().any(|wave| wave.is_empty() || wave.len() > 4) || waves.len() > 8 {
        return Err(OccurrenceBattleCompileError::InvalidEnemyCount);
    }
    Ok(waves)
}

fn parse_reward(
    references: &[Box<str>],
) -> Result<Option<OccurrenceBattleReward>, OccurrenceBattleCompileError> {
    let rewards = references
        .iter()
        .filter_map(|reference| {
            if reference.as_ref() == DEFEATED_ENEMY_BLESSING_REWARD {
                return Some(Some(OccurrenceBattleReward::BlessingPerDefeatedEnemy));
            }
            if let Some(value) = reference.strip_prefix(FIXED_BLESSING_REWARD_PREFIX) {
                return Some(
                    value
                        .parse::<u8>()
                        .ok()
                        .filter(|value| (1..=4).contains(value))
                        .map(OccurrenceBattleReward::FixedBlessings),
                );
            }
            reference
                .strip_prefix(CYCLE_BLESSING_REWARD_PREFIX)
                .map(parse_cycle_reward)
        })
        .collect::<Vec<_>>();
    match rewards.as_slice() {
        [] => Ok(None),
        [Some(reward)] => Ok(Some(*reward)),
        _ => Err(OccurrenceBattleCompileError::InvalidReward),
    }
}

fn parse_cycle_reward(value: &str) -> Option<OccurrenceBattleReward> {
    let mut parts = value.split('.');
    let cycles = parts.next()?.parse::<u8>().ok()?;
    (parts.next()? == "base").then_some(())?;
    let base = parts.next()?.parse::<u8>().ok()?;
    (parts.next()? == "bonus").then_some(())?;
    let bonus = parts.next()?.parse::<u8>().ok()?;
    (parts.next().is_none()
        && (1..=8).contains(&cycles)
        && (1..=4).contains(&base)
        && (1..=4).contains(&bonus)
        && base.checked_add(bonus)? <= 4)
        .then_some(OccurrenceBattleReward::BlessingsWithinCycles {
            cycles,
            base,
            bonus,
        })
}

fn wave_id(
    choice: OccurrenceChoiceId,
    index: usize,
) -> Result<EncounterWaveId, OccurrenceBattleCompileError> {
    let offset = u32::try_from(index)
        .ok()
        .and_then(|value| value.checked_mul(1_000_000))
        .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?;
    EncounterWaveId::new(
        WAVE_ID_BASE
            .checked_add(choice.get())
            .and_then(|value| value.checked_add(offset))
            .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?,
    )
    .ok_or(OccurrenceBattleCompileError::IdentityOverflow)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceBattleCompileError {
    DuplicateChoice,
    IdentityOverflow,
    IncompleteBridge,
    InvalidEnemyCount,
    InvalidLevel,
    InvalidReward,
}
