use super::transaction::Transaction;
use crate::{
    DamageAmount, Hp, LifeState,
    battle::fault::BattleFault,
    event::{
        cause::Cause,
        model::{
            BattleEventKind, BreakDamageEventData, BreakDamageKind, DamageEventData, DamageKind,
            EffectEventData, ShieldEventData, ToughnessEventData, UnitEventData,
        },
    },
    formula,
    id::EventId,
    operation::{
        AddWeaknessOp, ApplyEffectOp, DamageOp, HitOperationScratch, Operation, ReduceToughnessOp,
        RemoveEffectsOp, SuperBreakOp,
    },
};
pub(super) mod fault;
mod sustain;
use fault::{invariant_fault, numeric_fault};

pub(super) fn execute_operation(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: Operation,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    txn.snapshot(operation.id());
    match operation {
        Operation::Damage(operation) => {
            execute_damage(catalog, txn, cause, parent, operation, scratch)
        }
        Operation::Heal(operation) => sustain::execute_heal(catalog, txn, cause, parent, operation),
        Operation::Shield(operation) => {
            sustain::execute_shield(catalog, txn, cause, parent, operation)
        }
        Operation::RemoveShields(operation) => {
            sustain::execute_remove_shields(txn, cause, parent, operation)
        }
        Operation::ConsumeHp(operation) => {
            sustain::execute_hp_consumption(txn, cause, parent, operation)
        }
        Operation::AddWeakness(operation) => execute_add_weakness(txn, cause, parent, operation),
        Operation::ReduceToughness(operation) => {
            execute_toughness_reduction(catalog, txn, cause, parent, operation, scratch)
        }
        Operation::ForceBreak(operation) => super::operation_break::execute_force_break(
            catalog, txn, cause, parent, operation, scratch,
        ),
        Operation::SuperBreak(operation) => {
            execute_super_break(catalog, txn, cause, parent, operation, scratch)
        }
        Operation::ApplyEffect(operation) => {
            execute_apply_effect(catalog, txn, cause, parent, operation)
        }
        Operation::RemoveEffects(operation) => {
            execute_remove_effects(txn, cause, parent, operation)
        }
        Operation::DetonateDots(operation) => {
            super::effect_operation::detonate_dots(catalog, txn, cause, parent, operation)
        }
        Operation::ModifyStateSlot(operation) => {
            super::operation_resource::execute_modify_state_slot(txn, cause, parent, operation)
        }
        Operation::ModifyTeamResource(operation) => {
            super::operation_resource::execute_modify_team_resource(txn, cause, parent, operation)
        }
        Operation::QueueAction(operation) => {
            super::schedule::execute_queue_action(catalog, txn, cause, parent, operation)
        }
        Operation::QueueRuleAction(operation) => {
            super::schedule::execute_queue_rule_action(catalog, txn, cause, parent, operation)
        }
        Operation::SummonLinked(operation) => {
            super::lifecycle::execute_summon(catalog, txn, cause, parent, operation)
        }
        Operation::CreateCountdown(operation) => {
            super::lifecycle::execute_countdown(catalog, txn, cause, parent, operation)
        }
        Operation::ChangePresence(operation) => {
            super::lifecycle::execute_presence(txn, cause, parent, operation)
        }
        Operation::Transform(operation) => {
            super::lifecycle::execute_transform(catalog, txn, cause, parent, operation)
        }
        Operation::EndTransformation(operation) => {
            super::lifecycle::execute_end_transform(txn, cause, parent, operation)
        }
        Operation::Revive(operation) => {
            super::lifecycle::execute_revive(txn, cause, parent, operation)
        }
        Operation::DespawnLinked(operation) => {
            super::lifecycle::execute_despawn(txn, cause, parent, operation)
        }
        Operation::RequestWaveTransition(_) => {
            super::settle::request_explicit_wave_transition(catalog, txn, cause, parent)
        }
        Operation::TransitionEnemyPhase(operation) => {
            super::lifecycle::execute_enemy_phase(catalog, txn, cause, parent, operation)
        }
    }
}

fn execute_add_weakness(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: AddWeaknessOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        let element = operation.definition.element();
        let added = txn.add_weakness(
            target,
            element,
            operation.definition.duration_turns(),
            cause.applier().ok_or_else(|| invariant_fault(11))?,
            operation.id,
        )?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Toughness(ToughnessEventData::WeaknessAdded {
                operation: operation.id,
                target,
                element,
                already_present: !added,
                duration_turns: operation.definition.duration_turns(),
            }),
        );
    }
    Ok(parent)
}

pub(super) fn execute_toughness_reduction(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ReduceToughnessOp,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let inputs = super::operation_formula::FormulaInputs::new(txn)?;
    for target in operation.targets {
        let mut definition = operation.definition;
        if !definition.ignores_weakness {
            let efficiency = inputs.weakness_break_efficiency(
                catalog,
                txn,
                cause,
                target,
                definition.element,
            )?;
            definition.reduction.weakness_break_efficiency = definition
                .reduction
                .weakness_break_efficiency
                .checked_add(efficiency)
                .map_err(|_| numeric_fault(11, efficiency.scaled()))?;
        }
        let calculation = formula::toughness::reduction(definition.reduction)
            .map_err(|_| numeric_fault(11, definition.reduction.base.get()))?;
        let (mut layers, weaknesses, was_broken, rank, max_hp) = txn
            .state
            .units
            .get(target)
            .map(|unit| {
                (
                    unit.toughness_layers.clone(),
                    unit.weaknesses.clone(),
                    unit.weakness_broken,
                    unit.rank,
                    unit.maximum_hp,
                )
            })
            .ok_or_else(|| invariant_fault(6))?;
        let routed = crate::toughness::state::route_reduction_with_override(
            &mut layers,
            &weaknesses,
            was_broken,
            definition.element,
            calculation.attempted,
            definition.ignores_weakness,
        );
        let zero = crate::RawToughness::new(0).expect("zero Toughness is valid");
        let (layer_key, effective, before, after) =
            routed.map_or((None, zero, zero, zero), |value| {
                (
                    Some(value.layer_key),
                    value.effective,
                    value.before,
                    value.after,
                )
            });
        scratch.effective_reductions.insert(target, effective);
        if let Some(value) = routed {
            txn.set_toughness(target, value.layer_key, value.after)?;
        }
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Toughness(ToughnessEventData::Reduced {
                operation: operation.id,
                target,
                element: definition.element,
                layer_key,
                attempted: calculation.attempted,
                effective,
                before,
                after,
            }),
        );
        let Some(value) = routed.filter(|value| value.depleted) else {
            continue;
        };
        let break_cause = match value.break_credit {
            crate::BreakCreditPolicy::HitApplier => cause,
            crate::BreakCreditPolicy::LayerProvider(source) => cause.with_source_definition(source),
        };
        if value.applies_break_damage {
            let mut break_damage = definition.break_damage;
            break_damage.mitigation_multiplier = dynamic_mitigation(
                catalog,
                txn,
                target,
                crate::modifier::model::FormulaPurpose::Break,
                value.break_element,
                break_damage.mitigation_multiplier,
            )?;
            let damage = formula::toughness::break_damage(
                break_damage,
                value.break_element,
                value.maximum,
                was_broken,
            )
            .map_err(|_| numeric_fault(12, value.maximum.get()))?;
            parent = apply_break_damage(
                catalog,
                txn,
                break_cause,
                parent,
                BreakDamageApplication {
                    operation: operation.id,
                    target,
                    element: value.break_element,
                    kind: BreakDamageKind::Initial,
                    raw: damage.raw,
                    calculated: damage.finalized,
                },
            )?;
        }
        // Universal 25% Break delay is exactly 2,500 Action Gauge.
        txn.delay_unit(target, 2_500_000_000)?;
        if value.changed_global_broken {
            txn.set_weakness_broken(target, true)?;
        }
        parent = txn.emit(
            break_cause
                .with_parent(parent)
                .with_primary_target(Some(target)),
            BattleEventKind::Toughness(ToughnessEventData::LayerDepleted {
                operation: operation.id,
                target,
                layer_key: value.layer_key,
                changed_global_broken: value.changed_global_broken,
            }),
        );
        if value.applies_break_effect {
            let applied = txn.roll_probability(
                definition.break_effect_chance,
                crate::rng::types::DrawPurpose::EFFECT_CHANCE,
            )?;
            if applied {
                let plan = formula::toughness::base_break_effect(
                    value.break_element,
                    rank,
                    max_hp,
                    definition.break_damage.attacker_level_multiplier,
                    value.maximum,
                    definition.break_damage.break_effect,
                )
                .map_err(|_| numeric_fault(13, value.maximum.get()))?;
                if plan.additional_delay.scaled() > 0 {
                    txn.delay_unit(
                        target,
                        plan.additional_delay
                            .scaled()
                            .checked_mul(10_000)
                            .ok_or_else(|| numeric_fault(14, plan.additional_delay.scaled()))?,
                    )?;
                }
                let effect = txn.allocate_effect();
                let speed_before = if plan.speed_reduction.scaled() > 0 {
                    let before = txn.unit_speed(target)?;
                    let multiplier = crate::Ratio::ONE
                        .checked_sub(plan.speed_reduction)
                        .map_err(|_| numeric_fault(19, plan.speed_reduction.scaled()))?;
                    let scaled = multiplier
                        .checked_apply(
                            crate::Scalar::from_scaled(before.scaled()),
                            crate::Rounding::NearestTiesEven,
                        )
                        .map_err(|_| numeric_fault(20, before.scaled()))?;
                    txn.set_unit_speed(
                        target,
                        crate::Speed::from_scaled(scaled.scaled())
                            .map_err(|_| numeric_fault(21, scaled.scaled()))?,
                    )?;
                    Some(before)
                } else {
                    None
                };
                txn.record_break_effect(crate::effect::break_effect::BreakEffectState {
                    id: effect,
                    owner: target,
                    applier: cause.applier().ok_or_else(|| invariant_fault(7))?,
                    source_operation: operation.id,
                    source_definition: break_cause
                        .source_definition()
                        .ok_or_else(|| invariant_fault(11))?,
                    plan,
                    damage: definition.break_damage,
                    remaining_turns: plan.duration_turns,
                    stacks: plan.initial_stacks,
                    speed_before,
                });
                parent = txn.emit(
                    break_cause
                        .with_parent(parent)
                        .with_primary_target(Some(target)),
                    BattleEventKind::Toughness(ToughnessEventData::BaseEffectApplied {
                        operation: operation.id,
                        target,
                        effect,
                        element: value.break_element,
                        duration_turns: plan.duration_turns,
                        stacks: plan.initial_stacks,
                    }),
                );
            } else {
                parent = txn.emit(
                    break_cause
                        .with_parent(parent)
                        .with_primary_target(Some(target)),
                    BattleEventKind::Toughness(ToughnessEventData::BaseEffectResisted {
                        operation: operation.id,
                        target,
                        element: value.break_element,
                    }),
                );
            }
        }
    }
    Ok(parent)
}

fn execute_super_break(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: SuperBreakOp,
    scratch: &HitOperationScratch,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        let effective = scratch
            .effective_reductions
            .get(&target)
            .copied()
            .unwrap_or(crate::RawToughness::new(0).expect("zero is valid"));
        let broken = txn
            .state
            .units
            .get(target)
            .map(|unit| unit.weakness_broken)
            .ok_or_else(|| invariant_fault(8))?;
        if !broken || effective.get() == 0 {
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Toughness(ToughnessEventData::SuperBreakSkipped {
                    operation: operation.id,
                    target,
                    effective_reduction: effective,
                }),
            );
            continue;
        }
        let mut definition = operation.definition;
        definition.mitigation_multiplier = dynamic_mitigation(
            catalog,
            txn,
            target,
            crate::modifier::model::FormulaPurpose::SuperBreak,
            definition.element,
            definition.mitigation_multiplier,
        )?;
        let damage = formula::toughness::super_break_damage(definition, effective)
            .map_err(|_| numeric_fault(15, effective.get()))?;
        parent = apply_break_damage(
            catalog,
            txn,
            cause,
            parent,
            BreakDamageApplication {
                operation: operation.id,
                target,
                element: operation.definition.element,
                kind: BreakDamageKind::SuperBreak,
                raw: damage.raw,
                calculated: damage.finalized,
            },
        )?;
    }
    Ok(parent)
}

#[derive(Clone, Copy)]
struct BreakDamageApplication {
    operation: crate::OperationId,
    target: crate::UnitId,
    element: crate::formula::model::CombatElement,
    kind: BreakDamageKind,
    raw: crate::Scalar,
    calculated: crate::DamageAmount,
}

fn apply_break_damage(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    application: BreakDamageApplication,
) -> Result<EventId, BattleFault> {
    let BreakDamageApplication {
        operation,
        target,
        element,
        kind,
        raw,
        mut calculated,
    } = application;
    let (hp_before, life_before) = txn
        .state
        .units
        .get(target)
        .map(|unit| (unit.current_hp, unit.life))
        .ok_or_else(|| invariant_fault(9))?;
    (parent, calculated) =
        apply_damage_guard(catalog, txn, cause, parent, operation, target, calculated)?;
    let (absorbed, changes) = txn
        .state
        .shields
        .absorb(target, calculated)
        .map_err(|_| numeric_fault(16, calculated.get()))?;
    for change in changes {
        txn.record_shield_change(change.before, change.after);
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Shield(ShieldEventData::Absorbed {
                shield: change.id,
                target,
                before: change.before,
                after: change.after,
            }),
        );
    }
    let applied_raw = (calculated.get() - absorbed.get()).min(hp_before.get());
    let applied = DamageAmount::new(applied_raw).map_err(|_| numeric_fault(17, applied_raw))?;
    let hp_after =
        Hp::new(hp_before.get() - applied_raw).map_err(|_| numeric_fault(18, hp_before.get()))?;
    txn.set_hp(target, hp_after)?;
    parent = txn.emit(
        cause.with_parent(parent).with_primary_target(Some(target)),
        BattleEventKind::BreakDamage(BreakDamageEventData {
            operation,
            target,
            kind,
            element,
            raw,
            calculated,
            absorbed,
            applied,
            hp_before,
            hp_after,
        }),
    );
    if hp_after.get() == 0 && life_before == LifeState::Alive {
        txn.set_life(target, LifeState::Downed)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Unit(UnitEventData::Downed { unit: target }),
        );
        txn.set_life(target, LifeState::Defeated)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Unit(UnitEventData::Defeated {
                unit: target,
                credited_to: cause.applier().ok_or_else(|| invariant_fault(10))?,
            }),
        );
        parent = super::lifecycle::settle_owner_defeat(txn, cause, parent, target)?;
    }
    Ok(parent)
}

pub(super) fn settle_break_effects_at_turn_start(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    owner: crate::UnitId,
) -> Result<(EventId, bool), BattleFault> {
    let effects = txn.state.break_effects.active_for(owner);
    let mut skips_action = false;
    for effect in effects {
        let expires = effect.remaining_turns == 1;
        let is_dot = matches!(
            effect.plan.element,
            crate::formula::model::CombatElement::Physical
                | crate::formula::model::CombatElement::Fire
                | crate::formula::model::CombatElement::Lightning
                | crate::formula::model::CombatElement::Wind
        );
        let expiry_damage = expires
            && matches!(
                effect.plan.element,
                crate::formula::model::CombatElement::Ice
                    | crate::formula::model::CombatElement::Quantum
            );
        if let (true, Some(mut base)) = (is_dot || expiry_damage, effect.plan.base_damage) {
            if matches!(
                effect.plan.element,
                crate::formula::model::CombatElement::Wind
                    | crate::formula::model::CombatElement::Quantum
            ) {
                base = base
                    .checked_mul_integer(i64::from(effect.stacks))
                    .map_err(|_| numeric_fault(22, i64::from(effect.stacks)))?;
            }
            let mut definition = effect.damage;
            definition.mitigation_multiplier = dynamic_mitigation(
                catalog,
                txn,
                owner,
                crate::modifier::model::FormulaPurpose::Break,
                effect.plan.element,
                definition.mitigation_multiplier,
            )?;
            let damage = formula::toughness::break_effect_damage(definition, base, true)
                .map_err(|_| numeric_fault(23, base.scaled()))?;
            parent = apply_break_damage(
                catalog,
                txn,
                cause
                    .with_applier(effect.applier)
                    .with_source_definition(effect.source_definition),
                parent,
                BreakDamageApplication {
                    operation: effect.source_operation,
                    target: owner,
                    element: effect.plan.element,
                    kind: BreakDamageKind::Effect,
                    raw: damage.raw,
                    calculated: damage.finalized,
                },
            )?;
        }
        let remaining = effect.remaining_turns - 1;
        txn.update_break_effect(effect.id, remaining, effect.stacks)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(owner)),
            BattleEventKind::Toughness(ToughnessEventData::BaseEffectTicked {
                operation: effect.source_operation,
                target: owner,
                effect: effect.id,
                remaining_turns: remaining,
                stacks: effect.stacks,
            }),
        );
        if remaining == 0 {
            if let Some(speed) = effect.speed_before {
                txn.set_unit_speed(owner, speed)?;
            }
            skips_action |= effect.plan.skips_action;
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(owner)),
                BattleEventKind::Toughness(ToughnessEventData::BaseEffectExpired {
                    target: owner,
                    effect: effect.id,
                    element: effect.plan.element,
                }),
            );
        }
    }
    Ok((parent, skips_action))
}

fn dynamic_mitigation(
    catalog: &crate::catalog::CombatCatalog,
    txn: &Transaction<'_>,
    target: crate::UnitId,
    purpose: crate::modifier::model::FormulaPurpose,
    element: crate::formula::model::CombatElement,
    existing: crate::Ratio,
) -> Result<crate::Ratio, BattleFault> {
    let dynamic = super::operation_formula::FormulaInputs::new(txn)?
        .target_mitigation(catalog, txn, target, purpose, element)?;
    existing
        .checked_mul(dynamic, crate::Rounding::NearestTiesEven)
        .map_err(|_| numeric_fault(24, dynamic.scaled()))
}

pub(super) fn settle_effects_at_turn_start(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    owner: crate::UnitId,
) -> Result<EventId, BattleFault> {
    parent = super::effect_boundary::tick(
        catalog,
        txn,
        cause,
        parent,
        crate::EffectTickPhase::TurnStart,
        owner,
    )?;
    super::effect_duration::advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::TargetTurnStart,
        Some(owner),
    )
    .and_then(|parent| {
        super::effect_duration::advance_effect_clock(
            txn,
            cause,
            parent,
            crate::DurationClock::OwnerTurnStart,
            Some(owner),
        )
    })
}

pub(super) fn settle_effects_at_turn_end(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    owner: crate::UnitId,
) -> Result<EventId, BattleFault> {
    let parent = super::effect_boundary::tick(
        catalog,
        txn,
        cause,
        parent,
        crate::EffectTickPhase::TurnEnd,
        owner,
    )?;
    let parent = super::effect_duration::advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::TargetTurnEnd,
        Some(owner),
    )?;
    super::effect_duration::advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::OwnerTurnEnd,
        Some(owner),
    )
}

pub(super) fn settle_effects_at_action_end(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    let owner = match cause.actor() {
        Some(crate::CauseActor::Unit(unit)) => unit,
        Some(crate::CauseActor::TimelineActor(actor)) => txn
            .state
            .actors
            .get(actor)
            .map(|state| state.unit.unwrap_or(state.owner))
            .ok_or_else(|| invariant_fault(50))?,
        None => cause.applier().ok_or_else(|| invariant_fault(50))?,
    };
    let parent = super::effect_boundary::tick(
        catalog,
        txn,
        cause,
        parent,
        crate::EffectTickPhase::ActionEnd,
        owner,
    )?;
    txn.reset_rule_slots(
        crate::rule::model::SlotResetPoint::ActionEnd,
        cause.applier(),
    );
    super::effect_duration::advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::ActionEnd,
        None,
    )
}

pub(super) fn settle_effects_at_wave_end(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    super::effect_duration::advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::WaveEnd,
        None,
    )
}

pub(super) fn settle_effects_at_battle_end(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    super::effect_duration::advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::BattleEnd,
        None,
    )
}

fn execute_damage(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DamageOp,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let inputs = super::operation_formula::FormulaInputs::new(txn)?;
    for target in operation.targets {
        let critical = (operation.crit_policy != crate::catalog::action::HitCritPolicy::Never)
            .then(|| {
                inputs.critical_profile(catalog, txn, cause, operation.formula.class(), target)
            })
            .transpose()?;
        let is_critical = match operation.crit_policy {
            crate::catalog::action::HitCritPolicy::Never => false,
            crate::catalog::action::HitCritPolicy::Shared => {
                let critical = critical.as_ref().ok_or_else(|| invariant_fault(56))?;
                txn.roll_shared_probability(
                    critical.chance,
                    crate::rng::types::DrawPurpose::CRIT,
                    &mut scratch.shared_critical_draw,
                )?
            }
            crate::catalog::action::HitCritPolicy::PerTarget => {
                match scratch.critical_by_target.get(&target).copied() {
                    Some(value) => value,
                    None => {
                        let critical = critical.as_ref().ok_or_else(|| invariant_fault(56))?;
                        let value = txn.roll_probability(
                            critical.chance,
                            crate::rng::types::DrawPurpose::CRIT,
                        )?;
                        scratch.critical_by_target.insert(target, value);
                        value
                    }
                }
            }
        };
        let formula = if is_critical {
            let critical = critical.as_ref().ok_or_else(|| invariant_fault(56))?;
            operation
                .formula
                .with_formula_modifier(crate::modifier::model::FormulaStage::Crit, critical.damage)
                .map_err(|_| numeric_fault(50, critical.damage.scaled()))?
        } else {
            operation.formula
        };
        let calculation = inputs.damage(catalog, txn, cause, formula, operation.element, target)?;
        parent = apply_ordinary_damage_with_floor(
            catalog,
            txn,
            cause,
            parent,
            operation.id,
            target,
            DamageKind::Direct,
            operation.formula.class(),
            operation.element,
            None,
            calculation.raw,
            calculation.finalized,
            operation.minimum_hp,
        )?;
    }
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_ordinary_damage(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    operation: crate::OperationId,
    target: crate::UnitId,
    kind: DamageKind,
    class: crate::formula::model::DamageClass,
    element: Option<crate::formula::model::CombatElement>,
    source_effect: Option<crate::EffectInstanceId>,
    raw: crate::Scalar,
    calculated: crate::DamageAmount,
) -> Result<EventId, BattleFault> {
    apply_ordinary_damage_with_floor(
        catalog,
        txn,
        cause,
        parent,
        operation,
        target,
        kind,
        class,
        element,
        source_effect,
        raw,
        calculated,
        0,
    )
}

#[allow(clippy::too_many_arguments)]
fn apply_ordinary_damage_with_floor(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: crate::OperationId,
    target: crate::UnitId,
    kind: DamageKind,
    class: crate::formula::model::DamageClass,
    element: Option<crate::formula::model::CombatElement>,
    source_effect: Option<crate::EffectInstanceId>,
    raw: crate::Scalar,
    mut calculated: crate::DamageAmount,
    minimum_hp: i64,
) -> Result<EventId, BattleFault> {
    let (hp_before, life_before) = txn
        .state
        .units
        .get(target)
        .map(|unit| (unit.current_hp, unit.life))
        .ok_or_else(|| invariant_fault(1))?;
    (parent, calculated) =
        apply_damage_guard(catalog, txn, cause, parent, operation, target, calculated)?;
    let (absorbed, shield_changes) = txn
        .state
        .shields
        .absorb(target, calculated)
        .map_err(|_| numeric_fault(8, calculated.get()))?;
    for change in shield_changes {
        txn.record_shield_change(change.before, change.after);
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Shield(ShieldEventData::Absorbed {
                shield: change.id,
                target,
                before: change.before,
                after: change.after,
            }),
        );
    }
    let overflow_raw = calculated.get() - absorbed.get();
    let applied_raw = overflow_raw.min(hp_before.get().saturating_sub(minimum_hp));
    let applied = DamageAmount::new(applied_raw).map_err(|_| numeric_fault(2, applied_raw))?;
    let hp_after =
        Hp::new(hp_before.get() - applied_raw).map_err(|_| numeric_fault(3, hp_before.get()))?;
    txn.set_hp(target, hp_after)?;
    parent = txn.emit(
        cause.with_parent(parent).with_primary_target(Some(target)),
        BattleEventKind::Damage(DamageEventData {
            operation,
            kind,
            class,
            element,
            source_effect,
            target,
            raw,
            calculated,
            absorbed,
            applied,
            hp_before,
            hp_after,
        }),
    );
    if hp_after.get() == 0 && life_before == LifeState::Alive {
        txn.set_life(target, LifeState::Downed)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Unit(UnitEventData::Downed { unit: target }),
        );
        txn.set_life(target, LifeState::Defeated)?;
        let credited_to = cause.applier().ok_or_else(|| invariant_fault(2))?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Unit(UnitEventData::Defeated {
                unit: target,
                credited_to,
            }),
        );
        parent = super::lifecycle::settle_owner_defeat(txn, cause, parent, target)?;
    }
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn apply_damage_guard(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: crate::OperationId,
    target: crate::UnitId,
    calculated: DamageAmount,
) -> Result<(EventId, DamageAmount), BattleFault> {
    let shield = txn
        .state
        .shields
        .effective_remaining(target)
        .map_err(|_| numeric_fault(53, i64::try_from(target.get()).unwrap_or(i64::MAX)))?;
    if shield.get() == 0 || calculated.get() <= shield.get() {
        return Ok((parent, calculated));
    }
    let guard = txn.state.effects.iter_by_id().find_map(|effect| {
        (effect.target == target
            && catalog.effect(effect.definition).is_some_and(|definition| {
                definition.runtime().is_some_and(|runtime| {
                    runtime.damage_guard() == crate::EffectDamageGuard::ShieldOverflowOnce
                }) || definition.runtime_template().is_some_and(|runtime| {
                    runtime.damage_guard() == crate::EffectDamageGuard::ShieldOverflowOnce
                })
            }))
        .then_some(effect.id)
    });
    let Some(effect) = guard else {
        return Ok((parent, calculated));
    };
    let removed = txn
        .state
        .effects
        .remove(effect)
        .ok_or_else(|| invariant_fault(54))?;
    txn.remove_effect_attachments(effect);
    txn.record_effect_change(1, 0, effect.get());
    parent = txn.emit(
        cause.with_parent(parent).with_primary_target(Some(target)),
        BattleEventKind::Effect(EffectEventData::Removed {
            operation,
            effect,
            definition: removed.definition,
            target,
        }),
    );
    let guarded = DamageAmount::new(shield.get()).map_err(|_| numeric_fault(55, shield.get()))?;
    Ok((parent, guarded))
}

fn execute_apply_effect(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ApplyEffectOp,
) -> Result<EventId, BattleFault> {
    let definition = catalog
        .effect(operation.definition.effect)
        .ok_or_else(|| invariant_fault(30))?;
    if let Some(runtime) = &operation.resolved_runtime
        && runtime.len() != operation.targets.len()
    {
        return Err(invariant_fault(31));
    }
    if let Some(chances) = &operation.resolved_chances
        && chances.len() != operation.targets.len()
    {
        return Err(invariant_fault(31));
    }
    let source = cause
        .source_definition()
        .ok_or_else(|| invariant_fault(32))?;
    let applier = cause.applier().ok_or_else(|| invariant_fault(33))?;
    for (index, target) in operation.targets.into_iter().enumerate() {
        let runtime = operation
            .resolved_runtime
            .as_ref()
            .map(|values| &values[index])
            .or_else(|| definition.runtime())
            .ok_or_else(|| invariant_fault(31))?;
        let chance = operation
            .resolved_chances
            .as_ref()
            .map(|values| values[index])
            .unwrap_or(operation.definition.chance);
        let (pre_clamp, probability) = match chance {
            crate::EffectChancePolicy::Guaranteed => (crate::Scalar::ONE, crate::Probability::ONE),
            crate::EffectChancePolicy::Fixed { chance } => (
                crate::Scalar::from_scaled(i64::from(chance.millionths())),
                chance,
            ),
            crate::EffectChancePolicy::Resistible {
                base_chance,
                attacker_effect_hit_rate,
                target_effect_resistance,
                target_specific_resistance,
            } => {
                let value = formula::effect::resistible_chance(
                    base_chance,
                    attacker_effect_hit_rate,
                    target_effect_resistance,
                    target_specific_resistance,
                )
                .map_err(|_| numeric_fault(30, base_chance.scaled()))?;
                (value.pre_clamp, value.probability)
            }
        };
        if !txn.roll_probability(
            probability,
            operation
                .rng_purpose
                .unwrap_or(crate::rng::types::DrawPurpose::EFFECT_CHANCE),
        )? {
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Effect(EffectEventData::Resisted {
                    operation: operation.id,
                    definition: operation.definition.effect,
                    target,
                    pre_clamp_chance: pre_clamp,
                }),
            );
            continue;
        }
        let candidate_id = txn.allocate_effect();
        let candidate = crate::effect::state::EffectState::from_definition(
            candidate_id,
            operation.definition.effect,
            runtime,
            crate::effect::state::EffectApplicationContext {
                source_definition: source,
                source_operation: operation.id,
                applier,
                target,
                stacks: operation.definition.stacks,
            },
        );
        let removed_definitions = txn
            .state
            .effects
            .iter_by_id()
            .map(|effect| (effect.id, effect.definition))
            .collect::<std::collections::BTreeMap<_, _>>();
        let before = txn.state.effects.canonical_entries().len() as u64;
        let result = txn.state.effects.apply(candidate);
        let after = txn.state.effects.canonical_entries().len() as u64;
        txn.record_effect_change(before, after, candidate_id.get());
        match result {
            crate::effect::state::EffectApplyResult::Inserted { effect, removed } => {
                for removed in removed {
                    txn.remove_effect_attachments(removed);
                    parent = txn.emit(
                        cause.with_parent(parent).with_primary_target(Some(target)),
                        BattleEventKind::Effect(EffectEventData::Removed {
                            operation: operation.id,
                            effect: removed,
                            definition: *removed_definitions
                                .get(&removed)
                                .ok_or_else(|| invariant_fault(34))?,
                            target,
                        }),
                    );
                }
                let state = txn
                    .state
                    .effects
                    .get(effect)
                    .ok_or_else(|| invariant_fault(34))?;
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Effect(EffectEventData::Applied {
                        operation: operation.id,
                        effect,
                        definition: operation.definition.effect,
                        target,
                        stacks: state.stacks,
                        remaining: state.remaining,
                    }),
                );
                super::effect_operation::instantiate_attachments(catalog, txn, effect)?;
            }
            crate::effect::state::EffectApplyResult::Refreshed {
                effect,
                stacks_before,
                stacks_after,
            } => {
                super::modifier_snapshot::refresh_effect_stacks(
                    catalog,
                    txn,
                    effect,
                    stacks_after,
                )?;
                let remaining = txn
                    .state
                    .effects
                    .get(effect)
                    .and_then(|state| state.remaining);
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Effect(EffectEventData::Refreshed {
                        operation: operation.id,
                        effect,
                        target,
                        stacks_before,
                        stacks_after,
                        remaining,
                    }),
                );
            }
        }
    }
    Ok(parent)
}

fn execute_remove_effects(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: RemoveEffectsOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        let mut ids = txn.state.effects.removable_for(
            target,
            operation.definition.category,
            operation.definition.include_cleanseable_control,
            operation.definition.required_definition,
            operation.definition.required_tag,
        );
        if operation.definition.order == crate::EffectRemovalOrder::NewestFirst {
            ids.reverse();
        }
        for effect in ids
            .into_iter()
            .take(usize::from(operation.definition.maximum))
        {
            let before = txn.state.effects.canonical_entries().len() as u64;
            let removed = txn
                .state
                .effects
                .remove(effect)
                .ok_or_else(|| invariant_fault(35))?;
            txn.remove_effect_attachments(effect);
            let after = txn.state.effects.canonical_entries().len() as u64;
            txn.record_effect_change(before, after, effect.get());
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Effect(EffectEventData::Removed {
                    operation: operation.id,
                    effect,
                    definition: removed.definition,
                    target,
                }),
            );
        }
    }
    Ok(parent)
}
