use crate::rule::model::RuleEmission;

pub(super) const fn emission_current_target(emission: &RuleEmission) -> Option<crate::UnitId> {
    match emission {
        RuleEmission::SetSlot { current_target, .. }
        | RuleEmission::AddSlot { current_target, .. }
        | RuleEmission::Damage { current_target, .. }
        | RuleEmission::TrueDamage { current_target, .. }
        | RuleEmission::Heal { current_target, .. }
        | RuleEmission::Shield { current_target, .. }
        | RuleEmission::RemoveShield { current_target, .. }
        | RuleEmission::ConsumeHp { current_target, .. }
        | RuleEmission::ReduceToughness { current_target, .. }
        | RuleEmission::Break { current_target, .. }
        | RuleEmission::SuperBreak { current_target, .. }
        | RuleEmission::AddWeakness { current_target, .. }
        | RuleEmission::RemoveWeakness { current_target, .. }
        | RuleEmission::CreateToughnessLayer { current_target, .. }
        | RuleEmission::RemoveToughnessLayer { current_target, .. }
        | RuleEmission::ModifyResource { current_target, .. }
        | RuleEmission::ApplyEffect { current_target, .. }
        | RuleEmission::RemoveEffect { current_target, .. }
        | RuleEmission::Cleanse { current_target, .. }
        | RuleEmission::DetonateDot { current_target, .. }
        | RuleEmission::ModifyStateSlot { current_target, .. }
        | RuleEmission::AdvanceAction { current_target, .. }
        | RuleEmission::DelayAction { current_target, .. }
        | RuleEmission::QueueAction { current_target, .. }
        | RuleEmission::GrantExtraTurn { current_target, .. }
        | RuleEmission::Summon { current_target, .. }
        | RuleEmission::Despawn { current_target, .. }
        | RuleEmission::Transform { current_target, .. }
        | RuleEmission::ReplaceAbility { current_target, .. }
        | RuleEmission::ChangePresence { current_target, .. }
        | RuleEmission::CreateCountdown { current_target, .. }
        | RuleEmission::Informational { current_target, .. }
        | RuleEmission::Replacement { current_target, .. }
        | RuleEmission::InvokeNative { current_target, .. } => *current_target,
    }
}
