use crate::{
    action::model::{ActionOrigin, ActionPhasePlan, ActionPlan, HitPlan, OperationPlan},
    catalog::{CombatCatalog, action::AbilityKind},
    id::{AbilityId, ActionId, HitId, OperationId, PhaseId, TimelineActorId, UnitId},
    target::model::TargetCommitment,
};

pub(crate) trait ActionIdentityAllocator {
    fn action(&mut self) -> ActionId;
    fn phase(&mut self) -> PhaseId;
    fn hit(&mut self) -> HitId;
    fn operation(&mut self) -> OperationId;
}

pub(crate) fn lower_normal_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    timeline_actor: TimelineActorId,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    lower_action(
        catalog,
        allocator,
        actor,
        ability,
        ActionOrigin::NormalTurn,
        Some(timeline_actor),
        targets,
    )
}

pub(crate) fn lower_interrupt_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    lower_action(
        catalog,
        allocator,
        actor,
        ability,
        ActionOrigin::UltimateInterrupt,
        None,
        targets,
    )
}

pub(crate) fn lower_queued_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    ability: AbilityId,
    origin: ActionOrigin,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    lower_action(catalog, allocator, actor, ability, origin, None, targets)
}

fn lower_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    ability: AbilityId,
    origin: ActionOrigin,
    normal_turn: Option<TimelineActorId>,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    let definition = catalog.ability(ability)?;
    let action = definition.action()?;
    let compatible = match origin {
        ActionOrigin::NormalTurn => action.kind().is_normal_turn(),
        ActionOrigin::UltimateInterrupt if action.kind() != AbilityKind::Ultimate => return None,
        ActionOrigin::UltimateInterrupt => true,
        ActionOrigin::FollowUp => action.kind() == AbilityKind::FollowUp,
        ActionOrigin::Counter => action.kind() == AbilityKind::Counter,
        ActionOrigin::ExtraTurn => action.kind() == AbilityKind::ExtraTurn,
        ActionOrigin::ExtraAction | ActionOrigin::Forced => {
            action.kind() == AbilityKind::ExtraAction
        }
        ActionOrigin::DelayedAction => action.kind() == AbilityKind::DelayedAction,
    };
    compatible.then_some(())?;
    let selector = catalog.selector(definition.selector())?.unit_targets()?;
    (selector == targets.selector && action.invalidation() == targets.invalidation).then_some(())?;

    let action_id = allocator.action();
    let phase_id = allocator.phase();
    let hits = action
        .hits()
        .iter()
        .map(|hit| HitPlan {
            id: allocator.hit(),
            invalidation: action.invalidation(),
            operations: hit
                .operations()
                .iter()
                .cloned()
                .map(|definition| OperationPlan {
                    id: allocator.operation(),
                    definition,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    Some(ActionPlan {
        id: action_id,
        actor,
        ability,
        origin,
        normal_turn,
        selector,
        targets,
        resources: action.resources(),
        phases: vec![ActionPhasePlan { id: phase_id, hits }].into_boxed_slice(),
    })
}
