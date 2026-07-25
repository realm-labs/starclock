use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const FUNERAL_BASE: &str = "StageAbility_612230";
const FUNERAL_ENHANCED: &str = "StageAbility_612230_2";
const MAN_IN_COVER: &str = "StageAbility_612231";
const EVERYTHING_DISAPPEARED: &str = "StageAbility_612232";
const BEGINNING_AND_END: &str = "StageAbility_612240";
const CAFE_SELF_DECEIT: &str = "StageAbility_612241";
const CALL_OF_WILDERNESS: &str = "StageAbility_612242";

const SUSPICION: EffectDefinitionId =
    EffectDefinitionId::new(0x77e0_0001).expect("reserved effect ID");
const SUPPORT_DECAY_PROGRAM: ProgramId = ProgramId::new(0x77e0_0002).expect("reserved program ID");
const SUPPORT_CLEANUP_PROGRAM: ProgramId =
    ProgramId::new(0x77e0_0003).expect("reserved program ID");
const SUPPORT_TARGET: SelectorId = SelectorId::new(0x77e0_0004).expect("reserved selector ID");
const SUPPORT_DEFEATED_TARGET: SelectorId =
    SelectorId::new(0x77e0_0005).expect("reserved selector ID");
const SUPPORT_DECAY_TRIGGER: TriggerId = TriggerId::new(0x77e0_0006).expect("reserved trigger ID");
const SUPPORT_CLEANUP_TRIGGER: TriggerId =
    TriggerId::new(0x77e0_0007).expect("reserved trigger ID");
const SUSPICION_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x77e0_0008).expect("reserved slot ID");
const SUSPICION_VULNERABILITY_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x77e0_0009).expect("reserved modifier group ID");
const SUSPICION_VULNERABILITY: ModifierDefinitionId =
    ModifierDefinitionId::new(0x77e0_000a).expect("reserved modifier ID");
const WILDERNESS_ATTACK_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x77e0_000b).expect("reserved modifier group ID");
const WILDERNESS_RESISTANCE_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x77e0_000c).expect("reserved modifier group ID");
const WILDERNESS_ATTACK: ModifierDefinitionId =
    ModifierDefinitionId::new(0x77e0_000d).expect("reserved modifier ID");
const WILDERNESS_RESISTANCE: ModifierDefinitionId =
    ModifierDefinitionId::new(0x77e0_000e).expect("reserved modifier ID");

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        FUNERAL_BASE,
        FUNERAL_ENHANCED,
        MAN_IN_COVER,
        EVERYTHING_DISAPPEARED,
        BEGINNING_AND_END,
        CAFE_SELF_DECEIT,
        CALL_OF_WILDERNESS,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            FUNERAL_BASE | FUNERAL_ENHANCED => funeral(binding, key)?,
            MAN_IN_COVER => man_in_cover(binding, parameters, selected_level(blessings, key)?)?,
            EVERYTHING_DISAPPEARED => everything_disappeared(binding, parameters)?,
            BEGINNING_AND_END => beginning_and_end(binding, parameters)?,
            CAFE_SELF_DECEIT => cafe_self_deceit(binding, parameters)?,
            CALL_OF_WILDERNESS => call_of_wilderness(binding, parameters)?,
            _ => unreachable!("closed Nihility S01 binding set"),
        });
    }
    if let Some(first) = output.first_mut() {
        let persistent = level_binding(bindings, FUNERAL_ENHANCED).is_some();
        let wilderness = level_binding(bindings, CALL_OF_WILDERNESS).is_some();
        add_shared_suspicion(first, persistent, wilderness)?;
    }
    Ok(output)
}

fn funeral(
    binding: &UniverseBattleRuleBinding,
    key: &str,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![apply_suspicion_program(
            program,
            target,
            ValueExpr::Literal(RuleValue::Integer(1)),
        )],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(target),
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Dot),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(key == FUNERAL_BASE || key == FUNERAL_ENHANCED),
            program,
        )],
    ))
}

fn man_in_cover(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    level: u8,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let apply_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let refresh_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let mut programs = vec![apply_suspicion_program(
        apply_program,
        target,
        integer(parameter(parameters, 0)?)?,
    )];
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::EffectApplied,
        OnceScope::Event,
        EventFilter {
            target_selector: Some(target),
            effect_category: Some(EffectCategory::Dot),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        apply_program,
    )];
    if level == 2 {
        programs.push(apply_suspicion_program(
            refresh_program,
            target,
            integer(parameter(parameters, 1)?)?,
        ));
        triggers.extend([
            trigger(
                id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::EffectRefreshed,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(target),
                    effect_category: Some(EffectCategory::Dot),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                refresh_program,
            ),
            trigger(
                id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::EffectStacksChanged,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(target),
                    effect_category: Some(EffectCategory::Dot),
                    ..EventFilter::default()
                },
                positive_stack_delta(),
                refresh_program,
            ),
        ]);
    }
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        programs,
        Vec::new(),
        triggers,
    ))
}

fn everything_disappeared(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![target], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DetonateDot {
                    selector: target,
                    fraction: scalar(parameter(parameters, 1)?),
                    required_tag: None,
                },
            )]);
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(target).with_rule_units(enemy_owner_selector()?)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![program_definition],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            OnceScope::Turn,
            EventFilter {
                owner_selector: Some(target),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    ))
}

fn beginning_and_end(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let defeated = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let random = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let maximum = whole(parameter(parameters, 0)?)?;
    let root = ProgramDefinition::new(
        program,
        vec![body],
        vec![defeated, random],
        vec![SUSPICION],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::ForEach {
        selector: random,
        body,
        maximum,
    }]);
    let body = apply_suspicion_program(
        body,
        random,
        ValueExpr::QueryEffectStacks {
            subject: StatQuerySubject::EventTarget,
            effect: SUSPICION,
        },
    );
    Ok(first_player_rule(
        binding,
        vec![
            SelectorDefinition::new(defeated).with_rule_units(defeated_target_selector()?),
            SelectorDefinition::new(random).with_rule_units(random_other_enemies(maximum)?),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![root, body],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::UnitDefeated,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(defeated),
                ..EventFilter::default()
            },
            ConditionExpr::EffectExists {
                selector: defeated,
                effect: SUSPICION,
            },
            program,
        )],
    ))
}

fn cafe_self_deceit(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let extra = whole(parameter(parameters, 0)?)?;
    let doubles = parameter(parameters, 1)? == 1_000_000;
    let stacks = if doubles {
        ValueExpr::ReadEventProperty(EventValueProperty::StackDelta)
    } else {
        ValueExpr::Literal(RuleValue::Integer(i64::from(extra)))
    };
    let program_definition = apply_suspicion_program(program, target, stacks);
    let filter = EventFilter {
        target_selector: Some(target),
        effect_definition: Some(SUSPICION),
        excluded_source: Some(binding.source().definition()),
        ..EventFilter::default()
    };
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![program_definition],
        Vec::new(),
        vec![
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::EffectApplied,
                OnceScope::Event,
                filter.clone(),
                positive_stack_delta(),
                program,
            ),
            trigger(
                id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::EffectStacksChanged,
                OnceScope::Event,
                filter,
                positive_stack_delta(),
                program,
            ),
        ],
    ))
}

fn call_of_wilderness(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let attack_ratio = rounded_parameter(parameters, 0)?;
    let attack_cap = rounded_parameter(parameters, 1)?;
    let resistance_ratio = rounded_parameter(parameters, 2)?;
    let resistance_cap = rounded_parameter(parameters, 3)?;
    let groups = vec![
        ModifierStackingGroup {
            id: WILDERNESS_ATTACK_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        },
        ModifierStackingGroup {
            id: WILDERNESS_RESISTANCE_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        },
    ];
    let modifiers = vec![
        stack_modifier(
            WILDERNESS_ATTACK,
            WILDERNESS_ATTACK_GROUP,
            StatKind::Atk,
            FormulaStage::PercentOfBase,
            attack_ratio,
            attack_cap,
        ),
        stack_modifier(
            WILDERNESS_RESISTANCE,
            WILDERNESS_RESISTANCE_GROUP,
            StatKind::EffectResistance,
            FormulaStage::Flat,
            resistance_ratio,
            resistance_cap,
        ),
    ];
    Ok(first_player_rule(
        binding,
        Vec::new(),
        groups,
        modifiers,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

fn add_shared_suspicion(
    rule: &mut ExecutableBattleRule,
    persistent: bool,
    wilderness: bool,
) -> Result<(), BattleRuleLoweringError> {
    let mut modifier_ids = vec![SUSPICION_VULNERABILITY];
    if wilderness {
        modifier_ids.extend([WILDERNESS_ATTACK, WILDERNESS_RESISTANCE]);
    }
    modifier_ids.sort_unstable();
    modifier_ids.dedup();
    let runtime = EffectRuntimeDefinition::new(
        EffectCategory::Debuff,
        DispelCategory::NonDispellable,
        99,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_teardown(starclock_combat::EffectTeardownPolicy::PersistByScope);
    let effect = EffectDefinition::new(SUSPICION, Vec::new(), modifier_ids).with_runtime(runtime);
    let vulnerability_group = ModifierStackingGroup {
        id: SUSPICION_VULNERABILITY_GROUP,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    };
    let vulnerability = ModifierDefinition {
        id: SUSPICION_VULNERABILITY,
        stat: StatKind::Hp,
        stage: FormulaStage::Vulnerability,
        purpose: FormulaPurpose::Dot,
        value: stack_ratio(10_000, None),
        stacking_group: SUSPICION_VULNERABILITY_GROUP,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Vulnerability,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: Some(SUSPICION_STACK_SLOT),
        filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)].into_boxed_slice(),
    };
    let mut selectors = rule.selectors.to_vec();
    selectors.extend([
        SelectorDefinition::new(SUPPORT_TARGET).with_rule_units(enemy_owner_selector()?),
        SelectorDefinition::new(SUPPORT_DEFEATED_TARGET)
            .with_rule_units(defeated_target_selector()?),
    ]);
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    selectors.dedup_by_key(|selector| selector.id());
    rule.selectors = selectors.into_boxed_slice();
    let mut groups = rule.modifier_groups.to_vec();
    groups.push(vulnerability_group);
    groups.sort_unstable_by_key(|group| group.id);
    groups.dedup_by_key(|group| group.id);
    rule.modifier_groups = groups.into_boxed_slice();
    let mut modifiers = rule.modifiers.to_vec();
    modifiers.push(vulnerability);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    modifiers.dedup_by_key(|modifier| modifier.id);
    rule.modifiers = modifiers.into_boxed_slice();
    let mut effects = rule.effects.to_vec();
    effects.push(effect);
    effects.sort_unstable_by_key(EffectDefinition::id);
    rule.effects = effects.into_boxed_slice();
    let mut programs = rule.programs.to_vec();
    if !persistent {
        programs.push(
            ProgramDefinition::new(
                SUPPORT_DECAY_PROGRAM,
                Vec::new(),
                vec![SUPPORT_TARGET],
                vec![SUSPICION],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AdjustEffectStacks {
                    selector: SUPPORT_TARGET,
                    effect: SUSPICION,
                    delta: ValueExpr::Literal(RuleValue::Integer(-2)),
                },
            )]),
        );
    }
    programs.push(
        ProgramDefinition::new(
            SUPPORT_CLEANUP_PROGRAM,
            Vec::new(),
            vec![SUPPORT_DEFEATED_TARGET],
            vec![SUSPICION],
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::RemoveEffect {
                selector: SUPPORT_DEFEATED_TARGET,
                effect: SUSPICION,
            },
        )]),
    );
    programs.sort_unstable_by_key(ProgramDefinition::id);
    rule.programs = programs.into_boxed_slice();
    let runtime = rule.definition.runtime().expect("executable rule").clone();
    let mut triggers = runtime.triggers().to_vec();
    if !persistent {
        triggers.push(with_priority(
            trigger(
                SUPPORT_DECAY_TRIGGER,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(SUPPORT_TARGET),
                    ..EventFilter::default()
                },
                ConditionExpr::EffectExists {
                    selector: SUPPORT_TARGET,
                    effect: SUSPICION,
                },
                SUPPORT_DECAY_PROGRAM,
            ),
            50,
        ));
    }
    triggers.push(with_priority(
        trigger(
            SUPPORT_CLEANUP_TRIGGER,
            RuleEventPoint::UnitDefeated,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(SUPPORT_DEFEATED_TARGET),
                ..EventFilter::default()
            },
            ConditionExpr::EffectExists {
                selector: SUPPORT_DEFEATED_TARGET,
                effect: SUSPICION,
            },
            SUPPORT_CLEANUP_PROGRAM,
        ),
        100,
    ));
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    rule.definition = RuleDefinition::new(
        rule.definition.id(),
        rule.programs.iter().map(ProgramDefinition::id).collect(),
        rule.selectors.iter().map(SelectorDefinition::id).collect(),
    )
    .with_runtime(BattleRuleDefinition::new(
        runtime.source().clone(),
        runtime.state_slots().to_vec(),
        triggers,
        None,
    ));
    Ok(())
}

fn apply_suspicion_program(
    program: ProgramId,
    selector: SelectorId,
    stacks: ValueExpr,
) -> ProgramDefinition {
    ProgramDefinition::new(
        program,
        Vec::new(),
        vec![selector],
        vec![SUSPICION],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector,
            effect: SUSPICION,
            stacks,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )])
}

fn stack_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    stage: FormulaStage,
    ratio: i64,
    cap: i64,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat,
        stage,
        purpose: FormulaPurpose::Stat,
        value: stack_ratio(-ratio, Some(-cap)),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: stage,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: Some(SUSPICION_STACK_SLOT),
        filters: Box::new([]),
    }
}

fn stack_ratio(ratio: i64, minimum: Option<i64>) -> ValueExpr {
    let value = multiply(
        ValueExpr::Convert {
            value: Box::new(ValueExpr::Slot(SUSPICION_STACK_SLOT)),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        },
        scalar(ratio),
    );
    minimum.map_or(value.clone(), |minimum| ValueExpr::Clamp {
        value: Box::new(value),
        minimum: Box::new(scalar(minimum)),
        maximum: Box::new(scalar(0)),
    })
}

fn positive_stack_delta() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::StackDelta)),
        operator: Comparison::Greater,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(0))),
    }
}

fn defeated_target_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Defeated,
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

fn enemy_owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
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

fn random_other_enemies(maximum: u16) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        1,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("behavior-choice".into()),
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
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
    let selector_ids = selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = programs.iter().map(ProgramDefinition::id).collect();
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

fn selected_level(
    blessings: &BlessingContributionSet,
    key: &str,
) -> Result<u8, BattleRuleLoweringError> {
    blessings
        .entries()
        .iter()
        .find(|entry| entry.level().source_binding_key() == key)
        .map(|entry| entry.level().level())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)
}

fn integer(value: i64) -> Result<ValueExpr, BattleRuleLoweringError> {
    if value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    Ok(ValueExpr::Literal(RuleValue::Integer(value / 1_000_000)))
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    if value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

fn rounded_parameter(
    parameters: &[ExactParameter],
    index: usize,
) -> Result<i64, BattleRuleLoweringError> {
    let exact = parameters
        .get(index)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    if exact.scale() <= 6 {
        return parameter(parameters, index);
    }
    let divisor = 10_i64
        .checked_pow(u32::from(exact.scale() - 6))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let quotient = exact.coefficient() / divisor;
    let remainder = exact.coefficient() % divisor;
    let half = divisor / 2;
    let increment = remainder.abs() > half || remainder.abs() == half && quotient % 2 != 0;
    quotient
        .checked_add(if increment {
            exact.coefficient().signum()
        } else {
            0
        })
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}

fn with_priority(mut trigger: TriggerDef, priority: i16) -> TriggerDef {
    trigger.priority = ReactionPriority::new(priority);
    trigger
}
