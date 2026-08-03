//! Rule-IR resource mutations and Energy-regeneration scaling.

use super::*;
use crate::{
    Energy, EventId, Scalar, UnitId, catalog::CombatCatalog,
    resolver::operation_formula::FormulaInputs,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn modify_resource(
    catalog: &CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    targets: Box<[UnitId]>,
    resource: RuleResourceKind,
    update: ResourceUpdateKind,
    amount: RuleValue,
    scales_with_regeneration: bool,
    rounding: Rounding,
) -> Result<EventId, BattleFault> {
    let amount = non_negative_scalar(amount)?;
    for target in targets {
        match &resource {
            RuleResourceKind::Energy => {
                let amount = if scales_with_regeneration {
                    let rate = FormulaInputs::new(txn)?
                        .energy_regeneration_rate(catalog, txn, cause, target)?;
                    amount
                        .checked_mul(rate, rounding)
                        .map_err(|_| program_fault(29, rate.scaled()))?
                } else {
                    amount
                };
                let (before, maximum) = txn
                    .state
                    .units
                    .get(target)
                    .map(|unit| (unit.current_energy, unit.maximum_energy))
                    .ok_or_else(|| program_fault(23, 0))?;
                let raw =
                    resource_value(before.scaled(), maximum.scaled(), amount.scaled(), update)?;
                let after = Energy::from_scaled(raw).map_err(|_| program_fault(24, raw))?;
                txn.set_energy(target, after)?;
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Resource(ResourceEventData::Energy {
                        unit: target,
                        before,
                        after,
                        overflow: Energy::ZERO,
                    }),
                );
            }
            RuleResourceKind::SkillPoints => {
                let side = txn
                    .state
                    .units
                    .get(target)
                    .ok_or_else(|| program_fault(25, 0))?
                    .side;
                let state = txn.state.teams.get(side);
                let raw = resource_value(
                    i64::from(state.skill_points),
                    i64::from(state.maximum_skill_points),
                    amount
                        .rounded_integer(Rounding::Floor)
                        .map_err(|_| program_fault(26, 0))?,
                    update,
                )?;
                let after = u16::try_from(raw).map_err(|_| program_fault(27, raw))?;
                let before = state.skill_points;
                txn.set_skill_points(side, after);
                parent = txn.emit(
                    cause.with_parent(parent),
                    BattleEventKind::Resource(ResourceEventData::SkillPoints {
                        side,
                        attempted: before.abs_diff(after),
                        payer: SkillPointPayer::TeamSkillPoints,
                        effective: before.abs_diff(after),
                        before,
                        after,
                        overflow: 0,
                    }),
                );
            }
            RuleResourceKind::Character(stable_key) => {
                let (before, maximum) = txn
                    .state
                    .units
                    .get(target)
                    .and_then(|unit| unit.resource(stable_key))
                    .map(|resource| (resource.current, resource.maximum))
                    .ok_or_else(|| program_fault(28, 0))?;
                let raw =
                    resource_value(before.scaled(), maximum.scaled(), amount.scaled(), update)?;
                let after = Scalar::from_scaled(raw);
                txn.set_character_resource(target, stable_key, after)?;
                parent = txn.emit(
                    cause.with_parent(parent).with_primary_target(Some(target)),
                    BattleEventKind::Resource(ResourceEventData::CharacterResource {
                        unit: target,
                        resource: stable_key.clone(),
                        before,
                        after,
                        maximum,
                    }),
                );
            }
            RuleResourceKind::Team(stable_key) => {
                let side = txn
                    .state
                    .units
                    .get(target)
                    .ok_or_else(|| program_fault(28, 1))?
                    .side;
                let resource = txn
                    .state
                    .teams
                    .get(side)
                    .keyed_by_name(stable_key)
                    .ok_or_else(|| program_fault(28, 2))?;
                let before = resource.current;
                let maximum = resource.maximum;
                let resource_id = resource.id;
                let raw = resource_value(
                    i64::from(before),
                    i64::from(maximum),
                    amount
                        .rounded_integer(Rounding::Floor)
                        .map_err(|_| program_fault(28, 3))?,
                    update,
                )?;
                let after = u16::try_from(raw).map_err(|_| program_fault(28, raw))?;
                txn.set_team_resource(side, resource_id, after)?;
                parent = txn.emit(
                    cause.with_parent(parent),
                    BattleEventKind::Resource(ResourceEventData::TeamResource {
                        side,
                        resource: resource_id,
                        attempted: before.abs_diff(after),
                        effective: before.abs_diff(after),
                        before,
                        after,
                        overflow: 0,
                    }),
                );
            }
        }
    }
    Ok(parent)
}

fn resource_value(
    before: i64,
    maximum: i64,
    amount: i64,
    update: ResourceUpdateKind,
) -> Result<i64, BattleFault> {
    match update {
        ResourceUpdateKind::Gain => before.checked_add(amount).map(|value| value.min(maximum)),
        ResourceUpdateKind::Spend | ResourceUpdateKind::Reserve => {
            before.checked_sub(amount).filter(|value| *value >= 0)
        }
        ResourceUpdateKind::Set => (amount <= maximum).then_some(amount),
    }
    .ok_or_else(|| program_fault(31, amount))
}
