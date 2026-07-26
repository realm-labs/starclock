use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
    RuleSelectorSide, RuleUnitSelector,
};

const VIRTUAL_GRIT_NORMAL: &str = "StageAbility_61253001";
const VIRTUAL_GRIT_ENHANCED: &str = "StageAbility_61253002";
const GRIT_ON_HIT_NORMAL: &str = "StageAbility_61253101";
const GRIT_ON_HIT_ENHANCED: &str = "StageAbility_61253102";
const DAMAGE_SHARE_NORMAL: &str = "StageAbility_61253201";
const DAMAGE_SHARE_ENHANCED: &str = "StageAbility_61253202";
const GRIT_RETALIATION_NORMAL: &str = "StageAbility_61254001";
const GRIT_RETALIATION_ENHANCED: &str = "StageAbility_61254002";
const HP_CONSUMPTION_NORMAL: &str = "StageAbility_61254101";
const HP_CONSUMPTION_ENHANCED: &str = "StageAbility_61254102";
const GRIT_MITIGATION_NORMAL: &str = "StageAbility_61254201";
const GRIT_MITIGATION_ENHANCED: &str = "StageAbility_61254202";

pub(super) const GRIT_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x79d0_0001).expect("reserved effect ID");
pub(super) const VIRTUAL_GRIT_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x79d0_0002).expect("reserved effect ID");
const GRIT_ENGINE_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x79d0_0003).expect("reserved effect ID");
const GRIT_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x79d0_0004).expect("reserved slot ID");
const GRIT_ATTACK_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x79d0_0005).expect("reserved group ID");
const GRIT_DEFENSE_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x79d0_0006).expect("reserved group ID");
const GRIT_MITIGATION_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x79d0_0007).expect("reserved group ID");
const GRIT_ATTACK_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x79d0_0008).expect("reserved modifier ID");
const GRIT_DEFENSE_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x79d0_0009).expect("reserved modifier ID");
const GRIT_ENGINE_SELECTOR: SelectorId =
    SelectorId::new(0x79d0_000a).expect("reserved selector ID");
const GRIT_ENGINE_PROGRAM: ProgramId = ProgramId::new(0x79d0_000b).expect("reserved program ID");
const GRIT_ENGINE_APPLY_PROGRAM: ProgramId =
    ProgramId::new(0x79d0_000c).expect("reserved program ID");
const GRIT_ENGINE_CLEAR_PROGRAM: ProgramId =
    ProgramId::new(0x79d0_000d).expect("reserved program ID");
const GRIT_ENGINE_TRIGGER_BASE: u32 = 0x79d0_0010;
const LOCAL_ID_BASE: u32 = 0x79e0_0000;
const CONTRIBUTION_RULE_ID_BASE: u32 = 0x7000_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        VIRTUAL_GRIT_NORMAL,
        VIRTUAL_GRIT_ENHANCED,
        GRIT_ON_HIT_NORMAL,
        GRIT_ON_HIT_ENHANCED,
        DAMAGE_SHARE_NORMAL,
        DAMAGE_SHARE_ENHANCED,
        GRIT_RETALIATION_NORMAL,
        GRIT_RETALIATION_ENHANCED,
        HP_CONSUMPTION_NORMAL,
        HP_CONSUMPTION_ENHANCED,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            VIRTUAL_GRIT_NORMAL | VIRTUAL_GRIT_ENHANCED => virtual_grit(binding, parameters)?,
            GRIT_ON_HIT_NORMAL | GRIT_ON_HIT_ENHANCED => grit_on_hit(binding, parameters)?,
            DAMAGE_SHARE_NORMAL | DAMAGE_SHARE_ENHANCED => damage_share(binding, parameters)?,
            GRIT_RETALIATION_NORMAL | GRIT_RETALIATION_ENHANCED => {
                grit_retaliation(binding, parameters)?
            }
            HP_CONSUMPTION_NORMAL | HP_CONSUMPTION_ENHANCED => hp_consumption(binding, parameters)?,
            _ => unreachable!("closed Destruction S01 binding set"),
        });
    }
    Ok(output)
}

fn virtual_grit(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let apply = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let clear = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let hp_ratio = hp_ratio(StatQuerySubject::Owner);
    let threshold = scalar(parameter(parameters, 5)?);
    let stacks = if parameters.len() == 9 {
        let intervals = ValueExpr::Convert {
            value: Box::new(ValueExpr::Divide {
                lhs: Box::new(ValueExpr::Subtract(
                    Box::new(threshold.clone()),
                    Box::new(hp_ratio.clone()),
                )),
                rhs: Box::new(scalar(parameter(parameters, 7)?)),
                rounding: Rounding::Floor,
            }),
            target: RuleValueKind::Integer,
            rounding: Rounding::Floor,
        };
        ValueExpr::Add(
            Box::new(integer(whole(parameter(parameters, 4)?)?)),
            Box::new(ValueExpr::Multiply {
                lhs: Box::new(intervals),
                rhs: Box::new(integer(whole(parameter(parameters, 8)?)?)),
                rounding: Rounding::TowardZero,
            }),
        )
    } else {
        integer(whole(parameter(parameters, 4)?)?)
    };
    let apply_program = ProgramDefinition::new(
        apply,
        Vec::new(),
        vec![owner],
        vec![VIRTUAL_GRIT_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: owner,
            effect: VIRTUAL_GRIT_EFFECT,
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: VIRTUAL_GRIT_EFFECT,
            stacks,
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
    ]);
    let clear_program = ProgramDefinition::new(
        clear,
        Vec::new(),
        vec![owner],
        vec![VIRTUAL_GRIT_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveEffect {
            selector: owner,
            effect: VIRTUAL_GRIT_EFFECT,
        },
    )]);
    let root_program = ProgramDefinition::new(
        root,
        vec![apply, clear],
        vec![owner],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::If {
        condition: ConditionExpr::Compare {
            lhs: Box::new(hp_ratio),
            operator: Comparison::Less,
            rhs: Box::new(threshold),
        },
        then_program: apply,
        else_program: Some(clear),
    }]);
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![root_program, apply_program, clear_program],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    root,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HpChanged,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    root,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn grit_on_hit(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let adjacent = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let gain = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let decay = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner_stacks = whole(parameter(parameters, 4)?)?;
    let adjacent_stacks = parameters
        .get(6)
        .map(|_| whole(parameter(parameters, 6)?))
        .transpose()?
        .unwrap_or(0);
    let mut gain_steps = vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: owner,
        effect: GRIT_EFFECT,
        stacks: integer(owner_stacks),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })];
    if adjacent_stacks > 0 {
        gain_steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: adjacent,
            effect: GRIT_EFFECT,
            stacks: integer(adjacent_stacks),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }));
    }
    let gain_program = ProgramDefinition::new(
        gain,
        Vec::new(),
        vec![owner, adjacent],
        vec![GRIT_EFFECT],
        Vec::new(),
    )
    .with_steps(gain_steps);
    let decay_program = ProgramDefinition::new(
        decay,
        Vec::new(),
        vec![owner],
        vec![GRIT_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::AdjustEffectStacks {
            selector: owner,
            effect: GRIT_EFFECT,
            delta: integer(-whole(parameter(parameters, 4)?)?),
        },
    )]);
    finish(
        binding,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(adjacent).with_rule_units(adjacent_allies_selector()?),
            ],
            programs: vec![gain_program, decay_program],
            triggers: vec![
                trigger_with_condition(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::DamageApplied,
                    OnceScope::Action,
                    EventFilter {
                        target_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    gain,
                ),
                trigger_with_condition(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HpChanged,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(owner),
                        target_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    negative_hp_change(),
                    gain,
                ),
                trigger(
                    id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnEnded,
                    OnceScope::Turn,
                    EventFilter {
                        owner_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    decay,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn damage_share(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let others = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let distribute = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let stack_slot = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let reduction = parameters
        .first()
        .map(|_| parameter(parameters, 0))
        .transpose()?
        .unwrap_or(0);
    let divisor = ValueExpr::Convert {
        value: Box::new(ValueExpr::Slot(stack_slot)),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    };
    let mitigation = ValueExpr::Subtract(
        Box::new(scalar(1_000_000)),
        Box::new(ValueExpr::Divide {
            lhs: Box::new(scalar(1_000_000 - reduction)),
            rhs: Box::new(divisor),
            rounding: Rounding::NearestTiesEven,
        }),
    );
    let purposes = damage_purposes();
    let modifiers = purposes
        .iter()
        .enumerate()
        .map(|(index, purpose)| {
            Ok(ModifierDefinition {
                id: local_id::<ModifierDefinitionId>(raw, index as u32)?,
                stat: StatKind::Hp,
                stage: FormulaStage::Mitigation,
                purpose: *purpose,
                value: mitigation.clone(),
                stacking_group: group,
                priority: 0,
                floor: Some(starclock_combat::Scalar::ZERO),
                cap: Some(starclock_combat::Scalar::ONE),
                cap_stage: FormulaStage::Mitigation,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: Some(stack_slot),
                filters: Box::new([]),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|value| value.id).collect();
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        16,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let apply_program = ProgramDefinition::new(
        apply,
        Vec::new(),
        vec![owner, allies],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect,
            stacks: ValueExpr::SelectorCount(allies),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )]);
    let distribute_program =
        ProgramDefinition::new(distribute, Vec::new(), vec![others], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::TrueDamage {
                    selector: others,
                    amount: ValueExpr::ReadEventProperty(EventValueProperty::DamageAmount),
                },
            )]);
    finish(
        binding,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers,
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
                SelectorDefinition::new(others).with_rule_units(other_allies_selector(owner)?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), modifier_ids)
                    .with_runtime_template(runtime),
            ],
            programs: vec![apply_program, distribute_program],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    apply,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::DamageApplied,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    distribute,
                ),
                trigger(
                    id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::UnitDefeated,
                    OnceScope::Event,
                    EventFilter::default(),
                    apply,
                ),
                trigger(
                    id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::PresenceChanged,
                    OnceScope::Event,
                    EventFilter::default(),
                    apply,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn grit_retaliation(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let attacker = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let stacks = effective_grit(StatQuerySubject::Owner);
    let attack = multiply(
        multiply(
            ValueExpr::QueryBaseStat {
                subject: StatQuerySubject::Owner,
                stat: StatKind::Atk,
            },
            scalar(parameter(parameters, 0)?),
        ),
        stacks.clone(),
    );
    let missing_hp = ValueExpr::Subtract(
        Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        Box::new(ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        }),
    );
    let amount = if parameters.len() == 2 {
        ValueExpr::Add(
            Box::new(attack),
            Box::new(multiply(
                multiply(missing_hp, scalar(parameter(parameters, 1)?)),
                stacks,
            )),
        )
    } else {
        attack
    };
    let definition =
        ProgramDefinition::new(program, Vec::new(), vec![attacker], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DamageFromEventElement {
                    selector: attacker,
                    amount,
                    class: DamageClass::Additional,
                    can_crit: false,
                    can_defeat: false,
                },
            )]);
    finish(
        binding,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(attacker).with_rule_units(actor_enemy_selector()?),
            ],
            programs: vec![definition],
            triggers: vec![trigger_with_condition(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::Action,
                EventFilter {
                    target_selector: Some(owner),
                    excluded_source: Some(binding.source().definition()),
                    ..EventFilter::default()
                },
                has_grit(),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn hp_consumption(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let prepare = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let clear = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let damage = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let amount_slot = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let hp_cost = multiply(
        ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        },
        scalar(parameter(parameters, 0)?),
    );
    let ratio = if parameters.len() == 3 {
        ValueExpr::Add(
            Box::new(scalar(parameter(parameters, 1)?)),
            Box::new(multiply(
                effective_grit(StatQuerySubject::Owner),
                scalar(parameter(parameters, 2)?),
            )),
        )
    } else {
        scalar(parameter(parameters, 1)?)
    };
    let root_program = ProgramDefinition::new(
        root,
        vec![prepare, clear],
        vec![owner],
        vec![GRIT_EFFECT, VIRTUAL_GRIT_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::If {
        condition: has_grit(),
        then_program: prepare,
        else_program: Some(clear),
    }]);
    let prepare_program =
        ProgramDefinition::new(prepare, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: amount_slot,
                    value: hp_cost.clone(),
                }),
                ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
                    selector: owner,
                    amount: hp_cost,
                    floor: scalar(1_000_000),
                }),
            ]);
    let clear_program =
        ProgramDefinition::new(clear, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: amount_slot,
                value: scalar(0),
            })],
        );
    let damage_program =
        ProgramDefinition::new(damage, Vec::new(), vec![target], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DamageFromEventElement {
                    selector: target,
                    amount: multiply(ValueExpr::Slot(amount_slot), ratio),
                    class: DamageClass::Additional,
                    can_crit: false,
                    can_defeat: true,
                },
            )]);
    finish(
        binding,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(target).with_rule_units(primary_target_selector()?),
            ],
            programs: vec![root_program, prepare_program, clear_program, damage_program],
            slots: vec![StateSlotDef::new(
                amount_slot,
                RuleValueKind::Scalar,
                BattleRuleScope::Battle,
                RuleValue::Scalar(starclock_combat::Scalar::ZERO),
            )],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionStarted,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(owner),
                        ability_tag: Some(AbilityTag::Attack),
                        ..EventFilter::default()
                    },
                    root,
                ),
                trigger_with_condition(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::DamageApplied,
                    OnceScope::TargetWithinAction,
                    EventFilter {
                        actor_selector: Some(owner),
                        ability_tag: Some(AbilityTag::Attack),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    positive_scalar_slot(amount_slot),
                    damage,
                ),
                trigger(
                    id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionResolved,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(owner),
                        ability_tag: Some(AbilityTag::Attack),
                        ..EventFilter::default()
                    },
                    clear,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

pub(super) fn add_grit_engine(
    rule: &mut ExecutableBattleRule,
    blessings: &BlessingContributionSet,
) -> Result<(), BattleRuleLoweringError> {
    let maximum = if selected_level_parameters(blessings, GRIT_MITIGATION_ENHANCED).is_some() {
        45
    } else {
        35
    };
    let grit_ratio = [
        VIRTUAL_GRIT_NORMAL,
        VIRTUAL_GRIT_ENHANCED,
        GRIT_ON_HIT_NORMAL,
        GRIT_ON_HIT_ENHANCED,
    ]
    .into_iter()
    .find_map(|key| selected_level_parameters(blessings, key))
    .map(|parameters| Ok((parameter(parameters, 0)?, parameter(parameters, 1)?)))
    .transpose()?;
    let mitigation = selected_level_parameters(blessings, GRIT_MITIGATION_NORMAL)
        .or_else(|| selected_level_parameters(blessings, GRIT_MITIGATION_ENHANCED))
        .map(|parameters| parameter_six(parameters, 0))
        .transpose()?;
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    if let Some((attack, defense)) = grit_ratio {
        groups.extend([
            ModifierStackingGroup {
                id: GRIT_ATTACK_GROUP,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            },
            ModifierStackingGroup {
                id: GRIT_DEFENSE_GROUP,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            },
        ]);
        modifiers.extend([
            grit_stat_modifier(
                GRIT_ATTACK_MODIFIER,
                GRIT_ATTACK_GROUP,
                StatKind::Atk,
                attack,
            ),
            grit_stat_modifier(
                GRIT_DEFENSE_MODIFIER,
                GRIT_DEFENSE_GROUP,
                StatKind::Def,
                defense,
            ),
        ]);
    }
    if let Some(ratio) = mitigation {
        groups.push(ModifierStackingGroup {
            id: GRIT_MITIGATION_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        for (index, purpose) in damage_purposes().into_iter().enumerate() {
            modifiers.push(ModifierDefinition {
                id: ModifierDefinitionId::new(
                    0x79d1_0000
                        + u32::try_from(index)
                            .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
                stat: StatKind::Hp,
                stage: FormulaStage::Mitigation,
                purpose,
                value: multiply(stack_slot_scalar(), scalar(ratio)),
                stacking_group: GRIT_MITIGATION_GROUP,
                priority: 0,
                floor: Some(starclock_combat::Scalar::ZERO),
                cap: Some(starclock_combat::Scalar::ONE),
                cap_stage: FormulaStage::Mitigation,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: Some(GRIT_STACK_SLOT),
                filters: Box::new([]),
            });
        }
    }
    let stack_runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        maximum,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let engine_runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        maximum,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    merge_groups(rule, groups);
    merge_modifiers(rule, modifiers);
    let mut effects = rule.effects.to_vec();
    effects.extend([
        EffectDefinition::new(GRIT_EFFECT, Vec::new(), Vec::new())
            .with_runtime_template(stack_runtime.clone()),
        EffectDefinition::new(VIRTUAL_GRIT_EFFECT, Vec::new(), Vec::new())
            .with_runtime_template(stack_runtime),
        EffectDefinition::new(GRIT_ENGINE_EFFECT, Vec::new(), modifier_ids)
            .with_runtime_template(engine_runtime),
    ]);
    effects.sort_unstable_by_key(EffectDefinition::id);
    effects.dedup_by_key(|effect| effect.id());
    rule.effects = effects.into_boxed_slice();

    let selector = SelectorDefinition::new(GRIT_ENGINE_SELECTOR).with_rule_units(owner_selector()?);
    let program = ProgramDefinition::new(
        GRIT_ENGINE_PROGRAM,
        vec![GRIT_ENGINE_APPLY_PROGRAM, GRIT_ENGINE_CLEAR_PROGRAM],
        vec![GRIT_ENGINE_SELECTOR],
        vec![GRIT_EFFECT, VIRTUAL_GRIT_EFFECT, GRIT_ENGINE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::If {
        condition: has_grit(),
        then_program: GRIT_ENGINE_APPLY_PROGRAM,
        else_program: Some(GRIT_ENGINE_CLEAR_PROGRAM),
    }]);
    let apply_program = ProgramDefinition::new(
        GRIT_ENGINE_APPLY_PROGRAM,
        Vec::new(),
        vec![GRIT_ENGINE_SELECTOR],
        vec![GRIT_EFFECT, VIRTUAL_GRIT_EFFECT, GRIT_ENGINE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyEffect {
            selector: GRIT_ENGINE_SELECTOR,
            effect: GRIT_ENGINE_EFFECT,
            stacks: effective_grit_integer(StatQuerySubject::Owner),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        },
    )]);
    let clear_program = ProgramDefinition::new(
        GRIT_ENGINE_CLEAR_PROGRAM,
        Vec::new(),
        vec![GRIT_ENGINE_SELECTOR],
        vec![GRIT_ENGINE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveEffect {
            selector: GRIT_ENGINE_SELECTOR,
            effect: GRIT_ENGINE_EFFECT,
        },
    )]);
    let mut selectors = rule.selectors.to_vec();
    selectors.push(selector);
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    selectors.dedup_by_key(|value| value.id());
    rule.selectors = selectors.into_boxed_slice();
    let mut programs = rule.programs.to_vec();
    programs.extend([program, apply_program, clear_program]);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    programs.dedup_by_key(|value| value.id());
    rule.programs = programs.into_boxed_slice();

    let runtime = rule
        .definition
        .runtime()
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let mut slots = runtime.state_slots().to_vec();
    let mut triggers = runtime.triggers().to_vec();
    for (index, (point, effect)) in [
        (RuleEventPoint::EffectApplied, GRIT_EFFECT),
        (RuleEventPoint::EffectRemoved, GRIT_EFFECT),
        (RuleEventPoint::EffectStacksChanged, GRIT_EFFECT),
        (RuleEventPoint::EffectApplied, VIRTUAL_GRIT_EFFECT),
        (RuleEventPoint::EffectRemoved, VIRTUAL_GRIT_EFFECT),
        (RuleEventPoint::EffectStacksChanged, VIRTUAL_GRIT_EFFECT),
    ]
    .into_iter()
    .enumerate()
    {
        triggers.push(trigger(
            TriggerId::new(
                GRIT_ENGINE_TRIGGER_BASE
                    + u32::try_from(index)
                        .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            point,
            OnceScope::Event,
            EventFilter {
                effect_definition: Some(effect),
                target_selector: Some(GRIT_ENGINE_SELECTOR),
                ..EventFilter::default()
            },
            GRIT_ENGINE_PROGRAM,
        ));
    }
    triggers.sort_unstable_by_key(|value| value.id);
    let mut program_ids = rule.definition.programs().to_vec();
    program_ids.extend([
        GRIT_ENGINE_PROGRAM,
        GRIT_ENGINE_APPLY_PROGRAM,
        GRIT_ENGINE_CLEAR_PROGRAM,
    ]);
    program_ids.sort_unstable();
    program_ids.dedup();
    let mut selector_ids = rule.definition.selectors().to_vec();
    selector_ids.push(GRIT_ENGINE_SELECTOR);
    selector_ids.sort_unstable();
    selector_ids.dedup();
    rule.definition = RuleDefinition::new(rule.definition.id(), program_ids, selector_ids)
        .with_runtime(BattleRuleDefinition::new(
            runtime.source().clone(),
            std::mem::take(&mut slots),
            triggers,
            None,
        ));
    Ok(())
}

fn grit_stat_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    ratio: i64,
) -> ModifierDefinition {
    ModifierDefinition {
        id,
        stat,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: multiply(stack_slot_scalar(), scalar(ratio)),
        stacking_group: group,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: Some(GRIT_STACK_SLOT),
        filters: Box::new([]),
    }
}

fn effective_grit(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::Convert {
        value: Box::new(effective_grit_integer(subject)),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    }
}

fn effective_grit_integer(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::Maximum(
        Box::new(ValueExpr::QueryEffectStacks {
            subject,
            effect: GRIT_EFFECT,
        }),
        Box::new(ValueExpr::QueryEffectStacks {
            subject,
            effect: VIRTUAL_GRIT_EFFECT,
        }),
    )
}

fn stack_slot_scalar() -> ValueExpr {
    ValueExpr::Convert {
        value: Box::new(ValueExpr::Slot(GRIT_STACK_SLOT)),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    }
}

fn has_grit() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(effective_grit(StatQuerySubject::Owner)),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    }
}

fn hp_ratio(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::Divide {
        lhs: Box::new(ValueExpr::QueryHp { subject }),
        rhs: Box::new(ValueExpr::QueryStat {
            subject,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rounding: Rounding::NearestTiesEven,
    }
}

fn negative_hp_change() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::HpChangeAmount,
        )),
        operator: Comparison::Less,
        rhs: Box::new(scalar(0)),
    }
}

fn positive_scalar_slot(slot: StateSlotDefinitionId) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
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

fn adjacent_allies_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Same,
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

fn other_allies_selector(owner: SelectorId) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    Ok(all_ally_selector()?.with_predicates(vec![RuleSelectorPredicate::Excludes(owner)]))
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

pub(super) fn parameter_six(
    parameters: &[ExactParameter],
    index: usize,
) -> Result<i64, BattleRuleLoweringError> {
    let value = *parameters
        .get(index)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    if value.scale() <= 6 {
        return parameter(parameters, index);
    }
    if value.coefficient() < 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let divisor = 10_i64
        .checked_pow(u32::from(value.scale() - 6))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let quotient = value.coefficient() / divisor;
    let remainder = value.coefficient() % divisor;
    let doubled = remainder
        .checked_mul(2)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    if doubled > divisor || doubled == divisor && quotient % 2 != 0 {
        quotient
            .checked_add(1)
            .ok_or(BattleRuleLoweringError::InvalidParameter)
    } else {
        Ok(quotient)
    }
}

fn local_id<T>(raw: u32, index: u32) -> Result<T, BattleRuleLoweringError>
where
    T: TryFrom<u32>,
{
    let offset = raw
        .checked_sub(CONTRIBUTION_RULE_ID_BASE)
        .and_then(|value| value.checked_mul(16))
        .and_then(|value| value.checked_add(index))
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    LOCAL_ID_BASE
        .checked_add(offset)
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn trigger(
    id: TriggerId,
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
    program: ProgramId,
) -> TriggerDef {
    trigger_with_condition(
        id,
        point,
        once_scope,
        filter,
        ConditionExpr::Literal(true),
        program,
    )
}

fn trigger_with_condition(
    id: TriggerId,
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase: TriggerPhase::AfterEvent,
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
    mut parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    parts.selectors.sort_unstable_by_key(SelectorDefinition::id);
    parts.selectors.dedup_by_key(|value| value.id());
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
    parts.triggers.sort_unstable_by_key(|value| value.id);
    let selector_ids = parts.selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = parts.programs.iter().map(ProgramDefinition::id).collect();
    let definition = RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
        BattleRuleDefinition::new(binding.source().clone(), parts.slots, parts.triggers, None),
    );
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryPlayer,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}

fn merge_groups(rule: &mut ExecutableBattleRule, groups: Vec<ModifierStackingGroup>) {
    let mut merged = rule.modifier_groups.to_vec();
    merged.extend(groups);
    merged.sort_unstable_by_key(|value| value.id);
    merged.dedup_by_key(|value| value.id);
    rule.modifier_groups = merged.into_boxed_slice();
}

fn merge_modifiers(rule: &mut ExecutableBattleRule, modifiers: Vec<ModifierDefinition>) {
    let mut merged = rule.modifiers.to_vec();
    merged.extend(modifiers);
    merged.sort_unstable_by_key(|value| value.id);
    merged.dedup_by_key(|value| value.id);
    rule.modifiers = merged.into_boxed_slice();
}
