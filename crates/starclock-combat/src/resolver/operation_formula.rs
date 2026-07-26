//! Modifier-aware formula preparation separated from authoritative state mutation.

use std::collections::BTreeMap;

use crate::{
    battle::fault::BattleFault,
    catalog::action::{HealingDefinition, OrdinaryDamageDefinition, ShieldDefinition},
    event::cause::{Cause, CauseActor},
    formula,
    modifier::{
        model::{
            ActiveModifier, FormulaModifierQuery, FormulaPurpose, FormulaStage, FormulaSubject,
            LifeFilter, ModifierQueryContext, PresenceFilter,
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
    shields: BTreeMap<crate::UnitId, crate::Scalar>,
    effect_category_stacks: BTreeMap<(crate::UnitId, crate::EffectCategory), i64>,
    modifiers: Vec<ActiveModifier>,
}

impl FormulaInputs {
    pub(super) fn new(txn: &Transaction<'_>) -> Result<Self, BattleFault> {
        Ok(Self {
            bases: super::program::stat_bases(txn)?,
            shields: super::stat_input::shield_values(txn),
            effect_category_stacks: effect_category_stacks(txn)?,
            modifiers: txn.state.modifiers.iter_by_id().cloned().collect(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn damage(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: OrdinaryDamageDefinition,
        element: Option<formula::model::CombatElement>,
        target: crate::UnitId,
        apply_source_modifiers: bool,
    ) -> Result<crate::formula::sustain::DamageCalculation, BattleFault> {
        let resolver = self.resolver(catalog);
        let purpose = damage_purpose(formula.class());
        let source = formula_source(txn, cause, purpose)?;
        let source_context = action_modifier_context(
            catalog,
            cause,
            modifier_context(txn, source, target, element, formula.class())?,
        )
        .with_formula_subject(FormulaSubject::Source);
        let incoming_context = IncomingModifierContext {
            cause,
            source,
            target,
            element,
            class: formula.class(),
        };
        if apply_source_modifiers {
            for stage in [
                FormulaStage::Crit,
                FormulaStage::DamageBoost,
                FormulaStage::Weaken,
            ] {
                let mut value =
                    formula_modifier(&resolver, source, stage, purpose, &source_context)?;
                if stage == FormulaStage::DamageBoost {
                    let target_value = formula_modifier(
                        &resolver,
                        target,
                        stage,
                        purpose,
                        &action_modifier_context(
                            catalog,
                            cause,
                            modifier_context(txn, target, target, element, formula.class())?,
                        )
                        .with_formula_subject(FormulaSubject::Target),
                    )?;
                    value = value
                        .checked_add(target_value)
                        .map_err(|_| numeric_fault(54, target_value.scaled()))?;
                }
                formula = formula
                    .with_formula_modifier(stage, value)
                    .map_err(|_| numeric_fault(41, value.scaled()))?;
            }
        }
        for stage in [
            FormulaStage::Defense,
            FormulaStage::Resistance,
            FormulaStage::Vulnerability,
            FormulaStage::Mitigation,
            FormulaStage::Broken,
        ] {
            let value = incoming_formula_modifier(
                &resolver,
                catalog,
                txn,
                incoming_context,
                stage,
                purpose,
            )?;
            formula = formula
                .with_formula_modifier(stage, value)
                .map_err(|_| numeric_fault(42, value.scaled()))?;
        }
        formula::ordinary_damage(formula)
            .map_err(|_| numeric_fault(1, formula.base_damage().scaled()))
    }

    pub(super) fn critical_profile(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        class: formula::model::DamageClass,
        target: crate::UnitId,
    ) -> Result<CriticalProfile, BattleFault> {
        use crate::modifier::model::StatKind;

        let purpose = damage_purpose(class);
        let source = formula_source(txn, cause, purpose)?;
        let resolver = self.resolver(catalog);
        let source_context = action_modifier_context(
            catalog,
            cause,
            modifier_context(txn, source, target, None, class)?,
        )
        .with_formula_subject(FormulaSubject::Source);
        let rate = resolver
            .query(
                crate::modifier::model::StatQuery {
                    subject: source,
                    stat: StatKind::CritRate,
                    purpose,
                },
                &source_context,
            )
            .map_err(|_| numeric_fault(50, i64::from(StatKind::CritRate as u8)))?;
        let target_bonus = formula_modifier(
            &resolver,
            target,
            FormulaStage::Probability,
            FormulaPurpose::CriticalChance,
            &action_modifier_context(
                catalog,
                cause,
                modifier_context(txn, target, target, None, class)?,
            )
            .with_formula_subject(FormulaSubject::Target),
        )?;
        let damage = resolver
            .query(
                crate::modifier::model::StatQuery {
                    subject: source,
                    stat: StatKind::CritDamage,
                    purpose,
                },
                &source_context,
            )
            .map_err(|_| numeric_fault(51, i64::from(StatKind::CritDamage as u8)))?;
        if damage.scaled() < 0 {
            return Err(numeric_fault(52, damage.scaled()));
        }
        Ok(CriticalProfile {
            chance: formula::model::clamp_probability(crate::Ratio::from_scaled(
                rate.checked_add(target_bonus)
                    .map_err(|_| numeric_fault(53, target_bonus.scaled()))?
                    .scaled(),
            )),
            damage,
        })
    }

    pub(super) fn energy_regeneration_rate(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        subject: crate::UnitId,
    ) -> Result<crate::Scalar, BattleFault> {
        use crate::modifier::model::StatKind;

        self.resolver(catalog)
            .query(
                crate::modifier::model::StatQuery {
                    subject,
                    stat: StatKind::EnergyRegenerationRate,
                    purpose: FormulaPurpose::Stat,
                },
                &action_modifier_context(
                    catalog,
                    cause,
                    modifier_context(
                        txn,
                        subject,
                        subject,
                        None,
                        formula::model::DamageClass::Direct,
                    )?,
                )
                .with_formula_subject(FormulaSubject::Source),
            )
            .map_err(|_| numeric_fault(69, i64::from(StatKind::EnergyRegenerationRate as u8)))
    }

    pub(super) fn break_damage(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: formula::toughness::BreakDamageDefinition,
        element: formula::model::CombatElement,
        target: crate::UnitId,
    ) -> Result<formula::toughness::BreakDamageDefinition, BattleFault> {
        let modifiers = self.break_formula_modifiers(
            catalog,
            txn,
            cause,
            target,
            FormulaPurpose::Break,
            element,
        )?;
        formula.break_damage_increase = formula
            .break_damage_increase
            .checked_add(crate::Ratio::from_scaled(modifiers.damage_boost.scaled()))
            .map_err(|_| numeric_fault(57, modifiers.damage_boost.scaled()))?;
        formula.vulnerability_multiplier = formula
            .vulnerability_multiplier
            .checked_add(crate::Ratio::from_scaled(modifiers.vulnerability.scaled()))
            .map_err(|_| numeric_fault(58, modifiers.vulnerability.scaled()))?;
        formula.mitigation_multiplier = formula
            .mitigation_multiplier
            .checked_mul(modifiers.mitigation, crate::Rounding::NearestTiesEven)
            .map_err(|_| numeric_fault(59, modifiers.mitigation.scaled()))?;
        Ok(formula)
    }

    pub(super) fn super_break_damage(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: formula::toughness::SuperBreakDefinition,
        target: crate::UnitId,
    ) -> Result<formula::toughness::SuperBreakDefinition, BattleFault> {
        let modifiers = self.break_formula_modifiers(
            catalog,
            txn,
            cause,
            target,
            FormulaPurpose::SuperBreak,
            formula.element,
        )?;
        formula.break_damage_increase = formula
            .break_damage_increase
            .checked_add(crate::Ratio::from_scaled(modifiers.damage_boost.scaled()))
            .map_err(|_| numeric_fault(60, modifiers.damage_boost.scaled()))?;
        formula.vulnerability_multiplier = formula
            .vulnerability_multiplier
            .checked_add(crate::Ratio::from_scaled(modifiers.vulnerability.scaled()))
            .map_err(|_| numeric_fault(61, modifiers.vulnerability.scaled()))?;
        formula.mitigation_multiplier = formula
            .mitigation_multiplier
            .checked_mul(modifiers.mitigation, crate::Rounding::NearestTiesEven)
            .map_err(|_| numeric_fault(62, modifiers.mitigation.scaled()))?;
        Ok(formula)
    }

    fn break_formula_modifiers(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        target: crate::UnitId,
        purpose: FormulaPurpose,
        element: formula::model::CombatElement,
    ) -> Result<BreakFormulaModifiers, BattleFault> {
        let source = formula_source(txn, cause, purpose)?;
        let resolver = self.resolver(catalog);
        let source_context = action_modifier_context(
            catalog,
            cause,
            break_modifier_context(txn, source, target, element, purpose)?,
        )
        .with_formula_subject(FormulaSubject::Source);
        let mut damage_boost = formula_modifier(
            &resolver,
            source,
            FormulaStage::DamageBoost,
            purpose,
            &source_context,
        )?;
        let target_context = action_modifier_context(
            catalog,
            cause,
            break_modifier_context(txn, target, target, element, purpose)?,
        );
        damage_boost = damage_boost
            .checked_add(formula_modifier(
                &resolver,
                target,
                FormulaStage::DamageBoost,
                purpose,
                &target_context
                    .clone()
                    .with_formula_subject(FormulaSubject::Target),
            )?)
            .map_err(|_| numeric_fault(63, damage_boost.scaled()))?;
        let vulnerability = formula_modifier(
            &resolver,
            target,
            FormulaStage::Vulnerability,
            purpose,
            &target_context.clone(),
        )?
        .checked_add(formula_modifier(
            &resolver,
            target,
            FormulaStage::Vulnerability,
            purpose,
            &target_context
                .clone()
                .with_formula_subject(FormulaSubject::Target),
        )?)
        .map_err(|_| numeric_fault(64, i64::from(FormulaStage::Vulnerability as u8)))?;
        let mitigation_value = formula_modifier(
            &resolver,
            target,
            FormulaStage::Mitigation,
            purpose,
            &target_context,
        )?
        .checked_add(formula_modifier(
            &resolver,
            target,
            FormulaStage::Mitigation,
            purpose,
            &target_context
                .clone()
                .with_formula_subject(FormulaSubject::Target),
        )?)
        .map_err(|_| numeric_fault(65, i64::from(FormulaStage::Mitigation as u8)))?;
        let mitigation = crate::Ratio::ONE
            .checked_sub(crate::Ratio::from_scaled(mitigation_value.scaled()))
            .map_err(|_| numeric_fault(65, i64::from(FormulaStage::Mitigation as u8)))?;
        Ok(BreakFormulaModifiers {
            damage_boost,
            vulnerability,
            mitigation,
        })
    }

    pub(super) fn weakness_break_efficiency(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        target: crate::UnitId,
        element: formula::model::CombatElement,
    ) -> Result<crate::Ratio, BattleFault> {
        let purpose = FormulaPurpose::Break;
        let source = formula_source(txn, cause, purpose)?;
        let context = action_modifier_context(
            catalog,
            cause,
            modifier_context(
                txn,
                source,
                target,
                Some(element),
                formula::model::DamageClass::Direct,
            )?,
        )
        .with_formula_subject(FormulaSubject::Source);
        let value = self
            .resolver(catalog)
            .query(
                crate::modifier::model::StatQuery {
                    subject: source,
                    stat: crate::modifier::model::StatKind::ToughnessDamage,
                    purpose,
                },
                &context,
            )
            .map_err(|_| {
                numeric_fault(
                    56,
                    i64::from(crate::modifier::model::StatKind::ToughnessDamage as u8),
                )
            })?;
        Ok(crate::Ratio::from_scaled(value.scaled()))
    }

    pub(super) fn toughness_recovery(
        &self,
        catalog: &crate::catalog::CombatCatalog,
        txn: &Transaction<'_>,
        target: crate::UnitId,
    ) -> Result<crate::Ratio, BattleFault> {
        let stat = crate::modifier::model::StatKind::ToughnessRecovery;
        let value = self
            .resolver(catalog)
            .query(
                crate::modifier::model::StatQuery {
                    subject: target,
                    stat,
                    purpose: FormulaPurpose::Stat,
                },
                &modifier_context(
                    txn,
                    target,
                    target,
                    None,
                    formula::model::DamageClass::Direct,
                )?,
            )
            .map_err(|_| numeric_fault(67, i64::from(stat as u8)))?;
        if value.scaled() < 0 {
            return Err(numeric_fault(68, value.scaled()));
        }
        Ok(crate::Ratio::from_scaled(value.scaled()))
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
            )
            .map(|context| action_modifier_context(catalog, cause, context))?
            .with_formula_subject(FormulaSubject::Source),
        )?;
        let incoming = incoming_formula_modifier(
            &resolver,
            catalog,
            txn,
            IncomingModifierContext {
                cause,
                source,
                target,
                element: None,
                class: formula::model::DamageClass::Direct,
            },
            FormulaStage::Healing,
            FormulaPurpose::Healing,
        )?;
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
            )
            .map(|context| action_modifier_context(catalog, cause, context))?
            .with_formula_subject(FormulaSubject::Source),
        )?;
        let incoming = incoming_formula_modifier(
            &resolver,
            catalog,
            txn,
            IncomingModifierContext {
                cause,
                source,
                target,
                element: None,
                class: formula::model::DamageClass::Direct,
            },
            FormulaStage::Shield,
            FormulaPurpose::Shield,
        )?;
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
            .with_shields(&self.shields)
            .with_effect_category_stacks(&self.effect_category_stacks)
    }
}

fn effect_category_stacks(
    txn: &Transaction<'_>,
) -> Result<BTreeMap<(crate::UnitId, crate::EffectCategory), i64>, BattleFault> {
    let mut output = BTreeMap::new();
    for effect in txn.state.effects.iter_by_id() {
        let stacks = output
            .entry((effect.target, effect.category))
            .or_insert(0_i64);
        *stacks = stacks
            .checked_add(i64::from(effect.stacks))
            .ok_or_else(|| numeric_fault(66, *stacks))?;
    }
    Ok(output)
}

pub(super) struct CriticalProfile {
    pub(super) chance: crate::Probability,
    pub(super) damage: crate::Scalar,
}

struct BreakFormulaModifiers {
    damage_boost: crate::Scalar,
    vulnerability: crate::Scalar,
    mitigation: crate::Ratio,
}

#[derive(Clone, Copy)]
struct IncomingModifierContext {
    cause: Cause,
    source: crate::UnitId,
    target: crate::UnitId,
    element: Option<formula::model::CombatElement>,
    class: formula::model::DamageClass,
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

fn incoming_formula_modifier(
    resolver: &StatResolver<'_>,
    catalog: &crate::catalog::CombatCatalog,
    txn: &Transaction<'_>,
    input: IncomingModifierContext,
    stage: FormulaStage,
    purpose: FormulaPurpose,
) -> Result<crate::Scalar, BattleFault> {
    let context = action_modifier_context(
        catalog,
        input.cause,
        modifier_context(txn, input.target, input.target, input.element, input.class)?,
    );
    let unscoped = if input.source == input.target
        && matches!(purpose, FormulaPurpose::Healing | FormulaPurpose::Shield)
    {
        crate::Scalar::ZERO
    } else {
        formula_modifier(resolver, input.target, stage, purpose, &context)?
    };
    let directional = formula_modifier(
        resolver,
        input.target,
        stage,
        purpose,
        &context.with_formula_subject(FormulaSubject::Target),
    )?;
    unscoped
        .checked_add(directional)
        .map_err(|_| numeric_fault(49, unscoped.scaled()))
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

fn break_modifier_context(
    txn: &Transaction<'_>,
    subject: crate::UnitId,
    target: crate::UnitId,
    element: formula::model::CombatElement,
    purpose: FormulaPurpose,
) -> Result<ModifierQueryContext, BattleFault> {
    let mut context = modifier_context(
        txn,
        subject,
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
    Ok(context)
}

fn action_modifier_context(
    catalog: &crate::catalog::CombatCatalog,
    cause: Cause,
    mut context: ModifierQueryContext,
) -> ModifierQueryContext {
    let Some(action) = cause
        .source_definition()
        .and_then(|source| crate::AbilityId::new(source.get()))
        .and_then(|ability| catalog.ability(ability))
        .and_then(crate::catalog::definition::AbilityDefinition::action)
    else {
        return context;
    };
    let mut tags = [
        (crate::catalog::action::AbilityTag::Attack, "attack"),
        (crate::catalog::action::AbilityTag::Basic, "basic"),
        (crate::catalog::action::AbilityTag::Skill, "skill"),
        (crate::catalog::action::AbilityTag::Ultimate, "ultimate"),
        (crate::catalog::action::AbilityTag::FollowUp, "follow_up"),
        (crate::catalog::action::AbilityTag::Counter, "counter"),
        (crate::catalog::action::AbilityTag::Summon, "summon"),
        (crate::catalog::action::AbilityTag::Memosprite, "memosprite"),
        (
            crate::catalog::action::AbilityTag::AdditionalDamage,
            "additional_damage",
        ),
        (crate::catalog::action::AbilityTag::Joint, "joint"),
        (
            crate::catalog::action::AbilityTag::ElationSkill,
            "elation_skill",
        ),
        (crate::catalog::action::AbilityTag::Assist, "assist"),
    ]
    .into_iter()
    .filter(|(tag, _)| action.tags().contains(*tag))
    .map(|(_, key)| Box::<str>::from(key))
    .collect::<Vec<_>>();
    tags.sort_unstable();
    context.ability_tags = tags.into_boxed_slice();
    context.action_kind = Some(action.kind() as u8);
    context.source_class = Some(crate::rule::model::SourceClass::Ability);
    context
}

const fn damage_purpose(class: formula::model::DamageClass) -> FormulaPurpose {
    match class {
        formula::model::DamageClass::Direct => FormulaPurpose::OrdinaryDamage,
        formula::model::DamageClass::Dot => FormulaPurpose::Dot,
        formula::model::DamageClass::Additional => FormulaPurpose::AdditionalDamage,
        formula::model::DamageClass::Elation => FormulaPurpose::ElationDamage,
    }
}
