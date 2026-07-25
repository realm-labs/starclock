use super::*;
use starclock_combat::catalog::action::{HitOperationDefinition, ScalingDamageDefinition};

const FREEZE_ENERGY: &str = "StageAbility_612156";
const FREEZE_SHIELD: &str = "StageAbility_612157";
pub(super) const RESONANCE: &str = "StageAbility_612120";
const TOTAL_RECALL: &str = "StageAbility_612121";
const RICH_EXPERIENCE: &str = "StageAbility_612122";
const FIRST_LOVE: &str = "StageAbility_612123";

const BEFORE_PROGRAM_ID_BASE: u32 = 0x77e0_0000;
const AFTER_PROGRAM_ID_BASE: u32 = 0x77f0_0000;
const FREEZE_EFFECT_ID_BASE: u32 = 0x7800_0000;
const TOTAL_EFFECT_ID_BASE: u32 = 0x7810_0000;
const EONIAN_EFFECT_ID_BASE: u32 = 0x7820_0000;
const TOTAL_MODIFIER_ID_BASE: u32 = 0x7830_0000;
const EONIAN_MODIFIER_ID_BASE: u32 = 0x7840_0000;
const TOTAL_GROUP_ID_BASE: u32 = 0x7850_0000;
const EONIAN_GROUP_ID_BASE: u32 = 0x7860_0000;

pub(super) fn lower_rules(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [FREEZE_ENERGY, FREEZE_SHIELD] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            FREEZE_ENERGY => freeze_energy(binding, parameters)?,
            FREEZE_SHIELD => freeze_shield(binding, parameters)?,
            _ => unreachable!("closed Remembrance S04 Blessing set"),
        });
    }
    if let Some(binding) = resonance_binding(bindings, FIRST_LOVE) {
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
    damage_ratio: i64,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let definition = catalog
        .resonances()
        .iter()
        .find(|definition| definition.stable_key() == binding.source_record_key())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    let parameters = definition.parameters();
    let raw = binding.rule().get();
    let before = id::<ProgramId>(BEFORE_PROGRAM_ID_BASE, raw)?;
    let after = id::<ProgramId>(AFTER_PROGRAM_ID_BASE, raw)?;
    let freeze = id::<EffectDefinitionId>(FREEZE_EFFECT_ID_BASE, raw)?;
    let total = id::<EffectDefinitionId>(TOTAL_EFFECT_ID_BASE, raw)?;
    let eonian = id::<EffectDefinitionId>(EONIAN_EFFECT_ID_BASE, raw)?;
    let total_modifier = id::<ModifierDefinitionId>(TOTAL_MODIFIER_ID_BASE, raw)?;
    let eonian_modifier = id::<ModifierDefinitionId>(EONIAN_MODIFIER_ID_BASE, raw)?;
    let total_group = id::<ModifierStackingGroupId>(TOTAL_GROUP_ID_BASE, raw)?;
    let eonian_group = id::<ModifierStackingGroupId>(EONIAN_GROUP_ID_BASE, raw)?;

    let mut before_steps = Vec::new();
    let mut before_effects = Vec::new();
    let total_enabled = resonance_binding(bindings, TOTAL_RECALL).is_some();
    let eonian_enabled = resonance_binding(bindings, RICH_EXPERIENCE).is_some();
    if total_enabled {
        before_effects.push(total);
        before_steps.push(apply_effect(
            RESONANCE_ENEMY_SELECTOR_ID,
            total,
            parameter(parameters, 0)?
                .checked_add(900_000)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?,
        ));
    }
    if eonian_enabled {
        before_effects.push(eonian);
        before_steps.push(apply_effect(RESONANCE_ENEMY_SELECTOR_ID, eonian, 1_500_000));
    }
    let main = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID],
        Vec::new(),
        Vec::new(),
    );
    let before_program = ProgramDefinition::new(
        before,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID],
        before_effects,
        Vec::new(),
    )
    .with_steps(before_steps);
    let after_program = ProgramDefinition::new(
        after,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID],
        vec![freeze],
        Vec::new(),
    )
    .with_steps(vec![apply_effect(
        RESONANCE_ENEMY_SELECTOR_ID,
        freeze,
        parameter(parameters, 1)?,
    )]);

    let ratio = Ratio::from_scaled(parameter(parameters, 0)?)
        .checked_mul(
            Ratio::ONE
                .checked_add(Ratio::from_scaled(damage_ratio))
                .map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
            Rounding::NearestTiesEven,
        )
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
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
    .with_hits(vec![
        ActionHitDefinition::new(vec![HitOperationDefinition::ScalingDamage(
            ScalingDamageDefinition::new(
                StatKind::Hp,
                ratio,
                DamageClass::Additional,
                CombatElement::Ice,
            )
            .map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
        )])
        .with_profile(
            starclock_combat::catalog::action::HitTargetGroup::Selected,
            Ratio::ONE,
            Ratio::ONE,
            HitCritPolicy::Never,
        ),
    ])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let selector = SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    );
    let enemies =
        SelectorDefinition::new(RESONANCE_ENEMY_SELECTOR_ID).with_rule_units(all_enemy_selector()?);
    let ability = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(action)
    .with_programs(vec![
        AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, before)
            .expect("non-zero sequence"),
        AbilityProgramBinding::new(2, AbilityProgramTiming::AfterHits, after)
            .expect("non-zero sequence"),
    ]);

    let mut groups = Vec::new();
    let mut modifiers = Vec::new();
    let mut effects = vec![
        EffectDefinition::new(freeze, Vec::new(), Vec::new())
            .with_runtime(freeze_runtime(whole(parameter(parameters, 2)?)?)?),
    ];
    if total_enabled {
        groups.push(unique_group(total_group));
        modifiers.push(stat_modifier(
            total_modifier,
            total_group,
            StatKind::FreezeResistance,
            -1_000_000,
        ));
        effects.push(
            EffectDefinition::new(total, Vec::new(), vec![total_modifier])
                .with_runtime(timed_debuff(whole(parameter(parameters, 3)?)?)?),
        );
    }
    if eonian_enabled {
        groups.push(unique_group(eonian_group));
        modifiers.push(stat_modifier(
            eonian_modifier,
            eonian_group,
            StatKind::DebuffDurationMultiplier,
            1_000_000,
        ));
        effects.push(
            EffectDefinition::new(eonian, Vec::new(), vec![eonian_modifier])
                .with_runtime(timed_debuff(1)?),
        );
    }
    groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    let mut programs = vec![main, before_program, after_program];
    programs.sort_unstable_by_key(ProgramDefinition::id);
    Ok(ExecutableResonance {
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: vec![selector, enemies].into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        ability,
        initial_energy: initial_energy.min(100),
        maximum_energy: 100,
    })
}

fn freeze_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner, enemies],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::ModifyResource {
            selector: owner,
            resource: RuleResourceKind::Energy,
            update: ResourceUpdateKind::Gain,
            amount: scalar(parameter(parameters, 0)?),
            scales_with_regeneration: false,
            rounding: Rounding::Floor,
        },
    )]);
    let mut rule = executable_rule(
        binding,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        Vec::new(),
        vec![program_definition],
        Vec::new(),
        freeze_applied_triggers(raw, owner, enemies, program, OnceScope::Action)?,
    );
    rule.attachment = RuleAttachment::EveryPlayer;
    Ok(rule)
}

fn freeze_shield(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let apply = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let amount = multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Owner,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        },
        scalar(parameter(parameters, 0)?),
    );
    preservation_s03::timed_shield_rule(
        binding,
        amount,
        whole(parameter(parameters, 1)?)?,
        RuleEventPoint::EffectApplied,
        EventFilter {
            applier_selector: Some(owner),
            target_selector: Some(enemies),
            effect_category: Some(EffectCategory::Control),
            effect_specific_resistance: Some(StatKind::FreezeResistance),
            ..EventFilter::default()
        },
        ConditionExpr::Literal(true),
        vec![SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?)],
        Vec::new(),
        Vec::new(),
        Vec::new(),
        vec![trigger(
            id::<TriggerId>(FOURTH_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::ToughnessChanged,
            OnceScope::Event,
            EventFilter {
                applier_selector: Some(owner),
                target_selector: Some(enemies),
                element: Some(CombatElement::Ice),
                toughness_kind: Some(RuleToughnessEventKind::BaseEffectApplied),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            apply,
        )],
    )
}

fn resonance_energy(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let start = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let frozen = id::<ProgramId>(AUX_PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let enemies = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let start_amount = parameter(parameters, 4)?
        .checked_mul(100)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let frozen_amount = parameter(parameters, 5)?
        .checked_mul(100)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let program = |id, amount: i64, selectors: Vec<SelectorId>| {
        ProgramDefinition::new(id, Vec::new(), selectors, Vec::new(), Vec::new()).with_steps(vec![
            ProgramStep::Operation(RuleOperationTemplate::ModifyResource {
                selector: owner,
                resource: RuleResourceKind::Team(RESONANCE_RESOURCE_KEY.into()),
                update: ResourceUpdateKind::Gain,
                amount: scalar(amount),
                scales_with_regeneration: false,
                rounding: Rounding::Floor,
            }),
        ])
    };
    let mut triggers = vec![trigger(
        id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
        RuleEventPoint::BattleStarted,
        OnceScope::Battle,
        EventFilter::default(),
        ConditionExpr::Literal(true),
        start,
    )];
    triggers.extend(freeze_applied_triggers(
        raw,
        owner,
        enemies,
        frozen,
        OnceScope::Event,
    )?);
    let mut rule = executable_rule(
        binding,
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?),
        ],
        Vec::new(),
        vec![
            program(start, start_amount, vec![owner]),
            program(frozen, frozen_amount, vec![owner, enemies]),
        ],
        Vec::new(),
        triggers,
    );
    rule.attachment = RuleAttachment::FirstPlayer;
    Ok(rule)
}

fn freeze_applied_triggers(
    raw: u32,
    owner: SelectorId,
    enemies: SelectorId,
    program: ProgramId,
    once_scope: OnceScope,
) -> Result<Vec<TriggerDef>, BattleRuleLoweringError> {
    Ok(vec![
        trigger(
            id::<TriggerId>(SECOND_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::EffectApplied,
            once_scope,
            EventFilter {
                applier_selector: Some(owner),
                target_selector: Some(enemies),
                effect_category: Some(EffectCategory::Control),
                effect_specific_resistance: Some(StatKind::FreezeResistance),
                has_action: Some(true),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        ),
        trigger(
            id::<TriggerId>(THIRD_TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::ToughnessChanged,
            once_scope,
            EventFilter {
                applier_selector: Some(owner),
                target_selector: Some(enemies),
                element: Some(CombatElement::Ice),
                toughness_kind: Some(RuleToughnessEventKind::BaseEffectApplied),
                has_action: Some(true),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        ),
    ])
}

fn apply_effect(selector: SelectorId, effect: EffectDefinitionId, chance: i64) -> ProgramStep {
    ProgramStep::Operation(RuleOperationTemplate::ApplyEffect {
        selector,
        effect,
        stacks: ValueExpr::Literal(RuleValue::Integer(1)),
        chance: RuleEffectChancePolicy::Resistible,
        base_chance: Some(scalar(chance)),
        rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
    })
}

fn timed_debuff(duration: u16) -> Result<EffectRuntimeDefinition, BattleRuleLoweringError> {
    EffectRuntimeDefinition::new(
        EffectCategory::Debuff,
        DispelCategory::DispellableDebuff,
        1,
        Some(duration),
        DurationClock::TargetTurnStart,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn freeze_runtime(duration: u16) -> Result<EffectRuntimeDefinition, BattleRuleLoweringError> {
    EffectRuntimeDefinition::new(
        EffectCategory::Control,
        DispelCategory::CleanseableControl,
        1,
        Some(duration),
        DurationClock::TargetTurnStart,
        EffectTickPhase::None,
        EffectStackPolicy::Refresh,
    )
    .and_then(|runtime| {
        runtime.with_control(vec![
            ControlledAction::NormalAction,
            ControlledAction::Ultimate,
            ControlledAction::FollowUp,
            ControlledAction::Counter,
            ControlledAction::SummonAction,
        ])
    })
    .map(|runtime| runtime.with_specific_resistance(StatKind::FreezeResistance))
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn stat_modifier(
    id: ModifierDefinitionId,
    group: ModifierStackingGroupId,
    stat: StatKind,
    value: i64,
) -> ModifierDefinition {
    let resistance = stat == StatKind::FreezeResistance;
    ModifierDefinition {
        id,
        stat,
        stage: FormulaStage::Flat,
        purpose: FormulaPurpose::Stat,
        value: scalar(value),
        stacking_group: group,
        priority: 0,
        floor: resistance.then_some(starclock_combat::Scalar::from_scaled(-1_000_000)),
        cap: resistance.then_some(starclock_combat::Scalar::ONE),
        cap_stage: FormulaStage::Flat,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    }
}

fn unique_group(id: ModifierStackingGroupId) -> ModifierStackingGroup {
    ModifierStackingGroup {
        id,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
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
    let integral = value / 1_000_000;
    u16::try_from(integral)
        .ok()
        .filter(|converted| *converted > 0 && i64::from(*converted) * 1_000_000 == value)
        .ok_or(BattleRuleLoweringError::InvalidParameter)
}
