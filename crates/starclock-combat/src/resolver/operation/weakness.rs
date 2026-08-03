use super::{Transaction, invariant_fault};
use crate::{
    LifeState, PresenceState,
    battle::fault::BattleFault,
    catalog::{CombatCatalog, action::WeaknessApplicationDefinition},
    event::{
        cause::Cause,
        model::{BattleEventKind, ToughnessEventData},
    },
    id::EventId,
    operation::{AddWeaknessFromAlliedElementsOp, AddWeaknessOp},
    resolver::program::actor_basic_element,
    rng::types::DrawPurpose,
};

pub(super) fn execute_add_weakness(
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

pub(super) fn execute_allied_element_weakness(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: AddWeaknessFromAlliedElementsOp,
) -> Result<EventId, BattleFault> {
    let applier = cause.applier().ok_or_else(|| invariant_fault(11))?;
    let side = txn
        .state
        .units
        .get(applier)
        .map(|unit| unit.side)
        .ok_or_else(|| invariant_fault(11))?;
    let units = txn
        .state
        .units
        .iter_by_id()
        .filter(|unit| {
            unit.side == side
                && unit.life == LifeState::Alive
                && unit.presence == PresenceState::Present
        })
        .map(|unit| unit.id)
        .collect::<Vec<_>>();
    let mut elements = units
        .into_iter()
        .map(|unit| actor_basic_element(catalog, txn, unit))
        .collect::<Result<Vec<_>, _>>()?;
    elements.sort_unstable();
    elements.dedup();
    let count = usize::from(operation.count).min(elements.len());
    for _ in 0..count {
        let index = txn
            .choose_index(DrawPurpose::WEAKNESS_ELEMENT, elements.len())?
            .ok_or_else(|| invariant_fault(11))?;
        let element = elements.remove(index);
        parent = execute_add_weakness(
            txn,
            cause,
            parent,
            AddWeaknessOp {
                id: operation.id,
                targets: operation.targets.clone(),
                definition: WeaknessApplicationDefinition::timed(element, operation.duration_turns)
                    .ok_or_else(|| invariant_fault(11))?,
            },
        )?;
    }
    Ok(parent)
}
