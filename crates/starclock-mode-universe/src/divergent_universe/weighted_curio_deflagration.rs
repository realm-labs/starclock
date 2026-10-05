//! Independent Most Raucous base-policy lowering; equipment remains unsupported.
use crate::{
    digest::Encoder,
    divergent_universe::{
        contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
        weighted_curio::target_level_hp_curve::{self, TargetLevelHpCurveError},
    },
};
use starclock_combat::{
    Rounding, Scalar,
    rule::model::{RuleValue, ValueExpr},
};
use starclock_data::divergent_universe_decisions::weighted_curio_deflagrations::{
    WeightedCurioDeflagrationBasePolicy, WeightedCurioDeflagrationDefinition,
};

/// Construction fails before any battle mutation or equipment admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeflagrationBaseDamagePolicyError {
    InvalidDefinition,
    InvalidProtocol,
    Arithmetic,
}

impl core::fmt::Display for DeflagrationBaseDamagePolicyError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "Deflagration base damage policy: {self:?}")
    }
}
impl std::error::Error for DeflagrationBaseDamagePolicyError {}

/// Base damage only: not the twice-base Burn magnitude or execution policy.
/// Identity includes every level, base policy/replacement note and immutable
/// Protocol snapshot. Neither construction nor a formula probe admits equipment.
#[derive(Clone, Debug)]
pub struct DeflagrationBaseDamagePolicy {
    expression: ValueExpr,
    identity: [u8; 32],
}

impl DeflagrationBaseDamagePolicy {
    /// Compile the explicitly authored target-level/Group-1/Protocol-HP policy.
    /// This is not decoded upstream postfix parity. Both Scalar products floor
    /// to six places. Consumers supply a real CurrentTarget; no live Activity
    /// lookup, owner-ATK substitution or nearest-level fallback is performed.
    /// Validates all 95 levels for overflow before constructing any catalog.
    pub fn from_authored(
        definition: &WeightedCurioDeflagrationDefinition,
        protocol: &DivergentUniverseDifficultyProtocolSnapshot,
    ) -> Result<Self, DeflagrationBaseDamagePolicyError> {
        if definition.fixed_base_damage_millionths != 100_000_000
            || definition.hard_level_group != 1
            || !definition
                .base_policy_note
                .starts_with("VersionedProjectPolicy:")
            || definition.base_replacement_condition.trim().is_empty()
        {
            return Err(DeflagrationBaseDamagePolicyError::InvalidDefinition);
        }
        let curve =
            target_level_hp_curve::compile(&definition.hp_ratios_millionths).map_err(|error| {
                match error {
                    TargetLevelHpCurveError::InvalidCurve => {
                        DeflagrationBaseDamagePolicyError::InvalidDefinition
                    }
                    TargetLevelHpCurveError::Arithmetic => {
                        DeflagrationBaseDamagePolicyError::Arithmetic
                    }
                }
            })?;
        let factor = protocol_factor(protocol.maximum_hp_increase_millionths())?;
        let fixed = Scalar::from_scaled(definition.fixed_base_damage_millionths);
        for ratio in &definition.hp_ratios_millionths {
            fixed
                .checked_mul(Scalar::from_scaled(*ratio), Rounding::Floor)
                .and_then(|value| value.checked_mul(factor, Rounding::Floor))
                .map_err(|_| DeflagrationBaseDamagePolicyError::Arithmetic)?;
        }
        let mut identity =
            Encoder::new(b"starclock.divergent-universe.deflagration.authored-base-policy");
        match definition.base_policy {
            WeightedCurioDeflagrationBasePolicy::VersionedProjectPolicyTargetGroupOneHpRatioProtocolHpFloor => {
                identity.text("TargetGroupOneHpRatioProtocolHpFloor");
            }
        }
        identity.i64(definition.fixed_base_damage_millionths);
        identity.u32(u32::from(definition.hard_level_group));
        identity.u32(95);
        for ratio in &definition.hp_ratios_millionths {
            identity.i64(*ratio);
        }
        identity.text(&definition.base_policy_note);
        identity.text(&definition.base_replacement_condition);
        identity.digest(protocol.digest());
        identity.i64(factor.scaled());
        Ok(Self {
            expression: multiply(multiply(scalar(fixed), curve), scalar(factor)),
            identity: identity.finish(),
        })
    }

    /// CurrentTarget-scoped base Scalar expression. The shared command pipeline
    /// owns evaluation, target lifetime, arithmetic faults and event emission.
    #[must_use]
    pub fn expression(&self) -> &ValueExpr {
        &self.expression
    }

    /// Bind this digest along with the expression in any consuming catalog.
    #[must_use]
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }
}

fn scalar(value: Scalar) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Scalar(value))
}
fn multiply(lhs: ValueExpr, rhs: ValueExpr) -> ValueExpr {
    ValueExpr::Multiply {
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        rounding: Rounding::Floor,
    }
}
fn protocol_factor(increase: i64) -> Result<Scalar, DeflagrationBaseDamagePolicyError> {
    if increase < 0 {
        return Err(DeflagrationBaseDamagePolicyError::InvalidProtocol);
    }
    Scalar::ONE
        .checked_add(Scalar::from_scaled(increase))
        .map_err(|_| DeflagrationBaseDamagePolicyError::Arithmetic)
}

#[cfg(test)]
mod tests {
    use super::{DeflagrationBaseDamagePolicyError, Scalar, protocol_factor};

    #[test]
    fn deflagration_base_protocol_factor_checks_signed_boundaries() {
        for (increase, expected) in [(0, 1_000_000), (1, 1_000_001), (2_000_000, 3_000_000)] {
            assert_eq!(
                protocol_factor(increase).unwrap(),
                Scalar::from_scaled(expected)
            );
        }
        assert_eq!(
            protocol_factor(-1),
            Err(DeflagrationBaseDamagePolicyError::InvalidProtocol)
        );
        assert_eq!(
            protocol_factor(i64::MAX),
            Err(DeflagrationBaseDamagePolicyError::Arithmetic)
        );
        assert_eq!(protocol_factor(i64::MAX - 1_000_000).unwrap(), Scalar::MAX);
    }
}
