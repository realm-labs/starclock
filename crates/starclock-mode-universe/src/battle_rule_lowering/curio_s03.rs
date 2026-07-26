//! Executable combat rules for Goal 07 Curio partition M11-S03.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectRuntimeTemplate, EffectStackPolicy,
    EffectTickPhase, Scalar,
    catalog::{
        definition::{EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority, RuleDamageClass,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleValue, TriggerDef,
        TriggerPhase, ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio_activity::ROBE_FRAGMENT_SNAPSHOT_KEY,
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, all_ally_selector, curio_s01,
    propagation_s01, scalar,
};

const ROBE_EFFECT: &str = "14";
const PROTECTION_EFFECT: &str = "19";
const LOCAL_PROGRAM_BASE: u32 = 0x77d1_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x77d2_0000;
const LOCAL_EFFECT_BASE: u32 = 0x77d3_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x77d4_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x77d5_0000;
const LOCAL_GROUP_BASE: u32 = 0x77d6_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some((binding, contribution)) = state_binding(bindings, curios, ROBE_EFFECT)? {
        let fragments = curios
            .runtime_value(ROBE_FRAGMENT_SNAPSHOT_KEY)
            .unwrap_or(0);
        let divisor = whole(super::parameter(contribution.state().parameters(), 0)?)?;
        let stacks = fragments / divisor;
        if stacks > 0 {
            let ratio = super::parameter(contribution.state().parameters(), 1)?
                .checked_mul(stacks)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?;
            output.push(curio_s01::permanent_team_modifiers(
                binding,
                1,
                curio_s01::damage_modifiers(binding.rule().get(), scalar(ratio), &[])?,
            )?);
        }
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, PROTECTION_EFFECT)? {
        output.push(entry_protection(binding, contribution)?);
    }
    Ok(output)
}

fn entry_protection(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let allies = local(LOCAL_SELECTOR_BASE, raw, 1)?;
    let attacked = local(LOCAL_SELECTOR_BASE, raw, 2)?;
    let protection = local(LOCAL_EFFECT_BASE, raw, 1)?;
    let resistance = local(LOCAL_EFFECT_BASE, raw, 2)?;
    let apply = local(LOCAL_PROGRAM_BASE, raw, 1)?;
    let remove = local(LOCAL_PROGRAM_BASE, raw, 2)?;
    let group = local(LOCAL_GROUP_BASE, raw, 1)?;
    let resistance_group = local(LOCAL_GROUP_BASE, raw, 2)?;
    let mut modifiers = Vec::new();
    for (index, purpose) in [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
        FormulaPurpose::TrueDamage,
    ]
    .into_iter()
    .enumerate()
    {
        modifiers.push(ModifierDefinition {
            id: local(
                LOCAL_MODIFIER_BASE,
                raw,
                u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )?,
            stat: StatKind::Hp,
            stage: FormulaStage::Mitigation,
            purpose,
            value: scalar(1_000_000),
            stacking_group: group,
            priority: 0,
            floor: Some(Scalar::ZERO),
            cap: Some(Scalar::ONE),
            cap_stage: FormulaStage::Mitigation,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    let resistance_modifier = local(LOCAL_MODIFIER_BASE, raw, 8)?;
    modifiers.push(ModifierDefinition {
        id: resistance_modifier,
        stat: StatKind::EffectResistance,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::EffectChance,
        value: scalar(1_000_000),
        stacking_group: resistance_group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    });
    let protection_modifiers = modifiers[..7]
        .iter()
        .map(|modifier| modifier.id)
        .collect::<Vec<_>>();
    let duration = u16::try_from(whole(super::parameter(
        contribution.state().parameters(),
        1,
    )?)?)
    .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let effects = vec![
        EffectDefinition::new(protection, Vec::new(), protection_modifiers)
            .with_runtime_template(runtime(None, DurationClock::Permanent)?),
        EffectDefinition::new(resistance, Vec::new(), vec![resistance_modifier])
            .with_runtime_template(runtime(Some(duration), DurationClock::TargetTurnEnd)?),
    ];
    let apply_definition = ProgramDefinition::new(
        apply,
        Vec::new(),
        vec![allies],
        vec![protection, resistance],
        Vec::new(),
    )
    .with_steps(vec![
        apply_effect(allies, protection),
        apply_effect(allies, resistance),
    ]);
    let remove_definition = ProgramDefinition::new(
        remove,
        Vec::new(),
        vec![attacked],
        vec![protection],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveEffect {
            selector: attacked,
            effect: protection,
        },
    )]);
    let triggers = vec![
        TriggerDef {
            id: local(LOCAL_TRIGGER_BASE, raw, 1)?,
            event: RuleEventPoint::BattleStarted.kind(),
            event_point: RuleEventPoint::BattleStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Battle,
            priority: ReactionPriority::new(0),
            program: apply,
        },
        TriggerDef {
            id: local(LOCAL_TRIGGER_BASE, raw, 2)?,
            event: RuleEventPoint::DamageApplied.kind(),
            event_point: RuleEventPoint::DamageApplied,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                target_selector: Some(attacked),
                damage_class: Some(RuleDamageClass::Ordinary),
                has_action: Some(true),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::TargetWithinAction,
            priority: ReactionPriority::new(0),
            program: remove,
        },
    ];
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        vec![
            ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            },
            ModifierStackingGroup {
                id: resistance_group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            },
        ],
        modifiers,
        vec![
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(attacked)
                .with_rule_units(event_target_with_effect(protection)?),
        ],
        effects,
        vec![apply_definition, remove_definition],
        triggers,
        Vec::new(),
    ))
}

fn apply_effect(
    selector: starclock_combat::SelectorId,
    effect: starclock_combat::EffectDefinitionId,
) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn event_target_with_effect(
    effect: starclock_combat::EffectDefinitionId,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        1,
        16,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .map(|selector| selector.with_predicates(vec![RuleSelectorPredicate::HasEffect(effect)]))
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn runtime(
    duration: Option<u16>,
    clock: DurationClock,
) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        duration.map(|value| ValueExpr::Literal(RuleValue::Integer(i64::from(value)))),
        clock,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
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

fn whole(value: i64) -> Result<i64, BattleRuleLoweringError> {
    if value <= 0 || value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    Ok(value / 1_000_000)
}

fn local<T: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<T, BattleRuleLoweringError> {
    base.checked_add((raw & 0xffff).saturating_mul(16))
        .and_then(|value| value.checked_add(offset))
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}
