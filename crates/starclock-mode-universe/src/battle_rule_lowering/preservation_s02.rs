use super::*;

pub(super) fn preservation_safe_load(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let value = ValueExpr::Minimum(
        Box::new(multiply(
            shield(StatQuerySubject::Owner, ShieldObservation::Current),
            scalar(parameter(parameters, 0)?),
        )),
        Box::new(multiply(
            ValueExpr::QueryBaseStat {
                subject: StatQuerySubject::Owner,
                stat: StatKind::Atk,
            },
            scalar(parameter(parameters, 1)?),
        )),
    );
    persistent_modifier_rule(
        binding,
        StatKind::Atk,
        FormulaStage::Flat,
        FormulaPurpose::Stat,
        value,
        Vec::new(),
    )
}

pub(super) fn preservation_sanctuary(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let apply_shield = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let turn_trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let applied_trigger = id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?;
    let chance = parameter(parameters, 0)?;
    let duration = whole(parameter(parameters, 2)?)?;
    if duration != 1 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let runtime = permanent_effect_runtime()?;
    let chance_policy = if chance >= 1_000_000 {
        RuleEffectChancePolicy::Guaranteed
    } else {
        RuleEffectChancePolicy::Fixed
    };
    let root_definition =
        ProgramDefinition::new(root, Vec::new(), vec![owner], vec![effect], Vec::new()).with_steps(
            vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    chance: chance_policy,
                    base_chance: (chance_policy == RuleEffectChancePolicy::Fixed)
                        .then(|| scalar(chance)),
                    rng_purpose: (chance_policy == RuleEffectChancePolicy::Fixed)
                        .then_some(DrawPurpose::EFFECT_CHANCE),
                }),
            ],
        );
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 1)?),
    );
    let shield_definition = ProgramDefinition::new(
        apply_shield,
        Vec::new(),
        vec![owner],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Shield {
            selector: owner,
            amount,
            effect,
        },
    )]);
    Ok(executable_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![EffectDefinition::new(effect, Vec::new(), Vec::new()).with_runtime_template(runtime)],
        vec![root_definition, shield_definition],
        Vec::new(),
        vec![
            trigger(
                turn_trigger,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                root,
            ),
            trigger(
                applied_trigger,
                RuleEventPoint::EffectApplied,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(owner),
                    source: Some(binding.source().definition()),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                apply_shield,
            ),
        ],
    ))
}

pub(super) fn preservation_shield_capacity(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let direction = match whole(parameter(parameters, 1)?)? {
        1 => FormulaSubject::Source,
        2 => FormulaSubject::Target,
        _ => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    persistent_modifier_rule(
        binding,
        StatKind::ShieldStrength,
        FormulaStage::Shield,
        FormulaPurpose::Shield,
        scalar(parameter(parameters, 0)?),
        vec![ModifierFilter::FormulaSubject(direction)],
    )
}

pub(super) fn preservation_provider_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let advance = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let expire = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let counter = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let apply_trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let advance_trigger = id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?;
    let expire_trigger = id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?;
    let duration = whole(parameter(parameters, 1)?)?;
    if duration != 2 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let amount = multiply(
        ValueExpr::ReadEventProperty(EventValueProperty::ShieldChangeAmount),
        scalar(parameter(parameters, 0)?),
    );
    let apply_definition =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: counter,
                    value: ValueExpr::Literal(RuleValue::Integer(0)),
                }),
                ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::Shield {
                    selector: owner,
                    amount,
                    effect,
                }),
            ]);
    let advance_definition =
        ProgramDefinition::new(advance, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::SetSlot {
                    slot: counter,
                    value: ValueExpr::Literal(RuleValue::Integer(1)),
                },
            )]);
    let expire_definition =
        ProgramDefinition::new(expire, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: counter,
                    value: ValueExpr::Literal(RuleValue::Integer(2)),
                }),
            ]);
    let positive = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ShieldChangeAmount,
        )),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    };
    let ally = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::EventTarget),
        operator: Comparison::NotEqual,
        rhs: Box::new(ValueExpr::EventOwner),
    };
    Ok(executable_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![EffectDefinition::new(effect, Vec::new(), Vec::new())],
        vec![apply_definition, advance_definition, expire_definition],
        vec![
            StateSlotDef::new(
                counter,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(2),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(2)),
        ],
        vec![
            trigger(
                apply_trigger,
                RuleEventPoint::ShieldChanged,
                OnceScope::Action,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                ConditionExpr::All(vec![positive, ally].into_boxed_slice()),
                apply,
            ),
            trigger(
                advance_trigger,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                integer_slot_equals(counter, 0),
                advance,
            ),
            trigger(
                expire_trigger,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                integer_slot_equals(counter, 1),
                expire,
            ),
        ],
    ))
}

pub(super) fn preservation_assemble(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    blessing_count: u16,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let cap = whole(parameter(parameters, 1)?)?;
    let count = i64::from(blessing_count.min(cap));
    let value = parameter(parameters, 0)?
        .checked_mul(count)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    persistent_modifier_rule(
        binding,
        StatKind::Def,
        FormulaStage::PercentOfBase,
        FormulaPurpose::Stat,
        scalar(value),
        Vec::new(),
    )
}

pub(super) fn preservation_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.preservation")
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

fn persistent_modifier_rule(
    binding: &UniverseBattleRuleBinding,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: ValueExpr,
    filters: Vec<ModifierFilter>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let trigger_id = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let runtime = permanent_effect_runtime()?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    let modifier_group = ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    };
    let modifier_definition = ModifierDefinition {
        id: modifier,
        stat,
        stage,
        purpose,
        value,
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: filters.into_boxed_slice(),
    };
    let selectors = vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)];
    let effects = vec![
        EffectDefinition::new(effect, Vec::new(), vec![modifier]).with_runtime_template(runtime),
    ];
    let definition = RuleDefinition::new(binding.rule(), vec![program], vec![owner]).with_runtime(
        BattleRuleDefinition::new(
            binding.source().clone(),
            Vec::new(),
            vec![trigger(
                trigger_id,
                RuleEventPoint::BattleStarted,
                OnceScope::Battle,
                EventFilter::default(),
                ConditionExpr::Literal(true),
                program,
            )],
            None,
        ),
    );
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: vec![modifier_group].into_boxed_slice(),
        modifiers: vec![modifier_definition].into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: vec![program_definition].into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn permanent_effect_runtime() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
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

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    let value = value
        .checked_div(1_000_000)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    u16::try_from(value).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}
