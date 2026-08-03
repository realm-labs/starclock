//! Executable combat rules for Goal 07 Curio partition M11-S01.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectRuntimeTemplate, EffectStackPolicy,
    EffectTickPhase,
    catalog::definition::{EffectDefinition, ProgramDefinition, SelectorDefinition},
    modifier::model::{
        FormulaPurpose, FormulaStage, FormulaSubject, ModifierAggregation, ModifierDefinition,
        ModifierFilter, ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleValue, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::parameter;
use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, all_ally_selector, multiply,
    propagation_s01, scalar,
};

const FAMILY_EFFECT: &str = "75";
const TONIC_EFFECT: &str = "83";
const LOCAL_PROGRAM_BASE: u32 = 0x77e1_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x77e2_0000;
const LOCAL_EFFECT_BASE: u32 = 0x77e3_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x77e4_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x77e5_0000;
const LOCAL_GROUP_BASE: u32 = 0x77e6_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some((binding, contribution)) = state_binding(bindings, curios, FAMILY_EFFECT)? {
        let destroyed = curios.destroyed_curios();
        if destroyed != 0 {
            output.push(family_ties(binding, contribution, destroyed)?);
        }
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, TONIC_EFFECT)? {
        output.push(tonic(binding, contribution)?);
    }
    Ok(output)
}

fn family_ties(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
    destroyed: u32,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let ratio = parameter(contribution.state().parameters(), 0)?
        .checked_mul(i64::from(destroyed))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    permanent_team_modifiers(
        binding,
        1,
        damage_modifiers(binding.rule().get(), scalar(ratio), &[])?,
    )
}

fn tonic(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let bonus = parameter(contribution.state().parameters(), 0)?;
    let hp_ratio = parameter(contribution.state().parameters(), 1)?;
    let filters = [ModifierFilter::AbilityTag("technique".into())];
    let mut modifiers = damage_modifiers(binding.rule().get(), scalar(bonus), &filters)?;
    modifiers.push(ModifierDefinition {
        id: local(LOCAL_MODIFIER_BASE, binding.rule().get(), 8)?,
        stat: StatKind::Hp,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: multiply(
            ValueExpr::QueryStat {
                subject: StatQuerySubject::Actor,
                stat: StatKind::Hp,
                purpose: FormulaPurpose::Stat,
            },
            scalar(hp_ratio),
        ),
        stacking_group: local(LOCAL_GROUP_BASE, binding.rule().get(), 8)?,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: source_filters(&filters),
    });
    permanent_team_modifiers(binding, 2, modifiers)
}

pub(super) fn damage_modifiers(
    raw: u32,
    value: ValueExpr,
    extra_filters: &[ModifierFilter],
) -> Result<Vec<ModifierDefinition>, BattleRuleLoweringError> {
    [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::ElationDamage,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, purpose)| {
        let index = u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
        Ok(ModifierDefinition {
            id: local(LOCAL_MODIFIER_BASE, raw, index)?,
            stat: StatKind::Hp,
            stage: FormulaStage::DamageBoost,
            purpose,
            value: value.clone(),
            stacking_group: local(LOCAL_GROUP_BASE, raw, index)?,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: source_filters(extra_filters),
        })
    })
    .collect()
}

pub(super) fn permanent_team_modifiers(
    binding: &UniverseBattleRuleBinding,
    identity: u32,
    modifiers: Vec<ModifierDefinition>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let selector = local(LOCAL_SELECTOR_BASE, raw, identity)?;
    let effect = local(LOCAL_EFFECT_BASE, raw, identity)?;
    let program = local(LOCAL_PROGRAM_BASE, raw, identity)?;
    let trigger = local(LOCAL_TRIGGER_BASE, raw, identity)?;
    let groups = modifiers
        .iter()
        .map(|modifier| ModifierStackingGroup {
            id: modifier.stacking_group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        })
        .collect::<Vec<_>>();
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let effect_definition = EffectDefinition::new(effect, Vec::new(), modifier_ids)
        .with_runtime_template(permanent_marker()?);
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![selector],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector,
            effect,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )]);
    let trigger_definition = TriggerDef {
        id: trigger,
        event: RuleEventPoint::BattleStarted.kind(),
        event_point: RuleEventPoint::BattleStarted,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Battle,
        priority: ReactionPriority::new(0),
        program,
    };
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        groups,
        modifiers,
        vec![SelectorDefinition::new(selector).with_rule_units(all_ally_selector()?)],
        vec![effect_definition],
        vec![program_definition],
        vec![trigger_definition],
        Vec::new(),
    ))
}

fn state_binding<'a>(
    bindings: &'a [UniverseBattleRuleBinding],
    curios: &'a CurioContributionSet,
    effect: &str,
) -> Result<Option<(&'a UniverseBattleRuleBinding, &'a CurioContribution)>, BattleRuleLoweringError>
{
    let Some(contribution) = curios
        .entries()
        .iter()
        .find(|entry| entry.state().source_effect_id() == effect)
    else {
        return Ok(None);
    };
    let binding = bindings
        .iter()
        .find(|binding| {
            binding.role() == UniverseBattleRuleRole::CurioState
                && binding.source_binding_key() == Some(effect)
        })
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    Ok(Some((binding, contribution)))
}

fn source_filters(extra: &[ModifierFilter]) -> Box<[ModifierFilter]> {
    let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)];
    filters.extend_from_slice(extra);
    filters.into_boxed_slice()
}

fn permanent_marker() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn local<T: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<T, BattleRuleLoweringError> {
    base.checked_add((raw & 0xffff).saturating_mul(16))
        .and_then(|value| value.checked_add(offset))
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}
