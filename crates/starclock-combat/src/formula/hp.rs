//! Pure HP boundary accounting, independent from battle mutation.

use crate::{DamageAmount, Hp, NumericError};

use super::model::HpConsumption;

/// Consumes up to `requested` HP while preserving the explicit legal floor.
pub fn consume(current: Hp, requested: Hp, floor: Hp) -> Result<HpConsumption, NumericError> {
    let effective_floor = floor.get().min(current.get());
    let available = current.get() - effective_floor;
    let effective_raw = requested.get().min(available);
    let overflow_raw = requested.get() - effective_raw;
    Ok(HpConsumption {
        requested,
        effective: Hp::new(effective_raw)?,
        overflow: Hp::new(overflow_raw)?,
        before: current,
        after: Hp::new(current.get() - effective_raw)?,
    })
}

/// Finalized damage above pre-operation HP after guards and shield absorption.
/// `calculated` must already include damage guards. Absorption cannot exceed it.
/// The lower bound is explicitly zero; a nonlethal HP floor does not count as
/// additional overflow, and neither rescue nor later healing changes this fact.
pub fn damage_overflow(
    calculated: DamageAmount,
    absorbed: DamageAmount,
    hp_before: Hp,
) -> Result<DamageAmount, NumericError> {
    let after_shield = calculated
        .get()
        .checked_sub(absorbed.get())
        .filter(|amount| *amount >= 0)
        .ok_or(NumericError::OutOfDomain)?;
    let excess = after_shield
        .checked_sub(hp_before.get())
        .ok_or(NumericError::Overflow)?;
    DamageAmount::new(excess.max(0))
}

#[cfg(test)]
mod tests {
    use super::damage_overflow;
    use crate::{DamageAmount, Hp, NumericError};

    #[test]
    fn damage_overflow_uses_finalized_post_guard_post_shield_hp_boundary() {
        for (calculated, absorbed, hp, expected) in [
            (0, 0, 100, 0),
            (99, 0, 100, 0),
            (100, 0, 100, 0),
            (101, 0, 100, 1),
            (150, 50, 100, 0),
            (151, 50, 100, 1),
            (150, 150, 100, 0),
            (150, 0, 0, 150),
            (i64::MAX, 0, 0, i64::MAX),
            (i64::MAX, i64::MAX, i64::MAX, 0),
        ] {
            assert_eq!(
                damage_overflow(
                    DamageAmount::new(calculated).unwrap(),
                    DamageAmount::new(absorbed).unwrap(),
                    Hp::new(hp).unwrap(),
                )
                .unwrap()
                .get(),
                expected,
            );
        }
        assert_eq!(
            damage_overflow(
                DamageAmount::new(10).unwrap(),
                DamageAmount::new(11).unwrap(),
                Hp::new(0).unwrap(),
            ),
            Err(NumericError::OutOfDomain),
        );
    }
}
