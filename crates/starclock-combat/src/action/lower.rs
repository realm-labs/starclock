use crate::{
    action::model::{ActionOrigin, ActionPhasePlan, ActionPlan, HitPlan},
    catalog::CombatCatalog,
    id::{AbilityId, ActionId, HitId, PhaseId, TimelineActorId, UnitId},
};

pub(crate) trait ActionIdentityAllocator {
    fn action(&mut self) -> ActionId;
    fn phase(&mut self) -> PhaseId;
    fn hit(&mut self) -> HitId;
}

pub(crate) fn lower_normal_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    timeline_actor: TimelineActorId,
    ability: AbilityId,
) -> Option<ActionPlan> {
    catalog
        .ability(ability)?
        .is_single_hit_action()
        .then(|| ActionPlan {
            id: allocator.action(),
            actor,
            ability,
            origin: ActionOrigin::NormalTurn,
            normal_turn: Some(timeline_actor),
            phases: vec![ActionPhasePlan {
                id: allocator.phase(),
                hits: vec![HitPlan {
                    id: allocator.hit(),
                }]
                .into_boxed_slice(),
            }]
            .into_boxed_slice(),
        })
}
