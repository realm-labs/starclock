use super::*;

const ULTIMATE_CRITICAL_RATE: &str = "StageAbility_612851";
const ULTIMATE_CRITICAL_DAMAGE: &str = "StageAbility_612852";
const ULTIMATE_NEXT_ATTACK: &str = "StageAbility_612853";
const AOE_ATTACK: &str = "StageAbility_612854";
const AOE_DEFENSE: &str = "StageAbility_612855";

const LOCAL_PROGRAM_BASE: u32 = 0x7e70_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7e80_0000;
const LOCAL_EFFECT_BASE: u32 = 0x7e90_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7ea0_0000;
const LOCAL_GROUP_BASE: u32 = 0x7eb0_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7ec0_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        ULTIMATE_CRITICAL_RATE,
        ULTIMATE_CRITICAL_DAMAGE,
        ULTIMATE_NEXT_ATTACK,
        AOE_ATTACK,
        AOE_DEFENSE,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            ULTIMATE_CRITICAL_RATE => ultimate_critical(binding, parameters, StatKind::CritRate)?,
            ULTIMATE_CRITICAL_DAMAGE => {
                ultimate_critical(binding, parameters, StatKind::CritDamage)?
            }
            ULTIMATE_NEXT_ATTACK => ultimate_next_attack(binding, parameters)?,
            AOE_ATTACK => aoe_timed_stat(binding, parameters, StatKind::Atk)?,
            AOE_DEFENSE => aoe_timed_stat(binding, parameters, StatKind::Def)?,
            _ => unreachable!("closed Erudition S03 binding set"),
        });
    }
    Ok(output)
}

fn ultimate_critical(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    stat: StatKind,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    Ok(finish(
        binding,
        Vec::new(),
        Vec::new(),
        vec![unique_group(group)],
        vec![ModifierDefinition {
            id: modifier,
            stat,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: scalar(parameter_six(parameters, 0)?),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: ultimate_source_filters(),
        }],
        Vec::new(),
        Vec::new(),
    ))
}

fn ultimate_next_attack(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let consume = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let arm = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let consume_program =
        ProgramDefinition::new(consume, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                },
            )]);
    let arm_program =
        ProgramDefinition::new(arm, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![apply_effect(owner, effect)]);
    let mut consume_trigger = trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(AbilityTag::Attack),
            ..EventFilter::default()
        },
        ConditionExpr::EffectExists {
            selector: owner,
            effect,
        },
        consume,
    );
    // An attacking Ultimate consumes any older charge before arming a new one.
    consume_trigger.priority = ReactionPriority::new(-10);
    let arm_trigger = trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
        EventFilter {
            actor_selector: Some(owner),
            action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        arm,
    );
    Ok(finish(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(permanent_marker()?),
        ],
        vec![unique_group(group)],
        vec![ModifierDefinition {
            id: modifier,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose: FormulaPurpose::OrdinaryDamage,
            value: scalar(parameter_six(parameters, 0)?),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: attack_source_filters(),
        }],
        vec![consume_program, arm_program],
        vec![consume_trigger, arm_trigger],
    ))
}

fn aoe_timed_stat(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    stat: StatKind,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let apply = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    // This effect is installed at ActionResolved and the owning normal turn
    // ticks immediately afterwards. Retain one extra internal tick so the
    // released number of future owner turns remains effective.
    let turns = whole(parameter_six(parameters, 1)?)?
        .checked_add(1)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let program = ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
        .with_steps(vec![apply_effect(owner, effect)]);
    Ok(finish(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(timed_stat_buff(turns)?),
        ],
        vec![unique_group(group)],
        vec![ModifierDefinition {
            id: modifier,
            stat,
            stage: FormulaStage::PercentOfBase,
            purpose: FormulaPurpose::Stat,
            value: scalar(parameter_six(parameters, 0)?),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::PercentOfBase,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        }],
        vec![program],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            EventFilter {
                actor_selector: Some(owner),
                ability_tag: Some(AbilityTag::Attack),
                target_pattern: Some(TargetPattern::All),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            apply,
        )],
    ))
}

fn finish(
    binding: &UniverseBattleRuleBinding,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    groups: Vec<ModifierStackingGroup>,
    modifiers: Vec<ModifierDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        groups,
        modifiers,
        selectors,
        effects,
        programs,
        triggers,
        Vec::new(),
    )
}

fn trigger(
    id: TriggerId,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: RuleEventPoint::ActionResolved.kind(),
        event_point: RuleEventPoint::ActionResolved,
        phase: TriggerPhase::AfterAction,
        filter,
        condition,
        once_scope: OnceScope::Action,
        priority: ReactionPriority::new(0),
        program,
    }
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

fn timed_stat_buff(turns: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
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

fn unique_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    }
}

fn ultimate_source_filters() -> Box<[ModifierFilter]> {
    source_filters("ultimate")
}

fn attack_source_filters() -> Box<[ModifierFilter]> {
    source_filters("attack")
}

fn source_filters(tag: &str) -> Box<[ModifierFilter]> {
    vec![
        ModifierFilter::FormulaSubject(FormulaSubject::Source),
        ModifierFilter::AbilityTag(tag.into()),
    ]
    .into_boxed_slice()
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
