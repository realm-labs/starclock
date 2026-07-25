use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
    RuleSelectorSide, RuleUnitSelector,
};

const INSENSITIVITY: &str = "StageAbility_612142";
const SENTIMENTALITY: &str = "StageAbility_612143";
const INDELIBILITY: &str = "StageAbility_612144";
const SHUDDER: &str = "StageAbility_612145";
const MAVERICK: &str = "StageAbility_612146";
const UNSPEAKABLE_SHAME: &str = "StageAbility_612150";

const DISSOCIATION: EffectDefinitionId =
    EffectDefinitionId::new(0x76f0_0001).expect("reserved Remembrance effect ID");
const RANDOM_SELECTOR_ID_BASE: u32 = 0x76e1_0000;
const SECOND_EFFECT_ID_BASE: u32 = 0x76e2_0000;

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        INSENSITIVITY,
        SENTIMENTALITY,
        INDELIBILITY,
        SHUDDER,
        MAVERICK,
        UNSPEAKABLE_SHAME,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            INSENSITIVITY => insensitivity(binding, parameters)?,
            SENTIMENTALITY => sentimentality(binding, parameters)?,
            INDELIBILITY => indelibility(binding, parameters)?,
            SHUDDER => shudder(binding, parameters)?,
            MAVERICK => maverick(binding, parameters)?,
            UNSPEAKABLE_SHAME => unspeakable_shame(
                binding,
                parameters,
                remembrance_blessing_count(catalog, blessings)?,
            )?,
            _ => unreachable!("closed Remembrance S02 binding set"),
        });
    }
    Ok(output)
}

fn insensitivity(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let runtime = freeze_runtime(whole(parameter(parameters, 1)?)?)?;
    first_player_rule(
        binding,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        Vec::new(),
        Vec::new(),
        vec![EffectDefinition::new(effect, Vec::new(), Vec::new()).with_runtime(runtime)],
        vec![apply_effect_program(
            program,
            vec![target],
            target,
            effect,
            parameter(parameters, 0)?,
        )],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectRemoved,
            OnceScope::Event,
            EventFilter {
                effect_definition: Some(DISSOCIATION),
                target_selector: Some(target),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn sentimentality(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let damage = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let splash = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let enhanced = whole(parameter(parameters, 1)?)? == 2;
    let splash_selector = if enhanced {
        all_enemy_selector()?
    } else {
        adjacent_enemies_selector()?
    };
    let programs = vec![
        ProgramDefinition::new(
            root,
            Vec::new(),
            vec![splash, allies],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: splash,
            body,
            maximum: 16,
        }]),
        ProgramDefinition::new(body, Vec::new(), vec![current], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::If {
                condition: ConditionExpr::Compare {
                    lhs: Box::new(ValueExpr::CurrentTarget),
                    operator: Comparison::NotEqual,
                    rhs: Box::new(ValueExpr::EventTarget),
                },
                then_program: damage,
                else_program: None,
            }],
        ),
        ProgramDefinition::new(damage, Vec::new(), vec![current], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::Damage {
                    selector: current,
                    amount: multiply(
                        ValueExpr::ReadEventProperty(EventValueProperty::DamageAmount),
                        scalar(parameter(parameters, 0)?),
                    ),
                    class: DamageClass::Additional,
                    element: CombatElement::Ice,
                    can_crit: false,
                    can_defeat: true,
                },
            )]),
    ];
    first_player_rule(
        binding,
        vec![
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(splash).with_rule_units(splash_selector),
            SelectorDefinition::new(current).with_rule_units(current_subject_selector()?),
        ],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        programs,
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(allies),
                element: Some(CombatElement::Ice),
                ..EventFilter::default()
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadEventProperty(
                    EventValueProperty::SourceDefinitionId,
                )),
                operator: Comparison::NotEqual,
                rhs: Box::new(ValueExpr::Literal(RuleValue::OptionalStableId(Some(
                    u64::from(binding.source().definition().get()),
                )))),
            },
            root,
        )],
    )
}

fn indelibility(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let freeze_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let resistance_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let freeze = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let resistance = id::<EffectDefinitionId>(SECOND_EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let reduction = parameter(parameters, 2)?;
    let mut effects = vec![
        EffectDefinition::new(freeze, Vec::new(), Vec::new())
            .with_runtime(freeze_runtime(whole(parameter(parameters, 1)?)?)?),
    ];
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut programs = vec![apply_effect_program(
        freeze_program,
        vec![allies, target],
        target,
        freeze,
        parameter(parameters, 0)?,
    )];
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::DamageApplied,
        OnceScope::Event,
        EventFilter {
            actor_selector: Some(allies),
            target_selector: Some(target),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        freeze_program,
    )];
    if reduction != 0 {
        groups.push(unique_group(group));
        modifiers.push(modifier_definition(
            modifier,
            group,
            StatKind::FreezeResistance,
            FormulaStage::Flat,
            FormulaPurpose::EffectChance,
            reduction
                .checked_neg()
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ));
        effects.push(
            EffectDefinition::new(resistance, Vec::new(), vec![modifier])
                .with_runtime(permanent_debuff()?),
        );
        programs.push(guaranteed_effect_program(
            resistance_program,
            enemies,
            resistance,
        ));
        triggers.push(trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            resistance_program,
        ));
    }
    first_player_rule(
        binding,
        vec![
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        groups,
        modifiers,
        effects,
        programs,
        triggers,
    )
}

fn shudder(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let chance_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let weakness_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let random = id::<SelectorId>(RANDOM_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let marker = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let enhanced = whole(parameter(parameters, 2)?)? == 2;
    let marker_runtime = EffectRuntimeDefinition::new(
        EffectCategory::Mark,
        DispelCategory::NonDispellable,
        1,
        Some(1),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let chance = apply_effect_program(
        chance_program,
        vec![allies, random],
        random,
        marker,
        parameter(parameters, 0)?,
    );
    let weakness = ProgramDefinition::new(
        weakness_program,
        Vec::new(),
        vec![target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::AddWeakness {
            selector: target,
            element: CombatElement::Ice,
            duration_turns: Some(scalar(parameter(parameters, 1)?)),
        },
    )]);
    first_player_rule(
        binding,
        vec![
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(random).with_rule_units(random_enemy_selector(enhanced)?),
            SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
        ],
        Vec::new(),
        Vec::new(),
        vec![EffectDefinition::new(marker, Vec::new(), Vec::new()).with_runtime(marker_runtime)],
        vec![chance, weakness],
        vec![
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::ActionResolved,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(allies),
                    ability_tag: Some(AbilityTag::Ultimate),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                chance_program,
            ),
            trigger(
                id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::EffectApplied,
                OnceScope::Event,
                EventFilter {
                    effect_definition: Some(marker),
                    target_selector: Some(target),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                weakness_program,
            ),
        ],
    )
}

fn maverick(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let speed_reduction = parameter(parameters, 2)?;
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut modifier_ids = Vec::new();
    if speed_reduction != 0 {
        groups.push(unique_group(group));
        modifiers.push(modifier_definition(
            modifier,
            group,
            StatKind::Spd,
            FormulaStage::PercentOfBase,
            FormulaPurpose::Stat,
            speed_reduction
                .checked_neg()
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ));
        modifier_ids.push(modifier);
    }
    let runtime = freeze_runtime(whole(parameter(parameters, 1)?)?)?;
    first_player_rule(
        binding,
        vec![SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?)],
        groups,
        modifiers,
        vec![EffectDefinition::new(effect, Vec::new(), modifier_ids).with_runtime(runtime)],
        vec![apply_effect_program(
            program,
            vec![enemies],
            enemies,
            effect,
            parameter(parameters, 0)?,
        )],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn unspeakable_shame(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    remembrance_count: u16,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let maximum = whole(parameter(parameters, 1)?)?;
    let value = parameter(parameters, 0)?
        .checked_mul(i64::from(remembrance_count.min(maximum)))
        .and_then(i64::checked_neg)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    first_player_rule(
        binding,
        vec![SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?)],
        vec![unique_group(group)],
        vec![modifier_definition(
            modifier,
            group,
            StatKind::FreezeResistance,
            FormulaStage::Flat,
            FormulaPurpose::EffectChance,
            value,
        )],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime(permanent_debuff()?),
        ],
        vec![guaranteed_effect_program(program, enemies, effect)],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn apply_effect_program(
    program: ProgramId,
    mut selectors: Vec<SelectorId>,
    selector: SelectorId,
    effect: EffectDefinitionId,
    base_chance: i64,
) -> ProgramDefinition {
    selectors.sort_unstable();
    selectors.dedup();
    ProgramDefinition::new(program, Vec::new(), selectors, vec![effect], Vec::new()).with_steps(
        vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector,
            effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Resistible,
            base_chance: Some(scalar(base_chance)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        })],
    )
}

fn guaranteed_effect_program(
    program: ProgramId,
    selector: SelectorId,
    effect: EffectDefinitionId,
) -> ProgramDefinition {
    ProgramDefinition::new(
        program,
        Vec::new(),
        vec![selector],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector,
            effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )])
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

fn permanent_debuff() -> Result<EffectRuntimeDefinition, BattleRuleLoweringError> {
    EffectRuntimeDefinition::new(
        EffectCategory::Debuff,
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

fn modifier_definition(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: i64,
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
        filters: Box::new([]),
    }
}

fn adjacent_enemies_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        3,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::PrimaryPlusAdjacent,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn random_enemy_selector(
    without_ice_weakness: bool,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    let selector = RuleUnitSelector::new(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        u16::from(!without_ice_weakness),
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("bounce-target".into()),
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(if without_ice_weakness {
        selector.with_predicates(vec![RuleSelectorPredicate::LacksWeakness(
            CombatElement::Ice,
        )])
    } else {
        selector
    })
}

fn remembrance_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.remembrance")
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

#[allow(clippy::too_many_arguments)]
fn first_player_rule(
    binding: &UniverseBattleRuleBinding,
    mut selectors: Vec<SelectorDefinition>,
    mut groups: Vec<ModifierStackingGroup>,
    mut modifiers: Vec<ModifierDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    mut triggers: Vec<TriggerDef>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::FirstPlayer,
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
            Vec::new(),
            triggers,
            None,
        )),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}
