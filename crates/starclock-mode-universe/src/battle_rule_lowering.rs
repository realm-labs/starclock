//! Executable Standard Universe combat slices lowered from validated contributions.

mod abundance_s01;
mod abundance_s02;
mod abundance_s03;
mod abundance_s04;
mod destruction_s01;
mod destruction_s02;
mod destruction_s03;
mod destruction_s04;
mod elation_s01;
mod hunt_resonance;
mod hunt_s01;
mod hunt_s02;
mod hunt_s03;
mod hunt_s04;
mod nihility_s01;
mod nihility_s02;
mod nihility_s03;
mod nihility_s04;
mod preservation_s02;
mod preservation_s03;
mod preservation_s04;
mod remembrance_s01;
mod remembrance_s02;
mod remembrance_s03;
mod remembrance_s04;
mod support;

use preservation_s02::*;
use starclock_combat::{
    AbilityId, EffectDefinitionId, ModifierDefinitionId, ModifierStackingGroupId, ProgramId, Ratio,
    Rounding, SelectorId, SourceDefinitionId, StateSlotDefinitionId, TriggerId,
    catalog::{
        action::{
            AbilityActionDefinition, AbilityKind, AbilityProgramBinding, AbilityProgramTiming,
            AbilityTag, ActionHitDefinition, ActionResourcePolicy, HitCritPolicy,
            TargetInvalidationPolicy, TargetPattern, TargetRelation, TeamResourceCost,
            UnitTargetSelector,
        },
        definition::{
            AbilityDefinition, EffectDefinition, ProgramDefinition, RuleBundle, RuleDefinition,
            SelectorDefinition,
        },
    },
    formula::model::{CombatElement, DamageClass},
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleDefinition, BattleRuleScope, Comparison, ConditionExpr, EventFilter,
        EventValueProperty, OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind,
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleResourceKind,
        RuleToughnessEventKind, RuleValue, RuleValueKind, ShieldObservation, SlotResetPoint,
        StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};
use starclock_combat::{
    ControlledAction, DispelCategory, DurationClock, EffectCategory, EffectRuntimeDefinition,
    EffectRuntimeTemplate, EffectSnapshotPolicy, EffectStackPolicy, EffectTickPhase,
    rng::types::DrawPurpose, rule::model::RuleEffectChancePolicy,
};
use support::*;

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    blessing_runtime::BlessingContributionSet,
    catalog::UniverseCatalog,
    curio_runtime::CurioContributionSet,
    path::ExactParameter,
};

const PROGRAM_ID_BASE: u32 = 0x7600_0000;
const BODY_PROGRAM_ID_BASE: u32 = 0x7601_0000;
const OWNER_SELECTOR_ID_BASE: u32 = 0x7610_0000;
const TARGET_SELECTOR_ID_BASE: u32 = 0x7611_0000;
const ALL_TARGET_SELECTOR_ID_BASE: u32 = 0x7612_0000;
const CURRENT_TARGET_SELECTOR_ID_BASE: u32 = 0x7613_0000;
const TRIGGER_ID_BASE: u32 = 0x7620_0000;
const AUX_PROGRAM_ID_BASE: u32 = 0x7640_0000;
const SECOND_AUX_PROGRAM_ID_BASE: u32 = 0x7650_0000;
const EFFECT_ID_BASE: u32 = 0x7660_0000;
const AMOUNT_SLOT_ID_BASE: u32 = 0x7670_0000;
const COUNTER_SLOT_ID_BASE: u32 = 0x7680_0000;
const SECOND_TRIGGER_ID_BASE: u32 = 0x7690_0000;
const THIRD_TRIGGER_ID_BASE: u32 = 0x76a0_0000;
const FOURTH_TRIGGER_ID_BASE: u32 = 0x76b0_0000;
const MODIFIER_ID_BASE: u32 = 0x76c0_0000;
const MODIFIER_GROUP_ID_BASE: u32 = 0x76d0_0000;

pub(crate) const RESONANCE_ABILITY_ID: AbilityId =
    AbilityId::new(0x7630_0001).expect("reserved ability ID is non-zero");
pub(crate) const RESONANCE_PROGRAM_ID: ProgramId =
    ProgramId::new(0x7630_0002).expect("reserved program ID is non-zero");
pub(crate) const RESONANCE_SELECTOR_ID: SelectorId =
    SelectorId::new(0x7630_0003).expect("reserved selector ID is non-zero");
pub(crate) const RESONANCE_RESOURCE_ID: SourceDefinitionId =
    SourceDefinitionId::new(0x7630_0004).expect("reserved resource ID is non-zero");
pub(crate) const RESONANCE_RESOURCE_KEY: &str = "standard-universe.path-resonance-energy";
pub(crate) const RESONANCE_ENEMY_SELECTOR_ID: SelectorId =
    SelectorId::new(0x7630_0005).expect("reserved selector ID is non-zero");
pub(crate) const RESONANCE_ALLY_SELECTOR_ID: SelectorId =
    SelectorId::new(0x7630_0006).expect("reserved selector ID is non-zero");

const ENTRY_ENEMY_DAMAGE_BINDING: &str = "8";
const HUNT_RESONANCE_BINDING: &str = "StageAbility_612420";
const PRESERVATION_ATTACK_QUAKE_BINDING: &str = "StageAbility_612030";
const PRESERVATION_RETALIATORY_QUAKE_BINDING: &str = "StageAbility_612031";
const PRESERVATION_MACROSEGREGATION_BINDING: &str = "StageAbility_612032";
const PRESERVATION_QUAKE_SPLASH_BINDING: &str = "StageAbility_612040";
const PRESERVATION_QUAKE_BLEED_BINDING: &str = "StageAbility_612041";
const PRESERVATION_SHIELD_STRENGTH_BINDING: &str = "StageAbility_612042";
const PRESERVATION_SAFE_LOAD_BINDING: &str = "StageAbility_612043";
const PRESERVATION_SANCTUARY_BINDING: &str = "StageAbility_612044";
const PRESERVATION_SHIELD_CAPACITY_BINDING: &str = "StageAbility_612045";
const PRESERVATION_PROVIDER_SHIELD_BINDING: &str = "StageAbility_612046";
const PRESERVATION_ASSEMBLE_BINDING: &str = "StageAbility_612050";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RuleAttachment {
    EveryPlayer,
    FirstPlayer,
    EveryEnemy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableBattleRule {
    attachment: RuleAttachment,
    modifier_groups: Box<[ModifierStackingGroup]>,
    modifiers: Box<[ModifierDefinition]>,
    selectors: Box<[SelectorDefinition]>,
    effects: Box<[EffectDefinition]>,
    programs: Box<[ProgramDefinition]>,
    definition: RuleDefinition,
    bundle: RuleBundle,
}

impl ExecutableBattleRule {
    pub(crate) const fn attachment(&self) -> RuleAttachment {
        self.attachment
    }
    pub(crate) fn modifier_groups(&self) -> &[ModifierStackingGroup] {
        &self.modifier_groups
    }
    pub(crate) fn modifiers(&self) -> &[ModifierDefinition] {
        &self.modifiers
    }
    pub(crate) fn selectors(&self) -> &[SelectorDefinition] {
        &self.selectors
    }
    pub(crate) fn programs(&self) -> &[ProgramDefinition] {
        &self.programs
    }
    pub(crate) fn effects(&self) -> &[EffectDefinition] {
        &self.effects
    }
    pub(crate) const fn definition(&self) -> &RuleDefinition {
        &self.definition
    }
    pub(crate) const fn bundle(&self) -> &RuleBundle {
        &self.bundle
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExecutableResonance {
    modifier_groups: Box<[ModifierStackingGroup]>,
    modifiers: Box<[ModifierDefinition]>,
    selectors: Box<[SelectorDefinition]>,
    effects: Box<[EffectDefinition]>,
    programs: Box<[ProgramDefinition]>,
    ability: AbilityDefinition,
    auxiliary_abilities: Box<[AbilityDefinition]>,
    countdowns: Box<[starclock_combat::CountdownCatalogDefinition]>,
    initial_energy: u16,
    maximum_energy: u16,
}

impl ExecutableResonance {
    pub(crate) fn modifier_groups(&self) -> &[ModifierStackingGroup] {
        &self.modifier_groups
    }
    pub(crate) fn modifiers(&self) -> &[ModifierDefinition] {
        &self.modifiers
    }
    pub(crate) fn selectors(&self) -> &[SelectorDefinition] {
        &self.selectors
    }
    pub(crate) fn effects(&self) -> &[EffectDefinition] {
        &self.effects
    }
    pub(crate) fn programs(&self) -> &[ProgramDefinition] {
        &self.programs
    }
    pub(crate) const fn ability(&self) -> &AbilityDefinition {
        &self.ability
    }
    pub(crate) fn auxiliary_abilities(&self) -> &[AbilityDefinition] {
        &self.auxiliary_abilities
    }
    pub(crate) fn countdowns(&self) -> &[starclock_combat::CountdownCatalogDefinition] {
        &self.countdowns
    }
    pub(crate) const fn initial_energy(&self) -> u16 {
        self.initial_energy
    }
    pub(crate) const fn maximum_energy(&self) -> u16 {
        self.maximum_energy
    }
}

pub(crate) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
    curios: &CurioContributionSet,
    initial_resonance_energy: u16,
    resonance_damage_ratio: i64,
) -> Result<(Vec<ExecutableBattleRule>, Option<ExecutableResonance>), BattleRuleLoweringError> {
    let mut output = Vec::new();
    let quake_boost = selected_level_parameters(blessings, PRESERVATION_QUAKE_SPLASH_BINDING)
        .map(|parameters| parameter(parameters, 2))
        .transpose()?
        .unwrap_or(0);
    let defense_quake = selected_level_parameters(blessings, PRESERVATION_SHIELD_STRENGTH_BINDING)
        .map(|parameters| parameter(parameters, 0))
        .transpose()?
        .unwrap_or(0);
    for binding in bindings.iter().filter(|binding| {
        binding.role() == UniverseBattleRuleRole::BlessingLevel
            && matches!(
                binding.source_binding_key(),
                Some(PRESERVATION_ATTACK_QUAKE_BINDING)
                    | Some(PRESERVATION_RETALIATORY_QUAKE_BINDING)
                    | Some(PRESERVATION_MACROSEGREGATION_BINDING)
            )
    }) {
        let contribution = blessings
            .entries()
            .iter()
            .find(|entry| {
                entry.level().source_binding_key()
                    == binding
                        .source_binding_key()
                        .expect("filtered binding has key")
            })
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        let parameters = contribution.level().parameters();
        output.push(match binding.source_binding_key() {
            Some(PRESERVATION_ATTACK_QUAKE_BINDING) => {
                preservation_attack_quake(binding, parameters, quake_boost, defense_quake)?
            }
            Some(PRESERVATION_RETALIATORY_QUAKE_BINDING) => {
                preservation_retaliatory_quake(binding, parameters, quake_boost, defense_quake)?
            }
            Some(PRESERVATION_MACROSEGREGATION_BINDING) => {
                preservation_macrosegregation(binding, parameters)?
            }
            _ => unreachable!("filtered binding"),
        });
    }
    let quake_sources = [
        PRESERVATION_ATTACK_QUAKE_BINDING,
        PRESERVATION_RETALIATORY_QUAKE_BINDING,
    ]
    .into_iter()
    .filter_map(|key| level_binding(bindings, key).map(|binding| binding.source().definition()))
    .collect::<Vec<_>>();
    if let Some(binding) = level_binding(bindings, PRESERVATION_QUAKE_SPLASH_BINDING) {
        let parameters = selected_level_parameters(blessings, PRESERVATION_QUAKE_SPLASH_BINDING)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        if !quake_sources.is_empty() {
            output.push(preservation_quake_splash(
                binding,
                parameters,
                &quake_sources,
            )?);
        }
    }
    if let Some(binding) = level_binding(bindings, PRESERVATION_QUAKE_BLEED_BINDING) {
        let parameters = selected_level_parameters(blessings, PRESERVATION_QUAKE_BLEED_BINDING)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        let mut sources = quake_sources;
        if let Some(splash) = level_binding(bindings, PRESERVATION_QUAKE_SPLASH_BINDING) {
            sources.push(splash.source().definition());
        }
        if !sources.is_empty() {
            output.push(preservation_quake_bleed(binding, parameters, &sources)?);
        }
    }
    for key in [
        PRESERVATION_SAFE_LOAD_BINDING,
        PRESERVATION_SANCTUARY_BINDING,
        PRESERVATION_SHIELD_CAPACITY_BINDING,
        PRESERVATION_PROVIDER_SHIELD_BINDING,
        PRESERVATION_ASSEMBLE_BINDING,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            PRESERVATION_SAFE_LOAD_BINDING => preservation_safe_load(binding, parameters)?,
            PRESERVATION_SANCTUARY_BINDING => preservation_sanctuary(binding, parameters)?,
            PRESERVATION_SHIELD_CAPACITY_BINDING => {
                preservation_shield_capacity(binding, parameters)?
            }
            PRESERVATION_PROVIDER_SHIELD_BINDING => {
                preservation_provider_shield(binding, parameters)?
            }
            PRESERVATION_ASSEMBLE_BINDING => preservation_assemble(
                binding,
                parameters,
                preservation_blessing_count(catalog, blessings)?,
            )?,
            _ => unreachable!("closed Preservation S02 binding set"),
        });
    }
    output.extend(preservation_s03::lower(bindings, blessings)?);
    output.extend(preservation_s04::lower_rules(catalog, bindings, blessings)?);
    output.extend(remembrance_s01::lower(bindings, blessings)?);
    output.extend(remembrance_s02::lower(catalog, bindings, blessings)?);
    output.extend(remembrance_s03::lower(bindings, blessings)?);
    output.extend(remembrance_s04::lower_rules(catalog, bindings, blessings)?);
    output.extend(nihility_s01::lower(bindings, blessings)?);
    output.extend(nihility_s02::lower(catalog, bindings, blessings)?);
    output.extend(nihility_s03::lower(bindings, blessings)?);
    output.extend(nihility_s04::lower_rules(catalog, bindings, blessings)?);
    output.extend(abundance_s01::lower(bindings, blessings)?);
    output.extend(abundance_s02::lower(catalog, bindings, blessings)?);
    output.extend(abundance_s03::lower(bindings, blessings)?);
    output.extend(abundance_s04::lower_rules(catalog, bindings, blessings)?);
    let mut destruction_rules = destruction_s01::lower(bindings, blessings)?;
    destruction_rules.extend(destruction_s02::lower(catalog, bindings, blessings)?);
    destruction_rules.extend(destruction_s03::lower(bindings, blessings)?);
    destruction_rules.extend(destruction_s04::lower_rules(catalog, bindings, blessings)?);
    if let Some(first) = destruction_rules.first_mut() {
        destruction_s01::add_grit_engine(first, blessings)?;
    }
    output.extend(destruction_rules);
    let mut hunt_rules = hunt_s01::lower(bindings, blessings)?;
    hunt_rules.extend(hunt_s02::lower(catalog, bindings, blessings)?);
    hunt_rules.extend(hunt_s03::lower(bindings, blessings)?);
    hunt_rules.extend(hunt_s04::lower(catalog, bindings, blessings)?);
    if let Some(first) = hunt_rules.first_mut() {
        hunt_s01::add_critical_boost(first, blessings)?;
    }
    output.extend(hunt_rules);
    output.extend(elation_s01::lower(bindings, blessings)?);
    if let Some(binding) = bindings.iter().find(|binding| {
        binding.role() == UniverseBattleRuleRole::CurioState
            && binding.source_binding_key() == Some(ENTRY_ENEMY_DAMAGE_BINDING)
    }) {
        let contribution = curios
            .entries()
            .iter()
            .find(|entry| entry.state().source_effect_id() == ENTRY_ENEMY_DAMAGE_BINDING)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        let ratio = parameter(contribution.state().parameters(), 0)?;
        output.push(entry_enemy_damage(binding, ratio)?);
    }
    output.sort_unstable_by_key(|rule| rule.bundle().id());

    let resonance = bindings
        .iter()
        .find(|binding| {
            binding.role() == UniverseBattleRuleRole::Resonance
                && matches!(
                    binding.source_binding_key(),
                    Some(HUNT_RESONANCE_BINDING)
                        | Some(preservation_s04::RESONANCE)
                        | Some(remembrance_s04::RESONANCE)
                        | Some(nihility_s04::RESONANCE)
                        | Some(abundance_s04::RESONANCE)
                        | Some(destruction_s04::RESONANCE)
                )
        })
        .map(|binding| match binding.source_binding_key() {
            Some(preservation_s04::RESONANCE) => preservation_s04::resonance(
                catalog,
                bindings,
                binding,
                initial_resonance_energy,
                resonance_damage_ratio,
            ),
            Some(remembrance_s04::RESONANCE) => remembrance_s04::resonance(
                catalog,
                bindings,
                binding,
                initial_resonance_energy,
                resonance_damage_ratio,
            ),
            Some(nihility_s04::RESONANCE) => {
                nihility_s04::resonance(catalog, bindings, binding, initial_resonance_energy)
            }
            Some(abundance_s04::RESONANCE) => {
                abundance_s04::resonance(catalog, bindings, binding, initial_resonance_energy)
            }
            Some(destruction_s04::RESONANCE) => destruction_s04::resonance(
                catalog,
                bindings,
                binding,
                initial_resonance_energy,
                resonance_damage_ratio,
            ),
            _ => hunt_resonance::lower(
                catalog,
                bindings,
                binding,
                initial_resonance_energy,
                resonance_damage_ratio,
            ),
        })
        .transpose()?;
    Ok((output, resonance))
}

fn preservation_attack_quake(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    quake_boost: i64,
    defense_quake: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
        SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
        SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
    ];
    let owner_shield = shield(StatQuerySubject::Owner, ShieldObservation::Current);
    let party_shield = ValueExpr::SelectorSum {
        selector: allies,
        value: Box::new(shield(
            StatQuerySubject::CurrentTarget,
            ShieldObservation::Current,
        )),
    };
    let teammate_shield =
        ValueExpr::Subtract(Box::new(party_shield), Box::new(owner_shield.clone()));
    let amount = multiply(
        ValueExpr::Add(
            Box::new(multiply(
                ValueExpr::Add(
                    Box::new(owner_shield),
                    Box::new(multiply(teammate_shield, scalar(parameter(parameters, 1)?))),
                ),
                scalar(parameter(parameters, 0)?),
            )),
            Box::new(multiply(
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::Owner,
                    stat: StatKind::Def,
                    purpose: FormulaPurpose::Stat,
                },
                scalar(defense_quake),
            )),
        ),
        scalar(
            1_000_000_i64
                .checked_add(quake_boost)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ),
    );
    let definition = executable_damage_rule(
        binding,
        program,
        trigger,
        target,
        selectors,
        amount,
        OnceScope::TargetWithinAction,
        EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(AbilityTag::Attack),
            ..EventFilter::default()
        },
        true,
    );
    Ok(definition)
}

fn preservation_retaliatory_quake(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    quake_boost: i64,
    defense_quake: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let attacker = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let observation = if parameter(parameters, 1)? == 2_000_000 {
        ShieldObservation::BeforeEvent
    } else {
        ShieldObservation::Current
    };
    let amount = multiply(
        ValueExpr::Add(
            Box::new(multiply(
                shield(StatQuerySubject::Owner, observation),
                scalar(parameter(parameters, 0)?),
            )),
            Box::new(multiply(
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::Owner,
                    stat: StatKind::Def,
                    purpose: FormulaPurpose::Stat,
                },
                scalar(defense_quake),
            )),
        ),
        scalar(
            1_000_000_i64
                .checked_add(quake_boost)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ),
    );
    Ok(executable_damage_rule(
        binding,
        program,
        trigger,
        attacker,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(attacker).with_rule_units(actor_enemy_selector()?),
        ],
        amount,
        OnceScope::Action,
        EventFilter {
            target_selector: Some(owner),
            ..EventFilter::default()
        },
        false,
    ))
}

fn preservation_macrosegregation(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let start_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let strengthen_program = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let advance_cycle_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let reset_cycle_program = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let amount_slot = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let counter_slot = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let start_trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let strengthen_trigger = id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?;
    let advance_trigger = id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?;
    let reset_trigger = id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?;
    let base = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 0)?),
    );
    let shield_delta = ValueExpr::ReadEventProperty(EventValueProperty::ShieldChangeAmount);
    let strengthened = ValueExpr::Add(
        Box::new(ValueExpr::Slot(amount_slot)),
        Box::new(multiply(
            shield_delta.clone(),
            scalar(parameter(parameters, 1)?),
        )),
    );
    let programs = vec![
        slot_and_shield_program(
            start_program,
            owner,
            effect,
            amount_slot,
            Some((counter_slot, RuleValue::Integer(0))),
            base.clone(),
        ),
        slot_and_shield_program(
            strengthen_program,
            owner,
            effect,
            amount_slot,
            None,
            strengthened,
        ),
        ProgramDefinition::new(
            advance_cycle_program,
            Vec::new(),
            vec![owner],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::SetSlot {
                slot: counter_slot,
                value: ValueExpr::Literal(RuleValue::Integer(1)),
            },
        )]),
        slot_and_shield_program(
            reset_cycle_program,
            owner,
            effect,
            amount_slot,
            Some((counter_slot, RuleValue::Integer(0))),
            base,
        ),
    ];
    let positive_shield = ConditionExpr::Compare {
        lhs: Box::new(shield_delta),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    };
    let foreign_source = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::SourceDefinitionId,
        )),
        operator: Comparison::NotEqual,
        rhs: Box::new(ValueExpr::Literal(RuleValue::OptionalStableId(Some(
            u64::from(binding.source().definition().get()),
        )))),
    };
    let triggers = vec![
        trigger(
            start_trigger,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            start_program,
        ),
        trigger(
            strengthen_trigger,
            RuleEventPoint::ShieldChanged,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            ConditionExpr::All(vec![positive_shield, foreign_source].into_boxed_slice()),
            strengthen_program,
        ),
        trigger(
            advance_trigger,
            RuleEventPoint::TurnEnded,
            OnceScope::Turn,
            EventFilter {
                owner_selector: Some(owner),
                ..EventFilter::default()
            },
            integer_slot_equals(counter_slot, 0),
            advance_cycle_program,
        ),
        trigger(
            reset_trigger,
            RuleEventPoint::TurnEnded,
            OnceScope::Turn,
            EventFilter {
                owner_selector: Some(owner),
                ..EventFilter::default()
            },
            integer_slot_equals(counter_slot, 1),
            reset_cycle_program,
        ),
    ];
    let state_slots = vec![
        StateSlotDef::new(
            amount_slot,
            RuleValueKind::Scalar,
            BattleRuleScope::Battle,
            RuleValue::Scalar(starclock_combat::Scalar::ZERO),
        ),
        StateSlotDef::new(
            counter_slot,
            RuleValueKind::Integer,
            BattleRuleScope::Battle,
            RuleValue::Integer(0),
        )
        .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
    ];
    let selectors = vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)];
    let definition = RuleDefinition::new(
        binding.rule(),
        programs.iter().map(ProgramDefinition::id).collect(),
        vec![owner],
    )
    .with_runtime(BattleRuleDefinition::new(
        binding.source().clone(),
        state_slots,
        triggers,
        None,
    ));
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: vec![EffectDefinition::new(effect, Vec::new(), Vec::new())].into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn preservation_quake_splash(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    quake_sources: &[SourceDefinitionId],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let filter_body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let damage_body = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let all_enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let programs = vec![
        ProgramDefinition::new(
            root,
            Vec::new(),
            vec![owner, all_enemies],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: all_enemies,
            body: filter_body,
            maximum: 16,
        }]),
        ProgramDefinition::new(
            filter_body,
            Vec::new(),
            vec![current],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::If {
            condition: ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::CurrentTarget),
                operator: Comparison::NotEqual,
                rhs: Box::new(ValueExpr::EventTarget),
            },
            then_program: damage_body,
            else_program: None,
        }]),
        ProgramDefinition::new(
            damage_body,
            Vec::new(),
            vec![current],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::Damage {
                selector: current,
                amount: multiply(
                    ValueExpr::ReadEventProperty(EventValueProperty::DamageAmount),
                    scalar(parameter(parameters, 0)?),
                ),
                class: DamageClass::Additional,
                element: CombatElement::Physical,
                can_crit: false,
                can_defeat: true,
            },
        )]),
    ];
    let triggers = quake_sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let base = match index {
                0 => TRIGGER_ID_BASE,
                1 => SECOND_TRIGGER_ID_BASE,
                _ => THIRD_TRIGGER_ID_BASE,
            };
            Ok(trigger(
                id::<TriggerId>(base, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::Event,
                EventFilter {
                    owner_selector: Some(owner),
                    source: Some(*source),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                root,
            ))
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    Ok(executable_rule(
        binding,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(all_enemies).with_rule_units(all_enemy_selector()?),
            SelectorDefinition::new(current).with_rule_units(current_subject_selector()?),
        ],
        Vec::new(),
        programs,
        Vec::new(),
        triggers,
    ))
}

fn preservation_quake_bleed(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    quake_sources: &[SourceDefinitionId],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let duration = parameter(parameters, 3)?
        .checked_div(1_000_000)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let magnitude = ValueExpr::Minimum(
        Box::new(multiply(
            ValueExpr::QueryStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Hp,
                purpose: FormulaPurpose::Stat,
            },
            scalar(parameter(parameters, 1)?),
        )),
        Box::new(multiply(
            ValueExpr::ReadEventProperty(EventValueProperty::DamageAmount),
            scalar(parameter(parameters, 2)?),
        )),
    );
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Dot,
        DispelCategory::DispellableDebuff,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(duration))),
        DurationClock::TargetTurnStart,
        EffectTickPhase::TurnStart,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_comparison(Some(magnitude), 0)
    .with_snapshot(EffectSnapshotPolicy::OnApplication)
    .with_dot(CombatElement::Physical, None)
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let chance = parameter(parameters, 0)?;
    let operation = RuleOperationTemplate::ApplyEffect {
        selector: target,
        effect,
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: if chance >= 1_000_000 {
            RuleEffectChancePolicy::Guaranteed
        } else {
            RuleEffectChancePolicy::Resistible
        },
        base_chance: (chance < 1_000_000).then(|| scalar(chance)),
        rng_purpose: (chance < 1_000_000).then_some(DrawPurpose::EFFECT_CHANCE),
    };
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner, target],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(operation)]);
    let triggers = quake_sources
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let base = match index {
                0 => TRIGGER_ID_BASE,
                1 => SECOND_TRIGGER_ID_BASE,
                _ => THIRD_TRIGGER_ID_BASE,
            };
            Ok(trigger(
                id::<TriggerId>(base, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::Event,
                EventFilter {
                    owner_selector: Some(owner),
                    source: Some(*source),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            ))
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    Ok(executable_rule(
        binding,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
        ],
        vec![EffectDefinition::new(effect, Vec::new(), Vec::new()).with_runtime_template(runtime)],
        vec![program_definition],
        Vec::new(),
        triggers,
    ))
}

fn executable_rule(
    binding: &UniverseBattleRuleBinding,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    state_slots: Vec<StateSlotDef>,
    triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    let selector_ids = selectors
        .iter()
        .map(SelectorDefinition::id)
        .collect::<Vec<_>>();
    let program_ids = programs
        .iter()
        .map(ProgramDefinition::id)
        .collect::<Vec<_>>();
    let definition = RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
        BattleRuleDefinition::new(binding.source().clone(), state_slots, triggers, None),
    );
    ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}

fn slot_and_shield_program(
    program: ProgramId,
    owner: SelectorId,
    effect: EffectDefinitionId,
    amount_slot: StateSlotDefinitionId,
    counter: Option<(StateSlotDefinitionId, RuleValue)>,
    amount: ValueExpr,
) -> ProgramDefinition {
    let mut steps = vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
        slot: amount_slot,
        value: amount.clone(),
    })];
    if let Some((slot, value)) = counter {
        steps.push(ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot,
            value: ValueExpr::Literal(value),
        }));
    }
    steps.extend([
        ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
            selector: owner,
            effect,
        }),
        ProgramStep::Operation(RuleOperationTemplate::Shield {
            selector: owner,
            amount,
            effect,
        }),
    ]);
    ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
        .with_steps(steps)
}

fn integer_slot_equals(slot: StateSlotDefinitionId, value: i64) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Equal,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(value))),
    }
}

fn trigger(
    id: TriggerId,
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
}

#[allow(clippy::too_many_arguments)]
fn executable_damage_rule(
    binding: &UniverseBattleRuleBinding,
    program: ProgramId,
    trigger: TriggerId,
    target: SelectorId,
    selectors: Vec<SelectorDefinition>,
    amount: ValueExpr,
    once_scope: OnceScope,
    filter: EventFilter,
    can_defeat: bool,
) -> ExecutableBattleRule {
    let selector_ids = selectors
        .iter()
        .map(SelectorDefinition::id)
        .collect::<Vec<_>>();
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        selector_ids.clone(),
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: target,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Physical,
            can_crit: false,
            can_defeat,
        },
    )]);
    let definition = RuleDefinition::new(binding.rule(), vec![program], selector_ids).with_runtime(
        BattleRuleDefinition::new(
            binding.source().clone(),
            Vec::new(),
            vec![TriggerDef {
                id: trigger,
                event: RuleEventKind::Damage,
                event_point: RuleEventPoint::DamageApplied,
                phase: TriggerPhase::AfterEvent,
                filter,
                condition: ConditionExpr::Literal(true),
                once_scope,
                priority: ReactionPriority::new(0),
                program,
            }],
            None,
        ),
    );
    ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: Box::new([]),
        programs: vec![program_definition].into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}

fn entry_enemy_damage(
    binding: &UniverseBattleRuleBinding,
    ratio: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let all_targets = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let current_target = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(all_targets).with_rule_units(all_enemy_selector()?),
        SelectorDefinition::new(current_target).with_rule_units(current_subject_selector()?),
    ];
    let root_definition =
        ProgramDefinition::new(root, Vec::new(), vec![all_targets], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::ForEach {
                selector: all_targets,
                body,
                maximum: 16,
            }]);
    let amount = ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::CurrentTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(ratio),
        ))),
        rounding: starclock_combat::Rounding::NearestTiesEven,
    };
    let body_definition = ProgramDefinition::new(
        body,
        Vec::new(),
        vec![current_target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::TrueDamage {
            selector: current_target,
            amount,
        },
    )]);
    let definition = RuleDefinition::new(
        binding.rule(),
        vec![root, body],
        vec![all_targets, current_target],
    )
    .with_runtime(BattleRuleDefinition::new(
        binding.source().clone(),
        Vec::new(),
        vec![TriggerDef {
            id: trigger,
            event: RuleEventKind::Battle,
            event_point: RuleEventPoint::BattleStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Battle,
            priority: ReactionPriority::new(-100),
            program: root,
        }],
        None,
    ));
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::FirstPlayer,
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: Box::new([]),
        programs: vec![root_definition, body_definition].into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BattleRuleLoweringError {
    SnapshotMismatch,
    InvalidParameter,
    InvalidDefinition,
}
