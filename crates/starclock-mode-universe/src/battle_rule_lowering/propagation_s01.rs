use super::propagation_s04;
use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const SPORE_DISCHARGE: &str = "StageAbility_612730";
const FUNGAL_PUSTULE: &str = "StageAbility_612731";
const SCYTHE_LIMBS: &str = "StageAbility_612732";
const PUTREFACTION_ULCER: &str = "StageAbility_612740";
const LYTIC_ENZYME: &str = "StageAbility_612741";

pub(super) const SPORE_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7b00_0001).expect("reserved Propagation effect ID");
const ENGINE_ACTOR: SelectorId =
    SelectorId::new(0x7b00_0002).expect("reserved Propagation selector ID");
const ENGINE_TARGET: SelectorId =
    SelectorId::new(0x7b00_0003).expect("reserved Propagation selector ID");
const ENGINE_ADJACENT: SelectorId =
    SelectorId::new(0x7b00_0004).expect("reserved Propagation selector ID");
const ENGINE_PRIMARY_AND_ADJACENT: SelectorId =
    SelectorId::new(0x7b00_0005).expect("reserved Propagation selector ID");
const ENGINE_OTHER_ENEMIES: SelectorId =
    SelectorId::new(0x7b00_0006).expect("reserved Propagation selector ID");
const ENGINE_PROGRAM: ProgramId =
    ProgramId::new(0x7b00_0010).expect("reserved Propagation program ID");
const ENGINE_BURST_PROGRAM: ProgramId =
    ProgramId::new(0x7b00_0011).expect("reserved Propagation program ID");
const ENGINE_DEFEAT_PROGRAM: ProgramId =
    ProgramId::new(0x7b00_0012).expect("reserved Propagation program ID");
const ENGINE_TRIGGER: TriggerId =
    TriggerId::new(0x7b00_0020).expect("reserved Propagation trigger ID");
const ENGINE_DEFEAT_TRIGGER: TriggerId =
    TriggerId::new(0x7b00_0021).expect("reserved Propagation trigger ID");
const SPORE_SNAPSHOT_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x7b00_0030).expect("reserved Propagation slot ID");
const CRIT_STACK_SLOT_BASE: u32 = 0x7b10_0000;
const SPENT_ACCOUNTING_SIGNAL: u32 = 0x6127_3201;
const RECOVERED_ACCOUNTING_SIGNAL: u32 = 0x6127_3202;
pub(super) const SPORE_BURST_SIGNAL: u32 = 0x6127_4001;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let has_spend_spores = selected_level_parameters(blessings, SPORE_DISCHARGE).is_some();
    let has_recovery_spores = selected_level_parameters(blessings, FUNGAL_PUSTULE).is_some();
    let mut output = Vec::new();
    for key in [
        SPORE_DISCHARGE,
        FUNGAL_PUSTULE,
        SCYTHE_LIMBS,
        PUTREFACTION_ULCER,
        LYTIC_ENZYME,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            SPORE_DISCHARGE => spore_discharge(binding, parameters)?,
            FUNGAL_PUSTULE => fungal_pustule(binding, parameters)?,
            SCYTHE_LIMBS => {
                scythe_limbs(binding, parameters, has_spend_spores, has_recovery_spores)?
            }
            PUTREFACTION_ULCER | LYTIC_ENZYME => passive_rule(binding),
            _ => unreachable!("closed Propagation S01 set"),
        });
    }
    Ok(output)
}

pub(super) fn add_spore_engine(
    rule: &mut ExecutableBattleRule,
    blessings: &BlessingContributionSet,
    metamorphosis_required: bool,
) -> Result<(), BattleRuleLoweringError> {
    let maximum = selected_level_parameters(blessings, FUNGAL_PUSTULE)
        .map(|parameters| whole(parameter(parameters, 2)?))
        .transpose()?
        .filter(|level| *level == 2)
        .map_or(6, |_| 9);
    let spread = selected_level_parameters(blessings, PUTREFACTION_ULCER)
        .map(|parameters| whole(parameter(parameters, 0)?))
        .transpose()?
        .unwrap_or(1);
    let include_original = selected_level_parameters(blessings, PUTREFACTION_ULCER).is_some();
    let lytic = selected_level_parameters(blessings, LYTIC_ENZYME);
    let damage_bonus = lytic
        .map(|parameters| parameter(parameters, 0))
        .transpose()?
        .unwrap_or(0);
    let defeat_spread = lytic
        .map(|parameters| whole(parameter(parameters, 1)?))
        .transpose()?;
    let source = rule
        .definition
        .runtime()
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?
        .source()
        .definition();
    let stack_runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::NonDispellable,
        u16::try_from(maximum).map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    merge_effects(
        rule,
        vec![
            EffectDefinition::new(SPORE_EFFECT, Vec::new(), Vec::new())
                .with_runtime_template(stack_runtime),
        ],
    );

    let spread_selector = if include_original {
        ENGINE_PRIMARY_AND_ADJACENT
    } else {
        ENGINE_ADJACENT
    };
    let stack_scalar = ValueExpr::Convert {
        value: Box::new(ValueExpr::Slot(SPORE_SNAPSHOT_SLOT)),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    };
    let base_damage = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Actor,
            stat: StatKind::BreakBaseDamage,
            purpose: FormulaPurpose::AdditionalDamage,
        },
        stack_scalar,
    );
    let amount = multiply(
        base_damage,
        scalar(
            1_000_000_i64
                .checked_add(damage_bonus)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ),
    );
    let burst = ProgramDefinition::new(
        ENGINE_BURST_PROGRAM,
        Vec::new(),
        vec![ENGINE_TARGET, spread_selector],
        vec![SPORE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::UnboostedDamage {
            selector: ENGINE_TARGET,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Wind,
            can_defeat: true,
        }),
        ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
            code: SPORE_BURST_SIGNAL,
            value: Some(ValueExpr::Slot(SPORE_SNAPSHOT_SLOT)),
        }),
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: ENGINE_TARGET,
            effect: SPORE_EFFECT,
        }),
        ProgramStep::Operation(RuleOperationTemplate::RandomGroupedEffect {
            selector: spread_selector,
            effect: SPORE_EFFECT,
            groups: integer(spread),
            applications_per_group: 1,
            stacks: integer(1),
            choice_rng_purpose: DrawPurpose::DAMAGE_TARGET,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            chance_rng_purpose: None,
        }),
    ]);
    let engine = ProgramDefinition::new(
        ENGINE_PROGRAM,
        vec![ENGINE_BURST_PROGRAM],
        vec![
            ENGINE_ACTOR,
            ENGINE_TARGET,
            ENGINE_ADJACENT,
            ENGINE_PRIMARY_AND_ADJACENT,
        ],
        vec![SPORE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: SPORE_SNAPSHOT_SLOT,
            value: ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::EventTarget,
                effect: SPORE_EFFECT,
            },
        }),
        ProgramStep::If {
            condition: ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(SPORE_SNAPSHOT_SLOT)),
                operator: Comparison::GreaterOrEqual,
                rhs: Box::new(integer(3)),
            },
            then_program: ENGINE_BURST_PROGRAM,
            else_program: None,
        },
    ]);
    let mut programs = vec![engine, burst];
    let mut triggers = vec![TriggerDef {
        id: ENGINE_TRIGGER,
        event: RuleEventKind::Damage,
        event_point: RuleEventPoint::DamageApplied,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter {
            actor_selector: Some(ENGINE_ACTOR),
            target_selector: Some(ENGINE_TARGET),
            excluded_source: Some(source),
            ..EventFilter::default()
        },
        condition: if metamorphosis_required {
            ConditionExpr::EffectExists {
                selector: ENGINE_ACTOR,
                effect: propagation_s04::METAMORPHOSIS_EFFECT,
            }
        } else {
            ConditionExpr::Literal(true)
        },
        once_scope: OnceScope::TargetWithinAction,
        priority: ReactionPriority::new(0),
        program: ENGINE_PROGRAM,
    }];
    if let Some(mode) = defeat_spread {
        let target_selector = if mode == 1 {
            ENGINE_ADJACENT
        } else {
            ENGINE_OTHER_ENEMIES
        };
        programs.push(
            ProgramDefinition::new(
                ENGINE_DEFEAT_PROGRAM,
                Vec::new(),
                vec![target_selector],
                vec![SPORE_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RandomGroupedEffect {
                    selector: target_selector,
                    effect: SPORE_EFFECT,
                    groups: ValueExpr::Slot(SPORE_SNAPSHOT_SLOT),
                    applications_per_group: 1,
                    stacks: integer(1),
                    choice_rng_purpose: DrawPurpose::DAMAGE_TARGET,
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    chance_rng_purpose: None,
                },
            )]),
        );
        triggers.push(TriggerDef {
            id: ENGINE_DEFEAT_TRIGGER,
            event: RuleEventKind::Unit,
            event_point: RuleEventPoint::UnitDefeated,
            phase: TriggerPhase::AfterDefeatSettlement,
            filter: EventFilter {
                actor_selector: Some(ENGINE_ACTOR),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(SPORE_SNAPSHOT_SLOT)),
                operator: Comparison::Greater,
                rhs: Box::new(integer(0)),
            },
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program: ENGINE_DEFEAT_PROGRAM,
        });
    }
    merge_selectors(
        rule,
        vec![
            SelectorDefinition::new(ENGINE_ACTOR).with_rule_units(actor_player_selector()?),
            SelectorDefinition::new(ENGINE_TARGET).with_rule_units(event_target_any_life()?),
            SelectorDefinition::new(ENGINE_ADJACENT).with_rule_units(adjacent_enemy_selector()?),
            SelectorDefinition::new(ENGINE_PRIMARY_AND_ADJACENT)
                .with_rule_units(primary_and_adjacent_any_life()?),
            SelectorDefinition::new(ENGINE_OTHER_ENEMIES).with_rule_units(all_enemy_selector()?),
        ],
    );
    merge_programs(rule, programs);
    let runtime = rule
        .definition
        .runtime()
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let mut slots = runtime.state_slots().to_vec();
    slots.push(
        StateSlotDef::new(
            SPORE_SNAPSHOT_SLOT,
            RuleValueKind::Integer,
            BattleRuleScope::Battle,
            RuleValue::Integer(0),
        )
        .with_bounds(RuleValue::Integer(0), RuleValue::Integer(9)),
    );
    slots.sort_unstable_by_key(StateSlotDef::id);
    let mut runtime_triggers = runtime.triggers().to_vec();
    runtime_triggers.extend(triggers);
    runtime_triggers.sort_unstable_by_key(|trigger| trigger.id);
    let mut program_ids = rule.definition.programs().to_vec();
    program_ids.extend([ENGINE_PROGRAM, ENGINE_BURST_PROGRAM]);
    if defeat_spread.is_some() {
        program_ids.push(ENGINE_DEFEAT_PROGRAM);
    }
    program_ids.sort_unstable();
    program_ids.dedup();
    let mut selector_ids = rule.definition.selectors().to_vec();
    selector_ids.extend([
        ENGINE_ACTOR,
        ENGINE_TARGET,
        ENGINE_ADJACENT,
        ENGINE_PRIMARY_AND_ADJACENT,
        ENGINE_OTHER_ENEMIES,
    ]);
    selector_ids.sort_unstable();
    selector_ids.dedup();
    rule.definition =
        RuleDefinition::new(rule.definition.id(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(runtime.source().clone(), slots, runtime_triggers, None),
        );
    Ok(())
}

fn spore_discharge(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let buff_program = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let mut effects = Vec::new();
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut programs = Vec::new();
    let enhanced = whole(parameter(parameters, 3)?)? == 2;
    let mut steps = vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: enemies,
        effect: SPORE_EFFECT,
        stacks: resource_points(false),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })];
    if enhanced {
        let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
        let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
        let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
        groups.push(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(ModifierDefinition {
            id: modifier,
            stat: StatKind::Spd,
            stage: FormulaStage::PercentOfBase,
            purpose: FormulaPurpose::Stat,
            value: scalar(parameter(parameters, 1)?),
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
            EffectDefinition::new(effect, Vec::new(), vec![modifier]).with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::Buff,
                    DispelCategory::DispellableBuff,
                    1,
                    Some(integer(whole(parameter(parameters, 2)?)?)),
                    DurationClock::OwnerTurnEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Replace,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ),
        );
        programs.push(
            ProgramDefinition::new(
                buff_program,
                Vec::new(),
                vec![actor],
                vec![effect],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: actor,
                    effect,
                    stacks: integer(1),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]),
        );
        steps.push(ProgramStep::If {
            condition: ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadResource {
                    selector: actor,
                    resource: RuleResourceKind::SkillPoints,
                }),
                operator: Comparison::Equal,
                rhs: Box::new(scalar(0)),
            },
            then_program: buff_program,
            else_program: None,
        });
    }
    programs.push(
        ProgramDefinition::new(
            program,
            enhanced.then_some(vec![buff_program]).unwrap_or_default(),
            vec![actor, enemies],
            effects.iter().map(EffectDefinition::id).collect(),
            Vec::new(),
        )
        .with_steps(steps),
    );
    Ok(finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        groups,
        modifiers,
        vec![
            SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        effects,
        programs,
        vec![resource_trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            actor,
            false,
            program,
        )],
        Vec::new(),
    ))
}

fn fungal_pustule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let applications = u16::try_from(whole(parameter(parameters, 0)?)?)
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![actor, enemies],
        vec![SPORE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RandomGroupedEffect {
            selector: enemies,
            effect: SPORE_EFFECT,
            groups: resource_points(true),
            applications_per_group: applications,
            stacks: integer(1),
            choice_rng_purpose: DrawPurpose::DAMAGE_TARGET,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            chance_rng_purpose: None,
        },
    )]);
    Ok(finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        Vec::new(),
        vec![definition],
        vec![resource_trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            actor,
            true,
            program,
        )],
        Vec::new(),
    ))
}

fn scythe_limbs(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    has_spend_spores: bool,
    has_recovery_spores: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let arm_program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let spend_program = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let recovery_program = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let clear_program = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let armed = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let effect_stack_slot = id::<StateSlotDefinitionId>(CRIT_STACK_SLOT_BASE, raw)?;
    let enhanced = whole(parameter(parameters, 2)?)? == 2;
    let modifier_definition = ModifierDefinition {
        id: modifier,
        stat: StatKind::CritDamage,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: multiply(
            effect_stack_scalar(effect_stack_slot),
            scalar(parameter(parameters, 0)?),
        ),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::RecomputeOnStackChange,
        source_stack_slot: Some(effect_stack_slot),
        filters: Box::new([]),
    };
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        u16::try_from(whole(parameter(parameters, 1)?)?)
            .map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let arm = ProgramDefinition::new(arm_program, Vec::new(), vec![owner], Vec::new(), Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::SetSlot {
                slot: armed,
                value: integer(1),
            },
        )]);
    let spend = accounting_program(
        spend_program,
        owner,
        enemies,
        effect,
        armed,
        false,
        has_spend_spores,
        SPENT_ACCOUNTING_SIGNAL,
    );
    let recovery = enhanced.then(|| {
        accounting_program(
            recovery_program,
            owner,
            enemies,
            effect,
            armed,
            true,
            has_recovery_spores,
            RECOVERED_ACCOUNTING_SIGNAL,
        )
    });
    let clear = ProgramDefinition::new(
        clear_program,
        Vec::new(),
        vec![owner],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveEffect {
            selector: owner,
            effect,
        },
    )]);
    let mut programs = vec![arm, spend, clear];
    if let Some(recovery) = recovery {
        programs.push(recovery);
    }
    let mut triggers = vec![
        action_trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            owner,
            AbilityTag::Ultimate,
            ConditionExpr::Literal(true),
            arm_program,
        ),
        resource_armed_trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            owner,
            armed,
            false,
            spend_program,
        ),
        action_trigger(
            id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?,
            owner,
            AbilityTag::Attack,
            ConditionExpr::EffectExists {
                selector: owner,
                effect,
            },
            clear_program,
        ),
    ];
    if enhanced {
        triggers.push(resource_armed_trigger(
            id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
            owner,
            armed,
            true,
            recovery_program,
        ));
    }
    Ok(finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        vec![modifier_definition],
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(runtime),
        ],
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

#[allow(clippy::too_many_arguments)]
fn accounting_program(
    program: ProgramId,
    owner: SelectorId,
    enemies: SelectorId,
    effect: EffectDefinitionId,
    armed: StateSlotDefinitionId,
    recovery: bool,
    applies_extra_spores: bool,
    signal: u32,
) -> ProgramDefinition {
    let effective = ValueExpr::Add(Box::new(resource_points(recovery)), Box::new(integer(1)));
    let mut steps = vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect,
            stacks: ValueExpr::Minimum(Box::new(effective.clone()), Box::new(integer(2))),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
            code: signal,
            value: Some(effective),
        }),
    ];
    if applies_extra_spores {
        steps.push(ProgramStep::Operation(if recovery {
            RuleOperationTemplate::RandomGroupedEffect {
                selector: enemies,
                effect: SPORE_EFFECT,
                groups: integer(1),
                applications_per_group: 2,
                stacks: integer(1),
                choice_rng_purpose: DrawPurpose::DAMAGE_TARGET,
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                chance_rng_purpose: None,
            }
        } else {
            RuleOperationTemplate::ApplyEffect {
                selector: enemies,
                effect: SPORE_EFFECT,
                stacks: integer(1),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            }
        }));
    }
    steps.push(ProgramStep::Operation(RuleOperationTemplate::SetSlot {
        slot: armed,
        value: integer(0),
    }));
    let mut effects = vec![effect, SPORE_EFFECT];
    effects.sort_unstable();
    effects.dedup();
    ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner, enemies],
        effects,
        Vec::new(),
    )
    .with_steps(steps)
}

fn passive_rule(binding: &UniverseBattleRuleBinding) -> ExecutableBattleRule {
    finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn finish_rule(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    mut groups: Vec<ModifierStackingGroup>,
    mut modifiers: Vec<ModifierDefinition>,
    mut selectors: Vec<SelectorDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    mut triggers: Vec<TriggerDef>,
    mut slots: Vec<StateSlotDef>,
) -> ExecutableBattleRule {
    groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    slots.sort_unstable_by_key(StateSlotDef::id);
    let program_ids = programs.iter().map(ProgramDefinition::id).collect();
    let selector_ids = selectors.iter().map(SelectorDefinition::id).collect();
    let definition = RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
        BattleRuleDefinition::new(binding.source().clone(), slots, triggers, None),
    );
    ExecutableBattleRule {
        attachment,
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}

fn resource_trigger(
    trigger_id: TriggerId,
    actor: SelectorId,
    recovery: bool,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id: trigger_id,
        event: RuleEventKind::Resource,
        event_point: RuleEventPoint::ResourceChanged,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter {
            actor_selector: Some(actor),
            resource: Some(RuleResourceKind::SkillPoints),
            ..EventFilter::default()
        },
        condition: resource_direction(recovery),
        once_scope: OnceScope::Event,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn resource_armed_trigger(
    trigger_id: TriggerId,
    owner: SelectorId,
    armed: StateSlotDefinitionId,
    recovery: bool,
    program: ProgramId,
) -> TriggerDef {
    let condition = ConditionExpr::All(
        vec![
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(armed)),
                operator: Comparison::Equal,
                rhs: Box::new(integer(1)),
            },
            resource_direction(recovery),
        ]
        .into_boxed_slice(),
    );
    let mut trigger = resource_trigger(trigger_id, owner, recovery, program);
    trigger.condition = condition;
    trigger
}

fn action_trigger(
    trigger_id: TriggerId,
    owner: SelectorId,
    tag: AbilityTag,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id: trigger_id,
        event: RuleEventKind::Action,
        event_point: RuleEventPoint::ActionResolved,
        phase: TriggerPhase::AfterAction,
        filter: EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(tag),
            ..EventFilter::default()
        },
        condition,
        once_scope: OnceScope::Action,
        priority: ReactionPriority::new(0),
        program,
    }
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

fn resource_points(recovery: bool) -> ValueExpr {
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

fn actor_player_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
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

fn event_target_any_life() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Any,
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

fn adjacent_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        0,
        2,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::AdjacentToPrimary,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn primary_and_adjacent_any_life() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Any,
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

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn whole(value: i64) -> Result<i64, BattleRuleLoweringError> {
    if value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    Ok(value / 1_000_000)
}

fn merge_selectors(rule: &mut ExecutableBattleRule, values: Vec<SelectorDefinition>) {
    let mut output = rule.selectors.to_vec();
    output.extend(values);
    output.sort_unstable_by_key(SelectorDefinition::id);
    output.dedup_by_key(|value| value.id());
    rule.selectors = output.into_boxed_slice();
}

fn merge_effects(rule: &mut ExecutableBattleRule, values: Vec<EffectDefinition>) {
    let mut output = rule.effects.to_vec();
    output.extend(values);
    output.sort_unstable_by_key(EffectDefinition::id);
    output.dedup_by_key(|value| value.id());
    rule.effects = output.into_boxed_slice();
}

fn merge_programs(rule: &mut ExecutableBattleRule, values: Vec<ProgramDefinition>) {
    let mut output = rule.programs.to_vec();
    output.extend(values);
    output.sort_unstable_by_key(ProgramDefinition::id);
    output.dedup_by_key(|value| value.id());
    rule.programs = output.into_boxed_slice();
}
