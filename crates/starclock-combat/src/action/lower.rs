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

#[derive(Clone, Copy)]
pub(crate) struct TimelineActionContext {
    pub(crate) actor: UnitId,
    pub(crate) owner: UnitId,
    pub(crate) timeline_actor: TimelineActorId,
    pub(crate) origin: ActionOrigin,
}

#[derive(Clone, Copy)]
struct ActionContext {
    actor: UnitId,
    owner: UnitId,
    origin: ActionOrigin,
    timeline_actor: Option<TimelineActorId>,
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
        ActionContext {
            actor,
            owner: actor,
            origin: ActionOrigin::NormalTurn,
            timeline_actor: Some(timeline_actor),
        },
        ability,
        targets,
        None,
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
        ActionContext {
            actor,
            owner: actor,
            origin: ActionOrigin::UltimateInterrupt,
            timeline_actor: None,
        },
        ability,
        targets,
        None,
    )
}

pub(crate) struct QueuedActionContext {
    pub(crate) actor: UnitId,
    pub(crate) owner: UnitId,
    pub(crate) origin: ActionOrigin,
    pub(crate) payment: Option<crate::catalog::action::SkillPointPaymentPolicy>,
}

pub(crate) fn lower_queued_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    context: QueuedActionContext,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    lower_action(
        catalog,
        allocator,
        ActionContext {
            actor: context.actor,
            owner: context.owner,
            origin: context.origin,
            timeline_actor: None,
        },
        ability,
        targets,
        context.payment,
    )
}

pub(crate) fn lower_timeline_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    context: TimelineActionContext,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    lower_action(
        catalog,
        allocator,
        ActionContext {
            actor: context.actor,
            owner: context.owner,
            origin: context.origin,
            timeline_actor: Some(context.timeline_actor),
        },
        ability,
        targets,
        None,
    )
}

fn lower_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    context: ActionContext,
    ability: AbilityId,
    targets: TargetCommitment,
    payment: Option<crate::catalog::action::SkillPointPaymentPolicy>,
) -> Option<ActionPlan> {
    let definition = catalog.ability(ability)?;
    let action = definition.action()?;
    let compatible = match context.origin {
        ActionOrigin::NormalTurn => action.kind().is_normal_turn(),
        ActionOrigin::UltimateInterrupt if action.kind() != AbilityKind::Ultimate => return None,
        ActionOrigin::UltimateInterrupt => true,
        ActionOrigin::FollowUp => action.kind() == AbilityKind::FollowUp,
        ActionOrigin::Counter => action.kind() == AbilityKind::Counter,
        ActionOrigin::ExtraTurn => action.kind() == AbilityKind::ExtraTurn,
        ActionOrigin::ExtraAction => action.kind() == AbilityKind::ExtraAction,
        ActionOrigin::Forced => {
            action.kind() == AbilityKind::ExtraAction
                || (action.kind() == AbilityKind::Skill
                    && action
                        .tags()
                        .contains(crate::catalog::action::AbilityTag::ElationSkill))
        }
        ActionOrigin::DelayedAction => action.kind() == AbilityKind::DelayedAction,
        ActionOrigin::SummonAction => action.kind() == AbilityKind::Summon,
        ActionOrigin::MemospriteAction => action.kind() == AbilityKind::Memosprite,
        ActionOrigin::Countdown => action.kind() == AbilityKind::Countdown,
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
            target_group: hit.target_group(),
            damage_share: hit.damage_share(),
            toughness_share: hit.toughness_share(),
            crit_policy: hit.crit_policy(),
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
        actor: context.actor,
        owner: context.owner,
        ability,
        origin: context.origin,
        tags: action.tags(),
        normal_turn: context.timeline_actor,
        selector,
        targets,
        resources: payment.map_or_else(
            || action.resources().clone(),
            |payment| action.resources().clone().with_skill_point_payment(payment),
        ),
        programs: definition.programs().into(),
        phases: vec![ActionPhasePlan { id: phase_id, hits }].into_boxed_slice(),
    })
}
