//! Selected enemy-affix contributions compiled before immutable battle creation.

mod rule;
mod static_modifier;

use starclock_combat::{Ratio, Rounding, formula::toughness::EnemyRank};

use crate::{
    CurrencyWarsContributionSnapshot, CurrencyWarsEnemyAffixSemantic,
    battle_assembly::{CurrencyWarsBattleAssemblyError, debug_error, error},
};

pub(super) use rule::install_reactions;
pub(super) use static_modifier::{EnemyAffixOverlays, install_static_modifiers};

pub(super) struct EnemyStatMultipliers {
    pub hp: Ratio,
    pub speed: Ratio,
}

pub(super) fn enemy_stat_multipliers(
    snapshot: &CurrencyWarsContributionSnapshot,
    rank: EnemyRank,
) -> Result<EnemyStatMultipliers, CurrencyWarsBattleAssemblyError> {
    let mut hp = Ratio::ONE;
    let mut speed = Ratio::ONE;
    for affix in &snapshot.enemy_affix_behaviors {
        if !stat_affix_applies(affix.semantic, snapshot.node.plane, rank) {
            continue;
        }
        let [speed_increase, hp_increase] = affix.parameters.as_ref() else {
            return Err(error(
                "Currency Wars enemy stat Affix has invalid parameters",
            ));
        };
        speed = speed
            .checked_mul(
                Ratio::ONE
                    .checked_add(Ratio::from_scaled(speed_increase.scaled()))
                    .map_err(debug_error)?,
                Rounding::NearestTiesAway,
            )
            .map_err(debug_error)?;
        hp = hp
            .checked_mul(
                Ratio::ONE
                    .checked_add(Ratio::from_scaled(hp_increase.scaled()))
                    .map_err(debug_error)?,
                Rounding::NearestTiesAway,
            )
            .map_err(debug_error)?;
    }
    Ok(EnemyStatMultipliers { hp, speed })
}

const fn stat_affix_applies(
    semantic: CurrencyWarsEnemyAffixSemantic,
    plane: u8,
    rank: EnemyRank,
) -> bool {
    match semantic {
        CurrencyWarsEnemyAffixSemantic::BossEnhancement => matches!(rank, EnemyRank::Boss),
        CurrencyWarsEnemyAffixSemantic::FollowerEnhancement => !matches!(rank, EnemyRank::Boss),
        CurrencyWarsEnemyAffixSemantic::FirstPlaneEnhancement => plane == 1,
        CurrencyWarsEnemyAffixSemantic::SecondPlaneEnhancement => plane == 2,
        CurrencyWarsEnemyAffixSemantic::ThirdPlaneEnhancement => plane == 3,
        _ => false,
    }
}
