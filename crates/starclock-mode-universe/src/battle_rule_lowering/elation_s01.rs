use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const RANDOM_AFTERTASTE: &str = "StageAbility_612630";
const BROKEN_AFTERTASTE: &str = "StageAbility_612631";
const ULTIMATE_AS_FOLLOW_UP: &str = "StageAbility_612632";
const EXTRA_AFTERTASTE: &str = "StageAbility_612640";
const AFTERTASTE_VULNERABILITY: &str = "StageAbility_612641";

const ELATION_EFFECT_BASE: u32 = 0x79e1_0000;
const ELATION_MODIFIER_BASE: u32 = 0x79e2_0000;
const ELATION_GROUP_BASE: u32 = 0x79e3_0000;
const ELATION_PROGRAM_BASE: u32 = 0x79e4_0000;
const ELATION_TRIGGER_BASE: u32 = 0x79e5_0000;

const ELEMENTS: [CombatElement; 7] = [
    CombatElement::Physical,
    CombatElement::Fire,
    CombatElement::Ice,
    CombatElement::Lightning,
    CombatElement::Wind,
    CombatElement::Quantum,
    CombatElement::Imaginary,
];

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let ultimate_as_follow_up =
        selected_level_parameters(blessings, ULTIMATE_AS_FOLLOW_UP).is_some();
    let mut output = Vec::new();
    for key in [
        RANDOM_AFTERTASTE,
        BROKEN_AFTERTASTE,
        ULTIMATE_AS_FOLLOW_UP,
        EXTRA_AFTERTASTE,
        AFTERTASTE_VULNERABILITY,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            RANDOM_AFTERTASTE => random_aftertaste(binding, parameters, ultimate_as_follow_up)?,
            BROKEN_AFTERTASTE => broken_aftertaste(binding, parameters, ultimate_as_follow_up)?,
            ULTIMATE_AS_FOLLOW_UP => ultimate_as_follow_up_rule(binding, parameters)?,
            EXTRA_AFTERTASTE => extra_aftertaste(binding, parameters)?,
            AFTERTASTE_VULNERABILITY => aftertaste_vulnerability(binding, parameters)?,
            _ => unreachable!("closed Elation S01 binding set"),
        });
    }
    Ok(output)
}

fn random_aftertaste(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let mut modifier_groups = Vec::new();
    let mut modifiers = Vec::new();
    if parameter(parameters, 3)? != 0 {
        let group = ModifierStackingGroupId::new(ELATION_GROUP_BASE + 1)
            .expect("reserved Elation group ID");
        let modifier = ModifierDefinitionId::new(ELATION_MODIFIER_BASE + 1)
            .expect("reserved Elation modifier ID");
        modifier_groups.push(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(ModifierDefinition {
            id: modifier,
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose: FormulaPurpose::ElationDamage,
            value: scalar(parameter(parameters, 3)?),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)]
                .into_boxed_slice(),
        });
    }
    let programs = vec![
        ProgramDefinition::new(
            program,
            vec![body],
            vec![owner, target],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: target,
            body,
            maximum: 16,
        }]),
        ProgramDefinition::new(body, Vec::new(), vec![current], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(random_elation_damage(
                current,
                multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::Owner,
                        stat: StatKind::Atk,
                        purpose: FormulaPurpose::Stat,
                    },
                    scalar(parameter(parameters, 2)?),
                ),
                whole(parameter(parameters, 0)?)?,
                whole(parameter(parameters, 1)?)?,
                false,
            ))],
        ),
    ];
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::EveryPlayer,
        modifier_groups,
        modifiers,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(target).with_rule_units(randomized_event_targets_selector()?),
            SelectorDefinition::new(current).with_rule_units(current_subject_selector()?),
        ],
        Vec::new(),
        programs,
        Vec::new(),
        follow_up_triggers(raw, owner, program, ultimate_as_follow_up)?,
    ))
}

fn broken_aftertaste(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let ordinary = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let broken = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 0)?),
    );
    let extra = whole(parameter(parameters, 1)?)?;
    let root_program = ProgramDefinition::new(
        root,
        vec![body],
        vec![owner, target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::ForEach {
        selector: target,
        body,
        maximum: 16,
    }]);
    let body_program = ProgramDefinition::new(
        body,
        vec![ordinary, broken],
        vec![current],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::If {
        condition: ConditionExpr::CurrentTargetIsBroken,
        then_program: broken,
        else_program: Some(ordinary),
    }]);
    let ordinary_program =
        ProgramDefinition::new(ordinary, Vec::new(), vec![current], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(random_elation_damage(
                current,
                amount.clone(),
                1,
                1,
                false,
            ))]);
    let broken_hits = 1_u16
        .checked_add(extra)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let broken_program =
        ProgramDefinition::new(broken, Vec::new(), vec![current], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(random_elation_damage(
                current,
                amount,
                broken_hits,
                broken_hits,
                false,
            ))]);
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(target).with_rule_units(randomized_event_targets_selector()?),
            SelectorDefinition::new(current).with_rule_units(current_subject_selector()?),
        ],
        Vec::new(),
        vec![root_program, body_program, ordinary_program, broken_program],
        Vec::new(),
        follow_up_triggers(raw, owner, root, ultimate_as_follow_up)?,
    ))
}

fn ultimate_as_follow_up_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let group =
        ModifierStackingGroupId::new(ELATION_GROUP_BASE + 2).expect("reserved Elation group ID");
    let value = scalar(parameter(parameters, 0)?);
    let modifiers = ["follow_up", "counter", "ultimate"]
        .into_iter()
        .enumerate()
        .map(|(index, tag)| ModifierDefinition {
            id: ModifierDefinitionId::new(
                ELATION_MODIFIER_BASE + 2 + u32::try_from(index).expect("three tags"),
            )
            .expect("reserved Elation modifier ID"),
            stat: StatKind::Atk,
            stage: FormulaStage::DamageBoost,
            purpose: FormulaPurpose::OrdinaryDamage,
            value: value.clone(),
            stacking_group: group,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::DamageBoost,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: vec![
                ModifierFilter::FormulaSubject(FormulaSubject::Source),
                ModifierFilter::AbilityTag(tag.into()),
            ]
            .into_boxed_slice(),
        })
        .collect::<Vec<_>>();
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::EveryPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        modifiers,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    ))
}

fn extra_aftertaste(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![target], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(random_elation_damage(
                target,
                multiply(
                    ValueExpr::ReadEventProperty(EventValueProperty::DamageRawAmount),
                    scalar(parameter(parameters, 1)?),
                ),
                whole(parameter(parameters, 0)?)?,
                whole(parameter(parameters, 0)?)?,
                true,
            ))]);
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        Vec::new(),
        vec![program_definition],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                excluded_source: Some(binding.source().definition()),
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Elation),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    ))
}

fn aftertaste_vulnerability(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let group =
        ModifierStackingGroupId::new(ELATION_GROUP_BASE + 3).expect("reserved Elation group ID");
    let purposes = [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ];
    let mut modifiers = Vec::new();
    let mut effects = Vec::new();
    let mut programs = Vec::new();
    let mut triggers = Vec::new();
    for (element_index, element) in ELEMENTS.into_iter().enumerate() {
        let element_index =
            u32::try_from(element_index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
        let effect = EffectDefinitionId::new(ELATION_EFFECT_BASE + element_index + 1)
            .expect("reserved Elation effect ID");
        let program = ProgramId::new(ELATION_PROGRAM_BASE + element_index + 1)
            .expect("reserved Elation program ID");
        let trigger_id = TriggerId::new(ELATION_TRIGGER_BASE + element_index + 1)
            .expect("reserved Elation trigger ID");
        let mut effect_modifiers = Vec::new();
        for (purpose_index, purpose) in purposes.into_iter().enumerate() {
            let modifier = ModifierDefinitionId::new(
                ELATION_MODIFIER_BASE
                    + 0x100
                    + element_index * 16
                    + u32::try_from(purpose_index)
                        .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )
            .expect("reserved Elation modifier ID");
            effect_modifiers.push(modifier);
            modifiers.push(ModifierDefinition {
                id: modifier,
                stat: StatKind::Hp,
                stage: FormulaStage::Vulnerability,
                purpose,
                value: scalar(parameter(parameters, 0)?),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Vulnerability,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)]
                    .into_boxed_slice(),
            });
        }
        effects.push(
            EffectDefinition::new(effect, Vec::new(), effect_modifiers).with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::Debuff,
                    DispelCategory::DispellableDebuff,
                    1,
                    Some(integer(1)),
                    DurationClock::TargetActionEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Refresh,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ),
        );
        programs.push(
            ProgramDefinition::new(program, Vec::new(), vec![target], vec![effect], Vec::new())
                .with_steps(vec![ProgramStep::Operation(
                    RuleOperationTemplate::ApplyEffect {
                        selector: target,
                        effect,
                        stacks: integer(1),
                        chance: RuleEffectChancePolicy::Guaranteed,
                        base_chance: None,
                        rng_purpose: None,
                    },
                )]),
        );
        triggers.push(trigger(
            trigger_id,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                element: Some(element),
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Elation),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        ));
    }
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::FirstPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::Sum,
            comparator: None,
        }],
        modifiers,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        effects,
        programs,
        Vec::new(),
        triggers,
    ))
}

fn follow_up_triggers(
    raw: u32,
    owner: SelectorId,
    program: ProgramId,
    ultimate_as_follow_up: bool,
) -> Result<Vec<TriggerDef>, BattleRuleLoweringError> {
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::ActionResolved,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(AbilityTag::FollowUp),
            excluded_source: Some(resonance_source()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        program,
    )];
    triggers.push(trigger(
        id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::ActionResolved,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(owner),
            ability_tag: Some(AbilityTag::Counter),
            excluded_source: Some(resonance_source()),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        program,
    ));
    if ultimate_as_follow_up {
        triggers.push(trigger(
            id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::ActionResolved,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                ability_tag: Some(AbilityTag::Ultimate),
                excluded_source: Some(resonance_source()),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        ));
    }
    Ok(triggers)
}

fn random_elation_damage(
    selector: SelectorId,
    amount: ValueExpr,
    minimum_hits: u16,
    maximum_hits: u16,
    exclude_event_element: bool,
) -> RuleOperationTemplate {
    RuleOperationTemplate::RandomRepeatedDamage {
        selector,
        amount,
        class: DamageClass::Elation,
        elements: ELEMENTS.into(),
        minimum_hits,
        maximum_hits,
        count_rng_purpose: DrawPurpose::REPEATED_DAMAGE_COUNT,
        element_rng_purpose: DrawPurpose::DAMAGE_ELEMENT,
        exclude_event_element,
        can_crit: false,
        can_defeat: true,
    }
}

fn randomized_event_targets_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::EventOrder,
        1,
        16,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("damage-target".into()),
        false,
    )
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

#[allow(clippy::too_many_arguments)]
fn executable_with_attachment(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    groups: Vec<ModifierStackingGroup>,
    modifiers: Vec<ModifierDefinition>,
    mut selectors: Vec<SelectorDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    slots: Vec<StateSlotDef>,
    mut triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = programs.iter().map(ProgramDefinition::id).collect();
    ExecutableBattleRule {
        attachment,
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), slots, triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}
