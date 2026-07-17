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
