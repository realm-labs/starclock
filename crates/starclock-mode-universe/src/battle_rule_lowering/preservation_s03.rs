use super::*;
use starclock_combat::Scalar;

const SENTINEL: &str = "StageAbility_612051";
const PATCH: &str = "StageAbility_612052";
const COMPENSATION: &str = "StageAbility_612053";
const FIRMNESS: &str = "StageAbility_612054";
const ROTATION: &str = "StageAbility_612055";
const FIRMNESS_MODIFIER_STRIDE: u32 = 0x0001_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [SENTINEL, PATCH, COMPENSATION, FIRMNESS, ROTATION] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            SENTINEL => timed_max_hp_shield(binding, parameters, RuleEventPoint::BattleStarted)?,
            PATCH => lost_hp_shield(binding, parameters)?,
            COMPENSATION => {
                timed_max_hp_shield(binding, parameters, RuleEventPoint::WeaknessBroken)?
            }
            FIRMNESS => shielded_damage_reduction(binding, parameters)?,
            ROTATION => shield_cleanse(binding, parameters)?,
            _ => unreachable!("closed Preservation S03 binding set"),
        });
    }
    Ok(output)
}

fn timed_max_hp_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    event: RuleEventPoint,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 0)?),
    );
    timed_shield_rule(
        binding,
        amount,
        whole(parameter(parameters, 1)?)?,
        event,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn lost_hp_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let accumulate = id::<ProgramId>(THIRD_TRIGGER_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let lost_hp = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let accumulate_trigger = id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?;
    let amount = multiply(ValueExpr::Slot(lost_hp), scalar(parameter(parameters, 0)?));
    let accumulate_program =
        ProgramDefinition::new(accumulate, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddSlot {
                    slot: lost_hp,
                    value: ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
                        EventValueProperty::HpChangeAmount,
                    ))),
                },
            )]);
    let slot = StateSlotDef::new(
        lost_hp,
        RuleValueKind::Scalar,
        BattleRuleScope::Battle,
        RuleValue::Scalar(Scalar::ZERO),
    );
    let damaged = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::HpChangeAmount,
        )),
        operator: Comparison::Less,
        rhs: Box::new(scalar(0)),
    };
    let has_loss = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(lost_hp)),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    };
    let accumulate_definition = trigger(
        accumulate_trigger,
        RuleEventPoint::DamageApplied,
        OnceScope::Event,
        EventFilter {
            target_selector: Some(owner),
            ..EventFilter::default()
        },
        damaged,
        accumulate,
    );
    timed_shield_rule(
        binding,
        amount,
        whole(parameter(parameters, 1)?)?,
        RuleEventPoint::ActionResolved,
        EventFilter::default(),
        has_loss,
        Vec::new(),
        vec![accumulate_program],
        vec![slot],
        vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: lost_hp,
            value: scalar(0),
        })],
        vec![accumulate_definition],
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn timed_shield_rule(
    binding: &UniverseBattleRuleBinding,
    amount: ValueExpr,
    duration: u16,
    event: RuleEventPoint,
    event_filter: EventFilter,
    event_condition: ConditionExpr,
    mut extra_selectors: Vec<SelectorDefinition>,
    mut extra_programs: Vec<ProgramDefinition>,
    mut extra_slots: Vec<StateSlotDef>,
    mut after_apply_steps: Vec<ProgramStep>,
    mut extra_triggers: Vec<TriggerDef>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    if duration == 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
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
    let mut apply_steps = vec![
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
    ];
    apply_steps.append(&mut after_apply_steps);
    let mut apply_selectors = vec![owner];
    for selector in [
        event_filter.owner_selector,
        event_filter.actor_selector,
        event_filter.applier_selector,
        event_filter.target_selector,
    ]
    .into_iter()
    .flatten()
    {
        apply_selectors.push(selector);
    }
    for trigger in extra_triggers
        .iter()
        .filter(|trigger| trigger.program == apply)
    {
        for selector in [
            trigger.filter.owner_selector,
            trigger.filter.actor_selector,
            trigger.filter.applier_selector,
            trigger.filter.target_selector,
        ]
        .into_iter()
        .flatten()
        {
            apply_selectors.push(selector);
        }
    }
    apply_selectors.sort_unstable();
    apply_selectors.dedup();
    let apply_definition =
        ProgramDefinition::new(apply, Vec::new(), apply_selectors, vec![effect], Vec::new())
            .with_steps(apply_steps);
    let advance_definition =
        ProgramDefinition::new(advance, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddSlot {
                    slot: counter,
                    value: ValueExpr::Literal(RuleValue::Integer(1)),
                },
            )]);
    let expire_steps = vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveShield {
            selector: owner,
            effect,
        },
    )];
    let expire_definition =
        ProgramDefinition::new(expire, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(expire_steps);
    extra_programs.extend([apply_definition, advance_definition, expire_definition]);
    extra_programs.sort_unstable_by_key(ProgramDefinition::id);
    extra_slots.push(
        StateSlotDef::new(
            counter,
            RuleValueKind::Integer,
            BattleRuleScope::Battle,
            RuleValue::Integer(i64::from(duration)),
        )
        .with_bounds(
            RuleValue::Integer(0),
            RuleValue::Integer(i64::from(duration)),
        ),
    );
    extra_selectors.push(SelectorDefinition::new(owner).with_rule_units(owner_selector()?));
    extra_selectors.sort_unstable_by_key(SelectorDefinition::id);
    extra_selectors.dedup_by_key(|selector| selector.id());
    let effects = vec![EffectDefinition::new(effect, Vec::new(), Vec::new())];
    let mut triggers = vec![
        trigger(
            apply_trigger,
            event,
            if event == RuleEventPoint::BattleStarted {
                OnceScope::Battle
            } else {
                OnceScope::Event
            },
            event_filter,
            event_condition,
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
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(counter)),
                operator: Comparison::Less,
                rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(
                    duration - 1,
                )))),
            },
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
            integer_slot_equals(counter, i64::from(duration - 1)),
            expire,
        ),
    ];
    triggers.append(&mut extra_triggers);
    Ok(executable_rule(
        binding,
        extra_selectors,
        effects,
        extra_programs,
        extra_slots,
        triggers,
    ))
}

fn shielded_damage_reduction(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let trigger_id = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let value = ValueExpr::Choose {
        condition: Box::new(ConditionExpr::Compare {
            lhs: Box::new(shield(StatQuerySubject::Owner, ShieldObservation::Current)),
            operator: Comparison::Greater,
            rhs: Box::new(scalar(0)),
        }),
        when_true: Box::new(scalar(parameter(parameters, 0)?)),
        when_false: Box::new(scalar(0)),
    };
    let purposes = [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ];
    let modifiers = purposes
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| {
            let base = MODIFIER_ID_BASE
                .checked_add(
                    u32::try_from(index)
                        .expect("purpose count fits u32")
                        .checked_mul(FIRMNESS_MODIFIER_STRIDE)
                        .expect("reserved modifier stride fits"),
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
            Ok(ModifierDefinition {
                id: id::<ModifierDefinitionId>(base, raw)?,
                stat: StatKind::Hp,
                stage: FormulaStage::Mitigation,
                purpose,
                value: value.clone(),
                stacking_group: group,
                priority: 0,
                floor: Some(Scalar::ZERO),
                cap: Some(Scalar::ONE),
                cap_stage: FormulaStage::Mitigation,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let runtime = permanent_effect_runtime()?;
    let program_definition =
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
            )]);
    let mut rule = executable_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), modifier_ids).with_runtime_template(runtime),
        ],
        vec![program_definition],
        Vec::new(),
        vec![trigger(
            trigger_id,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            program,
        )],
    );
    rule.modifier_groups = vec![ModifierStackingGroup {
        id: group,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    }]
    .into_boxed_slice();
    rule.modifiers = modifiers.into_boxed_slice();
    Ok(rule)
}

fn shield_cleanse(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let cleanse = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let marker = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let shield_trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let applied_trigger = id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?;
    let chance = parameter(parameters, 0)?;
    let apply_definition =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![marker], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect: marker,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Fixed,
                    base_chance: Some(scalar(chance)),
                    rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
                },
            )]);
    let cleanse_definition =
        ProgramDefinition::new(cleanse, Vec::new(), vec![owner], vec![marker], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::Cleanse {
                    selector: owner,
                    maximum: 1,
                }),
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect: marker,
                }),
            ]);
    let positive = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ShieldChangeAmount,
        )),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    };
    Ok(executable_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(marker, Vec::new(), Vec::new())
                .with_runtime_template(permanent_effect_runtime()?),
        ],
        vec![apply_definition, cleanse_definition],
        Vec::new(),
        vec![
            trigger(
                shield_trigger,
                RuleEventPoint::ShieldChanged,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(owner),
                    ..EventFilter::default()
                },
                positive,
                apply,
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
                cleanse,
            ),
        ],
    ))
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
    u16::try_from(
        value
            .checked_div(1_000_000)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
    )
    .map_err(|_| BattleRuleLoweringError::InvalidParameter)
}
