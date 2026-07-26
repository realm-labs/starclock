use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const SPINAL_SPUR: &str = "StageAbility_612751";
const CHANNELED_NEEDLE: &str = "StageAbility_612752";
const CONJUNCTIVA: &str = "StageAbility_612753";
const SCALED_WING: &str = "StageAbility_612754";
const COMPOUND_EYE: &str = "StageAbility_612755";

const LOCAL_PROGRAM_BASE: u32 = 0x7bb0_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7bc0_0000;
const LOCAL_EFFECT_BASE: u32 = 0x7bd0_0000;
const LOCAL_SLOT_BASE: u32 = 0x7be0_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7bf0_0000;
const LOCAL_GROUP_BASE: u32 = 0x7c00_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7c10_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        SPINAL_SPUR,
        CHANNELED_NEEDLE,
        CONJUNCTIVA,
        SCALED_WING,
        COMPOUND_EYE,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            SPINAL_SPUR => {
                basic_critical_stat(binding, parameters, StatKind::CritRate, FormulaStage::Flat)?
            }
            CHANNELED_NEEDLE => basic_critical_stat(
                binding,
                parameters,
                StatKind::CritDamage,
                FormulaStage::Flat,
            )?,
            CONJUNCTIVA => basic_timed_stat(
                binding,
                parameters,
                StatKind::Def,
                FormulaStage::PercentOfBase,
            )?,
            SCALED_WING => basic_timed_stat(
                binding,
                parameters,
                StatKind::Spd,
                FormulaStage::PercentOfBase,
            )?,
            COMPOUND_EYE => compound_eye(binding, parameters)?,
            _ => unreachable!("closed Propagation S03 binding set"),
        });
    }
    Ok(output)
}

fn basic_critical_stat(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    stat: StatKind,
    stage: FormulaStage,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let install = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let value = parameter_six(parameters, 0)?;
    let definition = ModifierDefinition {
        id: modifier,
        stat,
        stage,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: scalar(value),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: vec![
            ModifierFilter::FormulaSubject(FormulaSubject::Source),
            ModifierFilter::AbilityTag("basic".into()),
        ]
        .into_boxed_slice(),
    };
    let install_definition =
        ProgramDefinition::new(install, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![apply_effect(owner, effect)]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![unique_group(group)],
        vec![definition],
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(permanent_buff()?),
        ],
        vec![install_definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::BattleStarted,
            TriggerPhase::AfterEvent,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            install,
        )],
        Vec::new(),
    ))
}

fn basic_timed_stat(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    stat: StatKind,
    stage: FormulaStage,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let apply = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let value = parameter_six(parameters, 0)?;
    let turns = whole(parameter_six(parameters, 1)?)?;
    let definition = ModifierDefinition {
        id: modifier,
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
    let apply_definition =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![apply_effect(owner, effect)]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![unique_group(group)],
        vec![definition],
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(timed_replace_buff(turns)?),
        ],
        vec![apply_definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Basic),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            apply,
        )],
        Vec::new(),
    ))
}

fn compound_eye(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let recover = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let counter = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let cap = whole(parameter_six(parameters, 0)?)?;
    let definition =
        ProgramDefinition::new(recover, Vec::new(), vec![actor], Vec::new(), Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    selector: actor,
                    resource: RuleResourceKind::SkillPoints,
                    update: ResourceUpdateKind::Gain,
                    amount: scalar(1_000_000),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                }),
                ProgramStep::Operation(RuleOperationTemplate::AddSlot {
                    slot: counter,
                    value: integer(1),
                }),
            ]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?)],
        Vec::new(),
        vec![definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::TurnEnded,
            TriggerPhase::AfterEvent,
            OnceScope::Turn,
            EventFilter {
                actor_selector: Some(actor),
                ..EventFilter::default()
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(counter)),
                operator: Comparison::Less,
                rhs: Box::new(integer(i64::from(cap))),
            },
            recover,
        )],
        vec![
            StateSlotDef::new(
                counter,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(i64::from(cap))),
        ],
    ))
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

fn unique_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    }
}

fn trigger(
    id: TriggerId,
    point: RuleEventPoint,
    phase: TriggerPhase,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn permanent_buff() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
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

fn timed_replace_buff(turns: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(integer(i64::from(turns))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(RuleSelectorOrigin::Owner, RuleSelectorSide::Same)
}

fn actor_player_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(RuleSelectorOrigin::Actor, RuleSelectorSide::Same)
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn local<I: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<I, BattleRuleLoweringError> {
    base.checked_add(
        (raw & 0xffff)
            .checked_mul(16)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    )
    .and_then(|value| value.checked_add(offset))
    .and_then(|value| I::try_from(value).ok())
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    value
        .checked_div(1_000_000)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}
