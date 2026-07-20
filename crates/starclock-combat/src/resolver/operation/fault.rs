use crate::battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy};

pub(super) fn numeric_fault(context: u32, value: i64) -> BattleFault {
    BattleFault::new(
        FaultKind::Numeric,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3200 + context,
        Some(value),
    )
}

pub(super) fn invariant_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3280 + context,
        None,
    )
}
