use super::*;
use starclock_combat::catalog::{
    action::HitTargetGroup,
    selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
};

const OFFERINGS: &str = "StageAbility_612256";
const BEFORE_SUNRISE: &str = "StageAbility_612257";
pub(super) const RESONANCE: &str = "StageAbility_612220";
const FOURFOLD_ROOT: &str = "StageAbility_612221";
const SUFFERING_SUNSHINE: &str = "StageAbility_612222";
const OUTSIDER: &str = "StageAbility_612223";

const RESONANCE_BEFORE: ProgramId = ProgramId::new(0x7920_0001).expect("reserved program ID");
const BURN: EffectDefinitionId = EffectDefinitionId::new(0x7920_0002).expect("reserved effect ID");
const SHOCK: EffectDefinitionId = EffectDefinitionId::new(0x7920_0003).expect("reserved effect ID");
const BLEED: EffectDefinitionId = EffectDefinitionId::new(0x7920_0004).expect("reserved effect ID");
const WIND_SHEAR: EffectDefinitionId =
    EffectDefinitionId::new(0x7920_0005).expect("reserved effect ID");
const CONFUSION: EffectDefinitionId =
    EffectDefinitionId::new(0x7920_0006).expect("reserved effect ID");
const DEVOID: EffectDefinitionId =
    EffectDefinitionId::new(0x7920_0007).expect("reserved effect ID");
const DEVOID_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7920_0008).expect("reserved group ID");
const DEVOID_RECOVERY: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7920_0009).expect("reserved modifier ID");
const DEVOID_STACK_SLOT: StateSlotDefinitionId =
    StateSlotDefinitionId::new(0x7920_000a).expect("reserved modifier slot ID");

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [OFFERINGS, BEFORE_SUNRISE] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            OFFERINGS => offerings(binding, parameters)?,
            BEFORE_SUNRISE => before_sunrise(binding, parameters)?,
            _ => unreachable!("closed Nihility S04 Blessing set"),
        });
    }
    if let Some(binding) = resonance_binding(bindings, SUFFERING_SUNSHINE) {
        output.push(confusion_rule(
            binding,
            resonance_parameters(catalog, binding)?,
        )?);
    }
    if let Some(binding) = resonance_binding(bindings, OUTSIDER) {
        output.push(resonance_energy(
            binding,
            resonance_parameters(catalog, binding)?,
        )?);
    }
    Ok(output)
}

pub(super) fn resonance(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let parameters = resonance_parameters(catalog, binding)?;
    let fourfold = resonance_binding(bindings, FOURFOLD_ROOT).is_some();
    let suffering = resonance_binding(bindings, SUFFERING_SUNSHINE).is_some();
    let duration = whole(parameter(parameters, 3)?)?
        .checked_add(u16::from(fourfold))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let base_chance = parameter(parameters, 0)?
        .checked_add(if fourfold { 1_000_000 } else { 0 })
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let stack_bonus = u16::from(fourfold);
    let attack_ratio = parameter(parameters, 1)?;
    let bleed_ratio = parameter(parameters, 2)?;

    let effects = vec![
        dot_effect(BURN, CombatElement::Fire, duration, attack(attack_ratio), 1)?,
        dot_effect(
            SHOCK,
            CombatElement::Lightning,
            duration,
            attack(attack_ratio),
            1,
        )?,
        dot_effect(
            BLEED,
            CombatElement::Physical,
            duration,
            ValueExpr::Minimum(
                Box::new(multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::CurrentTarget,
                        stat: StatKind::Hp,
                        purpose: FormulaPurpose::Stat,
                    },
                    scalar(bleed_ratio),
                )),
                Box::new(multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::Applier,
                        stat: StatKind::BreakBaseDamage,
                        purpose: FormulaPurpose::Break,
                    },
                    scalar(2_000_000),
                )),
            ),
            1,
        )?,
        dot_effect(
            WIND_SHEAR,
            CombatElement::Wind,
            duration,
            attack(attack_ratio),
            5,
        )?,
    ];
    let mut effect_ids = vec![BURN, SHOCK, BLEED, WIND_SHEAR];
    let mut steps = vec![
        apply(BURN, 1, base_chance),
        apply(SHOCK, 1, base_chance),
        apply(BLEED, 1, base_chance),
        apply(
            WIND_SHEAR,
            whole(parameter(parameters, 4)?)?
                .checked_add(stack_bonus)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            base_chance,
        ),
    ];
    let mut all_effects = effects;
    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    if suffering {
        let status_duration = whole(parameter(parameters, 4)?)?
            .checked_add(u16::from(fourfold))
            .ok_or(BattleRuleLoweringError::InvalidParameter)?;
        let status_stacks = 2_u16
            .checked_add(stack_bonus)
            .ok_or(BattleRuleLoweringError::InvalidParameter)?;
        let status_chance = 1_000_000_i64
            .checked_add(if fourfold { 1_000_000 } else { 0 })
            .ok_or(BattleRuleLoweringError::InvalidParameter)?;
        steps.extend([
            apply(CONFUSION, status_stacks, status_chance),
            apply(DEVOID, status_stacks, status_chance),
        ]);
        effect_ids.extend([CONFUSION, DEVOID]);
        groups.push(ModifierStackingGroup {
            id: DEVOID_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(devoid_modifier(parameter(parameters, 6)?));
        all_effects.extend([
            status_effect(CONFUSION, status_duration, Vec::new())?,
            status_effect(DEVOID, status_duration, vec![DEVOID_RECOVERY])?,
        ]);
    }
    let before = ProgramDefinition::new(
        RESONANCE_BEFORE,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID],
        effect_ids,
        Vec::new(),
    )
    .with_steps(steps);
    let main = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID],
        Vec::new(),
        Vec::new(),
    );
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
    .with_tags(&[AbilityTag::Assist, AbilityTag::PathResonance])
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
        AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, RESONANCE_BEFORE)
            .expect("non-zero sequence"),
    ]);
    let selector = SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    );
    let enemies =
        SelectorDefinition::new(RESONANCE_ENEMY_SELECTOR_ID).with_rule_units(all_enemy_selector()?);
    all_effects.sort_unstable_by_key(EffectDefinition::id);
    Ok(ExecutableResonance {
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: vec![selector, enemies].into_boxed_slice(),
        effects: all_effects.into_boxed_slice(),
        programs: vec![main, before].into_boxed_slice(),
        ability,
        auxiliary_abilities: Box::new([]),
        countdowns: Box::new([]),
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn offerings(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let current = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let root_program =
        ProgramDefinition::new(root, vec![body], vec![allies], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::ForEach {
                selector: allies,
                body,
                maximum: 16,
            }],
        );
    let body_program =
        ProgramDefinition::new(body, Vec::new(), vec![current], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                selector: current,
                amount: multiply(
                    ValueExpr::QueryStat {
                        subject: StatQuerySubject::CurrentTarget,
                        stat: StatKind::Hp,
                        purpose: FormulaPurpose::Healing,
                    },
                    scalar(parameter(parameters, 0)?),
                ),
                apply_formula_modifiers: true,
            })],
        );
    Ok(first_player_rule(
        binding,
        vec![
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(current).with_rule_units(current_subject_selector()?),
        ],
        Vec::new(),
        vec![root_program, body_program],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Dot),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            root,
        )],
    ))
}

fn before_sunrise(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let random = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let definition =
        ProgramDefinition::new(program, Vec::new(), vec![random], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: random,
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Gain,
                    amount: scalar(parameter(parameters, 0)?),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(random).with_rule_units(random_ally_selector()?)],
        Vec::new(),
        vec![definition],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Event,
            EventFilter {
                damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Dot),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    ))
}

fn confusion_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let stacks = ValueExpr::Convert {
        value: Box::new(ValueExpr::QueryEffectStacks {
            subject: StatQuerySubject::Owner,
            effect: CONFUSION,
        }),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    };
    let definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner],
        vec![CONFUSION],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::DetonateDot {
            selector: owner,
            fraction: multiply(stacks, scalar(parameter(parameters, 5)?)),
            required_tag: None,
            selection: starclock_combat::rule::model::RuleDotSelection::All,
        }),
        ProgramStep::Operation(RuleOperationTemplate::AdjustEffectStacks {
            selector: owner,
            effect: CONFUSION,
            delta: ValueExpr::Literal(RuleValue::Integer(-1)),
        }),
    ]);
    Ok(enemy_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![definition],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::WeaknessBroken,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                ..EventFilter::default()
            },
            ConditionExpr::EffectExists {
                selector: owner,
                effect: CONFUSION,
            },
            program,
        )],
    ))
}

fn resonance_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let start = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let dot = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let resource = RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into());
    let amount = |index| {
        parameter(parameters, index)?
            .checked_mul(100)
            .ok_or(BattleRuleLoweringError::InvalidParameter)
    };
    let program = |id, scaled: i64| {
        ProgramDefinition::new(id, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: resource.clone(),
                    update: ResourceUpdateKind::Gain,
                    amount: scalar(scaled),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )],
        )
    };
    Ok(first_player_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![program(start, amount(7)?), program(dot, amount(8)?)],
        vec![
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::BattleStarted,
                OnceScope::Battle,
                EventFilter::default(),
                ConditionExpr::Literal(true),
                start,
            ),
            trigger(
                id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::DamageApplied,
                OnceScope::Event,
                EventFilter {
                    damage_class: Some(starclock_combat::rule::model::RuleDamageClass::Dot),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                dot,
            ),
        ],
    ))
}

fn devoid_modifier(ratio: i64) -> ModifierDefinition {
    ModifierDefinition {
        id: DEVOID_RECOVERY,
        stat: StatKind::ToughnessRecovery,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Negate(Box::new(multiply(
            ValueExpr::Convert {
                value: Box::new(ValueExpr::Slot(DEVOID_STACK_SLOT)),
                target: RuleValueKind::Scalar,
                rounding: Rounding::NearestTiesEven,
            },
            scalar(ratio),
        ))),
        stacking_group: DEVOID_GROUP,
        priority: 0,
        floor: Some(starclock_combat::Scalar::ZERO),
        cap: Some(starclock_combat::Scalar::ONE),
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: Some(DEVOID_STACK_SLOT),
        filters: Box::new([]),
    }
}

fn dot_effect(
    id: EffectDefinitionId,
    element: CombatElement,
    duration: u16,
    magnitude: ValueExpr,
    stack_limit: u16,
) -> Result<EffectDefinition, BattleRuleLoweringError> {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Dot,
        DispelCategory::DispellableDebuff,
        stack_limit,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(duration)))),
        DurationClock::TargetTurnStart,
        EffectTickPhase::TurnStart,
        if stack_limit > 1 {
            EffectStackPolicy::RefreshAndAddStacks
        } else {
            EffectStackPolicy::Refresh
        },
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_comparison(Some(magnitude), 0)
    .with_snapshot(EffectSnapshotPolicy::OnApplication)
    .with_dot(element, None)
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(EffectDefinition::new(id, Vec::new(), Vec::new()).with_runtime_template(runtime))
}

fn status_effect(
    id: EffectDefinitionId,
    duration: u16,
    modifiers: Vec<ModifierDefinitionId>,
) -> Result<EffectDefinition, BattleRuleLoweringError> {
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        5,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(duration)))),
        DurationClock::TargetTurnStart,
        EffectTickPhase::None,
        EffectStackPolicy::RefreshAndAddStacks,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(EffectDefinition::new(id, Vec::new(), modifiers).with_runtime_template(runtime))
}

fn apply(effect: EffectDefinitionId, stacks: u16, chance: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector: RESONANCE_ENEMY_SELECTOR_ID,
        effect,
        stacks: ValueExpr::Literal(RuleValue::Integer(i64::from(stacks))),
        chance: RuleEffectChancePolicy::Resistible,
        base_chance: Some(scalar(chance)),
        rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
    })
}

fn attack(ratio: i64) -> ValueExpr {
    multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Applier,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::Dot,
        },
        scalar(ratio),
    )
}

fn random_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        1,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("behavior-choice".into()),
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn first_player_rule(
    binding: &UniverseBattleRuleBinding,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    executable_rule(
        binding,
        RuleAttachment::FirstPlayer,
        selectors,
        effects,
        programs,
        triggers,
    )
}

fn enemy_rule(
    binding: &UniverseBattleRuleBinding,
    selectors: Vec<SelectorDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    executable_rule(
        binding,
        RuleAttachment::EveryEnemy,
        selectors,
        Vec::new(),
        programs,
        triggers,
    )
}

fn executable_rule(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    mut selectors: Vec<SelectorDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
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
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), Vec::new(), triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
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

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    u16::try_from(value / 1_000_000)
        .ok()
        .filter(|converted| i64::from(*converted) * 1_000_000 == value)
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}
