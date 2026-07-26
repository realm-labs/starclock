use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const HOURGLASS: &str = "StageAbility_612642";
const PAINTED_ALBATROSS: &str = "StageAbility_612643";
const TWELVE_MONKEYS: &str = "StageAbility_612644";
const AIDEN: &str = "StageAbility_612645";
const MILITARY_RULE: &str = "StageAbility_612646";
const EXEMPLARY_CONDUCT: &str = "StageAbility_612650";
const ULTIMATE_AS_FOLLOW_UP: &str = "StageAbility_612632";

const LOCAL_MODIFIER_BASE: u32 = 0x79e7_0000;
const LOCAL_MODIFIER_SECOND_BASE: u32 = 0x79ec_0000;
const LOCAL_GROUP_BASE: u32 = 0x79e8_0000;
const LOCAL_PROGRAM_BASE: u32 = 0x79e9_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x79ea_0000;
const HOURGLASS_EFFECT_BASE: u32 = 0x79eb_0000;

const ELEMENTS: [CombatElement; 7] = [
    CombatElement::Physical,
    CombatElement::Fire,
    CombatElement::Ice,
    CombatElement::Lightning,
    CombatElement::Wind,
    CombatElement::Quantum,
    CombatElement::Imaginary,
];

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let ultimate_as_follow_up =
        selected_level_parameters(blessings, ULTIMATE_AS_FOLLOW_UP).is_some();
    let mut output = Vec::new();
    for key in [
        HOURGLASS,
        PAINTED_ALBATROSS,
        TWELVE_MONKEYS,
        AIDEN,
        MILITARY_RULE,
        EXEMPLARY_CONDUCT,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            HOURGLASS => hourglass(binding, parameters)?,
            PAINTED_ALBATROSS => painted_albatross(binding, parameters, ultimate_as_follow_up)?,
            TWELVE_MONKEYS => twelve_monkeys(binding, parameters, ultimate_as_follow_up)?,
            AIDEN => aiden(binding, parameters, ultimate_as_follow_up)?,
            MILITARY_RULE => military_rule(binding, parameters, ultimate_as_follow_up)?,
            EXEMPLARY_CONDUCT => exemplary_conduct(
                catalog,
                blessings,
                binding,
                parameters,
                ultimate_as_follow_up,
            )?,
            _ => unreachable!("closed Elation S02 binding set"),
        });
    }
    Ok(output)
}

fn hourglass(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let group =
        ModifierStackingGroupId::new(LOCAL_GROUP_BASE + 1).expect("reserved Elation S02 group ID");
    let modifier = ModifierDefinitionId::new(LOCAL_MODIFIER_BASE + 1)
        .expect("reserved Elation S02 modifier ID");
    let reduction = parameter(parameters, 0)?
        .checked_neg()
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let mut effects = Vec::with_capacity(ELEMENTS.len());
    let mut programs = Vec::with_capacity(ELEMENTS.len());
    let mut triggers = Vec::with_capacity(ELEMENTS.len());
    for (index, element) in ELEMENTS.into_iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
        let effect = EffectDefinitionId::new(HOURGLASS_EFFECT_BASE + index + 1)
            .expect("reserved Hourglass effect ID");
        let program =
            ProgramId::new(LOCAL_PROGRAM_BASE + index + 1).expect("reserved Hourglass program ID");
        let trigger_id =
            TriggerId::new(LOCAL_TRIGGER_BASE + index + 1).expect("reserved Hourglass trigger ID");
        effects.push(
            EffectDefinition::new(effect, Vec::new(), vec![modifier]).with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::Debuff,
                    DispelCategory::DispellableDebuff,
                    1,
                    Some(integer(1)),
                    DurationClock::TargetActionEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Refresh,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ),
        );
        programs.push(
            ProgramDefinition::new(program, Vec::new(), vec![target], vec![effect], Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::ApplyEffect {
                        selector: target,
                        effect,
                        stacks: integer(1),
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    },
                )]),
        );
        triggers.push(trigger(
            trigger_id,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                element: Some(element),
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Elation),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        ));
    }
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::Sum,
                comparator: None,
            }],
            modifiers: vec![ModifierDefinition {
                id: modifier,
                stat: StatKind::Atk,
                stage: FormulaStage::PercentOfBase,
                purpose: FormulaPurpose::Stat,
                value: scalar(reduction),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::PercentOfBase,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            }],
            selectors: vec![
                SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
            ],
            effects,
            programs,
            triggers,
            ..RuleParts::default()
        },
    )
}

fn painted_albatross(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let targets = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 0)?),
    );
    let programs = vec![
        ProgramDefinition::new(
            root,
            vec![body],
            vec![owner, targets],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: targets,
            body,
            maximum: 16,
        }]),
        ProgramDefinition::new(body, Vec::new(), vec![targets], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::DamageFromEventElement {
                    selector: targets,
                    amount,
                    class: DamageClass::Additional,
                    can_crit: false,
                    can_defeat: true,
                },
            )],
        ),
    ];
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(targets).with_rule_units(event_targets_selector()?),
            ],
            programs,
            triggers: follow_up_triggers(
                raw,
                RuleEventPoint::ActionResolved,
                OnceScope::Action,
                owner,
                root,
                ultimate_as_follow_up,
            )?,
            ..RuleParts::default()
        },
    )
}

fn twelve_monkeys(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let stack_slot = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let mut tags = vec![AbilityTag::FollowUp, AbilityTag::Counter];
    if ultimate_as_follow_up {
        tags.push(AbilityTag::Ultimate);
    }
    let modifiers = tags
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            Ok(ModifierDefinition {
                id: indexed_modifier(raw, index)?,
                stat: StatKind::Atk,
                stage: FormulaStage::DamageBoost,
                purpose: FormulaPurpose::OrdinaryDamage,
                value: multiply(
                    ValueExpr::Convert {
                        value: Box::new(ValueExpr::Slot(stack_slot)),
                        target: RuleValueKind::Scalar,
                        rounding: Rounding::NearestTiesEven,
                    },
                    scalar(parameter(parameters, 0)?),
                ),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: Some(stack_slot),
                filters: vec![
                    ModifierFilter::FormulaSubject(FormulaSubject::Source),
                    ModifierFilter::AbilityTag(tag_name(*tag).into()),
                ]
                .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        64,
        Some(integer(1)),
        DurationClock::ActionEnd,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let apply = ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::ApplyEffect {
                selector: owner,
                effect,
                stacks: integer(1),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            },
        )]);
    finish(
        binding,
        RuleAttachment::EveryPlayer,
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
            programs: vec![apply],
            triggers: follow_up_triggers(
                raw,
                RuleEventPoint::HitStarted,
                OnceScope::Hit,
                owner,
                program,
                ultimate_as_follow_up,
            )?,
            ..RuleParts::default()
        },
    )
}

fn aiden(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let applied_target = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let effect_delay = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let chance = parameter(parameters, 1)?;
    let mode = whole(parameter(parameters, 3)?)?;
    if !matches!((chance, mode), (0, 1) | (100_000, 2)) {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let mut effects = Vec::new();
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut body_steps = vec![ProgramStep::Operation(RuleOperationTemplate::DelayAction {
        selector: current,
        amount: scalar(parameter(parameters, 0)?),
    })];
    let mut programs = Vec::new();
    let mut triggers = follow_up_triggers(
        raw,
        RuleEventPoint::ActionResolved,
        OnceScope::Action,
        owner,
        root,
        ultimate_as_follow_up,
    )?;
    if chance != 0 {
        let duration = whole(parameter(parameters, 2)?)?;
        let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
        let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
        groups.push(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(ModifierDefinition {
            id: modifier,
            stat: StatKind::Spd,
            stage: FormulaStage::PercentOfBase,
            purpose: FormulaPurpose::Stat,
            value: scalar(
                parameter(parameters, 4)?
                    .checked_neg()
                    .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            ),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::PercentOfBase,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
        effects.push(
            EffectDefinition::new(effect, Vec::new(), vec![modifier]).with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::Control,
                    DispelCategory::CleanseableControl,
                    1,
                    Some(integer(i64::from(duration))),
                    DurationClock::TargetTurnEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Refresh,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ),
        );
        body_steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: current,
            effect,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Resistible,
            base_chance: Some(scalar(chance)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        }));
        programs.push(
            ProgramDefinition::new(
                effect_delay,
                Vec::new(),
                vec![applied_target],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DelayAction {
                    selector: applied_target,
                    amount: scalar(parameter(parameters, 5)?),
                },
            )]),
        );
        triggers.push(trigger(
            id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectApplied,
            OnceScope::Event,
            EventFilter {
                effect_definition: Some(effect),
                source: Some(binding.source().definition()),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            effect_delay,
        ));
    }
    programs.push(
        ProgramDefinition::new(
            root,
            vec![body],
            vec![owner, target],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: target,
            body,
            maximum: 16,
        }]),
    );
    programs.push(
        ProgramDefinition::new(
            body,
            Vec::new(),
            vec![current],
            effects.iter().map(EffectDefinition::id).collect(),
            Vec::new(),
        )
        .with_steps(body_steps),
    );
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            groups,
            modifiers,
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(target)
                    .with_rule_units(randomized_event_targets_selector()?),
                SelectorDefinition::new(current).with_rule_units(current_subject_selector()?),
                SelectorDefinition::new(applied_target).with_rule_units(primary_target_selector()?),
            ],
            effects,
            programs,
            triggers,
            ..RuleParts::default()
        },
    )
}

fn military_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let chance_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let resource_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let marker = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let chance = parameter(parameters, 0)?;
    let chance_policy = if chance == 1_000_000 {
        RuleEffectChancePolicy::Guaranteed
    } else {
        RuleEffectChancePolicy::Fixed
    };
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        Some(integer(1)),
        DurationClock::ActionEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let chance_definition = ProgramDefinition::new(
        chance_program,
        Vec::new(),
        vec![owner],
        vec![marker],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: marker,
            stacks: integer(1),
            chance: chance_policy,
            base_chance: (chance_policy == RuleEffectChancePolicy::Fixed).then(|| scalar(chance)),
            rng_purpose: (chance_policy == RuleEffectChancePolicy::Fixed)
                .then_some(DrawPurpose::EFFECT_CHANCE),
        },
    )]);
    let resource_definition = ProgramDefinition::new(
        resource_program,
        Vec::new(),
        vec![owner],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ModifyResource {
            selector: owner,
            resource: RuleResourceKind::SkillPoints,
            update: ResourceUpdateKind::Gain,
            amount: scalar(1_000_000),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        },
    )]);
    let mut triggers = follow_up_triggers(
        raw,
        RuleEventPoint::ActionResolved,
        OnceScope::Action,
        owner,
        chance_program,
        ultimate_as_follow_up,
    )?;
    triggers.push(trigger(
        id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::EffectApplied,
        OnceScope::Event,
        EventFilter {
            target_selector: Some(owner),
            effect_definition: Some(marker),
            source: Some(binding.source().definition()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        resource_program,
    ));
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(marker, Vec::new(), Vec::new())
                    .with_runtime_template(runtime),
            ],
            programs: vec![chance_definition, resource_definition],
            triggers,
            ..RuleParts::default()
        },
    )
}

fn exemplary_conduct(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let cap = whole(parameter(parameters, 1)?)?;
    let count = i64::from(elation_blessing_count(catalog, blessings)?.min(cap));
    let value = parameter(parameters, 0)?
        .checked_mul(count)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let raw = binding.rule().get();
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let mut tags = vec![AbilityTag::FollowUp, AbilityTag::Counter];
    if ultimate_as_follow_up {
        tags.push(AbilityTag::Ultimate);
    }
    let modifiers = tags
        .iter()
        .enumerate()
        .map(|(index, tag)| {
            Ok(ModifierDefinition {
                id: indexed_modifier(raw, index)?,
                stat: StatKind::Atk,
                stage: FormulaStage::DamageBoost,
                purpose: FormulaPurpose::OrdinaryDamage,
                value: scalar(value),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: vec![
                    ModifierFilter::FormulaSubject(FormulaSubject::Source),
                    ModifierFilter::AbilityTag(tag_name(*tag).into()),
                ]
                .into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers,
            ..RuleParts::default()
        },
    )
}

fn elation_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.elation")
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    u16::try_from(
        blessings
            .entries()
            .iter()
            .filter(|entry| entry.path() == path.id())
            .count(),
    )
    .map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

fn follow_up_triggers(
    raw: u32,
    point: RuleEventPoint,
    scope: OnceScope,
    owner: SelectorId,
    program: ProgramId,
    ultimate_as_follow_up: bool,
) -> Result<Vec<TriggerDef>, BattleRuleLoweringError> {
    let mut tags = vec![AbilityTag::FollowUp, AbilityTag::Counter];
    if ultimate_as_follow_up {
        tags.push(AbilityTag::Ultimate);
    }
    tags.into_iter()
        .zip([
            TRIGGER_ID_BASE,
            SECOND_TRIGGER_ID_BASE,
            THIRD_TRIGGER_ID_BASE,
        ])
        .map(|(tag, base)| {
            Ok(trigger(
                id::<TriggerId>(base, raw)?,
                point,
                scope,
                EventFilter {
                    actor_selector: Some(owner),
                    ability_tag: Some(tag),
                    excluded_source: Some(resonance_source()),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            ))
        })
        .collect()
}

fn event_targets_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
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
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn randomized_event_targets_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        1,
        16,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("damage-target".into()),
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn tag_name(tag: AbilityTag) -> &'static str {
    match tag {
        AbilityTag::FollowUp => "follow_up",
        AbilityTag::Counter => "counter",
        AbilityTag::Ultimate => "ultimate",
        _ => unreachable!("closed follow-up tag set"),
    }
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

fn indexed_modifier(
    raw: u32,
    index: usize,
) -> Result<ModifierDefinitionId, BattleRuleLoweringError> {
    let base = [
        MODIFIER_ID_BASE,
        LOCAL_MODIFIER_BASE,
        LOCAL_MODIFIER_SECOND_BASE,
    ]
    .get(index)
    .copied()
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    id(base, raw)
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
    attachment: RuleAttachment,
    mut parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    parts.groups.sort_unstable_by_key(|group| group.id);
    parts.modifiers.sort_unstable_by_key(|modifier| modifier.id);
    parts.selectors.sort_unstable_by_key(SelectorDefinition::id);
    parts.effects.sort_unstable_by_key(EffectDefinition::id);
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
    parts.slots.sort_unstable_by_key(StateSlotDef::id);
    parts.triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = parts.selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = parts.programs.iter().map(ProgramDefinition::id).collect();
    Ok(ExecutableBattleRule {
        attachment,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), parts.slots, parts.triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
