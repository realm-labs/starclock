//! Modifier-aware formula preparation separated from authoritative state mutation.

use crate::{
    catalog::{
        CombatCatalog,
        action::{AbilityKind, AbilityTag},
        definition::AbilityDefinition,
    },
    formula::sustain::{DamageCalculation, HealingCalculation},
    modifier::model::{StatKind, StatQuery},
    rule::model::SourceClass,
};
use std::collections::BTreeMap;

use crate::{
    AbilityId, EffectCategory, EffectDefinitionId, LifeState, PresenceState, Probability, Ratio,
    Rounding, Scalar, UnitId,
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
use super::{program, stat_input};

pub(super) struct FormulaInputs {
    bases: BTreeMap<(UnitId, StatKind), Scalar>,
    shields: BTreeMap<UnitId, Scalar>,
    effect_stacks: BTreeMap<(UnitId, EffectDefinitionId), i64>,
    effect_category_stacks: BTreeMap<(UnitId, EffectCategory), i64>,
    modifiers: Vec<ActiveModifier>,
}

impl FormulaInputs {
    pub(super) fn new(txn: &Transaction<'_>) -> Result<Self, BattleFault> {
        Ok(Self {
            bases: program::stat_bases(txn)?,
            shields: stat_input::shield_values(txn),
            effect_stacks: effect_stacks(txn)?,
            effect_category_stacks: effect_category_stacks(txn)?,
            modifiers: txn.state.modifiers.iter_by_id().cloned().collect(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn damage(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: OrdinaryDamageDefinition,
        element: Option<formula::model::CombatElement>,
        target: UnitId,
        apply_source_modifiers: bool,
        ultimate_semantics: bool,
    ) -> Result<DamageCalculation, BattleFault> {
        let resolver = self.resolver(catalog);
        let purpose = damage_purpose(formula.class());
        let source = formula_source(txn, cause, purpose)?;
        let source_context = damage_modifier_context(
            catalog,
            cause,
            modifier_context(txn, source, target, element, formula.class())?,
            ultimate_semantics,
        )
        .with_formula_subject(FormulaSubject::Source);
        let incoming_context = IncomingModifierContext {
            cause,
            source,
            target,
            element,
            class: formula.class(),
            ultimate_semantics,
        };
        if apply_source_modifiers {
            let flat = formula_modifier(
                &resolver,
                source,
                FormulaStage::Flat,
                purpose,
                &source_context,
            )?;
            formula = formula
                .with_flat_base(flat)
                .map_err(|_| numeric_fault(55, flat.scaled()))?;
            for stage in [
                FormulaStage::Crit,
                FormulaStage::DamageBoost,
                FormulaStage::Weaken,
                FormulaStage::Resistance,
            ] {
                let mut value =
                    formula_modifier(&resolver, source, stage, purpose, &source_context)?;
                if stage == FormulaStage::DamageBoost {
                    let target_value = formula_modifier(
                        &resolver,
                        target,
                        stage,
                        purpose,
                        &damage_modifier_context(
                            catalog,
                            cause,
                            modifier_context(txn, target, target, element, formula.class())?,
                            ultimate_semantics,
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
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        class: formula::model::DamageClass,
        target: UnitId,
        ultimate_semantics: bool,
    ) -> Result<CriticalProfile, BattleFault> {
        let purpose = damage_purpose(class);
        let source = formula_source(txn, cause, purpose)?;
        let resolver = self.resolver(catalog);
        let source_context = damage_modifier_context(
            catalog,
            cause,
            modifier_context(txn, source, target, None, class)?,
            ultimate_semantics,
        )
        .with_formula_subject(FormulaSubject::Source);
        let rate = resolver
            .query(
                StatQuery {
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
            &damage_modifier_context(
                catalog,
                cause,
                modifier_context(txn, target, target, None, class)?,
                ultimate_semantics,
            )
            .with_formula_subject(FormulaSubject::Target),
        )?;
        let damage = resolver
            .query(
                StatQuery {
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
            chance: formula::model::clamp_probability(Ratio::from_scaled(
                rate.checked_add(target_bonus)
                    .map_err(|_| numeric_fault(53, target_bonus.scaled()))?
                    .scaled(),
            )),
            damage,
        })
    }

    pub(super) fn energy_regeneration_rate(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        subject: UnitId,
    ) -> Result<Scalar, BattleFault> {
        self.resolver(catalog)
            .query(
                StatQuery {
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
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: formula::toughness::BreakDamageDefinition,
        element: formula::model::CombatElement,
        target: UnitId,
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
            .checked_add(Ratio::from_scaled(modifiers.damage_boost.scaled()))
            .map_err(|_| numeric_fault(57, modifiers.damage_boost.scaled()))?;
        formula.vulnerability_multiplier = formula
            .vulnerability_multiplier
            .checked_add(Ratio::from_scaled(modifiers.vulnerability.scaled()))
            .map_err(|_| numeric_fault(58, modifiers.vulnerability.scaled()))?;
        formula.mitigation_multiplier = formula
            .mitigation_multiplier
            .checked_mul(modifiers.mitigation, Rounding::NearestTiesEven)
            .map_err(|_| numeric_fault(59, modifiers.mitigation.scaled()))?;
        Ok(formula)
    }

    pub(super) fn super_break_damage(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        mut formula: formula::toughness::SuperBreakDefinition,
        target: UnitId,
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
            .checked_add(Ratio::from_scaled(modifiers.damage_boost.scaled()))
            .map_err(|_| numeric_fault(60, modifiers.damage_boost.scaled()))?;
        formula.vulnerability_multiplier = formula
            .vulnerability_multiplier
            .checked_add(Ratio::from_scaled(modifiers.vulnerability.scaled()))
            .map_err(|_| numeric_fault(61, modifiers.vulnerability.scaled()))?;
        formula.mitigation_multiplier = formula
            .mitigation_multiplier
            .checked_mul(modifiers.mitigation, Rounding::NearestTiesEven)
            .map_err(|_| numeric_fault(62, modifiers.mitigation.scaled()))?;
        Ok(formula)
    }

    fn break_formula_modifiers(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        target: UnitId,
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
        let mitigation = Ratio::ONE
            .checked_sub(Ratio::from_scaled(mitigation_value.scaled()))
            .map_err(|_| numeric_fault(65, i64::from(FormulaStage::Mitigation as u8)))?;
        Ok(BreakFormulaModifiers {
            damage_boost,
            vulnerability,
            mitigation,
        })
    }

    pub(super) fn weakness_break_efficiency(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        target: UnitId,
        element: formula::model::CombatElement,
    ) -> Result<Ratio, BattleFault> {
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
                StatQuery {
                    subject: source,
                    stat: StatKind::ToughnessDamage,
                    purpose,
                },
                &context,
            )
            .map_err(|_| numeric_fault(56, i64::from(StatKind::ToughnessDamage as u8)))?;
        Ok(Ratio::from_scaled(value.scaled()))
    }

    pub(super) fn toughness_recovery(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        target: UnitId,
    ) -> Result<Ratio, BattleFault> {
        let stat = StatKind::ToughnessRecovery;
        let value = self
            .resolver(catalog)
            .query(
                StatQuery {
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
        Ok(Ratio::from_scaled(value.scaled()))
    }

    pub(super) fn healing(
        &self,
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        formula: HealingDefinition,
        target: UnitId,
    ) -> Result<HealingCalculation, BattleFault> {
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
                ultimate_semantics: false,
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
        catalog: &CombatCatalog,
        txn: &Transaction<'_>,
        cause: Cause,
        formula: ShieldDefinition,
        target: UnitId,
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
                ultimate_semantics: false,
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
                ratio: Ratio::ONE,
            }]
            .into_boxed_slice(),
            additive_base: Scalar::ZERO,
            bonuses: vec![formula.bonus()].into_boxed_slice(),
        };
        formula::shield::calculate(&context)
            .map_err(|_| numeric_fault(9, formula.base_shield().scaled()))
    }

    fn resolver<'a>(&'a self, catalog: &'a CombatCatalog) -> StatResolver<'a> {
        StatResolver::new(catalog.modifier_registry(), &self.bases, &self.modifiers)
            .with_shields(&self.shields)
            .with_effect_stacks(&self.effect_stacks)
            .with_effect_category_stacks(&self.effect_category_stacks)
    }
}

fn effect_stacks(
    txn: &Transaction<'_>,
) -> Result<BTreeMap<(UnitId, EffectDefinitionId), i64>, BattleFault> {
    let mut output = BTreeMap::new();
    for effect in txn.state.effects.iter_by_id() {
        let stacks = output
            .entry((effect.target, effect.definition))
            .or_insert(0_i64);
        *stacks = stacks
            .checked_add(i64::from(effect.stacks))
            .ok_or_else(|| numeric_fault(67, *stacks))?;
    }
    Ok(output)
}

fn effect_category_stacks(
    txn: &Transaction<'_>,
) -> Result<BTreeMap<(UnitId, EffectCategory), i64>, BattleFault> {
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
    pub(super) chance: Probability,
    pub(super) damage: Scalar,
}

struct BreakFormulaModifiers {
    damage_boost: Scalar,
    vulnerability: Scalar,
    mitigation: Ratio,
}

#[derive(Clone, Copy)]
struct IncomingModifierContext {
    cause: Cause,
    source: UnitId,
    target: UnitId,
    element: Option<formula::model::CombatElement>,
    class: formula::model::DamageClass,
    ultimate_semantics: bool,
}

fn formula_modifier(
    resolver: &StatResolver<'_>,
    subject: UnitId,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    context: &ModifierQueryContext,
) -> Result<Scalar, BattleFault> {
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
    catalog: &CombatCatalog,
    txn: &Transaction<'_>,
    input: IncomingModifierContext,
    stage: FormulaStage,
    purpose: FormulaPurpose,
) -> Result<Scalar, BattleFault> {
    let context = damage_modifier_context(
        catalog,
        input.cause,
        modifier_context(txn, input.target, input.target, input.element, input.class)?,
        input.ultimate_semantics,
    );
    let unscoped = if input.source == input.target
        && matches!(purpose, FormulaPurpose::Healing | FormulaPurpose::Shield)
    {
        Scalar::ZERO
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
) -> Result<UnitId, BattleFault> {
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
    subject: UnitId,
    target: UnitId,
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
            LifeState::Alive => LifeFilter::Alive,
            LifeState::Downed => LifeFilter::Downed,
            LifeState::Defeated => LifeFilter::Defeated,
        }),
        presence: Some(match unit.presence {
            PresenceState::Present => PresenceFilter::Present,
            PresenceState::Reserved => PresenceFilter::Reserved,
            PresenceState::Departed => PresenceFilter::Departed,
            PresenceState::Untargetable => PresenceFilter::Untargetable,
            PresenceState::Linked => PresenceFilter::Linked,
            PresenceState::Transformed => PresenceFilter::Transformed,
        }),
        target: Some(target),
        ..ModifierQueryContext::default()
    })
}

fn break_modifier_context(
    txn: &Transaction<'_>,
    subject: UnitId,
    target: UnitId,
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
    catalog: &CombatCatalog,
    cause: Cause,
    mut context: ModifierQueryContext,
) -> ModifierQueryContext {
    let Some(action) = cause
        .source_definition()
        .and_then(|source| AbilityId::new(source.get()))
        .and_then(|ability| catalog.ability(ability))
        .and_then(AbilityDefinition::action)
    else {
        return context;
    };
    let mut tags = [
        (AbilityTag::Attack, "attack"),
        (AbilityTag::Basic, "basic"),
        (AbilityTag::Skill, "skill"),
        (AbilityTag::Ultimate, "ultimate"),
        (AbilityTag::FollowUp, "follow_up"),
        (AbilityTag::Counter, "counter"),
        (AbilityTag::Summon, "summon"),
        (AbilityTag::Memosprite, "memosprite"),
        (AbilityTag::AdditionalDamage, "additional_damage"),
        (AbilityTag::Joint, "joint"),
        (AbilityTag::ElationSkill, "elation_skill"),
        (AbilityTag::Assist, "assist"),
        (AbilityTag::PathResonance, "path_resonance"),
        (AbilityTag::Technique, "technique"),
    ]
    .into_iter()
    .filter(|(tag, _)| action.tags().contains(*tag))
    .map(|(_, key)| Box::<str>::from(key))
    .collect::<Vec<_>>();
    tags.sort_unstable();
    context.ability_tags = tags.into_boxed_slice();
    context.action_kind = Some(action.kind() as u8);
    context.source_class = Some(SourceClass::Ability);
    context
}

fn damage_modifier_context(
    catalog: &CombatCatalog,
    cause: Cause,
    mut context: ModifierQueryContext,
    ultimate_semantics: bool,
) -> ModifierQueryContext {
    if !ultimate_semantics {
        return action_modifier_context(catalog, cause, context);
    }
    context.ability_tags = vec!["attack".into(), "ultimate".into()].into_boxed_slice();
    context.action_kind = Some(AbilityKind::Ultimate as u8);
    context.source_class = Some(SourceClass::Ability);
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
