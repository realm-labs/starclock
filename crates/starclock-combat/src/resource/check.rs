use crate::{
    UnitId,
    actor::store::{TeamStateStore, UnitStore},
    catalog::action::ActionResourcePolicy,
};

pub(crate) fn can_pay(
    units: &UnitStore,
    teams: &TeamStateStore,
    actor: UnitId,
    policy: ActionResourcePolicy,
) -> bool {
    can_pay_with_policy(units, teams, actor, policy, policy.skill_point_payment())
}

pub(crate) fn can_pay_with_policy(
    units: &UnitStore,
    teams: &TeamStateStore,
    actor: UnitId,
    policy: ActionResourcePolicy,
    payment: crate::catalog::action::SkillPointPaymentPolicy,
) -> bool {
    units.get(actor).is_some_and(|unit| {
        let payable_sp = match payment {
            crate::catalog::action::SkillPointPaymentPolicy::TeamSkillPoints => {
                teams.get(unit.side).skill_points >= policy.skill_point_cost()
            }
            crate::catalog::action::SkillPointPaymentPolicy::Suppressed => true,
            crate::catalog::action::SkillPointPaymentPolicy::TeamResource(resource) => teams
                .get(unit.side)
                .keyed(resource)
                .is_some_and(|state| state.current >= policy.skill_point_cost()),
        };
        unit.life == crate::LifeState::Alive
            && unit.presence.is_active()
            && payable_sp
            && unit.current_energy >= policy.energy_cost()
    })
}
