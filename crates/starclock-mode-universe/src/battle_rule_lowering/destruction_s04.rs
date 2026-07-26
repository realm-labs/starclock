use super::*;
use starclock_combat::{
    catalog::action::{HitTargetGroup, ReactionBoundary},
    rule::model::{RuleActionOwner, RuleActionPaymentPolicy},
};

const LOST_HP_DEFENSE: &str = "StageAbility_61255601";
const LOST_HP_EFFECT_RESISTANCE: &str = "StageAbility_61255701";
pub(super) const RESONANCE: &str = "StageAbility_612520";
const CATACLYSMIC_VARIABLE: &str = "StageAbility_612521";
const EXTREME_HELIUM_FLASH: &str = "StageAbility_612522";
const EVENT_HORIZON: &str = "StageAbility_612523";

const CATACLYSMIC_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x79f0_0001).expect("reserved effect ID");
const ENTROPIC_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x79f0_0002).expect("reserved effect ID");
const ENTROPIC_DEFENSE: ModifierDefinitionId =
    ModifierDefinitionId::new(0x79f0_0003).expect("reserved modifier ID");
const ENTROPIC_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x79f0_0004).expect("reserved group ID");
const AUTO_RESONANCE_ABILITY: AbilityId = AbilityId::new(0x79f0_0005).expect("reserved ability ID");
const CATACLYSMIC_ROOT: ProgramId = ProgramId::new(0x79f0_0006).expect("reserved program ID");
const CATACLYSMIC_BODY: ProgramId = ProgramId::new(0x79f0_0007).expect("reserved program ID");
const ENTROPIC_APPLY: ProgramId = ProgramId::new(0x79f0_0008).expect("reserved program ID");
const CURRENT_ALLY: SelectorId = SelectorId::new(0x79f0_0009).expect("reserved selector ID");
const ENTROPIC_BASE_CHANCE: i64 = 1_500_000;

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for (key, stat) in [
        (LOST_HP_DEFENSE, StatKind::Def),
        (LOST_HP_EFFECT_RESISTANCE, StatKind::EffectResistance),
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(destruction_s02::missing_hp_stat_rule(
            binding,
            &[(stat, destruction_s01::parameter_six(parameters, 0)?)],
        )?);
    }
    for key in [CATACLYSMIC_VARIABLE, EXTREME_HELIUM_FLASH, EVENT_HORIZON] {
        let Some(binding) = resonance_binding(bindings, key) else {
            continue;
        };
        let parameters = resonance_parameters(catalog, binding)?;
        output.push(match key {
            CATACLYSMIC_VARIABLE => cataclysmic_rule(binding, parameters)?,
            EXTREME_HELIUM_FLASH => entropic_rule(binding, parameters)?,
            EVENT_HORIZON => event_horizon_rule(binding, parameters)?,
            _ => unreachable!("closed Destruction formation set"),
        });
    }
    Ok(output)
}

pub(super) fn resonance(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
    damage_ratio: i64,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let parameters = resonance_parameters(catalog, binding)?;
    let cataclysmic = resonance_binding(bindings, CATACLYSMIC_VARIABLE).is_some();
    let entropic = resonance_binding(bindings, EXTREME_HELIUM_FLASH).is_some();
    let formation_bonus = if cataclysmic {
        parameter(parameters, 3)?
    } else {
        0
    };
    let multiplier = 1_000_000_i64
        .checked_add(damage_ratio)
        .and_then(|value| value.checked_add(formation_bonus))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let amount = multiply(
        ValueExpr::SelectorSum {
            selector: RESONANCE_ALLY_SELECTOR_ID,
            value: Box::new(ValueExpr::Subtract(
                Box::new(maximum_hp(StatQuerySubject::CurrentTarget)),
                Box::new(ValueExpr::QueryHp {
                    subject: StatQuerySubject::CurrentTarget,
                }),
            )),
        },
        multiply(scalar(parameter(parameters, 0)?), scalar(multiplier)),
    );
    let main = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID, RESONANCE_ALLY_SELECTOR_ID],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: RESONANCE_ENEMY_SELECTOR_ID,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Fire,
            can_crit: false,
            can_defeat: true,
        },
    )]);

    let mut programs = vec![main];
    let mut bindings_for_ability = Vec::new();
    if cataclysmic {
        let floor_ratio = parameter(parameters, 2)?;
        let floor = multiply(
            maximum_hp(StatQuerySubject::CurrentTarget),
            scalar(floor_ratio),
        );
        let consumed = ValueExpr::Maximum(
            Box::new(ValueExpr::Subtract(
                Box::new(ValueExpr::QueryHp {
                    subject: StatQuerySubject::CurrentTarget,
                }),
                Box::new(floor.clone()),
            )),
            Box::new(scalar(0)),
        );
        let root = ProgramDefinition::new(
            CATACLYSMIC_ROOT,
            vec![CATACLYSMIC_BODY],
            vec![RESONANCE_ALLY_SELECTOR_ID],
            Vec::new(),
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::ForEach {
            selector: RESONANCE_ALLY_SELECTOR_ID,
            body: CATACLYSMIC_BODY,
            maximum: 16,
        }]);
        let body = ProgramDefinition::new(
            CATACLYSMIC_BODY,
            Vec::new(),
            vec![CURRENT_ALLY],
            vec![CATACLYSMIC_EFFECT],
            Vec::new(),
        )
        .with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::ConsumeHp {
                selector: CURRENT_ALLY,
                amount: ValueExpr::QueryHp {
                    subject: StatQuerySubject::CurrentTarget,
                },
                floor,
            }),
            ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
                selector: CURRENT_ALLY,
                effect: CATACLYSMIC_EFFECT,
            }),
            ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
                selector: CURRENT_ALLY,
                effect: CATACLYSMIC_EFFECT,
                stacks: integer(1),
                chance: RuleEffectChancePolicy::Guaranteed,
                base_chance: None,
                rng_purpose: None,
            }),
            ProgramStep::Operation(RuleOperationTemplate::Shield {
                selector: CURRENT_ALLY,
                amount: consumed,
                effect: CATACLYSMIC_EFFECT,
            }),
        ]);
        programs.extend([root, body]);
        bindings_for_ability.push(
            AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, CATACLYSMIC_ROOT)
                .expect("non-zero sequence"),
        );
    }
    if entropic {
        let apply = ProgramDefinition::new(
            ENTROPIC_APPLY,
            Vec::new(),
            vec![RESONANCE_ENEMY_SELECTOR_ID],
            vec![ENTROPIC_EFFECT],
            Vec::new(),
        )
        .with_steps(vec![ProgramStep::Operation(
            RuleOperationTemplate::ApplyEffect {
                selector: RESONANCE_ENEMY_SELECTOR_ID,
                effect: ENTROPIC_EFFECT,
                stacks: integer(1),
                chance: RuleEffectChancePolicy::Resistible,
                base_chance: Some(scalar(ENTROPIC_BASE_CHANCE)),
                rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
            },
        )]);
        programs.push(apply);
        bindings_for_ability.push(
            AbilityProgramBinding::new(2, AbilityProgramTiming::BeforeHits, ENTROPIC_APPLY)
                .expect("non-zero sequence"),
        );
    }
    bindings_for_ability.push(
        AbilityProgramBinding::new(3, AbilityProgramTiming::Hits, RESONANCE_PROGRAM_ID)
            .expect("non-zero sequence"),
    );
    programs.sort_unstable_by_key(ProgramDefinition::id);

    let manual = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(resonance_action(AbilityKind::Ultimate, true)?)
    .with_programs(bindings_for_ability.clone());
    let automatic = AbilityDefinition::new(
        AUTO_RESONANCE_ABILITY,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(resonance_action(AbilityKind::ExtraAction, false)?)
    .with_programs(bindings_for_ability);
    let mut selectors = vec![
        SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
            UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        ),
        SelectorDefinition::new(RESONANCE_ENEMY_SELECTOR_ID).with_rule_units(all_enemy_selector()?),
        SelectorDefinition::new(RESONANCE_ALLY_SELECTOR_ID).with_rule_units(all_ally_selector()?),
    ];
    if cataclysmic {
        selectors.push(
            SelectorDefinition::new(CURRENT_ALLY).with_rule_units(current_subject_selector()?),
        );
    }
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    Ok(ExecutableResonance {
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: Box::new([]),
        programs: programs.into_boxed_slice(),
        ability: manual,
        auxiliary_abilities: vec![automatic].into_boxed_slice(),
        countdowns: Box::new([]),
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn cataclysmic_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let expire = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let duration = whole(parameter(parameters, 4)?)?;
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Shield,
        DispelCategory::NonDispellable,
        1,
        Some(integer(i64::from(duration))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let program = ProgramDefinition::new(
        expire,
        Vec::new(),
        vec![owner],
        vec![CATACLYSMIC_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveShield {
            selector: owner,
            effect: CATACLYSMIC_EFFECT,
        },
    )]);
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(CATACLYSMIC_EFFECT, Vec::new(), Vec::new())
                .with_runtime_template(runtime),
        ],
        vec![program],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectRemoved,
            OnceScope::Event,
            EventFilter {
                target_selector: Some(owner),
                effect_definition: Some(CATACLYSMIC_EFFECT),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            expire,
        )],
    ))
}

fn entropic_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let players = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let tick = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let duration = whole(parameter(parameters, 5)?)?;
    let amount = multiply(
        ValueExpr::SelectorSum {
            selector: players,
            value: Box::new(ValueExpr::Subtract(
                Box::new(maximum_hp(StatQuerySubject::CurrentTarget)),
                Box::new(ValueExpr::QueryHp {
                    subject: StatQuerySubject::CurrentTarget,
                }),
            )),
        },
        scalar(parameter(parameters, 7)?),
    );
    let program = ProgramDefinition::new(
        tick,
        Vec::new(),
        vec![owner, players],
        vec![ENTROPIC_EFFECT],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: owner,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Fire,
            can_crit: false,
            can_defeat: true,
        },
    )]);
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(integer(i64::from(duration))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let modifier = ModifierDefinition {
        id: ENTROPIC_DEFENSE,
        stat: StatKind::Def,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: ValueExpr::Negate(Box::new(scalar(parameter(parameters, 6)?))),
        stacking_group: ENTROPIC_GROUP,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    };
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::EveryEnemy,
        vec![ModifierStackingGroup {
            id: ENTROPIC_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }],
        vec![modifier],
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(players).with_rule_units(all_enemy_selector()?),
        ],
        vec![
            EffectDefinition::new(ENTROPIC_EFFECT, Vec::new(), vec![ENTROPIC_DEFENSE])
                .with_runtime_template(runtime),
        ],
        vec![program],
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::TurnStarted,
            OnceScope::Turn,
            EventFilter {
                owner_selector: Some(owner),
                ..EventFilter::default()
            },
            ConditionExpr::EffectExists {
                selector: owner,
                effect: ENTROPIC_EFFECT,
            },
            tick,
        )],
    ))
}

fn event_horizon_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let actor = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let allies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let activate = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let count = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let maximum = whole(parameter(parameters, 9)?)?;
    let program = ProgramDefinition::new(
        activate,
        Vec::new(),
        vec![actor, enemies],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::AddSlot {
            slot: count,
            value: integer(1),
        }),
        ProgramStep::Operation(RuleOperationTemplate::QueueAction {
            actor_selector: actor,
            target_selector: enemies,
            ability: AUTO_RESONANCE_ABILITY,
            priority: ReactionPriority::new(-100),
            forced_use: true,
            boundary: ReactionBoundary::AfterAction,
            owner: RuleActionOwner::Actor,
            payment: Some(RuleActionPaymentPolicy::Suppressed),
        }),
    ]);
    let hp_ratio = ValueExpr::Divide {
        lhs: Box::new(ValueExpr::ReadEventProperty(EventValueProperty::HpAfter)),
        rhs: Box::new(maximum_hp(StatQuerySubject::EventTarget)),
        rounding: Rounding::NearestTiesEven,
    };
    let condition = ConditionExpr::All(
        vec![
            ConditionExpr::Compare {
                lhs: Box::new(hp_ratio),
                operator: Comparison::Less,
                rhs: Box::new(scalar(parameter(parameters, 8)?)),
            },
            ConditionExpr::Compare {
                lhs: Box::new(ValueExpr::Slot(count)),
                operator: Comparison::Less,
                rhs: Box::new(integer(i64::from(maximum))),
            },
        ]
        .into_boxed_slice(),
    );
    Ok(executable_with_attachment(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(actor).with_rule_units(owner_selector()?),
            SelectorDefinition::new(allies).with_rule_units(all_ally_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        Vec::new(),
        vec![program],
        vec![
            StateSlotDef::new(
                count,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(
                RuleValue::Integer(0),
                RuleValue::Integer(i64::from(maximum)),
            ),
        ],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::Action,
            EventFilter {
                actor_selector: Some(enemies),
                ability_tag: Some(AbilityTag::Attack),
                has_action: Some(true),
                ..EventFilter::default()
            },
            condition,
            activate,
        )],
    ))
}

fn resonance_action(
    kind: AbilityKind,
    consumes_energy: bool,
) -> Result<AbilityActionDefinition, BattleRuleLoweringError> {
    let mut resources = ActionResourcePolicy::new(
        0,
        0,
        starclock_combat::Energy::ZERO,
        starclock_combat::Energy::ZERO,
    );
    if consumes_energy {
        resources = resources
            .with_team_resource_costs(vec![
                TeamResourceCost::new(RESONANCE_RESOURCE_KEY, 100)
                    .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
            ])
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    }
    AbilityActionDefinition::new(
        kind,
        1,
        TargetInvalidationPolicy::CancelRemainingForTarget,
        resources,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?
    .with_tags(&[AbilityTag::Attack, AbilityTag::Ultimate, AbilityTag::Assist])
    .with_hits(vec![ActionHitDefinition::new(Vec::new()).with_profile(
        HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    )])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn maximum_hp(subject: StatQuerySubject) -> ValueExpr {
    ValueExpr::QueryStat {
        subject,
        stat: StatKind::Hp,
        purpose: FormulaPurpose::Stat,
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

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    value
        .checked_div(1_000_000)
        .and_then(|value| u16::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidParameter)
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
