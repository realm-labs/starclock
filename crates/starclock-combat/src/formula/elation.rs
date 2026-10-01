//! Explicit Elation factors, independent of the ordinary damage-boost pipeline.
//!
//! This is a pure calculator, not a mutation path or an implicit conversion
//! from `DamageClass::Elation`. Callers own level-base selection, stat/resource
//! snapshots and the authored meter-to-multiplier policy.
//! Factor composition and precision are an explicit ProjectPolicy contract,
//! not a claim of observed original-game damage parity.

use crate::{DamageAmount, NumericError, Ratio, Rounding, Scalar};

use super::{
    damage,
    model::{CritDecision, DefenseInput, ResistanceInput},
};

/// Immutable inputs for one already-selected Elation damage target/hit.
///
/// `base_damage` is the resolved level-derived amount, not ATK or Break base
/// damage. `elation` and `merrymaking` are additive ratios, whereas
/// `meter_multiplier` is an already-resolved factor. This type deliberately
/// accepts no ordinary damage boost or weaken input. It does not choose a
/// resource cap, consume the meter or decide a CRIT result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElationDamageContext {
    pub base_damage: Scalar,
    pub original_damage_multiplier: Ratio,
    pub crit: CritDecision,
    pub crit_damage: Ratio,
    pub elation: Ratio,
    pub meter_multiplier: Ratio,
    pub merrymaking: Ratio,
    pub defense: DefenseInput,
    pub resistance: ResistanceInput,
    pub vulnerabilities: Box<[Ratio]>,
    pub mitigations: Box<[Ratio]>,
    pub broken: bool,
    pub unbroken_multiplier: Ratio,
    /// Already-filtered stage contributions, with no ordinary boost/weaken.
    pub modifiers: ElationDamageModifiers,
}

/// Explicit stage contributions in the shared modifier language.
/// DEF/RES/Broken contributions add to the corresponding resolved factor;
/// mitigation is a further reduction fraction. These are not DEF-ignore fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElationDamageModifiers {
    pub flat_base: Scalar,
    pub crit: Ratio,
    pub defense: Ratio,
    pub resistance: Ratio,
    pub vulnerability: Ratio,
    pub mitigation: Ratio,
    pub broken: Ratio,
}
impl Default for ElationDamageModifiers {
    fn default() -> Self {
        Self {
            flat_base: Scalar::ZERO,
            crit: Ratio::ZERO,
            defense: Ratio::ZERO,
            resistance: Ratio::ZERO,
            vulnerability: Ratio::ZERO,
            mitigation: Ratio::ZERO,
            broken: Ratio::ZERO,
        }
    }
}

/// Named factors and the once-finalized result of the Elation calculator.
///
/// The raw value retains six decimal places. No intermediate factor is rounded
/// to an integral damage amount, including when a factor reduces the damage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ElationDamageCalculation {
    pub base: Scalar,
    pub original_damage_multiplier: Ratio,
    pub crit_multiplier: Ratio,
    pub elation_multiplier: Ratio,
    pub meter_multiplier: Ratio,
    pub merrymaking_multiplier: Ratio,
    pub defense_multiplier: Ratio,
    pub resistance_multiplier: Ratio,
    pub vulnerability_multiplier: Ratio,
    pub mitigation_multiplier: Ratio,
    pub broken_multiplier: Ratio,
    pub raw: Scalar,
    pub finalized: DamageAmount,
}

/// Calculates one Elation result without reading state, mutating it or using RNG.
///
/// Factors are applied in the documented order with nearest-ties-even decimal
/// arithmetic; applied damage is floored exactly once. Negative base damage,
/// negative CRIT DMG, negative effective factors and invalid shared DEF/RES or
/// mitigation inputs return `OutOfDomain`; checked overflow returns `Overflow`.
/// Additive Elation/merrymaking ratios may reduce their factor down to zero,
/// but are never silently clamped. See `docs/combat-elation-formula.md` for the
/// explicit project precision policy and native/production integration boundary.
pub fn calculate(context: &ElationDamageContext) -> Result<ElationDamageCalculation, NumericError> {
    if context.base_damage.scaled() < 0 || context.crit_damage.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let base = context
        .base_damage
        .checked_add(context.modifiers.flat_base)?;
    if base.scaled() < 0 || !(0..=1_000_000).contains(&context.modifiers.mitigation.scaled()) {
        return Err(NumericError::OutOfDomain);
    }
    let crit_multiplier = match context.crit {
        CritDecision::Ineligible | CritDecision::Normal => Ratio::ONE,
        CritDecision::Critical => Ratio::ONE.checked_add(context.crit_damage)?,
    }
    .checked_add(context.modifiers.crit)?;
    let elation_multiplier = Ratio::ONE.checked_add(context.elation)?;
    let merrymaking_multiplier = Ratio::ONE.checked_add(context.merrymaking)?;
    let defense_multiplier =
        damage::defense_multiplier(context.defense)?.checked_add(context.modifiers.defense)?;
    let resistance_multiplier = damage::resistance_multiplier(context.resistance)?
        .checked_add(context.modifiers.resistance)?;
    let vulnerability_multiplier = damage::additive_multiplier(&context.vulnerabilities)?
        .checked_add(context.modifiers.vulnerability)?;
    let mitigation_multiplier = damage::mitigation_multiplier(&context.mitigations)?.checked_mul(
        Ratio::ONE.checked_sub(context.modifiers.mitigation)?,
        Rounding::NearestTiesEven,
    )?;
    let broken_multiplier = if context.broken {
        Ratio::ONE
    } else {
        context.unbroken_multiplier
    }
    .checked_add(context.modifiers.broken)?;
    if context.unbroken_multiplier.scaled() < 0 {
        return Err(NumericError::OutOfDomain);
    }
    let raw = damage::apply_factors(
        base,
        &[
            context.original_damage_multiplier,
            crit_multiplier,
            elation_multiplier,
            context.meter_multiplier,
            merrymaking_multiplier,
            defense_multiplier,
            resistance_multiplier,
            vulnerability_multiplier,
            mitigation_multiplier,
            broken_multiplier,
        ],
    )?;
    Ok(ElationDamageCalculation {
        base,
        original_damage_multiplier: context.original_damage_multiplier,
        crit_multiplier,
        elation_multiplier,
        meter_multiplier: context.meter_multiplier,
        merrymaking_multiplier,
        defense_multiplier,
        resistance_multiplier,
        vulnerability_multiplier,
        mitigation_multiplier,
        broken_multiplier,
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

#[cfg(test)]
mod tests;
