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
const ENEMY_PREFIX: &str = "enemy.";
const DEFEATED_ENEMY_BLESSING_REWARD: &str =
    "universe.occurrence-battle.reward.defeated-enemy-blessing";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceBattleReward {
    BlessingPerDefeatedEnemy,
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
        let enemies = outcome
            .parameter_refs()
            .iter()
            .filter(|value| value.starts_with(ENEMY_PREFIX))
            .map(AsRef::as_ref)
            .collect::<Vec<_>>();
        let reward = outcome
            .parameter_refs()
            .iter()
            .any(|value| value.as_ref() == DEFEATED_ENEMY_BLESSING_REWARD)
            .then_some(OccurrenceBattleReward::BlessingPerDefeatedEnemy);
        if stage.is_none() && enemies.is_empty() && reward.is_none() {
            continue;
        }
        let stage = stage.ok_or(OccurrenceBattleCompileError::IncompleteBridge)?;
        let reward = reward.ok_or(OccurrenceBattleCompileError::IncompleteBridge)?;
        if enemies.is_empty() || enemies.len() > 4 {
            return Err(OccurrenceBattleCompileError::InvalidEnemyCount);
        }
        let level = outcome
            .numeric_literals()
            .first()
            .copied()
            .filter(|value| value.unit() == AuthoredScalarUnit::Scalar)
            .map(exact_integer)
            .transpose()
            .map_err(|_| OccurrenceBattleCompileError::InvalidLevel)?
            .and_then(|value| u32::try_from(value).ok())
            .filter(|value| (1..=80).contains(value))
            .ok_or(OccurrenceBattleCompileError::InvalidLevel)?;
        let member_id = EncounterMemberId::new(
            MEMBER_ID_BASE
                .checked_add(choice.id().get())
                .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?,
        )
        .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?;
        let wave_id = EncounterWaveId::new(
            WAVE_ID_BASE
                .checked_add(choice.id().get())
                .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?,
        )
        .ok_or(OccurrenceBattleCompileError::IdentityOverflow)?;
        let slots = enemies
            .iter()
            .enumerate()
            .map(|(index, enemy)| {
                EncounterEnemySlot::new(
                    &format!("occurrence-battle-slot-{}", index + 1),
                    enemy,
                    enemy,
                )
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let member = EncounterMemberDefinition::new(
            member_id,
            choice.stable_key(),
            enemies[0],
            stage,
            ExactParameter::new(1, 0),
            level,
            2,
            Box::new([]),
            None,
            vec![EncounterWaveDefinition::new(wave_id, slots)].into_boxed_slice(),
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OccurrenceBattleCompileError {
    DuplicateChoice,
    IdentityOverflow,
    IncompleteBridge,
    InvalidEnemyCount,
    InvalidLevel,
}
