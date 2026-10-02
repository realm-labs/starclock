//! Dedicated calculation followed by the shared guard/shield/HP/defeat boundary.
use super::{
    apply_ordinary_damage,
    critical::{self, CriticalRequest},
};
use crate::{
    EventId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    event::cause::Cause,
    event::model::DamageKind,
    formula::model::DamageClass,
    operation::{ElationDamageOp, HitOperationScratch},
    resolver::{operation_formula::FormulaInputs, transaction::Transaction},
};

pub(super) fn execute(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ElationDamageOp,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    let inputs = FormulaInputs::new(txn)?;
    if operation.targets.is_empty() {
        return Ok(parent);
    }
    let semantics = inputs.damage_semantics(catalog, txn, cause, DamageClass::Elation, false)?;
    for target in operation.targets {
        let critical = critical::resolve(
            &inputs,
            catalog,
            txn,
            cause,
            CriticalRequest {
                target,
                class: DamageClass::Elation,
                policy: operation.crit_policy,
                ultimate_semantics: false,
                element: Some(operation.definition.element()),
            },
            scratch,
        )?;
        let calculated =
            inputs.elation_damage(catalog, txn, cause, operation.definition, target, critical)?;
        parent = apply_ordinary_damage(
            catalog,
            txn,
            cause,
            parent,
            operation.id,
            target,
            DamageKind::Direct,
            DamageClass::Elation,
            semantics,
            Some(operation.definition.element()),
            None,
            calculated.raw,
            calculated.finalized,
        )?;
    }
    Ok(parent)
}
