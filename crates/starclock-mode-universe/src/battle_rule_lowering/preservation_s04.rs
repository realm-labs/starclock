use super::*;

const BURST: &str = "StageAbility_612056";
const CONCENTRATION: &str = "StageAbility_612057";
pub(super) const RESONANCE: &str = "StageAbility_612020";
const CRITICAL: &str = "StageAbility_612021";
const EUTECTIC: &str = "StageAbility_612022";
const ISOMORPHOUS: &str = "StageAbility_612023";
const SECOND_EFFECT_ID_BASE: u32 = 0x76e0_0000;
pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for (key, stat) in [
        (BURST, StatKind::CritDamage),
        (CONCENTRATION, StatKind::CritRate),
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        let value = ValueExpr::Choose {
            condition: Box::new(ConditionExpr::Compare {
                lhs: Box::new(shield(StatQuerySubject::Owner, ShieldObservation::Current)),
                operator: Comparison::Greater,
                rhs: Box::new(scalar(0)),
            }),
            when_true: Box::new(scalar(parameter(parameters, 0)?)),
            when_false: Box::new(scalar(0)),
        };
        output.push(preservation_s02::persistent_modifier_rule(
            binding,
            stat,
            FormulaStage::Flat,
            FormulaPurpose::Stat,
            value,
            Vec::new(),
        )?);
    }
    for key in [EUTECTIC, ISOMORPHOUS] {
        let Some(binding) = resonance_binding(bindings, key) else {
            continue;
        };
        output.push(match key {
            EUTECTIC => eutectic_rule(binding, resonance_parameters(catalog, binding)?)?,
            ISOMORPHOUS => energy_rule(binding, resonance_parameters(catalog, binding)?)?,
            _ => unreachable!("closed Preservation resonance set"),
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
    let definition = catalog
        .resonances()
        .iter()
        .find(|definition| definition.stable_key() == binding.source_record_key())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    let base_ratio = parameter(definition.parameters(), 0)?;
    let boosted_ratio = multiply(
        scalar(base_ratio),
        scalar(
            1_000_000_i64
                .checked_add(damage_ratio)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ),
    );
    let party_shield = ValueExpr::SelectorSum {
        selector: RESONANCE_ALLY_SELECTOR_ID,
        value: Box::new(shield(
            StatQuerySubject::CurrentTarget,
            ShieldObservation::Current,
        )),
    };
    let mut amount = multiply(party_shield, boosted_ratio);
    if let Some(critical) = resonance_binding(bindings, CRITICAL) {
        let parameters = resonance_parameters(catalog, critical)?;
        let shielded = ValueExpr::SelectorSum {
            selector: RESONANCE_ALLY_SELECTOR_ID,
            value: Box::new(ValueExpr::Choose {
                condition: Box::new(ConditionExpr::Compare {
                    lhs: Box::new(shield(
                        StatQuerySubject::CurrentTarget,
                        ShieldObservation::Current,
                    )),
                    operator: Comparison::Greater,
                    rhs: Box::new(scalar(0)),
                }),
                when_true: Box::new(scalar(1_000_000)),
                when_false: Box::new(scalar(0)),
            }),
        };
        let critical_multiplier = ValueExpr::Add(
            Box::new(scalar(
                1_000_000_i64
                    .checked_add(parameter(parameters, 1)?)
                    .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            )),
            Box::new(multiply(shielded, scalar(parameter(parameters, 3)?))),
        );
        amount = multiply(amount, critical_multiplier);
    }
    let program = ProgramDefinition::new(
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
            element: CombatElement::Physical,
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
    .with_tags(&[AbilityTag::Attack, AbilityTag::Ultimate, AbilityTag::Assist])
    .with_hits(vec![ActionHitDefinition::new(Vec::new()).with_profile(
        starclock_combat::catalog::action::HitTargetGroup::Selected,
        Ratio::ONE,
        Ratio::ONE,
        HitCritPolicy::Never,
    )])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let ability_selector = SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    );
    let enemy_selector =
        SelectorDefinition::new(RESONANCE_ENEMY_SELECTOR_ID).with_rule_units(all_enemy_selector()?);
    let ally_selector =
        SelectorDefinition::new(RESONANCE_ALLY_SELECTOR_ID).with_rule_units(all_ally_selector()?);
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
        selectors: vec![ability_selector, enemy_selector, ally_selector].into_boxed_slice(),
        effects: Box::new([]),
        programs: vec![program].into_boxed_slice(),
        ability,
        auxiliary_abilities: Box::new([]),
        countdowns: Box::new([]),
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn eutectic_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let advance = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let expire = id::<ProgramId>(SECOND_AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let shield_effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let amber_effect = id::<EffectDefinitionId>(SECOND_EFFECT_ID_BASE, raw)?;
    let counter = id::<StateSlotDefinitionId>(COUNTER_SLOT_ID_BASE, raw)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 7)?),
    );
    let apply_definition = ProgramDefinition::new(
        apply,
        Vec::new(),
        vec![owner],
        vec![shield_effect, amber_effect],
        Vec::new(),
    )
    .with_steps(vec![
        ProgramStep::Operation(RuleOperationTemplate::SetSlot {
            slot: counter,
            value: ValueExpr::Literal(RuleValue::Integer(0)),
        }),
        ProgramStep::Operation(RuleOperationTemplate::RemoveShield {
            selector: owner,
            effect: shield_effect,
        }),
        ProgramStep::Operation(RuleOperationTemplate::Shield {
            selector: owner,
            amount,
            effect: shield_effect,
        }),
        ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
            selector: owner,
            effect: amber_effect,
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            chance: RuleEffectChancePolicy::Guaranteed,
            base_chance: None,
            rng_purpose: None,
        }),
    ]);
    let advance_definition =
        ProgramDefinition::new(advance, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddSlot {
                    slot: counter,
                    value: ValueExpr::Literal(RuleValue::Integer(1)),
                },
            )]);
    let expire_definition = ProgramDefinition::new(
        expire,
        Vec::new(),
        vec![owner],
        vec![shield_effect],
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::RemoveShield {
            selector: owner,
            effect: shield_effect,
        },
    )]);
    let amber_runtime = timed_effect_runtime(2)?
        .with_damage_guard(starclock_combat::EffectDamageGuard::ShieldOverflowOnce);
    let mut rule = executable_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        vec![
            EffectDefinition::new(shield_effect, Vec::new(), Vec::new()),
            EffectDefinition::new(amber_effect, Vec::new(), Vec::new())
                .with_runtime_template(amber_runtime),
        ],
        vec![apply_definition, advance_definition, expire_definition],
        vec![
            StateSlotDef::new(
                counter,
                RuleValueKind::Integer,
                BattleRuleScope::Battle,
                RuleValue::Integer(0),
            )
            .with_bounds(RuleValue::Integer(0), RuleValue::Integer(2)),
        ],
        vec![
            trigger(
                id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::ActionResolved,
                OnceScope::Action,
                EventFilter {
                    source: Some(resonance_source()),
                    ..EventFilter::default()
                },
                ConditionExpr::Literal(true),
                apply,
            ),
            trigger(
                id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                integer_slot_equals(counter, 0),
                advance,
            ),
            trigger(
                id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
                RuleEventPoint::TurnEnded,
                OnceScope::Turn,
                EventFilter {
                    owner_selector: Some(owner),
                    ..EventFilter::default()
                },
                integer_slot_equals(counter, 1),
                expire,
            ),
        ],
    );
    rule.attachment = RuleAttachment::EveryPlayer;
    Ok(rule)
}

fn energy_rule(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let start = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let shield = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let resource = RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into());
    let program = |id, amount| {
        ProgramDefinition::new(id, Vec::new(), vec![owner], Vec::new(), Vec::new()).with_steps(
            vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: resource.clone(),
                    update: ResourceUpdateKind::Gain,
                    amount: scalar(amount),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )],
        )
    };
    let positive_shield = ConditionExpr::Compare {
        lhs: Box::new(ValueExpr::ReadEventProperty(
            EventValueProperty::ShieldChangeAmount,
        )),
        operator: Comparison::Greater,
        rhs: Box::new(scalar(0)),
    };
    let mut rule = executable_rule(
        binding,
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![
            program(
                start,
                parameter(parameters, 4)?
                    .checked_mul(100)
                    .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            ),
            program(
                shield,
                parameter(parameters, 5)?
                    .checked_mul(100)
                    .ok_or(BattleRuleLoweringError::InvalidParameter)?,
            ),
        ],
        Vec::new(),
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
                RuleEventPoint::ShieldChanged,
                OnceScope::Event,
                EventFilter::default(),
                positive_shield,
                shield,
            ),
        ],
    );
    rule.attachment = RuleAttachment::FirstPlayer;
    Ok(rule)
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

fn timed_effect_runtime(duration: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        Some(ValueExpr::Literal(RuleValue::Integer(i64::from(duration)))),
        DurationClock::OwnerTurnEnd,
        EffectTickPhase::TurnEnd,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn resonance_source() -> SourceDefinitionId {
    SourceDefinitionId::new(RESONANCE_ABILITY_ID.get()).expect("ability ID is non-zero")
}
