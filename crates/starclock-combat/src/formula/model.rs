//! Explicit immutable inputs and traces for damage and sustain formula families.

use crate::{DamageAmount, HealingAmount, Hp, Probability, Ratio, Scalar, ShieldAmount};

/// Seven base combat elements. Weakness remains a separate authored concept.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum CombatElement {
    Physical,
    Fire,
    Ice,
    Lightning,
    Wind,
    Quantum,
    Imaginary,
}

/// Named ordinary formula families that share the general multiplier pipeline.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DamageClass {
    Direct,
    Dot,
    Additional,
}

/// One authored term in a base amount, preserving mixed-stat expressions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScalingTerm {
    pub stat: Scalar,
    pub ratio: Ratio,
}

/// Already-decided per-target CRIT result. RNG ownership stays outside calculators.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CritDecision {
    Ineligible,
    Normal,
    Critical,
}

/// DEF calculation input selected explicitly by the authored operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DefenseInput {
    Actual {
        target_defense: Scalar,
        attacker_level: u16,
    },
    LevelBased {
        attacker_level: u16,
        enemy_level: u16,
        defense_bonus: Ratio,
        defense_reduction: Ratio,
        defense_ignore: Ratio,
    },
}

/// Configurable effective-RES bounds and attacker penetration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResistanceInput {
    pub target_resistance: Ratio,
    pub penetration: Ratio,
    pub minimum: Ratio,
    pub maximum: Ratio,
}

/// Complete context for the ordinary direct/DoT/additional damage pipeline.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DamageContext {
    pub class: DamageClass,
    pub element: CombatElement,
    pub scaling_terms: Box<[ScalingTerm]>,
    pub additive_base: Scalar,
    pub original_damage_multiplier: Ratio,
    pub crit: CritDecision,
    pub crit_damage: Ratio,
    pub damage_boosts: Box<[Ratio]>,
    pub total_weaken: Ratio,
    pub defense: DefenseInput,
    pub resistance: ResistanceInput,
    pub vulnerabilities: Box<[Ratio]>,
    pub mitigations: Box<[Ratio]>,
    pub broken: bool,
    pub unbroken_multiplier: Ratio,
}

/// Auditable named factors and once-finalized ordinary damage result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageCalculation {
    pub base: Scalar,
    pub crit_multiplier: Ratio,
    pub damage_boost_multiplier: Ratio,
    pub weaken_multiplier: Ratio,
    pub defense_multiplier: Ratio,
    pub resistance_multiplier: Ratio,
    pub vulnerability_multiplier: Ratio,
    pub mitigation_multiplier: Ratio,
    pub broken_multiplier: Ratio,
    pub raw: Scalar,
    pub finalized: DamageAmount,
}

/// Explicit healing formula input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealingContext {
    pub scaling_terms: Box<[ScalingTerm]>,
    pub additive_base: Scalar,
    pub outgoing_boosts: Box<[Ratio]>,
    pub incoming_boosts: Box<[Ratio]>,
    pub incoming_reductions: Box<[Ratio]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HealingCalculation {
    pub base: Scalar,
    pub multiplier: Ratio,
    pub raw: Scalar,
    pub finalized: HealingAmount,
}

/// Explicit shield-creation formula input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShieldContext {
    pub scaling_terms: Box<[ScalingTerm]>,
    pub additive_base: Scalar,
    pub bonuses: Box<[Ratio]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldCalculation {
    pub base: Scalar,
    pub multiplier: Ratio,
    pub raw: Scalar,
    pub finalized: ShieldAmount,
}

/// Checked HP-consumption outcome. Overflow is the amount blocked by the floor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HpConsumption {
    pub requested: Hp,
    pub effective: Hp,
    pub overflow: Hp,
    pub before: Hp,
    pub after: Hp,
}

/// Converts a clamped ratio into the exact integer threshold used by battle RNG.
#[must_use]
pub fn clamp_probability(value: Ratio) -> Probability {
    let raw = value.scaled().clamp(0, 1_000_000) as u32;
    Probability::from_millionths(raw).expect("clamped probability is valid")
}
