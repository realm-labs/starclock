//! Ordinary-effect Entanglement: capture, one delay, hit facts and expiry damage.
use std::collections::BTreeSet;

use crate::{
    EffectEventData, EffectInstanceId, EventId, LifeState, PresenceState, UnitId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    effect::{model::EntanglementLevelSource, state::EntanglementState},
    event::{
        cause::Cause,
        model::{BattleEventKind, BreakDamageKind},
    },
    formula::{model::CombatElement, toughness},
    resolver::{
        operation::{
            self, BreakDamageApplication,
            fault::{invariant_fault, numeric_fault},
        },
        operation_formula::FormulaInputs,
        program_timeline,
        transaction::Transaction,
    },
};

pub(super) fn delay(
    txn: &mut Transaction<'_>,
    cause: Cause,
    parent: EventId,
    effect: EffectInstanceId,
) -> Result<EventId, BattleFault> {
    let state = txn
        .state
        .effects
        .get(effect)
        .ok_or_else(|| invariant_fault(82))?;
    let Some(entanglement) = state.entanglement else {
        return Ok(parent);
    };
    let target = state.target;
    program_timeline::shift_actions(
        txn,
        cause,
        parent,
        vec![target].into_boxed_slice(),
        entanglement.delay,
        false,
    )
}

pub(super) fn accumulate(
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
) -> Result<EventId, BattleFault> {
    if !txn
        .state
        .effects
        .iter_by_id()
        .any(|effect| effect.entanglement.is_some())
    {
        return Ok(parent);
    }
    // One positive calculated damage packet is enough for the completed hit;
    // shield absorption does not turn a damaging hit into a non-hit. Pure
    // application/heal/resource hits cannot build the delayed-damage counter.
    let damaged = txn
        .events
        .iter()
        .filter(|event| event.cause().hit() == cause.hit())
        .filter_map(|event| match event.kind() {
            BattleEventKind::Damage(data) if data.calculated.get() > 0 => Some(data.target),
            BattleEventKind::BreakDamage(data) if data.calculated.get() > 0 => Some(data.target),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let effects = txn
        .state
        .effects
        .iter_by_id()
        .filter_map(|effect| {
            let delayed = effect.entanglement?;
            let eligible = txn.state.units.get(effect.target).is_some_and(|unit| {
                unit.life == LifeState::Alive && unit.presence == PresenceState::Present
            });
            (eligible
                && damaged.contains(&effect.target)
                && delayed.hits < 5
                && (delayed.application_hit.is_none() || delayed.application_hit != cause.hit()))
            .then_some((
                effect.id,
                effect.target,
                effect.source_operation,
                delayed.hits,
            ))
        })
        .collect::<Vec<_>>();
    for (id, target, operation, before) in effects {
        let after = before
            .checked_add(1)
            .ok_or_else(|| numeric_fault(83, i64::from(before)))?;
        txn.state
            .effects
            .get_mut(id)
            .and_then(|effect| effect.entanglement.as_mut())
            .ok_or_else(|| invariant_fault(83))?
            .hits = after;
        txn.record_effect_change(u64::from(before), u64::from(after), id.get());
        parent = txn.emit(
            cause.with_parent(parent).with_primary_target(Some(target)),
            BattleEventKind::Effect(EffectEventData::HitAccumulated {
                operation,
                effect: id,
                target,
                before,
                after,
            }),
        );
    }
    Ok(parent)
}

pub(super) fn tick(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    owner: UnitId,
) -> Result<EventId, BattleFault> {
    let effects = txn
        .state
        .effects
        .iter_by_id()
        .filter(|effect| {
            effect.target == owner && effect.remaining == Some(1) && effect.entanglement.is_some()
        })
        .cloned()
        .collect::<Vec<_>>();
    for effect in effects {
        let Some(delayed) = effect.entanglement else {
            continue;
        };
        if !txn.state.units.get(owner).is_some_and(|unit| {
            unit.life == LifeState::Alive && unit.presence == PresenceState::Present
        }) {
            continue;
        }
        let base = delayed
            .base
            .checked_mul_integer(i64::from(delayed.hits))
            .map_err(|_| numeric_fault(84, delayed.base.scaled()))?;
        let attributed = cause
            .with_applier(effect.applier)
            .with_source_definition(effect.source_definition);
        let inputs = FormulaInputs::new(txn)?;
        let definition = inputs.break_damage(
            catalog,
            txn,
            attributed,
            delayed.damage,
            CombatElement::Quantum,
            owner,
        )?;
        let broken = txn
            .state
            .units
            .get(owner)
            .ok_or_else(|| invariant_fault(84))?
            .weakness_broken;
        let damage = toughness::break_effect_damage(definition, base, broken)
            .map_err(|_| numeric_fault(85, base.scaled()))?;
        parent = operation::apply_break_damage(
            catalog,
            txn,
            attributed,
            parent,
            BreakDamageApplication {
                operation: effect.source_operation,
                target: owner,
                element: CombatElement::Quantum,
                kind: BreakDamageKind::Effect,
                raw: damage.raw,
            },
        )?;
    }
    Ok(parent)
}

pub(super) fn capture(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    cause: Cause,
    target: UnitId,
    mut damage: toughness::BreakDamageDefinition,
    level_source: EntanglementLevelSource,
) -> Result<EntanglementState, BattleFault> {
    if level_source == EntanglementLevelSource::Applier {
        let applier = cause.applier().ok_or_else(|| invariant_fault(86))?;
        let level = txn
            .state
            .units
            .get(applier)
            .ok_or_else(|| invariant_fault(86))?
            .level;
        damage.attacker_level_multiplier = toughness::attacker_level_multiplier(level)
            .ok_or_else(|| numeric_fault(86, i64::from(level.get())))?;
    }
    let plan = FormulaInputs::new(txn)?.entanglement_plan(catalog, txn, cause, target, damage)?;
    Ok(EntanglementState {
        damage,
        base: plan.base_damage.ok_or_else(|| invariant_fault(85))?,
        delay: plan.additional_delay,
        hits: 0,
        application_hit: cause.hit(),
    })
}
