use super::*;
use starclock_combat::catalog::{
    action::HitTargetGroup,
    selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
};

const PLATINUM_AGE: &str = "StageAbility_612656";
const CLOCKWORK_APPLE: &str = "StageAbility_612657";
const ULTIMATE_AS_FOLLOW_UP: &str = "StageAbility_612632";
pub(super) const RESONANCE: &str = "StageAbility_612620";
const DOOMSDAY_CARNIVAL: &str = "StageAbility_612621";
const DANCE_OF_GROWTH: &str = "StageAbility_612622";
const INSTANT_WIN: &str = "StageAbility_612623";

const RESONANCE_LOW: ProgramId =
    ProgramId::new(0x7a00_0001).expect("reserved Elation S04 program ID");
const RESONANCE_HIGH: ProgramId =
    ProgramId::new(0x7a00_0002).expect("reserved Elation S04 program ID");
const RESONANCE_ACTOR: SelectorId =
    SelectorId::new(0x7a00_0003).expect("reserved Elation S04 selector ID");
const HIGHEST_ATTACK: SelectorId =
    SelectorId::new(0x7a00_0004).expect("reserved Elation S04 selector ID");
const SENSORY_MODIFIER_SECOND_BASE: u32 = 0x7a01_0000;
const SENSORY_MODIFIER_THIRD_BASE: u32 = 0x7a02_0000;
const SENSORY_MODIFIER_FOURTH_BASE: u32 = 0x7a03_0000;
const SENSORY_STACK_SLOT_BASE: u32 = 0x7a04_0000;

const ELEMENTS: [CombatElement; 7] = [
    CombatElement::Physical,
    CombatElement::Fire,
    CombatElement::Ice,
    CombatElement::Lightning,
    CombatElement::Wind,
    CombatElement::Quantum,
    CombatElement::Imaginary,
];

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let ultimate_as_follow_up =
        selected_level_parameters(blessings, ULTIMATE_AS_FOLLOW_UP).is_some();
    let mut output = Vec::new();
    for key in [PLATINUM_AGE, CLOCKWORK_APPLE] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            PLATINUM_AGE => {
                timed_follow_up_stat(binding, parameters, StatKind::Def, ultimate_as_follow_up)?
            }
            CLOCKWORK_APPLE => {
                timed_follow_up_stat(binding, parameters, StatKind::Spd, ultimate_as_follow_up)?
            }
            _ => unreachable!("closed Elation S04 Blessing set"),
        });
    }
    if let Some(binding) = resonance_binding(bindings, DOOMSDAY_CARNIVAL) {
        output.push(sensory_pursuit(
            binding,
            resonance_parameters(catalog, binding)?,
        )?);
    }
    if let Some(binding) = resonance_binding(bindings, INSTANT_WIN) {
        output.push(follow_up_resonance_energy(
            binding,
            resonance_parameters(catalog, binding)?,
            resonance_binding(bindings, DANCE_OF_GROWTH).is_some(),
            ultimate_as_follow_up,
        )?);
    }
    Ok(output)
}

pub(super) fn resonance(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    _blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
    damage_ratio: i64,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let parameters = resonance_parameters(catalog, binding)?;
    let dance = resonance_binding(bindings, DANCE_OF_GROWTH).is_some();
    let instant = resonance_binding(bindings, INSTANT_WIN).is_some();
    let maximum_energy = if dance { 200 } else { 100 };
    let initial_bonus = if instant {
        scaled_resource(maximum_energy, parameter(parameters, 5)?)?
    } else {
        0
    };
    let initial_energy = initial_energy
        .checked_add(initial_bonus)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?
        .min(maximum_energy);
    let source_ratio = Ratio::from_scaled(parameter(parameters, 0)?)
        .checked_mul(
            Ratio::ONE
                .checked_add(Ratio::from_scaled(damage_ratio))
                .map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
            Rounding::NearestTiesEven,
        )
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let amount = multiply(
        ValueExpr::SelectorSum {
            selector: HIGHEST_ATTACK,
            value: Box::new(ValueExpr::QueryStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Atk,
                purpose: FormulaPurpose::ElationDamage,
            }),
        },
        scalar(source_ratio.scaled()),
    );
    let minimum_hits = whole(parameter(parameters, 1)?)?;
    let maximum_hits = whole(parameter(parameters, 2)?)?;
    let extra_hits = if dance {
        let threshold = parameter(parameters, 4)?;
        if threshold <= 0 {
            return Err(BattleRuleLoweringError::InvalidParameter);
        }
        u16::try_from(1_000_000_i64 / threshold)
            .map_err(|_| BattleRuleLoweringError::InvalidParameter)?
    } else {
        0
    };
    let low = ProgramDefinition::new(
        RESONANCE_LOW,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID, HIGHEST_ATTACK],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(random_damage(
        RESONANCE_ENEMY_SELECTOR_ID,
        amount.clone(),
        minimum_hits,
        maximum_hits,
    ))]);
    let high = ProgramDefinition::new(
        RESONANCE_HIGH,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID, RESONANCE_ACTOR, HIGHEST_ATTACK],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            selector: RESONANCE_ACTOR,
            resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
            update: ResourceUpdateKind::Spend,
            amount: scalar(100_000_000),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        }),
        ProgramStep::Operation(random_damage(
            RESONANCE_ENEMY_SELECTOR_ID,
            amount,
            minimum_hits
                .checked_add(extra_hits)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            maximum_hits
                .checked_add(extra_hits)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        )),
    ]);
    let main_steps = if dance {
        vec![ProgramStep::If {
            condition: ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::ReadResource {
                    selector: RESONANCE_ACTOR,
                    resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
                }),
                operator: Comparison::GreaterOrEqual,
                rhs: Box::new(scalar(100_000_000)),
            },
            then_program: RESONANCE_HIGH,
            else_program: Some(RESONANCE_LOW),
        }]
    } else {
        vec![ProgramStep::If {
            condition: ConditionExpr::Literal(true),
            then_program: RESONANCE_LOW,
            else_program: None,
        }]
    };
    let main = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        vec![RESONANCE_LOW, RESONANCE_HIGH],
        vec![RESONANCE_ENEMY_SELECTOR_ID, RESONANCE_ACTOR, HIGHEST_ATTACK],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(main_steps);
    let action = AbilityActionDefinition::new(
        AbilityKind::Ultimate,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        )
        .with_team_resource_costs(vec![
            TeamResourceCost::new(RESONANCE_RESOURCE_KEY, 100)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        ])
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_tags(&[
        AbilityTag::Attack,
        AbilityTag::Ultimate,
        AbilityTag::Assist,
        AbilityTag::FollowUp,
    ])
    .with_hits(vec![ActionHitDefinition::new(Vec::new()).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    )])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let ability = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(action)
    .with_programs(vec![
        AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, RESONANCE_PROGRAM_ID)
            .expect("non-zero sequence"),
    ]);
    Ok(ExecutableResonance {
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: vec![
            SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
                UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
                    .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ),
            SelectorDefinition::new(RESONANCE_ENEMY_SELECTOR_ID)
                .with_rule_units(all_enemy_selector()?),
            SelectorDefinition::new(RESONANCE_ACTOR).with_rule_units(actor_selector()?),
            SelectorDefinition::new(HIGHEST_ATTACK).with_rule_units(highest_attack_selector()?),
        ]
        .into_boxed_slice(),
        effects: Box::new([]),
        programs: vec![main, low, high].into_boxed_slice(),
        ability,
        auxiliary_abilities: Box::new([]),
        countdowns: Box::new([]),
        initial_energy,
        maximum_energy,
    })
}

fn timed_follow_up_stat(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    stat: StatKind,
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let modifier = id::<ModifierDefinitionId>(MODIFIER_ID_BASE, raw)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::DispellableBuff,
        1,
        Some(integer(i64::from(whole(parameter(parameters, 1)?)?))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: owner,
                    effect,
                    stacks: integer(1),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        vec![ModifierDefinition {
            id: modifier,
            stat,
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
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), vec![modifier])
                .with_runtime_template(runtime),
        ],
        vec![definition],
        follow_up_triggers(raw, owner, program, ultimate_as_follow_up)?,
    )
}

fn sensory_pursuit(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let target = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let group = id::<ModifierStackingGroupId>(MODIFIER_GROUP_ID_BASE, raw)?;
    let stack_slot = id::<StateSlotDefinitionId>(SENSORY_STACK_SLOT_BASE, raw)?;
    let purposes = [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
    ];
    let modifiers = purposes
        .into_iter()
        .enumerate()
        .map(|(index, purpose)| {
            let base = [
                MODIFIER_ID_BASE,
                SENSORY_MODIFIER_SECOND_BASE,
                SENSORY_MODIFIER_THIRD_BASE,
                SENSORY_MODIFIER_FOURTH_BASE,
            ][index];
            let mut filters = vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)];
            if purpose != FormulaPurpose::ElationDamage {
                filters.push(ModifierFilter::AbilityTag("follow_up".into()));
            }
            Ok(ModifierDefinition {
                id: id(base, raw)?,
                stat: StatKind::Hp,
                stage: FormulaStage::Vulnerability,
                purpose,
                value: multiply(
                    ValueExpr::Convert {
                        value: Box::new(ValueExpr::Slot(stack_slot)),
                        target: RuleValueKind::Scalar,
                        rounding: Rounding::NearestTiesEven,
                    },
                    scalar(parameter(parameters, 3)?),
                ),
                stacking_group: group,
                priority: 0,
                floor: None,
                cap: None,
                cap_stage: FormulaStage::Vulnerability,
                snapshot: SnapshotPolicy::RecomputeOnStackChange,
                source_stack_slot: Some(stack_slot),
                filters: filters.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, BattleRuleLoweringError>>()?;
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        64,
        Some(integer(1)),
        DurationClock::TargetActionEnd,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let apply = ProgramDefinition::new(program, Vec::new(), vec![target], vec![effect], Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::ApplyEffect {
                selector: target,
                effect,
                stacks: integer(1),
                chance: RuleEffectChancePolicy::Resistible,
                base_chance: Some(scalar(1_500_000)),
                rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
            },
        )]);
    finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        modifiers,
        vec![SelectorDefinition::new(target).with_rule_units(primary_target_selector()?)],
        vec![
            EffectDefinition::new(effect, Vec::new(), modifier_ids).with_runtime_template(runtime),
        ],
        vec![apply],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Elation),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn follow_up_resonance_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    dance: bool,
    ultimate_as_follow_up: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let maximum = if dance { 200 } else { 100 };
    let amount = scaled_resource(maximum, parameter(parameters, 6)?)?;
    let definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
                    update: ResourceUpdateKind::Gain,
                    amount: scalar(i64::from(amount) * 1_000_000),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
    finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![definition],
        follow_up_triggers(raw, owner, program, ultimate_as_follow_up)?,
    )
}

fn follow_up_triggers(
    raw: u32,
    owner: SelectorId,
    program: ProgramId,
    ultimate_as_follow_up: bool,
) -> Result<Vec<TriggerDef>, BattleRuleLoweringError> {
    let mut tags = vec![
        (TRIGGER_ID_BASE, AbilityTag::FollowUp),
        (SECOND_TRIGGER_ID_BASE, AbilityTag::Counter),
    ];
    if ultimate_as_follow_up {
        tags.push((THIRD_TRIGGER_ID_BASE, AbilityTag::Ultimate));
    }
    tags.into_iter()
        .map(|(base, tag)| {
            Ok(trigger(
                id::<TriggerId>(base, raw)?,
                RuleEventPoint::ActionResolved,
                OnceScope::Action,
                EventFilter {
                    actor_selector: Some(owner),
                    ability_tag: Some(tag),
                    excluded_source: Some(resonance_source()),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                program,
            ))
        })
        .collect()
}

fn random_damage(
    selector: SelectorId,
    amount: ValueExpr,
    minimum_hits: u16,
    maximum_hits: u16,
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
        exclude_event_element: false,
        can_crit: false,
        can_defeat: true,
    }
}

fn highest_attack_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    Ok(RuleUnitSelector::new(
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
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_weight(Some(ValueExpr::QueryStat {
        subject: StatQuerySubject::CurrentTarget,
        stat: StatKind::Atk,
        purpose: FormulaPurpose::ElationDamage,
    })))
}

fn actor_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        1,
        1,
        RuleEmptyPoolPolicy::Fault,
        RuleSelectorChoice::First,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn scaled_resource(maximum: u16, ratio: i64) -> Result<u16, BattleRuleLoweringError> {
    i64::from(maximum)
        .checked_mul(ratio)
        .and_then(|value| value.checked_div(1_000_000))
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    value
        .checked_div(1_000_000)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
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

#[allow(clippy::too_many_arguments)]
fn finish_rule(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    mut groups: Vec<ModifierStackingGroup>,
    mut modifiers: Vec<ModifierDefinition>,
    mut selectors: Vec<SelectorDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    mut triggers: Vec<TriggerDef>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = programs.iter().map(ProgramDefinition::id).collect();
    Ok(ExecutableBattleRule {
        attachment,
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), Vec::new(), triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
