use crate::{
    DamageAmount, HealingAmount, Hp, LifeState,
    battle::fault::{BattleFault, FaultBoundary, FaultKind, FaultPolicy},
    event::{
        cause::Cause,
        model::{
            BattleEventKind, BreakDamageEventData, BreakDamageKind, DamageEventData, HealEventData,
            HpConsumptionEventData, ShieldEventData, ToughnessEventData, UnitEventData,
        },
    },
    formula,
    id::EventId,
    operation::{
        AddWeaknessOp, ConsumeHpOp, DamageOp, HealOp, HitOperationScratch, Operation,
        ReduceToughnessOp, ShieldOp, SuperBreakOp,
    },
};

use super::transaction::Transaction;

pub(super) fn execute_operation(
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
            let applied = txn.roll_probability(operation.definition.break_effect_chance)?;
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

fn execute_damage(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DamageOp,
) -> Result<EventId, BattleFault> {
    let calculation = formula::ordinary_damage(operation.formula)
        .map_err(|_| numeric_fault(1, operation.formula.base_damage().scaled()))?;
    for target in operation.targets {
        let (hp_before, life_before) = txn
            .state
            .units
            .get(target)
            .map(|unit| (unit.current_hp, unit.life))
            .ok_or_else(|| invariant_fault(1))?;
        let (absorbed, shield_changes) = txn
            .state
            .shields
            .absorb(target, calculation.finalized)
            .map_err(|_| numeric_fault(8, calculation.finalized.get()))?;
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
        let overflow_raw = calculation.finalized.get() - absorbed.get();
        let applied_raw = overflow_raw.min(hp_before.get());
        let applied = DamageAmount::new(applied_raw).map_err(|_| numeric_fault(2, applied_raw))?;
        let hp_after = Hp::new(hp_before.get() - applied_raw)
            .map_err(|_| numeric_fault(3, hp_before.get()))?;
        txn.set_hp(target, hp_after)?;
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Damage(DamageEventData {
                operation: operation.id,
                target,
                raw: calculation.raw,
                calculated: calculation.finalized,
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

fn numeric_fault(context: u32, value: i64) -> BattleFault {
    BattleFault::new(
        FaultKind::Numeric,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3200 + context,
        Some(value),
    )
}

fn invariant_fault(context: u32) -> BattleFault {
    BattleFault::new(
        FaultKind::InvariantViolation,
        FaultBoundary::Command,
        FaultPolicy::Rollback,
        0x3280 + context,
        None,
    )
}
