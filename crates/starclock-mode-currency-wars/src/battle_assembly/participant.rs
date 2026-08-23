//! Currency Wars participant materialization and enemy stat scaling.

use std::collections::BTreeMap;

use starclock_combat::{
    FormationIndex, ParticipantSource, ParticipantSpec, Ratio, ResolvedCombatantSpec, Rounding,
    TeamSide, UnitLevel,
};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsEncounterCatalog, CurrencyWarsEnemyScaling,
    CurrencyWarsEnemySlot, CurrencyWarsPositionKind, CurrencyWarsReleasedStage,
    CurrencyWarsReleasedStageWave, CurrencyWarsRoleId,
};

use super::{
    CurrencyWarsBattleAssemblyError, CurrencyWarsBattleResources,
    affix::{EnemyAffixOverlays, enemy_stat_multipliers},
    combatant_digest, debug_error, error, monster_identity,
    scaling::scale_enemy,
};

pub(super) fn time_assassin_formation(
    wave: &CurrencyWarsReleasedStageWave,
) -> Result<FormationIndex, CurrencyWarsBattleAssemblyError> {
    (0_u8..=31)
        .find(|candidate| {
            wave.enemies
                .iter()
                .all(|enemy| enemy.formation != *candidate)
        })
        .and_then(FormationIndex::new)
        .ok_or_else(|| error("Currency Wars Time Assassin has no free formation slot"))
}

pub(super) fn player_participants(
    snapshot: &CurrencyWarsContributionSnapshot,
    combatants: &BTreeMap<CurrencyWarsRoleId, ResolvedCombatantSpec>,
) -> Result<Vec<ParticipantSpec>, CurrencyWarsBattleAssemblyError> {
    let players = snapshot
        .roles
        .iter()
        .filter(|role| role.position.kind() == CurrencyWarsPositionKind::Front)
        .map(|role| {
            let combatant = combatants
                .get(&role.role.id)
                .ok_or_else(|| error("Currency Wars player combatant is missing"))?;
            Ok(ParticipantSpec::new(
                TeamSide::Player,
                FormationIndex::new(role.position.index().saturating_sub(1))
                    .ok_or_else(|| error("Currency Wars player formation is invalid"))?,
                ParticipantSource::Player,
                combatant.clone(),
            )
            .with_locked_combatant_digest(role.combatant.digest()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if players.is_empty() {
        return Err(error("Currency Wars battle has no front-row participant"));
    }
    Ok(players)
}

pub(super) struct CurrencyWarsEnemyParticipantInputs<'a> {
    pub(super) resources: &'a CurrencyWarsBattleResources,
    pub(super) encounters: &'a CurrencyWarsEncounterCatalog,
    pub(super) snapshot: &'a CurrencyWarsContributionSnapshot,
    pub(super) stage: &'a CurrencyWarsReleasedStage,
    pub(super) selected_enemies: &'a [&'a CurrencyWarsEnemySlot],
    pub(super) time_assassin: Option<&'a CurrencyWarsEnemySlot>,
    pub(super) level: UnitLevel,
    pub(super) scaling: CurrencyWarsEnemyScaling,
    pub(super) enemy_star: u8,
    pub(super) root_digest: [u8; 32],
    pub(super) affix_overlays: &'a EnemyAffixOverlays,
}

pub(super) fn enemy_participants(
    inputs: CurrencyWarsEnemyParticipantInputs<'_>,
) -> Result<Vec<ParticipantSpec>, CurrencyWarsBattleAssemblyError> {
    let CurrencyWarsEnemyParticipantInputs {
        resources,
        encounters,
        snapshot,
        stage,
        selected_enemies,
        time_assassin,
        level,
        scaling,
        enemy_star,
        root_digest,
        affix_overlays,
    } = inputs;
    let hp_ratio = source_base_ratio(snapshot.enemy_battle_base.hp)?
        .checked_mul(
            Ratio::from_scaled(
                snapshot
                    .difficulty
                    .enemy_scaling
                    .level_base_hp_ratio
                    .scaled(),
            ),
            Rounding::NearestTiesAway,
        )
        .map_err(debug_error)?
        .checked_mul(scaling.hp_ratio, Rounding::NearestTiesAway)
        .map_err(debug_error)?;
    let attack_ratio = source_base_ratio(snapshot.enemy_battle_base.attack)?
        .checked_mul(
            Ratio::from_scaled(
                snapshot
                    .difficulty
                    .enemy_scaling
                    .level_base_attack_ratio
                    .scaled(),
            ),
            Rounding::NearestTiesAway,
        )
        .map_err(debug_error)?
        .checked_mul(scaling.attack_ratio, Rounding::NearestTiesAway)
        .map_err(debug_error)?;
    let mut participants = Vec::new();
    let mut push_enemy = |wave_index: usize,
                          slot_index: usize,
                          formation: FormationIndex,
                          selected: &CurrencyWarsEnemySlot|
     -> Result<(), CurrencyWarsBattleAssemblyError> {
        let (source_monster_id, stable_key) = monster_identity(selected)?;
        let input = resources
            .enemy(stable_key, level)
            .ok_or_else(|| error("Currency Wars selected enemy input is missing"))?;
        let star = encounters
            .enemy_star_scaling(source_monster_id, enemy_star)
            .ok_or_else(|| error("Currency Wars enemy star scaling is missing"))?;
        let affix = enemy_stat_multipliers(snapshot, input.combatant.rank())?;
        let enemy_hp_ratio = hp_ratio
            .checked_mul(star.hp, Rounding::NearestTiesAway)
            .and_then(|ratio| ratio.checked_mul(affix.hp, Rounding::NearestTiesAway))
            .map_err(debug_error)?;
        let enemy_attack_ratio = attack_ratio
            .checked_mul(star.attack, Rounding::NearestTiesAway)
            .map_err(debug_error)?;
        let effective_scaling = CurrencyWarsEnemyScaling {
            chapter: scaling.chapter,
            difficulty_level: scaling.difficulty_level,
            hp_ratio: scaling.hp_ratio,
            attack_ratio: scaling.attack_ratio,
            defense_ratio: scaling
                .defense_ratio
                .checked_mul(star.defense, Rounding::NearestTiesAway)
                .map_err(debug_error)?,
            speed_ratio: scaling
                .speed_ratio
                .checked_mul(star.speed, Rounding::NearestTiesAway)
                .and_then(|ratio| ratio.checked_mul(affix.speed, Rounding::NearestTiesAway))
                .map_err(debug_error)?,
            stance_ratio: scaling
                .stance_ratio
                .checked_mul(star.stance, Rounding::NearestTiesAway)
                .map_err(debug_error)?,
        };
        let combatant = scale_enemy(
            &input.combatant,
            enemy_hp_ratio,
            enemy_attack_ratio,
            effective_scaling,
            combatant_digest(root_digest, wave_index, slot_index),
        )?;
        let combatant = affix_overlays.apply_enemy(combatant)?;
        participants.push(
            ParticipantSpec::new(
                TeamSide::Enemy,
                formation,
                ParticipantSource::EncounterEnemy(input.definition),
                combatant,
            )
            .with_wave(u16::try_from(wave_index + 1).map_err(debug_error)?)
            .ok_or_else(|| error("Currency Wars enemy wave assignment is invalid"))?,
        );
        Ok(())
    };
    let mut selected_enemies = selected_enemies.iter();
    for (wave_index, wave) in stage.waves.iter().enumerate() {
        for (slot_index, enemy) in wave.enemies.iter().enumerate() {
            let selected = selected_enemies
                .next()
                .ok_or_else(|| error("Currency Wars selected enemy roster is truncated"))?;
            push_enemy(
                wave_index,
                slot_index,
                FormationIndex::new(enemy.formation)
                    .ok_or_else(|| error("Currency Wars enemy formation is invalid"))?,
                selected,
            )?;
        }
    }
    if let Some(selected) = time_assassin {
        push_enemy(
            0,
            stage.waves[0].enemies.len(),
            time_assassin_formation(&stage.waves[0])?,
            selected,
        )?;
    }
    Ok(participants)
}

fn source_base_ratio(value: u32) -> Result<Ratio, CurrencyWarsBattleAssemblyError> {
    let scaled = i64::from(value)
        .checked_mul(100)
        .ok_or_else(|| error("Currency Wars shared battle base overflows"))?;
    Ok(Ratio::from_scaled(scaled))
}
