use super::*;

const TORMENT: &str = "StageAbility_612151";
const LOST_MEMORY: &str = "StageAbility_612152";
const STONE_COLD_HATRED: &str = "StageAbility_612153";
const PAIN_AND_SUFFERING: &str = "StageAbility_612154";
const PRIMORDIAL_HARDSHIP: &str = "StageAbility_612155";

const LOCAL_EFFECT_BASE: u32 = 0x7800_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7810_0000;
const LOCAL_GROUP_BASE: u32 = 0x7820_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7830_0000;
const LOCAL_PROGRAM_BASE: u32 = 0x7840_0000;
const CONTRIBUTION_RULE_ID_BASE: u32 = 0x7000_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        TORMENT,
        LOST_MEMORY,
        STONE_COLD_HATRED,
        PAIN_AND_SUFFERING,
        PRIMORDIAL_HARDSHIP,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            TORMENT => torment(binding, parameters)?,
            LOST_MEMORY => lost_memory(binding, parameters)?,
            STONE_COLD_HATRED => stone_cold_hatred(binding, parameters)?,
            PAIN_AND_SUFFERING => pain_and_suffering(binding, parameters)?,
            PRIMORDIAL_HARDSHIP => primordial_hardship(binding, parameters)?,
            _ => unreachable!("closed Remembrance S03 binding set"),
        });
    }
    Ok(output)
}

fn torment(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    first_player_rule(
        binding,
        RuleAttachment::FirstPlayer,
        vec![SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?)],
        vec![unique_group(group)],
        vec![modifier_definition(
            modifier,
            group,
            StatKind::EffectHitRate,
            FormulaStage::Flat,
            FormulaPurpose::EffectChance,
            parameter(parameters, 0)?,
            Vec::new(),
        )],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime(permanent_effect(EffectCategory::Buff)?),
        ],
        vec![effect_program(program, vec![allies], allies, effect, true)],
        Vec::new(),
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn lost_memory(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let attackers = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let freeze = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let threshold = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::EventTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 0)?),
    );
    first_player_rule(
        binding,
        RuleAttachment::EveryEnemy,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(attackers).with_rule_units(all_enemy_selector()?),
        ],
        Vec::new(),
        Vec::new(),
        vec![
            EffectDefinition::new(freeze, Vec::new(), Vec::new())
                .with_runtime(freeze_runtime(whole(parameter(parameters, 2)?)?)?),
        ],
        vec![
            effect_program(program, vec![owner, attackers], owner, freeze, false).with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect: freeze,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Resistible,
                    base_chance: Some(scalar(parameter(parameters, 1)?)),
                    rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
                }),
            ]),
        ],
        Vec::new(),
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Battle,
            EventFilter {
                actor_selector: Some(attackers),
                target_selector: Some(owner),
                ability_tag: Some(AbilityTag::Attack),
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Ordinary),
                ..EventFilter::default()
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::HpAfter)),
                operator: Comparison::Less,
                rhs: Box::new(threshold),
            },
            program,
        )],
    )
}

fn stone_cold_hatred(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    freeze_linked_modifier(
        binding,
        parameter(parameters, 0)?,
        FormulaStage::DamageBoost,
        &[FormulaPurpose::OrdinaryDamage],
        &["skill", "ultimate"],
    )
}

fn primordial_hardship(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    freeze_linked_modifier(
        binding,
        parameter(parameters, 0)?,
        FormulaStage::Vulnerability,
        &damage_purposes(),
        &[],
    )
}

fn freeze_linked_modifier(
    binding: &UniverseBattleRuleBinding,
    value: i64,
    stage: FormulaStage,
    purposes: &[FormulaPurpose],
    ability_tags: &[&str],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let remove = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let mut modifiers = Vec::new();
    for (purpose_index, purpose) in purposes.iter().copied().enumerate() {
        let tags = if ability_tags.is_empty() {
            vec![None]
        } else {
            ability_tags.iter().copied().map(Some).collect()
        };
        for (tag_index, tag) in tags.into_iter().enumerate() {
            let index = u32::try_from(purpose_index * 4 + tag_index)
                .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
            let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)];
            if let Some(tag) = tag {
                filters.push(ModifierFilter::AbilityTag(tag.into()));
            }
            modifiers.push(modifier_definition(
                local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, index)?,
                group,
                StatKind::Hp,
                stage,
                purpose,
                value,
                filters,
            ));
        }
    }
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    first_player_rule(
        binding,
        RuleAttachment::EveryEnemy,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![unique_group(group)],
        modifiers,
        vec![
            EffectDefinition::new(effect, Vec::new(), modifier_ids)
                .with_runtime(permanent_effect(EffectCategory::Debuff)?),
        ],
        vec![
            effect_program(apply, vec![owner], owner, effect, true),
            remove_effect_program(remove, owner, &[effect]),
        ],
        Vec::new(),
        freeze_lifecycle_triggers(raw, owner, apply, remove)?,
    )
}

fn pain_and_suffering(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let activate = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let clear = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let mark = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let consume = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let remove_first = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let remove_second = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 2)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let attackers = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let first = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let second = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 1)?;
    let first_modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let second_modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 1)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let pending = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let charges = whole(parameter(parameters, 0)?)?;
    if !(1..=2).contains(&charges) {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let mut activate_steps = vec![apply_effect_step(owner, first)];
    if charges == 2 {
        activate_steps.push(apply_effect_step(owner, second));
    }
    let activate_program = ProgramDefinition::new(
        activate,
        Vec::new(),
        vec![owner],
        vec![first, second],
        Vec::new(),
    )
    .with_steps(activate_steps);
    let clear_program = remove_effect_program(clear, owner, &[first, second]).with_steps(vec![
        remove_effect_step(owner, first),
        remove_effect_step(owner, second),
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: pending,
            value: ValueExpr::Literal(RuleValue::Integer(0)),
        }),
    ]);
    let mark_program = ProgramDefinition::new(
        mark,
        Vec::new(),
        sorted(vec![owner, attackers]),
        vec![first, second],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::SetSlot {
            slot: pending,
            value: ValueExpr::Literal(RuleValue::Integer(1)),
        },
    )]);
    let consume_program = ProgramDefinition::new(
        consume,
        sorted(vec![remove_first, remove_second]),
        sorted(vec![owner, attackers]),
        vec![first, second],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::If {
            condition: ConditionExpr::EffectExists {
                selector: owner,
                effect: second,
            },
            then_program: remove_second,
            else_program: Some(remove_first),
        },
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: pending,
            value: ValueExpr::Literal(RuleValue::Integer(0)),
        }),
    ]);
    let charge_runtime = permanent_effect(EffectCategory::Mark)?;
    let crit_modifier = |id| {
        modifier_definition(
            id,
            group,
            StatKind::CritRate,
            FormulaStage::Probability,
            FormulaPurpose::CriticalChance,
            1_000_000,
            vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)],
        )
    };
    let mut triggers = freeze_lifecycle_triggers(raw, owner, activate, clear)?;
    triggers.push(trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 4)?,
        RuleEventPoint::DamageApplied,
        OnceScope::TargetWithinAction,
        EventFilter {
            actor_selector: Some(attackers),
            target_selector: Some(owner),
            ability_tag: Some(AbilityTag::Attack),
            damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Ordinary),
            ..EventFilter::default()
        },
        ConditionExpr::Any(
            vec![
                ConditionExpr::EffectExists {
                    selector: owner,
                    effect: first,
                },
                ConditionExpr::EffectExists {
                    selector: owner,
                    effect: second,
                },
            ]
            .into_boxed_slice(),
        ),
        mark,
    ));
    triggers.push(trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 5)?,
        RuleEventPoint::ActionResolved,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(attackers),
            ability_tag: Some(AbilityTag::Attack),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::Slot(pending)),
            operator: Comparison::Greater,
            rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
        },
        consume,
    ));
    first_player_rule(
        binding,
        RuleAttachment::EveryEnemy,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(attackers).with_rule_units(all_enemy_selector()?),
        ],
        vec![unique_group(group)],
        vec![
            crit_modifier(first_modifier),
            crit_modifier(second_modifier),
        ],
        vec![
            EffectDefinition::new(first, Vec::new(), vec![first_modifier])
                .with_runtime(charge_runtime.clone()),
            EffectDefinition::new(second, Vec::new(), vec![second_modifier])
                .with_runtime(charge_runtime),
        ],
        vec![
            activate_program,
            clear_program,
            mark_program,
            consume_program,
            remove_effect_program(remove_first, owner, &[first]),
            remove_effect_program(remove_second, owner, &[second]),
        ],
        vec![
            StateSlotDef::new(
                pending,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1))
            .with_reset_points(vec![SlotResetPoint::WaveStart]),
        ],
        triggers,
    )
}

fn freeze_lifecycle_triggers(
    raw: u32,
    owner: SelectorId,
    apply: ProgramId,
    remove: ProgramId,
) -> Result<Vec<TriggerDef>, BattleRuleLoweringError> {
    let frozen = ConditionExpr::IsFrozen(owner);
    let thawed = ConditionExpr::Not(Box::new(frozen.clone()));
    Ok(vec![
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::EffectApplied,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                effect_category: Some(EffectCategory::Control),
                effect_specific_resistance: Some(StatKind::FreezeResistance),
                ..EventFilter::default()
            },
            frozen.clone(),
            apply,
        ),
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::ToughnessChanged,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                element: Some(CombatElement::Ice),
                toughness_kind: Some(RuleToughnessEventKind::BaseEffectApplied),
                ..EventFilter::default()
            },
            frozen,
            apply,
        ),
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 2)?,
            RuleEventPoint::EffectRemoved,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                effect_category: Some(EffectCategory::Control),
                effect_specific_resistance: Some(StatKind::FreezeResistance),
                ..EventFilter::default()
            },
            thawed.clone(),
            remove,
        ),
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 3)?,
            RuleEventPoint::ToughnessChanged,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                element: Some(CombatElement::Ice),
                toughness_kind: Some(RuleToughnessEventKind::BaseEffectExpired),
                ..EventFilter::default()
            },
            thawed,
            remove,
        ),
    ])
}

fn effect_program(
    program: ProgramId,
    selectors: Vec<SelectorId>,
    selector: SelectorId,
    effect: EffectDefinitionId,
    guaranteed: bool,
) -> ProgramDefinition {
    ProgramDefinition::new(
        program,
        Vec::new(),
        sorted(selectors),
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector,
            effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: if guaranteed {
                RuleEffectChancePolicy::Guaranteed
            } else {
                RuleEffectChancePolicy::Resistible
            },
            base_chance: None,
            rng_purpose: (!guaranteed).then_some(DrawPurpose::EFFECT_CHANCE),
        },
    )])
}

fn remove_effect_program(
    program: ProgramId,
    selector: SelectorId,
    effects: &[EffectDefinitionId],
) -> ProgramDefinition {
    let mut effect_ids = effects.to_vec();
    effect_ids.sort_unstable();
    effect_ids.dedup();
    ProgramDefinition::new(program, Vec::new(), vec![selector], effect_ids, Vec::new()).with_steps(
        effects
            .iter()
            .copied()
            .map(|effect| remove_effect_step(selector, effect))
            .collect(),
    )
}

fn apply_effect_step(selector: SelectorId, effect: EffectDefinitionId) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn remove_effect_step(selector: SelectorId, effect: EffectDefinitionId) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::RemoveEffect { selector, effect })
}

fn freeze_runtime(duration: u16) -> Result<EffectRuntimeDefinition, BattleRuleLoweringError> {
    EffectRuntimeDefinition::new(
        EffectCategory::Control,
        DispelCategory::CleanseableControl,
        1,
        Some(duration),
        DurationClock::TargetTurnStart,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .and_then(|runtime| {
        runtime.with_control(vec![
            ControlledAction::NormalAction,
            ControlledAction::Ultimate,
            ControlledAction::FollowUp,
            ControlledAction::Counter,
            ControlledAction::SummonAction,
        ])
    })
    .map(|runtime| runtime.with_specific_resistance(StatKind::FreezeResistance))
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn permanent_effect(
    category: EffectCategory,
) -> Result<EffectRuntimeDefinition, BattleRuleLoweringError> {
    EffectRuntimeDefinition::new(
        category,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
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

#[allow(clippy::too_many_arguments)]
fn modifier_definition(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: i64,
    filters: Vec<ModifierFilter>,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
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
        filters: filters.into_boxed_slice(),
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

#[allow(clippy::too_many_arguments)]
fn first_player_rule(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    mut selectors: Vec<SelectorDefinition>,
    mut groups: Vec<ModifierStackingGroup>,
    mut modifiers: Vec<ModifierDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    slots: Vec<StateSlotDef>,
    mut triggers: Vec<TriggerDef>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    Ok(ExecutableBattleRule {
        attachment,
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.clone().into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.clone().into_boxed_slice(),
        definition: RuleDefinition::new(
            binding.rule(),
            programs.iter().map(ProgramDefinition::id).collect(),
            selectors.iter().map(SelectorDefinition::id).collect(),
        )
        .with_runtime(BattleRuleDefinition::new(
            binding.source().clone(),
            slots,
            triggers,
            None,
        )),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn sorted<T: Ord>(mut values: Vec<T>) -> Vec<T> {
    values.sort_unstable();
    values.dedup();
    values
}

fn local<T>(base: u32, raw: u32, index: u32) -> Result<T, BattleRuleLoweringError>
where
    T: TryFrom<u32>,
{
    let offset = raw
        .checked_sub(CONTRIBUTION_RULE_ID_BASE)
        .and_then(|value| value.checked_mul(64))
        .and_then(|value| value.checked_add(index))
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    base.checked_add(offset)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}
