//! Closed generated-enum mappings for authored generic effects.

use crate::generated::{
    combat_element::CombatElement as CombatElementCombatElement,
    dispel_category::DispelCategory as DispelCategoryDispelCategory,
    duration_clock::DurationClock as DurationClockDurationClock,
    effect_category::EffectCategory as EffectCategoryEffectCategory,
    effect_stack_policy::EffectStackPolicy as EffectStackPolicyEffectStackPolicy,
    effect_teardown_policy::EffectTeardownPolicy as EffectTeardownPolicyEffectTeardownPolicy,
    effect_tick_phase::EffectTickPhase as EffectTickPhaseEffectTickPhase,
    snapshot_policy::SnapshotPolicy,
};
use starclock_combat::formula::model::CombatElement;
use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectSnapshotPolicy, EffectStackPolicy,
    EffectTeardownPolicy, EffectTickPhase,
};

pub(super) fn lower_element(value: CombatElementCombatElement) -> CombatElement {
    use crate::generated::combat_element::CombatElement as V;
    match value {
        V::Physical => CombatElement::Physical,
        V::Fire => CombatElement::Fire,
        V::Ice => CombatElement::Ice,
        V::Lightning => CombatElement::Lightning,
        V::Wind => CombatElement::Wind,
        V::Quantum => CombatElement::Quantum,
        V::Imaginary => CombatElement::Imaginary,
    }
}

pub(super) fn lower_effect_category(value: EffectCategoryEffectCategory) -> EffectCategory {
    use crate::generated::effect_category::EffectCategory as V;
    match value {
        V::Buff => EffectCategory::Buff,
        V::Debuff => EffectCategory::Debuff,
        V::Control => EffectCategory::Control,
        V::Dot => EffectCategory::Dot,
        V::Mark => EffectCategory::Mark,
        V::Field => EffectCategory::Field,
        V::Shield => EffectCategory::Shield,
        V::NeutralState => EffectCategory::NeutralState,
    }
}

pub(super) fn lower_dispel(value: DispelCategoryDispelCategory) -> DispelCategory {
    use crate::generated::dispel_category::DispelCategory as V;
    match value {
        V::DispellableBuff => DispelCategory::DispellableBuff,
        V::DispellableDebuff => DispelCategory::DispellableDebuff,
        V::CleanseableControl => DispelCategory::CleanseableControl,
        V::NonDispellable => DispelCategory::NonDispellable,
    }
}

pub(super) fn lower_duration_clock(value: DurationClockDurationClock) -> DurationClock {
    use crate::generated::duration_clock::DurationClock as V;
    match value {
        V::Permanent => DurationClock::Permanent,
        V::OwnerTurnStart => DurationClock::OwnerTurnStart,
        V::OwnerTurnEnd => DurationClock::OwnerTurnEnd,
        V::TargetTurnStart => DurationClock::TargetTurnStart,
        V::TargetTurnEnd => DurationClock::TargetTurnEnd,
        V::ActionEnd => DurationClock::ActionEnd,
        V::WaveEnd => DurationClock::WaveEnd,
        V::BattleEnd => DurationClock::BattleEnd,
    }
}

pub(super) fn lower_tick_phase(value: EffectTickPhaseEffectTickPhase) -> EffectTickPhase {
    use crate::generated::effect_tick_phase::EffectTickPhase as V;
    match value {
        V::None => EffectTickPhase::None,
        V::TurnStart => EffectTickPhase::TurnStart,
        V::TurnEnd => EffectTickPhase::TurnEnd,
        V::ActionStart => EffectTickPhase::ActionStart,
        V::ActionEnd => EffectTickPhase::ActionEnd,
        V::AfterEvent => EffectTickPhase::AfterEvent,
    }
}

pub(super) fn lower_stack_policy(value: EffectStackPolicyEffectStackPolicy) -> EffectStackPolicy {
    use crate::generated::effect_stack_policy::EffectStackPolicy as V;
    match value {
        V::Replace => EffectStackPolicy::Replace,
        V::Refresh => EffectStackPolicy::Refresh,
        V::RefreshAndAddStacks => EffectStackPolicy::RefreshAndAddStacks,
        V::StrongestWins => EffectStackPolicy::StrongestWins,
        V::IndependentBySource => EffectStackPolicy::IndependentBySource,
        V::IndependentInstances => EffectStackPolicy::IndependentInstances,
        V::UniqueGlobal => EffectStackPolicy::UniqueGlobal,
        V::UniquePerSource => EffectStackPolicy::UniquePerSource,
    }
}

pub(super) fn lower_snapshot_policy(value: SnapshotPolicy) -> EffectSnapshotPolicy {
    use crate::generated::snapshot_policy::SnapshotPolicy as V;
    match value {
        V::Dynamic => EffectSnapshotPolicy::Dynamic,
        V::OnApplication => EffectSnapshotPolicy::OnApplication,
        V::OnActionStart => EffectSnapshotPolicy::OnActionStart,
        V::OnPhaseStart => EffectSnapshotPolicy::OnPhaseStart,
        V::OnHitStart => EffectSnapshotPolicy::OnHitStart,
        V::SourceSnapshotTargetDynamic => EffectSnapshotPolicy::SourceSnapshotTargetDynamic,
        V::SourceDynamicTargetSnapshot => EffectSnapshotPolicy::SourceDynamicTargetSnapshot,
        V::RecomputeOnStackChange => EffectSnapshotPolicy::RecomputeOnStackChange,
        V::ExplicitFields => EffectSnapshotPolicy::ExplicitFields,
    }
}

pub(super) fn lower_teardown(
    value: EffectTeardownPolicyEffectTeardownPolicy,
) -> EffectTeardownPolicy {
    use crate::generated::effect_teardown_policy::EffectTeardownPolicy as V;
    match value {
        V::RemoveWithOwner => EffectTeardownPolicy::RemoveWithOwner,
        V::TransferToTeam => EffectTeardownPolicy::TransferToTeam,
        V::FreezeSnapshot => EffectTeardownPolicy::FreezeSnapshot,
        V::PersistByScope => EffectTeardownPolicy::PersistByScope,
        V::ExplicitRule => EffectTeardownPolicy::ExplicitRule,
    }
}
