use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
    RuleSelectorSide, RuleUnitSelector,
};

const HEALING_DEWDROP: &str = "StageAbility_612330";
const TURN_DEWDROP: &str = "StageAbility_612331";
const SHARED_HEALING: &str = "StageAbility_612332";
const RUPTURE_HEALING: &str = "StageAbility_612340";
const FULL_HP_EFFICIENCY: &str = "StageAbility_612341";

pub(super) const DEWDROP_CHARGE_SIGNAL: u32 = 0x00ab_0001;
pub(super) const DEWDROP_RUPTURE_SIGNAL: u32 = 0x00ab_0002;
const LOCAL_PROGRAM_BASE: u32 = 0x7930_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let healing = selected(bindings, blessings, HEALING_DEWDROP)?;
    let turn = selected(bindings, blessings, TURN_DEWDROP)?;
    let shared = selected(bindings, blessings, SHARED_HEALING)?;
    let rupture_healing = selected(bindings, blessings, RUPTURE_HEALING)?;
    let efficiency = selected(bindings, blessings, FULL_HP_EFFICIENCY)?;
    let host = healing
        .map(|(binding, _)| binding.rule())
        .or_else(|| turn.map(|(binding, _)| binding.rule()));
    let damage_boost = healing
        .map(|(_, parameters)| parameter(parameters, 1))
        .transpose()?
        .unwrap_or(0);

    let mut output = Vec::new();
    if let Some((binding, parameters)) = healing {
        output.push(healing_dewdrop(
            binding,
            parameters,
            host == Some(binding.rule()),
            damage_boost,
        )?);
    }
    if let Some((binding, parameters)) = turn {
        output.push(turn_dewdrop(
            binding,
            parameters,
            host == Some(binding.rule()),
            damage_boost,
        )?);
    }
    if let Some((binding, parameters)) = shared {
        output.push(shared_healing(binding, parameters)?);
    }
    if let Some((binding, parameters)) = rupture_healing {
        output.push(rupture_healing_rule(binding, parameters)?);
    }
    if let Some((binding, parameters)) = efficiency {
        output.push(full_hp_efficiency(
            binding,
            parameters,
            healing.map(|(_, values)| values),
            turn.map(|(_, values)| values),
        )?);
    }
    Ok(output)
}

fn selected<'a>(
    bindings: &'a [UniverseBattleRuleBinding],
    blessings: &'a BlessingContributionSet,
    key: &str,
) -> Result<Option<(&'a UniverseBattleRuleBinding, &'a [ExactParameter])>, BattleRuleLoweringError>
{
    let Some(binding) = level_binding(bindings, key) else {
        return Ok(None);
    };
    let parameters = selected_level_parameters(blessings, key)
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    Ok(Some((binding, parameters)))
}

fn healing_dewdrop(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    host: bool,
    damage_boost: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let amount = multiply(
        ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount),
        scalar(parameter(parameters, 0)?),
    );
    let mut parts = dewdrop_generator(
        owner,
        root,
        body,
        amount,
        DewdropTrigger {
            point: RuleEventPoint::HealApplied,
            filter: EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
        },
        raw,
    )?;
    if host {
        add_dewdrop_engine(&mut parts, owner, target, raw, damage_boost)?;
    }
    finish(binding, parts)
}

fn turn_dewdrop(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    host: bool,
    damage_boost: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let amount = turn_charge(parameters)?;
    let mut parts = dewdrop_generator(
        owner,
        root,
        body,
        amount,
        DewdropTrigger {
            point: RuleEventPoint::TurnStarted,
            filter: EventFilter {
                actor_selector: Some(owner),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
        },
        raw,
    )?;
    if host {
        add_dewdrop_engine(&mut parts, owner, target, raw, damage_boost)?;
    }
    finish(binding, parts)
}

fn turn_charge(parameters: &[ExactParameter]) -> Result<ValueExpr, BattleRuleLoweringError> {
    let source = match parameter(parameters, 1)? {
        1_000_000 => ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        },
        2_000_000 => ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        _ => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    Ok(multiply(source, scalar(parameter(parameters, 0)?)))
}

fn dewdrop_generator(
    owner: SelectorId,
    root: ProgramId,
    body: ProgramId,
    amount: ValueExpr,
    observed: DewdropTrigger,
    raw: u32,
) -> Result<RuleParts, BattleRuleLoweringError> {
    Ok(RuleParts {
        selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        programs: vec![
            ProgramDefinition::new(root, vec![body], vec![owner], Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::ForEach {
                    selector: owner,
                    body,
                    maximum: 1,
                }]),
            signal_program(body, DEWDROP_CHARGE_SIGNAL, amount),
        ],
        slots: Vec::new(),
        triggers: vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            observed.point,
            OnceScope::Event,
            observed.filter,
            observed.condition,
            root,
        )],
        ..RuleParts::default()
    })
}

struct DewdropTrigger {
    point: RuleEventPoint,
    filter: EventFilter,
    condition: ConditionExpr,
}

fn add_dewdrop_engine(
    parts: &mut RuleParts,
    owner: SelectorId,
    target: SelectorId,
    raw: u32,
    damage_boost: i64,
) -> Result<(), BattleRuleLoweringError> {
    let charge = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let rupture_root = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let rupture_body = id::<ProgramId>(LOCAL_PROGRAM_BASE, raw)?;
    let slot = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    parts
        .selectors
        .push(SelectorDefinition::new(target).with_rule_units(event_attack_target_selector()?));
    parts.programs.extend([
        ProgramDefinition::new(charge, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot,
                value: ValueExpr::Minimum(
                    Box::new(ValueExpr::Add(
                        Box::new(ValueExpr::Slot(slot)),
                        Box::new(ValueExpr::ReadEventProperty(
                            EventValueProperty::RuleSignalValue,
                        )),
                    )),
                    Box::new(ValueExpr::QueryStat {
                        subject: StatQuerySubject::Owner,
                        stat: StatKind::Hp,
                        purpose: FormulaPurpose::Stat,
                    }),
                ),
            })],
        ),
        ProgramDefinition::new(
            rupture_root,
            vec![rupture_body],
            vec![owner, target],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: owner,
            body: rupture_body,
            maximum: 1,
        }]),
        ProgramDefinition::new(
            rupture_body,
            Vec::new(),
            vec![owner, target],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::DamageFromEventElement {
                selector: target,
                amount: multiply(
                    ValueExpr::Slot(slot),
                    scalar(
                        1_000_000_i64
                            .checked_add(damage_boost)
                            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
                    ),
                ),
                class: DamageClass::Additional,
                can_crit: false,
                can_defeat: true,
            }),
            ProgramStep::Operation(RuleOperationTemplate::EmitRuleEvent {
                code: DEWDROP_RUPTURE_SIGNAL,
                value: Some(ValueExpr::Slot(slot)),
            }),
            ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot,
                value: scalar(0),
            }),
        ]),
    ]);
    parts.slots.push(StateSlotDef::new(
        slot,
        RuleValueKind::Scalar,
        BattleRuleScope::Battle,
        RuleValue::Scalar(starclock_combat::Scalar::ZERO),
    ));
    parts.triggers.extend([
        trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::InformationalRule,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            signal_condition(DEWDROP_CHARGE_SIGNAL),
            charge,
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
            positive_slot(slot),
            rupture_root,
        ),
    ]);
    Ok(())
}

fn rupture_healing_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let maximum_hp = ValueExpr::QueryStat {
        subject: StatQuerySubject::Owner,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Healing,
    };
    let minimum = multiply(maximum_hp.clone(), scalar(parameter(parameters, 2)?));
    let maximum = multiply(maximum_hp, scalar(parameter(parameters, 1)?));
    let amount = ValueExpr::Minimum(
        Box::new(ValueExpr::Maximum(
            Box::new(multiply(
                ValueExpr::ReadEventProperty(EventValueProperty::RuleSignalValue),
                scalar(parameter(parameters, 0)?),
            )),
            Box::new(minimum),
        )),
        Box::new(maximum),
    );
    let parts = RuleParts {
        selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        programs: vec![
            ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                    selector: owner,
                    amount,
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
            signal_condition(DEWDROP_RUPTURE_SIGNAL),
            program,
        )],
        ..RuleParts::default()
    };
    finish(binding, parts)
}

fn full_hp_efficiency(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    healing_parameters: Option<&[ExactParameter]>,
    turn_parameters: Option<&[ExactParameter]>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let ratio = scalar(parameter(parameters, 0)?);
    let mut parts = RuleParts {
        selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        ..RuleParts::default()
    };
    if let Some(healing) = healing_parameters {
        let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
        let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
        parts.programs.extend([
            ProgramDefinition::new(root, vec![body], vec![owner], Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::ForEach {
                    selector: owner,
                    body,
                    maximum: 1,
                }]),
            signal_program(
                body,
                DEWDROP_CHARGE_SIGNAL,
                multiply(
                    multiply(
                        ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount),
                        scalar(parameter(healing, 0)?),
                    ),
                    ratio.clone(),
                ),
            ),
        ]);
        parts.triggers.push(trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::HealApplied,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            full_hp_after_heal(),
            root,
        ));
    }
    if let Some(turn) = turn_parameters {
        let root = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
        let body = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
        parts.programs.extend([
            ProgramDefinition::new(root, vec![body], vec![owner], Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::ForEach {
                    selector: owner,
                    body,
                    maximum: 1,
                }]),
            signal_program(
                body,
                DEWDROP_CHARGE_SIGNAL,
                multiply(turn_charge(turn)?, ratio),
            ),
        ]);
        parts.triggers.push(trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                ..EventFilter::default()
            },
            full_hp_now(),
            root,
        ));
    }
    finish(binding, parts)
}

fn shared_healing(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let healed = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let others = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(0x7931_0000, raw)?;
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let apply = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let stack_slot = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let mut selectors = vec![
        SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
        SelectorDefinition::new(healed).with_rule_units(event_target_ally_selector()?),
        SelectorDefinition::new(others).with_rule_units(other_allies_selector(healed)?),
        SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
        SelectorDefinition::new(current).with_rule_units(current_ally_selector()?),
    ];
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    let shared_amount = multiply(
        ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount),
        scalar(parameter(parameters, 0)?),
    );
    let mut root_steps = vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
        selector: others,
        amount: shared_amount,
        apply_formula_modifiers: false,
    })];
    let mut effects = Vec::new();
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut programs = Vec::new();
    let mut slots = Vec::new();
    if parameter(parameters, 1)? > 0 {
        root_steps.push(ProgramStep::ForEach {
            selector: allies,
            body,
            maximum: 16,
        });
        let current_stacks = ValueExpr::Convert {
            value: Box::new(ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::CurrentTarget,
                effect,
            }),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        };
        let cap = multiply(
            ValueExpr::QueryBaseStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Atk,
            },
            scalar(parameter(parameters, 2)?),
        );
        let remaining = ValueExpr::Maximum(
            Box::new(scalar(0)),
            Box::new(ValueExpr::Subtract(Box::new(cap), Box::new(current_stacks))),
        );
        let gain = ValueExpr::Minimum(
            Box::new(multiply(
                ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount),
                scalar(parameter(parameters, 1)?),
            )),
            Box::new(remaining),
        );
        let stack_gain = ValueExpr::Convert {
            value: Box::new(gain.clone()),
            target: RuleValueKind::Integer,
            rounding: Rounding::Floor,
        };
        programs.extend([
            ProgramDefinition::new(body, vec![apply], vec![current], vec![effect], Vec::new())
                .with_steps(vec![ProgramStep::If {
                    condition: ConditionExpr::Compare {
                        lhs: Box::new(gain),
                        operator: Comparison::Greater,
                        rhs: Box::new(scalar(0)),
                    },
                    then_program: apply,
                    else_program: None,
                }]),
            ProgramDefinition::new(apply, Vec::new(), vec![current], vec![effect], Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::ApplyEffect {
                        selector: current,
                        effect,
                        stacks: stack_gain,
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    },
                )]),
        ]);
        groups.push(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(ModifierDefinition {
            id: modifier,
            stat: StatKind::Atk,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(stack_slot)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            },
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::RecomputeOnStackChange,
            source_stack_slot: Some(stack_slot),
            filters: Box::new([]),
        });
        let runtime = EffectRuntimeTemplate::new(
            EffectCategory::Buff,
            DispelCategory::DispellableBuff,
            u16::MAX,
            Some(ValueExpr::Literal(RuleValue::Integer(2))),
            DurationClock::OwnerTurnEnd,
            EffectTickPhase::None,
            EffectStackPolicy::RefreshAndAddStacks,
        )
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
        effects.push(
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(runtime),
        );
        slots.push(
            StateSlotDef::new(
                stack_slot,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(
                RuleValue::Integer(0),
                RuleValue::Integer(i64::from(u16::MAX)),
            ),
        );
    }
    programs.insert(
        0,
        ProgramDefinition::new(
            root,
            programs.iter().map(ProgramDefinition::id).collect(),
            vec![owner, healed, others, allies, current],
            effects.iter().map(EffectDefinition::id).collect(),
            Vec::new(),
        )
        .with_steps(root_steps),
    );
    let trigger = trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::HealApplied,
        OnceScope::Event,
        EventFilter {
            target_selector: Some(owner),
            excluded_source: Some(binding.source().definition()),
            ..EventFilter::default()
        },
        ConditionExpr::Compare {
            lhs: Box::new(ValueExpr::ReadEventProperty(
                EventValueProperty::HpChangeAmount,
            )),
            operator: Comparison::Greater,
            rhs: Box::new(scalar(0)),
        },
        root,
    );
    finish(
        binding,
        RuleParts {
            groups,
            modifiers,
            selectors,
            effects,
            programs,
            slots,
            triggers: vec![trigger],
        },
    )
}

fn signal_program(program: ProgramId, code: u32, value: ValueExpr) -> ProgramDefinition {
    ProgramDefinition::new(program, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(
        vec![ProgramStep::Operation(
            RuleOperationTemplate::EmitRuleEvent {
                code,
                value: Some(value),
            },
        )],
    )
}

fn signal_condition(code: u32) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::RuleSignalCode,
        )),
        operator: Comparison::Equal,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(i64::from(code)))),
    }
}

fn positive_slot(slot: StateSlotDefinitionId) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    }
}

fn full_hp_after_heal() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::HpAfter)),
        operator: Comparison::GreaterOrEqual,
        rhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
    }
}

fn full_hp_now() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        }),
        operator: Comparison::GreaterOrEqual,
        rhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
    }
}

fn event_target_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

fn event_attack_target_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        1,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("bounce-target".into()),
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn other_allies_selector(
    excluded: SelectorId,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    Ok(selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::All,
        16,
    )?
    .with_predicates(vec![RuleSelectorPredicate::Excludes(excluded)]))
}

fn current_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::CurrentSubject,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    choice: RuleSelectorChoice,
    maximum: u16,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
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
    parts.selectors.dedup_by_key(|selector| selector.id());
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
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
