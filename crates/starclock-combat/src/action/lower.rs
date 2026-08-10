use crate::{
    action::model::{ActionOrigin, ActionPhasePlan, ActionPlan, HitPlan, OperationPlan},
    catalog::{
        CombatCatalog,
        action::{AbilityKind, SkillPointPaymentPolicy, TargetRelation},
    },
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
    context: TimelineActionContext,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    if !matches!(
        context.origin,
        ActionOrigin::NormalTurn | ActionOrigin::ExtraTurn
    ) {
        return None;
    }
    let mut plan = lower_action(
        catalog,
        allocator,
        ActionContext {
            actor: context.actor,
            owner: context.owner,
            origin: ActionOrigin::NormalTurn,
            timeline_actor: Some(context.timeline_actor),
        },
        ability,
        targets,
        None,
    )?;
    if context.origin == ActionOrigin::ExtraTurn {
        plan.origin = context.origin;
        plan.normal_turn = None;
    }
    Some(plan)
}

pub(crate) fn lower_ultimate_action(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    owner: UnitId,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    lower_action(
        catalog,
        allocator,
        ActionContext {
            actor,
            owner,
            origin: ActionOrigin::UltimateInterrupt,
            timeline_actor: None,
        },
        ability,
        targets,
        None,
    )
}

pub(crate) fn lower_segmented_ultimate_header(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    actor: UnitId,
    owner: UnitId,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    let definition = catalog.ability(ability)?;
    let action = definition.action()?;
    (action.kind() == AbilityKind::Ultimate && action.segmented_flow().is_some()).then_some(())?;
    let selector = catalog.selector(definition.selector())?.unit_targets()?;
    (selector == targets.selector && action.invalidation() == targets.invalidation).then_some(())?;
    Some(ActionPlan {
        id: allocator.action(),
        actor,
        owner,
        ability,
        origin: ActionOrigin::UltimateInterrupt,
        tags: action.tags(),
        normal_turn: None,
        selector,
        targets,
        resources: action.resources().clone(),
        programs: definition.programs().into(),
        phases: Box::new([]),
    })
}

pub(crate) fn restore_segmented_ultimate_header(
    catalog: &CombatCatalog,
    action_id: ActionId,
    actor: UnitId,
    owner: UnitId,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    let definition = catalog.ability(ability)?;
    let action = definition.action()?;
    (action.kind() == AbilityKind::Ultimate && action.segmented_flow().is_some()).then_some(())?;
    let selector = catalog.selector(definition.selector())?.unit_targets()?;
    Some(ActionPlan {
        id: action_id,
        actor,
        owner,
        ability,
        origin: ActionOrigin::UltimateInterrupt,
        tags: action.tags(),
        normal_turn: None,
        selector,
        targets,
        resources: action.resources().clone(),
        programs: definition.programs().into(),
        phases: Box::new([]),
    })
}

pub(crate) fn lower_action_segment(
    catalog: &CombatCatalog,
    allocator: &mut impl ActionIdentityAllocator,
    action_id: ActionId,
    actor: UnitId,
    owner: UnitId,
    ability: AbilityId,
    targets: TargetCommitment,
) -> Option<ActionPlan> {
    let definition = catalog.ability(ability)?;
    let action = definition.action()?;
    let selector = catalog.selector(definition.selector())?.unit_targets()?;
    (selector == targets.selector && action.invalidation() == targets.invalidation).then_some(())?;
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
        actor,
        owner,
        ability,
        origin: ActionOrigin::UltimateInterrupt,
        tags: action.tags(),
        normal_turn: None,
        selector,
        targets,
        resources: action.resources().clone(),
        programs: definition.programs().into(),
        phases: vec![ActionPhasePlan { id: phase_id, hits }].into_boxed_slice(),
    })
}

pub(crate) struct QueuedActionContext {
    pub(crate) actor: UnitId,
    pub(crate) owner: UnitId,
    pub(crate) origin: ActionOrigin,
    pub(crate) payment: Option<SkillPointPaymentPolicy>,
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

pub(crate) fn lower_forced_basic_action(
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
            origin: ActionOrigin::Forced,
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
    payment: Option<SkillPointPaymentPolicy>,
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
            action.kind() == AbilityKind::Basic
                || action.kind() == AbilityKind::ExtraAction
                || (action.kind() == AbilityKind::Skill && action.tags().supports_forced_skill())
        }
        ActionOrigin::DelayedAction => action.kind() == AbilityKind::DelayedAction,
        ActionOrigin::SummonAction => action.kind() == AbilityKind::Summon,
        ActionOrigin::MemospriteAction => action.kind() == AbilityKind::Memosprite,
        ActionOrigin::Countdown => action.kind() == AbilityKind::Countdown,
    };
    compatible.then_some(())?;
    let selector = catalog.selector(definition.selector())?.unit_targets()?;
    let forced_basic_target_override = context.origin == ActionOrigin::Forced
        && action.kind() == AbilityKind::Basic
        && selector.pattern() == targets.selector.pattern()
        && targets.selector.relation() == TargetRelation::Allied;
    ((selector == targets.selector || forced_basic_target_override)
        && action.invalidation() == targets.invalidation)
        .then_some(())?;
    let committed_selector = if forced_basic_target_override {
        targets.selector
    } else {
        selector
    };

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
        selector: committed_selector,
        targets,
        resources: payment.map_or_else(
            || action.resources().clone(),
            |payment| match payment {
                SkillPointPaymentPolicy::Suppressed => {
                    action.resources().clone().with_costs_suppressed()
                }
                payment => action.resources().clone().with_skill_point_payment(payment),
            },
        ),
        programs: definition.programs().into(),
        phases: vec![ActionPhasePlan { id: phase_id, hits }].into_boxed_slice(),
    })
}
