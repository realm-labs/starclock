//! Forced Break lowered through the ordinary Toughness mutation path.

use crate::catalog::CombatCatalog;
use crate::formula::model::CombatElement;
use crate::formula::toughness::BreakDamageDefinition;
use crate::formula::toughness::ToughnessReductionContext;
use crate::{
    Probability, Ratio, RawToughness, Scalar, ToughnessReductionDefinition,
    battle::fault::BattleFault,
    event::cause::Cause,
    id::EventId,
    operation::{ForceBreakOp, HitOperationScratch, ReduceToughnessOp},
};

use super::{operation::execute_toughness_reduction, transaction::Transaction};

pub(super) fn execute_force_break(
    catalog: &CombatCatalog,
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
            .unwrap_or_else(|| RawToughness::new(0).expect("zero Toughness is valid"));
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

fn definition(element: CombatElement, base: RawToughness) -> ToughnessReductionDefinition {
    ToughnessReductionDefinition {
        element,
        ignores_weakness: true,
        reduction: ToughnessReductionContext {
            base,
            additive: RawToughness::new(0).expect("zero Toughness is valid"),
            reduction_increase: Ratio::ZERO,
            weakness_break_efficiency: Ratio::ZERO,
            weakness_break_efficiency_cap: Ratio::from_scaled(3_000_000),
            toughness_vulnerability: Ratio::ZERO,
            ability_multiplier: Ratio::ONE,
        },
        break_damage: BreakDamageDefinition {
            attacker_level_multiplier: Scalar::ONE,
            ability_multiplier: Ratio::ONE,
            break_effect: Ratio::ZERO,
            break_damage_increase: Ratio::ZERO,
            defense_multiplier: Ratio::ONE,
            resistance_multiplier: Ratio::ONE,
            vulnerability_multiplier: Ratio::ONE,
            mitigation_multiplier: Ratio::ONE,
            unbroken_multiplier: Ratio::ONE,
        },
        break_effect_chance: Probability::ONE,
    }
}
