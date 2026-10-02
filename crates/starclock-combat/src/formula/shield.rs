//! Shield creation and deterministic multi-instance absorption policies.

use crate::{DamageAmount, NumericError, Rounding, ShieldAmount, ShieldInstanceId};

use super::{
    damage,
    model::{ShieldCalculation, ShieldContext},
};

const ROUNDING: Rounding = Rounding::NearestTiesEven;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldInstance {
    pub id: ShieldInstanceId,
    pub remaining: ShieldAmount,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShieldAbsorptionPolicy {
    ConcurrentLargest,
    AdditiveByInstance,
}

/// Capacity mutation, independent of shield creation bonuses or damage absorption.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShieldAdjustmentKind {
    Increase,
    /// Removes at most the remaining capacity; never creates a negative shield.
    Decrease,
}

/// Checked integral capacity change. Decreases explicitly clamp at zero.
pub fn adjust(
    before: ShieldAmount,
    requested: ShieldAmount,
    kind: ShieldAdjustmentKind,
) -> Result<ShieldAmount, NumericError> {
    let after = match kind {
        ShieldAdjustmentKind::Increase => before
            .get()
            .checked_add(requested.get())
            .ok_or(NumericError::Overflow)?,
        ShieldAdjustmentKind::Decrease => before.get() - before.get().min(requested.get()),
    };
    ShieldAmount::new(after)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ShieldDamageResult {
    pub incoming: DamageAmount,
    pub absorbed: DamageAmount,
    pub hp_overflow: DamageAmount,
}

pub fn calculate(context: &ShieldContext) -> Result<ShieldCalculation, NumericError> {
    let base = damage::base_amount(&context.scaling_terms, context.additive_base)?;
    let multiplier = damage::additive_multiplier(&context.bonuses)?;
    let raw = multiplier.checked_apply(base, ROUNDING)?;
    Ok(ShieldCalculation {
        base,
        multiplier,
        raw,
        finalized: ShieldAmount::from_scalar(raw, Rounding::Floor)?,
    })
}

/// Applies one incoming amount while retaining every authored shield instance.
pub fn absorb(
    instances: &mut [ShieldInstance],
    incoming: DamageAmount,
    policy: ShieldAbsorptionPolicy,
) -> Result<ShieldDamageResult, NumericError> {
    instances.sort_unstable_by_key(|instance| instance.id);
    let absorbed_raw = match policy {
        ShieldAbsorptionPolicy::ConcurrentLargest => {
            let visible = instances
                .iter()
                .map(|instance| instance.remaining.get())
                .max()
                .unwrap_or(0);
            for instance in instances.iter_mut() {
                instance.remaining = ShieldAmount::new(
                    instance
                        .remaining
                        .get()
                        .saturating_sub(incoming.get())
                        .max(0),
                )?;
            }
            incoming.get().min(visible)
        }
        ShieldAbsorptionPolicy::AdditiveByInstance => {
            let mut remaining = incoming.get();
            for instance in instances.iter_mut() {
                let consumed = remaining.min(instance.remaining.get());
                instance.remaining = ShieldAmount::new(instance.remaining.get() - consumed)?;
                remaining -= consumed;
            }
            incoming.get() - remaining
        }
    };
    Ok(ShieldDamageResult {
        incoming,
        absorbed: DamageAmount::new(absorbed_raw)?,
        hp_overflow: DamageAmount::new(incoming.get() - absorbed_raw)?,
    })
}

#[cfg(test)]
mod adjustment_tests {
    use super::*;

    #[test]
    fn integral_adjustment_boundaries_and_overflow_are_explicit() {
        for (before, amount, kind, expected) in [
            (0, 0, ShieldAdjustmentKind::Increase, 0),
            (0, 9, ShieldAdjustmentKind::Increase, 9),
            (9, 0, ShieldAdjustmentKind::Decrease, 9),
            (9, 4, ShieldAdjustmentKind::Decrease, 5),
            (9, 9, ShieldAdjustmentKind::Decrease, 0),
            (9, 10, ShieldAdjustmentKind::Decrease, 0),
            (i64::MAX, i64::MAX, ShieldAdjustmentKind::Decrease, 0),
        ] {
            assert_eq!(
                adjust(
                    ShieldAmount::new(before).unwrap(),
                    ShieldAmount::new(amount).unwrap(),
                    kind
                )
                .unwrap()
                .get(),
                expected
            );
        }
        assert_eq!(
            adjust(
                ShieldAmount::new(i64::MAX).unwrap(),
                ShieldAmount::new(1).unwrap(),
                ShieldAdjustmentKind::Increase
            ),
            Err(NumericError::Overflow)
        );
    }
}
