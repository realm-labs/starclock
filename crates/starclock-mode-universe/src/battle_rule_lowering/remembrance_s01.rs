use super::*;

const FULI: &str = "StageAbility_612130";
const INNOCENCE: &str = "StageAbility_612131";
const RETICENCE: &str = "StageAbility_612132";
const MELANCHOLIA: &str = "StageAbility_612140";
const DIZZINESS: &str = "StageAbility_612141";

const DISSOCIATION: EffectDefinitionId =
    EffectDefinitionId::new(0x76f0_0001).expect("reserved effect ID");
const FREEZE: EffectDefinitionId =
    EffectDefinitionId::new(0x76f0_0002).expect("reserved effect ID");
const SUPPORT_PROGRAM: ProgramId = ProgramId::new(0x76f0_0003).expect("reserved program ID");
const SUPPORT_SELECTOR: SelectorId = SelectorId::new(0x76f0_0004).expect("reserved selector ID");
const SUPPORT_TRIGGER: TriggerId = TriggerId::new(0x76f0_0005).expect("reserved trigger ID");
const SUPPORT_BODY_PROGRAM: ProgramId = ProgramId::new(0x76f0_0006).expect("reserved program ID");
const DIZZINESS_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x76f0_0010).expect("reserved modifier group ID");
const DIZZINESS_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x76f0_0011).expect("reserved effect ID");
const DIZZINESS_MODIFIER_BASE: u32 = 0x76f0_0020;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    let removal_bonus = selected_level(blessings, FULI)
        .filter(|level| *level == 2)
        .map_or(0, |_| 200_000);
    let mut support_pending = true;

    for key in [FULI, INNOCENCE, MELANCHOLIA, DIZZINESS, RETICENCE] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        let support = support_pending && key != RETICENCE;
        let mut rule = match key {
            FULI => fuli(binding, parameters)?,
            INNOCENCE => innocence(binding, parameters)?,
            MELANCHOLIA => melancholia(binding, parameters, removal_bonus)?,
            DIZZINESS => dizziness(binding, parameters)?,
            RETICENCE => reticence(binding, parameters)?,
            _ => unreachable!("closed Remembrance S01 binding set"),
        };
        if support {
            add_shared_support(&mut rule, removal_bonus)?;
            support_pending = false;
        }
        output.push(rule);
    }
    if support_pending && !output.is_empty() {
        return Err(BattleRuleLoweringError::InvalidDefinition);
    }
    Ok(output)
}

fn fuli(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
        SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
    ];
    let program_definition = apply_effect_program(
        program,
        vec![allies, target],
        target,
        DISSOCIATION,
        parameter(parameters, 0)?,
        RuleEffectChancePolicy::Resistible,
    );
    Ok(first_player_rule(
        binding,
        selectors,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![program_definition],
        Vec::new(),
        vec![with_priority(
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::TargetWithinAction,
                EventFilter {
                    actor_selector: Some(allies),
                    ability_tag: Some(AbilityTag::Attack),
                    damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Ordinary),
                    ..EventFilter::default()
                },
                ConditionExpr::IsFrozen(target),
                program,
            ),
            -10,
        )],
    ))
}

fn innocence(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
        SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
    ];
    Ok(first_player_rule(
        binding,
        selectors,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![apply_effect_program(
            program,
            vec![allies, target],
            target,
            DISSOCIATION,
            parameter(parameters, 0)?,
            if parameter(parameters, 2)? == 0 {
                RuleEffectChancePolicy::Resistible
            } else {
                RuleEffectChancePolicy::ResistibleIgnoringSpecificResistance
            },
        )],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::WeaknessBroken,
            OnceScope::Event,
            EventFilter {
                applier_selector: Some(allies),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    ))
}

fn reticence(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let freeze_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let increment_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let counter = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let threshold = whole(parameter(parameters, 0)?)?;
    if threshold == 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let freeze_program_definition = ProgramDefinition::new(
        freeze_program,
        Vec::new(),
        vec![owner],
        vec![FREEZE],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: FREEZE,
            chance: RuleEffectChancePolicy::Resistible,
            base_chance: Some(scalar(parameter(parameters, 1)?)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        }),
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: counter,
            value: ValueExpr::Literal(RuleValue::Integer(0)),
        }),
    ]);
    let increment_program_definition = ProgramDefinition::new(
        increment_program,
        Vec::new(),
        vec![owner],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::AddSlot {
            slot: counter,
            value: ValueExpr::Literal(RuleValue::Integer(1)),
        },
    )]);
    let before_threshold = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(counter)),
        operator: Comparison::Less,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(
            threshold - 1,
        )))),
    };
    let at_threshold = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(counter)),
        operator: Comparison::GreaterOrEqual,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(
            threshold - 1,
        )))),
    };
    let filter = EventFilter {
        target_selector: Some(owner),
        source_class: Some(starclock_combat::rule::model::SourceClass::Ability),
        ability_tag: Some(AbilityTag::Attack),
        damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Ordinary),
        ..EventFilter::default()
    };
    let mut rule = first_player_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![freeze_program_definition, increment_program_definition],
        vec![
            StateSlotDef::new(
                counter,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(
                RuleValue::Integer(0),
                RuleValue::Integer(i64::from(threshold - 1)),
            ),
        ],
        vec![
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::TargetWithinAction,
                filter.clone(),
                at_threshold,
                freeze_program,
            ),
            trigger(
                id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::TargetWithinAction,
                filter,
                before_threshold,
                increment_program,
            ),
        ],
    );
    rule.attachment = RuleAttachment::EveryEnemy;
    Ok(rule)
}

fn melancholia(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    removal_bonus: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let detonation = parameter(parameters, 0)?;
    let incremental = detonation
        .checked_sub(1_000_000)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let ratio = 300_000_i64
        .checked_mul(
            1_000_000_i64
                .checked_add(removal_bonus)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        )
        .and_then(|value| value.checked_div(1_000_000))
        .and_then(|value| value.checked_mul(incremental))
        .and_then(|value| value.checked_div(1_000_000))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::CurrentTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(ratio),
    );
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        {
            let mut selectors = vec![allies, target];
            selectors.sort_unstable();
            selectors
        },
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::ForEach {
        selector: target,
        body,
        maximum: 1,
    }]);
    let body_definition = ProgramDefinition::new(
        body,
        Vec::new(),
        vec![target],
        vec![DISSOCIATION],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::Damage {
            selector: target,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Ice,
            can_crit: false,
            can_defeat: true,
        }),
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: target,
            effect: DISSOCIATION,
        }),
    ]);
    Ok(first_player_rule(
        binding,
        vec![
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![program_definition, body_definition],
        Vec::new(),
        vec![with_priority(
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::TargetWithinAction,
                EventFilter {
                    actor_selector: Some(allies),
                    ability_tag: Some(AbilityTag::Attack),
                    damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Ordinary),
                    ..EventFilter::default()
                },
                ConditionExpr::EffectExists {
                    selector: target,
                    effect: DISSOCIATION,
                },
                program,
            ),
            -20,
        )],
    ))
}

fn dizziness(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let purposes = [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ];
    let modifier_group = ModifierStackingGroup {
        id: DIZZINESS_GROUP,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    };
    let modifiers = purposes
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| ModifierDefinition {
            id: ModifierDefinitionId::new(
                DIZZINESS_MODIFIER_BASE + u32::try_from(index).expect("bounded purpose count"),
            )
            .expect("reserved modifier ID"),
            stat: StatKind::Hp,
            stage: FormulaStage::Vulnerability,
            purpose,
            value: scalar(parameter(parameters, 0).expect("validated parameter")),
            stacking_group: DIZZINESS_GROUP,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Vulnerability,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                .into_boxed_slice(),
        })
        .collect::<Vec<_>>();
    let modifier_ids = modifiers
        .iter()
        .map(|modifier| modifier.id)
        .collect::<Vec<_>>();
    let runtime = EffectRuntimeDefinition::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(2),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![target],
        vec![DIZZINESS_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: target,
            effect: DIZZINESS_EFFECT,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )]);
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        vec![modifier_group],
        modifiers,
        vec![
            EffectDefinition::new(DIZZINESS_EFFECT, Vec::new(), modifier_ids).with_runtime(runtime),
        ],
        vec![program_definition],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectApplied,
            OnceScope::Event,
            EventFilter {
                effect_definition: Some(DISSOCIATION),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    ))
}

fn add_shared_support(
    rule: &mut ExecutableBattleRule,
    removal_bonus: i64,
) -> Result<(), BattleRuleLoweringError> {
    let freeze = EffectRuntimeDefinition::new(
        EffectCategory::Control,
        DispelCategory::CleanseableControl,
        1,
        Some(1),
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
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let dissociation = EffectRuntimeDefinition::new(
        EffectCategory::Mark,
        DispelCategory::DispellableDebuff,
        1,
        Some(1),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .map(|runtime| runtime.with_specific_resistance(StatKind::FreezeResistance))
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let ratio = 300_000_i64
        .checked_mul(
            1_000_000_i64
                .checked_add(removal_bonus)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        )
        .and_then(|value| value.checked_div(1_000_000))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let program = ProgramDefinition::new(
        SUPPORT_PROGRAM,
        Vec::new(),
        vec![SUPPORT_SELECTOR],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::ForEach {
        selector: SUPPORT_SELECTOR,
        body: SUPPORT_BODY_PROGRAM,
        maximum: 1,
    }]);
    let body = ProgramDefinition::new(
        SUPPORT_BODY_PROGRAM,
        Vec::new(),
        vec![SUPPORT_SELECTOR],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: SUPPORT_SELECTOR,
            amount: multiply(
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::CurrentTarget,
                    stat: StatKind::Hp,
                    purpose: FormulaPurpose::Stat,
                },
                scalar(ratio),
            ),
            class: DamageClass::Additional,
            element: CombatElement::Ice,
            can_crit: false,
            can_defeat: true,
        },
    )]);
    let support_trigger = trigger(
        SUPPORT_TRIGGER,
        RuleEventPoint::EffectRemoved,
        OnceScope::Event,
        EventFilter {
            effect_definition: Some(DISSOCIATION),
            target_selector: Some(SUPPORT_SELECTOR),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        SUPPORT_PROGRAM,
    );

    let mut selectors = rule.selectors.to_vec();
    selectors.push(
        SelectorDefinition::new(SUPPORT_SELECTOR).with_rule_units(primary_target_selector()?),
    );
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    rule.selectors = selectors.into_boxed_slice();
    let mut effects = rule.effects.to_vec();
    effects.extend([
        EffectDefinition::new(DISSOCIATION, Vec::new(), Vec::new()).with_runtime(dissociation),
        EffectDefinition::new(FREEZE, Vec::new(), Vec::new()).with_runtime(freeze),
    ]);
    effects.sort_unstable_by_key(EffectDefinition::id);
    rule.effects = effects.into_boxed_slice();
    let mut programs = rule.programs.to_vec();
    programs.extend([program, body]);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    rule.programs = programs.into_boxed_slice();

    let runtime = rule.definition.runtime().expect("executable rule").clone();
    let mut triggers = runtime.triggers().to_vec();
    triggers.push(support_trigger);
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

fn apply_effect_program(
    program: ProgramId,
    mut selectors: Vec<SelectorId>,
    target: SelectorId,
    effect: EffectDefinitionId,
    base_chance: i64,
    chance: RuleEffectChancePolicy,
) -> ProgramDefinition {
    selectors.sort_unstable();
    ProgramDefinition::new(program, Vec::new(), selectors, vec![effect], Vec::new()).with_steps(
        vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: target,
            effect,
            chance,
            base_chance: Some(scalar(base_chance)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        })],
    )
}

#[allow(clippy::too_many_arguments)]
fn first_player_rule(
    binding: &UniverseBattleRuleBinding,
    mut selectors: Vec<SelectorDefinition>,
    mut modifier_groups: Vec<ModifierStackingGroup>,
    mut modifiers: Vec<ModifierDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    slots: Vec<StateSlotDef>,
    mut triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    modifier_groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = selectors
        .iter()
        .map(SelectorDefinition::id)
        .collect::<Vec<_>>();
    let program_ids = programs
        .iter()
        .map(ProgramDefinition::id)
        .collect::<Vec<_>>();
    ExecutableBattleRule {
        attachment: RuleAttachment::FirstPlayer,
        modifier_groups: modifier_groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), slots, triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}

fn selected_level(blessings: &BlessingContributionSet, key: &str) -> Option<u8> {
    blessings
        .entries()
        .iter()
        .find(|entry| entry.level().source_binding_key() == key)
        .map(|entry| entry.level().level())
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

fn with_priority(mut trigger: TriggerDef, priority: i16) -> TriggerDef {
    trigger.priority = ReactionPriority::new(priority);
    trigger
}
