use crate::{
    UnitId,
    actor::store::{TeamStateStore, UnitStore},
    catalog::action::ActionResourcePolicy,
};

pub(crate) fn can_pay(
    units: &UnitStore,
    teams: &TeamStateStore,
    actor: UnitId,
    policy: &ActionResourcePolicy,
) -> bool {
    can_pay_with_policy(units, teams, actor, policy, policy.skill_point_payment())
}

pub(crate) fn can_pay_with_policy(
    units: &UnitStore,
    teams: &TeamStateStore,
    actor: UnitId,
    policy: &ActionResourcePolicy,
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
        let suppresses_costs =
            payment == crate::catalog::action::SkillPointPaymentPolicy::Suppressed;
        let payable_character_resources = suppresses_costs
            || policy.character_resource_costs().iter().all(|cost| {
                unit.resource(cost.stable_key())
                    .is_some_and(|state| state.current >= cost.amount())
            });
        let payable_team_resources = suppresses_costs
            || policy.team_resource_costs().iter().all(|cost| {
                teams
                    .get(unit.side)
                    .keyed_by_name(cost.stable_key())
                    .is_some_and(|state| state.current >= cost.amount())
            });
        unit.life == crate::LifeState::Alive
            && unit.presence.is_active()
            && payable_sp
            && (suppresses_costs || unit.current_energy >= policy.energy_cost())
            && payable_character_resources
            && payable_team_resources
    })
}
