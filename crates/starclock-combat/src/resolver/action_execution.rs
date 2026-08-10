use crate::{
    ActionOrigin, EffectTickPhase, UnitId,
    action::model::{ActionPlan, HitPlan, OperationPlan},
    battle::fault::BattleFault,
    catalog::{
        CombatCatalog,
        action::{AbilityKind, AbilityProgramTiming, HitOperationDefinition, ReactionBoundary},
        encounter::WaveTransitionPolicy,
    },
    event::{
        cause::{Cause, CauseActor},
        model::{ActionEventData, BattleEventKind, HitEventData, PhaseEventData},
    },
    id::{CommandId, EventId, SourceDefinitionId},
    modifier::model::SnapshotPolicy,
    operation::{
        AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp, DamageOp, DetonateDotsOp,
        EncounterLifecycleOp, EnemyPhaseOp, HealOp, HitOperationScratch, ModifyStateSlotOp,
        ModifyTeamResourceOp, Operation, QueueActionOp, ReduceToughnessOp, RemoveEffectsOp,
        ReviveOp, ShieldOp, SummonLinkedOp, SuperBreakOp, TransformOp, UnitLifecycleOp,
    },
    rule::model::SlotResetPoint,
};

use super::operation::execute_operation;
use super::{
    action::{
        ReactionWork, apply_resource_costs, apply_resource_gains, dequeue_reaction,
        project_hit_targets, resolve_scaling_damage, run_programs_at,
    },
    effect_boundary, lifecycle, modifier_snapshot, operation, rule, settle,
    transaction::{Transaction, action_fault},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExecutionStage {
    StartAction,
    StartSegment,
    StartPhase,
    ExecuteHit,
    DrainAfterHit,
    SettleAfterHit,
    FinishPhase,
    DrainAfterPhase,
    SettleAfterPhase,
    FinishAction,
}

struct ExecutionFrame {
    root: CommandId,
    parent: EventId,
    plan: ActionPlan,
    stage: ExecutionStage,
    phase_index: usize,
    hit_index: usize,
    boundary_cause: Option<Cause>,
    complete_envelope: bool,
    run_segment_programs: bool,
}

impl ExecutionFrame {
    fn new(root: CommandId, parent: EventId, plan: ActionPlan) -> Self {
        Self {
            root,
            parent,
            plan,
            stage: ExecutionStage::StartAction,
            phase_index: 0,
            hit_index: 0,
            boundary_cause: None,
            complete_envelope: true,
            run_segment_programs: true,
        }
    }

    fn segment(root: CommandId, parent: EventId, plan: ActionPlan) -> Self {
        Self {
            root,
            parent,
            plan,
            stage: ExecutionStage::StartSegment,
            phase_index: 0,
            hit_index: 0,
            boundary_cause: None,
            complete_envelope: false,
            run_segment_programs: true,
        }
    }

    fn parent_segment(root: CommandId, parent: EventId, plan: ActionPlan) -> Self {
        Self {
            root,
            parent,
            plan,
            stage: ExecutionStage::StartPhase,
            phase_index: 0,
            hit_index: 0,
            boundary_cause: None,
            complete_envelope: false,
            run_segment_programs: false,
        }
    }

    fn base_cause(&self) -> Result<Cause, BattleFault> {
        let source =
            SourceDefinitionId::new(self.plan.ability.get()).ok_or_else(|| action_fault(7))?;
        Ok(Cause::for_action(
            self.root,
            self.plan.id,
            self.plan.owner,
            CauseActor::Unit(self.plan.actor),
            source,
        )
        .with_primary_target(self.plan.targets.primary)
        .with_applier(self.plan.owner))
    }
}

pub(super) struct ResolvedAction {
    pub(super) parent: EventId,
    pub(super) plan: ActionPlan,
}

pub(super) fn execute_action_plan(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    plan: ActionPlan,
) -> Result<ResolvedAction, BattleFault> {
    let mut frames = vec![ExecutionFrame::new(root, parent, plan)];
    run_frames(catalog, txn, &mut frames)
}

fn run_frames(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frames: &mut Vec<ExecutionFrame>,
) -> Result<ResolvedAction, BattleFault> {
    loop {
        let index = frames
            .len()
            .checked_sub(1)
            .ok_or_else(|| action_fault(79))?;
        match frames[index].stage {
            ExecutionStage::StartAction => start_action(catalog, txn, &mut frames[index])?,
            ExecutionStage::StartSegment => start_segment(catalog, txn, &mut frames[index])?,
            ExecutionStage::StartPhase => start_phase(catalog, txn, &mut frames[index])?,
            ExecutionStage::ExecuteHit => execute_hit(catalog, txn, &mut frames[index])?,
            ExecutionStage::DrainAfterHit => {
                if push_reaction(catalog, txn, frames, ReactionBoundary::AfterHit)? {
                    continue;
                }
                frames[index].stage = ExecutionStage::SettleAfterHit;
            }
            ExecutionStage::SettleAfterHit => settle_after_hit(catalog, txn, &mut frames[index])?,
            ExecutionStage::FinishPhase => finish_phase(catalog, txn, &mut frames[index])?,
            ExecutionStage::DrainAfterPhase => {
                if push_reaction(catalog, txn, frames, ReactionBoundary::AfterPhase)? {
                    continue;
                }
                frames[index].stage = ExecutionStage::SettleAfterPhase;
            }
            ExecutionStage::SettleAfterPhase => {
                settle_after_phase(catalog, txn, &mut frames[index])?
            }
            ExecutionStage::FinishAction => {
                finish_action(catalog, txn, &mut frames[index])?;
                let completed = frames.pop().ok_or_else(|| action_fault(79))?;
                if let Some(parent_frame) = frames.last_mut() {
                    let cause = completed.base_cause()?;
                    parent_frame.parent = operation::settle_effects_at_action_end(
                        catalog,
                        txn,
                        cause,
                        completed.parent,
                    )?;
                } else {
                    return Ok(ResolvedAction {
                        parent: completed.parent,
                        plan: completed.plan,
                    });
                }
            }
        }
    }
}

pub(super) fn start_action_envelope(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    plan: ActionPlan,
) -> Result<ResolvedAction, BattleFault> {
    let mut frame = ExecutionFrame::new(root, parent, plan);
    start_action(catalog, txn, &mut frame)?;
    Ok(ResolvedAction {
        parent: frame.parent,
        plan: frame.plan,
    })
}

pub(super) fn execute_action_segment(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    plan: ActionPlan,
) -> Result<ResolvedAction, BattleFault> {
    let mut frames = vec![ExecutionFrame::segment(root, parent, plan)];
    run_frames(catalog, txn, &mut frames)
}

pub(super) fn execute_parent_action_segment(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    plan: ActionPlan,
) -> Result<ResolvedAction, BattleFault> {
    let mut frames = vec![ExecutionFrame::parent_segment(root, parent, plan)];
    run_frames(catalog, txn, &mut frames)
}

pub(super) fn finish_action_envelope(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    parent: EventId,
    plan: ActionPlan,
) -> Result<ResolvedAction, BattleFault> {
    let mut frame = ExecutionFrame::new(root, parent, plan);
    frame.stage = ExecutionStage::FinishAction;
    finish_action(catalog, txn, &mut frame)?;
    Ok(ResolvedAction {
        parent: frame.parent,
        plan: frame.plan,
    })
}

fn push_reaction(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frames: &mut Vec<ExecutionFrame>,
    boundary: ReactionBoundary,
) -> Result<bool, BattleFault> {
    let Some(work) = dequeue_reaction(catalog, txn, boundary)? else {
        return Ok(false);
    };
    match work {
        ReactionWork::Cancelled(event) => {
            frames.last_mut().ok_or_else(|| action_fault(79))?.parent = event;
        }
        ReactionWork::Execute { queued, plan } => {
            frames.push(ExecutionFrame::new(queued.root, queued.parent, *plan));
        }
    }
    Ok(true)
}

fn start_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let plan = &frame.plan;
    debug_assert_eq!(
        plan.normal_turn.is_some(),
        plan.origin.owns_timeline_turn()
            || plan.origin == ActionOrigin::Forced
                && catalog
                    .ability(plan.ability)
                    .and_then(|ability| ability.action())
                    .is_some_and(|action| action.kind() == AbilityKind::Basic)
    );
    let _selector = plan.selector;
    let base = frame.base_cause()?;
    let mut parent = txn.emit(
        base.with_parent(frame.parent),
        BattleEventKind::Action(ActionEventData::Declared {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
            tags: plan.tags,
        }),
    );
    parent = run_programs_at(
        catalog,
        txn,
        base,
        parent,
        plan,
        AbilityProgramTiming::Entry,
        None,
        &mut HitOperationScratch::default(),
    )?;
    parent = apply_resource_costs(txn, base, parent, plan)?;
    modifier_snapshot::refresh(catalog, txn, SnapshotPolicy::OnActionStart)?;
    txn.reset_rule_slots(SlotResetPoint::ActionStart, Some(plan.actor));
    parent = effect_boundary::tick(
        catalog,
        txn,
        base,
        parent,
        EffectTickPhase::ActionStart,
        plan.actor,
    )?;
    parent = txn.emit(
        base.with_parent(parent),
        BattleEventKind::Action(ActionEventData::Started {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
            tags: plan.tags,
        }),
    );
    parent = rule::dispatch_pending_after_events(catalog, txn, parent)?;
    parent = run_programs_at(
        catalog,
        txn,
        base,
        parent,
        plan,
        AbilityProgramTiming::BeforeHits,
        None,
        &mut HitOperationScratch::default(),
    )?;
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, parent)?;
    frame.stage = ExecutionStage::StartPhase;
    Ok(())
}

fn start_segment(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let base = frame.base_cause()?;
    frame.parent = run_programs_at(
        catalog,
        txn,
        base,
        frame.parent,
        &frame.plan,
        AbilityProgramTiming::Entry,
        None,
        &mut HitOperationScratch::default(),
    )?;
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.parent = run_programs_at(
        catalog,
        txn,
        base,
        frame.parent,
        &frame.plan,
        AbilityProgramTiming::BeforeHits,
        None,
        &mut HitOperationScratch::default(),
    )?;
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.stage = ExecutionStage::StartPhase;
    Ok(())
}

fn start_phase(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let Some(phase) = frame.plan.phases.get(frame.phase_index) else {
        frame.stage = ExecutionStage::FinishAction;
        return Ok(());
    };
    let phase_cause = frame.base_cause()?.with_phase(phase.id);
    modifier_snapshot::refresh(catalog, txn, SnapshotPolicy::OnPhaseStart)?;
    frame.parent = txn.emit(
        phase_cause.with_parent(frame.parent),
        BattleEventKind::Phase(PhaseEventData::Started {
            action: frame.plan.id,
            phase: phase.id,
        }),
    );
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.hit_index = 0;
    frame.stage = ExecutionStage::ExecuteHit;
    Ok(())
}

fn execute_hit(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let phase = frame
        .plan
        .phases
        .get(frame.phase_index)
        .ok_or_else(|| action_fault(80))?;
    let Some(hit) = phase.hits.get(frame.hit_index).cloned() else {
        frame.stage = ExecutionStage::FinishPhase;
        return Ok(());
    };
    txn.reset_rule_slots(SlotResetPoint::HitStart, Some(frame.plan.actor));
    let mut scratch = HitOperationScratch::default();
    debug_assert_eq!(hit.invalidation, frame.plan.targets.invalidation);
    let selected = txn.resolve_hit_targets(frame.plan.actor, &mut frame.plan.targets)?;
    let targets = project_hit_targets(
        txn,
        frame.plan.actor,
        &frame.plan.targets,
        hit.target_group,
        selected,
    )?;
    modifier_snapshot::refresh(catalog, txn, SnapshotPolicy::OnHitStart)?;
    let hit_cause = frame
        .base_cause()?
        .with_phase(phase.id)
        .with_hit(hit.id)
        .with_primary_target(frame.plan.targets.primary);
    frame.parent = txn.emit(
        hit_cause.with_parent(frame.parent),
        BattleEventKind::Hit(HitEventData::Started {
            action: frame.plan.id,
            phase: phase.id,
            hit: hit.id,
            targets: targets.clone(),
        }),
    );
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.parent = run_programs_at(
        catalog,
        txn,
        hit_cause,
        frame.parent,
        &frame.plan,
        AbilityProgramTiming::Hits,
        Some(&hit),
        &mut scratch,
    )?;
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    for operation_plan in &hit.operations {
        let request = lower_operation(catalog, txn, &frame.plan, &targets, &hit, operation_plan)?;
        frame.parent =
            execute_operation(catalog, txn, hit_cause, frame.parent, request, &mut scratch)?;
        frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    }
    txn.increment_entanglement_for_hit(&targets)?;
    frame.parent = txn.emit(
        hit_cause.with_parent(frame.parent),
        BattleEventKind::Hit(HitEventData::Ended {
            action: frame.plan.id,
            phase: phase.id,
            hit: hit.id,
            targets,
        }),
    );
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.boundary_cause = Some(hit_cause);
    frame.stage = ExecutionStage::DrainAfterHit;
    Ok(())
}

fn lower_operation(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    plan: &ActionPlan,
    targets: &[UnitId],
    hit: &HitPlan,
    operation_plan: &OperationPlan,
) -> Result<Operation, BattleFault> {
    let targets = targets.to_vec().into_boxed_slice();
    let operation = match &operation_plan.definition {
        HitOperationDefinition::ScalingDamage(definition) => Operation::Damage(DamageOp {
            id: operation_plan.id,
            targets,
            formula: resolve_scaling_damage(catalog, txn, plan.owner, *definition)?,
            element: Some(definition.element()),
            crit_policy: hit.crit_policy,
            apply_source_modifiers: true,
            ultimate_semantics: false,
            minimum_hp: 0,
        }),
        HitOperationDefinition::Damage(formula) => Operation::Damage(DamageOp {
            id: operation_plan.id,
            targets,
            formula: *formula,
            element: None,
            crit_policy: hit.crit_policy,
            apply_source_modifiers: true,
            ultimate_semantics: false,
            minimum_hp: 0,
        }),
        HitOperationDefinition::Heal(formula) => Operation::Heal(HealOp {
            id: operation_plan.id,
            targets,
            formula: *formula,
            apply_formula_modifiers: true,
        }),
        HitOperationDefinition::Shield(formula) => Operation::Shield(ShieldOp {
            id: operation_plan.id,
            targets,
            formula: *formula,
            source_effect: None,
        }),
        HitOperationDefinition::ConsumeHp(definition) => Operation::ConsumeHp(ConsumeHpOp {
            id: operation_plan.id,
            targets,
            definition: *definition,
        }),
        HitOperationDefinition::AddWeakness(definition) => Operation::AddWeakness(AddWeaknessOp {
            id: operation_plan.id,
            targets,
            definition: *definition,
        }),
        HitOperationDefinition::ReduceToughness(definition) => {
            Operation::ReduceToughness(ReduceToughnessOp {
                id: operation_plan.id,
                targets,
                definition: *definition,
            })
        }
        HitOperationDefinition::SuperBreak(definition) => Operation::SuperBreak(SuperBreakOp {
            id: operation_plan.id,
            targets,
            definition: *definition,
        }),
        HitOperationDefinition::ApplyEffect(definition) => Operation::ApplyEffect(ApplyEffectOp {
            id: operation_plan.id,
            targets,
            definition: *definition,
            rng_purpose: None,
            resolved_chances: None,
            resolved_runtime: None,
        }),
        HitOperationDefinition::RemoveEffects(definition) => {
            Operation::RemoveEffects(RemoveEffectsOp {
                id: operation_plan.id,
                targets,
                definition: *definition,
            })
        }
        HitOperationDefinition::DetonateDots(definition) => {
            Operation::DetonateDots(DetonateDotsOp {
                id: operation_plan.id,
                targets,
                definition: *definition,
            })
        }
        HitOperationDefinition::ModifyStateSlot(definition) => {
            Operation::ModifyStateSlot(ModifyStateSlotOp {
                id: operation_plan.id,
                owner: plan.actor,
                instance: None,
                definition: definition.clone(),
            })
        }
        HitOperationDefinition::ModifyTeamResource(definition) => {
            Operation::ModifyTeamResource(ModifyTeamResourceOp {
                id: operation_plan.id,
                actor: plan.actor,
                definition: *definition,
            })
        }
        HitOperationDefinition::QueueAction(definition) => Operation::QueueAction(QueueActionOp {
            id: operation_plan.id,
            definition: *definition,
        }),
        HitOperationDefinition::SummonLinked(definition) => {
            Operation::SummonLinked(SummonLinkedOp {
                id: operation_plan.id,
                owners: vec![plan.actor].into_boxed_slice(),
                definition: definition.clone(),
            })
        }
        HitOperationDefinition::ChangePresence(presence) => {
            Operation::ChangePresence(ChangePresenceOp {
                id: operation_plan.id,
                targets,
                presence: *presence,
            })
        }
        HitOperationDefinition::Transform(definition) => Operation::Transform(TransformOp {
            id: operation_plan.id,
            targets,
            definition: definition.clone(),
        }),
        HitOperationDefinition::EndTransformation => {
            Operation::EndTransformation(UnitLifecycleOp {
                id: operation_plan.id,
                targets,
            })
        }
        HitOperationDefinition::Revive(definition) => Operation::Revive(ReviveOp {
            id: operation_plan.id,
            targets,
            definition: *definition,
        }),
        HitOperationDefinition::DespawnLinked => Operation::DespawnLinked(UnitLifecycleOp {
            id: operation_plan.id,
            targets,
        }),
        HitOperationDefinition::RequestWaveTransition => {
            Operation::RequestWaveTransition(EncounterLifecycleOp {
                id: operation_plan.id,
            })
        }
        HitOperationDefinition::TransitionEnemyPhase(phase) => {
            Operation::TransitionEnemyPhase(EnemyPhaseOp {
                id: operation_plan.id,
                targets,
                phase: *phase,
            })
        }
    };
    Ok(operation)
}

fn settle_after_hit(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let cause = frame
        .boundary_cause
        .take()
        .ok_or_else(|| action_fault(80))?;
    frame.parent = settle::settle_wave_boundary(
        catalog,
        txn,
        cause,
        frame.parent,
        WaveTransitionPolicy::AfterHit,
    )?;
    frame.hit_index = frame
        .hit_index
        .checked_add(1)
        .ok_or_else(|| action_fault(80))?;
    frame.stage = ExecutionStage::ExecuteHit;
    Ok(())
}

fn finish_phase(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let phase = frame
        .plan
        .phases
        .get(frame.phase_index)
        .ok_or_else(|| action_fault(80))?;
    let phase_cause = frame.base_cause()?.with_phase(phase.id);
    frame.parent = run_programs_at(
        catalog,
        txn,
        phase_cause,
        frame.parent,
        &frame.plan,
        AbilityProgramTiming::AfterHits,
        None,
        &mut HitOperationScratch::default(),
    )?;
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.parent = txn.emit(
        phase_cause.with_parent(frame.parent),
        BattleEventKind::Phase(PhaseEventData::Ended {
            action: frame.plan.id,
            phase: phase.id,
        }),
    );
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    frame.boundary_cause = Some(phase_cause);
    frame.stage = ExecutionStage::DrainAfterPhase;
    Ok(())
}

fn settle_after_phase(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let cause = frame
        .boundary_cause
        .take()
        .ok_or_else(|| action_fault(80))?;
    frame.parent = settle::settle_wave_boundary(
        catalog,
        txn,
        cause,
        frame.parent,
        WaveTransitionPolicy::AfterPhase,
    )?;
    frame.phase_index = frame
        .phase_index
        .checked_add(1)
        .ok_or_else(|| action_fault(80))?;
    frame.stage = ExecutionStage::StartPhase;
    Ok(())
}

fn finish_action(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    frame: &mut ExecutionFrame,
) -> Result<(), BattleFault> {
    let base = frame.base_cause()?;
    if frame.complete_envelope || frame.run_segment_programs {
        frame.parent = run_programs_at(
            catalog,
            txn,
            base,
            frame.parent,
            &frame.plan,
            AbilityProgramTiming::Resolved,
            None,
            &mut HitOperationScratch::default(),
        )?;
        frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    }
    if !frame.complete_envelope {
        return Ok(());
    }
    if frame.plan.origin == ActionOrigin::Countdown
        && catalog
            .countdown_for_ability(frame.plan.ability)
            .is_some_and(|definition| definition.definition().ends_transformation())
    {
        let operation = txn.allocate_operation();
        frame.parent = lifecycle::execute_end_transform(
            txn,
            base,
            frame.parent,
            UnitLifecycleOp {
                id: operation,
                targets: vec![frame.plan.owner].into_boxed_slice(),
            },
        )?;
    }
    frame.parent = apply_resource_gains(catalog, txn, base, frame.parent, &frame.plan)?;
    frame.parent = txn.emit(
        base.with_parent(frame.parent),
        BattleEventKind::Action(ActionEventData::Resolved {
            action: frame.plan.id,
            actor: frame.plan.actor,
            ability: frame.plan.ability,
            origin: frame.plan.origin,
            tags: frame.plan.tags,
            targets: frame.plan.targets.targets.clone(),
        }),
    );
    frame.parent = rule::dispatch_pending_after_events(catalog, txn, frame.parent)?;
    Ok(())
}
