use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
    RuleSelectorSide, RuleUnitSelector,
};

const TURN_ENERGY: &str = "StageAbility_61245601";
const LAST_ALLY_ATTACK: &str = "StageAbility_61245701";
const STAR_HUNTER: &str = "StageAbility_612421";
const BOW_AND_ARROW: &str = "StageAbility_612422";
const PERFECT_AIM: &str = "StageAbility_612423";

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [TURN_ENERGY, LAST_ALLY_ATTACK] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            TURN_ENERGY => turn_energy(binding, parameters)?,
            LAST_ALLY_ATTACK => last_ally_attack(binding, parameters)?,
            _ => unreachable!("closed Hunt S04 Blessing set"),
        });
    }
    for key in [STAR_HUNTER, BOW_AND_ARROW, PERFECT_AIM] {
        let Some(binding) = resonance_binding(bindings, key) else {
            continue;
        };
        let parameters = resonance_parameters(catalog, binding)?;
        output.push(match key {
            STAR_HUNTER => star_hunter(binding, parameters)?,
            BOW_AND_ARROW => bow_and_arrow(
                binding,
                parameters,
                resonance_binding(bindings, PERFECT_AIM).is_some(),
            )?,
            PERFECT_AIM => perfect_aim(binding, parameters)?,
            _ => unreachable!("closed Hunt S04 formation set"),
        });
    }
    Ok(output)
}

fn turn_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ModifyResource {
                            selector: owner,
                            resource: RuleResourceKind::Energy,
                            update: ResourceUpdateKind::Gain,
                            amount: scalar(parameter(parameters, 0)?),
                            scales_with_regeneration: false,
                            rounding: Rounding::Floor,
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::TurnStarted,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn last_ally_attack(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let actor = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let others = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let mark_last = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let clear_last = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let apply = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let last = id::<StateSlotDefinitionId>(AMOUNT_SLOT_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let set = |program, value| {
        ProgramDefinition::new(program, Vec::new(), Vec::new(), Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::SetSlot {
                slot: last,
                value: ValueExpr::Literal(RuleValue::Integer(value)),
            })],
        )
    };
    let apply_program =
        ProgramDefinition::new(apply, Vec::new(), vec![actor], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: actor,
                    effect,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                    selector: actor,
                    effect,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                }),
            ]);
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers: vec![ModifierDefinition {
                id: modifier,
                stat: StatKind::Atk,
                stage: FormulaStage::Flat,
                purpose: FormulaPurpose::Stat,
                value: multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::Owner,
                        stat: StatKind::Atk,
                        purpose: FormulaPurpose::Stat,
                    },
                    scalar(parameter(parameters, 0)?),
                ),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Flat,
                snapshot: SnapshotPolicy::OnApplication,
                source_stack_slot: None,
                filters: Box::new([]),
            }],
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(actor).with_rule_units(actor_ally_selector()?),
                SelectorDefinition::new(others).with_rule_units(other_allies_selector(owner)?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier])
                    .with_runtime_template(runtime),
            ],
            programs: vec![set(mark_last, 1), set(clear_last, 0), apply_program],
            slots: vec![
                StateSlotDef::new(
                    last,
                    RuleValueKind::Integer,
                    BattleRuleScope::Battle,
                    RuleValue::Integer(0),
                )
                .with_bounds(RuleValue::Integer(0), RuleValue::Integer(1)),
            ],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnEnded,
                    OnceScope::Turn,
                    EventFilter {
                        actor_selector: Some(owner),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    mark_last,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnEnded,
                    OnceScope::Turn,
                    EventFilter {
                        actor_selector: Some(others),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    clear_last,
                ),
                trigger(
                    id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::TurnStarted,
                    OnceScope::Turn,
                    EventFilter {
                        actor_selector: Some(actor),
                        ..EventFilter::default()
                    },
                    integer_slot_equals(last, 1),
                    apply,
                ),
            ],
        },
    )
}

fn star_hunter(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let highest = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let actor = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let activate = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let reward = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let expire = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let activate_program = ProgramDefinition::new(
        activate,
        Vec::new(),
        vec![highest],
        vec![effect],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: highest,
            effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::GrantExtraTurn {
            actor_selector: highest,
        }),
    ]);
    let reward_program =
        ProgramDefinition::new(reward, Vec::new(), vec![actor], vec![effect], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::GrantExtraTurn {
                    actor_selector: actor,
                }),
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: actor,
                    effect,
                }),
            ]);
    let expire_program =
        ProgramDefinition::new(expire, Vec::new(), vec![actor], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: actor,
                    effect,
                },
            )]);
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers: vec![ModifierDefinition {
                id: modifier,
                stat: StatKind::CritDamage,
                stage: FormulaStage::Flat,
                purpose: FormulaPurpose::Stat,
                value: multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::CurrentTarget,
                        stat: StatKind::CritRate,
                        purpose: FormulaPurpose::Stat,
                    },
                    scalar(parameter(parameters, 0)?),
                ),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Flat,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            }],
            selectors: vec![
                SelectorDefinition::new(highest).with_rule_units(highest_attack_selector()?),
                SelectorDefinition::new(actor).with_rule_units(actor_ally_selector()?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier])
                    .with_runtime_template(runtime),
            ],
            programs: vec![activate_program, reward_program, expire_program],
            triggers: vec![
                trigger(
                    id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionResolved,
                    OnceScope::Action,
                    EventFilter {
                        source: Some(resonance_source()),
                        ..EventFilter::default()
                    },
                    ConditionExpr::Literal(true),
                    activate,
                ),
                trigger(
                    id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::UnitDefeated,
                    OnceScope::Event,
                    EventFilter {
                        actor_selector: Some(actor),
                        ..EventFilter::default()
                    },
                    ConditionExpr::EffectExists {
                        selector: actor,
                        effect,
                    },
                    reward,
                ),
                trigger(
                    id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                    RuleEventPoint::ActionResolved,
                    OnceScope::Action,
                    EventFilter {
                        actor_selector: Some(actor),
                        excluded_source: Some(resonance_source()),
                        ..EventFilter::default()
                    },
                    ConditionExpr::EffectExists {
                        selector: actor,
                        effect,
                    },
                    expire,
                ),
            ],
            ..RuleParts::default()
        },
    )
}

fn bow_and_arrow(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    perfect_aim: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let maximum = if perfect_aim { 200 } else { 100 };
    resonance_energy_rule(
        binding,
        RuleEventPoint::UnitDefeated,
        OnceScope::Event,
        EventFilter {
            source: Some(resonance_source()),
            ..EventFilter::default()
        },
        parameter(parameters, 5)?
            .checked_mul(maximum)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
    )
}

fn perfect_aim(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let mut rule = resonance_energy_rule(
        binding,
        RuleEventPoint::TurnStarted,
        OnceScope::Turn,
        EventFilter {
            actor_selector: Some(allies),
            ..EventFilter::default()
        },
        parameter(parameters, 6)?
            .checked_mul(200)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
    )?;
    let mut selectors = rule.selectors.to_vec();
    selectors.push(SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?));
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    rule.selectors = selectors.into_boxed_slice();
    let selector_ids = rule
        .selectors
        .iter()
        .map(SelectorDefinition::id)
        .collect::<Vec<_>>();
    let program_ids = rule.definition.programs().to_vec();
    let runtime = rule
        .definition
        .runtime()
        .expect("energy rule has runtime")
        .clone();
    rule.definition =
        RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(runtime);
    Ok(rule)
}

fn resonance_energy_rule(
    binding: &UniverseBattleRuleBinding,
    point: RuleEventPoint,
    once_scope: OnceScope,
    filter: EventFilter,
    amount: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            programs: vec![
                ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                    .with_steps(vec![ProgramStep::Operation(
                        RuleOperationTemplate::ModifyResource {
                            selector: owner,
                            resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
                            update: ResourceUpdateKind::Gain,
                            amount: scalar(amount),
                            scales_with_regeneration: false,
                            rounding: Rounding::Floor,
                        },
                    )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                point,
                once_scope,
                filter,
                ConditionExpr::Literal(true),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn highest_attack_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StatDescending,
        1,
        1,
        RuleEmptyPoolPolicy::Fault,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
    .map(|selector| {
        selector.with_weight(Some(ValueExpr::QueryStat {
            subject: StatQuerySubject::CurrentTarget,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::Stat,
        }))
    })
}

fn actor_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        1,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn other_allies_selector(owner: SelectorId) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        1,
        16,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::All,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
    .map(|selector| selector.with_predicates(vec![RuleSelectorPredicate::Excludes(owner)]))
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
        .map(|definition| definition.parameters())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)
}

fn resonance_source() -> SourceDefinitionId {
    SourceDefinitionId::new(RESONANCE_ABILITY_ID.get()).expect("ability ID is non-zero")
}

fn integer_slot_equals(slot: StateSlotDefinitionId, value: i64) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::Slot(slot)),
        operator: Comparison::Equal,
        rhs: Box::new(ValueExpr::Literal(RuleValue::Integer(value))),
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
    mut parts: RuleParts,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    parts.groups.sort_unstable_by_key(|group| group.id);
    parts.modifiers.sort_unstable_by_key(|modifier| modifier.id);
    parts.selectors.sort_unstable_by_key(SelectorDefinition::id);
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
    Ok(ExecutableBattleRule {
        attachment,
        modifier_groups: parts.groups.into_boxed_slice(),
        modifiers: parts.modifiers.into_boxed_slice(),
        selectors: parts.selectors.into_boxed_slice(),
        effects: parts.effects.into_boxed_slice(),
        programs: parts.programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), parts.slots, parts.triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
