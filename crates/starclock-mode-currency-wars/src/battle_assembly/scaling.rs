//! Exact enemy scaling at the immutable battle-input boundary.

use starclock_combat::{
    CombatantSpecDigest, Hp, Ratio, ResolvedCombatantSpec, ResolvedDefinitionBindings, Rounding,
    Scalar, Speed, ToughnessLayerSpec,
};

use crate::{CurrencyWarsEnemyScaling, battle_assembly::CurrencyWarsBattleAssemblyError};

use super::debug_error;

pub(super) fn scale_enemy(
    base: &ResolvedCombatantSpec,
    hp_ratio: Ratio,
    attack_ratio: Ratio,
    scaling: CurrencyWarsEnemyScaling,
    digest: [u8; 32],
) -> Result<ResolvedCombatantSpec, CurrencyWarsBattleAssemblyError> {
    let hp = hp_ratio
        .checked_apply(
            Scalar::checked_from_integer(base.maximum_hp().get()).map_err(debug_error)?,
            Rounding::NearestTiesAway,
        )
        .and_then(|value| Hp::from_scalar(value, Rounding::NearestTiesAway))
        .map_err(debug_error)?;
    let speed = scaling
        .speed_ratio
        .checked_apply(
            Scalar::from_scaled(base.speed().scaled()),
            Rounding::NearestTiesAway,
        )
        .and_then(|value| Speed::from_scaled(value.scaled()))
        .map_err(debug_error)?;
    let toughness = base
        .toughness_layers()
        .iter()
        .map(|layer| scale_toughness(layer, scaling.stance_ratio))
        .collect::<Result<Vec<_>, _>>()?;
    let attack = base
        .base_attack()
        .checked_scale(attack_ratio, Rounding::NearestTiesAway)
        .map_err(debug_error)?;
    let defense = base
        .base_defense()
        .checked_scale(scaling.defense_ratio, Rounding::NearestTiesAway)
        .map_err(debug_error)?;
    ResolvedCombatantSpec::new(
        base.form(),
        base.level(),
        hp,
        speed,
        ResolvedDefinitionBindings::new(
            base.abilities().to_vec(),
            base.rule_bundles().to_vec(),
            base.modifiers().to_vec(),
        )
        .map_err(debug_error)?,
        CombatantSpecDigest::new(digest).expect("SHA-256 combatant digest is non-zero"),
    )
    .map_err(debug_error)
    .map(|value| {
        value
            .with_base_attack_defense(attack, defense)
            .with_base_effect_stats(base.base_effect_hit_rate(), base.base_effect_resistance())
            .with_build_bonuses(base.build_bonuses())
    })?
    .with_energy(base.current_energy(), base.maximum_energy())
    .and_then(|value| value.with_toughness(base.rank(), base.weaknesses().to_vec(), toughness))
    .and_then(|value| value.with_sources(base.sources().to_vec()))
    .and_then(|value| value.with_modifier_bindings(base.modifier_bindings().to_vec()))
    .map_err(debug_error)
}

fn scale_toughness(
    layer: &ToughnessLayerSpec,
    ratio: Ratio,
) -> Result<ToughnessLayerSpec, CurrencyWarsBattleAssemblyError> {
    let maximum = ratio
        .checked_apply(
            Scalar::checked_from_integer(layer.maximum().get()).map_err(debug_error)?,
            Rounding::NearestTiesAway,
        )
        .and_then(|value| {
            starclock_combat::RawToughness::from_scalar(value, Rounding::NearestTiesAway)
        })
        .map_err(debug_error)?;
    let mut scaled = ToughnessLayerSpec::ordinary(layer.key(), maximum)
        .map_err(debug_error)?
        .with_kind(layer.kind())
        .with_active(layer.active())
        .with_locked(layer.locked())
        .with_weakness_policy(layer.weakness_policy())
        .map_err(debug_error)?
        .with_break_behavior(
            layer.reducible_while_broken(),
            layer.applies_break_damage(),
            layer.applies_break_effect(),
            layer.changes_global_broken(),
        )
        .with_break_credit(layer.break_credit())
        .with_recovery_ratio(layer.recovery_ratio())
        .map_err(debug_error)?;
    if let Some(element) = layer.break_element() {
        scaled = scaled.with_break_element(element);
    }
    Ok(scaled)
}
