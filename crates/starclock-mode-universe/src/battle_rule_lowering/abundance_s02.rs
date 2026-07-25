use super::*;
use starclock_combat::{
    EffectRemovalOrder, Scalar,
    catalog::selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
    rule::model::SourceClass,
};

use super::abundance_s01::DEWDROP_RUPTURE_SIGNAL;

const DEWDROP_DISPEL: &str = "StageAbility_612342";
const HEALING_ATTACK: &str = "StageAbility_612343";
const HP_ADDITIONAL_DAMAGE: &str = "StageAbility_612344";
const FULL_HP_DEFENSE: &str = "StageAbility_612345";
const ALLY_HEALING_BONUS: &str = "StageAbility_612346";
const BLESSING_MAXIMUM_HP: &str = "StageAbility_612350";

const CONTRIBUTION_RULE_ID_BASE: u32 = 0x7000_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7950_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7960_0000;
const LOCAL_GROUP_BASE: u32 = 0x7970_0000;

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        DEWDROP_DISPEL,
        HEALING_ATTACK,
        HP_ADDITIONAL_DAMAGE,
        FULL_HP_DEFENSE,
        ALLY_HEALING_BONUS,
        BLESSING_MAXIMUM_HP,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            DEWDROP_DISPEL => dewdrop_dispel(binding, parameters)?,
            HEALING_ATTACK => healing_attack(binding, parameters)?,
            HP_ADDITIONAL_DAMAGE => hp_additional_damage(binding, parameters)?,
            FULL_HP_DEFENSE => full_hp_defense(binding, parameters)?,
            ALLY_HEALING_BONUS => ally_healing_bonus(binding, parameters)?,
            BLESSING_MAXIMUM_HP => blessing_maximum_hp(catalog, blessings, binding, parameters)?,
            _ => unreachable!("closed Abundance S02 binding set"),
        });
    }
    Ok(output)
}

fn dewdrop_dispel(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let cleanse = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let marker = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let programs = vec![
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![marker], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect: marker,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Fixed,
                    base_chance: Some(scalar(parameter(parameters, 0)?)),
                    rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
                },
            )]),
        ProgramDefinition::new(cleanse, Vec::new(), vec![owner], vec![marker], Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::Cleanse {
                    selector: owner,
                    maximum: 1,
                    order: EffectRemovalOrder::OldestFirst,
                }),
                ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect: marker,
                }),
            ]),
    ];
    let triggers = vec![
        trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::InformationalRule,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            signal_condition(DEWDROP_RUPTURE_SIGNAL),
            apply,
        ),
        trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectApplied,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                source: Some(binding.source().definition()),
                effect_definition: Some(marker),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            cleanse,
        ),
    ];
    finish(
        binding,
        RuleParts {
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(marker, Vec::new(), Vec::new())
                    .with_runtime_template(permanent_effect_runtime()?),
            ],
            programs,
            triggers,
            ..RuleParts::default()
        },
    )
}

fn healing_attack(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let healed = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let enhanced = parameter(parameters, 2)? == 1_000_000;
    let target = if enhanced { allies } else { healed };
    let mut steps = vec![ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: target,
        effect,
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })];
    if !enhanced {
        steps.push(ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }));
    }
    let duration = whole(parameter(parameters, 1)?)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(duration)))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    finish(
        binding,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers: vec![ModifierDefinition {
                id: modifier,
                stat: StatKind::Atk,
                stage: FormulaStage::PercentOfBase,
                purpose: FormulaPurpose::Stat,
                value: scalar(parameter(parameters, 0)?),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::PercentOfBase,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            }],
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(healed).with_rule_units(event_healed_ally_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), vec![modifier])
                    .with_runtime_template(runtime),
            ],
            programs: vec![
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![owner, healed, allies],
                    vec![effect],
                    Vec::new(),
                )
                .with_steps(steps),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::HealApplied,
                OnceScope::Event,
                EventFilter {
                    applier_selector: Some(owner),
                    target_selector: Some(healed),
                    source_class: Some(SourceClass::Ability),
                    ..EventFilter::default()
                },
                positive_healing(),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn hp_additional_damage(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let basis = match parameter(parameters, 1)? {
        0 => ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        },
        1_000_000 => ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        _ => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    finish(
        binding,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(target)
                    .with_rule_units(random_event_attack_target_selector()?),
            ],
            programs: vec![
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![owner, target],
                    Vec::new(),
                    Vec::new(),
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::DamageFromEventElement {
                        selector: target,
                        amount: multiply(basis, scalar(parameter(parameters, 0)?)),
                        class: DamageClass::Additional,
                        can_crit: false,
                        can_defeat: true,
                    },
                )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::ActionResolved,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    ability_tag: Some(AbilityTag::Attack),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn full_hp_defense(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let remove = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let purposes = [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ];
    let mut modifiers = purposes
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| {
            Ok(ModifierDefinition {
                id: local::<ModifierDefinitionId>(
                    LOCAL_MODIFIER_BASE,
                    raw,
                    u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                )?,
                stat: StatKind::Hp,
                stage: FormulaStage::Mitigation,
                purpose,
                value: scalar(parameter(parameters, 0)?),
                stacking_group: group,
                priority: 0,
                floor: Some(Scalar::ZERO),
                cap: Some(Scalar::ONE),
                cap_stage: FormulaStage::Mitigation,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: Box::new([]),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    if parameter(parameters, 1)? > 0 {
        modifiers.push(ModifierDefinition {
            id: local::<ModifierDefinitionId>(
                LOCAL_MODIFIER_BASE,
                raw,
                u32::try_from(modifiers.len())
                    .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )?,
            stat: StatKind::EffectResistance,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::EffectChance,
            value: scalar(parameter(parameters, 1)?),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
    }
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let apply_definition =
        ProgramDefinition::new(apply, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    let remove_definition =
        ProgramDefinition::new(remove, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: owner,
                    effect,
                },
            )]);
    let mut triggers = vec![
        trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::BattleStarted,
            OnceScope::Battle,
            EventFilter::default(),
            full_hp_now(),
            apply,
        ),
        trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::HealApplied,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            full_hp_after_event(),
            apply,
        ),
    ];
    for (index, point) in [RuleEventPoint::DamageApplied, RuleEventPoint::HpChanged]
        .into_iter()
        .enumerate()
    {
        triggers.push(trigger(
            local::<TriggerId>(
                LOCAL_TRIGGER_BASE,
                raw,
                u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )?,
            point,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            below_full_hp_after_event(),
            remove,
        ));
    }
    finish(
        binding,
        RuleParts {
            groups: vec![ModifierStackingGroup {
                id: group,
                aggregation: ModifierAggregation::UniquePerSource,
                comparator: None,
            }],
            modifiers,
            selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
            effects: vec![
                EffectDefinition::new(effect, Vec::new(), modifier_ids)
                    .with_runtime_template(permanent_effect_runtime()?),
            ],
            programs: vec![apply_definition, remove_definition],
            triggers,
            ..RuleParts::default()
        },
    )
}

fn ally_healing_bonus(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    finish(
        binding,
        RuleParts {
            selectors: vec![
                SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
                SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            ],
            programs: vec![
                ProgramDefinition::new(
                    program,
                    Vec::new(),
                    vec![owner, allies],
                    Vec::new(),
                    Vec::new(),
                )
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::Heal {
                        selector: owner,
                        amount: multiply(
                            ValueExpr::ReadEventProperty(EventValueProperty::HpChangeAmount),
                            scalar(parameter(parameters, 0)?),
                        ),
                        apply_formula_modifiers: false,
                    },
                )]),
            ],
            triggers: vec![trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::HealApplied,
                OnceScope::Event,
                EventFilter {
                    applier_selector: Some(allies),
                    target_selector: Some(owner),
                    excluded_source: Some(binding.source().definition()),
                    source_class: Some(SourceClass::Ability),
                    ..EventFilter::default()
                },
                positive_healing(),
                program,
            )],
            ..RuleParts::default()
        },
    )
}

fn blessing_maximum_hp(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let cap = whole(parameter(parameters, 1)?)?;
    let count = i64::from(abundance_blessing_count(catalog, blessings)?.min(cap));
    let value = parameter(parameters, 0)?
        .checked_mul(count)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    persistent_modifier_rule(
        binding,
        StatKind::Hp,
        FormulaStage::PercentOfBase,
        FormulaPurpose::Stat,
        scalar(value),
        Vec::new(),
    )
}

fn abundance_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.abundance")
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

fn event_healed_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
        RuleSelectorOrdering::Formation,
        None,
    )
}

fn random_event_attack_target_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::RngUniform,
        1,
        RuleSelectorOrdering::EventOrder,
        Some("bounce-target".into()),
    )
}

fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    choice: RuleSelectorChoice,
    maximum: u16,
    ordering: RuleSelectorOrdering,
    rng_purpose: Option<Box<str>>,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        ordering,
        1,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        rng_purpose,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn positive_healing() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::HpChangeAmount,
        )),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    }
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

fn full_hp_after_event() -> ConditionExpr {
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

fn below_full_hp_after_event() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::HpAfter)),
        operator: Comparison::Less,
        rhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
    }
}

fn permanent_effect_runtime() -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
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

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(
        value
            .checked_div(1_000_000)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?,
    )
    .map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

fn local<T>(base: u32, raw: u32, index: u32) -> Result<T, BattleRuleLoweringError>
where
    T: TryFrom<u32>,
{
    let offset = raw
        .checked_sub(CONTRIBUTION_RULE_ID_BASE)
        .and_then(|value| value.checked_mul(16))
        .and_then(|value| value.checked_add(index))
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    base.checked_add(offset)
        .and_then(|value| T::try_from(value).ok())
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
