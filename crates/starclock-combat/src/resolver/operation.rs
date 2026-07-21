use crate::{
    DamageAmount, HealingAmount, Hp, LifeState,
    battle::fault::BattleFault,
    event::{
        cause::Cause,
        model::{
            BattleEventKind, BreakDamageEventData, BreakDamageKind, DamageEventData, DamageKind,
            EffectEventData, HealEventData, HpConsumptionEventData, ShieldEventData,
            ToughnessEventData, UnitEventData,
        },
    },
    formula,
    id::EventId,
    operation::{
        AddWeaknessOp, ApplyEffectOp, ConsumeHpOp, DamageOp, DetonateDotsOp, HealOp,
        HitOperationScratch, Operation, ReduceToughnessOp, RemoveEffectsOp, ShieldOp, SuperBreakOp,
    },
};

use super::transaction::Transaction;

mod fault;

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
        Operation::Damage(operation) => execute_damage(txn, cause, parent, operation),
        Operation::Heal(operation) => execute_heal(txn, cause, parent, operation),
        Operation::Shield(operation) => execute_shield(txn, cause, parent, operation),
        Operation::ConsumeHp(operation) => execute_hp_consumption(txn, cause, parent, operation),
        Operation::AddWeakness(operation) => execute_add_weakness(txn, cause, parent, operation),
        Operation::ReduceToughness(operation) => {
            execute_toughness_reduction(txn, cause, parent, operation, scratch)
        }
        Operation::SuperBreak(operation) => {
            execute_super_break(txn, cause, parent, operation, scratch)
        }
        Operation::ApplyEffect(operation) => {
            execute_apply_effect(catalog, txn, cause, parent, operation)
        }
        Operation::RemoveEffects(operation) => {
            execute_remove_effects(txn, cause, parent, operation)
        }
        Operation::DetonateDots(operation) => execute_detonate_dots(txn, cause, parent, operation),
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

fn execute_toughness_reduction(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ReduceToughnessOp,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let calculation = formula::toughness::reduction(operation.definition.reduction)
        .map_err(|_| numeric_fault(11, operation.definition.reduction.base.get()))?;
    for target in operation.targets {
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
        let routed = crate::toughness::state::route_reduction(
            &mut layers,
            &weaknesses,
            was_broken,
            operation.definition.element,
            calculation.attempted,
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
            let damage = formula::toughness::break_damage(
                operation.definition.break_damage,
                value.break_element,
                value.maximum,
                was_broken,
            )
            .map_err(|_| numeric_fault(12, value.maximum.get()))?;
            parent = apply_break_damage(
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
                operation.definition.break_effect_chance,
                crate::rng::types::DrawPurpose::EFFECT_CHANCE,
            )?;
            if applied {
                let plan = formula::toughness::base_break_effect(
                    value.break_element,
                    rank,
                    max_hp,
                    operation.definition.break_damage.attacker_level_multiplier,
                    value.maximum,
                    operation.definition.break_damage.break_effect,
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
                    damage: operation.definition.break_damage,
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
        let damage = formula::toughness::super_break_damage(operation.definition, effective)
            .map_err(|_| numeric_fault(15, effective.get()))?;
        parent = apply_break_damage(
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
        calculated,
    } = application;
    let (hp_before, life_before) = txn
        .state
        .units
        .get(target)
        .map(|unit| (unit.current_hp, unit.life))
        .ok_or_else(|| invariant_fault(9))?;
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
            let damage = formula::toughness::break_effect_damage(effect.damage, base, true)
                .map_err(|_| numeric_fault(23, base.scaled()))?;
            parent = apply_break_damage(
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

pub(super) fn settle_effects_at_turn_start(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    owner: crate::UnitId,
) -> Result<EventId, BattleFault> {
    let effects = txn
        .state
        .effects
        .iter_by_id()
        .filter(|effect| {
            effect.target == owner && effect.tick_phase == crate::EffectTickPhase::TurnStart
        })
        .cloned()
        .collect::<Vec<_>>();
    for effect in effects {
        if let Some(dot) = effect.dot {
            let calculation = formula::ordinary_damage(dot.formula())
                .map_err(|_| numeric_fault(34, dot.formula().base_damage().scaled()))?;
            let attributed = cause
                .with_applier(effect.applier)
                .with_source_definition(effect.source_definition);
            parent = apply_ordinary_damage(
                txn,
                attributed,
                parent,
                effect.source_operation,
                owner,
                DamageKind::DotTick,
                dot.formula().class(),
                Some(dot.element()),
                Some(effect.id),
                calculation.raw,
                calculation.finalized,
            )?;
        }
    }
    advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::TargetTurnStart,
        Some(owner),
    )
    .and_then(|parent| {
        advance_effect_clock(
            txn,
            cause,
            parent,
            crate::DurationClock::OwnerTurnStart,
            Some(owner),
        )
    })
}

pub(super) fn settle_effects_at_turn_end(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    owner: crate::UnitId,
) -> Result<EventId, BattleFault> {
    let parent = advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::TargetTurnEnd,
        Some(owner),
    )?;
    advance_effect_clock(
        txn,
        cause,
        parent,
        crate::DurationClock::OwnerTurnEnd,
        Some(owner),
    )
}

pub(super) fn settle_effects_at_action_end(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
) -> Result<EventId, BattleFault> {
    txn.reset_rule_slots(
        crate::rule::model::SlotResetPoint::ActionEnd,
        cause.applier(),
    );
    advance_effect_clock(txn, cause, parent, crate::DurationClock::ActionEnd, None)
}

fn advance_effect_clock(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    clock: crate::DurationClock,
    owner: Option<crate::UnitId>,
) -> Result<EventId, BattleFault> {
    let ids =
        txn.state
            .effects
            .iter_by_id()
            .filter(|effect| {
                effect.duration_clock == clock
                    && match clock {
                        crate::DurationClock::OwnerTurnStart
                        | crate::DurationClock::OwnerTurnEnd => owner == Some(effect.applier),
                        crate::DurationClock::TargetTurnStart
                        | crate::DurationClock::TargetTurnEnd => owner == Some(effect.target),
                        _ => true,
                    }
            })
            .map(|effect| effect.id)
            .collect::<Vec<_>>();
    for id in ids {
        let (operation, target, before, after) = {
            let effect = txn
                .state
                .effects
                .get_mut(id)
                .ok_or_else(|| invariant_fault(37))?;
            let before = effect.remaining.ok_or_else(|| invariant_fault(38))?;
            let after = before.checked_sub(1).ok_or_else(|| invariant_fault(39))?;
            effect.remaining = Some(after);
            (effect.source_operation, effect.target, before, after)
        };
        txn.record_effect_change(u64::from(before) + 1, u64::from(after) + 1, id.get());
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Effect(EffectEventData::Ticked {
                operation,
                effect: id,
                target,
                remaining: Some(after),
            }),
        );
        if after == 0 {
            txn.state
                .effects
                .remove(id)
                .ok_or_else(|| invariant_fault(40))?;
            txn.remove_effect_attachments(id);
            txn.record_effect_change(1, 0, id.get());
            parent = txn.emit(
                cause.with_parent(parent).with_primary_target(Some(target)),
                BattleEventKind::Effect(EffectEventData::Removed {
                    operation,
                    effect: id,
                    target,
                }),
            );
        }
    }
    Ok(parent)
}

fn execute_damage(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DamageOp,
) -> Result<EventId, BattleFault> {
    let calculation = formula::ordinary_damage(operation.formula)
        .map_err(|_| numeric_fault(1, operation.formula.base_damage().scaled()))?;
    for target in operation.targets {
        parent = apply_ordinary_damage(
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
        )?;
    }
    Ok(parent)
}

#[allow(clippy::too_many_arguments)]
fn apply_ordinary_damage(
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
    calculated: crate::DamageAmount,
) -> Result<EventId, BattleFault> {
    let (hp_before, life_before) = txn
        .state
        .units
        .get(target)
        .map(|unit| (unit.current_hp, unit.life))
        .ok_or_else(|| invariant_fault(1))?;
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
    let applied_raw = overflow_raw.min(hp_before.get());
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
        let (pre_clamp, probability) = match operation.definition.chance {
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
                .map_err(|_| numeric_fault(30, i64::from(base_chance.millionths())))?;
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
                instantiate_effect_attachments(catalog, txn, effect)?;
            }
            crate::effect::state::EffectApplyResult::Refreshed {
                effect,
                stacks_before,
                stacks_after,
            } => {
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
        let ids = txn.state.effects.removable_for(
            target,
            operation.definition.category,
            operation.definition.required_definition,
            operation.definition.required_tag,
        );
        for effect in ids
            .into_iter()
            .take(usize::from(operation.definition.maximum))
        {
            let before = txn.state.effects.canonical_entries().len() as u64;
            txn.state
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
                    target,
                }),
            );
        }
    }
    Ok(parent)
}

fn instantiate_effect_attachments(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    effect: crate::EffectInstanceId,
) -> Result<(), BattleFault> {
    let state = txn
        .state
        .effects
        .get(effect)
        .cloned()
        .ok_or_else(|| invariant_fault(38))?;
    let definition = catalog
        .effect(state.definition)
        .ok_or_else(|| invariant_fault(39))?;
    for modifier in definition.modifiers() {
        let instance = txn.allocate_modifier();
        txn.insert_modifier(crate::modifier::model::ActiveModifier {
            instance,
            definition: *modifier,
            owner: state.applier,
            subject: state.target,
            source: state.source_definition,
            source_class: crate::rule::model::SourceClass::Effect,
            insertion_sequence: instance.get(),
            application_action: None,
            source_effect: Some(effect),
            slots: Box::new([]),
            captured_value: None,
            captured_stats: Box::new([]),
        })?;
    }
    for rule in definition.rules() {
        let runtime = catalog
            .rule(*rule)
            .and_then(crate::catalog::definition::RuleDefinition::runtime)
            .ok_or_else(|| invariant_fault(40))?;
        let instance = txn.allocate_rule();
        if !txn
            .state
            .rules
            .insert_attached(instance, *rule, state.target, effect, runtime)
        {
            return Err(invariant_fault(41));
        }
    }
    Ok(())
}

fn execute_detonate_dots(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DetonateDotsOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        for effect in txn
            .state
            .effects
            .dots_for(target, operation.definition.required_tag)
        {
            let dot = effect.dot.ok_or_else(|| invariant_fault(36))?;
            let calculation = formula::ordinary_damage(dot.formula())
                .map_err(|_| numeric_fault(31, dot.formula().base_damage().scaled()))?;
            let raw = operation
                .definition
                .fraction
                .checked_apply(calculation.raw, crate::Rounding::NearestTiesEven)
                .map_err(|_| numeric_fault(32, calculation.raw.scaled()))?;
            let finalized = crate::DamageAmount::from_scalar(raw, crate::Rounding::Floor)
                .map_err(|_| numeric_fault(33, raw.scaled()))?;
            let attributed = cause
                .with_applier(effect.applier)
                .with_source_definition(effect.source_definition);
            parent = apply_ordinary_damage(
                txn,
                attributed,
                parent,
                operation.id,
                target,
                DamageKind::DotDetonation,
                dot.formula().class(),
                Some(dot.element()),
                Some(effect.id),
                raw,
                finalized,
            )?;
            parent = txn.emit(
                attributed
                    .with_parent(parent)
                    .with_primary_target(Some(target)),
                BattleEventKind::Effect(EffectEventData::Detonated {
                    operation: operation.id,
                    effect: effect.id,
                    target,
                    fraction: operation.definition.fraction,
                }),
            );
        }
    }
    Ok(parent)
}

fn execute_shield(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ShieldOp,
) -> Result<EventId, BattleFault> {
    let context = formula::model::ShieldContext {
        scaling_terms: vec![formula::model::ScalingTerm {
            stat: operation.formula.base_shield(),
            ratio: crate::Ratio::ONE,
        }]
        .into_boxed_slice(),
        additive_base: crate::Scalar::ZERO,
        bonuses: vec![operation.formula.bonus()].into_boxed_slice(),
    };
    let calculation = formula::shield::calculate(&context)
        .map_err(|_| numeric_fault(9, operation.formula.base_shield().scaled()))?;
    for target in operation.targets {
        let shield = txn.allocate_shield();
        txn.state
            .shields
            .insert(crate::effect::shield::ShieldState {
                id: shield,
                owner: target,
                source_operation: operation.id,
                remaining: calculation.finalized,
                policy: operation.formula.policy(),
            })
            .map_err(|_| invariant_fault(4))?;
        txn.record_shield_change(
            crate::ShieldAmount::new(0).expect("zero shield amount is valid"),
            calculation.finalized,
        );
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Shield(ShieldEventData::Applied {
                operation: operation.id,
                shield,
                target,
                raw: calculation.raw,
                amount: calculation.finalized,
            }),
        );
    }
    Ok(parent)
}

fn execute_hp_consumption(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ConsumeHpOp,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        let before = txn
            .state
            .units
            .get(target)
            .map(|unit| unit.current_hp)
            .ok_or_else(|| invariant_fault(5))?;
        let result = formula::hp::consume(
            before,
            operation.definition.requested(),
            operation.definition.floor(),
        )
        .map_err(|_| numeric_fault(10, operation.definition.requested().get()))?;
        txn.set_hp(target, result.after)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::HpConsumption(HpConsumptionEventData {
                operation: operation.id,
                target,
                requested: result.requested,
                effective: result.effective,
                overflow: result.overflow,
                hp_before: result.before,
                hp_after: result.after,
            }),
        );
    }
    Ok(parent)
}

fn execute_heal(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: HealOp,
) -> Result<EventId, BattleFault> {
    let calculation = formula::healing(operation.formula)
        .map_err(|_| numeric_fault(4, operation.formula.base_healing().scaled()))?;
    for target in operation.targets {
        let (hp_before, maximum_hp, life) = txn
            .state
            .units
            .get(target)
            .map(|unit| (unit.current_hp, unit.maximum_hp, unit.life))
            .ok_or_else(|| invariant_fault(3))?;
        let missing = if life == LifeState::Alive {
            maximum_hp.get() - hp_before.get()
        } else {
            0
        };
        let effective_raw = calculation.finalized.get().min(missing);
        let overheal_raw = calculation.finalized.get() - effective_raw;
        let effective =
            HealingAmount::new(effective_raw).map_err(|_| numeric_fault(5, effective_raw))?;
        let overheal =
            HealingAmount::new(overheal_raw).map_err(|_| numeric_fault(6, overheal_raw))?;
        let hp_after = Hp::new(hp_before.get() + effective_raw)
            .map_err(|_| numeric_fault(7, hp_before.get()))?;
        txn.set_hp(target, hp_after)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Heal(HealEventData {
                operation: operation.id,
                target,
                raw: calculation.raw,
                calculated: calculation.finalized,
                effective,
                overheal,
                hp_before,
                hp_after,
            }),
        );
    }
    Ok(parent)
}
