//! Generated-row-free authored hit formula bindings.

#[derive(Debug)]
pub(crate) struct AbilityHitPlanDefinition {
    pub(crate) phase_sequence: u16,
    pub(crate) hit_plan_id: u32,
    pub(crate) damage_parameter_key: Option<Box<str>>,
    pub(crate) damage_scaling_stat: Option<starclock_combat::modifier::model::StatKind>,
    pub(crate) damage_class: Option<starclock_combat::formula::model::DamageClass>,
    pub(crate) element: Option<starclock_combat::formula::model::CombatElement>,
    pub(crate) base_toughness: Option<starclock_combat::Scalar>,
}
