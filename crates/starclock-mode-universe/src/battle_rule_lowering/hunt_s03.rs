use super::*;

const CRITICAL_RATE: &str = "StageAbility_61245101";
const CRITICAL_DAMAGE: &str = "StageAbility_61245201";
const BREAK_DELAY: &str = "StageAbility_61245301";
const ENTRY_SPEED: &str = "StageAbility_61245401";
const TURN_END_ADVANCE: &str = "StageAbility_61245501";

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        CRITICAL_RATE,
        CRITICAL_DAMAGE,
        BREAK_DELAY,
        ENTRY_SPEED,
        TURN_END_ADVANCE,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            CRITICAL_RATE => persistent_stat(binding, parameters, StatKind::CritRate)?,
            CRITICAL_DAMAGE => persistent_stat(binding, parameters, StatKind::CritDamage)?,
            BREAK_DELAY => break_delay(binding, parameters)?,
            ENTRY_SPEED => entry_speed(binding, parameters)?,
            TURN_END_ADVANCE => turn_end_advance(binding, parameters)?,
            _ => unreachable!("closed Hunt S03 binding set"),
        });
    }
    Ok(output)
}

fn persistent_stat(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    stat: StatKind,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    preservation_s02::persistent_modifier_rule(
        binding,
        stat,
        FormulaStage::Flat,
        FormulaPurpose::Stat,
        scalar(parameter(parameters, 0)?),
        Vec::new(),
    )
}

fn break_delay(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    finish(
        binding,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
            ],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![target], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::DelayAction {
                            selector: target,
                            amount: scalar(parameter(parameters, 0)?),
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::WeaknessBroken,
                OnceScope::Event,
                EventFilter {
                    applier_selector: Some(owner),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn entry_speed(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let remove = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let apply_program =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    let remove_program =
        ProgramDefinition::new(remove, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                },
            )],
        );
    finish(
        binding,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers: vec![ModifierDefinition {
                id: modifier,
                stat: StatKind::Spd,
                stage: FormulaStage::PercentOfBase,
                purpose: FormulaPurpose::Stat,
                value: scalar(parameter(parameters, 0)?),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::PercentOfBase,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            }],
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier])
                    .with_runtime_template(runtime),
            ],
            programs: vec![apply_program, remove_program],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    ConditionExpr::Literal(true),
                    apply,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::DamageApplied,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    remove,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn turn_end_advance(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::AdvanceAction {
                            selector: owner,
                            amount: scalar(parameter(parameters, 0)?),
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

#[derive(Default)]
struct RuleParts {
    groups: Vec<ModifierStackingGroup>,
    modifiers: Vec<ModifierDefinition>,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    slots: Vec<StateSlotDef>,
    triggers: Vec<TriggerDef>,
}

fn finish(
    binding: &UniverseBattleRuleBinding,
    mut parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    parts.selectors.sort_unstable_by_key(SelectorDefinition::id);
    parts.selectors.dedup_by_key(|selector| selector.id());
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
    parts.triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = parts
        .selectors
        .iter()
        .map(SelectorDefinition::id)
        .collect::<Vec<_>>();
    let program_ids = parts
        .programs
        .iter()
        .map(ProgramDefinition::id)
        .collect::<Vec<_>>();
    let definition = RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
        BattleRuleDefinition::new(binding.source().clone(), parts.slots, parts.triggers, None),
    );
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
