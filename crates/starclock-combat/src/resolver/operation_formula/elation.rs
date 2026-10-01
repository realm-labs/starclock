//! Live native Elation inputs and explicit common-stage modifier projection.
use super::{
    FormulaInputs, IncomingModifierContext, damage_modifier_context, final_damage,
    formula_modifier, formula_source, incoming_formula_modifier, modifier_context,
};
use crate::{
    DamageAmount, Ratio, Rounding, Scalar, UnitId,
    battle::fault::BattleFault,
    catalog::{CombatCatalog, action::elation::ElationDamageDefinition},
    event::cause::Cause,
    formula::{
        elation::{self, ElationDamageContext, ElationDamageModifiers},
        model::{CritDecision, DamageClass, DefenseInput},
        sustain::DamageCalculation,
    },
    modifier::model::{FormulaPurpose, FormulaStage, FormulaSubject, StatKind, StatQuery},
    resolver::{
        operation::{
            critical::CriticalResolution,
            fault::{invariant_fault, numeric_fault},
        },
        transaction::Transaction,
    },
};
impl FormulaInputs {
    pub(in crate::resolver) fn elation_damage(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        definition: ElationDamageDefinition,
        target: UnitId,
        critical: CriticalResolution,
    ) -> Result<DamageCalculation, BattleFault> {
        let purpose = FormulaPurpose::ElationDamage;
        let source = formula_source(txn, cause, purpose)?;
        let resolver = self.resolver(catalog);
        let source_context = damage_modifier_context(
            catalog,
            cause,
            modifier_context(
                txn,
                source,
                target,
                Some(definition.element()),
                DamageClass::Elation,
            )?,
            false,
        )
        .with_formula_subject(FormulaSubject::Source);
        let target_context = damage_modifier_context(
            catalog,
            cause,
            modifier_context(
                txn,
                target,
                target,
                Some(definition.element()),
                DamageClass::Elation,
            )?,
            false,
        )
        .with_formula_subject(FormulaSubject::Target);
        let elation = resolver
            .query(
                StatQuery {
                    subject: source,
                    stat: StatKind::Elation,
                    purpose,
                },
                &source_context,
            )
            .map_err(|_| numeric_fault(80, i64::from(StatKind::Elation as u8)))?;
        let defense = resolver
            .query(
                StatQuery {
                    subject: target,
                    stat: StatKind::Def,
                    purpose,
                },
                // DEF is the target's own derived stat, not an incoming factor
                // contribution. Preserve ordinary unfiltered stat buffs here;
                // incoming formula stages still require explicit Target metadata.
                &target_context
                    .clone()
                    .with_formula_subject(FormulaSubject::Source),
            )
            .map_err(|_| numeric_fault(81, i64::from(StatKind::Def as u8)))?;
        let incoming = IncomingModifierContext {
            cause,
            source,
            target,
            element: Some(definition.element()),
            class: DamageClass::Elation,
            ultimate_semantics: false,
        };
        let contribution = |stage| {
            incoming_formula_modifier(&resolver, catalog, txn, incoming, stage, purpose)
                .map(|value| Ratio::from_scaled(value.scaled()))
        };
        let source_resistance = formula_modifier(
            &resolver,
            source,
            FormulaStage::Resistance,
            purpose,
            &source_context,
        )?;
        let modifiers = ElationDamageModifiers {
            flat_base: formula_modifier(
                &resolver,
                source,
                FormulaStage::Flat,
                purpose,
                &source_context,
            )?,
            crit: Ratio::from_scaled(
                formula_modifier(
                    &resolver,
                    source,
                    FormulaStage::Crit,
                    purpose,
                    &source_context,
                )?
                .scaled(),
            ),
            defense: contribution(FormulaStage::Defense)?,
            resistance: contribution(FormulaStage::Resistance)?
                .checked_add(Ratio::from_scaled(source_resistance.scaled()))
                .map_err(|_| numeric_fault(82, source_resistance.scaled()))?,
            vulnerability: contribution(FormulaStage::Vulnerability)?,
            mitigation: contribution(FormulaStage::Mitigation)?,
            broken: contribution(FormulaStage::Broken)?,
        };
        let source_unit = txn
            .state
            .units
            .get(source)
            .ok_or_else(|| invariant_fault(43))?;
        let target_unit = txn
            .state
            .units
            .get(target)
            .ok_or_else(|| invariant_fault(45))?;
        let context = ElationDamageContext {
            base_damage: definition.base_damage,
            original_damage_multiplier: definition.original_multiplier,
            crit: if critical.is_critical {
                CritDecision::Critical
            } else {
                CritDecision::Normal
            },
            crit_damage: Ratio::from_scaled(critical.damage.scaled()),
            elation: Ratio::from_scaled(elation.scaled()),
            meter_multiplier: definition.meter_multiplier,
            merrymaking: definition.merrymaking,
            defense: DefenseInput::Actual {
                target_defense: defense,
                attacker_level: u16::from(source_unit.level.get()),
            },
            resistance: definition.resistance,
            vulnerabilities: Box::default(),
            mitigations: Box::default(),
            broken: target_unit.weakness_broken,
            unbroken_multiplier: definition.unbroken_multiplier,
            modifiers,
        };
        let calculated = elation::calculate(&context)
            .map_err(|_| numeric_fault(83, definition.base_damage.scaled()))?;
        let override_amount = formula_modifier(
            &resolver,
            source,
            FormulaStage::DamageOverride,
            purpose,
            &source_context,
        )?;
        if override_amount > Scalar::ZERO {
            return Ok(DamageCalculation {
                raw: override_amount,
                finalized: DamageAmount::from_scalar(override_amount, Rounding::Floor)
                    .map_err(|_| numeric_fault(76, override_amount.scaled()))?,
            });
        }
        let factor = final_damage::multiplier(&resolver, source, purpose, &source_context)?;
        final_damage::apply(calculated.raw, factor)
    }
}
