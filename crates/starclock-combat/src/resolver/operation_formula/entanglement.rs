//! Application-time capture for ordinary Quantum Entanglement.

use super::{FormulaInputs, action_modifier_context, break_modifier_context};
use crate::{
    Ratio, RawToughness, ToughnessLayerKind, UnitId,
    battle::fault::BattleFault,
    catalog::CombatCatalog,
    event::cause::Cause,
    formula::{
        model::CombatElement,
        toughness::{self, BaseBreakEffect, BreakDamageDefinition},
    },
    modifier::model::{FormulaPurpose, FormulaSubject, StatKind, StatQuery},
    resolver::{
        operation::fault::{invariant_fault, numeric_fault},
        transaction::Transaction,
    },
};

impl FormulaInputs {
    pub(in crate::resolver) fn entanglement_plan(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        target: UnitId,
        damage: BreakDamageDefinition,
    ) -> Result<BaseBreakEffect, BattleFault> {
        let applier = cause.applier().ok_or_else(|| invariant_fault(80))?;
        let unit = txn
            .state
            .units
            .get(target)
            .ok_or_else(|| invariant_fault(81))?;
        // Ordinary maximum only: Exo/Sequential/Shared bars do not increase it.
        // A barless target has an explicit zero maximum, not an invented bar.
        let maximum = unit
            .toughness_layers
            .iter()
            .filter(|layer| layer.spec.kind() == ToughnessLayerKind::Ordinary)
            .min_by_key(|layer| layer.spec.key())
            .map_or(
                RawToughness::new(0).expect("zero Toughness is valid"),
                |layer| layer.spec.maximum(),
            );
        let context = action_modifier_context(
            catalog,
            cause,
            break_modifier_context(
                txn,
                applier,
                target,
                CombatElement::Quantum,
                FormulaPurpose::Break,
            )?,
        )
        .with_formula_subject(FormulaSubject::Source);
        let bonus = self
            .resolver(catalog)
            .query(
                StatQuery {
                    subject: applier,
                    stat: StatKind::BreakEffect,
                    purpose: FormulaPurpose::Break,
                },
                &context,
            )
            .map_err(|_| numeric_fault(80, i64::from(StatKind::BreakEffect as u8)))?;
        let break_effect = damage
            .break_effect
            .checked_add(Ratio::from_scaled(bonus.scaled()))
            .map_err(|_| numeric_fault(81, bonus.scaled()))?;
        toughness::base_break_effect(
            CombatElement::Quantum,
            unit.rank,
            unit.maximum_hp,
            damage.attacker_level_multiplier,
            maximum,
            break_effect,
        )
        .map_err(|_| numeric_fault(82, maximum.get()))
    }
}
