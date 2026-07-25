//! Forced Break lowered through the ordinary Toughness mutation path.

use crate::{
    battle::fault::BattleFault,
    event::cause::Cause,
    id::EventId,
    operation::{ForceBreakOp, HitOperationScratch, ReduceToughnessOp},
};

use super::{operation::execute_toughness_reduction, transaction::Transaction};

pub(super) fn execute_force_break(
    catalog: &crate::catalog::CombatCatalog,
    txn: &mut Transaction<'_>,
    cause: Cause,
    mut parent: EventId,
    operation: ForceBreakOp,
    scratch: &mut HitOperationScratch,
) -> Result<EventId, BattleFault> {
    for target in operation.targets {
        let attempted = txn
            .state
            .units
            .get(target)
            .and_then(|unit| {
                unit.toughness_layers
                    .iter()
                    .map(|layer| layer.current)
                    .max_by_key(|value| value.get())
            })
            .unwrap_or_else(|| crate::RawToughness::new(0).expect("zero Toughness is valid"));
        parent = execute_toughness_reduction(
            catalog,
            txn,
            cause,
            parent,
            ReduceToughnessOp {
                id: operation.id,
                targets: vec![target].into_boxed_slice(),
                definition: definition(operation.element, attempted),
            },
            scratch,
        )?;
    }
    Ok(parent)
}

fn definition(
    element: crate::formula::model::CombatElement,
    base: crate::RawToughness,
) -> crate::ToughnessReductionDefinition {
    crate::ToughnessReductionDefinition {
        element,
        ignores_weakness: true,
        reduction: crate::formula::toughness::ToughnessReductionContext {
            base,
            additive: crate::RawToughness::new(0).expect("zero Toughness is valid"),
            reduction_increase: crate::Ratio::ZERO,
            weakness_break_efficiency: crate::Ratio::ZERO,
            weakness_break_efficiency_cap: crate::Ratio::from_scaled(3_000_000),
            toughness_vulnerability: crate::Ratio::ZERO,
            ability_multiplier: crate::Ratio::ONE,
        },
        break_damage: crate::formula::toughness::BreakDamageDefinition {
            attacker_level_multiplier: crate::Scalar::ONE,
            ability_multiplier: crate::Ratio::ONE,
            break_effect: crate::Ratio::ZERO,
            break_damage_increase: crate::Ratio::ZERO,
            defense_multiplier: crate::Ratio::ONE,
            resistance_multiplier: crate::Ratio::ONE,
            vulnerability_multiplier: crate::Ratio::ONE,
            mitigation_multiplier: crate::Ratio::ONE,
            unbroken_multiplier: crate::Ratio::ONE,
        },
        break_effect_chance: crate::Probability::ONE,
    }
}
