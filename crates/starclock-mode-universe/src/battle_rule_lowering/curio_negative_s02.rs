//! Executable combat rules for Goal 07 negative Curio partition M12-S02.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId, ProgramId,
    Rounding, SelectorId, TriggerId,
    catalog::{
        action::AbilityTag,
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
        Comparison, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        ResourceUpdateKind, RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate,
        RuleResourceKind, RuleValue, TriggerDef, TriggerPhase, ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio::CurioStateKind,
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::curio_s01;
use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, all_ally_selector,
    all_enemy_selector, multiply, owner_selector, parameter, propagation_s01, scalar,
};

const NORMAL_CODE: &str = "49";
const ELEGANT_CODE: &str = "51";
const MYSTERIOUS_CODE: &str = "53";
const RECURSIVE_CODE: &str = "55";
const STAR_BAIT: &str = "57";
const INSECT_WEB: &str = "59";
const PROGRAM_BASE: u32 = 0xf100_0000;
const SELECTOR_BASE: u32 = 0xf200_0000;
const EFFECT_BASE: u32 = 0xf300_0000;
const TRIGGER_BASE: u32 = 0xf400_0000;
const MODIFIER_BASE: u32 = 0xf500_0000;
const GROUP_BASE: u32 = 0xf600_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some((binding, contribution)) = state_binding(bindings, curios, NORMAL_CODE)?
        && contribution.state().kind() == CurioStateKind::Repairing
    {
        output.push(normal_code(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, ELEGANT_CODE)? {
        output.push(elegant_code(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, MYSTERIOUS_CODE)? {
        output.push(mysterious_code(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, RECURSIVE_CODE)? {
        output.push(recursive_code(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, STAR_BAIT)? {
        output.push(star_bait(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, INSECT_WEB)? {
        output.push(insect_web(binding, contribution)?);
    }
    Ok(output)
}

fn normal_code(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let threshold = parameter(contribution.state().parameters(), 0)?;
    let ratio = parameter(contribution.state().parameters(), 1)?;
    let maximum_hp = ValueExpr::QueryStat {
        subject: StatQuerySubject::Owner,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
    };
    let value = ValueExpr::Choose {
        condition: Box::new(ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::QueryHp {
                subject: StatQuerySubject::Owner,
            }),
            operator: Comparison::Less,
            rhs: Box::new(multiply(maximum_hp, scalar(threshold))),
        }),
        when_true: Box::new(scalar(ratio)),
        when_false: Box::new(scalar(0)),
    };
    let modifiers = damage_modifiers(binding.rule().get(), FormulaStage::Vulnerability, value)?;
    curio_s01::permanent_team_modifiers(binding, 11, modifiers)
}

fn elegant_code(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(SELECTOR_BASE, raw, 0)?;
    let target = local::<SelectorId>(SELECTOR_BASE, raw, 1)?;
    let program = local::<ProgramId>(PROGRAM_BASE, raw, 0)?;
    let trigger = local::<TriggerId>(TRIGGER_BASE, raw, 0)?;
    let (selector, selector_definition) = match contribution.state().kind() {
        CurioStateKind::Repairing => (
            target,
            SelectorDefinition::new(target).with_rule_units(random_enemy_selector()?),
        ),
        CurioStateKind::Fixed => (
            actor,
            SelectorDefinition::new(actor).with_rule_units(owner_selector()?),
        ),
        CurioStateKind::Active => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![selector], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AdvanceAction {
                    selector,
                    amount: scalar(parameter(contribution.state().parameters(), 0)?),
                },
            )]);
    Ok(finish_event_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![selector_definition],
        vec![program_definition],
        TriggerDef {
            id: trigger,
            event: RuleEventPoint::ActionResolved.kind(),
            event_point: RuleEventPoint::ActionResolved,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(if selector == actor {
                    actor
                } else {
                    owner_for(raw)?
                }),
                ability_tag: Some(AbilityTag::Skill),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Action,
            priority: ReactionPriority::new(0),
            program,
        },
        if selector == actor {
            Vec::new()
        } else {
            vec![SelectorDefinition::new(owner_for(raw)?).with_rule_units(owner_selector()?)]
        },
        Vec::new(),
        Vec::new(),
    ))
}

fn mysterious_code(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let killer = local::<SelectorId>(SELECTOR_BASE, raw, 2)?;
    let targets = local::<SelectorId>(SELECTOR_BASE, raw, 3)?;
    let effect = local::<EffectDefinitionId>(EFFECT_BASE, raw, 0)?;
    let program = local::<ProgramId>(PROGRAM_BASE, raw, 1)?;
    let trigger = local::<TriggerId>(TRIGGER_BASE, raw, 1)?;
    let stage = match contribution.state().kind() {
        CurioStateKind::Repairing | CurioStateKind::Fixed => FormulaStage::DamageBoost,
        CurioStateKind::Active => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    let modifiers = damage_modifiers(
        raw,
        stage,
        scalar(parameter(contribution.state().parameters(), 0)?),
    )?;
    let groups = modifier_groups(&modifiers);
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let effect_definition = EffectDefinition::new(effect, Vec::new(), modifier_ids)
        .with_runtime_template(permanent_effect()?);
    let target_selector = if contribution.state().kind() == CurioStateKind::Repairing {
        all_enemy_selector()?
    } else {
        all_ally_selector()?
    };
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![killer, targets],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: targets,
            effect,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        groups,
        modifiers,
        vec![
            SelectorDefinition::new(killer).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(targets).with_rule_units(target_selector),
        ],
        vec![effect_definition],
        vec![program_definition],
        vec![TriggerDef {
            id: trigger,
            event: RuleEventPoint::UnitDefeated.kind(),
            event_point: RuleEventPoint::UnitDefeated,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(killer),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program,
        }],
        Vec::new(),
    ))
}

fn recursive_code(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(SELECTOR_BASE, raw, 4)?;
    let program = local::<ProgramId>(PROGRAM_BASE, raw, 2)?;
    let trigger = local::<TriggerId>(TRIGGER_BASE, raw, 2)?;
    let amount = scalar(parameter(contribution.state().parameters(), 0)?);
    let (tag, update, amount) = match contribution.state().kind() {
        CurioStateKind::Repairing => (
            AbilityTag::Skill,
            ResourceUpdateKind::Spend,
            ValueExpr::Minimum(
                Box::new(amount),
                Box::new(ValueExpr::ReadResource {
                    selector: actor,
                    resource: RuleResourceKind::SkillPoints,
                }),
            ),
        ),
        CurioStateKind::Fixed => (AbilityTag::Basic, ResourceUpdateKind::Gain, amount),
        CurioStateKind::Active => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![actor], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: actor,
                    resource: RuleResourceKind::SkillPoints,
                    update,
                    amount,
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
    Ok(finish_event_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![SelectorDefinition::new(actor).with_rule_units(owner_selector()?)],
        vec![program_definition],
        TriggerDef {
            id: trigger,
            event: RuleEventPoint::ActionResolved.kind(),
            event_point: RuleEventPoint::ActionResolved,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(actor),
                ability_tag: Some(tag),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Action,
            priority: ReactionPriority::new(0),
            program,
        },
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

fn star_bait(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(SELECTOR_BASE, raw, 5)?;
    let program = local::<ProgramId>(PROGRAM_BASE, raw, 3)?;
    let trigger = local::<TriggerId>(TRIGGER_BASE, raw, 3)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![actor], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AdvanceAction {
                    selector: actor,
                    amount: scalar(parameter(contribution.state().parameters(), 1)?),
                },
            )]);
    Ok(finish_event_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![SelectorDefinition::new(actor).with_rule_units(owner_selector()?)],
        vec![program_definition],
        TriggerDef {
            id: trigger,
            event: RuleEventPoint::ActionResolved.kind(),
            event_point: RuleEventPoint::ActionResolved,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(actor),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Action,
            priority: ReactionPriority::new(0),
            program,
        },
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

fn insect_web(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let highest = local::<SelectorId>(SELECTOR_BASE, raw, 6)?;
    let marked_actor = local::<SelectorId>(SELECTOR_BASE, raw, 7)?;
    let downed = local::<SelectorId>(SELECTOR_BASE, raw, 8)?;
    let recipient = local::<SelectorId>(SELECTOR_BASE, raw, 9)?;
    let effect = local::<EffectDefinitionId>(EFFECT_BASE, raw, 1)?;
    let apply = local::<ProgramId>(PROGRAM_BASE, raw, 4)?;
    let drain = local::<ProgramId>(PROGRAM_BASE, raw, 5)?;
    let transfer = local::<ProgramId>(PROGRAM_BASE, raw, 6)?;
    let battle_trigger = local::<TriggerId>(TRIGGER_BASE, raw, 4)?;
    let turn_trigger = local::<TriggerId>(TRIGGER_BASE, raw, 5)?;
    let downed_trigger = local::<TriggerId>(TRIGGER_BASE, raw, 6)?;
    let modifier = ModifierDefinition {
        id: local::<ModifierDefinitionId>(MODIFIER_BASE, raw, 8)?,
        stat: StatKind::Atk,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: scalar(parameter(contribution.state().parameters(), 0)?),
        stacking_group: local::<ModifierStackingGroupId>(GROUP_BASE, raw, 8)?,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    };
    let group = ModifierStackingGroup {
        id: modifier.stacking_group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    };
    let effect_definition = EffectDefinition::new(effect, Vec::new(), vec![modifier.id])
        .with_runtime_template(permanent_effect()?);
    let apply_program =
        ProgramDefinition::new(apply, Vec::new(), vec![highest], vec![effect], Vec::new())
            .with_steps(vec![apply_effect(highest, effect)]);
    let drain_program = ProgramDefinition::new(
        drain,
        Vec::new(),
        vec![marked_actor],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ConsumeHp {
            selector: marked_actor,
            amount: multiply(
                ValueExpr::QueryHp {
                    subject: StatQuerySubject::Actor,
                },
                scalar(parameter(contribution.state().parameters(), 1)?),
            ),
            floor: scalar(1_000_000),
        },
    )]);
    let transfer_program = ProgramDefinition::new(
        transfer,
        Vec::new(),
        vec![downed, recipient],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: downed,
            effect,
        }),
        apply_effect(recipient, effect),
    ]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        vec![group],
        vec![modifier],
        vec![
            SelectorDefinition::new(highest).with_rule_units(highest_attack_selector()?),
            SelectorDefinition::new(marked_actor)
                .with_rule_units(actor_with_effect_selector(effect)?),
            SelectorDefinition::new(downed).with_rule_units(downed_with_effect_selector(effect)?),
            SelectorDefinition::new(recipient).with_rule_units(random_ally_selector()?),
        ],
        vec![effect_definition],
        vec![apply_program, drain_program, transfer_program],
        vec![
            TriggerDef {
                id: battle_trigger,
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
                program: drain,
            },
            TriggerDef {
                id: downed_trigger,
                event: RuleEventPoint::UnitDowned.kind(),
                event_point: RuleEventPoint::UnitDowned,
                phase: TriggerPhase::AfterEvent,
                filter: EventFilter {
                    target_selector: Some(downed),
                    ..EventFilter::default()
                },
                condition: ConditionExpr::Literal(true),
                once_scope: OnceScope::Event,
                priority: ReactionPriority::new(0),
                program: transfer,
            },
        ],
        Vec::new(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn finish_event_rule(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    mut selectors: Vec<SelectorDefinition>,
    programs: Vec<ProgramDefinition>,
    trigger: TriggerDef,
    extra_selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    modifiers: Vec<ModifierDefinition>,
) -> ExecutableBattleRule {
    selectors.extend(extra_selectors);
    let groups = modifier_groups(&modifiers);
    propagation_s01::finish_rule(
        binding,
        attachment,
        groups,
        modifiers,
        selectors,
        effects,
        programs,
        vec![trigger],
        Vec::new(),
    )
}

fn apply_effect(selector: SelectorId, effect: EffectDefinitionId) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: integer(1),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn damage_modifiers(
    raw: u32,
    stage: FormulaStage,
    value: ValueExpr,
) -> Result<Vec<ModifierDefinition>, BattleRuleLoweringError> {
    [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
        FormulaPurpose::TrueDamage,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, purpose)| {
        let index = u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
        Ok(ModifierDefinition {
            id: local(MODIFIER_BASE, raw, index)?,
            stat: StatKind::Hp,
            stage,
            purpose,
            value: value.clone(),
            stacking_group: local(GROUP_BASE, raw, index)?,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        })
    })
    .collect()
}

fn modifier_groups(modifiers: &[ModifierDefinition]) -> Vec<ModifierStackingGroup> {
    modifiers
        .iter()
        .map(|modifier| ModifierStackingGroup {
            id: modifier.stacking_group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        })
        .collect()
}

fn random_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::StableId,
        1,
        1,
        RuleSelectorChoice::RngUniform,
        Some("damage-target".into()),
    )
}

fn highest_attack_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    Ok(selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::StatDescending,
        1,
        1,
        RuleSelectorChoice::First,
        None,
    )?
    .with_weight(Some(ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::Stat,
    })))
}

fn actor_with_effect_selector(
    effect: EffectDefinitionId,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    Ok(selector(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::StableId,
        1,
        1,
        RuleSelectorChoice::First,
        None,
    )?
    .with_predicates(vec![RuleSelectorPredicate::HasEffect(effect)]))
}

fn downed_with_effect_selector(
    effect: EffectDefinitionId,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    Ok(selector(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Same,
        RuleLifePredicate::Downed,
        RulePresencePredicate::Any,
        RuleSelectorOrdering::EventOrder,
        1,
        1,
        RuleSelectorChoice::First,
        None,
    )?
    .with_predicates(vec![RuleSelectorPredicate::HasEffect(effect)]))
}

fn random_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::StableId,
        0,
        1,
        RuleSelectorChoice::RngUniform,
        Some("behavior-choice".into()),
    )
}

#[allow(clippy::too_many_arguments)]
fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    ordering: RuleSelectorOrdering,
    minimum: u16,
    maximum: u16,
    choice: RuleSelectorChoice,
    rng_purpose: Option<Box<str>>,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        life,
        presence,
        RuleSelectorReference::CurrentState,
        ordering,
        minimum,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        rng_purpose,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn permanent_effect() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
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

fn owner_for(raw: u32) -> Result<SelectorId, BattleRuleLoweringError> {
    local(SELECTOR_BASE, raw, 10)
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

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn local<T: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<T, BattleRuleLoweringError> {
    base.checked_add((raw & 0xffff).saturating_mul(16))
        .and_then(|value| value.checked_add(offset))
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}
