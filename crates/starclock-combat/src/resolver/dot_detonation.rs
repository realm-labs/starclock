//! One canonical candidate pool over ordinary and explicitly admitted Break DoTs.

use super::{
    operation::fault::{invariant_fault, numeric_fault},
    operation::{BreakDamageApplication, apply_finalized_break_damage, apply_ordinary_damage},
    operation_formula::{FormulaInputs, final_damage::FinalBreakDamage},
    transaction::Transaction,
};
use crate::{
    DamageAmount, DamageKind, DotDetonationDefinition, DotDetonationScope, DotDetonationSelection,
    EffectInstanceId, EventId, OperationId, Rounding,
    battle::fault::BattleFault,
    catalog::{CombatCatalog, action::OrdinaryDamageDefinition},
    effect::{break_effect::BreakEffectState, state::EffectState},
    event::{
        cause::Cause,
        model::{BattleEventKind, BreakDamageKind, EffectEventData, ToughnessEventData},
    },
    formula::{sustain::DamageCalculation, toughness::break_effect_damage},
    modifier::model::FormulaPurpose,
    operation::DetonateDotsOp,
};

enum Candidate {
    Ordinary(EffectState),
    Break(BreakEffectState),
}

impl Candidate {
    fn id(&self) -> EffectInstanceId {
        match self {
            Self::Ordinary(effect) => effect.id,
            Self::Break(effect) => effect.id,
        }
    }
}

#[derive(Clone, Copy)]
struct DetonationContext {
    cause: Cause,
    operation: OperationId,
    definition: DotDetonationDefinition,
}

pub(super) fn execute(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: DetonateDotsOp,
) -> Result<EventId, BattleFault> {
    let inputs = FormulaInputs::new(txn)?;
    let definition = operation.definition;
    let context = DetonationContext {
        cause,
        operation: operation.id,
        definition,
    };
    for target in operation.targets {
        let filter = definition.filter();
        let mut candidates = txn
            .state
            .effects
            .dots_for(target, definition.required_tag())
            .into_iter()
            .filter(|effect| {
                filter.excluded_effect() != Some(effect.definition)
                    && filter.required_family().is_none_or(|family| {
                        catalog
                            .effect(effect.definition)
                            .is_some_and(|definition| definition.dot_family() == Some(family))
                    })
            })
            .map(Candidate::Ordinary)
            .collect::<Vec<_>>();
        // Base Break statuses have no authored tags or effect-definition ID.
        if definition.scope() == DotDetonationScope::OrdinaryAndBreakEffects
            && definition.required_tag().is_none()
        {
            candidates.extend(
                txn.state
                    .break_effects
                    .active_for(target)
                    .into_iter()
                    .filter(|effect| {
                        effect.dot_family().is_some_and(|family| {
                            filter
                                .required_family()
                                .is_none_or(|required| required == family)
                        })
                    })
                    .map(Candidate::Break),
            );
        }
        candidates.sort_unstable_by_key(Candidate::id);
        if let DotDetonationSelection::RandomOne(purpose) = definition.selection()
            && candidates.len() > 1
        {
            let index = txn
                .choose_index(purpose, candidates.len())?
                .ok_or_else(|| invariant_fault(43))?;
            candidates = vec![candidates.swap_remove(index)];
        }
        for candidate in candidates {
            parent = match candidate {
                Candidate::Ordinary(effect) => {
                    ordinary(catalog, txn, &inputs, parent, context, effect)?
                }
                Candidate::Break(effect) => {
                    break_dot(catalog, txn, &inputs, parent, context, effect)?
                }
            };
        }
    }
    Ok(parent)
}

fn ordinary(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    inputs: &FormulaInputs,
    parent: EventId,
    context: DetonationContext,
    effect: EffectState,
) -> Result<EventId, BattleFault> {
    let dot = effect.dot.ok_or_else(|| invariant_fault(36))?;
    let per_stack = dot.formula();
    let base = per_stack
        .base_damage()
        .checked_mul_integer(i64::from(effect.stacks))
        .map_err(|_| numeric_fault(31, per_stack.base_damage().scaled()))?;
    let formula = OrdinaryDamageDefinition::new(base, per_stack.multipliers())
        .map_err(|_| numeric_fault(31, base.scaled()))?
        .with_class(per_stack.class());
    let attributed = context
        .cause
        .with_applier(effect.applier)
        .with_source_definition(effect.source_definition);
    let calculation = inputs.damage(
        catalog,
        txn,
        attributed,
        formula,
        Some(dot.element()),
        effect.target,
        true,
        false,
    )?;
    let damage = fraction(calculation, context.definition)?;
    let parent = apply_ordinary_damage(
        catalog,
        txn,
        attributed,
        parent,
        context.operation,
        effect.target,
        DamageKind::DotDetonation,
        formula.class(),
        Some(dot.element()),
        Some(effect.id),
        damage.raw,
        damage.finalized,
    )?;
    Ok(txn.emit(
        attributed
            .with_parent(parent)
            .with_primary_target(Some(effect.target)),
        BattleEventKind::Effect(EffectEventData::Detonated {
            operation: context.operation,
            effect: effect.id,
            target: effect.target,
            fraction: context.definition.fraction(),
        }),
    ))
}

fn break_dot(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    inputs: &FormulaInputs,
    parent: EventId,
    context: DetonationContext,
    effect: BreakEffectState,
) -> Result<EventId, BattleFault> {
    let base = effect
        .damage_base()
        .map_err(|_| numeric_fault(22, i64::from(effect.stacks)))?
        .ok_or_else(|| invariant_fault(44))?;
    let attributed = context
        .cause
        .with_applier(effect.applier)
        .with_source_definition(effect.source_definition);
    let definition = inputs.break_damage(
        catalog,
        txn,
        attributed,
        effect.damage,
        effect.plan.element,
        effect.owner,
    )?;
    let damage = break_effect_damage(definition, base, true)
        .map_err(|_| numeric_fault(23, base.scaled()))?;
    let calculation = inputs.final_break_damage(
        catalog,
        txn,
        attributed,
        FinalBreakDamage {
            target: effect.owner,
            element: effect.plan.element,
            purpose: FormulaPurpose::Break,
            raw: damage.raw,
        },
    )?;
    let calculation = fraction(calculation, context.definition)?;
    let parent = apply_finalized_break_damage(
        catalog,
        txn,
        attributed,
        parent,
        BreakDamageApplication {
            operation: context.operation,
            target: effect.owner,
            element: effect.plan.element,
            kind: BreakDamageKind::EffectDetonation,
            raw: calculation.raw,
        },
        calculation,
    )?;
    Ok(txn.emit(
        attributed
            .with_parent(parent)
            .with_primary_target(Some(effect.owner)),
        BattleEventKind::Toughness(ToughnessEventData::BaseEffectDetonated {
            operation: context.operation,
            target: effect.owner,
            effect: effect.id,
            element: effect.plan.element,
            fraction: context.definition.fraction(),
        }),
    ))
}

fn fraction(
    calculation: DamageCalculation,
    definition: DotDetonationDefinition,
) -> Result<DamageCalculation, BattleFault> {
    let raw = definition
        .fraction()
        .checked_apply(calculation.raw, Rounding::NearestTiesEven)
        .map_err(|_| numeric_fault(32, calculation.raw.scaled()))?;
    Ok(DamageCalculation {
        raw,
        finalized: DamageAmount::from_scalar(raw, Rounding::Floor)
            .map_err(|_| numeric_fault(33, raw.scaled()))?,
    })
}
