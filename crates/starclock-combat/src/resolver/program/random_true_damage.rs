//! Bounded per-hit random-target True DMG emitted by typed Rule IR.

use crate::{
    EventId, Ratio, Rounding, Scalar, SelectorId, UnitId,
    battle::fault::BattleFault,
    catalog::{
        CombatCatalog,
        action::{HitCritPolicy, OrdinaryDamageDefinition, OrdinaryDamageMultipliers},
    },
    event::cause::Cause,
    formula::{model::DamageClass, toughness::EnemyRank},
    operation::{DamageOp, HitOperationScratch, Operation},
    rng::types::DrawPurpose,
    rule::model::RuleValue,
};

use crate::resolver::{operation::execute_operation, transaction::Transaction};

use super::{AbilityProgramContext, emission_targets, program_fault, scale};

pub(super) struct Request<'a> {
    pub context: &'a AbilityProgramContext,
    pub resolved: &'a [(SelectorId, Box<[UnitId]>)],
    pub selector: SelectorId,
    pub repetitions: RuleValue,
    pub maximum_repetitions: u16,
    pub coefficients: [Scalar; 3],
    pub target_rng_purpose: DrawPurpose,
    pub current_target: Option<UnitId>,
}

pub(super) fn execute(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    request: Request<'_>,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let count = match request.repetitions {
        RuleValue::Integer(value) => u16::try_from(value)
            .ok()
            .filter(|value| *value <= request.maximum_repetitions)
            .ok_or_else(|| program_fault(87, value))?,
        _ => return Err(program_fault(87, 0)),
    };
    let targets = emission_targets(
        catalog,
        request.resolved,
        request.selector,
        request.current_target,
    )?;
    for _ in 0..count {
        if targets.is_empty() {
            break;
        }
        let index = txn
            .choose_index(request.target_rng_purpose, targets.len())?
            .ok_or_else(|| program_fault(87, 1))?;
        let target = *targets
            .get(index)
            .ok_or_else(|| program_fault(87, i64::try_from(index).unwrap_or(i64::MAX)))?;
        let unit =
            txn.state.units.get(target).ok_or_else(|| {
                program_fault(87, i64::try_from(target.get()).unwrap_or(i64::MAX))
            })?;
        let coefficient = match unit.rank {
            EnemyRank::Normal => request.coefficients[0],
            EnemyRank::Elite => request.coefficients[1],
            EnemyRank::Boss => request.coefficients[2],
        };
        let base_hp = Scalar::checked_from_integer(unit.maximum_hp.get())
            .map_err(|_| program_fault(87, unit.maximum_hp.get()))?;
        let amount = Ratio::from_scaled(coefficient.scaled())
            .checked_apply(base_hp, Rounding::Floor)
            .map_err(|_| program_fault(87, coefficient.scaled()))?;
        let amount = scale(amount, request.context.damage_share)?;
        let formula = OrdinaryDamageDefinition::new(
            amount,
            OrdinaryDamageMultipliers::new([Ratio::ONE; 9]).expect("neutral multipliers are valid"),
        )
        .map_err(|_| program_fault(87, amount.scaled()))?
        .with_class(DamageClass::Additional);
        let operation = Operation::Damage(DamageOp {
            id: txn.allocate_operation(),
            targets: vec![target].into_boxed_slice(),
            formula,
            element: None,
            crit_policy: HitCritPolicy::Never,
            apply_source_modifiers: false,
            ultimate_semantics: false,
            minimum_hp: 0,
        });
        parent = execute_operation(catalog, txn, cause, parent, operation, scratch)?;
    }
    Ok(parent)
}
