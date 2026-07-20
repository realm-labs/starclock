use crate::{
    action::model::ActionPlan,
    battle::fault::BattleFault,
    catalog::action::HitOperationDefinition,
    event::{
        cause::{Cause, CauseActor},
        model::{
            ActionEventData, BattleEventKind, HitEventData, PhaseEventData, ResourceEventData,
        },
    },
    id::{CommandId, EventId, SourceDefinitionId},
    operation::{
        AddWeaknessOp, ApplyEffectOp, ChangePresenceOp, ConsumeHpOp, DamageOp, DetonateDotsOp,
        EncounterLifecycleOp, EnemyPhaseOp, HealOp, HitOperationScratch, ModifyStateSlotOp,
        ModifyTeamResourceOp, Operation, QueueActionOp, ReduceToughnessOp, RemoveEffectsOp,
        ReviveOp, ShieldOp, SummonLinkedOp, SuperBreakOp, TransformOp, UnitLifecycleOp,
    },
};

use super::{
    operation::execute_operation,
    transaction::{Transaction, action_fault},
};

const MAX_REACTIONS_PER_COMMAND: usize = 256;

pub(super) fn drain_reactions(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    boundary: crate::catalog::action::ReactionBoundary,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    while let Some(queued) = txn.reactions.pop_ready(boundary) {
        if !txn.consume_reaction_budget(MAX_REACTIONS_PER_COMMAND) {
            return Err(BattleFault::new(
                crate::FaultKind::BudgetExceeded,
                crate::FaultBoundary::Command,
                crate::FaultPolicy::Rollback,
                0x3171,
                Some(MAX_REACTIONS_PER_COMMAND as i64),
            ));
        }
        let eligible = txn.state.units.get(queued.actor).is_some_and(|unit| {
            unit.life == crate::LifeState::Alive
                && unit.presence.is_active()
                && unit.abilities.binary_search(&queued.ability).is_ok()
                && !(matches!(
                    queued.origin,
                    crate::ActionOrigin::FollowUp | crate::ActionOrigin::Counter
                ) && txn
                    .state
                    .effects
                    .blocks(queued.actor, crate::ControlledAction::FollowUp))
        });
        if !eligible {
            parent = cancel_queued(txn, &queued);
            continue;
        }
        let Some(action) = catalog
            .ability(queued.ability)
            .and_then(|ability| ability.action())
        else {
            parent = cancel_queued(txn, &queued);
            continue;
        };
        let payment = queued
            .payment
            .unwrap_or(action.resources().skill_point_payment());
        if !crate::resource::check::can_pay_with_policy(
            &txn.state.units,
            &txn.state.teams,
            queued.actor,
            action.resources(),
            payment,
        ) {
            parent = cancel_queued(txn, &queued);
            continue;
        }
        let mut plan = crate::action::lower::lower_queued_action(
            catalog,
            txn,
            crate::action::lower::QueuedActionContext {
                actor: queued.actor,
                owner: queued.owner,
                origin: queued.origin,
                payment: queued.payment,
            },
            queued.ability,
            queued.targets.clone(),
        )
        .ok_or_else(|| action_fault(72))?;
        if !txn
            .resolve_hit_targets(plan.actor, &mut plan.targets)
            .is_ok_and(|targets| !targets.is_empty())
        {
            parent = cancel_queued(txn, &queued);
            continue;
        }
        parent = execute_action_plan(catalog, txn, queued.root, queued.parent, &mut plan)?;
        let cause = super::transaction::action_cause(queued.root, &plan)?;
        parent = super::operation::settle_effects_at_action_end(txn, cause, parent)?;
    }
    Ok(parent)
}

fn cancel_queued(
    txn: &mut Transaction<'_>,
    queued: &crate::reaction::queue::QueuedAction,
) -> EventId {
    txn.emit(
        Cause::root(queued.root)
            .with_parent(queued.parent)
            .with_primary_target(queued.targets.primary),
        BattleEventKind::Action(ActionEventData::Cancelled {
            insertion: queued.order.insertion,
            actor: queued.actor,
            ability: queued.ability,
            origin: queued.origin,
        }),
    )
}

pub(super) fn execute_action_plan(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    root: CommandId,
    command_parent: EventId,
    plan: &mut ActionPlan,
) -> Result<EventId, BattleFault> {
    debug_assert_eq!(plan.normal_turn.is_some(), plan.origin.owns_timeline_turn());
    let _selector = plan.selector;
    let source = SourceDefinitionId::new(plan.ability.get()).ok_or_else(|| action_fault(7))?;
    let base = Cause::for_action(
        root,
        plan.id,
        plan.owner,
        CauseActor::Unit(plan.actor),
        source,
    )
    .with_primary_target(plan.targets.primary);
    let mut parent = txn.emit(
        base.with_parent(command_parent),
        BattleEventKind::Action(ActionEventData::Declared {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
            tags: plan.tags,
        }),
    );
    parent = apply_resource_costs(txn, base, parent, plan)?;
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
    txn.reset_rule_slots(
        crate::rule::model::SlotResetPoint::ActionStart,
        Some(plan.actor),
    );
    let phases = plan.phases.clone();
    for phase in &phases {
        let phase_cause = base.with_phase(phase.id);
        parent = txn.emit(
            phase_cause.with_parent(parent),
            BattleEventKind::Phase(PhaseEventData::Started {
                action: plan.id,
                phase: phase.id,
            }),
        );
        for hit in &phase.hits {
            txn.reset_rule_slots(
                crate::rule::model::SlotResetPoint::HitStart,
                Some(plan.actor),
            );
            let mut operation_scratch = HitOperationScratch::default();
            debug_assert_eq!(hit.invalidation, plan.targets.invalidation);
            let targets = txn.resolve_hit_targets(plan.actor, &mut plan.targets)?;
            let hit_cause = phase_cause
                .with_hit(hit.id)
                .with_primary_target(plan.targets.primary);
            parent = txn.emit(
                hit_cause.with_parent(parent),
                BattleEventKind::Hit(HitEventData::Started {
                    action: plan.id,
                    phase: phase.id,
                    hit: hit.id,
                    targets: targets.clone(),
                }),
            );
            for operation in &hit.operations {
                let request = match &operation.definition {
                    HitOperationDefinition::Damage(formula) => Operation::Damage(DamageOp {
                        id: operation.id,
                        targets: targets.clone(),
                        formula: *formula,
                    }),
                    HitOperationDefinition::Heal(formula) => Operation::Heal(HealOp {
                        id: operation.id,
                        targets: targets.clone(),
                        formula: *formula,
                    }),
                    HitOperationDefinition::Shield(formula) => Operation::Shield(ShieldOp {
                        id: operation.id,
                        targets: targets.clone(),
                        formula: *formula,
                    }),
                    HitOperationDefinition::ConsumeHp(definition) => {
                        Operation::ConsumeHp(ConsumeHpOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::AddWeakness(definition) => {
                        Operation::AddWeakness(AddWeaknessOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::ReduceToughness(definition) => {
                        Operation::ReduceToughness(ReduceToughnessOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::SuperBreak(definition) => {
                        Operation::SuperBreak(SuperBreakOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::ApplyEffect(definition) => {
                        Operation::ApplyEffect(ApplyEffectOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::RemoveEffects(definition) => {
                        Operation::RemoveEffects(RemoveEffectsOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::DetonateDots(definition) => {
                        Operation::DetonateDots(DetonateDotsOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::ModifyStateSlot(definition) => {
                        Operation::ModifyStateSlot(ModifyStateSlotOp {
                            id: operation.id,
                            owner: plan.actor,
                            definition: definition.clone(),
                        })
                    }
                    HitOperationDefinition::ModifyTeamResource(definition) => {
                        Operation::ModifyTeamResource(ModifyTeamResourceOp {
                            id: operation.id,
                            actor: plan.actor,
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::QueueAction(definition) => {
                        Operation::QueueAction(QueueActionOp {
                            id: operation.id,
                            definition: *definition,
                        })
                    }
                    HitOperationDefinition::SummonLinked(definition) => {
                        Operation::SummonLinked(SummonLinkedOp {
                            id: operation.id,
                            owners: vec![plan.actor].into_boxed_slice(),
                            definition: definition.clone(),
                        })
                    }
                    HitOperationDefinition::ChangePresence(presence) => {
                        Operation::ChangePresence(ChangePresenceOp {
                            id: operation.id,
                            targets: targets.clone(),
                            presence: *presence,
                        })
                    }
                    HitOperationDefinition::Transform(definition) => {
                        Operation::Transform(TransformOp {
                            id: operation.id,
                            targets: targets.clone(),
                            definition: definition.clone(),
                        })
                    }
                    HitOperationDefinition::EndTransformation => {
                        Operation::EndTransformation(UnitLifecycleOp {
                            id: operation.id,
                            targets: targets.clone(),
                        })
                    }
                    HitOperationDefinition::Revive(definition) => Operation::Revive(ReviveOp {
                        id: operation.id,
                        targets: targets.clone(),
                        definition: *definition,
                    }),
                    HitOperationDefinition::DespawnLinked => {
                        Operation::DespawnLinked(UnitLifecycleOp {
                            id: operation.id,
                            targets: targets.clone(),
                        })
                    }
                    HitOperationDefinition::RequestWaveTransition => {
                        Operation::RequestWaveTransition(EncounterLifecycleOp { id: operation.id })
                    }
                    HitOperationDefinition::TransitionEnemyPhase(phase) => {
                        Operation::TransitionEnemyPhase(EnemyPhaseOp {
                            id: operation.id,
                            targets: targets.clone(),
                            phase: *phase,
                        })
                    }
                };
                parent = execute_operation(
                    catalog,
                    txn,
                    hit_cause.with_applier(plan.actor),
                    parent,
                    request,
                    &mut operation_scratch,
                )?;
            }
            txn.increment_entanglement_for_hit(&targets)?;
            parent = txn.emit(
                hit_cause.with_parent(parent),
                BattleEventKind::Hit(HitEventData::Ended {
                    action: plan.id,
                    phase: phase.id,
                    hit: hit.id,
                    targets,
                }),
            );
            parent = drain_reactions(
                catalog,
                txn,
                crate::catalog::action::ReactionBoundary::AfterHit,
                parent,
            )?;
            parent = super::settle::settle_wave_boundary(
                catalog,
                txn,
                hit_cause,
                parent,
                crate::catalog::encounter::WaveTransitionPolicy::AfterHit,
            )?;
        }
        parent = txn.emit(
            phase_cause.with_parent(parent),
            BattleEventKind::Phase(PhaseEventData::Ended {
                action: plan.id,
                phase: phase.id,
            }),
        );
        parent = drain_reactions(
            catalog,
            txn,
            crate::catalog::action::ReactionBoundary::AfterPhase,
            parent,
        )?;
        parent = super::settle::settle_wave_boundary(
            catalog,
            txn,
            phase_cause,
            parent,
            crate::catalog::encounter::WaveTransitionPolicy::AfterPhase,
        )?;
    }
    parent = apply_resource_gains(txn, base, parent, plan)?;
    Ok(txn.emit(
        base.with_parent(parent),
        BattleEventKind::Action(ActionEventData::Resolved {
            action: plan.id,
            actor: plan.actor,
            ability: plan.ability,
            origin: plan.origin,
            tags: plan.tags,
        }),
    ))
}

fn apply_resource_costs(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    plan: &ActionPlan,
) -> Result<EventId, BattleFault> {
    let policy = plan.resources;
    let side = txn
        .state
        .units
        .get(plan.actor)
        .ok_or_else(|| action_fault(20))?
        .side;
    if policy.skill_point_cost() > 0 {
        let attempted = policy.skill_point_cost();
        let (payer, before, after, effective) = match policy.skill_point_payment() {
            crate::catalog::action::SkillPointPaymentPolicy::TeamSkillPoints => {
                let before = txn.state.teams.get(side).skill_points;
                let after = before
                    .checked_sub(attempted)
                    .ok_or_else(|| action_fault(21))?;
                txn.set_skill_points(side, after);
                (
                    crate::SkillPointPayer::TeamSkillPoints,
                    before,
                    after,
                    attempted,
                )
            }
            crate::catalog::action::SkillPointPaymentPolicy::Suppressed => {
                (crate::SkillPointPayer::Suppressed, 0, 0, 0)
            }
            crate::catalog::action::SkillPointPaymentPolicy::TeamResource(resource) => {
                let before = txn
                    .state
                    .teams
                    .get(side)
                    .keyed(resource)
                    .ok_or_else(|| action_fault(31))?
                    .current;
                let after = before
                    .checked_sub(attempted)
                    .ok_or_else(|| action_fault(32))?;
                txn.set_team_resource(side, resource, after)?;
                (
                    crate::SkillPointPayer::TeamResource(resource),
                    before,
                    after,
                    attempted,
                )
            }
        };
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Resource(ResourceEventData::SkillPoints {
                side,
                attempted,
                payer,
                effective,
                before,
                after,
                overflow: 0,
            }),
        );
    }
    if policy.energy_cost() > crate::Energy::ZERO {
        let before = txn
            .state
            .units
            .get(plan.actor)
            .ok_or_else(|| action_fault(22))?
            .current_energy;
        let after = crate::Energy::from_scaled(
            before
                .scaled()
                .checked_sub(policy.energy_cost().scaled())
                .ok_or_else(|| action_fault(23))?,
        )
        .map_err(|_| action_fault(24))?;
        txn.set_energy(plan.actor, after)?;
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Resource(ResourceEventData::Energy {
                unit: plan.actor,
                before,
                after,
                overflow: crate::Energy::ZERO,
            }),
        );
    }
    Ok(parent)
}

fn apply_resource_gains(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    plan: &ActionPlan,
) -> Result<EventId, BattleFault> {
    let policy = plan.resources;
    let side = txn
        .state
        .units
        .get(plan.actor)
        .ok_or_else(|| action_fault(25))?
        .side;
    if policy.skill_point_gain() > 0 {
        let (before, maximum) = {
            let team = txn.state.teams.get(side);
            (team.skill_points, team.maximum_skill_points)
        };
        let uncapped = u32::from(before) + u32::from(policy.skill_point_gain());
        let after =
            u16::try_from(uncapped.min(u32::from(maximum))).map_err(|_| action_fault(26))?;
        let overflow = u16::try_from(uncapped - u32::from(after)).map_err(|_| action_fault(26))?;
        txn.set_skill_points(side, after);
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Resource(ResourceEventData::SkillPoints {
                side,
                attempted: policy.skill_point_gain(),
                payer: crate::SkillPointPayer::TeamSkillPoints,
                effective: after - before,
                before,
                after,
                overflow,
            }),
        );
    }
    if policy.energy_gain() > crate::Energy::ZERO {
        let unit = txn
            .state
            .units
            .get(plan.actor)
            .ok_or_else(|| action_fault(27))?;
        let before = unit.current_energy;
        let maximum = unit.maximum_energy;
        let uncapped = before
            .scaled()
            .checked_add(policy.energy_gain().scaled())
            .ok_or_else(|| action_fault(28))?;
        let after_scaled = uncapped.min(maximum.scaled());
        let overflow =
            crate::Energy::from_scaled(uncapped - after_scaled).map_err(|_| action_fault(29))?;
        let after = crate::Energy::from_scaled(after_scaled).map_err(|_| action_fault(30))?;
        txn.set_energy(plan.actor, after)?;
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Resource(ResourceEventData::Energy {
                unit: plan.actor,
                before,
                after,
                overflow,
            }),
        );
    }
    Ok(parent)
}
