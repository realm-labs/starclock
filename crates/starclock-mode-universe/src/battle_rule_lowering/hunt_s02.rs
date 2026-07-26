use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const ADEPTS_BOW: &str = "StageAbility_61244201";
const ADEPTS_BOW_ENHANCED: &str = "StageAbility_61244202";
const MISTWRAITH: &str = "StageAbility_61244301";
const MISTWRAITH_ENHANCED: &str = "StageAbility_61244302";
const STARLIT_HUNT: &str = "StageAbility_61244401";
const STARLIT_HUNT_ENHANCED: &str = "StageAbility_61244402";
const BORISIN_CHASE: &str = "StageAbility_61244501";
const BORISIN_CHASE_ENHANCED: &str = "StageAbility_61244502";
const RAINBOW_FANG: &str = "StageAbility_61244601";
const RAINBOW_FANG_ENHANCED: &str = "StageAbility_61244602";
const VERMEIL_BOW: &str = "StageAbility_61245001";

const MISTWRAITH_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7950_0001).expect("reserved effect ID");
const MISTWRAITH_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x7950_0002).expect("reserved slot ID");
const MISTWRAITH_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7950_0003).expect("reserved modifier ID");
const MISTWRAITH_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7950_0004).expect("reserved group ID");
const SKILL_POINT_MARKER: EffectDefinitionId =
    EffectDefinitionId::new(0x7950_0005).expect("reserved effect ID");

const ACTOR_SELECTOR_ID_BASE: u32 = 0x7951_0000;
const ALLIES_SELECTOR_ID_BASE: u32 = 0x7952_0000;
const LAST_ACTOR_SLOT_ID_BASE: u32 = 0x7953_0000;
const SECOND_SLOT_ID_BASE: u32 = 0x7954_0000;
const GAIN_PROGRAM_ID_BASE: u32 = 0x7957_0000;

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        ADEPTS_BOW,
        ADEPTS_BOW_ENHANCED,
        MISTWRAITH,
        MISTWRAITH_ENHANCED,
        STARLIT_HUNT,
        STARLIT_HUNT_ENHANCED,
        BORISIN_CHASE,
        BORISIN_CHASE_ENHANCED,
        RAINBOW_FANG,
        RAINBOW_FANG_ENHANCED,
        VERMEIL_BOW,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            ADEPTS_BOW | ADEPTS_BOW_ENHANCED => adepts_bow(binding, parameters, key)?,
            MISTWRAITH | MISTWRAITH_ENHANCED => mistwraith(binding, parameters, key)?,
            STARLIT_HUNT | STARLIT_HUNT_ENHANCED => starlit_hunt(binding, parameters)?,
            BORISIN_CHASE | BORISIN_CHASE_ENHANCED => borisin_chase(binding, parameters)?,
            RAINBOW_FANG | RAINBOW_FANG_ENHANCED => rainbow_fang(binding, parameters)?,
            VERMEIL_BOW => vermeil_bow(catalog, binding, blessings, parameters)?,
            _ => unreachable!("closed Hunt S02 binding set"),
        });
    }
    Ok(output)
}

fn adepts_bow(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    key: &str,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    if whole(parameter(parameters, 0)?)? != 1
        || whole(parameter(parameters, 1)?)? != 8
        || parameter(parameters, 2)? != 60_000
        || parameter(parameters, 3)? != 120_000
    {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let raw = binding.rule().get();
    let actor = id::<SelectorId>(ACTOR_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALLIES_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let stacks = ValueExpr::SelectorSum {
        selector: allies,
        value: Box::new(ValueExpr::QueryEffectStacks {
            subject: StatQuerySubject::CurrentTarget,
            effect: hunt_s01::CRITICAL_BOOST,
        }),
    };
    let inherited = ValueExpr::Add(
        Box::new(stacks.clone()),
        Box::new(ValueExpr::Literal(RuleValue::Integer(1))),
    );
    let body = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![actor, allies],
        vec![hunt_s01::CRITICAL_BOOST],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: allies,
            effect: hunt_s01::CRITICAL_BOOST,
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: actor,
            effect: hunt_s01::CRITICAL_BOOST,
            stacks: inherited,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
    ]);
    let positive = ConditionExpr::Compare {
        lhs: Box::new(stacks),
        operator: Comparison::Greater,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
    };
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::ActionStarted,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(allies),
            action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
            ..EventFilter::default()
        },
        positive.clone(),
        program,
    )];
    if key == ADEPTS_BOW_ENHANCED {
        triggers.push(trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::ActionStarted,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(allies),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::FollowUp),
                ..EventFilter::default()
            },
            positive,
            program,
        ));
    }
    finish_attached(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(actor).with_rule_units(event_actor_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            programs: vec![body],
            triggers,
            ..RuleParts::default()
        },
    )
}

fn mistwraith(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    key: &str,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let attack = parameter(parameters, 0)?;
    let maximum = whole(parameter(parameters, 1)?)?;
    if attack != 400_000 || maximum != 2 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let enhanced = key == MISTWRAITH_ENHANCED;
    let raw = binding.rule().get();
    let actor = id::<SelectorId>(ACTOR_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALLIES_SELECTOR_ID_BASE, raw)?;
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let clear = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let remember = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let gain = id::<ProgramId>(GAIN_PROGRAM_ID_BASE, raw)?;
    let last_actor = id::<StateSlotDefinitionId>(LAST_ACTOR_SLOT_ID_BASE, raw)?;

    let mut effects = vec![mistwraith_effect(attack, u16::try_from(maximum).unwrap())?];
    let mut applied_effects = vec![MISTWRAITH_EFFECT];
    let mut apply_steps = vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: actor,
        effect: MISTWRAITH_EFFECT,
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })];
    let mut programs = Vec::new();
    let mut triggers = Vec::new();
    if enhanced {
        if parameter(parameters, 2)? != 500_000 {
            return Err(BattleRuleLoweringError::InvalidParameter);
        }
        effects.push(skill_point_marker()?);
        applied_effects.push(SKILL_POINT_MARKER);
        apply_steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: actor,
            effect: SKILL_POINT_MARKER,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Fixed,
            base_chance: Some(scalar(parameter(parameters, 2)?)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        }));
        programs.push(
            ProgramDefinition::new(gain, Vec::new(), vec![actor], Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::ModifyResource {
                        selector: actor,
                        resource: RuleResourceKind::SkillPoints,
                        update: ResourceUpdateKind::Gain,
                        amount: scalar(1_000_000),
                        scales_with_regeneration: false,
                        rounding: Rounding::Floor,
                    },
                )]),
        );
        triggers.push(trigger(
            id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectApplied,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(allies),
                effect_definition: Some(SKILL_POINT_MARKER),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            gain,
        ));
    }
    programs.extend([
        ProgramDefinition::new(apply, Vec::new(), vec![actor], applied_effects, Vec::new())
            .with_steps(apply_steps),
        ProgramDefinition::new(clear, Vec::new(), vec![allies], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: allies,
                    effect: MISTWRAITH_EFFECT,
                },
            )],
        ),
        ProgramDefinition::new(remember, Vec::new(), Vec::new(), Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::SetSlot {
                    slot: last_actor,
                    value: ValueExpr::EventActor,
                },
            )]),
    ]);
    let same_actor = optional_id_slot_equals(last_actor, ValueExpr::EventActor);
    triggers.extend([
        priority_trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            EventFilter {
                actor_selector: Some(allies),
                ..EventFilter::default()
            },
            same_actor.clone(),
            apply,
            -50,
        ),
        priority_trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            EventFilter {
                actor_selector: Some(allies),
                ..EventFilter::default()
            },
            ConditionExpr::Not(Box::new(same_actor)),
            clear,
            -100,
        ),
        priority_trigger(
            id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            EventFilter {
                actor_selector: Some(allies),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            remember,
            100,
        ),
    ]);
    finish_attached(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: MISTWRAITH_GROUP,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers: vec![ModifierDefinition {
                id: MISTWRAITH_MODIFIER,
                stat: StatKind::Atk,
                stage: FormulaStage::PercentOfBase,
                purpose: FormulaPurpose::Stat,
                value: multiply(
                    ValueExpr::Convert {
                        value: Box::new(ValueExpr::Slot(MISTWRAITH_STACK_SLOT)),
                        target: RuleValueKind::Scalar,
                        rounding: Rounding::NearestTiesEven,
                    },
                    scalar(attack),
                ),
                stacking_group: MISTWRAITH_GROUP,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::PercentOfBase,
                snapshot: SnapshotPolicy::RecomputeOnStackChange,
                source_stack_slot: Some(MISTWRAITH_STACK_SLOT),
                filters: Box::new([]),
            }],
            selectors: vec![
                SelectorDefinition::new(actor).with_rule_units(event_actor_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            effects,
            programs,
            slots: vec![StateSlotDef::new(
                last_actor,
                RuleValueKind::OptionalStableId,
                BattleRuleScope::Battle,
                RuleValue::OptionalStableId(None),
            )],
            triggers,
        },
    )
}

fn starlit_hunt(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let ratio = parameter(parameters, 0)?;
    let amount = multiply(
        ValueExpr::QueryMaximumEnergy(StatQuerySubject::Owner),
        scalar(ratio),
    );
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ModifyResource {
                            selector: owner,
                            resource: RuleResourceKind::Energy,
                            update: ResourceUpdateKind::Gain,
                            amount,
                            scales_with_regeneration: false,
                            rounding: Rounding::NearestTiesEven,
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::UnitDefeated,
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

fn borisin_chase(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let interval = whole(parameter(parameters, 0)?)?;
    let initial = parameters
        .get(1)
        .map(|_| parameter(parameters, 1).and_then(whole))
        .transpose()?
        .unwrap_or(0);
    if interval != 6 || !matches!(initial, 0 | 5) {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let raw = binding.rule().get();
    let actor = id::<SelectorId>(ACTOR_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALLIES_SELECTOR_ID_BASE, raw)?;
    let counter = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let skip_actor = id::<StateSlotDefinitionId>(SECOND_SLOT_ID_BASE, raw)?;
    let increment = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let clear_skip = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let advance = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let incremented = ValueExpr::Minimum(
        Box::new(ValueExpr::Add(
            Box::new(ValueExpr::Slot(counter)),
            Box::new(ValueExpr::Literal(RuleValue::Integer(1))),
        )),
        Box::new(ValueExpr::Literal(RuleValue::Integer(interval))),
    );
    let same_as_skip = optional_id_slot_equals(skip_actor, ValueExpr::EventActor);
    let programs = vec![
        ProgramDefinition::new(increment, Vec::new(), Vec::new(), Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::SetSlot {
                    slot: counter,
                    value: incremented,
                },
            )]),
        ProgramDefinition::new(clear_skip, Vec::new(), Vec::new(), Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::SetSlot {
                    slot: skip_actor,
                    value: ValueExpr::Literal(RuleValue::OptionalStableId(None)),
                },
            )]),
        ProgramDefinition::new(advance, Vec::new(), vec![actor], Vec::new(), Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::AdvanceAction {
                    selector: actor,
                    amount: scalar(1_000_000),
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: counter,
                    value: ValueExpr::Literal(RuleValue::Integer(0)),
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: skip_actor,
                    value: ValueExpr::EventActor,
                }),
            ]),
    ];
    let reached_interval = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(counter)),
        operator: Comparison::GreaterOrEqual,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(interval))),
    };
    let triggers = vec![
        priority_trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            EventFilter {
                actor_selector: Some(allies),
                ..EventFilter::default()
            },
            ConditionExpr::Not(Box::new(same_as_skip.clone())),
            increment,
            0,
        ),
        priority_trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            EventFilter {
                actor_selector: Some(allies),
                ..EventFilter::default()
            },
            same_as_skip,
            clear_skip,
            10,
        ),
        trigger(
            id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnEnded,
            OnceScope::Turn,
            EventFilter {
                actor_selector: Some(allies),
                ..EventFilter::default()
            },
            reached_interval,
            advance,
        ),
    ];
    finish_attached(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(actor).with_rule_units(event_actor_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            programs,
            slots: vec![
                StateSlotDef::new(
                    counter,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(initial),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(interval)),
                StateSlotDef::new(
                    skip_actor,
                    RuleValueKind::OptionalStableId,
                    BattleRuleScope::Battle,
                    RuleValue::OptionalStableId(None),
                ),
            ],
            triggers,
            ..RuleParts::default()
        },
    )
}

fn rainbow_fang(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let defeat = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let break_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let heal = |program, ratio| {
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: owner,
                amount: multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::Owner,
                        stat: StatKind::Hp,
                        purpose: FormulaPurpose::Stat,
                    },
                    scalar(ratio),
                ),
                apply_formula_modifiers: true,
            })],
        )
    };
    let mut programs = vec![heal(defeat, parameter(parameters, 0)?)];
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::UnitDefeated,
        OnceScope::Event,
        EventFilter {
            applier_selector: Some(owner),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        defeat,
    )];
    if parameters.len() == 2 {
        programs.push(heal(break_program, parameter(parameters, 1)?));
        triggers.push(trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::WeaknessBroken,
            OnceScope::Event,
            EventFilter {
                applier_selector: Some(owner),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            break_program,
        ));
    }
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs,
            triggers,
            ..RuleParts::default()
        },
    )
}

fn vermeil_bow(
    catalog: &UniverseCatalog,
    binding: &UniverseBattleRuleBinding,
    blessings: &BlessingContributionSet,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let count = hunt_blessing_count(catalog, blessings)?;
    let cap = u16::try_from(whole(parameter(parameters, 1)?)?)
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let multiplier = i64::from(count.min(cap));
    let value = parameter(parameters, 0)?
        .checked_mul(multiplier)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    preservation_s02::persistent_modifier_rule(
        binding,
        StatKind::Spd,
        FormulaStage::PercentOfBase,
        FormulaPurpose::Stat,
        scalar(value),
        Vec::new(),
    )
}

fn mistwraith_effect(
    attack: i64,
    maximum: u16,
) -> Result<EffectDefinition, BattleRuleLoweringError> {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        maximum,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    if attack != 400_000 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    Ok(
        EffectDefinition::new(MISTWRAITH_EFFECT, Vec::new(), vec![MISTWRAITH_MODIFIER])
            .with_runtime_template(runtime),
    )
}

fn skill_point_marker() -> Result<EffectDefinition, BattleRuleLoweringError> {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(1))),
        DurationClock::ActionEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(
        EffectDefinition::new(SKILL_POINT_MARKER, Vec::new(), Vec::new())
            .with_runtime_template(runtime),
    )
}

fn hunt_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.hunt")
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

fn event_actor_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn optional_id_slot_equals(slot: StateSlotDefinitionId, value: ValueExpr) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Equal,
        rhs: Box::new(value),
    }
}

fn priority_trigger(
    id: TriggerId,
    point: RuleEventPoint,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
    priority: i16,
) -> TriggerDef {
    let mut result = trigger(id, point, OnceScope::Event, filter, condition, program);
    result.priority = ReactionPriority::new(priority);
    result
}

fn finish_attached(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let mut rule = finish(binding, parts)?;
    rule.attachment = attachment;
    Ok(rule)
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

fn whole(value: i64) -> Result<i64, BattleRuleLoweringError> {
    if value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    Ok(value / 1_000_000)
}
