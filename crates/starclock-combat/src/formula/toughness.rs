//! Toughness, Break and Super Break fixed-point calculators.

use crate::{DamageAmount, NumericError, Ratio, RawToughness, Rounding, Scalar};

use super::model::CombatElement;

const ROUNDING: Rounding = Rounding::NearestTiesEven;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToughnessReductionContext {
    pub base: RawToughness,
    pub additive: RawToughness,
    pub reduction_increase: Ratio,
    pub weakness_break_efficiency: Ratio,
    pub weakness_break_efficiency_cap: Ratio,
    pub toughness_vulnerability: Ratio,
    pub ability_multiplier: Ratio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToughnessReductionCalculation {
    pub uncapped_efficiency: Ratio,
    pub capped_efficiency: Ratio,
    pub raw: Scalar,
    pub attempted: RawToughness,
}

pub fn reduction(
    context: ToughnessReductionContext,
) -> Result<ToughnessReductionCalculation, NumericError> {
    for value in [
        context.reduction_increase,
        context.weakness_break_efficiency,
        context.weakness_break_efficiency_cap,
        context.toughness_vulnerability,
        context.ability_multiplier,
    ] {
        non_negative(value)?;
    }
    let base = context
        .base
        .get()
        .checked_add(context.additive.get())
        .ok_or(NumericError::Overflow)?;
    let capped_efficiency = Ratio::from_scaled(
        context
            .weakness_break_efficiency
            .scaled()
            .min(context.weakness_break_efficiency_cap.scaled()),
    );
    let mut raw = Scalar::checked_from_integer(base)?;
    for factor in [
        Ratio::ONE.checked_add(context.reduction_increase)?,
        Ratio::ONE
            .checked_add(capped_efficiency)?
            .checked_add(context.toughness_vulnerability)?,
        context.ability_multiplier,
    ] {
        raw = factor.checked_apply(raw, ROUNDING)?;
    }
    Ok(ToughnessReductionCalculation {
        uncapped_efficiency: context.weakness_break_efficiency,
        capped_efficiency,
        raw,
        attempted: RawToughness::from_scalar(raw, Rounding::Floor)?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreakDamageDefinition {
    pub attacker_level_multiplier: Scalar,
    pub ability_multiplier: Ratio,
    pub break_effect: Ratio,
    pub break_damage_increase: Ratio,
    pub defense_multiplier: Ratio,
    pub resistance_multiplier: Ratio,
    pub vulnerability_multiplier: Ratio,
    pub mitigation_multiplier: Ratio,
    pub unbroken_multiplier: Ratio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BreakDamageCalculation {
    pub maximum_toughness_multiplier: Ratio,
    pub base: Scalar,
    pub raw: Scalar,
    pub finalized: DamageAmount,
}

pub fn break_damage(
    definition: BreakDamageDefinition,
    element: CombatElement,
    maximum_toughness: RawToughness,
    was_broken: bool,
) -> Result<BreakDamageCalculation, NumericError> {
    validate_break_definition(definition)?;
    let max_factor = Ratio::from_scaled(500_000).checked_add(Ratio::from_scaled(
        Scalar::checked_from_integer(maximum_toughness.get())?
            .checked_div_integer(40, ROUNDING)?
            .scaled(),
    ))?;
    let coefficient = element_break_coefficient(element);
    let mut base = coefficient.checked_apply(definition.attacker_level_multiplier, ROUNDING)?;
    base = max_factor.checked_apply(base, ROUNDING)?;
    let mut raw = base;
    for factor in [
        definition.ability_multiplier,
        Ratio::ONE.checked_add(definition.break_effect)?,
        Ratio::ONE.checked_add(definition.break_damage_increase)?,
        definition.defense_multiplier,
        definition.resistance_multiplier,
        definition.vulnerability_multiplier,
        definition.mitigation_multiplier,
        if was_broken {
            Ratio::ONE
        } else {
            definition.unbroken_multiplier
        },
    ] {
        raw = factor.checked_apply(raw, ROUNDING)?;
    }
    Ok(BreakDamageCalculation {
        maximum_toughness_multiplier: max_factor,
        base,
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SuperBreakDefinition {
    pub element: CombatElement,
    pub attacker_level_multiplier: Scalar,
    pub ability_multiplier: Ratio,
    pub break_effect: Ratio,
    pub break_damage_increase: Ratio,
    pub super_break_increase: Ratio,
    pub defense_multiplier: Ratio,
    pub resistance_multiplier: Ratio,
    pub vulnerability_multiplier: Ratio,
    pub mitigation_multiplier: Ratio,
    pub broken_multiplier: Ratio,
}

pub fn super_break_damage(
    definition: SuperBreakDefinition,
    effective_reduction: RawToughness,
) -> Result<BreakDamageCalculation, NumericError> {
    for value in [
        definition.ability_multiplier,
        definition.break_effect,
        definition.break_damage_increase,
        definition.super_break_increase,
        definition.defense_multiplier,
        definition.resistance_multiplier,
        definition.vulnerability_multiplier,
        definition.mitigation_multiplier,
        definition.broken_multiplier,
    ] {
        non_negative(value)?;
    }
    if definition.attacker_level_multiplier.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let units = Scalar::checked_from_integer(effective_reduction.get())?
        .checked_div_integer(10, ROUNDING)?;
    let base = units.checked_mul(definition.attacker_level_multiplier, ROUNDING)?;
    let mut raw = base;
    for factor in [
        definition.ability_multiplier,
        Ratio::ONE.checked_add(definition.break_effect)?,
        Ratio::ONE.checked_add(definition.break_damage_increase)?,
        Ratio::ONE.checked_add(definition.super_break_increase)?,
        definition.defense_multiplier,
        definition.resistance_multiplier,
        definition.vulnerability_multiplier,
        definition.mitigation_multiplier,
        definition.broken_multiplier,
    ] {
        raw = factor.checked_apply(raw, ROUNDING)?;
    }
    Ok(BreakDamageCalculation {
        maximum_toughness_multiplier: Ratio::ONE,
        base,
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

/// Passes an element-effect base amount through the same common Break factors.
pub fn break_effect_damage(
    definition: BreakDamageDefinition,
    base: Scalar,
    broken: bool,
) -> Result<BreakDamageCalculation, NumericError> {
    validate_break_definition(definition)?;
    if base.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let mut raw = base;
    for factor in [
        definition.ability_multiplier,
        Ratio::ONE.checked_add(definition.break_effect)?,
        Ratio::ONE.checked_add(definition.break_damage_increase)?,
        definition.defense_multiplier,
        definition.resistance_multiplier,
        definition.vulnerability_multiplier,
        definition.mitigation_multiplier,
        if broken {
            Ratio::ONE
        } else {
            definition.unbroken_multiplier
        },
    ] {
        raw = factor.checked_apply(raw, ROUNDING)?;
    }
    Ok(BreakDamageCalculation {
        maximum_toughness_multiplier: Ratio::ONE,
        base,
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnemyRank {
    Normal,
    EliteOrBoss,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BaseBreakEffect {
    pub element: CombatElement,
    pub base_damage: Option<Scalar>,
    pub duration_turns: u8,
    pub initial_stacks: u8,
    pub maximum_stacks: u8,
    pub additional_delay: Ratio,
    pub speed_reduction: Ratio,
    pub skips_action: bool,
}

pub fn base_break_effect(
    element: CombatElement,
    rank: EnemyRank,
    target_max_hp: crate::Hp,
    level_multiplier: Scalar,
    maximum_toughness: RawToughness,
    break_effect: Ratio,
) -> Result<BaseBreakEffect, NumericError> {
    non_negative(break_effect)?;
    if level_multiplier.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let max_factor = Ratio::from_scaled(500_000).checked_add(Ratio::from_scaled(
        Scalar::checked_from_integer(maximum_toughness.get())?
            .checked_div_integer(40, ROUNDING)?
            .scaled(),
    ))?;
    let hp = Scalar::checked_from_integer(target_max_hp.get())?;
    let one_plus_break = Ratio::ONE.checked_add(break_effect)?;
    let (base_damage, duration, initial_stacks, maximum_stacks, delay, speed, skips) = match element
    {
        CombatElement::Physical => {
            let normal = Ratio::from_scaled(160_000).checked_apply(hp, ROUNDING)?;
            let elite = Ratio::from_scaled(70_000).checked_apply(hp, ROUNDING)?;
            let cap = max_factor.checked_apply(
                Ratio::from_scaled(2_000_000).checked_apply(level_multiplier, ROUNDING)?,
                ROUNDING,
            )?;
            (
                Some(if rank == EnemyRank::Normal {
                    normal
                } else {
                    elite.min(cap)
                }),
                2,
                1,
                1,
                Ratio::ZERO,
                Ratio::ZERO,
                false,
            )
        }
        CombatElement::Fire => (
            Some(level_multiplier),
            2,
            1,
            1,
            Ratio::ZERO,
            Ratio::ZERO,
            false,
        ),
        CombatElement::Ice => (
            Some(level_multiplier),
            1,
            1,
            1,
            Ratio::ZERO,
            Ratio::ZERO,
            true,
        ),
        CombatElement::Lightning => (
            Some(Ratio::from_scaled(2_000_000).checked_apply(level_multiplier, ROUNDING)?),
            2,
            1,
            1,
            Ratio::ZERO,
            Ratio::ZERO,
            false,
        ),
        CombatElement::Wind => (
            Some(level_multiplier),
            2,
            if rank == EnemyRank::Normal { 1 } else { 3 },
            5,
            Ratio::ZERO,
            Ratio::ZERO,
            false,
        ),
        CombatElement::Quantum => (
            Some(max_factor.checked_apply(
                Ratio::from_scaled(600_000).checked_apply(level_multiplier, ROUNDING)?,
                ROUNDING,
            )?),
            1,
            0,
            5,
            Ratio::from_scaled(200_000).checked_mul(one_plus_break, ROUNDING)?,
            Ratio::ZERO,
            false,
        ),
        CombatElement::Imaginary => (
            None,
            1,
            1,
            1,
            Ratio::from_scaled(300_000).checked_mul(one_plus_break, ROUNDING)?,
            Ratio::from_scaled(100_000),
            false,
        ),
    };
    Ok(BaseBreakEffect {
        element,
        base_damage,
        duration_turns: duration,
        initial_stacks,
        maximum_stacks,
        additional_delay: delay,
        speed_reduction: speed,
        skips_action: skips,
    })
}

#[must_use]
pub const fn element_break_coefficient(element: CombatElement) -> Ratio {
    Ratio::from_scaled(match element {
        CombatElement::Physical | CombatElement::Fire => 2_000_000,
        CombatElement::Ice | CombatElement::Lightning => 1_000_000,
        CombatElement::Wind => 1_500_000,
        CombatElement::Quantum | CombatElement::Imaginary => 500_000,
    })
}

fn validate_break_definition(value: BreakDamageDefinition) -> Result<(), NumericError> {
    if value.attacker_level_multiplier.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    for ratio in [
        value.ability_multiplier,
        value.break_effect,
        value.break_damage_increase,
        value.defense_multiplier,
        value.resistance_multiplier,
        value.vulnerability_multiplier,
        value.mitigation_multiplier,
        value.unbroken_multiplier,
    ] {
        non_negative(ratio)?;
    }
    Ok(())
}

fn non_negative(value: Ratio) -> Result<(), NumericError> {
    if value.scaled() < 0 {
        Err(NumericError::OutOfDomain)
    } else {
        Ok(())
    }
}
