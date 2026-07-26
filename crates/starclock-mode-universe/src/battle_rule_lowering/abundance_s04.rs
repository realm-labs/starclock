use super::*;
use starclock_combat::{
    ActionGauge, CountdownCatalogDefinition, CountdownDefinition, EffectApplicationGuard,
    EffectDamageGuard, OwnerLinkPolicy, Scalar, Speed, WaveLinkPolicy,
    catalog::selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
    rule::model::{RuleActionOwner, SourceClass},
};

const FORCE_VICTOIRE: &str = "StageAbility_612356";
const EMPOWER: &str = "StageAbility_612357";
pub(super) const RESONANCE: &str = "StageAbility_612320";
const TERMINAL_NIRVANA: &str = "StageAbility_612321";
const ANICCA: &str = "StageAbility_612322";
const ANATTA: &str = "StageAbility_612323";

const AUTO_ABILITY: AbilityId = AbilityId::new(0x7930_0001).expect("reserved ability ID");
const TERMINAL_ABILITY: AbilityId = AbilityId::new(0x7930_0002).expect("reserved ability ID");
const AUTO_ROOT: ProgramId = ProgramId::new(0x7930_0003).expect("reserved program ID");
const AUTO_BODY: ProgramId = ProgramId::new(0x7930_0004).expect("reserved program ID");
const MANUAL_BODY: ProgramId = ProgramId::new(0x7930_0005).expect("reserved program ID");
const CURRENT_ALLY: SelectorId = SelectorId::new(0x7930_0006).expect("reserved selector ID");
const MAX_HP_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7930_0007).expect("reserved effect ID");
const MAX_HP_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7930_0008).expect("reserved modifier ID");
const MAX_HP_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7930_0009).expect("reserved group ID");
const SUBDUING_EVILS_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7930_000a).expect("reserved effect ID");
const ANATTA_COUNTDOWN_CODE: u32 = 0x7930_000b;

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [FORCE_VICTOIRE, EMPOWER] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            FORCE_VICTOIRE => healed_speed(binding, parameters)?,
            EMPOWER => healing_skill_point(binding, parameters)?,
            _ => unreachable!("closed Abundance S04 blessing set"),
        });
    }
    for key in [TERMINAL_NIRVANA, ANICCA, ANATTA] {
        let Some(binding) = resonance_binding(bindings, key) else {
            continue;
        };
        output.push(match key {
            TERMINAL_NIRVANA => terminal_nirvana(binding)?,
            ANICCA => anicca(binding, resonance_parameters(catalog, binding)?)?,
            ANATTA => anatta(binding)?,
            _ => unreachable!("closed Abundance S04 formation set"),
        });
    }
    Ok(output)
}

pub(super) fn resonance(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let parameters = resonance_parameters(catalog, binding)?;
    let heal_ratio = parameter(parameters, 0)?;
    let hp_ratio = parameter(parameters, 1)?;
    let hp_duration = whole(parameter(parameters, 2)?)?;
    let anicca = resonance_binding(bindings, ANICCA).is_some();
    let anatta = resonance_binding(bindings, ANATTA).is_some();

    let max_hp_runtime = timed_buff(hp_duration, EffectStackPolicy::Refresh)?;
    let mut effects = vec![
        EffectDefinition::new(MAX_HP_EFFECT, Vec::new(), vec![MAX_HP_MODIFIER])
            .with_runtime_template(max_hp_runtime),
    ];
    if anicca {
        let subduing_runtime = EffectRuntimeTemplate::new(
            EffectCategory::Buff,
            DispelCategory::NonDispellable,
            whole(parameter(parameters, 6)?)?,
            Some(ValueExpr::Literal(RuleValue::Integer(i64::from(whole(
                parameter(parameters, 4)?,
            )?)))),
            DurationClock::TargetTurnEnd,
            EffectTickPhase::None,
            EffectStackPolicy::RefreshAndAddStacks,
        )
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?
        .with_application_guard(EffectApplicationGuard::NegativeEffectOnce);
        effects.push(
            EffectDefinition::new(SUBDUING_EVILS_EFFECT, Vec::new(), Vec::new())
                .with_runtime_template(subduing_runtime),
        );
    }

    let manual_steps = resonance_body_steps(heal_ratio, anicca, parameters, CURRENT_ALLY)?;
    let automatic_ratio = heal_ratio
        .checked_mul(
            1_000_000_i64
                .checked_sub(parameter(parameters, 7)?)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        )
        .and_then(|value| value.checked_div(1_000_000))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let automatic_steps = resonance_body_steps(automatic_ratio, anicca, parameters, CURRENT_ALLY)?;
    let root = foreach_program(RESONANCE_PROGRAM_ID, MANUAL_BODY);
    let manual_body = body_program(MANUAL_BODY, manual_steps, anicca);
    let auto_root = foreach_program(AUTO_ROOT, AUTO_BODY);
    let auto_body = body_program(AUTO_BODY, automatic_steps, anicca);

    let manual_action = resonance_action(AbilityKind::Ultimate, true)?;
    let manual_ability = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(manual_action)
    .with_programs(vec![
        AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, RESONANCE_PROGRAM_ID)
            .expect("non-zero sequence"),
    ]);
    let automatic_ability =
        AbilityDefinition::new(AUTO_ABILITY, AUTO_ROOT, RESONANCE_SELECTOR_ID, Vec::new())
            .with_action(resonance_action(AbilityKind::Countdown, false)?)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, AUTO_ROOT)
                    .expect("non-zero sequence"),
            ]);
    let terminal_ability = AbilityDefinition::new(
        TERMINAL_ABILITY,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(resonance_action(AbilityKind::ExtraAction, false)?)
    .with_programs(vec![
        AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, RESONANCE_PROGRAM_ID)
            .expect("non-zero sequence"),
    ]);

    let countdowns = if anatta {
        vec![
            CountdownCatalogDefinition::new(
                ANATTA_COUNTDOWN_CODE,
                CountdownDefinition::new(
                    AUTO_ABILITY,
                    ActionGauge::from_scaled(10_000_000_000)
                        .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                    Speed::from_scaled(200_000_000)
                        .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                    OwnerLinkPolicy::Persist,
                    OwnerLinkPolicy::Persist,
                    WaveLinkPolicy::Persist,
                ),
            )
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        ]
    } else {
        Vec::new()
    };
    Ok(ExecutableResonance {
        modifier_groups: vec![ModifierStackingGroup {
            id: MAX_HP_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }]
        .into_boxed_slice(),
        modifiers: vec![ModifierDefinition {
            id: MAX_HP_MODIFIER,
            stat: StatKind::Hp,
            stage: FormulaStage::PercentOfBase,
            purpose: FormulaPurpose::Stat,
            value: scalar(hp_ratio),
            stacking_group: MAX_HP_GROUP,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::PercentOfBase,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        }]
        .into_boxed_slice(),
        selectors: vec![
            SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
                UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::All)
                    .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ),
            SelectorDefinition::new(RESONANCE_ALLY_SELECTOR_ID)
                .with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(CURRENT_ALLY).with_rule_units(current_ally_selector()?),
        ]
        .into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: vec![root, manual_body, auto_root, auto_body].into_boxed_slice(),
        ability: manual_ability,
        auxiliary_abilities: vec![automatic_ability, terminal_ability].into_boxed_slice(),
        countdowns: countdowns.into_boxed_slice(),
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn healed_speed(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            groups: vec![unique_group(group)],
            modifiers: vec![percent_modifier(
                modifier,
                group,
                StatKind::Spd,
                parameter(parameters, 0)?,
            )],
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier]).with_runtime_template(
                    timed_buff(
                        whole(parameter(parameters, 1)?)?,
                        EffectStackPolicy::Refresh,
                    )?,
                ),
            ],
            programs: vec![apply_effect_program(program, owner, effect)],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::HealApplied,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(owner),
                    ..EventFilter::default()
                },
                positive_healing(),
                program,
            )],
        },
    )
}

fn healing_skill_point(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let chance_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let resource_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let chance_effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let chance_runtime = EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(1))),
        DurationClock::ActionEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let chance = ProgramDefinition::new(
        chance_program,
        Vec::new(),
        vec![owner],
        vec![chance_effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: chance_effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Fixed,
            base_chance: Some(scalar(parameter(parameters, 0)?)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        },
    )]);
    let resource = ProgramDefinition::new(
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
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(chance_effect, Vec::new(), Vec::new())
                    .with_runtime_template(chance_runtime),
            ],
            programs: vec![chance, resource],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HealApplied,
                    OnceScope::Action,
                    EventFilter {
                        applier_selector: Some(owner),
                        source_class: Some(SourceClass::Ability),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    positive_healing(),
                    chance_program,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::EffectApplied,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        effect_definition: Some(chance_effect),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    resource_program,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn terminal_nirvana(
    binding: &UniverseBattleRuleBinding,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let setup = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let activate = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Field,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_damage_guard(EffectDamageGuard::TeamDefeatOnce);
    let setup_program = apply_effect_program(setup, allies, effect);
    let activate_program = ProgramDefinition::new(
        activate,
        Vec::new(),
        vec![owner, allies],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            selector: owner,
            resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
            update: ResourceUpdateKind::Set,
            amount: scalar(0),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        }),
        ProgramStep::Operation(RuleOperationTemplate::QueueAction {
            actor_selector: owner,
            target_selector: allies,
            ability: TERMINAL_ABILITY,
            priority: ReactionPriority::new(-100),
            forced_use: true,
            boundary: starclock_combat::catalog::action::ReactionBoundary::AfterHit,
            owner: RuleActionOwner::Actor,
            payment: None,
        }),
    ]);
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), Vec::new())
                    .with_runtime_template(runtime),
            ],
            programs: vec![setup_program, activate_program],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    ConditionExpr::Literal(true),
                    setup,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::EffectRemoved,
                    OnceScope::Battle,
                    EventFilter {
                        effect_definition: Some(effect),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    activate,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn anicca(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let signal = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::RuleSignalCode,
        )),
        operator: Comparison::Equal,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(
            starclock_combat::NEGATIVE_EFFECT_GUARDED_SIGNAL,
        )))),
    };
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                        selector: owner,
                        amount: maximum_hp(StatQuerySubject::Owner, parameter(parameters, 5)?),
                        apply_formula_modifiers: true,
                    })]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::InformationalRule,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(owner),
                    ..EventFilter::default()
                },
                signal,
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn anatta(
    binding: &UniverseBattleRuleBinding,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let source = ValueExpr::ReadEventProperty(EventValueProperty::SourceDefinitionId);
    let condition = ConditionExpr::Any(
        [RESONANCE_ABILITY_ID, TERMINAL_ABILITY]
            .into_iter()
            .map(|ability| ConditionExpr::Compare {
                lhs: Box::new(source.clone()),
                operator: Comparison::Equal,
                rhs: Box::new(ValueExpr::Literal(RuleValue::OptionalStableId(Some(
                    u64::from(ability.get()),
                )))),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), Vec::new(), Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::CreateCountdown {
                            code: ANATTA_COUNTDOWN_CODE,
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::ActionResolved,
                OnceScope::Battle,
                EventFilter::default(),
                condition,
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn resonance_body_steps(
    heal_ratio: i64,
    anicca: bool,
    parameters: &[ExactParameter],
    target: SelectorId,
) -> Result<Vec<ProgramStep>, BattleRuleLoweringError> {
    let mut steps = vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: target,
            effect: MAX_HP_EFFECT,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::Heal {
            selector: target,
            amount: maximum_hp(StatQuerySubject::CurrentTarget, heal_ratio),
            apply_formula_modifiers: true,
        }),
    ];
    if anicca {
        steps.push(ProgramStep::Operation(RuleOperationTemplate::Cleanse {
            selector: target,
            maximum: u16::MAX,
            order: starclock_combat::EffectRemovalOrder::OldestFirst,
        }));
        steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: target,
            effect: SUBDUING_EVILS_EFFECT,
            stacks: ValueExpr::Literal(RuleValue::Integer(i64::from(whole(parameter(
                parameters, 3,
            )?)?))),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }));
    }
    Ok(steps)
}

fn resonance_action(
    kind: AbilityKind,
    consumes_energy: bool,
) -> Result<AbilityActionDefinition, BattleRuleLoweringError> {
    let mut resources = ActionResourcePolicy::new(
        0,
        0,
        starclock_combat::Energy::ZERO,
        starclock_combat::Energy::ZERO,
    );
    if consumes_energy {
        resources = resources
            .with_team_resource_costs(vec![
                TeamResourceCost::new(RESONANCE_RESOURCE_KEY, 100)
                    .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ])
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    }
    AbilityActionDefinition::new(kind, 1, TargetInvalidationPolicy::KeepIfPresent, resources)
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?
        .with_tags(&[AbilityTag::Assist, AbilityTag::PathResonance])
        .with_hits(vec![ActionHitDefinition::new(Vec::new()).with_profile(
            starclock_combat::catalog::action::HitTargetGroup::Selected,
            Ratio::ONE,
            Ratio::ONE,
            HitCritPolicy::Never,
        )])
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn foreach_program(id: ProgramId, body: ProgramId) -> ProgramDefinition {
    ProgramDefinition::new(
        id,
        Vec::new(),
        vec![RESONANCE_ALLY_SELECTOR_ID],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::ForEach {
        selector: RESONANCE_ALLY_SELECTOR_ID,
        body,
        maximum: 16,
    }])
}

fn body_program(id: ProgramId, steps: Vec<ProgramStep>, anicca: bool) -> ProgramDefinition {
    ProgramDefinition::new(
        id,
        Vec::new(),
        vec![CURRENT_ALLY],
        if anicca {
            vec![MAX_HP_EFFECT, SUBDUING_EVILS_EFFECT]
        } else {
            vec![MAX_HP_EFFECT]
        },
        Vec::new(),
    )
    .with_steps(steps)
}

fn current_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::CurrentSubject,
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

fn timed_buff(
    duration: u16,
    stacking: EffectStackPolicy,
) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(duration)))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        stacking,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn apply_effect_program(
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

fn percent_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    value: i64,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: scalar(value),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    }
}

fn unique_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    }
}

fn maximum_hp(subject: StatQuerySubject, ratio: i64) -> ValueExpr {
    multiply(
        ValueExpr::QueryStat {
            subject,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(ratio),
    )
}

fn positive_healing() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::HpChangeAmount,
        )),
        operator: Comparison::Greater,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(Scalar::ZERO))),
    }
}

fn resonance_binding<'a>(
    bindings: &'a [UniverseBattleRuleBinding],
    key: &str,
) -> Option<&'a UniverseBattleRuleBinding> {
    bindings.iter().find(|binding| {
        matches!(
            binding.role(),
            UniverseBattleRuleRole::Resonance | UniverseBattleRuleRole::Formation
        ) && binding.source_binding_key() == Some(key)
    })
}

fn resonance_parameters<'a>(
    catalog: &'a UniverseCatalog,
    binding: &UniverseBattleRuleBinding,
) -> Result<&'a [ExactParameter], BattleRuleLoweringError> {
    catalog
        .resonances()
        .iter()
        .find(|definition| definition.stable_key() == binding.source_record_key())
        .map(crate::path::ResonanceDefinition::parameters)
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(
        value
            .checked_div(1_000_000)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
    )
    .map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

#[derive(Default)]
struct RuleParts {
    groups: Vec<ModifierStackingGroup>,
    modifiers: Vec<ModifierDefinition>,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
}

fn finish(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
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
        BattleRuleDefinition::new(binding.source().clone(), Vec::new(), parts.triggers, None),
    );
    Ok(ExecutableBattleRule {
        attachment,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
