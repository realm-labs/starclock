use super::preservation_s03::timed_shield_rule;
use super::*;
use starclock_combat::{
    EffectDamageGuard,
    catalog::selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
};

const DEFEAT_PREVENTION: &str = "StageAbility_61255101";
const MAXIMUM_HP: &str = "StageAbility_61255201";
const HIT_ENERGY: &str = "StageAbility_61255301";
const ENTRY_SHIELD: &str = "StageAbility_61255401";
const LOW_HP_SHIELD: &str = "StageAbility_61255501";

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        DEFEAT_PREVENTION,
        MAXIMUM_HP,
        HIT_ENERGY,
        ENTRY_SHIELD,
        LOW_HP_SHIELD,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            DEFEAT_PREVENTION => defeat_prevention(binding, parameters)?,
            MAXIMUM_HP => persistent_modifier_rule(
                binding,
                StatKind::Hp,
                FormulaStage::PercentOfBase,
                FormulaPurpose::Stat,
                scalar(parameter(parameters, 0)?),
                Vec::new(),
            )?,
            HIT_ENERGY => hit_energy(binding, parameters)?,
            ENTRY_SHIELD => entry_shield(binding, parameters)?,
            LOW_HP_SHIELD => low_hp_shield(binding, parameters)?,
            _ => unreachable!("closed Destruction S03 binding set"),
        });
    }
    Ok(output)
}

fn defeat_prevention(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    if whole(parameter(parameters, 1)?)? != 1 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let setup = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let heal = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
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
    let setup_program =
        ProgramDefinition::new(setup, Vec::new(), vec![allies], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: allies,
                    effect,
                    stacks: integer(1),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    let heal_program =
        ProgramDefinition::new(heal, Vec::new(), vec![target], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: target,
                amount: multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::EventTarget,
                        stat: StatKind::Hp,
                        purpose: FormulaPurpose::Stat,
                    },
                    scalar(parameter(parameters, 0)?),
                ),
                apply_formula_modifiers: false,
            })],
        );
    let signal = ConditionExpr::All(
        vec![
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadEventProperty(
                    EventValueProperty::RuleSignalCode,
                )),
                operator: Comparison::Equal,
                rhs: Box::new(integer(i64::from(
                    starclock_combat::TEAM_DEFEAT_GUARDED_SIGNAL,
                ))),
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadEventProperty(
                    EventValueProperty::RuleSignalValue,
                )),
                operator: Comparison::Equal,
                rhs: Box::new(ValueExpr::Literal(RuleValue::StableId(u64::from(
                    effect.get(),
                )))),
            },
        ]
        .into_boxed_slice(),
    );
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
                SelectorDefinition::new(target).with_rule_units(event_target_ally_selector()?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), Vec::new())
                    .with_runtime_template(runtime),
            ],
            programs: vec![setup_program, heal_program],
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
                    RuleEventPoint::InformationalRule,
                    OnceScope::Battle,
                    EventFilter {
                        target_selector: Some(target),
                        ..EventFilter::default()
                    },
                    signal,
                    heal,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn hit_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let gain = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let granted = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let gain_program =
        ProgramDefinition::new(gain, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![
                ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                    slot: granted,
                    value: integer(1),
                }),
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Gain,
                    amount: scalar(parameter(parameters, 0)?),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                }),
            ],
        );
    let not_granted = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(granted)),
        operator: Comparison::Equal,
        rhs: Box::new(integer(0)),
    };
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![gain_program],
            slots: vec![
                StateSlotDef::new(
                    granted,
                    RuleValueKind::Integer,
                    BattleRuleScope::Action,
                    RuleValue::Integer(0),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1))
                .with_reset_points(vec![SlotResetPoint::ActionStart]),
            ],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::DamageApplied,
                    OnceScope::Event,
                    EventFilter {
                        target_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    not_granted.clone(),
                    gain,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::HpChanged,
                    OnceScope::Event,
                    EventFilter {
                        actor_selector: Some(owner),
                        target_selector: Some(owner),
                        excluded_source: Some(binding.source().definition()),
                        ..EventFilter::default()
                    },
                    ConditionExpr::All(vec![negative_hp_change(), not_granted].into_boxed_slice()),
                    gain,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn entry_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    timed_shield_rule(
        binding,
        multiply(missing_hp(), scalar(parameter(parameters, 0)?)),
        whole(parameter(parameters, 1)?)?,
        RuleEventPoint::BattleStarted,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn low_hp_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    if whole(parameter(parameters, 3)?)? != 1 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let used = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let condition = ConditionExpr::All(
        vec![
            ConditionExpr::Compare {
                lhs: Box::new(hp_ratio()),
                operator: Comparison::Less,
                rhs: Box::new(scalar(parameter(parameters, 0)?)),
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(used)),
                operator: Comparison::Equal,
                rhs: Box::new(integer(0)),
            },
        ]
        .into_boxed_slice(),
    );
    timed_shield_rule(
        binding,
        multiply(maximum_hp(), scalar(parameter(parameters, 1)?)),
        whole(parameter(parameters, 2)?)?,
        RuleEventPoint::DamageApplied,
        EventFilter {
            target_selector: Some(owner),
            excluded_source: Some(binding.source().definition()),
            ..EventFilter::default()
        },
        condition,
        Vec::new(),
        Vec::new(),
        vec![
            StateSlotDef::new(
                used,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
        ],
        vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: used,
            value: integer(1),
        })],
        Vec::new(),
    )
}

fn maximum_hp() -> ValueExpr {
    ValueExpr::QueryStat {
        subject: StatQuerySubject::Owner,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
    }
}

fn missing_hp() -> ValueExpr {
    ValueExpr::Subtract(
        Box::new(maximum_hp()),
        Box::new(ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        }),
    )
}

fn hp_ratio() -> ValueExpr {
    ValueExpr::Divide {
        lhs: Box::new(ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        }),
        rhs: Box::new(maximum_hp()),
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

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    let value = value
        .checked_div(1_000_000)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    u16::try_from(value).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

fn trigger(
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

fn event_target_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
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
    mut parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    parts.selectors.sort_unstable_by_key(SelectorDefinition::id);
    parts.selectors.dedup_by_key(|selector| selector.id());
    parts.programs.sort_unstable_by_key(ProgramDefinition::id);
    parts.triggers.sort_unstable_by_key(|trigger| trigger.id);
    let programs = parts.programs.iter().map(ProgramDefinition::id).collect();
    let selectors = parts.selectors.iter().map(SelectorDefinition::id).collect();
    Ok(ExecutableBattleRule {
        attachment,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), programs, selectors).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), parts.slots, parts.triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
