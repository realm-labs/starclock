use super::*;
use starclock_combat::formula::toughness::EnemyRank;

const EMPYREAN_IMPERIUM: &str = "StageAbility_61243001";
const EMPYREAN_IMPERIUM_ENHANCED: &str = "StageAbility_61243002";
const RADIANT_SUPREME: &str = "StageAbility_61243101";
const SOVEREIGN_SKYBREAKER: &str = "StageAbility_61243201";
const SKYWARD_VENDETTA: &str = "StageAbility_61244001";
const SKYWARD_VENDETTA_ENHANCED: &str = "StageAbility_61244002";
const ARCHERY_DUEL: &str = "StageAbility_61244101";
const ARCHERY_DUEL_ENHANCED: &str = "StageAbility_61244102";

pub(super) const CRITICAL_BOOST: EffectDefinitionId =
    EffectDefinitionId::new(0x7940_0001).expect("reserved effect ID");
const CRITICAL_BOOST_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x7940_0002).expect("reserved slot ID");
const CRIT_RATE_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7940_0003).expect("reserved modifier group ID");
const CRIT_DAMAGE_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7940_0004).expect("reserved modifier group ID");
const CRIT_RATE_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7940_0005).expect("reserved modifier ID");
const CRIT_DAMAGE_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7940_0006).expect("reserved modifier ID");
const VENDETTA_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7940_0007).expect("reserved modifier group ID");
const VENDETTA_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7940_0008).expect("reserved modifier ID");

const READY_PROGRAM_ID_BASE: u32 = 0x7941_0000;
const ELITE_PROGRAM_ID_BASE: u32 = 0x7942_0000;
const READY_TRIGGER_ID_BASE: u32 = 0x7943_0000;
const CONSUME_TRIGGER_ID_BASE: u32 = 0x7944_0000;
const SECOND_EFFECT_ID_BASE: u32 = 0x7945_0000;
const SECOND_MODIFIER_ID_BASE: u32 = 0x7946_0000;
const SECOND_GROUP_ID_BASE: u32 = 0x7947_0000;
const ADVANCE_PROGRAM_ID_BASE: u32 = 0x7948_0000;
const ADVANCE_TRIGGER_ID_BASE: u32 = 0x7949_0000;
const CRITICAL_TRANSFER_PROGRAM: ProgramId =
    ProgramId::new(0x794a_0001).expect("reserved program ID");
const CRITICAL_RESET_PROGRAM: ProgramId = ProgramId::new(0x794a_0002).expect("reserved program ID");
const CRITICAL_OWNER_SELECTOR: SelectorId =
    SelectorId::new(0x794b_0001).expect("reserved selector ID");
const CRITICAL_ALLIES_SELECTOR: SelectorId =
    SelectorId::new(0x794b_0002).expect("reserved selector ID");
const CRITICAL_TRANSFER_TRIGGER: TriggerId =
    TriggerId::new(0x794c_0001).expect("reserved trigger ID");
const CRITICAL_RESET_TRIGGER: TriggerId = TriggerId::new(0x794c_0002).expect("reserved trigger ID");

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        EMPYREAN_IMPERIUM,
        EMPYREAN_IMPERIUM_ENHANCED,
        RADIANT_SUPREME,
        SOVEREIGN_SKYBREAKER,
        SKYWARD_VENDETTA,
        SKYWARD_VENDETTA_ENHANCED,
        ARCHERY_DUEL,
        ARCHERY_DUEL_ENHANCED,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            EMPYREAN_IMPERIUM | EMPYREAN_IMPERIUM_ENHANCED => {
                empyrean_imperium(binding, parameters)?
            }
            RADIANT_SUPREME => radiant_supreme(binding, parameters)?,
            SOVEREIGN_SKYBREAKER => sovereign_skybreaker(binding, parameters)?,
            SKYWARD_VENDETTA | SKYWARD_VENDETTA_ENHANCED => empty_rule(binding),
            ARCHERY_DUEL | ARCHERY_DUEL_ENHANCED => archery_duel(binding, parameters)?,
            _ => unreachable!("closed Hunt S01 binding set"),
        });
    }
    Ok(output)
}

fn empyrean_imperium(
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
            programs: vec![apply_critical_boost(
                program,
                owner,
                ValueExpr::Literal(RuleValue::Integer(whole(parameter(parameters, 2)?)?)),
            )],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::TurnStarted,
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

fn radiant_supreme(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let defeat = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let apply = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let advance = id::<ProgramId>(ADVANCE_PROGRAM_ID_BASE, raw)?;
    let pending = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let gain = whole(parameter(parameters, 2)?)?;
    if whole(parameter(parameters, 3)?)? != 8 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let defeat_program =
        ProgramDefinition::new(defeat, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::AddSlot {
                slot: pending,
                value: ValueExpr::Literal(RuleValue::Integer(gain)),
            })],
        );
    let advance_program =
        ProgramDefinition::new(advance, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AdvanceAction {
                    selector: owner,
                    amount: scalar(1_000_000),
                },
            )]);
    let apply_program = ProgramDefinition::new(
        apply,
        Vec::new(),
        vec![owner],
        vec![CRITICAL_BOOST],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: CRITICAL_BOOST,
            stacks: ValueExpr::Slot(pending),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: pending,
            value: ValueExpr::Literal(RuleValue::Integer(0)),
        }),
    ]);
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![defeat_program, apply_program, advance_program],
            slots: vec![
                StateSlotDef::new(
                    pending,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1_024)),
            ],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::UnitDefeated,
                    OnceScope::Event,
                    EventFilter {
                        applier_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    defeat,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnEnded,
                    OnceScope::Turn,
                    EventFilter {
                        owner_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    positive_integer_slot(pending),
                    advance,
                ),
                trigger(
                    id::<TriggerId>(ADVANCE_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnStarted,
                    OnceScope::Turn,
                    EventFilter {
                        owner_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    positive_integer_slot(pending),
                    apply,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn sovereign_skybreaker(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let break_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let ready_program = id::<ProgramId>(READY_PROGRAM_ID_BASE, raw)?;
    let consume_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let elite_program = id::<ProgramId>(ELITE_PROGRAM_ID_BASE, raw)?;
    let advance_program = id::<ProgramId>(ADVANCE_PROGRAM_ID_BASE, raw)?;
    let pending = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(SECOND_EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(SECOND_MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(SECOND_GROUP_ID_BASE, raw)?;
    let enhanced = whole(parameter(parameters, 1)?)? == 1;
    let mut break_steps = vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
        slot: pending,
        value: ValueExpr::Literal(RuleValue::Integer(1)),
    })];
    if enhanced {
        break_steps.push(ProgramStep::If {
            condition: ConditionExpr::EnemyRank(target, EnemyRank::EliteOrBoss),
            then_program: elite_program,
            else_program: None,
        });
    }
    let break_definition = ProgramDefinition::new(
        break_program,
        enhanced.then_some(elite_program).into_iter().collect(),
        vec![owner, target, allies],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(break_steps);
    let ready_definition = ProgramDefinition::new(
        ready_program,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::SetSlot {
            slot: pending,
            value: ValueExpr::Literal(RuleValue::Integer(2)),
        },
    )]);
    let consume_definition = ProgramDefinition::new(
        consume_program,
        Vec::new(),
        vec![owner],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: pending,
            value: ValueExpr::Literal(RuleValue::Integer(0)),
        }),
    ]);
    let elite_definition = ProgramDefinition::new(
        elite_program,
        Vec::new(),
        vec![allies],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::AdvanceAction {
            selector: allies,
            amount: scalar(1_000_000),
        },
    )]);
    let advance_definition = ProgramDefinition::new(
        advance_program,
        Vec::new(),
        vec![owner],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::AdvanceAction {
            selector: owner,
            amount: scalar(1_000_000),
        },
    )]);
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(1))),
        DurationClock::ActionEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let mut programs = vec![
        break_definition,
        ready_definition,
        consume_definition,
        advance_definition,
    ];
    if enhanced {
        programs.push(elite_definition);
    }
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
                stat: StatKind::Hp,
                stage: FormulaStage::DamageBoost,
                purpose: FormulaPurpose::OrdinaryDamage,
                value: scalar(parameter(parameters, 0)?),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            }],
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier])
                    .with_runtime_template(runtime),
            ],
            programs,
            slots: vec![
                StateSlotDef::new(
                    pending,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(2)),
            ],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::WeaknessBroken,
                    OnceScope::Action,
                    EventFilter {
                        applier_selector: Some(owner),
                        target_selector: Some(target),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    break_program,
                ),
                trigger(
                    id::<TriggerId>(READY_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionResolved,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(owner),
                        ability_tag: Some(AbilityTag::Attack),
                        ..EventFilter::default()
                    },
                    integer_slot_equals(pending, 1),
                    ready_program,
                ),
                trigger(
                    id::<TriggerId>(CONSUME_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionStarted,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(owner),
                        ability_tag: Some(AbilityTag::Attack),
                        ..EventFilter::default()
                    },
                    integer_slot_equals(pending, 2),
                    consume_program,
                ),
                trigger(
                    id::<TriggerId>(ADVANCE_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnEnded,
                    OnceScope::Turn,
                    EventFilter {
                        owner_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    positive_integer_slot(pending),
                    advance_program,
                ),
            ],
        },
    )
}

fn archery_duel(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let heal = multiply(
        multiply(
            ValueExpr::QueryStat {
                subject: StatQuerySubject::Owner,
                stat: StatKind::Hp,
                purpose: FormulaPurpose::Healing,
            },
            scalar(parameter(parameters, 0)?),
        ),
        ValueExpr::Convert {
            value: Box::new(ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::Owner,
                effect: CRITICAL_BOOST,
            }),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        },
    );
    let definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: owner,
                amount: heal,
                apply_formula_modifiers: true,
            })]);
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::TurnStarted,
        OnceScope::Turn,
        EventFilter {
            owner_selector: Some(owner),
            ..EventFilter::default()
        },
        has_critical_boost(owner),
        program,
    )];
    if binding.source_binding_key() == Some(ARCHERY_DUEL_ENHANCED) {
        triggers.push(trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::ActionResolved,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                ability_tag: Some(AbilityTag::Ultimate),
                ..EventFilter::default()
            },
            has_critical_boost(owner),
            program,
        ));
    }
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![definition],
            triggers,
            ..RuleParts::default()
        },
    )
}

pub(super) fn add_critical_boost(
    rule: &mut ExecutableBattleRule,
    blessings: &BlessingContributionSet,
) -> Result<(), BattleRuleLoweringError> {
    let maximum = if level_binding_key_selected(blessings, EMPYREAN_IMPERIUM_ENHANCED) {
        12
    } else {
        8
    };
    let mut groups = vec![
        ModifierStackingGroup {
            id: CRIT_RATE_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        },
        ModifierStackingGroup {
            id: CRIT_DAMAGE_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        },
    ];
    let mut modifiers = vec![
        stack_modifier(
            CRIT_RATE_MODIFIER,
            CRIT_RATE_GROUP,
            StatKind::CritRate,
            60_000,
        ),
        stack_modifier(
            CRIT_DAMAGE_MODIFIER,
            CRIT_DAMAGE_GROUP,
            StatKind::CritDamage,
            120_000,
        ),
    ];
    let vendetta = selected_level_parameters(blessings, SKYWARD_VENDETTA)
        .or_else(|| selected_level_parameters(blessings, SKYWARD_VENDETTA_ENHANCED));
    if let Some(parameters) = vendetta {
        groups.push(ModifierStackingGroup {
            id: VENDETTA_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(vendetta_modifier(parameters)?);
    }
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
    let mut all_groups = rule.modifier_groups.to_vec();
    all_groups.extend(groups);
    all_groups.sort_unstable_by_key(|group| group.id);
    all_groups.dedup_by_key(|group| group.id);
    rule.modifier_groups = all_groups.into_boxed_slice();
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let mut all_modifiers = rule.modifiers.to_vec();
    all_modifiers.extend(modifiers);
    all_modifiers.sort_unstable_by_key(|modifier| modifier.id);
    all_modifiers.dedup_by_key(|modifier| modifier.id);
    rule.modifiers = all_modifiers.into_boxed_slice();
    let mut effects = rule.effects.to_vec();
    effects.push(
        EffectDefinition::new(CRITICAL_BOOST, Vec::new(), modifier_ids)
            .with_runtime_template(runtime),
    );
    effects.sort_unstable_by_key(EffectDefinition::id);
    effects.dedup_by_key(|effect| effect.id());
    rule.effects = effects.into_boxed_slice();
    if rule.attachment == RuleAttachment::EveryPlayer {
        add_critical_boost_lifecycle(rule)?;
    }
    Ok(())
}

fn add_critical_boost_lifecycle(
    rule: &mut ExecutableBattleRule,
) -> Result<(), BattleRuleLoweringError> {
    let total_stacks = ValueExpr::SelectorSum {
        selector: CRITICAL_ALLIES_SELECTOR,
        value: Box::new(ValueExpr::QueryEffectStacks {
            subject: StatQuerySubject::CurrentTarget,
            effect: CRITICAL_BOOST,
        }),
    };
    let transfer = ProgramDefinition::new(
        CRITICAL_TRANSFER_PROGRAM,
        Vec::new(),
        vec![CRITICAL_OWNER_SELECTOR, CRITICAL_ALLIES_SELECTOR],
        vec![CRITICAL_BOOST],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: CRITICAL_ALLIES_SELECTOR,
            effect: CRITICAL_BOOST,
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: CRITICAL_OWNER_SELECTOR,
            effect: CRITICAL_BOOST,
            stacks: total_stacks.clone(),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
    ]);
    let reset = ProgramDefinition::new(
        CRITICAL_RESET_PROGRAM,
        Vec::new(),
        vec![CRITICAL_ALLIES_SELECTOR],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveEffect {
            selector: CRITICAL_ALLIES_SELECTOR,
            effect: CRITICAL_BOOST,
        },
    )]);
    let mut transfer_trigger = trigger(
        CRITICAL_TRANSFER_TRIGGER,
        RuleEventPoint::TurnStarted,
        OnceScope::Turn,
        EventFilter {
            owner_selector: Some(CRITICAL_OWNER_SELECTOR),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(total_stacks),
            operator: Comparison::Greater,
            rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
        },
        CRITICAL_TRANSFER_PROGRAM,
    );
    transfer_trigger.priority = ReactionPriority::new(-200);
    let mut reset_trigger = trigger(
        CRITICAL_RESET_TRIGGER,
        RuleEventPoint::DamageApplied,
        OnceScope::Event,
        EventFilter {
            target_selector: Some(CRITICAL_OWNER_SELECTOR),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        CRITICAL_RESET_PROGRAM,
    );
    reset_trigger.priority = ReactionPriority::new(-200);

    let mut selectors = rule.selectors.to_vec();
    selectors.extend([
        SelectorDefinition::new(CRITICAL_OWNER_SELECTOR).with_rule_units(owner_selector()?),
        SelectorDefinition::new(CRITICAL_ALLIES_SELECTOR).with_rule_units(all_ally_selector()?),
    ]);
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    selectors.dedup_by_key(|selector| selector.id());
    rule.selectors = selectors.into_boxed_slice();
    let mut programs = rule.programs.to_vec();
    programs.extend([transfer, reset]);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    programs.dedup_by_key(|program| program.id());
    rule.programs = programs.into_boxed_slice();

    let runtime = rule
        .definition
        .runtime()
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let mut triggers = runtime.triggers().to_vec();
    triggers.extend([transfer_trigger, reset_trigger]);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    let source = runtime.source().clone();
    let slots = runtime.state_slots().to_vec();
    rule.definition = RuleDefinition::new(
        rule.definition.id(),
        rule.programs.iter().map(ProgramDefinition::id).collect(),
        rule.selectors.iter().map(SelectorDefinition::id).collect(),
    )
    .with_runtime(BattleRuleDefinition::new(source, slots, triggers, None));
    Ok(())
}

fn vendetta_modifier(
    parameters: &[ExactParameter],
) -> Result<ModifierDefinition, BattleRuleLoweringError> {
    let base = ValueExpr::Divide {
        lhs: Box::new(multiply(
            ValueExpr::Maximum(
                Box::new(ValueExpr::Subtract(
                    Box::new(ValueExpr::QueryStat {
                        subject: StatQuerySubject::Owner,
                        stat: StatKind::CritRate,
                        purpose: FormulaPurpose::Stat,
                    }),
                    Box::new(scalar(1_000_000)),
                )),
                Box::new(scalar(0)),
            ),
            scalar(parameter(parameters, 0)?),
        )),
        rhs: Box::new(scalar(10_000)),
        rounding: Rounding::NearestTiesEven,
    };
    let per_stack = parameters
        .get(2)
        .map(|_| {
            Ok(multiply(
                ValueExpr::Convert {
                    value: Box::new(ValueExpr::Slot(CRITICAL_BOOST_STACK_SLOT)),
                    target: RuleValueKind::Scalar,
                    rounding: Rounding::NearestTiesEven,
                },
                scalar(parameter_six(parameters, 2)?),
            ))
        })
        .transpose()?
        .unwrap_or_else(|| scalar(0));
    Ok(ModifierDefinition {
        id: VENDETTA_MODIFIER,
        stat: StatKind::CritDamage,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Clamp {
            value: Box::new(ValueExpr::Add(Box::new(base), Box::new(per_stack))),
            minimum: Box::new(scalar(0)),
            maximum: Box::new(scalar(parameter(parameters, 1)?)),
        },
        stacking_group: VENDETTA_GROUP,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: Some(CRITICAL_BOOST_STACK_SLOT),
        filters: Box::new([]),
    })
}

fn stack_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    ratio: i64,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: multiply(
            ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(CRITICAL_BOOST_STACK_SLOT)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            },
            scalar(ratio),
        ),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(CRITICAL_BOOST_STACK_SLOT),
        filters: Box::new([]),
    }
}

fn apply_critical_boost(
    program: ProgramId,
    owner: SelectorId,
    stacks: ValueExpr,
) -> ProgramDefinition {
    ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner],
        vec![CRITICAL_BOOST],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: CRITICAL_BOOST,
            stacks,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )])
}

fn has_critical_boost(owner: SelectorId) -> ConditionExpr {
    ConditionExpr::EffectExists {
        selector: owner,
        effect: CRITICAL_BOOST,
    }
}

fn positive_integer_slot(slot: StateSlotDefinitionId) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Greater,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
    }
}

fn level_binding_key_selected(blessings: &BlessingContributionSet, key: &str) -> bool {
    selected_level_parameters(blessings, key).is_some()
}

fn whole(value: i64) -> Result<i64, BattleRuleLoweringError> {
    value
        .checked_div(1_000_000)
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}

fn parameter_six(
    parameters: &[ExactParameter],
    index: usize,
) -> Result<i64, BattleRuleLoweringError> {
    let value = *parameters
        .get(index)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    if value.scale() <= 6 {
        return parameter(parameters, index);
    }
    if value.coefficient() < 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let divisor = 10_i64
        .checked_pow(u32::from(value.scale() - 6))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let quotient = value.coefficient() / divisor;
    let remainder = value.coefficient() % divisor;
    let doubled = remainder
        .checked_mul(2)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    if doubled > divisor || doubled == divisor && quotient % 2 != 0 {
        quotient
            .checked_add(1)
            .ok_or(BattleRuleLoweringError::InvalidParameter)
    } else {
        Ok(quotient)
    }
}

fn empty_rule(binding: &UniverseBattleRuleBinding) -> ExecutableBattleRule {
    finish_unchecked(binding, RuleParts::default())
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
    parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    Ok(finish_unchecked(binding, parts))
}

fn finish_unchecked(
    binding: &UniverseBattleRuleBinding,
    mut parts: RuleParts,
) -> ExecutableBattleRule {
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
    ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}
