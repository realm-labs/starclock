//! Pure authored-policy lowering; no battle or Activity mutation.
use crate::{
    digest::Encoder,
    divergent_universe::{
        contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
        weighted_curio_overflow::{OverflowBaseDamagePolicy, OverflowBaseDamagePolicyError},
    },
};
use starclock_combat::{
    Rounding, Scalar,
    modifier::model::StatQuerySubject,
    rule::model::{Comparison, ConditionExpr, RuleValue, ValueExpr},
};
use starclock_data::divergent_universe_decisions::weighted_curio_overflows::{
    WeightedCurioOverflowBaseDefinition, WeightedCurioOverflowBasePolicy,
};

pub(super) fn compile_authored(
    definition: &WeightedCurioOverflowBaseDefinition,
    protocol: &DivergentUniverseDifficultyProtocolSnapshot,
) -> Result<OverflowBaseDamagePolicy, OverflowBaseDamagePolicyError> {
    if definition.fixed_damage_millionths != 100_000_000
        || definition.hard_level_group != 1
        || definition.hp_ratios_millionths.len() != 95
        || !definition
            .policy_note
            .starts_with("VersionedProjectPolicy:")
        || definition.replacement_condition.trim().is_empty()
    {
        return Err(OverflowBaseDamagePolicyError::InvalidDefinition);
    }
    let factor = protocol_factor(protocol.maximum_hp_increase_millionths())?;
    let fixed = Scalar::from_scaled(definition.fixed_damage_millionths);
    for ratio in &definition.hp_ratios_millionths {
        if *ratio <= 0 {
            return Err(OverflowBaseDamagePolicyError::InvalidDefinition);
        }
        fixed
            .checked_mul(Scalar::from_scaled(*ratio), Rounding::Floor)
            .and_then(|value| value.checked_mul(factor, Rounding::Floor))
            .map_err(|_| OverflowBaseDamagePolicyError::Arithmetic)?;
    }
    let mut identity = Encoder::new(b"starclock.divergent-universe.overflow.authored-base-policy");
    match definition.policy {
        WeightedCurioOverflowBasePolicy::VersionedProjectPolicyEnemyGroupOneHpRatioProtocolHpMultiplierFloor => {
            identity.text("EnemyGroupOneHpRatioProtocolHpMultiplierFloor");
        }
    }
    identity.i64(definition.fixed_damage_millionths);
    identity.u32(u32::from(definition.hard_level_group));
    identity.u32(95);
    for value in &definition.hp_ratios_millionths {
        identity.i64(*value);
    }
    identity.text(&definition.policy_note);
    identity.text(&definition.replacement_condition);
    identity.digest(protocol.digest());
    identity.i64(factor.scaled());
    Ok(OverflowBaseDamagePolicy::new(
        multiply(
            multiply(scalar(fixed), curve(&definition.hp_ratios_millionths, 1)?),
            scalar(factor),
        ),
        identity.finish(),
    ))
}

// A balanced tree has at most seven level comparisons. UnitLevel guarantees
// 1..=95; there is no clamped, zero or nearest-level fallback outside that domain.
fn curve(values: &[i64], first_level: u16) -> Result<ValueExpr, OverflowBaseDamagePolicyError> {
    if let [value] = values {
        return Ok(scalar(Scalar::from_scaled(*value)));
    }
    if values.is_empty() {
        return Err(OverflowBaseDamagePolicyError::InvalidDefinition);
    }
    let split = values.len() / 2;
    let next = first_level
        .checked_add(u16::try_from(split).map_err(|_| OverflowBaseDamagePolicyError::Arithmetic)?)
        .ok_or(OverflowBaseDamagePolicyError::Arithmetic)?;
    let maximum_left = next
        .checked_sub(1)
        .ok_or(OverflowBaseDamagePolicyError::Arithmetic)?;
    Ok(ValueExpr::Choose {
        condition: Box::new(ConditionExpr::Compare {
            operator: Comparison::LessOrEqual,
            lhs: Box::new(ValueExpr::QueryUnitLevel(StatQuerySubject::CurrentTarget)),
            rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(
                maximum_left,
            )))),
        }),
        when_true: Box::new(curve(&values[..split], first_level)?),
        when_false: Box::new(curve(&values[split..], next)?),
    })
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

fn protocol_factor(increase: i64) -> Result<Scalar, OverflowBaseDamagePolicyError> {
    if increase < 0 {
        return Err(OverflowBaseDamagePolicyError::InvalidProtocol);
    }
    Scalar::ONE
        .checked_add(Scalar::from_scaled(increase))
        .map_err(|_| OverflowBaseDamagePolicyError::Arithmetic)
}

#[cfg(test)]
mod tests {
    use super::{OverflowBaseDamagePolicyError, Scalar, protocol_factor};

    #[test]
    fn weighted_curio_overflow_base_protocol_factor_checks_signed_boundaries() {
        for (increase, expected) in [(0, 1_000_000), (1, 1_000_001), (2_000_000, 3_000_000)] {
            assert_eq!(
                protocol_factor(increase).unwrap(),
                Scalar::from_scaled(expected)
            );
        }
        assert_eq!(
            protocol_factor(-1),
            Err(OverflowBaseDamagePolicyError::InvalidProtocol)
        );
        assert_eq!(
            protocol_factor(i64::MAX),
            Err(OverflowBaseDamagePolicyError::Arithmetic)
        );
        assert_eq!(protocol_factor(i64::MAX - 1_000_000).unwrap(), Scalar::MAX);
    }
}
