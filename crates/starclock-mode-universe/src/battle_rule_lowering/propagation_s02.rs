use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
    RuleSelectorSide, RuleUnitSelector,
};

const METABOLIC_CAVITY: &str = "StageAbility_612742";
const EXCITATORY_GLAND: &str = "StageAbility_612743";
const EXPOSED_BRAIN_MATTER: &str = "StageAbility_612744";
const INTERSEGMENTAL_MEMBRANE: &str = "StageAbility_612745";
const CATALYST: &str = "StageAbility_612746";
const OSSEUS_BLADE: &str = "StageAbility_612750";

const LOCAL_PROGRAM_BASE: u32 = 0x7b20_0000;
const LOCAL_AUX_PROGRAM_BASE: u32 = 0x7b30_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7b40_0000;
const LOCAL_SECOND_SELECTOR_BASE: u32 = 0x7b50_0000;
const LOCAL_EFFECT_BASE: u32 = 0x7b60_0000;
const LOCAL_SLOT_BASE: u32 = 0x7b70_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7b80_0000;
const LOCAL_GROUP_BASE: u32 = 0x7b90_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7ba0_0000;

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        METABOLIC_CAVITY,
        EXCITATORY_GLAND,
        EXPOSED_BRAIN_MATTER,
        INTERSEGMENTAL_MEMBRANE,
        CATALYST,
        OSSEUS_BLADE,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            METABOLIC_CAVITY => metabolic_cavity(binding, parameters)?,
            EXCITATORY_GLAND => excitatory_gland(binding, parameters)?,
            EXPOSED_BRAIN_MATTER => exposed_brain_matter(binding, parameters)?,
            INTERSEGMENTAL_MEMBRANE => intersegmental_membrane(binding, parameters)?,
            CATALYST => catalyst(binding, parameters)?,
            OSSEUS_BLADE => osseus_blade(catalog, blessings, binding, parameters)?,
            _ => unreachable!("closed Propagation S02 binding set"),
        });
    }
    Ok(output)
}

fn metabolic_cavity(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let heal = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let sync = local::<ProgramId>(LOCAL_AUX_PROGRAM_BASE, raw, 0)?;
    let apply = local::<ProgramId>(LOCAL_AUX_PROGRAM_BASE, raw, 1)?;
    let lowest = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let allies = local::<SelectorId>(LOCAL_SECOND_SELECTOR_BASE, raw, 0)?;
    let enemies = local::<SelectorId>(LOCAL_SECOND_SELECTOR_BASE, raw, 1)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let stack_slot = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let heal_ratio = parameter_six(parameters, 0)?;
    let mitigation_per_spore = parameter_six(parameters, 1)?;

    let heal_amount = multiply(
        multiply(
            ValueExpr::QueryStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Hp,
                purpose: FormulaPurpose::Stat,
            },
            scalar(heal_ratio),
        ),
        ValueExpr::Convert {
            value: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::RuleSignalValue,
            )),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        },
    );
    let heal_program =
        ProgramDefinition::new(heal, Vec::new(), vec![lowest], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: lowest,
                amount: heal_amount,
                apply_formula_modifiers: false,
            })],
        );

    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut effects = Vec::new();
    let mut programs = vec![heal_program];
    let mut triggers = vec![trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        RuleEventPoint::InformationalRule,
        TriggerPhase::AfterEvent,
        OnceScope::Event,
        EventFilter::default(),
        signal_condition(propagation_s01::SPORE_BURST_SIGNAL),
        heal,
    )];
    if mitigation_per_spore > 0 {
        groups.push(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        let total_spores = ValueExpr::SelectorSum {
            selector: enemies,
            value: Box::new(ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::CurrentTarget,
                effect: propagation_s01::SPORE_EFFECT,
            }),
        };
        let modifier_ids = damage_purposes()
            .into_iter()
            .enumerate()
            .map(|(index, purpose)| {
                let id = local::<ModifierDefinitionId>(
                    LOCAL_MODIFIER_BASE,
                    raw,
                    u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                )?;
                modifiers.push(ModifierDefinition {
                    id,
                    stat: StatKind::Hp,
                    stage: FormulaStage::Mitigation,
                    purpose,
                    value: multiply(
                        effect_stack_scalar(stack_slot),
                        scalar(mitigation_per_spore),
                    ),
                    stacking_group: group,
                    priority: 0,
                    floor: Some(starclock_combat::Scalar::ZERO),
                    cap: None,
                    cap_stage: FormulaStage::Mitigation,
                    snapshot: SnapshotPolicy::RecomputeOnStackChange,
                    source_stack_slot: Some(stack_slot),
                    filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                        .into_boxed_slice(),
                });
                Ok(id)
            })
            .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
        effects.push(
            EffectDefinition::new(effect, Vec::new(), modifier_ids)
                .with_runtime_template(permanent_buff(144)?),
        );
        programs.push(
            ProgramDefinition::new(
                apply,
                Vec::new(),
                vec![allies, enemies],
                vec![effect],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: allies,
                    effect,
                    stacks: total_spores.clone(),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]),
        );
        programs.push(
            ProgramDefinition::new(
                sync,
                vec![apply],
                vec![allies, enemies],
                vec![effect],
                Vec::new(),
            )
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: allies,
                    effect,
                }),
                ProgramStep::If {
                    condition: ConditionExpr::Compare {
                        lhs: Box::new(total_spores),
                        operator: Comparison::Greater,
                        rhs: Box::new(integer(0)),
                    },
                    then_program: apply,
                    else_program: None,
                },
            ]),
        );
        for (offset, point, once_scope) in [
            (1, RuleEventPoint::BattleStarted, OnceScope::Battle),
            (2, RuleEventPoint::EffectApplied, OnceScope::Event),
            (3, RuleEventPoint::EffectStacksChanged, OnceScope::Event),
            (4, RuleEventPoint::EffectRemoved, OnceScope::Event),
        ] {
            triggers.push(trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, offset)?,
                point,
                TriggerPhase::AfterEvent,
                once_scope,
                if point == RuleEventPoint::BattleStarted {
                    EventFilter::default()
                } else {
                    EventFilter {
                        effect_definition: Some(propagation_s01::SPORE_EFFECT),
                        ..EventFilter::default()
                    }
                },
                ConditionExpr::Literal(true),
                sync,
            ));
        }
    }
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        groups,
        modifiers,
        vec![
            SelectorDefinition::new(lowest).with_rule_units(lowest_ally_selector()?),
            SelectorDefinition::new(allies).with_rule_units(all_allies_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemies_selector()?),
        ],
        effects,
        programs,
        triggers,
        Vec::new(),
    ))
}

fn excitatory_gland(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let arm = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let recover = local::<ProgramId>(LOCAL_AUX_PROGRAM_BASE, raw, 0)?;
    let bonus = local::<ProgramId>(LOCAL_AUX_PROGRAM_BASE, raw, 1)?;
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let armed = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let marker = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let bonus_chance = parameter_six(parameters, 0)?;

    let arm_program = ProgramDefinition::new(arm, Vec::new(), vec![owner], Vec::new(), Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::SetSlot {
                slot: armed,
                value: integer(1),
            },
        )]);
    let mut recover_steps = vec![skill_point_gain(owner)];
    let mut effects = Vec::new();
    let mut programs = vec![arm_program];
    let mut triggers = Vec::new();
    if bonus_chance > 0 {
        effects.push(
            EffectDefinition::new(marker, Vec::new(), Vec::new())
                .with_runtime_template(permanent_buff(1)?),
        );
        recover_steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: marker,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Fixed,
            base_chance: Some(scalar(bonus_chance)),
            rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        }));
        programs.push(
            ProgramDefinition::new(bonus, Vec::new(), vec![owner], vec![marker], Vec::new())
                .with_steps(vec![
                    skill_point_gain(owner),
                    ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                        selector: owner,
                        effect: marker,
                    }),
                ]),
        );
        triggers.push(trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 2)?,
            RuleEventPoint::EffectApplied,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                effect_definition: Some(marker),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            bonus,
        ));
    }
    recover_steps.push(ProgramStep::Operation(RuleOperationTemplate::SetSlot {
        slot: armed,
        value: integer(0),
    }));
    programs.push(
        ProgramDefinition::new(
            recover,
            (bonus_chance > 0)
                .then_some(vec![bonus])
                .unwrap_or_default(),
            vec![owner],
            effects.iter().map(EffectDefinition::id).collect(),
            Vec::new(),
        )
        .with_steps(recover_steps),
    );
    triggers.extend([
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ActionStarted,
            TriggerPhase::AfterEvent,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Basic),
                ..EventFilter::default()
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadResource {
                    selector: owner,
                    resource: RuleResourceKind::SkillPoints,
                }),
                operator: Comparison::Equal,
                rhs: Box::new(scalar(0)),
            },
            arm,
        ),
        trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Basic),
                ..EventFilter::default()
            },
            slot_equals(armed, 1),
            recover,
        ),
    ]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        effects,
        programs,
        triggers,
        vec![
            StateSlotDef::new(
                armed,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
        ],
    ))
}

fn exposed_brain_matter(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let actor = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let adjacent = local::<SelectorId>(LOCAL_SECOND_SELECTOR_BASE, raw, 0)?;
    let all_adjacent = whole(parameter_six(parameters, 1)?)? == 2;
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![actor, adjacent],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::UnboostedDamageFromEventElement {
            selector: adjacent,
            amount: multiply(
                ValueExpr::ReadEventProperty(EventValueProperty::DamageRawAmount),
                scalar(parameter_six(parameters, 0)?),
            ),
            class: DamageClass::Additional,
            can_defeat: true,
        },
    )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?),
            SelectorDefinition::new(adjacent).with_rule_units(adjacent_selector(!all_adjacent)?),
        ],
        Vec::new(),
        vec![program_definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::DamageApplied,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(actor),
                ability_tag: Some(AbilityTag::Basic),
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Ordinary),
                excluded_source: Some(binding.source().definition()),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
        Vec::new(),
    ))
}

fn intersegmental_membrane(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let stack_slot = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let maximum = whole(parameter_six(parameters, 2)?)?;
    let ratio = parameter_six(parameters, 0)?;
    let modifier_ids = damage_purposes()
        .into_iter()
        .enumerate()
        .map(|(index, _)| {
            local::<ModifierDefinitionId>(
                LOCAL_MODIFIER_BASE,
                raw,
                u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifiers = damage_purposes()
        .into_iter()
        .zip(modifier_ids.iter().copied())
        .map(|(purpose, id)| ModifierDefinition {
            id,
            stat: StatKind::Hp,
            stage: FormulaStage::Mitigation,
            purpose,
            value: multiply(effect_stack_scalar(stack_slot), scalar(ratio)),
            stacking_group: group,
            priority: 0,
            floor: Some(starclock_combat::Scalar::ZERO),
            cap: Some(starclock_combat::Scalar::ONE),
            cap_stage: FormulaStage::Mitigation,
            snapshot: SnapshotPolicy::RecomputeOnStackChange,
            source_stack_slot: Some(stack_slot),
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                .into_boxed_slice(),
        })
        .collect();
    let runtime = timed_stack_buff(whole(parameter_six(parameters, 1)?)?, maximum)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: resource_magnitude(false),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        modifiers,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), modifier_ids).with_runtime_template(runtime),
        ],
        vec![program_definition],
        vec![resource_trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            owner,
            false,
            program,
        )],
        Vec::new(),
    ))
}

fn catalyst(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let arm = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let clear = local::<ProgramId>(LOCAL_AUX_PROGRAM_BASE, raw, 0)?;
    let apply = local::<ProgramId>(LOCAL_AUX_PROGRAM_BASE, raw, 1)?;
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let allies = local::<SelectorId>(LOCAL_SECOND_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let armed = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 0)?;
    let stack_slot = local::<StateSlotDefinitionId>(LOCAL_SLOT_BASE, raw, 1)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let maximum = whole(parameter_six(parameters, 2)?)?;
    let modifier_ids = damage_purposes()
        .into_iter()
        .enumerate()
        .map(|(index, _)| {
            local::<ModifierDefinitionId>(
                LOCAL_MODIFIER_BASE,
                raw,
                u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let ratio = parameter_six(parameters, 0)?;
    let modifiers = damage_purposes()
        .into_iter()
        .zip(modifier_ids.iter().copied())
        .map(|(purpose, id)| ModifierDefinition {
            id,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose,
            value: multiply(effect_stack_scalar(stack_slot), scalar(ratio)),
            stacking_group: group,
            priority: 0,
            floor: Some(starclock_combat::Scalar::ZERO),
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::RecomputeOnStackChange,
            source_stack_slot: Some(stack_slot),
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)]
                .into_boxed_slice(),
        })
        .collect();
    let arm_program = ProgramDefinition::new(arm, Vec::new(), vec![owner], Vec::new(), Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::SetSlot {
                slot: armed,
                value: integer(1),
            },
        )]);
    let clear_program =
        ProgramDefinition::new(clear, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: armed,
                value: integer(0),
            })],
        );
    let apply_program = ProgramDefinition::new(
        apply,
        Vec::new(),
        vec![owner, allies],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: allies,
            effect,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: armed,
            value: integer(0),
        }),
    ]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        modifiers,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(allies).with_rule_units(all_allies_selector()?),
        ],
        vec![
            EffectDefinition::new(effect, Vec::new(), modifier_ids).with_runtime_template(
                timed_stack_buff(whole(parameter_six(parameters, 1)?)?, maximum)?,
            ),
        ],
        vec![arm_program, clear_program, apply_program],
        vec![
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
                RuleEventPoint::ActionStarted,
                TriggerPhase::AfterEvent,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    action_kind: Some(starclock_combat::rule::model::RuleActionKind::Skill),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                arm,
            ),
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                RuleEventPoint::HitStarted,
                TriggerPhase::Before,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    ability_tag: Some(AbilityTag::Attack),
                    ..EventFilter::default()
                },
                slot_equals(armed, 1),
                clear,
            ),
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 2)?,
                RuleEventPoint::ActionResolved,
                TriggerPhase::AfterAction,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    action_kind: Some(starclock_combat::rule::model::RuleActionKind::Skill),
                    ..EventFilter::default()
                },
                slot_equals(armed, 1),
                apply,
            ),
        ],
        vec![
            StateSlotDef::new(
                armed,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
        ],
    ))
}

fn osseus_blade(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let cap = whole(parameter_six(parameters, 1)?)?;
    let count = i64::from(propagation_blessing_count(catalog, blessings)?.min(cap));
    let value = parameter_six(parameters, 0)?
        .checked_mul(count)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let modifier = local::<ModifierDefinitionId>(LOCAL_MODIFIER_BASE, raw, 0)?;
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
    let install = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        vec![ModifierDefinition {
            id: modifier,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose: FormulaPurpose::OrdinaryDamage,
            value: scalar(value),
            stacking_group: group,
            priority: 0,
            floor: Some(starclock_combat::Scalar::ZERO),
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![
                ModifierFilter::FormulaSubject(FormulaSubject::Source),
                ModifierFilter::AbilityTag("basic".into()),
            ]
            .into_boxed_slice(),
        }],
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(permanent_buff(1)?),
        ],
        vec![
            ProgramDefinition::new(install, Vec::new(), vec![owner], vec![effect], Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::ApplyEffect {
                        selector: owner,
                        effect,
                        stacks: integer(1),
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    },
                )]),
        ],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::BattleStarted,
            TriggerPhase::AfterEvent,
            OnceScope::Battle,
            EventFilter::default(),
            ConditionExpr::Literal(true),
            install,
        )],
        Vec::new(),
    ))
}

fn propagation_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.propagation")
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

fn skill_point_gain(selector: SelectorId) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
        selector,
        resource: RuleResourceKind::SkillPoints,
        update: ResourceUpdateKind::Gain,
        amount: scalar(1_000_000),
        scales_with_regeneration: false,
        rounding: Rounding::Floor,
    })
}

fn signal_condition(code: u32) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::RuleSignalCode,
        )),
        operator: Comparison::Equal,
        rhs: Box::new(integer(i64::from(code))),
    }
}

fn slot_equals(slot: StateSlotDefinitionId, value: i64) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Equal,
        rhs: Box::new(integer(value)),
    }
}

fn resource_trigger(
    id: TriggerId,
    owner: SelectorId,
    recovery: bool,
    program: ProgramId,
) -> TriggerDef {
    trigger(
        id,
        RuleEventPoint::ResourceChanged,
        TriggerPhase::AfterEvent,
        OnceScope::Event,
        EventFilter {
            actor_selector: Some(owner),
            resource: Some(RuleResourceKind::SkillPoints),
            ..EventFilter::default()
        },
        resource_direction(recovery),
        program,
    )
}

fn resource_direction(recovery: bool) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ResourceDelta,
        )),
        operator: if recovery {
            Comparison::Greater
        } else {
            Comparison::Less
        },
        rhs: Box::new(scalar(0)),
    }
}

fn resource_magnitude(recovery: bool) -> ValueExpr {
    let value = ValueExpr::ReadEventProperty(EventValueProperty::ResourceDelta);
    ValueExpr::Convert {
        value: Box::new(if recovery {
            value
        } else {
            ValueExpr::Negate(Box::new(value))
        }),
        target: RuleValueKind::Integer,
        rounding: Rounding::Floor,
    }
}

fn effect_stack_scalar(slot: StateSlotDefinitionId) -> ValueExpr {
    ValueExpr::Convert {
        value: Box::new(ValueExpr::Slot(slot)),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    }
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

fn permanent_buff(maximum: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        maximum,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn timed_stack_buff(
    turns: u16,
    maximum: u16,
) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        maximum,
        Some(integer(i64::from(turns))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
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

fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorOrdering::Formation,
        1,
        RuleSelectorChoice::First,
        None,
    )
}

fn actor_player_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleSelectorOrdering::Formation,
        1,
        RuleSelectorChoice::First,
        None,
    )
}

fn all_allies_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorOrdering::Formation,
        16,
        RuleSelectorChoice::All,
        None,
    )
}

fn all_enemies_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleSelectorOrdering::Formation,
        16,
        RuleSelectorChoice::All,
        None,
    )
}

fn lowest_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorOrdering::HpRatioAscending,
        1,
        RuleSelectorChoice::First,
        None,
    )
}

fn adjacent_selector(random_one: bool) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleSelectorOrdering::Formation,
        if random_one { 1 } else { 2 },
        if random_one {
            RuleSelectorChoice::RngUniform
        } else {
            RuleSelectorChoice::All
        },
        random_one.then(|| "damage-target".into()),
    )
    .map(|selector| selector.with_predicates(vec![RuleSelectorPredicate::AdjacentToPrimary]))
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    ordering: RuleSelectorOrdering,
    maximum: u16,
    choice: RuleSelectorChoice,
    rng_purpose: Option<Box<str>>,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        ordering,
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        rng_purpose,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
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
