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
    units.get(actor).is_some_and(|unit| {
        unit.life == crate::LifeState::Alive
            && unit.presence == crate::PresenceState::Present
            && teams.get(unit.side).skill_points >= policy.skill_point_cost()
            && unit.current_energy >= policy.energy_cost()
    })
}
