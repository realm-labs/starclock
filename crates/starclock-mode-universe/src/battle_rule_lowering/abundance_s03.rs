use super::*;
use starclock_combat::{Scalar, modifier::model::FormulaSubject, rule::model::SourceClass};

const HEALING_RECEIVED: &str = "StageAbility_612351";
const ENTRY_HEALING: &str = "StageAbility_612352";
const BREAK_HEALING: &str = "StageAbility_612353";
const HEALED_DEFENSE: &str = "StageAbility_612354";
const PROVIDER_HEALING: &str = "StageAbility_612355";

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        HEALING_RECEIVED,
        ENTRY_HEALING,
        BREAK_HEALING,
        HEALED_DEFENSE,
        PROVIDER_HEALING,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            HEALING_RECEIVED => healing_received(binding, parameters)?,
            ENTRY_HEALING => event_maximum_hp_healing(
                binding,
                parameters,
                RuleEventPoint::BattleStarted,
                OnceScope::Battle,
                EventFilter::default(),
            )?,
            BREAK_HEALING => event_maximum_hp_healing(
                binding,
                parameters,
                RuleEventPoint::WeaknessBroken,
                OnceScope::Event,
                EventFilter {
                    actor_selector: Some(id::<SelectorId>(
                        OWNER_SELECTOR_ID_BASE,
                        binding.rule().get(),
                    )?),
                    ..EventFilter::default()
                },
            )?,
            HEALED_DEFENSE => healed_defense(binding, parameters)?,
            PROVIDER_HEALING => provider_healing(binding, parameters)?,
            _ => unreachable!("closed Abundance S03 binding set"),
        });
    }
    Ok(output)
}

fn healing_received(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    persistent_modifier_rule(
        binding,
        StatKind::IncomingHealing,
        FormulaStage::Healing,
        FormulaPurpose::Healing,
        scalar(parameter(parameters, 0)?),
        vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)],
    )
}

fn event_maximum_hp_healing(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
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
                    .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                        selector: owner,
                        amount: maximum_hp_ratio(parameter(parameters, 0)?),
                        apply_formula_modifiers: true,
                    })]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                point,
                once_scope,
                filter,
                ConditionExpr::Literal(true),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn healed_defense(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let duration = whole(parameter(parameters, 1)?)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(duration)))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
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
                stat: StatKind::Def,
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
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: owner,
                            effect,
                            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::HealApplied,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(owner),
                    ..EventFilter::default()
                },
                positive_healing(),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn provider_healing(
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
                    .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                        selector: owner,
                        amount: maximum_hp_ratio(parameter(parameters, 0)?),
                        apply_formula_modifiers: true,
                    })]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::HealApplied,
                OnceScope::Action,
                EventFilter {
                    applier_selector: Some(owner),
                    source_class: Some(SourceClass::Ability),
                    has_action: Some(true),
                    ..EventFilter::default()
                },
                positive_healing(),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn maximum_hp_ratio(ratio: i64) -> ValueExpr {
    multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(ratio),
    )
}

fn positive_healing() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::HpChangeAmount,
        )),
        operator: Comparison::Greater,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
    }
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(
        value
            .checked_div(1_000_000)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
    )
    .map_err(|_| BattleRuleLoweringError::InvalidParameter)
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
