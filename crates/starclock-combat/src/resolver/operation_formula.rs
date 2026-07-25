//! Modifier-aware formula preparation separated from authoritative state mutation.

use std::collections::BTreeMap;

use crate::{
    battle::fault::BattleFault,
    catalog::action::{HealingDefinition, OrdinaryDamageDefinition, ShieldDefinition},
    event::cause::{Cause, CauseActor},
    formula,
    modifier::{
        model::{
            ActiveModifier, FormulaModifierQuery, FormulaPurpose, FormulaStage, LifeFilter,
            ModifierQueryContext, PresenceFilter,
        },
        resolve::StatResolver,
    },
};

use super::{
    operation::fault::{invariant_fault, numeric_fault},
    transaction::Transaction,
};

pub(super) struct FormulaInputs {
    bases: BTreeMap<(crate::UnitId, crate::modifier::model::StatKind), crate::Scalar>,
    modifiers: Vec<ActiveModifier>,
}

impl FormulaInputs {
    pub(super) fn new(txn: &Transaction<'_>) -> Result<Self, BattleFault> {
        Ok(Self {
            bases: super::program::stat_bases(txn)?,
            modifiers: txn.state.modifiers.iter_by_id().cloned().collect(),
        })
    }

    pub(super) fn damage(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: OrdinaryDamageDefinition,
        element: Option<formula::model::CombatElement>,
        target: crate::UnitId,
    ) -> Result<crate::formula::sustain::DamageCalculation, BattleFault> {
        let resolver = self.resolver(catalog);
        let purpose = damage_purpose(formula.class());
        let source = formula_source(txn, cause, purpose)?;
        let source_context = modifier_context(txn, source, target, element, formula.class())?;
        let target_context = modifier_context(txn, target, target, element, formula.class())?;
        for stage in [
            FormulaStage::Crit,
            FormulaStage::DamageBoost,
            FormulaStage::Weaken,
        ] {
            let value = formula_modifier(&resolver, source, stage, purpose, &source_context)?;
            formula = formula
                .with_formula_modifier(stage, value)
                .map_err(|_| numeric_fault(41, value.scaled()))?;
        }
        for stage in [
            FormulaStage::Defense,
            FormulaStage::Resistance,
            FormulaStage::Vulnerability,
            FormulaStage::Mitigation,
            FormulaStage::Broken,
        ] {
            let value = formula_modifier(&resolver, target, stage, purpose, &target_context)?;
            formula = formula
                .with_formula_modifier(stage, value)
                .map_err(|_| numeric_fault(42, value.scaled()))?;
        }
        formula::ordinary_damage(formula)
            .map_err(|_| numeric_fault(1, formula.base_damage().scaled()))
    }

    pub(super) fn target_mitigation(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        target: crate::UnitId,
        purpose: FormulaPurpose,
        element: formula::model::CombatElement,
    ) -> Result<crate::Ratio, BattleFault> {
        let resolver = self.resolver(catalog);
        let mut context = modifier_context(
            txn,
            target,
            target,
            Some(element),
            formula::model::DamageClass::Direct,
        )?;
        context.damage_tags = vec![match purpose {
            FormulaPurpose::Break => "break".into(),
            FormulaPurpose::SuperBreak => "super_break".into(),
            _ => return Err(invariant_fault(47)),
        }]
        .into_boxed_slice();
        let value = formula_modifier(
            &resolver,
            target,
            FormulaStage::Mitigation,
            purpose,
            &context,
        )?;
        crate::Ratio::ONE
            .checked_sub(crate::Ratio::from_scaled(value.scaled()))
            .map_err(|_| numeric_fault(48, value.scaled()))
    }

    pub(super) fn healing(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        formula: HealingDefinition,
        target: crate::UnitId,
    ) -> Result<crate::formula::sustain::HealingCalculation, BattleFault> {
        let resolver = self.resolver(catalog);
        let source = formula_source(txn, cause, FormulaPurpose::Healing)?;
        let outgoing = formula_modifier(
            &resolver,
            source,
            FormulaStage::Healing,
            FormulaPurpose::Healing,
            &modifier_context(
                txn,
                source,
                target,
                None,
                formula::model::DamageClass::Direct,
            )?,
        )?;
        let incoming = if source == target {
            crate::Scalar::ZERO
        } else {
            formula_modifier(
                &resolver,
                target,
                FormulaStage::Healing,
                FormulaPurpose::Healing,
                &modifier_context(
                    txn,
                    target,
                    target,
                    None,
                    formula::model::DamageClass::Direct,
                )?,
            )?
        };
        let formula = formula
            .with_formula_modifier(outgoing, true)
            .and_then(|formula| formula.with_formula_modifier(incoming, false))
            .map_err(|_| numeric_fault(45, outgoing.scaled()))?;
        formula::healing(formula).map_err(|_| numeric_fault(4, formula.base_healing().scaled()))
    }

    pub(super) fn shield(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        formula: ShieldDefinition,
        target: crate::UnitId,
    ) -> Result<formula::model::ShieldCalculation, BattleFault> {
        let resolver = self.resolver(catalog);
        let source = formula_source(txn, cause, FormulaPurpose::Shield)?;
        let outgoing = formula_modifier(
            &resolver,
            source,
            FormulaStage::Shield,
            FormulaPurpose::Shield,
            &modifier_context(
                txn,
                source,
                target,
                None,
                formula::model::DamageClass::Direct,
            )?,
        )?;
        let incoming = if source == target {
            crate::Scalar::ZERO
        } else {
            formula_modifier(
                &resolver,
                target,
                FormulaStage::Shield,
                FormulaPurpose::Shield,
                &modifier_context(
                    txn,
                    target,
                    target,
                    None,
                    formula::model::DamageClass::Direct,
                )?,
            )?
        };
        let formula = formula
            .with_formula_modifier(
                outgoing
                    .checked_add(incoming)
                    .map_err(|_| numeric_fault(43, outgoing.scaled()))?,
            )
            .map_err(|_| numeric_fault(44, outgoing.scaled()))?;
        let context = formula::model::ShieldContext {
            scaling_terms: vec![formula::model::ScalingTerm {
                stat: formula.base_shield(),
                ratio: crate::Ratio::ONE,
            }]
            .into_boxed_slice(),
            additive_base: crate::Scalar::ZERO,
            bonuses: vec![formula.bonus()].into_boxed_slice(),
        };
        formula::shield::calculate(&context)
            .map_err(|_| numeric_fault(9, formula.base_shield().scaled()))
    }

    fn resolver<'a>(&'a self, catalog: &'a crate::catalog::CombatCatalog) -> StatResolver<'a> {
        StatResolver::new(catalog.modifier_registry(), &self.bases, &self.modifiers)
    }
}

fn formula_modifier(
    resolver: &StatResolver<'_>,
    subject: crate::UnitId,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    context: &ModifierQueryContext,
) -> Result<crate::Scalar, BattleFault> {
    resolver
        .query_formula(
            FormulaModifierQuery {
                subject,
                stage,
                purpose,
            },
            context,
        )
        .map_err(|_| numeric_fault(46, i64::from(stage as u8)))
}

fn formula_source(
    txn: &Transaction<'_>,
    cause: Cause,
    purpose: FormulaPurpose,
) -> Result<crate::UnitId, BattleFault> {
    if purpose == FormulaPurpose::Dot
        && let Some(applier) = cause.applier()
    {
        return Ok(applier);
    }
    match cause.actor() {
        Some(CauseActor::Unit(unit)) => Ok(unit),
        Some(CauseActor::TimelineActor(actor)) => txn
            .state
            .actors
            .get(actor)
            .map(|state| state.unit.unwrap_or(state.owner))
            .ok_or_else(|| invariant_fault(43)),
        None => cause
            .applier()
            .or(cause.owner())
            .ok_or_else(|| invariant_fault(44)),
    }
}

fn modifier_context(
    txn: &Transaction<'_>,
    subject: crate::UnitId,
    target: crate::UnitId,
    element: Option<formula::model::CombatElement>,
    class: formula::model::DamageClass,
) -> Result<ModifierQueryContext, BattleFault> {
    let unit = txn
        .state
        .units
        .get(subject)
        .ok_or_else(|| invariant_fault(45))?;
    Ok(ModifierQueryContext {
        damage_tags: vec![match class {
            formula::model::DamageClass::Direct => "direct".into(),
            formula::model::DamageClass::Dot => "dot".into(),
            formula::model::DamageClass::Additional => "additional".into(),
            formula::model::DamageClass::Elation => "elation".into(),
        }]
        .into_boxed_slice(),
        element: element.map(|value| value as u8),
        life: Some(match unit.life {
            crate::LifeState::Alive => LifeFilter::Alive,
            crate::LifeState::Downed => LifeFilter::Downed,
            crate::LifeState::Defeated => LifeFilter::Defeated,
        }),
        presence: Some(match unit.presence {
            crate::PresenceState::Present => PresenceFilter::Present,
            crate::PresenceState::Reserved => PresenceFilter::Reserved,
            crate::PresenceState::Departed => PresenceFilter::Departed,
            crate::PresenceState::Untargetable => PresenceFilter::Untargetable,
            crate::PresenceState::Linked => PresenceFilter::Linked,
            crate::PresenceState::Transformed => PresenceFilter::Transformed,
        }),
        target: Some(target),
        ..ModifierQueryContext::default()
    })
}

const fn damage_purpose(class: formula::model::DamageClass) -> FormulaPurpose {
    match class {
        formula::model::DamageClass::Direct => FormulaPurpose::OrdinaryDamage,
        formula::model::DamageClass::Dot => FormulaPurpose::Dot,
        formula::model::DamageClass::Additional => FormulaPurpose::AdditionalDamage,
        formula::model::DamageClass::Elation => FormulaPurpose::ElationDamage,
    }
}
