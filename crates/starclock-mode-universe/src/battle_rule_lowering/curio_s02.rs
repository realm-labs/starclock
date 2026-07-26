//! Executable combat rules for Goal 07 Curio partition M11-S02.

use std::collections::BTreeSet;

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectRuntimeTemplate, EffectStackPolicy,
    EffectTickPhase, Rounding,
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
        ModifierStackingGroup, SnapshotPolicy, StatKind, StatQuerySubject,
    },
    rule::model::{
        BattleRuleScope, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleValue, RuleValueKind,
        StateSlotDef, TriggerDef, TriggerPhase, ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    blessing_runtime::BlessingContributionSet,
    curio_activity::CAVITY_CRITICAL_STACK_KEY,
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, curio_s01, multiply,
    propagation_s01, scalar,
};

const CAVITY_EFFECT: &str = "85";
const HEALING_EFFECT: &str = "87";
const TOXI_EFFECT: &str = "89";
const BREAK_EFFECT: &str = "90";
const LOCAL_PROGRAM_BASE: u32 = 0x77f1_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x77f2_0000;
const LOCAL_EFFECT_BASE: u32 = 0x77f3_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x77f4_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x77f5_0000;
const LOCAL_GROUP_BASE: u32 = 0x77f6_0000;
const LOCAL_SLOT_BASE: u32 = 0x77f7_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some((binding, contribution)) = state_binding(bindings, curios, CAVITY_EFFECT)? {
        let stacks = curios.runtime_value(CAVITY_CRITICAL_STACK_KEY).unwrap_or(0);
        if stacks > 0 {
            output.push(permanent_stat(
                binding,
                contribution,
                1,
                StatKind::CritDamage,
                FormulaStage::Flat,
                super::parameter(contribution.state().parameters(), 0)?
                    .checked_mul(stacks)
                    .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            )?);
        }
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, HEALING_EFFECT)? {
        output.push(turn_healing(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, TOXI_EFFECT)? {
        output.push(toxi_flame(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, BREAK_EFFECT)? {
        let path_count = blessings
            .entries()
            .iter()
            .map(|entry| entry.path())
            .collect::<BTreeSet<_>>()
            .len();
        let path_count =
            i64::try_from(path_count).map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
        output.push(permanent_stat(
            binding,
            contribution,
            2,
            StatKind::BreakEffect,
            FormulaStage::Flat,
            super::parameter(contribution.state().parameters(), 0)?
                .checked_mul(path_count)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        )?);
    }
    Ok(output)
}

fn permanent_stat(
    binding: &UniverseBattleRuleBinding,
    _contribution: &CurioContribution,
    identity: u32,
    stat: StatKind,
    stage: FormulaStage,
    value: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let group = local(LOCAL_GROUP_BASE, raw, identity)?;
    let modifier = ModifierDefinition {
        id: local(LOCAL_MODIFIER_BASE, raw, identity)?,
        stat,
        stage,
        purpose: FormulaPurpose::Stat,
        value: scalar(value),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    };
    curio_s01::permanent_team_modifiers(binding, identity + 8, vec![modifier])
}

fn turn_healing(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local(LOCAL_SELECTOR_BASE, raw, 1)?;
    let program = local(LOCAL_PROGRAM_BASE, raw, 1)?;
    let trigger = local(LOCAL_TRIGGER_BASE, raw, 1)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Actor,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Healing,
        },
        scalar(super::parameter(contribution.state().parameters(), 0)?),
    );
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![actor], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: actor,
                amount,
                apply_formula_modifiers: true,
            })]);
    let trigger_definition = TriggerDef {
        id: trigger,
        event: RuleEventPoint::TurnStarted.kind(),
        event_point: RuleEventPoint::TurnStarted,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter {
            actor_selector: Some(actor),
            ..EventFilter::default()
        },
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Turn,
        priority: ReactionPriority::new(0),
        program,
    };
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(actor).with_rule_units(actor_selector(None)?)],
        Vec::new(),
        vec![program_definition],
        vec![trigger_definition],
        Vec::new(),
    ))
}

fn toxi_flame(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let highest = local(LOCAL_SELECTOR_BASE, raw, 2)?;
    let marked_actor = local(LOCAL_SELECTOR_BASE, raw, 3)?;
    let mark_effect = local(LOCAL_EFFECT_BASE, raw, 1)?;
    let speed_effect = local(LOCAL_EFFECT_BASE, raw, 2)?;
    let speed_slot = local(LOCAL_SLOT_BASE, raw, 1)?;
    let mark_program = local(LOCAL_PROGRAM_BASE, raw, 2)?;
    let turn_program = local(LOCAL_PROGRAM_BASE, raw, 3)?;
    let speed_modifier = local(LOCAL_MODIFIER_BASE, raw, 3)?;
    let speed_group = local(LOCAL_GROUP_BASE, raw, 3)?;
    let battle_trigger = local(LOCAL_TRIGGER_BASE, raw, 2)?;
    let turn_trigger = local(LOCAL_TRIGGER_BASE, raw, 3)?;
    let hp_ratio = super::parameter(contribution.state().parameters(), 0)?;
    let speed_ratio = super::parameter(contribution.state().parameters(), 1)?;
    let maximum_stacks = whole(super::parameter(contribution.state().parameters(), 2)?)?;

    let groups = vec![ModifierStackingGroup {
        id: speed_group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    }];
    let modifiers = vec![ModifierDefinition {
        id: speed_modifier,
        stat: StatKind::Spd,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: multiply(
            ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(speed_slot)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            },
            scalar(speed_ratio),
        ),
        stacking_group: speed_group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(speed_slot),
        filters: Box::new([]),
    }];
    let effects = vec![
        EffectDefinition::new(mark_effect, Vec::new(), Vec::new())
            .with_runtime_template(permanent_effect(1, EffectStackPolicy::Replace)?),
        EffectDefinition::new(speed_effect, Vec::new(), vec![speed_modifier])
            .with_runtime_template(permanent_effect(
                maximum_stacks,
                EffectStackPolicy::RefreshAndAddStacks,
            )?),
    ];
    let mark_program_definition = ProgramDefinition::new(
        mark_program,
        Vec::new(),
        vec![highest],
        vec![mark_effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: highest,
            effect: mark_effect,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )]);
    let turn_program_definition = ProgramDefinition::new(
        turn_program,
        Vec::new(),
        vec![marked_actor],
        vec![speed_effect],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
            selector: marked_actor,
            amount: multiply(
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::CurrentTarget,
                    stat: StatKind::Hp,
                    purpose: FormulaPurpose::Stat,
                },
                scalar(hp_ratio),
            ),
            floor: scalar(1_000_000),
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: marked_actor,
            effect: speed_effect,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
    ]);
    let triggers = vec![
        TriggerDef {
            id: battle_trigger,
            event: RuleEventPoint::BattleStarted.kind(),
            event_point: RuleEventPoint::BattleStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Battle,
            priority: ReactionPriority::new(0),
            program: mark_program,
        },
        TriggerDef {
            id: turn_trigger,
            event: RuleEventPoint::TurnStarted.kind(),
            event_point: RuleEventPoint::TurnStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(marked_actor),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Turn,
            priority: ReactionPriority::new(0),
            program: turn_program,
        },
    ];
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        groups,
        modifiers,
        vec![
            SelectorDefinition::new(highest).with_rule_units(highest_attack_selector()?),
            SelectorDefinition::new(marked_actor)
                .with_rule_units(actor_selector(Some(mark_effect))?),
        ],
        effects,
        vec![mark_program_definition, turn_program_definition],
        triggers,
        vec![
            StateSlotDef::new(
                speed_slot,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(
                RuleValue::Integer(0),
                RuleValue::Integer(i64::from(maximum_stacks)),
            ),
        ],
    ))
}

fn highest_attack_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StatDescending,
        1,
        1,
        RuleEmptyPoolPolicy::Fault,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(selector.with_weight(Some(ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::Stat,
    })))
}

fn actor_selector(
    effect: Option<starclock_combat::EffectDefinitionId>,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        1,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(match effect {
        Some(effect) => selector.with_predicates(vec![RuleSelectorPredicate::HasEffect(effect)]),
        None => selector,
    })
}

fn permanent_effect(
    maximum_stacks: u16,
    policy: EffectStackPolicy,
) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        maximum_stacks,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        policy,
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

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    if value < 0 || value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
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
