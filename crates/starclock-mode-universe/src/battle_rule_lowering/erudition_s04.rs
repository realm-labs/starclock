use super::*;
use starclock_combat::{
    EffectDamageGuard,
    catalog::{
        action::HitTargetGroup,
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorPredicate, RuleSelectorReference,
            RuleSelectorSide, RuleUnitSelector,
        },
    },
};

const ULTIMATE_HEALING: &str = "StageAbility_612856";
const LETHAL_ENERGY_HEALING: &str = "StageAbility_612857";
pub(super) const RESONANCE: &str = "StageAbility_612820";
const MELT_CORE: &str = "StageAbility_612821";
const CHAIN_CONTAGION: &str = "StageAbility_612822";
const MEMETIC_INVERSION: &str = "StageAbility_612823";

const LOCAL_PROGRAM_BASE: u32 = 0x7ed0_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7ee0_0000;
const LOCAL_EFFECT_BASE: u32 = 0x7ef0_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7d80_0000;

const SYNAPSE_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7df0_0001).expect("reserved Erudition effect ID");
const RESONANCE_BODY: ProgramId =
    ProgramId::new(0x7df0_0002).expect("reserved Erudition program ID");
const RESONANCE_CURRENT_ENEMY: SelectorId =
    SelectorId::new(0x7df0_0003).expect("reserved Erudition selector ID");

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [ULTIMATE_HEALING, LETHAL_ENERGY_HEALING] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            ULTIMATE_HEALING => ultimate_healing(binding, parameters)?,
            LETHAL_ENERGY_HEALING => lethal_energy_healing(binding, parameters)?,
            _ => unreachable!("closed Erudition S04 blessing set"),
        });
    }

    let base_parameters = resonance_binding(bindings, RESONANCE)
        .map(|binding| resonance_parameters(catalog, binding))
        .transpose()?;
    for key in [RESONANCE, MELT_CORE, CHAIN_CONTAGION, MEMETIC_INVERSION] {
        let Some(binding) = resonance_binding(bindings, key) else {
            continue;
        };
        output.push(match key {
            RESONANCE => synapse_rule(
                binding,
                base_parameters.ok_or(BattleRuleLoweringError::SnapshotMismatch)?,
            )?,
            MELT_CORE => melt_core(
                binding,
                resonance_parameters(catalog, binding)?,
                resonance_binding(bindings, CHAIN_CONTAGION).is_some(),
            )?,
            CHAIN_CONTAGION => chain_contagion(
                binding,
                resonance_parameters(catalog, binding)?,
                base_parameters.ok_or(BattleRuleLoweringError::SnapshotMismatch)?,
            )?,
            MEMETIC_INVERSION => {
                memetic_inversion(binding, resonance_parameters(catalog, binding)?)?
            }
            _ => unreachable!("closed Erudition S04 formation set"),
        });
    }
    Ok(output)
}

pub(super) fn resonance(
    catalog: &UniverseCatalog,
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
    resonance_damage_ratio: i64,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let parameters = resonance_parameters(catalog, binding)?;
    let link_ratio = parameter_six(parameters, 0)?;
    let initial_ratio = Ratio::from_scaled(parameter_six(parameters, 1)?)
        .checked_mul(
            Ratio::ONE
                .checked_add(Ratio::from_scaled(resonance_damage_ratio))
                .map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
            Rounding::NearestTiesEven,
        )
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let maximum_triggers = whole(parameter_six(parameters, 2)?)?;
    if link_ratio <= 0 || maximum_triggers == 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }

    let root = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        vec![RESONANCE_BODY],
        vec![RESONANCE_ENEMY_SELECTOR_ID, RESONANCE_CURRENT_ENEMY],
        vec![SYNAPSE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: RESONANCE_ENEMY_SELECTOR_ID,
            effect: SYNAPSE_EFFECT,
            stacks: integer(i64::from(maximum_triggers)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::ForEach {
            selector: RESONANCE_ENEMY_SELECTOR_ID,
            body: RESONANCE_BODY,
            maximum: 16,
        },
    ]);
    let body = ProgramDefinition::new(
        RESONANCE_BODY,
        Vec::new(),
        vec![RESONANCE_CURRENT_ENEMY],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: RESONANCE_CURRENT_ENEMY,
            amount: multiply(
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::CurrentTarget,
                    stat: StatKind::Hp,
                    purpose: FormulaPurpose::AdditionalDamage,
                },
                scalar(initial_ratio.scaled()),
            ),
            class: DamageClass::Additional,
            element: CombatElement::Imaginary,
            can_crit: false,
            can_defeat: true,
        },
    )]);
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
        AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, RESONANCE_PROGRAM_ID)
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
                .with_rule_units(all_linkable_enemies_selector()?),
            SelectorDefinition::new(RESONANCE_CURRENT_ENEMY)
                .with_rule_units(current_enemy_selector()?),
        ]
        .into_boxed_slice(),
        effects: vec![
            EffectDefinition::new(SYNAPSE_EFFECT, Vec::new(), Vec::new())
                .with_runtime_template(synapse_runtime(maximum_triggers)?),
        ]
        .into_boxed_slice(),
        programs: vec![root, body].into_boxed_slice(),
        ability,
        auxiliary_abilities: Box::new([]),
        countdowns: Box::new([]),
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn ultimate_healing(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![
            ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
                .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Heal {
                    selector: owner,
                    amount: multiply(
                        maximum_hp(StatQuerySubject::Owner),
                        scalar(parameter_six(parameters, 0)?),
                    ),
                    apply_formula_modifiers: true,
                })]),
        ],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                excluded_source: Some(resonance_source()),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn lethal_energy_healing(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let setup = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let recover = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let effect = local::<EffectDefinitionId>(LOCAL_EFFECT_BASE, raw, 0)?;
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
    let energy_ratio = ValueExpr::Divide {
        lhs: Box::new(ValueExpr::ReadResource {
            selector: owner,
            resource: RuleResourceKind::Energy,
        }),
        rhs: Box::new(ValueExpr::QueryMaximumEnergy(StatQuerySubject::Owner)),
        rounding: Rounding::NearestTiesEven,
    };
    let recover_program =
        ProgramDefinition::new(recover, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![
                ProgramStep::Operation(RuleOperationTemplate::Heal {
                    selector: owner,
                    amount: multiply(
                        maximum_hp(StatQuerySubject::Owner),
                        multiply(energy_ratio, scalar(parameter_six(parameters, 0)?)),
                    ),
                    apply_formula_modifiers: false,
                }),
                ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Set,
                    amount: scalar(0),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                }),
            ]);
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![EffectDefinition::new(effect, Vec::new(), Vec::new()).with_runtime_template(runtime)],
        vec![
            ProgramDefinition::new(setup, Vec::new(), vec![owner], vec![effect], Vec::new())
                .with_steps(vec![apply_effect(owner, effect, 1)]),
            recover_program,
        ],
        vec![
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
                RuleEventPoint::BattleStarted,
                TriggerPhase::AfterEvent,
                OnceScope::Battle,
                EventFilter::default(),
                ConditionExpr::Literal(true),
                setup,
            ),
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                RuleEventPoint::InformationalRule,
                TriggerPhase::AfterEvent,
                OnceScope::Battle,
                EventFilter {
                    target_selector: Some(owner),
                    ..EventFilter::default()
                },
                defeat_guard_signal(effect),
                recover,
            ),
        ],
    )
}

fn synapse_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let attacked = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let highest = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 2)?;
    let linked = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 3)?;
    let program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let ratio = parameter_six(parameters, 0)?;
    let amount = actor_attack(ratio);
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner, attacked, highest, linked],
        vec![SYNAPSE_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ultimate_damage(attacked, amount.clone()),
        ultimate_damage(highest, amount),
        adjust_synapse(linked, -1),
    ]);
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(attacked).with_rule_units(attacked_linked_selector()?),
            SelectorDefinition::new(highest).with_rule_units(highest_linked_selector()?),
            SelectorDefinition::new(linked).with_rule_units(all_linked_selector()?),
        ],
        Vec::new(),
        vec![program_definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ActionResolved,
            TriggerPhase::AfterAction,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(owner),
                ability_tag: Some(AbilityTag::Attack),
                excluded_source: Some(resonance_source()),
                ..EventFilter::default()
            },
            ConditionExpr::All(
                vec![
                    ConditionExpr::SelectorCardinality {
                        selector: attacked,
                        operator: Comparison::Equal,
                        count: 1,
                    },
                    not_path_resonance(),
                ]
                .into_boxed_slice(),
            ),
            program,
        )],
    )
}

fn melt_core(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    chain_selected: bool,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let attacked = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let highest = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 2)?;
    let defeated = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 3)?;
    let attack_program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let defeat_program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let amount = actor_attack(parameter_six(parameters, 0)?);
    let mut programs = vec![
        ProgramDefinition::new(
            attack_program,
            Vec::new(),
            vec![owner, attacked, highest],
            vec![SYNAPSE_EFFECT],
            Vec::new(),
        )
        .with_steps(vec![ultimate_damage(highest, amount.clone())]),
    ];
    let mut triggers = vec![trigger(
        local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
        RuleEventPoint::ActionResolved,
        TriggerPhase::AfterAction,
        OnceScope::Action,
        EventFilter {
            actor_selector: Some(owner),
            action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
            excluded_source: Some(resonance_source()),
            ..EventFilter::default()
        },
        ConditionExpr::All(
            vec![
                ConditionExpr::SelectorCardinality {
                    selector: attacked,
                    operator: Comparison::Equal,
                    count: 1,
                },
                not_path_resonance(),
            ]
            .into_boxed_slice(),
        ),
        attack_program,
    )];
    if chain_selected {
        let repeats = 2;
        programs.push(
            ProgramDefinition::new(
                defeat_program,
                Vec::new(),
                vec![owner, highest, defeated],
                vec![SYNAPSE_EFFECT],
                Vec::new(),
            )
            .with_steps(
                (0..repeats)
                    .map(|_| ultimate_damage(highest, amount.clone()))
                    .collect(),
            ),
        );
        triggers.push(trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
            RuleEventPoint::UnitDefeated,
            TriggerPhase::AfterDefeatSettlement,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                target_selector: Some(defeated),
                action_kind: Some(starclock_combat::rule::model::RuleActionKind::Ultimate),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            defeat_program,
        ));
    }
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(attacked).with_rule_units(attacked_linked_selector()?),
            SelectorDefinition::new(highest).with_rule_units(highest_linked_selector()?),
            SelectorDefinition::new(defeated).with_rule_units(defeated_linked_selector()?),
        ],
        Vec::new(),
        programs,
        triggers,
    )
}

fn chain_contagion(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    base_parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let defeated = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let highest = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 2)?;
    let program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let repeats = whole(parameter_six(parameters, 0)?)?;
    let amount = actor_attack(parameter_six(base_parameters, 0)?);
    let steps = (0..repeats)
        .map(|_| ultimate_damage(highest, amount.clone()))
        .collect();
    finish(
        binding,
        RuleAttachment::EveryPlayer,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(defeated).with_rule_units(defeated_linked_selector()?),
            SelectorDefinition::new(highest).with_rule_units(highest_linked_selector()?),
        ],
        Vec::new(),
        vec![
            ProgramDefinition::new(
                program,
                Vec::new(),
                vec![owner, defeated, highest],
                vec![SYNAPSE_EFFECT],
                Vec::new(),
            )
            .with_steps(steps),
        ],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::UnitDefeated,
            TriggerPhase::AfterDefeatSettlement,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                target_selector: Some(defeated),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    )
}

fn memetic_inversion(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let allies = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let enemies = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 2)?;
    let entered = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 3)?;
    let group_program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let entered_program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 1)?;
    let ratio = scalar(parameter_six(parameters, 0)?);
    let combined_maximum = ValueExpr::SelectorSum {
        selector: allies,
        value: Box::new(ValueExpr::QueryMaximumEnergy(
            StatQuerySubject::CurrentTarget,
        )),
    };
    let group_amount = multiply(
        multiply(combined_maximum.clone(), ratio.clone()),
        ValueExpr::Convert {
            value: Box::new(ValueExpr::SelectorCount(enemies)),
            target: RuleValueKind::Scalar,
            rounding: Rounding::Floor,
        },
    );
    finish(
        binding,
        RuleAttachment::FirstPlayer,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_linkable_enemies_selector()?),
            SelectorDefinition::new(entered).with_rule_units(entered_enemy_selector()?),
        ],
        Vec::new(),
        vec![
            resource_gain_program(group_program, owner, allies, enemies, group_amount),
            resource_gain_program(
                entered_program,
                owner,
                allies,
                allies,
                multiply(combined_maximum, ratio),
            ),
        ],
        vec![
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
                RuleEventPoint::BattleStarted,
                TriggerPhase::AfterEvent,
                OnceScope::Battle,
                EventFilter::default(),
                ConditionExpr::SelectorCardinality {
                    selector: enemies,
                    operator: Comparison::GreaterOrEqual,
                    count: 1,
                },
                group_program,
            ),
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 1)?,
                RuleEventPoint::WaveStarted,
                TriggerPhase::AfterEvent,
                OnceScope::Wave,
                EventFilter::default(),
                ConditionExpr::SelectorCardinality {
                    selector: enemies,
                    operator: Comparison::GreaterOrEqual,
                    count: 1,
                },
                group_program,
            ),
            trigger(
                local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 2)?,
                RuleEventPoint::UnitSummoned,
                TriggerPhase::AfterEvent,
                OnceScope::Event,
                EventFilter {
                    target_selector: Some(entered),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                entered_program,
            ),
        ],
    )
}

fn resource_gain_program(
    id: ProgramId,
    owner: SelectorId,
    dependency: SelectorId,
    second_dependency: SelectorId,
    amount: ValueExpr,
) -> ProgramDefinition {
    ProgramDefinition::new(
        id,
        Vec::new(),
        {
            let mut selectors = vec![owner, dependency, second_dependency];
            selectors.sort_unstable();
            selectors.dedup();
            selectors
        },
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ModifyResource {
            selector: owner,
            resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
            update: ResourceUpdateKind::Gain,
            amount,
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        },
    )])
}

fn finish(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    selectors: Vec<SelectorDefinition>,
    effects: Vec<EffectDefinition>,
    programs: Vec<ProgramDefinition>,
    triggers: Vec<TriggerDef>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    Ok(propagation_s01::finish_rule(
        binding,
        attachment,
        Vec::new(),
        Vec::new(),
        selectors,
        effects,
        programs,
        triggers,
        Vec::new(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn trigger(
    id: TriggerId,
    event_point: RuleEventPoint,
    phase: TriggerPhase,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: event_point.kind(),
        event_point,
        phase,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn apply_effect(selector: SelectorId, effect: EffectDefinitionId, stacks: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: integer(stacks),
        chance: RuleEffectChancePolicy::Guaranteed,
        base_chance: None,
        rng_purpose: None,
    })
}

fn ultimate_damage(selector: SelectorId, amount: ValueExpr) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::UltimateDamageFromActorBasicElement {
        selector,
        amount,
        class: DamageClass::Additional,
        can_crit: true,
        can_defeat: true,
    })
}

fn adjust_synapse(selector: SelectorId, delta: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::AdjustEffectStacks {
        selector,
        effect: SYNAPSE_EFFECT,
        delta: integer(delta),
    })
}

fn actor_attack(ratio: i64) -> ValueExpr {
    multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Actor,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::AdditionalDamage,
        },
        scalar(ratio),
    )
}

fn maximum_hp(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::QueryStat {
        subject,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Healing,
    }
}

fn defeat_guard_signal(effect: EffectDefinitionId) -> ConditionExpr {
    ConditionExpr::All(
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
    )
}

fn not_path_resonance() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::SourceDefinitionId,
        )),
        operator: Comparison::NotEqual,
        rhs: Box::new(ValueExpr::Literal(RuleValue::OptionalStableId(Some(
            u64::from(resonance_source().get()),
        )))),
    }
}

fn synapse_runtime(maximum: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Mark,
        DispelCategory::NonDispellable,
        maximum,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn attacked_linked_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::EventTargets,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::RngUniform,
        1,
        Some("damage-target".into()),
    )
    .map(|selector| {
        selector.with_predicates(vec![RuleSelectorPredicate::HasEffect(SYNAPSE_EFFECT)])
    })
}

fn highest_linked_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::StatDescending,
        RuleSelectorChoice::First,
        1,
        None,
    )
    .map(|selector| {
        selector
            .with_predicates(vec![RuleSelectorPredicate::HasEffect(SYNAPSE_EFFECT)])
            .with_weight(Some(maximum_hp(StatQuerySubject::CurrentTarget)))
    })
}

fn all_linked_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Any,
        RulePresencePredicate::Any,
        RuleSelectorOrdering::StableId,
        RuleSelectorChoice::All,
        16,
        None,
    )
    .map(|selector| {
        selector.with_predicates(vec![RuleSelectorPredicate::HasEffect(SYNAPSE_EFFECT)])
    })
}

fn defeated_linked_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Defeated,
        RulePresencePredicate::Any,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::First,
        1,
        None,
    )
    .map(|selector| {
        selector.with_predicates(vec![RuleSelectorPredicate::HasEffect(SYNAPSE_EFFECT)])
    })
}

fn all_linkable_enemies_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Encounter,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::Formation,
        RuleSelectorChoice::All,
        16,
        None,
    )
}

fn current_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::CurrentSubject,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::StableId,
        RuleSelectorChoice::First,
        1,
        None,
    )
}

fn entered_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorOrdering::EventOrder,
        RuleSelectorChoice::First,
        1,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn selector(
    origin: RuleSelectorOrigin,
    side: RuleSelectorSide,
    life: RuleLifePredicate,
    presence: RulePresencePredicate,
    ordering: RuleSelectorOrdering,
    choice: RuleSelectorChoice,
    maximum: u16,
    rng_purpose: Option<Box<str>>,
) -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        origin,
        side,
        life,
        presence,
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
        .map(crate::path::ResonanceDefinition::parameters)
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)
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
