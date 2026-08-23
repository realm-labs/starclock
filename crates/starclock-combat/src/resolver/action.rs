use crate::{
    ActionCancellationReason, ActionOrigin, ControlledAction, DiagnosticRecord, Energy,
    FaultBoundary, FaultKind, FaultPolicy, LifeState, Ratio, Rounding, Scalar, SkillPointPayer,
    UnitId,
    action::{
        lower::{QueuedActionContext, lower_queued_action},
        model::{ActionPlan, HitPlan},
    },
    battle::fault::BattleFault,
    catalog::{
        CombatCatalog,
        action::{
            AbilityProgramTiming, HitCritPolicy, HitTargetGroup, OrdinaryDamageDefinition,
            ReactionBoundary, ScalingDamageDefinition, SkillPointPaymentPolicy,
        },
    },
    command::legal::ability_owner,
    event::{
        cause::Cause,
        model::{ActionEventData, BattleEventKind, ResourceEventData},
    },
    formula::model::DamageClass,
    id::EventId,
    modifier::resolve::StatResolver,
    operation::HitOperationScratch,
    reaction::queue::QueuedAction,
    resource::check::can_pay_with_policy,
    target::model::TargetCommitment,
};

use super::{
    action_execution::execute_action_plan,
    command_resolution::action_cause,
    program::{AbilityProgramContext, execute_ability_program},
    transaction::{Transaction, action_fault},
};
use super::{operation, operation_formula, program, stat_input};

const MAX_REACTIONS_PER_COMMAND: usize = 256;

pub(super) fn drain_reactions(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    boundary: ReactionBoundary,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    while let Some(work) = dequeue_reaction(catalog, txn, boundary)? {
        match work {
            ReactionWork::Cancelled(event) => parent = event,
            ReactionWork::Execute { queued, plan } => {
                let resolved =
                    execute_action_plan(catalog, txn, queued.root, queued.parent, *plan)?;
                let cause = action_cause(queued.root, &resolved.plan)?;
                parent =
                    operation::settle_effects_at_action_end(catalog, txn, cause, resolved.parent)?;
            }
        }
    }
    Ok(parent)
}

pub(super) enum ReactionWork {
    Cancelled(EventId),
    Execute {
        queued: QueuedAction,
        plan: Box<ActionPlan>,
    },
}

pub(super) fn dequeue_reaction(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    boundary: ReactionBoundary,
) -> Result<Option<ReactionWork>, BattleFault> {
    let Some(queued) = txn.pop_ready_reaction(boundary) else {
        return Ok(None);
    };
    txn.record_diagnostic(|| DiagnosticRecord::ReactionDequeued {
        insertion: queued.order.insertion,
        actor: queued.actor,
        ability: queued.ability,
        boundary: queued.order.boundary,
    });
    if !txn.consume_reaction_budget(MAX_REACTIONS_PER_COMMAND) {
        return Err(BattleFault::new(
            FaultKind::BudgetExceeded,
            FaultBoundary::Command,
            FaultPolicy::Rollback,
            0x3171,
            Some(MAX_REACTIONS_PER_COMMAND as i64),
        ));
    }
    let Some(unit) = txn.state.units.get(queued.actor) else {
        let event = cancel_queued(txn, &queued, ActionCancellationReason::ActorUnavailable);
        return Ok(Some(ReactionWork::Cancelled(event)));
    };
    if unit.life != LifeState::Alive || !unit.presence.is_active() {
        let event = cancel_queued(txn, &queued, ActionCancellationReason::ActorUnavailable);
        return Ok(Some(ReactionWork::Cancelled(event)));
    }
    if ability_owner(txn.state, catalog, queued.actor, queued.ability).is_none() {
        let event = cancel_queued(txn, &queued, ActionCancellationReason::AbilityUnavailable);
        return Ok(Some(ReactionWork::Cancelled(event)));
    }
    if matches!(
        queued.origin,
        ActionOrigin::FollowUp | ActionOrigin::Counter
    ) && txn
        .state
        .effects
        .blocks(queued.actor, ControlledAction::FollowUp)
    {
        let event = cancel_queued(txn, &queued, ActionCancellationReason::FollowUpBlocked);
        return Ok(Some(ReactionWork::Cancelled(event)));
    }
    let Some(action) = catalog
        .ability(queued.ability)
        .and_then(|ability| ability.action())
    else {
        let event = cancel_queued(
            txn,
            &queued,
            ActionCancellationReason::MissingActionDefinition,
        );
        return Ok(Some(ReactionWork::Cancelled(event)));
    };
    let payment = queued
        .payment
        .unwrap_or(action.resources().skill_point_payment());
    if !can_pay_with_policy(
        &txn.state.units,
        &txn.state.teams,
        queued.actor,
        action.resources(),
        payment,
    ) {
        let event = cancel_queued(txn, &queued, ActionCancellationReason::ResourceUnavailable);
        return Ok(Some(ReactionWork::Cancelled(event)));
    }
    let mut plan = lower_queued_action(
        catalog,
        txn,
        QueuedActionContext {
            actor: queued.actor,
            owner: queued.owner,
            origin: queued.origin,
            payment: queued.payment,
        },
        queued.ability,
        queued.targets.clone(),
    )
    .ok_or_else(|| action_fault(72))?;
    let targets_accepted = txn
        .resolve_hit_targets(plan.actor, &mut plan.targets)
        .is_ok_and(|targets| !targets.is_empty());
    txn.record_diagnostic(|| DiagnosticRecord::ReactionTargetsValidated {
        insertion: queued.order.insertion,
        targets: (&plan.targets).into(),
        accepted: targets_accepted,
    });
    if !targets_accepted {
        let event = cancel_queued(txn, &queued, ActionCancellationReason::TargetInvalid);
        return Ok(Some(ReactionWork::Cancelled(event)));
    }
    Ok(Some(ReactionWork::Execute {
        queued,
        plan: Box::new(plan),
    }))
}

pub(super) fn cancel_queued(
    txn: &mut Transaction<'_>,
    queued: &QueuedAction,
    reason: ActionCancellationReason,
) -> EventId {
    txn.record_diagnostic(|| DiagnosticRecord::ReactionCancelled {
        insertion: queued.order.insertion,
        actor: queued.actor,
        ability: queued.ability,
        reason,
    });
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

pub(super) fn resolve_scaling_damage(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    actor: UnitId,
    definition: ScalingDamageDefinition,
) -> Result<OrdinaryDamageDefinition, BattleFault> {
    use crate::{
        modifier::model::{FormulaPurpose, StatQuerySubject},
        rule::evaluate::StatQueryReader,
    };

    let bases = program::stat_bases(txn)?;
    let modifiers = txn
        .state
        .modifiers
        .iter_by_id()
        .cloned()
        .collect::<Vec<_>>();
    let shields = stat_input::shield_values(txn);
    let reader =
        StatResolver::new(catalog.modifier_registry(), &bases, &modifiers).with_shields(&shields);
    let purpose = match definition.class() {
        DamageClass::Direct => FormulaPurpose::OrdinaryDamage,
        DamageClass::Dot => FormulaPurpose::Dot,
        DamageClass::Additional => FormulaPurpose::AdditionalDamage,
        DamageClass::Elation => FormulaPurpose::ElationDamage,
    };
    let stat = reader
        .query_stat(
            StatQuerySubject::Actor,
            actor,
            definition.scaling_stat(),
            purpose,
        )
        .map_err(|_| action_fault(34))?;
    definition.resolve(stat).map_err(|_| action_fault(35))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_programs_at(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    plan: &ActionPlan,
    timing: AbilityProgramTiming,
    hit: Option<&HitPlan>,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    for binding in plan
        .programs
        .iter()
        .filter(|binding| binding.timing() == timing)
    {
        parent = execute_ability_program(
            catalog,
            txn,
            cause,
            parent,
            AbilityProgramContext {
                program: binding.program(),
                owner: plan.owner,
                actor: plan.actor,
                ability: plan.ability,
                action: plan.id,
                rule: None,
                rule_instance: None,
                trigger: None,
                hit: hit.map(|value| value.id),
                primary: plan.targets.primary,
                damage_share: hit.map_or(Ratio::ONE, |value| value.damage_share),
                toughness_share: hit.map_or(Ratio::ONE, |value| value.toughness_share),
                crit_policy: hit.map_or(HitCritPolicy::Never, |value| value.crit_policy),
            },
            scratch,
        )?;
    }
    Ok(parent)
}

pub(super) fn project_hit_targets(
    txn: &mut Transaction<'_>,
    actor: UnitId,
    commitment: &TargetCommitment,
    group: HitTargetGroup,
    selected: Box<[UnitId]>,
) -> Result<Box<[UnitId]>, BattleFault> {
    use crate::catalog::action::HitTargetGroup;
    let targets = match group {
        HitTargetGroup::Primary => commitment
            .primary
            .or_else(|| selected.first().copied())
            .into_iter()
            .collect::<Vec<_>>(),
        HitTargetGroup::Adjacent => selected
            .iter()
            .copied()
            .filter(|target| Some(*target) != commitment.primary)
            .collect(),
        HitTargetGroup::Selected | HitTargetGroup::All => selected.into_vec(),
        HitTargetGroup::BounceDraw => {
            if selected.is_empty() {
                Vec::new()
            } else {
                vec![txn.draw_bounce_target(actor, commitment.selector.relation())?]
            }
        }
        HitTargetGroup::SelfTarget => vec![actor],
    };
    Ok(targets.into_boxed_slice())
}

pub(super) fn apply_resource_costs(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    plan: &ActionPlan,
) -> Result<EventId, BattleFault> {
    let policy = &plan.resources;
    let side = txn
        .state
        .units
        .get(plan.actor)
        .ok_or_else(|| action_fault(20))?
        .side;
    if policy.skill_point_cost() > 0 {
        let attempted = policy.skill_point_cost();
        let (payer, before, after, effective) = match policy.skill_point_payment() {
            SkillPointPaymentPolicy::TeamSkillPoints => {
                let before = txn.state.teams.get(side).skill_points;
                let after = before
                    .checked_sub(attempted)
                    .ok_or_else(|| action_fault(21))?;
                txn.set_skill_points(side, after);
                (SkillPointPayer::TeamSkillPoints, before, after, attempted)
            }
            SkillPointPaymentPolicy::Suppressed => (SkillPointPayer::Suppressed, 0, 0, 0),
            SkillPointPaymentPolicy::TeamResource(resource) => {
                let before = txn
                    .state
                    .teams
                    .get(side)
                    .keyed(resource)
                    .ok_or_else(|| action_fault(31))?
                    .current;
                let after = before.checked_sub(attempted).ok_or_else(|| {
                    BattleFault::new(
                        FaultKind::InvariantViolation,
                        FaultBoundary::Command,
                        FaultPolicy::Rollback,
                        0x3120,
                        Some(i64::from(plan.ability.get())),
                    )
                })?;
                txn.set_team_resource(side, resource, after)?;
                (
                    SkillPointPayer::TeamResource(resource),
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
    if policy.energy_cost() > Energy::ZERO {
        let before = txn
            .state
            .units
            .get(plan.actor)
            .ok_or_else(|| action_fault(22))?
            .current_energy;
        let after = Energy::from_scaled(
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
                overflow: Energy::ZERO,
            }),
        );
    }
    for cost in policy.character_resource_costs() {
        let (before, maximum) = txn
            .state
            .units
            .get(plan.actor)
            .and_then(|unit| unit.resource(cost.stable_key()))
            .map(|resource| (resource.current, resource.maximum))
            .ok_or_else(|| action_fault(73))?;
        let after = Scalar::from_scaled(
            before
                .scaled()
                .checked_sub(cost.amount().scaled())
                .filter(|value| *value >= 0)
                .ok_or_else(|| action_fault(74))?,
        );
        txn.set_character_resource(plan.actor, cost.stable_key(), after)?;
        parent = txn.emit(
            cause
                .with_parent(parent)
                .with_primary_target(Some(plan.actor)),
            BattleEventKind::Resource(ResourceEventData::CharacterResource {
                unit: plan.actor,
                resource: cost.stable_key().into(),
                before,
                after,
                maximum,
            }),
        );
    }
    for cost in policy.team_resource_costs() {
        let (resource, before, maximum) = txn
            .state
            .teams
            .get(side)
            .keyed_by_name(cost.stable_key())
            .map(|state| (state.id, state.current, state.maximum))
            .ok_or_else(|| action_fault(75))?;
        let after = before
            .checked_sub(cost.amount())
            .ok_or_else(|| action_fault(76))?;
        txn.set_team_resource(side, resource, after)?;
        parent = txn.emit(
            cause.with_parent(parent),
            BattleEventKind::Resource(ResourceEventData::TeamResource {
                side,
                resource,
                attempted: cost.amount(),
                effective: cost.amount(),
                before,
                after,
                overflow: 0,
            }),
        );
        debug_assert!(after <= maximum);
    }
    Ok(parent)
}

pub(super) fn apply_resource_gains(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    plan: &ActionPlan,
) -> Result<EventId, BattleFault> {
    let policy = &plan.resources;
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
                payer: SkillPointPayer::TeamSkillPoints,
                effective: after - before,
                before,
                after,
                overflow,
            }),
        );
    }
    if policy.energy_gain() > Energy::ZERO {
        let rate = operation_formula::FormulaInputs::new(txn)?
            .energy_regeneration_rate(catalog, txn, cause, plan.actor)?;
        let gain = Scalar::from_scaled(policy.energy_gain().scaled())
            .checked_mul(rate, Rounding::NearestTiesEven)
            .map_err(|_| action_fault(77))?;
        if gain.scaled() < 0 {
            return Err(action_fault(78));
        }
        let unit = txn
            .state
            .units
            .get(plan.actor)
            .ok_or_else(|| action_fault(27))?;
        let before = unit.current_energy;
        let maximum = unit.maximum_energy;
        let uncapped = before
            .scaled()
            .checked_add(gain.scaled())
            .ok_or_else(|| action_fault(28))?;
        let after_scaled = uncapped.min(maximum.scaled());
        let overflow =
            Energy::from_scaled(uncapped - after_scaled).map_err(|_| action_fault(29))?;
        let after = Energy::from_scaled(after_scaled).map_err(|_| action_fault(30))?;
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
