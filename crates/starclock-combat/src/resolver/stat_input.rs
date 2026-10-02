//! Shared immutable inputs for staged stat and formula queries.

use crate::{
    Scalar, TeamSide, UnitId, battle::fault::BattleFault,
    formula::toughness::attacker_level_multiplier, modifier::model::StatKind,
    resolver::program::fault::program_fault,
};
use std::collections::BTreeMap;

use super::transaction::Transaction;

pub(super) fn stat_bases(
    txn: &Transaction<'_>,
) -> Result<BTreeMap<(UnitId, StatKind), Scalar>, BattleFault> {
    use crate::modifier::model::StatKind::{
        Aggro, Atk, BreakBaseDamage, BreakEffect, ControlResistance, CritDamage, CritRate,
        DebuffDurationMultiplier, Def, DotDurationAddition, EffectHitRate, EffectResistance,
        Elation, EnergyRegenerationRate, FireDamageBoost, FreezeResistance, Hp, IceDamageBoost,
        ImaginaryDamageBoost, LightningDamageBoost, OutgoingHealing, PhysicalDamageBoost,
        QuantumDamageBoost, Spd, ToughnessDamage, ToughnessRecovery, WindDamageBoost,
    };

    let mut bases = BTreeMap::new();
    for unit in txn.state.units.iter_by_id() {
        bases.insert((unit.id, Aggro), Scalar::ONE);
        bases.insert((unit.id, Elation), Scalar::ZERO);
        bases.insert(
            (unit.id, Hp),
            Scalar::checked_from_integer(unit.maximum_hp.get())
                .map_err(|_| program_fault(44, unit.maximum_hp.get()))?,
        );
        bases.insert(
            (unit.id, Atk),
            Scalar::from_scaled(unit.base_attack.scaled()),
        );
        bases.insert(
            (unit.id, Def),
            Scalar::from_scaled(unit.base_defense.scaled()),
        );
        bases.insert(
            (unit.id, Spd),
            Scalar::from_scaled(unit.base_speed.scaled()),
        );
        let player = unit.side == TeamSide::Player;
        let [
            critical_rate,
            critical_damage,
            break_effect,
            energy_regeneration,
            outgoing_healing,
        ] = unit.build_bonuses.secondary();
        bases.insert(
            (unit.id, CritRate),
            Scalar::from_scaled(if player { 50_000 } else { 0 })
                .checked_add(critical_rate)
                .map_err(|_| program_fault(45, critical_rate.scaled()))?,
        );
        bases.insert(
            (unit.id, CritDamage),
            Scalar::from_scaled(if player { 500_000 } else { 0 })
                .checked_add(critical_damage)
                .map_err(|_| program_fault(46, critical_damage.scaled()))?,
        );
        bases.insert((unit.id, EffectHitRate), unit.base_effect_hit_rate);
        bases.insert((unit.id, EffectResistance), unit.base_effect_resistance);
        bases.insert((unit.id, BreakEffect), break_effect);
        bases.insert(
            (unit.id, EnergyRegenerationRate),
            Scalar::ONE
                .checked_add(energy_regeneration)
                .map_err(|_| program_fault(47, energy_regeneration.scaled()))?,
        );
        bases.insert((unit.id, OutgoingHealing), outgoing_healing);
        for (stat, value) in [
            PhysicalDamageBoost,
            FireDamageBoost,
            IceDamageBoost,
            LightningDamageBoost,
            WindDamageBoost,
            QuantumDamageBoost,
            ImaginaryDamageBoost,
        ]
        .into_iter()
        .zip(unit.build_bonuses.element_damage_boosts())
        {
            bases.insert((unit.id, stat), value);
        }
        bases.insert((unit.id, FreezeResistance), Scalar::ZERO);
        bases.insert((unit.id, ControlResistance), Scalar::ZERO);
        bases.insert((unit.id, ToughnessDamage), Scalar::ZERO);
        bases.insert((unit.id, ToughnessRecovery), Scalar::ONE);
        if let Some(value) = attacker_level_multiplier(unit.level) {
            bases.insert((unit.id, BreakBaseDamage), value);
        }
        bases.insert((unit.id, DotDurationAddition), Scalar::ZERO);
        bases.insert((unit.id, DebuffDurationMultiplier), Scalar::ONE);
    }
    Ok(bases)
}

pub(super) fn shield_values(txn: &Transaction<'_>) -> BTreeMap<UnitId, Scalar> {
    txn.state
        .units
        .iter_by_id()
        .map(|unit| {
            let value = txn
                .state
                .shields
                .effective_remaining(unit.id)
                .ok()
                .and_then(|value| Scalar::checked_from_integer(value.get()).ok())
                .unwrap_or(Scalar::ZERO);
            (unit.id, value)
        })
        .collect()
}
