use super::*;
use starclock_combat::catalog::{
    action::{HitTargetGroup, ReactionBoundary},
    selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
};
use starclock_combat::rule::model::{RuleActionOwner, RuleActionPaymentPolicy};

const SPORANGIUM: &str = "StageAbility_612756";
const VESICLE: &str = "StageAbility_612757";
pub(super) const RESONANCE: &str = "StageAbility_612720";
const PROBOSCIS: &str = "StageAbility_612721";
const PHENOL_COMPOUNDS: &str = "StageAbility_612722";
const CRYSTAL_PINCERS: &str = "StageAbility_612723";

pub(super) const METAMORPHOSIS_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7ca0_0001).expect("reserved Propagation S04 effect ID");
const CRYSTAL_MARKER_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7ca0_0002).expect("reserved Propagation S04 effect ID");
const CRYSTAL_ABILITY: AbilityId =
    AbilityId::new(0x7ca0_0003).expect("reserved Propagation S04 ability ID");
const RESONANCE_PRIMARY_ALLY: SelectorId =
    SelectorId::new(0x7ca0_0004).expect("reserved Propagation S04 selector ID");
const RESONANCE_ACTOR: SelectorId =
    SelectorId::new(0x7ca0_0005).expect("reserved Propagation S04 selector ID");
const CRYSTAL_TARGET: SelectorId =
    SelectorId::new(0x7ca0_0006).expect("reserved Propagation S04 selector ID");
const CRYSTAL_PROGRAM: ProgramId =
    ProgramId::new(0x7ca0_0007).expect("reserved Propagation S04 program ID");
const CRYSTAL_REMOVE_PROGRAM: ProgramId =
    ProgramId::new(0x7ca0_0008).expect("reserved Propagation S04 program ID");

const LOCAL_PROGRAM_BASE: u32 = 0x7c20_0000;
const LOCAL_SELECTOR_BASE: u32 = 0x7c30_0000;
const LOCAL_TRIGGER_BASE: u32 = 0x7c60_0000;

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [SPORANGIUM, VESICLE] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            SPORANGIUM => spend_energy(binding, parameters)?,
            VESICLE => spend_healing(binding, parameters)?,
            _ => unreachable!("closed Propagation S04 Blessing set"),
        });
    }
    if let Some(binding) = resonance_binding(bindings, PROBOSCIS) {
        output.push(proboscis(
            binding,
            resonance_parameters(catalog, binding)?,
            resonance_maximum(bindings),
        )?);
    }
    if let Some(binding) = resonance_binding(bindings, PHENOL_COMPOUNDS) {
        output.push(phenol_compounds(
            binding,
            resonance_parameters(catalog, binding)?,
        )?);
    }
    if let Some(binding) = resonance_binding(bindings, CRYSTAL_PINCERS) {
        output.push(crystal_pincers(
            binding,
            resonance_parameters(catalog, binding)?,
        )?);
    }
    Ok(output)
}

pub(super) fn crystal_pincers_selected(bindings: &[UniverseBattleRuleBinding]) -> bool {
    resonance_binding(bindings, CRYSTAL_PINCERS).is_some()
}

pub(super) fn resonance(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let parameters = resonance_parameters(catalog, binding)?;
    let proboscis = resonance_binding(bindings, PROBOSCIS).is_some();
    let crystal = resonance_binding(bindings, CRYSTAL_PINCERS)
        .map(|binding| resonance_parameters(catalog, binding))
        .transpose()?;
    let duration = if proboscis { 2 } else { 1 };
    let maximum_energy = resonance_maximum(bindings);

    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut metamorph_modifiers = Vec::new();
    if let Some(crystal) = crystal {
        let group = ModifierStackingGroupId::new(0x7ca0_0010)
            .expect("reserved Propagation S04 modifier group ID");
        groups.push(ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        for (index, purpose) in damage_purposes().into_iter().enumerate() {
            let id = ModifierDefinitionId::new(
                0x7ca0_0020_u32
                    .checked_add(
                        u32::try_from(index)
                            .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
                    )
                    .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            )
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
            modifiers.push(ModifierDefinition {
                id,
                stat: StatKind::Atk,
                stage: FormulaStage::DamageBoost,
                purpose,
                value: scalar(parameter_six(crystal, 0)?),
                stacking_group: group,
                priority: 0,
                floor: Some(starclock_combat::Scalar::ZERO),
                cap: None,
                cap_stage: FormulaStage::DamageBoost,
                snapshot: SnapshotPolicy::Dynamic,
                source_stack_slot: None,
                filters: vec![ModifierFilter::FormulaSubject(FormulaSubject::Source)]
                    .into_boxed_slice(),
            });
            metamorph_modifiers.push(id);
        }
    }
    let metamorphosis =
        EffectDefinition::new(METAMORPHOSIS_EFFECT, Vec::new(), metamorph_modifiers)
            .with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::Buff,
                    DispelCategory::NonDispellable,
                    1,
                    Some(integer(i64::from(duration))),
                    DurationClock::OwnerTurnEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Replace,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            );
    let main = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![
            RESONANCE_ALLY_SELECTOR_ID,
            RESONANCE_PRIMARY_ALLY,
            RESONANCE_ACTOR,
        ],
        vec![METAMORPHOSIS_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::RemoveEffect {
            selector: RESONANCE_ALLY_SELECTOR_ID,
            effect: METAMORPHOSIS_EFFECT,
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: RESONANCE_PRIMARY_ALLY,
            effect: METAMORPHOSIS_EFFECT,
            stacks: integer(1),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
            selector: RESONANCE_PRIMARY_ALLY,
            resource: RuleResourceKind::SkillPoints,
            update: ResourceUpdateKind::Gain,
            amount: scalar(parameter_six(parameters, 0)?),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        }),
        ProgramStep::Operation(RuleOperationTemplate::AdvanceAction {
            selector: RESONANCE_PRIMARY_ALLY,
            amount: scalar(1_000_000),
        }),
    ]);
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
    .with_tags(&[AbilityTag::Ultimate, AbilityTag::Assist])
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

    let mut effects = vec![metamorphosis];
    let mut programs = vec![main];
    let mut auxiliary_abilities = Vec::new();
    let mut selectors = vec![
        SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Allied, TargetPattern::Single)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        ),
        SelectorDefinition::new(RESONANCE_PRIMARY_ALLY).with_rule_units(primary_ally_selector()?),
        SelectorDefinition::new(RESONANCE_ALLY_SELECTOR_ID).with_rule_units(all_ally_selector()?),
        SelectorDefinition::new(RESONANCE_ACTOR).with_rule_units(actor_player_selector()?),
    ];
    if let Some(crystal) = crystal {
        let marker = EffectDefinition::new(CRYSTAL_MARKER_EFFECT, Vec::new(), Vec::new())
            .with_runtime_template(
                EffectRuntimeTemplate::new(
                    EffectCategory::NeutralState,
                    DispelCategory::NonDispellable,
                    whole(parameter_six(crystal, 2)?)?,
                    Some(integer(1)),
                    DurationClock::ActionEnd,
                    EffectTickPhase::None,
                    EffectStackPolicy::Replace,
                )
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            );
        effects.push(marker);
        let stack_scalar = ValueExpr::Convert {
            value: Box::new(ValueExpr::QueryEffectStacks {
                subject: StatQuerySubject::CurrentTarget,
                effect: CRYSTAL_MARKER_EFFECT,
            }),
            target: RuleValueKind::Scalar,
            rounding: Rounding::NearestTiesEven,
        };
        let amount = multiply(
            multiply(
                ValueExpr::QueryStat {
                    subject: StatQuerySubject::Actor,
                    stat: StatKind::Atk,
                    purpose: FormulaPurpose::OrdinaryDamage,
                },
                scalar(parameter_six(crystal, 1)?),
            ),
            stack_scalar,
        );
        programs.extend([
            ProgramDefinition::new(
                CRYSTAL_PROGRAM,
                Vec::new(),
                vec![CRYSTAL_TARGET],
                vec![CRYSTAL_MARKER_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DamageFromActorBasicElement {
                    selector: CRYSTAL_TARGET,
                    amount,
                    class: DamageClass::Direct,
                    can_crit: true,
                    can_defeat: true,
                },
            )]),
            ProgramDefinition::new(
                CRYSTAL_REMOVE_PROGRAM,
                Vec::new(),
                vec![CRYSTAL_TARGET],
                vec![CRYSTAL_MARKER_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: CRYSTAL_TARGET,
                    effect: CRYSTAL_MARKER_EFFECT,
                },
            )]),
        ]);
        selectors.push(
            SelectorDefinition::new(CRYSTAL_TARGET)
                .with_unit_targets(
                    UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::Single)
                        .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
                )
                .with_rule_units(primary_enemy_selector()?),
        );
        auxiliary_abilities.push(crystal_ability()?);
    }
    Ok(ExecutableResonance {
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        ability,
        auxiliary_abilities: auxiliary_abilities.into_boxed_slice(),
        countdowns: Box::new([]),
        initial_energy: initial_energy.min(maximum_energy),
        maximum_energy,
    })
}

fn spend_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let amount = multiply(
        ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ResourceDelta,
        ))),
        scalar(parameter_six(parameters, 0)?),
    );
    spend_rule(
        binding,
        RuleOperationTemplate::ModifyResource {
            selector: local::<SelectorId>(LOCAL_SELECTOR_BASE, binding.rule().get(), 0)?,
            resource: RuleResourceKind::Energy,
            update: ResourceUpdateKind::Gain,
            amount,
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        },
    )
}

fn spend_healing(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, binding.rule().get(), 0)?;
    let points = ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
        EventValueProperty::ResourceDelta,
    )));
    let amount = multiply(
        multiply(
            ValueExpr::QueryStat {
                subject: StatQuerySubject::Owner,
                stat: StatKind::Hp,
                purpose: FormulaPurpose::Healing,
            },
            scalar(parameter_six(parameters, 0)?),
        ),
        points,
    );
    spend_rule(
        binding,
        RuleOperationTemplate::Heal {
            selector: owner,
            amount,
            apply_formula_modifiers: false,
        },
    )
}

fn spend_rule(
    binding: &UniverseBattleRuleBinding,
    operation: RuleOperationTemplate,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let program = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(operation)]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ResourceChanged,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                resource: Some(RuleResourceKind::SkillPoints),
                ..EventFilter::default()
            },
            resource_spent(),
            program,
        )],
        Vec::new(),
    ))
}

fn proboscis(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
    maximum_energy: u16,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let gain = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let amount = i64::from(maximum_energy)
        .checked_mul(parameter_six(parameters, 0)?)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let definition = ProgramDefinition::new(gain, Vec::new(), vec![actor], Vec::new(), Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::ModifyResource {
                selector: actor,
                resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
                update: ResourceUpdateKind::Gain,
                amount: scalar(amount),
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            },
        )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?)],
        Vec::new(),
        vec![definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::UnitDefeated,
            TriggerPhase::AfterDefeatSettlement,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(actor),
                ..EventFilter::default()
            },
            ConditionExpr::EffectExists {
                selector: actor,
                effect: METAMORPHOSIS_EFFECT,
            },
            gain,
        )],
        Vec::new(),
    ))
}

fn phenol_compounds(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let gain = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let per_point = 200_i64
        .checked_mul(parameter_six(parameters, 0)?)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let absolute_delta = ValueExpr::Choose {
        condition: Box::new(resource_spent()),
        when_true: Box::new(ValueExpr::Negate(Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ResourceDelta,
        )))),
        when_false: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ResourceDelta,
        )),
    };
    let amount = multiply(absolute_delta, scalar(per_point));
    let definition = ProgramDefinition::new(gain, Vec::new(), vec![actor], Vec::new(), Vec::new())
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::ModifyResource {
                selector: actor,
                resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
                update: ResourceUpdateKind::Gain,
                amount,
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            },
        )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?)],
        Vec::new(),
        vec![definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::ResourceChanged,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(actor),
                resource: Some(RuleResourceKind::SkillPoints),
                ..EventFilter::default()
            },
            resource_changed(),
            gain,
        )],
        Vec::new(),
    ))
}

fn crystal_pincers(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 0)?;
    let target = local::<SelectorId>(LOCAL_SELECTOR_BASE, raw, 1)?;
    let queue = local::<ProgramId>(LOCAL_PROGRAM_BASE, raw, 0)?;
    let maximum = whole(parameter_six(parameters, 2)?)?;
    let definition = ProgramDefinition::new(
        queue,
        Vec::new(),
        vec![actor, target],
        vec![CRYSTAL_MARKER_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: target,
            effect: CRYSTAL_MARKER_EFFECT,
            stacks: ValueExpr::Minimum(
                Box::new(ValueExpr::Convert {
                    value: Box::new(ValueExpr::ReadEventProperty(
                        EventValueProperty::RuleSignalValue,
                    )),
                    target: RuleValueKind::Integer,
                    rounding: Rounding::Floor,
                }),
                Box::new(integer(i64::from(maximum))),
            ),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
        ProgramStep::Operation(RuleOperationTemplate::QueueAction {
            actor_selector: actor,
            target_selector: target,
            ability: CRYSTAL_ABILITY,
            priority: ReactionPriority::new(0),
            forced_use: true,
            boundary: ReactionBoundary::AfterHit,
            owner: RuleActionOwner::Actor,
            payment: Some(RuleActionPaymentPolicy::Suppressed),
        }),
    ]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(actor).with_rule_units(actor_player_selector()?),
            SelectorDefinition::new(target).with_rule_units(primary_enemy_selector()?),
        ],
        Vec::new(),
        vec![definition],
        vec![trigger(
            local::<TriggerId>(LOCAL_TRIGGER_BASE, raw, 0)?,
            RuleEventPoint::InformationalRule,
            TriggerPhase::AfterEvent,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(actor),
                ..EventFilter::default()
            },
            ConditionExpr::All(
                vec![
                    signal_condition(propagation_s01::SPORE_BURST_SIGNAL),
                    ConditionExpr::EffectExists {
                        selector: actor,
                        effect: METAMORPHOSIS_EFFECT,
                    },
                ]
                .into_boxed_slice(),
            ),
            queue,
        )],
        Vec::new(),
    ))
}

fn crystal_ability() -> Result<AbilityDefinition, BattleRuleLoweringError> {
    let action = AbilityActionDefinition::new(
        AbilityKind::ExtraAction,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        ActionResourcePolicy::new(
            0,
            0,
            starclock_combat::Energy::ZERO,
            starclock_combat::Energy::ZERO,
        ),
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_tags(&[AbilityTag::Basic, AbilityTag::AdditionalDamage])
    .with_hits(vec![ActionHitDefinition::new(Vec::new()).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::PerTarget,
    )])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    Ok(
        AbilityDefinition::new(CRYSTAL_ABILITY, CRYSTAL_PROGRAM, CRYSTAL_TARGET, Vec::new())
            .with_action(action)
            .with_programs(vec![
                AbilityProgramBinding::new(1, AbilityProgramTiming::Hits, CRYSTAL_PROGRAM)
                    .expect("non-zero sequence"),
                AbilityProgramBinding::new(
                    2,
                    AbilityProgramTiming::AfterHits,
                    CRYSTAL_REMOVE_PROGRAM,
                )
                .expect("non-zero sequence"),
            ]),
    )
}

fn resonance_maximum(bindings: &[UniverseBattleRuleBinding]) -> u16 {
    if resonance_binding(bindings, PHENOL_COMPOUNDS).is_some() {
        200
    } else {
        100
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

fn trigger(
    id: TriggerId,
    point: RuleEventPoint,
    phase: TriggerPhase,
    once_scope: OnceScope,
    filter: EventFilter,
    condition: ConditionExpr,
    program: ProgramId,
) -> TriggerDef {
    TriggerDef {
        id,
        event: point.kind(),
        event_point: point,
        phase,
        filter,
        condition,
        once_scope,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn resource_spent() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ResourceDelta,
        )),
        operator: Comparison::Less,
        rhs: Box::new(scalar(0)),
    }
}

fn resource_changed() -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ResourceDelta,
        )),
        operator: Comparison::NotEqual,
        rhs: Box::new(scalar(0)),
    }
}

fn signal_condition(code: u32) -> ConditionExpr {
    ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::RuleSignalCode,
        )),
        operator: Comparison::Equal,
        rhs: Box::new(integer(i64::from(code))),
    }
}

fn owner_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

fn actor_player_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

fn primary_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Same,
        RuleSelectorChoice::First,
        1,
    )
}

fn primary_enemy_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleSelectorChoice::First,
        1,
    )
}

fn all_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    selector(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleSelectorChoice::All,
        16,
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
        0,
        maximum,
        RuleEmptyPoolPolicy::NoOp,
        choice,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
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
