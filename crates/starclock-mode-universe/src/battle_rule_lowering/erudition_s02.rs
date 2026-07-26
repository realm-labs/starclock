use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const TACTILE_PATHWAY: &str = "StageAbility_612843";
const SUBLIMINAL_SENSATION: &str = "StageAbility_612844";
const STRIATED_CORTEX: &str = "StageAbility_612845";
const SALTATORY_CONDUCTION: &str = "StageAbility_612846";
const ENGAGED_GEARS: &str = "StageAbility_612850";

const LOCAL_PROGRAM_BASE: u32 = 0x7e00_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7e10_0000;
const LOCAL_EFFECT_BASE: u32 = 0x7e20_0000;
const LOCAL_SLOT_BASE: u32 = 0x7e30_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7e40_0000;
const LOCAL_GROUP_BASE: u32 = 0x7e50_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7e60_0000;

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        TACTILE_PATHWAY,
        SUBLIMINAL_SENSATION,
        STRIATED_CORTEX,
        SALTATORY_CONDUCTION,
        ENGAGED_GEARS,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            TACTILE_PATHWAY => tactile_pathway(binding, parameters)?,
            SUBLIMINAL_SENSATION => subliminal_sensation(binding, parameters)?,
            STRIATED_CORTEX => striated_cortex(binding, parameters)?,
            SALTATORY_CONDUCTION => saltatory_conduction(binding, parameters)?,
            ENGAGED_GEARS => engaged_gears(catalog, blessings, binding, parameters)?,
            _ => unreachable!("closed Erudition S02 binding set"),
        });
    }
    Ok(output)
}

fn tactile_pathway(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let action_targets = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let defeated_target = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 2)?;
    let damage = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let remember_defeat = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let defeated = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let counts_defeated = parameter_six(parameters, 1)? > 0;
    let total_targets = ValueExpr::Minimum(
        Box::new(ValueExpr::Add(
            Box::new(ValueExpr::SelectorCount(action_targets)),
            Box::new(ValueExpr::Slot(defeated)),
        )),
        Box::new(integer(5)),
    );
    let target_ratio = multiply(
        scalar(parameter_six(parameters, 0)?),
        ValueExpr::Convert {
            value: Box::new(total_targets),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        },
    );
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Actor,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::AdditionalDamage,
        },
        target_ratio,
    );
    let mut programs = vec![
        ProgramDefinition::new(
            damage,
            Vec::new(),
            vec![owner, action_targets],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::DamageFromEventElement {
                selector: action_targets,
                amount,
                class: DamageClass::Additional,
                can_crit: true,
                can_defeat: true,
            },
        )]),
    ];
    let mut triggers = vec![trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        RuleEventPoint::ActionResolved,
        TriggerPhase::AfterAction,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(AbilityTag::Attack),
            excluded_source: Some(binding.source().definition()),
            ..EventFilter::default()
        },
        ConditionExpr::SelectorCardinality {
            selector: action_targets,
            operator: Comparison::GreaterOrEqual,
            count: 1,
        },
        damage,
    )];
    if counts_defeated {
        programs.push(
            ProgramDefinition::new(
                remember_defeat,
                Vec::new(),
                vec![defeated_target],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddSlot {
                    slot: defeated,
                    value: integer(1),
                },
            )]),
        );
        triggers.push(trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::UnitDefeated,
            TriggerPhase::AfterDefeatSettlement,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(defeated_target),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            remember_defeat,
        ));
    }
    Ok(finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(action_targets).with_rule_units(action_targets_selector()?),
                SelectorDefinition::new(defeated_target)
                    .with_rule_units(defeated_enemy_selector()?),
            ],
            programs,
            slots: vec![
                StateSlotDef::new(
                    defeated,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(5)),
            ],
            triggers,
            ..RuleParts::default()
        },
    ))
}

fn subliminal_sensation(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let entry = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let consume = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let persistent = parameter_six(parameters, 2)? > 0;
    let entry_program =
        ProgramDefinition::new(entry, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: integer(1),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Gain,
                    amount: multiply(
                        ValueExpr::QueryMaximumEnergy(StatQuerySubject::Owner),
                        scalar(parameter_six(parameters, 1)?),
                    ),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                }),
            ]);
    let mut programs = vec![entry_program];
    let mut triggers = vec![trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        RuleEventPoint::BattleStarted,
        TriggerPhase::AfterEvent,
        OnceScope::Battle,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        entry,
    )];
    if !persistent {
        programs.push(
            ProgramDefinition::new(consume, Vec::new(), vec![owner], vec![effect], Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::RemoveEffect {
                        selector: owner,
                        effect,
                    },
                )]),
        );
        triggers.push(trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                ..EventFilter::default()
            },
            ConditionExpr::EffectExists {
                selector: owner,
                effect,
            },
            consume,
        ));
    }
    let modifier_definition = ModifierDefinition {
        id: modifier,
        stat: StatKind::Atk,
        stage: FormulaStage::DamageBoost,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: scalar(parameter_six(parameters, 0)?),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::DamageBoost,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: ultimate_source_filters(),
    };
    Ok(finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            groups: vec![unique_group(group)],
            modifiers: vec![modifier_definition],
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier])
                    .with_runtime_template(permanent_buff()?),
            ],
            programs,
            triggers,
            ..RuleParts::default()
        },
    ))
}

fn striated_cortex(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let target = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let accumulate = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let discharge = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let original_damage = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let accumulate_program =
        ProgramDefinition::new(accumulate, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddSlot {
                    slot: original_damage,
                    value: ValueExpr::ReadEventProperty(EventValueProperty::DamageRawAmount),
                },
            )]);
    let discharge_program = ProgramDefinition::new(
        discharge,
        Vec::new(),
        vec![owner, target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::TrueDamage {
            selector: target,
            amount: multiply(
                ValueExpr::Slot(original_damage),
                scalar(parameter_six(parameters, 0)?),
            ),
        },
    )]);
    Ok(finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(target).with_rule_units(action_targets_selector()?),
            ],
            programs: vec![accumulate_program, discharge_program],
            slots: vec![
                StateSlotDef::new(
                    original_damage,
                    RuleValueKind::Scalar,
                    BattleRuleScope::Action,
                    RuleValue::Scalar(starclock_combat::Scalar::ZERO),
                )
                .with_reset_points(vec![SlotResetPoint::ActionStart]),
            ],
            triggers: vec![
                trigger(
                    local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
                    RuleEventPoint::DamageApplied,
                    TriggerPhase::AfterEvent,
                    OnceScope::Event,
                    EventFilter {
                        actor_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        source_class: Some(SourceClass::Ability),
                        damage_class: Some(
                            starclock_combat::rule::model::RuleDamageClass::Ordinary,
                        ),
                        has_action: Some(true),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    accumulate,
                ),
                trigger(
                    local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                    RuleEventPoint::ActionResolved,
                    TriggerPhase::AfterAction,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(owner),
                        target_pattern: Some(TargetPattern::All),
                        ..EventFilter::default()
                    },
                    ConditionExpr::All(
                        vec![
                            ConditionExpr::SelectorCardinality {
                                selector: target,
                                operator: Comparison::Equal,
                                count: 1,
                            },
                            ConditionExpr::Compare {
                                lhs: Box::new(ValueExpr::Slot(original_damage)),
                                operator: Comparison::Greater,
                                rhs: Box::new(scalar(0)),
                            },
                        ]
                        .into_boxed_slice(),
                    ),
                    discharge,
                ),
            ],
            ..RuleParts::default()
        },
    ))
}

fn saltatory_conduction(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let attackers = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let reset = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let delay = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let count = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let maximum = whole(parameter_six(parameters, 1)?)?;
    let reset_program =
        ProgramDefinition::new(reset, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: count,
                value: integer(0),
            })],
        );
    let delay_program = ProgramDefinition::new(
        delay,
        Vec::new(),
        vec![owner, attackers],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::DelayAction {
            selector: owner,
            amount: scalar(parameter_six(parameters, 0)?),
        }),
        ProgramStep::Operation(RuleOperationTemplate::AddSlot {
            slot: count,
            value: integer(1),
        }),
    ]);
    Ok(finish(
        binding,
        RuleAttachment::EveryEnemy,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(attackers).with_rule_units(opposing_actors_selector()?),
            ],
            programs: vec![reset_program, delay_program],
            slots: vec![
                StateSlotDef::new(
                    count,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )
                .with_bounds(
                    RuleValue::Integer(0),
                    RuleValue::Integer(i64::from(maximum)),
                ),
            ],
            triggers: vec![
                trigger(
                    local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
                    RuleEventPoint::WeaknessBroken,
                    TriggerPhase::AfterEvent,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    reset,
                ),
                trigger(
                    local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                    RuleEventPoint::DamageApplied,
                    TriggerPhase::AfterEvent,
                    OnceScope::TargetWithinAction,
                    EventFilter {
                        actor_selector: Some(attackers),
                        target_selector: Some(owner),
                        action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                        damage_class: Some(
                            starclock_combat::rule::model::RuleDamageClass::Ordinary,
                        ),
                        ..EventFilter::default()
                    },
                    ConditionExpr::All(
                        vec![
                            ConditionExpr::IsBroken(owner),
                            ConditionExpr::Compare {
                                lhs: Box::new(ValueExpr::Slot(count)),
                                operator: Comparison::Less,
                                rhs: Box::new(integer(i64::from(maximum))),
                            },
                        ]
                        .into_boxed_slice(),
                    ),
                    delay,
                ),
            ],
            ..RuleParts::default()
        },
    ))
}

fn engaged_gears(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let count = erudition_blessing_count(catalog, blessings)?;
    let cap = whole(parameter_six(parameters, 1)?)?;
    let value = parameter_six(parameters, 0)?
        .checked_mul(i64::from(count.min(cap)))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let raw = binding.rule().get();
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    Ok(finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            groups: vec![unique_group(group)],
            modifiers: vec![ModifierDefinition {
                id: modifier,
                stat: StatKind::Atk,
                stage: FormulaStage::DamageBoost,
                purpose: FormulaPurpose::OrdinaryDamage,
                value: scalar(value),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: ultimate_source_filters(),
            }],
            ..RuleParts::default()
        },
    ))
}

fn erudition_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.erudition")
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

fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        RuleSelectorChoice::First,
        1,
    )
}

fn opposing_actors_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        RuleSelectorChoice::All,
        16,
    )
}

fn action_targets_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::All,
        16,
    )
}

fn defeated_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Defeated,
        RulePresencePredicate::Any,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::First,
        1,
    )
}

#[allow(clippy::too_many_arguments)]
fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    reference: RuleSelectorReference,
    ordering: RuleSelectorOrdering,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        life,
        presence,
        reference,
        ordering,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn ultimate_source_filters() -> Box<[ModifierFilter]> {
    vec![
        ModifierFilter::FormulaSubject(FormulaSubject::Source),
        ModifierFilter::AbilityTag("ultimate".into()),
    ]
    .into_boxed_slice()
}

fn unique_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    }
}

fn permanent_buff() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
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

fn trigger(
    id: TriggerId,
    point: RuleEventPoint,
    phase: TriggerPhase,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
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
    attachment: RuleAttachment,
    parts: RuleParts,
) -> ExecutableBattleRule {
    propagation_s01::finish_rule(
        binding,
        attachment,
        parts.groups,
        parts.modifiers,
        parts.selectors,
        parts.effects,
        parts.programs,
        parts.triggers,
        parts.slots,
    )
}

fn local<I: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<I, BattleRuleLoweringError> {
    base.checked_add(
        (raw & 0xffff)
            .checked_mul(16)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    )
    .and_then(|value| value.checked_add(offset))
    .and_then(|value| I::try_from(value).ok())
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    value
        .checked_div(1_000_000)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}
