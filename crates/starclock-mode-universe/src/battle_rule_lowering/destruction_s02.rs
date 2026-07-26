use super::preservation_s03::timed_shield_rule;
use super::*;

const GRIT_MITIGATION_NORMAL: &str = "StageAbility_61254201";
const GRIT_MITIGATION_ENHANCED: &str = "StageAbility_61254202";
const LOST_HP_STATS_NORMAL: &str = "StageAbility_61254301";
const LOST_HP_STATS_ENHANCED: &str = "StageAbility_61254302";
const LOW_HP_DAMAGE: &str = "StageAbility_61254401";
const LOW_HP_HEALING_NORMAL: &str = "StageAbility_61254501";
const LOW_HP_HEALING_ENHANCED: &str = "StageAbility_61254502";
const ULTIMATE_SHIELD_NORMAL: &str = "StageAbility_61254601";
const ULTIMATE_SHIELD_ENHANCED: &str = "StageAbility_61254602";
const BLESSING_ATTACK: &str = "StageAbility_61255001";
const MISSING_HP_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x79d2_0001).expect("reserved slot ID");
const LOW_HP_DAMAGE_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x79d2_0002).expect("reserved slot ID");

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        GRIT_MITIGATION_NORMAL,
        GRIT_MITIGATION_ENHANCED,
        LOST_HP_STATS_NORMAL,
        LOST_HP_STATS_ENHANCED,
        LOW_HP_DAMAGE,
        LOW_HP_HEALING_NORMAL,
        LOW_HP_HEALING_ENHANCED,
        ULTIMATE_SHIELD_NORMAL,
        ULTIMATE_SHIELD_ENHANCED,
        BLESSING_ATTACK,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            GRIT_MITIGATION_NORMAL | GRIT_MITIGATION_ENHANCED => empty_rule(binding),
            LOST_HP_STATS_NORMAL | LOST_HP_STATS_ENHANCED => {
                let mut stats = vec![(
                    StatKind::Atk,
                    destruction_s01::parameter_six(parameters, 0)?,
                )];
                if parameters.len() == 2 {
                    stats.push((StatKind::Def, parameter(parameters, 1)?));
                }
                missing_hp_stat_rule(binding, &stats)?
            }
            LOW_HP_DAMAGE => low_hp_damage(binding, parameters)?,
            LOW_HP_HEALING_NORMAL | LOW_HP_HEALING_ENHANCED => low_hp_healing(binding, parameters)?,
            ULTIMATE_SHIELD_NORMAL | ULTIMATE_SHIELD_ENHANCED => {
                ultimate_shield(binding, parameters)?
            }
            BLESSING_ATTACK => blessing_attack(catalog, blessings, binding, parameters)?,
            _ => unreachable!("closed Destruction S02 binding set"),
        });
    }
    Ok(output)
}

fn low_hp_damage(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let apply = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let clear = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let primary_threshold = scalar(parameter(parameters, 0)?);
    let primary_bonus = scalar(parameter(parameters, 1)?);
    let secondary_threshold = scalar(parameter(parameters, 2)?);
    let secondary_bonus = scalar(parameter(parameters, 3)?);
    let root_program = ProgramDefinition::new(
        root,
        vec![apply, clear],
        vec![owner],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::If {
        condition: ConditionExpr::Compare {
            lhs: Box::new(hp_ratio()),
            operator: Comparison::Less,
            rhs: Box::new(primary_threshold),
        },
        then_program: apply,
        else_program: Some(clear),
    }]);
    let apply_program =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: ValueExpr::Choose {
                        condition: Box::new(ConditionExpr::Compare {
                            lhs: Box::new(hp_ratio()),
                            operator: Comparison::Less,
                            rhs: Box::new(secondary_threshold),
                        }),
                        when_true: Box::new(integer(2)),
                        when_false: Box::new(integer(1)),
                    },
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
            ]);
    let clear_program =
        ProgramDefinition::new(clear, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                },
            )]);
    let value = ValueExpr::Choose {
        condition: Box::new(ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::Slot(LOW_HP_DAMAGE_STACK_SLOT)),
            operator: Comparison::Less,
            rhs: Box::new(integer(2)),
        }),
        when_true: Box::new(primary_bonus.clone()),
        when_false: Box::new(ValueExpr::Add(
            Box::new(primary_bonus.clone()),
            Box::new(secondary_bonus),
        )),
    };
    let modifiers = damage_purposes()
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| {
            Ok(ModifierDefinition {
                id: local_id::<ModifierDefinitionId>(0x79d5_0000, raw, index)?,
                stat: StatKind::Hp,
                stage: FormulaStage::DamageBoost,
                purpose,
                value: value.clone(),
                stacking_group: group,
                priority: 0,
                floor: Some(starclock_combat::Scalar::ZERO),
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::RecomputeOnStackChange,
                source_stack_slot: Some(LOW_HP_DAMAGE_STACK_SLOT),
                filters: Box::new([]),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
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
                    .with_runtime_template(permanent_effect_runtime(2)?),
            ],
            programs: vec![root_program, apply_program, clear_program],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    root,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HpChanged,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    root,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn low_hp_healing(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let reset = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let heal = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let amount_slot = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let maximum_hp_value = maximum_hp();
    let heal_amount = multiply(maximum_hp_value.clone(), scalar(parameter(parameters, 1)?));
    let remaining = ValueExpr::Maximum(
        Box::new(ValueExpr::Subtract(
            Box::new(multiply(
                maximum_hp_value,
                scalar(parameter(parameters, 2)?),
            )),
            Box::new(ValueExpr::Slot(amount_slot)),
        )),
        Box::new(scalar(0)),
    );
    let planned = ValueExpr::Minimum(Box::new(heal_amount), Box::new(remaining));
    let reset_program =
        ProgramDefinition::new(reset, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: amount_slot,
                value: scalar(0),
            })],
        );
    let heal_program =
        ProgramDefinition::new(heal, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![
                ProgramStep::Operation(RuleOperationTemplate::Heal {
                    selector: owner,
                    amount: planned.clone(),
                    apply_formula_modifiers: false,
                }),
                ProgramStep::Operation(RuleOperationTemplate::AddSlot {
                    slot: amount_slot,
                    value: planned,
                }),
            ],
        );
    let condition = ConditionExpr::All(
        vec![
            ConditionExpr::Compare {
                lhs: Box::new(hp_ratio()),
                operator: Comparison::Less,
                rhs: Box::new(scalar(parameter(parameters, 0)?)),
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(amount_slot)),
                operator: Comparison::Less,
                rhs: Box::new(multiply(maximum_hp(), scalar(parameter(parameters, 2)?))),
            },
        ]
        .into_boxed_slice(),
    );
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![reset_program, heal_program],
            slots: vec![StateSlotDef::new(
                amount_slot,
                RuleValueKind::Scalar,
                BattleRuleScope::Battle,
                RuleValue::Scalar(starclock_combat::Scalar::ZERO),
            )],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionStarted,
                    OnceScope::Action,
                    EventFilter::default(),
                    reset,
                ),
                trigger_with_condition(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::DamageApplied,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    condition.clone(),
                    heal,
                ),
                trigger_with_condition(
                    id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HpChanged,
                    OnceScope::Event,
                    EventFilter {
                        actor_selector: Some(owner),
                        target_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    ConditionExpr::All(vec![negative_hp_change(), condition].into_boxed_slice()),
                    heal,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn ultimate_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let missing_hp = ValueExpr::Subtract(
        Box::new(maximum_hp()),
        Box::new(ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        }),
    );
    let mut amount = multiply(missing_hp, scalar(parameter(parameters, 0)?));
    if parameters.len() == 3 {
        amount = ValueExpr::Add(
            Box::new(amount),
            Box::new(multiply(maximum_hp(), scalar(parameter(parameters, 2)?))),
        );
    }
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    timed_shield_rule(
        binding,
        amount,
        whole(parameter(parameters, 1)?)?,
        RuleEventPoint::ActionResolved,
        EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(AbilityTag::Ultimate),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn blessing_attack(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let cap = whole(parameter(parameters, 1)?)?;
    let count = i64::from(destruction_blessing_count(catalog, blessings)?.min(cap));
    let value = parameter(parameters, 0)?
        .checked_mul(count)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    persistent_modifier_rule(
        binding,
        StatKind::Atk,
        FormulaStage::PercentOfBase,
        FormulaPurpose::Stat,
        scalar(value),
        Vec::new(),
    )
}

fn destruction_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.destruction")
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

pub(super) fn missing_hp_stat_rule(
    binding: &UniverseBattleRuleBinding,
    stats: &[(StatKind, i64)],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    if stats.is_empty() {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let apply = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let clear = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let missing_percent = missing_hp_percent();
    let root_program = ProgramDefinition::new(
        root,
        vec![apply, clear],
        vec![owner],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::If {
        condition: ConditionExpr::Compare {
            lhs: Box::new(missing_percent.clone()),
            operator: Comparison::Greater,
            rhs: Box::new(integer(0)),
        },
        then_program: apply,
        else_program: Some(clear),
    }]);
    let apply_program =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: missing_percent,
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
            ]);
    let clear_program =
        ProgramDefinition::new(clear, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                },
            )]);
    let groups = stats
        .iter()
        .enumerate()
        .map(|(index, _)| {
            let base = [MODIFIER_GROUP_ID_BASE, 0x79d3_0000]
                .get(index)
                .copied()
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
            Ok(ModifierStackingGroup {
                id: id::<ModifierStackingGroupId>(base, raw)?,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifiers = stats
        .iter()
        .zip(&groups)
        .enumerate()
        .map(|(index, ((stat, ratio), group))| {
            let base = [MODIFIER_ID_BASE, 0x79d4_0000]
                .get(index)
                .copied()
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
            Ok(stack_stat_modifier(
                id::<ModifierDefinitionId>(base, raw)?,
                group.id,
                *stat,
                *ratio,
            ))
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        100,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    finish(
        binding,
        RuleParts {
            groups,
            modifiers,
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), modifier_ids)
                    .with_runtime_template(runtime),
            ],
            programs: vec![root_program, apply_program, clear_program],
            slots: Vec::new(),
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    root,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HpChanged,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    root,
                ),
            ],
        },
    )
}

fn missing_hp_percent() -> ValueExpr {
    ValueExpr::Convert {
        value: Box::new(ValueExpr::Multiply {
            lhs: Box::new(ValueExpr::Divide {
                lhs: Box::new(ValueExpr::Subtract(
                    Box::new(ValueExpr::QueryStat {
                        subject: StatQuerySubject::Owner,
                        stat: StatKind::Hp,
                        purpose: FormulaPurpose::Stat,
                    }),
                    Box::new(ValueExpr::QueryHp {
                        subject: StatQuerySubject::Owner,
                    }),
                )),
                rhs: Box::new(ValueExpr::QueryStat {
                    subject: StatQuerySubject::Owner,
                    stat: StatKind::Hp,
                    purpose: FormulaPurpose::Stat,
                }),
                rounding: Rounding::NearestTiesEven,
            }),
            rhs: Box::new(scalar(100_000_000)),
            rounding: Rounding::NearestTiesEven,
        }),
        target: RuleValueKind::Integer,
        rounding: Rounding::Floor,
    }
}

fn maximum_hp() -> ValueExpr {
    ValueExpr::QueryStat {
        subject: StatQuerySubject::Owner,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
    }
}

fn hp_ratio() -> ValueExpr {
    ValueExpr::Divide {
        lhs: Box::new(ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        }),
        rhs: Box::new(maximum_hp()),
        rounding: Rounding::NearestTiesEven,
    }
}

fn negative_hp_change() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::HpChangeAmount,
        )),
        operator: Comparison::Less,
        rhs: Box::new(scalar(0)),
    }
}

fn damage_purposes() -> [FormulaPurpose; 7] {
    [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ]
}

fn local_id<T>(base: u32, raw: u32, index: usize) -> Result<T, BattleRuleLoweringError>
where
    T: TryFrom<u32>,
{
    base.checked_add(raw)
        .and_then(|value| value.checked_add(u32::try_from(index).ok()?))
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn stack_stat_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    ratio: i64,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: multiply(
            ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(MISSING_HP_STACK_SLOT)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            },
            scalar(ratio),
        ),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(MISSING_HP_STACK_SLOT),
        filters: Box::new([]),
    }
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn trigger(
    id: TriggerId,
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
        filter,
        condition: ConditionExpr::Literal(true),
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn trigger_with_condition(
    id: TriggerId,
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn permanent_effect_runtime(
    maximum_stacks: u16,
) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        maximum_stacks,
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
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
    parts.triggers.sort_unstable_by_key(|trigger| trigger.id);
    let programs = parts.programs.iter().map(ProgramDefinition::id).collect();
    let selectors = parts.selectors.iter().map(SelectorDefinition::id).collect();
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), programs, selectors).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), parts.slots, parts.triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn empty_rule(binding: &UniverseBattleRuleBinding) -> ExecutableBattleRule {
    finish(binding, RuleParts::default()).expect("empty rule is valid")
}
