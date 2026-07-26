use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const GRAY_MATTER: &str = "StageAbility_612830";
const AMYGDALA: &str = "StageAbility_612831";
const OCCIPITAL_LOBE: &str = "StageAbility_612832";
const VESTIBULAR_SYSTEM: &str = "StageAbility_612840";
const TRANSMITTER_SYNTHESIS: &str = "StageAbility_612841";
const EXPLICIT_MEMORY: &str = "StageAbility_612842";

pub(super) const BRAIN_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7d00_0001).expect("reserved Erudition effect ID");
pub(super) const BRAIN_ULTIMATE_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7d00_0002).expect("reserved Erudition effect ID");
const ENGINE_OWNER: SelectorId =
    SelectorId::new(0x7d00_0003).expect("reserved Erudition selector ID");
const ENGINE_ARM: ProgramId = ProgramId::new(0x7d00_0004).expect("reserved Erudition program ID");
const ENGINE_ACTIVATE: ProgramId =
    ProgramId::new(0x7d00_0005).expect("reserved Erudition program ID");
const ENGINE_CLEANUP: ProgramId =
    ProgramId::new(0x7d00_0006).expect("reserved Erudition program ID");
const ENGINE_FREE_ACTION_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x7d00_0007).expect("reserved Erudition slot ID");
const ENGINE_ARM_TRIGGER: TriggerId =
    TriggerId::new(0x7d00_0008).expect("reserved Erudition trigger ID");
const ENGINE_ACTIVATE_TRIGGER: TriggerId =
    TriggerId::new(0x7d00_0009).expect("reserved Erudition trigger ID");
const ENGINE_CLEANUP_TRIGGER: TriggerId =
    TriggerId::new(0x7d00_000a).expect("reserved Erudition trigger ID");

const LOCAL_PROGRAM_BASE: u32 = 0x7d10_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7d20_0000;
const LOCAL_EFFECT_BASE: u32 = 0x7d30_0000;
const LOCAL_SLOT_BASE: u32 = 0x7d40_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7d50_0000;
const LOCAL_GROUP_BASE: u32 = 0x7d60_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7d70_0000;

const BRAIN_MAXIMUM_STACKS: i64 = 1_000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        GRAY_MATTER,
        AMYGDALA,
        OCCIPITAL_LOBE,
        VESTIBULAR_SYSTEM,
        TRANSMITTER_SYNTHESIS,
        EXPLICIT_MEMORY,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            GRAY_MATTER => gray_matter(binding, parameters)?,
            AMYGDALA => amygdala(binding, parameters)?,
            OCCIPITAL_LOBE => occipital_lobe(binding, parameters)?,
            VESTIBULAR_SYSTEM => vestibular_system(binding, parameters)?,
            TRANSMITTER_SYNTHESIS => transmitter_synthesis(binding, parameters)?,
            EXPLICIT_MEMORY => explicit_memory(binding, parameters)?,
            _ => unreachable!("closed Erudition S01 binding set"),
        });
    }
    Ok(output)
}

pub(super) fn add_brain_engine(
    rule: &mut ExecutableBattleRule,
) -> Result<(), BattleRuleLoweringError> {
    if rule.attachment != RuleAttachment::EveryPlayer {
        return Err(BattleRuleLoweringError::InvalidDefinition);
    }
    merge_selectors(
        rule,
        vec![SelectorDefinition::new(ENGINE_OWNER).with_rule_units(owner_selector()?)],
    );
    merge_effects(
        rule,
        vec![
            EffectDefinition::new(BRAIN_EFFECT, Vec::new(), Vec::new())
                .with_runtime_template(brain_runtime()?),
            EffectDefinition::new(BRAIN_ULTIMATE_EFFECT, Vec::new(), Vec::new())
                .with_runtime_template(permanent_effect(EffectCategory::NeutralState, 1)?),
        ],
    );
    merge_programs(
        rule,
        vec![
            ProgramDefinition::new(
                ENGINE_ARM,
                Vec::new(),
                vec![ENGINE_OWNER],
                vec![BRAIN_ULTIMATE_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::SetSlot {
                    slot: ENGINE_FREE_ACTION_SLOT,
                    value: integer(1),
                },
            )]),
            ProgramDefinition::new(
                ENGINE_ACTIVATE,
                Vec::new(),
                vec![ENGINE_OWNER],
                vec![BRAIN_EFFECT, BRAIN_ULTIMATE_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: ENGINE_OWNER,
                    effect: BRAIN_EFFECT,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: ENGINE_OWNER,
                    effect: BRAIN_ULTIMATE_EFFECT,
                    stacks: integer(1),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    selector: ENGINE_OWNER,
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Gain,
                    amount: ValueExpr::QueryMaximumEnergy(StatQuerySubject::Owner),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                }),
            ]),
            ProgramDefinition::new(
                ENGINE_CLEANUP,
                Vec::new(),
                vec![ENGINE_OWNER],
                vec![BRAIN_ULTIMATE_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: ENGINE_OWNER,
                    effect: BRAIN_ULTIMATE_EFFECT,
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: ENGINE_FREE_ACTION_SLOT,
                    value: integer(0),
                }),
            ]),
        ],
    );
    let runtime = rule
        .definition
        .runtime()
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let mut slots = runtime.state_slots().to_vec();
    slots.push(
        StateSlotDef::new(
            ENGINE_FREE_ACTION_SLOT,
            RuleValueKind::Integer,
            BattleRuleScope::Battle,
            RuleValue::Integer(0),
        )
        .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
    );
    slots.sort_unstable_by_key(StateSlotDef::id);
    let ultimate_filter = EventFilter {
        actor_selector: Some(ENGINE_OWNER),
        action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
        ..EventFilter::default()
    };
    let mut triggers = runtime.triggers().to_vec();
    triggers.extend([
        trigger_with_priority(
            ENGINE_ARM_TRIGGER,
            RuleEventPoint::ActionStarted,
            TriggerPhase::AfterEvent,
            OnceScope::Action,
            ultimate_filter.clone(),
            ConditionExpr::EffectExists {
                selector: ENGINE_OWNER,
                effect: BRAIN_ULTIMATE_EFFECT,
            },
            ENGINE_ARM,
            -100,
        ),
        trigger_with_priority(
            ENGINE_ACTIVATE_TRIGGER,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            ultimate_filter.clone(),
            all(vec![
                slot_equals(ENGINE_FREE_ACTION_SLOT, 0),
                ConditionExpr::EffectExists {
                    selector: ENGINE_OWNER,
                    effect: BRAIN_EFFECT,
                },
                effect_stacks_at_least(BRAIN_EFFECT, BRAIN_MAXIMUM_STACKS),
            ]),
            ENGINE_ACTIVATE,
            50,
        ),
        trigger_with_priority(
            ENGINE_CLEANUP_TRIGGER,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            ultimate_filter,
            slot_equals(ENGINE_FREE_ACTION_SLOT, 1),
            ENGINE_CLEANUP,
            100,
        ),
    ]);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    let mut program_ids = rule.definition.programs().to_vec();
    program_ids.extend([ENGINE_ARM, ENGINE_ACTIVATE, ENGINE_CLEANUP]);
    program_ids.sort_unstable();
    program_ids.dedup();
    let mut selector_ids = rule.definition.selectors().to_vec();
    selector_ids.push(ENGINE_OWNER);
    selector_ids.sort_unstable();
    selector_ids.dedup();
    rule.definition =
        RuleDefinition::new(rule.definition.id(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(runtime.source().clone(), slots, triggers, None),
        );
    Ok(())
}

fn gray_matter(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let target = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let entry = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let break_charge = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let broken_hit = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 2)?;
    let enhanced = parameter_six(parameters, 3)? > 0;
    let mut programs = vec![
        charge_program(entry, owner, charge_stacks(parameters, 1)?)?,
        charge_program(break_charge, owner, charge_stacks(parameters, 0)?)?,
    ];
    let mut triggers = vec![
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::BattleStarted,
            TriggerPhase::AfterEvent,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            entry,
        ),
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::WeaknessBroken,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            break_charge,
        ),
    ];
    if enhanced {
        programs.push(charge_program(
            broken_hit,
            owner,
            charge_stacks(parameters, 3)?,
        )?);
        triggers.push(trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 2)?,
            RuleEventPoint::DamageApplied,
            TriggerPhase::AfterEvent,
            OnceScope::TargetWithinAction,
            EventFilter {
                actor_selector: Some(owner),
                target_selector: Some(target),
                ability_tag: Some(AbilityTag::Attack),
                ..EventFilter::default()
            },
            ConditionExpr::IsBroken(target),
            broken_hit,
        ));
    }
    Ok(finish(
        binding,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(target).with_rule_units(event_target_selector()?),
        ],
        Vec::new(),
        programs,
        triggers,
        Vec::new(),
    ))
}

fn amygdala(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let charge = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let speed = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let speed_effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier_id = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let enhanced = parameter_six(parameters, 1)? > 0;
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut effects = Vec::new();
    let mut programs = vec![charge_program(
        charge,
        owner,
        charge_stacks(parameters, 0)?,
    )?];
    let mut triggers = vec![trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        RuleEventPoint::UnitDefeated,
        TriggerPhase::AfterDefeatSettlement,
        OnceScope::Event,
        EventFilter {
            actor_selector: Some(owner),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        charge,
    )];
    if enhanced {
        groups.push(unique_group(group));
        modifiers.push(ModifierDefinition {
            id: modifier_id,
            stat: StatKind::Spd,
            stage: FormulaStage::PercentOfBase,
            purpose: FormulaPurpose::Stat,
            value: scalar(parameter_six(parameters, 1)?),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::PercentOfBase,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
        effects.push(
            EffectDefinition::new(speed_effect, Vec::new(), vec![modifier_id])
                .with_runtime_template(owner_turn_effect(whole(parameter_six(parameters, 2)?)?)?),
        );
        programs.push(
            ProgramDefinition::new(
                speed,
                Vec::new(),
                vec![owner],
                vec![speed_effect],
                Vec::new(),
            )
            .with_steps(vec![apply_effect(owner, speed_effect, integer(1))]),
        );
        for (offset, point) in [
            RuleEventPoint::EffectApplied,
            RuleEventPoint::EffectStacksChanged,
        ]
        .into_iter()
        .enumerate()
        {
            triggers.push(trigger(
                local::<TriggerId>(
                    LOCAL_TRIGGER_BASE,
                    raw,
                    u32::try_from(offset + 1)
                        .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                )?,
                point,
                TriggerPhase::AfterEvent,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(owner),
                    effect_definition: Some(BRAIN_EFFECT),
                    ..EventFilter::default()
                },
                ConditionExpr::Compare {
                    lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::StackCount)),
                    operator: Comparison::GreaterOrEqual,
                    rhs: Box::new(integer(BRAIN_MAXIMUM_STACKS)),
                },
                speed,
            ));
        }
    }
    Ok(finish(
        binding,
        groups,
        modifiers,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        effects,
        programs,
        triggers,
        Vec::new(),
    ))
}

fn occipital_lobe(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let targets = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let install = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let update = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let stack_slot = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let base_modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let count_modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 1)?;
    let enhanced = parameter_six(parameters, 2)? > 0;
    let current_count = ValueExpr::Add(
        Box::new(ValueExpr::SelectorCount(targets)),
        Box::new(integer(1)),
    );
    let stacks = if enhanced {
        ValueExpr::Maximum(
            Box::new(ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::Owner,
                effect,
            }),
            Box::new(current_count),
        )
    } else {
        current_count
    };
    let base = ModifierDefinition {
        id: base_modifier,
        stat: StatKind::Atk,
        stage: FormulaStage::Resistance,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: scalar(parameter_six(parameters, 0)?),
        stacking_group: group,
        priority: 0,
        floor: Some(starclock_combat::Scalar::ZERO),
        cap: None,
        cap_stage: FormulaStage::Resistance,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: ultimate_source_filters(),
    };
    let count_value = multiply(
        ValueExpr::Convert {
            value: Box::new(ValueExpr::Subtract(
                Box::new(ValueExpr::Slot(stack_slot)),
                Box::new(integer(1)),
            )),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        },
        scalar(parameter_six(parameters, 1)?),
    );
    let per_target = ModifierDefinition {
        id: count_modifier,
        stat: StatKind::Atk,
        stage: FormulaStage::Resistance,
        purpose: FormulaPurpose::OrdinaryDamage,
        value: count_value,
        stacking_group: group,
        priority: 1,
        floor: Some(starclock_combat::Scalar::ZERO),
        cap: None,
        cap_stage: FormulaStage::Resistance,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(stack_slot),
        filters: ultimate_source_filters(),
    };
    let install_definition =
        ProgramDefinition::new(install, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![apply_effect(owner, effect, integer(1))]);
    let update_definition = ProgramDefinition::new(
        update,
        Vec::new(),
        vec![owner, targets],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![apply_effect(owner, effect, stacks)]);
    Ok(finish(
        binding,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        }],
        vec![base, per_target],
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(targets).with_rule_units(action_targets_selector()?),
        ],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![base_modifier, count_modifier])
                .with_runtime_template(permanent_effect(EffectCategory::Buff, 17)?),
        ],
        vec![install_definition, update_definition],
        vec![
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
                RuleEventPoint::BattleStarted,
                TriggerPhase::AfterEvent,
                OnceScope::Battle,
                EventFilter::default(),
                ConditionExpr::Literal(true),
                install,
            ),
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                RuleEventPoint::ActionResolved,
                TriggerPhase::AfterAction,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                update,
            ),
        ],
        Vec::new(),
    ))
}

fn vestibular_system(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let apply = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let remove = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let arm = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 2)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier_id = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let armed = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let enhanced = parameter_six(parameters, 1)? > 0;
    let modifier = ModifierDefinition {
        id: modifier_id,
        stat: StatKind::CritDamage,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: scalar(parameter_six(parameters, 0)?),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)].into_boxed_slice(),
    };
    let apply_definition =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![apply_effect(owner, effect, integer(1))]);
    let remove_definition =
        ProgramDefinition::new(remove, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: armed,
                    value: integer(0),
                }),
            ]);
    let arm_definition =
        ProgramDefinition::new(arm, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: armed,
                value: integer(1),
            })],
        );
    let free_ultimate = ConditionExpr::EffectExists {
        selector: owner,
        effect: BRAIN_ULTIMATE_EFFECT,
    };
    let mut programs = vec![apply_definition, remove_definition];
    let mut triggers = vec![trigger_with_priority(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        RuleEventPoint::ActionStarted,
        TriggerPhase::AfterEvent,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(owner),
            action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
            ..EventFilter::default()
        },
        free_ultimate.clone(),
        apply,
        0,
    )];
    let slots = vec![
        StateSlotDef::new(
            armed,
            RuleValueKind::Integer,
            BattleRuleScope::Battle,
            RuleValue::Integer(0),
        )
        .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
    ];
    if enhanced {
        programs.push(arm_definition);
        triggers.extend([
            trigger_with_priority(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                RuleEventPoint::ActionResolved,
                TriggerPhase::AfterAction,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                    ..EventFilter::default()
                },
                free_ultimate,
                arm,
                0,
            ),
            trigger_with_priority(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 2)?,
                RuleEventPoint::ActionResolved,
                TriggerPhase::AfterAction,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    ability_tag: Some(AbilityTag::Attack),
                    ..EventFilter::default()
                },
                all(vec![
                    slot_equals(armed, 1),
                    ConditionExpr::Not(Box::new(ConditionExpr::EffectExists {
                        selector: owner,
                        effect: BRAIN_ULTIMATE_EFFECT,
                    })),
                ]),
                remove,
                20,
            ),
        ]);
    } else {
        triggers.push(trigger_with_priority(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                ..EventFilter::default()
            },
            free_ultimate,
            remove,
            20,
        ));
    }
    Ok(finish(
        binding,
        vec![unique_group(group)],
        vec![modifier],
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier_id])
                .with_runtime_template(permanent_effect(EffectCategory::Buff, 1)?),
        ],
        programs,
        triggers,
        slots,
    ))
}

fn transmitter_synthesis(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let charge = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let per_energy = parameter_six(parameters, 0)?;
    let amount = ValueExpr::Convert {
        value: Box::new(multiply(
            multiply(
                ValueExpr::ReadEventProperty(EventValueProperty::ResourceOverflow),
                scalar(per_energy),
            ),
            scalar(1_000_000_000),
        )),
        target: RuleValueKind::Integer,
        rounding: Rounding::NearestTiesEven,
    };
    let program = ProgramDefinition::new(
        charge,
        Vec::new(),
        vec![owner],
        vec![BRAIN_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![apply_effect(owner, BRAIN_EFFECT, amount)]);
    Ok(finish(
        binding,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![program],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ResourceChanged,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                resource: Some(RuleResourceKind::Energy),
                ..EventFilter::default()
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadEventProperty(
                    EventValueProperty::ResourceOverflow,
                )),
                operator: Comparison::Greater,
                rhs: Box::new(scalar(0)),
            },
            charge,
        )],
        Vec::new(),
    ))
}

fn explicit_memory(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let shield = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Shield,
        },
        scalar(parameter_six(parameters, 0)?),
    );
    let duration = whole(parameter_six(parameters, 1)?)?;
    let program = ProgramDefinition::new(shield, Vec::new(), vec![owner], vec![effect], Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::Shield {
                selector: owner,
                amount,
                effect,
            },
        )]);
    Ok(finish(
        binding,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), Vec::new())
                .with_runtime_template(shield_runtime(duration)?),
        ],
        vec![program],
        vec![trigger_with_priority(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
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
                effect: BRAIN_ULTIMATE_EFFECT,
            },
            shield,
            0,
        )],
        Vec::new(),
    ))
}

fn charge_program(
    id: ProgramId,
    owner: SelectorId,
    stacks: i64,
) -> Result<ProgramDefinition, BattleRuleLoweringError> {
    if stacks <= 0 || stacks > BRAIN_MAXIMUM_STACKS {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    Ok(
        ProgramDefinition::new(id, Vec::new(), vec![owner], vec![BRAIN_EFFECT], Vec::new())
            .with_steps(vec![apply_effect(owner, BRAIN_EFFECT, integer(stacks))]),
    )
}

// Keeping the six catalog families separate makes every call site expose its
// complete deterministic rule contribution instead of hiding partial state.
#[allow(clippy::too_many_arguments)]
fn finish(
    binding: &UniverseBattleRuleBinding,
    groups: Vec<ModifierStackingGroup>,
    modifiers: Vec<ModifierDefinition>,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
    slots: Vec<StateSlotDef>,
) -> ExecutableBattleRule {
    propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        groups,
        modifiers,
        selectors,
        effects,
        programs,
        triggers,
        slots,
    )
}

fn apply_effect(
    selector: SelectorId,
    effect: EffectDefinitionId,
    stacks: ValueExpr,
) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks,
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
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
    trigger_with_priority(id, point, phase, once_scope, filter, condition, program, 0)
}

#[allow(clippy::too_many_arguments)]
fn trigger_with_priority(
    id: TriggerId,
    point: RuleEventPoint,
    phase: TriggerPhase,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
    priority: i16,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(priority),
        program,
    }
}

fn all(values: Vec<ConditionExpr>) -> ConditionExpr {
    ConditionExpr::All(values.into_boxed_slice())
}

fn slot_equals(slot: StateSlotDefinitionId, value: i64) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Equal,
        rhs: Box::new(integer(value)),
    }
}

fn effect_stacks_at_least(effect: EffectDefinitionId, minimum: i64) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::QueryEffectStacks {
            subject: StatQuerySubject::Owner,
            effect,
        }),
        operator: Comparison::GreaterOrEqual,
        rhs: Box::new(integer(minimum)),
    }
}

fn brain_runtime() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        u16::try_from(BRAIN_MAXIMUM_STACKS)
            .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn permanent_effect(
    category: EffectCategory,
    stack_limit: u16,
) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        category,
        DispelCategory::NonDispellable,
        stack_limit,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn owner_turn_effect(turns: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(integer(i64::from(turns))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn shield_runtime(turns: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Shield,
        DispelCategory::NonDispellable,
        1,
        Some(integer(i64::from(turns))),
        DurationClock::OwnerTurnEnd,
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

fn ultimate_source_filters() -> Box<[ModifierFilter]> {
    vec![
        ModifierFilter::FormulaSubject(FormulaSubject::Source),
        ModifierFilter::AbilityTag("ultimate".into()),
    ]
    .into_boxed_slice()
}

fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        RuleSelectorChoice::First,
        1,
    )
}

fn event_target_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::First,
        1,
    )
}

fn action_targets_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleSelectorReference::ActionSnapshot,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::All,
        16,
    )
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    reference: RuleSelectorReference,
    ordering: RuleSelectorOrdering,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
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

fn charge_stacks(
    parameters: &[ExactParameter],
    index: usize,
) -> Result<i64, BattleRuleLoweringError> {
    parameter_six(parameters, index)?
        .checked_mul(BRAIN_MAXIMUM_STACKS)
        .and_then(|value| value.checked_div(1_000_000))
        .ok_or(BattleRuleLoweringError::InvalidParameter)
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

fn merge_selectors(rule: &mut ExecutableBattleRule, values: Vec<SelectorDefinition>) {
    let mut merged = rule.selectors.to_vec();
    merged.extend(values);
    merged.sort_unstable_by_key(SelectorDefinition::id);
    merged.dedup_by_key(|value| value.id());
    rule.selectors = merged.into_boxed_slice();
}

fn merge_effects(rule: &mut ExecutableBattleRule, values: Vec<EffectDefinition>) {
    let mut merged = rule.effects.to_vec();
    merged.extend(values);
    merged.sort_unstable_by_key(EffectDefinition::id);
    merged.dedup_by_key(|value| value.id());
    rule.effects = merged.into_boxed_slice();
}

fn merge_programs(rule: &mut ExecutableBattleRule, values: Vec<ProgramDefinition>) {
    let mut merged = rule.programs.to_vec();
    merged.extend(values);
    merged.sort_unstable_by_key(ProgramDefinition::id);
    merged.dedup_by_key(|value| value.id());
    rule.programs = merged.into_boxed_slice();
}
