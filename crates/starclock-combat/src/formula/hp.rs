//! HP-consumption policies independent from ordinary damage.

use crate::{Hp, NumericError};

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
