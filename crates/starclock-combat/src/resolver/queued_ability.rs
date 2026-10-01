//! Queue-time selection of one explicitly bound effective ability variant.
use crate::{
    AbilityId, FaultBoundary, FaultKind, FaultPolicy, UnitId, battle::fault::BattleFault,
    catalog::CombatCatalog, command::legal::effective_abilities,
    resolver::transaction::Transaction,
};

pub(super) fn resolve(
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    actor: UnitId,
    requested: AbilityId,
) -> Result<AbilityId, BattleFault> {
    let Some(unit) = txn.state.units.get(actor) else {
        return Ok(requested);
    };
    let bound = effective_abilities(&unit.abilities, &txn.state.effects, catalog, actor);
    if bound.binary_search(&requested).is_ok()
        || catalog
            .ability(requested)
            .is_none_or(|ability| ability.family() != requested)
    {
        return Ok(requested);
    }
    let mut matches = bound.into_iter().filter(|id| {
        catalog
            .ability(*id)
            .is_some_and(|ability| ability.family() == requested)
    });
    let Some(resolved) = matches.next() else {
        // Preserve normal dequeue-time cancellation for an unbound ability.
        return Ok(requested);
    };
    if matches.next().is_some() {
        return Err(BattleFault::new(
            FaultKind::InvariantViolation,
            FaultBoundary::Command,
            FaultPolicy::Rollback,
            0x32f0,
            Some(i64::from(requested.get())),
        ));
    }
    Ok(resolved)
}
