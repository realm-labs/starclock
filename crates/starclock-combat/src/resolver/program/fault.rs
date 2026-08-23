use crate::{
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    rule::model::RuleEmission,
};

pub(super) const fn emission_code(emission: &RuleEmission) -> i64 {
    match emission {
        RuleEmission::SetSlot { .. } => 1,
        RuleEmission::AddSlot { .. } => 2,
        RuleEmission::TrueDamage { .. } => 3,
        RuleEmission::NonlethalTrueDamage { .. } => 17,
        RuleEmission::ReduceMaximumHp { .. } => 21,
        RuleEmission::DeductActionValue { .. } => 22,
        RuleEmission::UnboostedDamage { .. } => 20,
        RuleEmission::Shield { .. } => 4,
        RuleEmission::RemoveShield { .. } => 18,
        RuleEmission::Break { .. } => 5,
        RuleEmission::RemoveWeakness { .. } => 6,
        RuleEmission::CreateToughnessLayer { .. } => 7,
        RuleEmission::RemoveToughnessLayer { .. } => 8,
        RuleEmission::RemoveEffect { .. } => 9,
        RuleEmission::Cleanse { .. } => 19,
        RuleEmission::ModifyStateSlot { .. } => 10,
        RuleEmission::QueueAction { .. } => 11,
        RuleEmission::GrantExtraTurn { .. } => 12,
        RuleEmission::Summon { .. } => 13,
        RuleEmission::CreateCountdown { .. } => 14,
        RuleEmission::Informational { .. } => 15,
        RuleEmission::Replacement { .. } => 16,
        RuleEmission::InvokeNative { .. } => 17,
        _ => 0,
    }
}

pub(in crate::resolver) fn program_fault(context: u32, detail: i64) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x33a0 + context,
        Some(detail),
    )
}
