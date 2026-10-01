//! One shared hit-policy decision path for ordinary and dedicated damage.
use super::fault::invariant_fault;
use crate::{
    Scalar, UnitId,
    battle::fault::BattleFault,
    catalog::{CombatCatalog, action::HitCritPolicy},
    event::cause::Cause,
    formula::model::{CombatElement, DamageClass},
    operation::HitOperationScratch,
    resolver::{
        operation_formula::{CriticalQuery, FormulaInputs},
        transaction::Transaction,
    },
    rng::types::DrawPurpose,
};

pub(super) struct CriticalRequest {
    pub(super) target: UnitId,
    pub(super) class: DamageClass,
    pub(super) policy: HitCritPolicy,
    pub(super) ultimate_semantics: bool,
    pub(super) element: Option<CombatElement>,
}
pub(in crate::resolver) struct CriticalResolution {
    pub(in crate::resolver) is_critical: bool,
    pub(in crate::resolver) damage: Scalar,
}
pub(super) fn resolve(
    inputs: &FormulaInputs,
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    request: CriticalRequest,
    scratch: &mut HitOperationScratch,
) -> Result<CriticalResolution, BattleFault> {
    let critical = (request.policy != HitCritPolicy::Never)
        .then(|| {
            inputs.critical_profile(
                catalog,
                txn,
                cause,
                CriticalQuery {
                    class: request.class,
                    target: request.target,
                    ultimate_semantics: request.ultimate_semantics,
                    element: request.element,
                },
            )
        })
        .transpose()?;
    let is_critical = match request.policy {
        HitCritPolicy::Never => false,
        HitCritPolicy::Shared => {
            let profile = critical.as_ref().ok_or_else(|| invariant_fault(56))?;
            txn.roll_shared_probability(
                profile.chance,
                DrawPurpose::CRIT,
                &mut scratch.shared_critical_draw,
            )?
        }
        HitCritPolicy::GuaranteedBelowHpRatio(threshold)
            if {
                let unit = txn
                    .state
                    .units
                    .get(request.target)
                    .ok_or_else(|| invariant_fault(57))?;
                i128::from(unit.current_hp.get()) * 1_000_000
                    < i128::from(unit.maximum_hp.get()) * i128::from(threshold.scaled())
            } =>
        {
            true
        }
        HitCritPolicy::PerTarget | HitCritPolicy::GuaranteedBelowHpRatio(_) => {
            match scratch.critical_by_target.get(&request.target).copied() {
                Some(value) => value,
                None => {
                    let profile = critical.as_ref().ok_or_else(|| invariant_fault(56))?;
                    let value = txn.roll_probability(profile.chance, DrawPurpose::CRIT)?;
                    scratch.critical_by_target.insert(request.target, value);
                    value
                }
            }
        }
    };
    let damage = if is_critical {
        critical.ok_or_else(|| invariant_fault(56))?.damage
    } else {
        Scalar::ZERO
    };
    Ok(CriticalResolution {
        is_critical,
        damage,
    })
}
