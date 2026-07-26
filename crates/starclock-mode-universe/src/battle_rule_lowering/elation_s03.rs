use super::*;

const MOSTLY_HARMFUL: &str = "StageAbility_612651";
const SUSPIRIA: &str = "StageAbility_612652";
const PALE_FIRE: &str = "StageAbility_612653";
const BACK_TO_LIGHTHOUSE: &str = "StageAbility_612654";
const DOCTOR_OF_LOVE: &str = "StageAbility_612655";
const ULTIMATE_AS_FOLLOW_UP: &str = "StageAbility_612632";

const SECOND_MODIFIER_BASE: u32 = 0x79ed_0000;
const THIRD_MODIFIER_BASE: u32 = 0x79ee_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let ultimate_as_follow_up =
        selected_level_parameters(blessings, ULTIMATE_AS_FOLLOW_UP).is_some();
    let mut output = Vec::new();
    for key in [
        MOSTLY_HARMFUL,
        SUSPIRIA,
        PALE_FIRE,
        BACK_TO_LIGHTHOUSE,
        DOCTOR_OF_LOVE,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            MOSTLY_HARMFUL => persistent_follow_up_modifier(
                binding,
                StatKind::ToughnessDamage,
                FormulaStage::Flat,
                FormulaPurpose::Break,
                parameter(parameters, 0)?,
                ultimate_as_follow_up,
            )?,
            SUSPIRIA => persistent_follow_up_modifier(
                binding,
                StatKind::Atk,
                FormulaStage::DamageBoost,
                FormulaPurpose::OrdinaryDamage,
                parameter(parameters, 0)?,
                ultimate_as_follow_up,
            )?,
            PALE_FIRE => persistent_follow_up_modifier(
                binding,
                StatKind::CritRate,
                FormulaStage::Flat,
                FormulaPurpose::Stat,
                parameter(parameters, 0)?,
                ultimate_as_follow_up,
            )?,
            BACK_TO_LIGHTHOUSE => persistent_follow_up_modifier(
                binding,
                StatKind::EnergyRegenerationRate,
                FormulaStage::Flat,
                FormulaPurpose::Stat,
                parameter(parameters, 0)?,
                ultimate_as_follow_up,
            )?,
            DOCTOR_OF_LOVE => doctor_of_love(binding, parameters, ultimate_as_follow_up)?,
            _ => unreachable!("closed Elation S03 binding set"),
        });
    }
    Ok(output)
}

fn persistent_follow_up_modifier(
    binding: &UniverseBattleRuleBinding,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: i64,
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let mut tags = vec![AbilityTag::FollowUp, AbilityTag::Counter];
    if ultimate_as_follow_up {
        tags.push(AbilityTag::Ultimate);
    }
    let modifiers = tags
        .into_iter()
        .enumerate()
        .map(|(index, tag)| {
            Ok(ModifierDefinition {
                id: indexed_modifier(raw, index)?,
                stat,
                stage,
                purpose,
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
                    ModifierFilter::AbilityTag(tag_name(tag).into()),
                ]
                .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
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
    finish(
        binding,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers,
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), modifier_ids)
                    .with_runtime_template(runtime),
            ],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ApplyEffect {
                            selector: owner,
                            effect,
                            stacks: integer(1),
                            chance: RuleEffectChancePolicy::Guaranteed,
                            base_chance: None,
                            rng_purpose: None,
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::BattleStarted,
                OnceScope::Battle,
                EventFilter::default(),
                ConditionExpr::Literal(true),
                program,
            )],
        },
    )
}

fn doctor_of_love(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Healing,
        },
        scalar(parameter(parameters, 0)?),
    );
    finish(
        binding,
        RuleParts {
            groups: Vec::new(),
            modifiers: Vec::new(),
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: Vec::new(),
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                        selector: owner,
                        amount,
                        apply_formula_modifiers: true,
                    })]),
            ],
            triggers: follow_up_triggers(
                raw,
                RuleEventPoint::ActionResolved,
                OnceScope::Action,
                owner,
                program,
                ultimate_as_follow_up,
            )?,
        },
    )
}

fn follow_up_triggers(
    raw: u32,
    point: RuleEventPoint,
    once: OnceScope,
    owner: SelectorId,
    program: ProgramId,
    ultimate_as_follow_up: bool,
) -> Result<Vec<TriggerDef>, BattleRuleLoweringError> {
    let mut tags = vec![
        (TRIGGER_ID_BASE, AbilityTag::FollowUp),
        (SECOND_TRIGGER_ID_BASE, AbilityTag::Counter),
    ];
    if ultimate_as_follow_up {
        tags.push((THIRD_TRIGGER_ID_BASE, AbilityTag::Ultimate));
    }
    tags.into_iter()
        .map(|(base, tag)| {
            Ok(trigger(
                id::<TriggerId>(base, raw)?,
                point,
                once,
                EventFilter {
                    ability_tag: Some(tag),
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            ))
        })
        .collect()
}

fn tag_name(tag: AbilityTag) -> &'static str {
    match tag {
        AbilityTag::FollowUp => "follow_up",
        AbilityTag::Counter => "counter",
        AbilityTag::Ultimate => "ultimate",
        _ => unreachable!("closed follow-up tag set"),
    }
}

fn indexed_modifier(
    raw: u32,
    index: usize,
) -> Result<ModifierDefinitionId, BattleRuleLoweringError> {
    let base = [MODIFIER_ID_BASE, SECOND_MODIFIER_BASE, THIRD_MODIFIER_BASE]
        .get(index)
        .copied()
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    id(base, raw)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

#[derive(Default)]
struct RuleParts {
    groups: Vec<ModifierStackingGroup>,
    modifiers: Vec<ModifierDefinition>,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
}

fn finish(
    binding: &UniverseBattleRuleBinding,
    mut parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    parts.groups.sort_unstable_by_key(|group| group.id);
    parts.modifiers.sort_unstable_by_key(|modifier| modifier.id);
    parts.selectors.sort_unstable_by_key(SelectorDefinition::id);
    parts.effects.sort_unstable_by_key(EffectDefinition::id);
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
    parts.triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = parts.selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = parts.programs.iter().map(ProgramDefinition::id).collect();
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), Vec::new(), parts.triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
