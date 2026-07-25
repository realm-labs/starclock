use super::*;
use starclock_combat::catalog::selector::{
    RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
    RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
    RuleUnitSelector,
};

const NIGHT_BEYOND_PYRE: &str = "StageAbility_612243";
const HELL_IS_OTHER_PEOPLE: &str = "StageAbility_612244";
const TWILIGHT_OF_EXISTENCE: &str = "StageAbility_612245";
const ALL_THINGS_ARE_POSSIBLE: &str = "StageAbility_612246";
const IGNOSTICISM: &str = "StageAbility_612250";

const BLEED: EffectDefinitionId = EffectDefinitionId::new(0x77f1_0001).expect("reserved effect ID");
const BURN: EffectDefinitionId = EffectDefinitionId::new(0x77f1_0002).expect("reserved effect ID");
const WIND_SHEAR: EffectDefinitionId =
    EffectDefinitionId::new(0x77f1_0003).expect("reserved effect ID");
const SHOCK: EffectDefinitionId = EffectDefinitionId::new(0x77f1_0004).expect("reserved effect ID");
const HELL_PROGRAM_BASE: u32 = 0x77f2_0000;
const HELL_TRIGGER_BASE: u32 = 0x77f2_0010;

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        NIGHT_BEYOND_PYRE,
        HELL_IS_OTHER_PEOPLE,
        TWILIGHT_OF_EXISTENCE,
        ALL_THINGS_ARE_POSSIBLE,
        IGNOSTICISM,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            NIGHT_BEYOND_PYRE => persistent_modifier_rule(
                binding,
                StatKind::ToughnessDamage,
                FormulaStage::Flat,
                FormulaPurpose::Break,
                scalar(parameter(parameters, 0)?),
                Vec::new(),
            )?,
            HELL_IS_OTHER_PEOPLE => hell_is_other_people(binding, parameters)?,
            TWILIGHT_OF_EXISTENCE => twilight_of_existence(binding, parameters)?,
            ALL_THINGS_ARE_POSSIBLE => all_things_are_possible(binding, parameters)?,
            IGNOSTICISM => ignosticism(catalog, blessings, binding, parameters)?,
            _ => unreachable!("closed Nihility S02 binding set"),
        });
    }
    Ok(output)
}

fn hell_is_other_people(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let enhanced = parameter(parameters, 0)? == 1_000_000;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, binding.rule().get())?;
    let targets = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, binding.rule().get())?;
    let target_selector = if enhanced {
        all_enemy_selector()?
    } else {
        adjacent_to_primary_selector()?
    };
    let elements = [
        CombatElement::Physical,
        CombatElement::Fire,
        CombatElement::Ice,
        CombatElement::Lightning,
        CombatElement::Wind,
        CombatElement::Quantum,
        CombatElement::Imaginary,
    ];
    let mut programs = Vec::new();
    let mut triggers = Vec::new();
    for (index, element) in elements.into_iter().enumerate() {
        let offset =
            u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
        let program = ProgramId::new(
            HELL_PROGRAM_BASE
                .checked_add(offset)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        )
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
        let trigger_id = TriggerId::new(
            HELL_TRIGGER_BASE
                .checked_add(offset)
                .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
        )
        .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
        programs.push(
            ProgramDefinition::new(
                program,
                Vec::new(),
                vec![owner, targets],
                Vec::new(),
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(RuleOperationTemplate::Break {
                selector: targets,
                element,
            })]),
        );
        triggers.push(trigger(
            trigger_id,
            RuleEventPoint::WeaknessBroken,
            OnceScope::Event,
            EventFilter {
                actor_selector: Some(owner),
                excluded_source: Some(binding.source().definition()),
                element: Some(element),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        ));
    }
    Ok(executable_rule(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(targets).with_rule_units(target_selector),
        ],
        Vec::new(),
        programs,
        triggers,
    ))
}

fn twilight_of_existence(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let attacker = id::<SelectorId>(TARGET_SELECTOR_ID_BASE, raw)?;
    let effects = [BLEED, BURN, WIND_SHEAR, SHOCK];
    let duration = whole(parameter(parameters, 1)?)?;
    let attack_ratio = parameter(parameters, 3)?;
    let hp_ratio = parameter(parameters, 4)?;
    let effect_definitions = vec![
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
                    scalar(hp_ratio),
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
            BURN,
            CombatElement::Fire,
            duration,
            applier_attack(attack_ratio),
            1,
        )?,
        dot_effect(
            WIND_SHEAR,
            CombatElement::Wind,
            duration,
            applier_attack(attack_ratio),
            whole(parameter(parameters, 2)?)?,
        )?,
        dot_effect(
            SHOCK,
            CombatElement::Lightning,
            duration,
            applier_attack(attack_ratio),
            1,
        )?,
    ];
    let mut steps = vec![ProgramStep::Operation(
        RuleOperationTemplate::ApplyRandomEffect {
            selector: owner,
            effects: effects.into(),
            stacks: ValueExpr::Literal(RuleValue::Integer(1)),
            choice_rng_purpose: DrawPurpose::BEHAVIOR_CHOICE,
            chance: RuleEffectChancePolicy::Resistible,
            base_chance: Some(scalar(parameter(parameters, 0)?)),
            chance_rng_purpose: Some(DrawPurpose::EFFECT_CHANCE),
        },
    )];
    if parameter(parameters, 5)? > 0 {
        steps.push(ProgramStep::Operation(RuleOperationTemplate::Cleanse {
            selector: attacker,
            maximum: 1,
            order: starclock_combat::EffectRemovalOrder::NewestFirst,
        }));
    }
    let program_definition = ProgramDefinition::new(
        program,
        Vec::new(),
        vec![owner, attacker],
        effects.into(),
        Vec::new(),
    )
    .with_steps(steps);
    Ok(executable_rule(
        binding,
        RuleAttachment::EveryEnemy,
        Vec::new(),
        Vec::new(),
        vec![
            SelectorDefinition::new(owner).with_rule_units(owner_selector()?),
            SelectorDefinition::new(attacker).with_rule_units(opposing_actor_selector()?),
        ],
        effect_definitions,
        vec![program_definition],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::TargetWithinAction,
            EventFilter {
                target_selector: Some(owner),
                ability_tag: Some(AbilityTag::Attack),
                ..EventFilter::default()
            },
            ConditionExpr::IsBroken(owner),
            program,
        )],
    ))
}

fn all_things_are_possible(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::DetonateDot {
                    selector: owner,
                    fraction: scalar(parameter(parameters, 0)?),
                    required_tag: None,
                    selection: starclock_combat::rule::model::RuleDotSelection::RandomOne(
                        DrawPurpose::BEHAVIOR_CHOICE,
                    ),
                },
            )]);
    Ok(executable_rule(
        binding,
        RuleAttachment::EveryEnemy,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![program_definition],
        vec![trigger(
            id::<TriggerId>(TRIGGER_ID_BASE, raw)?,
            RuleEventPoint::DamageApplied,
            OnceScope::TargetWithinAction,
            EventFilter {
                target_selector: Some(owner),
                ability_tag: Some(AbilityTag::Attack),
                ..EventFilter::default()
            },
            ConditionExpr::Literal(true),
            program,
        )],
    ))
}

fn ignosticism(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let cap = whole(parameter(parameters, 1)?)?;
    let count = i64::from(nihility_blessing_count(catalog, blessings)?.min(cap));
    let value = parameter(parameters, 0)?
        .checked_mul(count)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    persistent_modifier_rule(
        binding,
        StatKind::Hp,
        FormulaStage::DamageBoost,
        FormulaPurpose::Dot,
        scalar(value),
        Vec::new(),
    )
}

fn nihility_blessing_count(
    catalog: &UniverseCatalog,
    blessings: &BlessingContributionSet,
) -> Result<u16, BattleRuleLoweringError> {
    let path = catalog
        .paths()
        .iter()
        .find(|path| path.stable_key() == "universe.path.nihility")
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

fn applier_attack(ratio: i64) -> ValueExpr {
    multiply(
        ValueExpr::QueryStat {
            subject: StatQuerySubject::Applier,
            stat: StatKind::Atk,
            purpose: FormulaPurpose::Dot,
        },
        scalar(ratio),
    )
}

fn adjacent_to_primary_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::PrimaryTarget,
        RuleSelectorSide::Opposing,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::Formation,
        1,
        2,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::AdjacentToPrimary,
        None,
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn opposing_actor_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Actor,
        RuleSelectorSide::Opposing,
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

#[allow(clippy::too_many_arguments)]
fn executable_rule(
    binding: &UniverseBattleRuleBinding,
    attachment: RuleAttachment,
    mut modifier_groups: Vec<ModifierStackingGroup>,
    mut modifiers: Vec<ModifierDefinition>,
    mut selectors: Vec<SelectorDefinition>,
    mut effects: Vec<EffectDefinition>,
    mut programs: Vec<ProgramDefinition>,
    mut triggers: Vec<TriggerDef>,
) -> ExecutableBattleRule {
    modifier_groups.sort_unstable_by_key(|group| group.id);
    modifiers.sort_unstable_by_key(|modifier| modifier.id);
    selectors.sort_unstable_by_key(SelectorDefinition::id);
    effects.sort_unstable_by_key(EffectDefinition::id);
    programs.sort_unstable_by_key(ProgramDefinition::id);
    triggers.sort_unstable_by_key(|trigger| trigger.id);
    let selector_ids = selectors.iter().map(SelectorDefinition::id).collect();
    let program_ids = programs.iter().map(ProgramDefinition::id).collect();
    ExecutableBattleRule {
        attachment,
        modifier_groups: modifier_groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: selectors.into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), program_ids, selector_ids).with_runtime(
            BattleRuleDefinition::new(binding.source().clone(), Vec::new(), triggers, None),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    }
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    if value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}
