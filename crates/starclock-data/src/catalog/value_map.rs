//! Stable compact encodings used by converted combat definitions.

use crate::{
    catalog::CatalogLoadError,
    generated::{
        ability_kind::AbilityKind, crit_policy::CritPolicy, hit_damage_class::HitDamageClass,
        hit_element::HitElement, hit_scaling_stat::HitScalingStat,
        hit_target_group::HitTargetGroup, resource_delta_kind::ResourceDeltaKind,
        resource_kind::ResourceKind, resource_timing::ResourceTiming,
        retarget_policy::RetargetPolicy, target_pattern::TargetPattern,
    },
};
pub(super) fn ability_resource_kind(value: ResourceKind) -> u8 {
    use crate::generated::resource_kind::ResourceKind as V;
    match value {
        V::Energy => 0,
        V::SkillPoints => 1,
        V::Hp => 2,
        V::CharacterResource => 3,
        V::TeamResource => 4,
    }
}

pub(super) fn resource_delta_kind(value: ResourceDeltaKind) -> u8 {
    use crate::generated::resource_delta_kind::ResourceDeltaKind as V;
    match value {
        V::Spend => 0,
        V::Reserve => 1,
        V::Gain => 2,
    }
}

pub(super) fn resource_timing(value: ResourceTiming) -> u8 {
    use crate::generated::resource_timing::ResourceTiming as V;
    match value {
        V::CommandAccepted => 0,
        V::ActionStarted => 1,
        V::PerHit => 2,
        V::AbilityResolved => 3,
        V::ActionFinished => 4,
    }
}

pub(super) fn ability_kind(value: AbilityKind) -> u8 {
    use crate::generated::ability_kind::AbilityKind as V;
    match value {
        V::Basic => 0,
        V::Skill => 1,
        V::Ultimate => 2,
        V::Talent => 3,
        V::Technique => 4,
        V::EnhancedBasic => 5,
        V::EnhancedSkill => 6,
        V::FollowUp => 7,
        V::Counter => 8,
        V::Summon => 9,
        V::Memosprite => 10,
        V::Passive => 11,
        V::Entry => 12,
        V::Countdown => 13,
    }
}

pub(super) fn target_pattern(value: TargetPattern) -> u8 {
    use crate::generated::target_pattern::TargetPattern as V;
    match value {
        V::SingleTarget => 0,
        V::Blast => 1,
        V::Aoe => 2,
        V::Bounce => 3,
        V::Support => 4,
        V::Enhance => 5,
        V::None => 6,
        V::ContentDefined => 7,
    }
}

pub(super) fn retarget_policy(value: RetargetPolicy) -> u8 {
    use crate::generated::retarget_policy::RetargetPolicy as V;
    match value {
        V::Locked => 0,
        V::CancelRemaining => 1,
        V::RetargetSameSide => 2,
        V::RecomputeEachHit => 3,
    }
}

pub(super) fn hit_target_group(value: HitTargetGroup) -> u8 {
    use crate::generated::hit_target_group::HitTargetGroup as V;
    match value {
        V::Primary => 0,
        V::Adjacent => 1,
        V::Selected => 2,
        V::All => 3,
        V::BounceDraw => 4,
        V::SelfTarget => 5,
    }
}

pub(super) fn crit_policy(value: CritPolicy) -> u8 {
    use crate::generated::crit_policy::CritPolicy as V;
    match value {
        V::PerTarget => 0,
        V::Shared => 1,
        V::Never => 2,
    }
}

pub(super) fn hit_scaling_stat(
    value: HitScalingStat,
) -> starclock_combat::modifier::model::StatKind {
    use crate::generated::hit_scaling_stat::HitScalingStat as V;
    use starclock_combat::modifier::model::StatKind as O;
    match value {
        V::Atk => O::Atk,
        V::Hp => O::Hp,
        V::Def => O::Def,
    }
}

pub(super) fn hit_damage_class(
    value: HitDamageClass,
) -> Result<starclock_combat::formula::model::DamageClass, CatalogLoadError> {
    use crate::generated::hit_damage_class::HitDamageClass as V;
    use starclock_combat::formula::model::DamageClass as O;
    match value {
        V::Ordinary => Ok(O::Direct),
        V::Dot => Ok(O::Dot),
        V::Additional => Ok(O::Additional),
        V::Elation => Ok(O::Elation),
    }
}

pub(super) fn hit_element(value: HitElement) -> starclock_combat::formula::model::CombatElement {
    use crate::generated::hit_element::HitElement as V;
    use starclock_combat::formula::model::CombatElement as O;
    match value {
        V::Physical => O::Physical,
        V::Fire => O::Fire,
        V::Ice => O::Ice,
        V::Lightning => O::Lightning,
        V::Wind => O::Wind,
        V::Quantum => O::Quantum,
        V::Imaginary => O::Imaginary,
    }
}
