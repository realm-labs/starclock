use super::*;

const QUESTIONING_OF_PURPOSE: &str = "StageAbility_612251";
const BLIND_VISION: &str = "StageAbility_612252";
const TRAGIC_LECTURE: &str = "StageAbility_612253";
const SENSORY_LABYRINTH: &str = "StageAbility_612254";
const EMOTIONAL_DECLUTTERING: &str = "StageAbility_612255";

const CONTRIBUTION_RULE_ID_BASE: u32 = 0x7000_0000;
const LOCAL_MODIFIER_BASE: u32 = 0x7900_0000;
const LOCAL_GROUP_BASE: u32 = 0x7910_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    blessings: &BlessingContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    for key in [
        QUESTIONING_OF_PURPOSE,
        BLIND_VISION,
        TRAGIC_LECTURE,
        SENSORY_LABYRINTH,
        EMOTIONAL_DECLUTTERING,
    ] {
        let Some(binding) = level_binding(bindings, key) else {
            continue;
        };
        let parameters = selected_level_parameters(blessings, key)
            .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
        output.push(match key {
            QUESTIONING_OF_PURPOSE => questioning_of_purpose(binding, parameters)?,
            BLIND_VISION => enemy_persistent_modifier(
                binding,
                StatKind::EffectResistance,
                FormulaStage::Flat,
                FormulaPurpose::EffectChance,
                scalar(
                    parameter(parameters, 0)?
                        .checked_neg()
                        .ok_or(BattleRuleLoweringError::InvalidParameter)?,
                ),
                Vec::new(),
            )?,
            TRAGIC_LECTURE => enemy_persistent_modifier(
                binding,
                StatKind::Hp,
                FormulaStage::Vulnerability,
                FormulaPurpose::Dot,
                scalar(parameter(parameters, 0)?),
                vec![ModifierFilter::FormulaSubject(FormulaSubject::Target)],
            )?,
            SENSORY_LABYRINTH => enemy_persistent_modifier(
                binding,
                StatKind::DotDurationAddition,
                FormulaStage::Flat,
                FormulaPurpose::Dot,
                scalar(parameter(parameters, 0)?),
                Vec::new(),
            )?,
            EMOTIONAL_DECLUTTERING => emotional_decluttering(binding, parameters)?,
            _ => unreachable!("closed Nihility S03 binding set"),
        });
    }
    Ok(output)
}

fn questioning_of_purpose(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    persistent_modifier_rule(
        binding,
        StatKind::Hp,
        FormulaStage::DamageBoost,
        FormulaPurpose::Break,
        scalar(parameter(parameters, 0)?),
        Vec::new(),
    )
}

fn enemy_persistent_modifier(
    binding: &UniverseBattleRuleBinding,
    stat: StatKind,
    stage: FormulaStage,
    purpose: FormulaPurpose,
    value: ValueExpr,
    filters: Vec<ModifierFilter>,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let mut rule = persistent_modifier_rule(binding, stat, stage, purpose, value, filters)?;
    rule.attachment = RuleAttachment::EveryEnemy;
    Ok(rule)
}

fn emotional_decluttering(
    binding: &UniverseBattleRuleBinding,
    parameters: &[ExactParameter],
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let owner = id::<SelectorId>(OWNER_SELECTOR_ID_BASE, raw)?;
    let effect = id::<EffectDefinitionId>(EFFECT_ID_BASE, raw)?;
    let trigger_id = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let group = local::<ModifierStackingGroupId>(LOCAL_GROUP_BASE, raw, 0)?;
    let ratio = parameter(parameters, 0)?;
    let cap = whole(parameter(parameters, 1)?)?;
    let cap_value = ratio
        .checked_mul(i64::from(cap))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let stacks = ValueExpr::Convert {
        value: Box::new(ValueExpr::QueryEffectCategoryStacks {
            subject: StatQuerySubject::CurrentTarget,
            category: EffectCategory::Dot,
        }),
        target: RuleValueKind::Scalar,
        rounding: Rounding::NearestTiesEven,
    };
    let value = ValueExpr::Minimum(
        Box::new(multiply(stacks, scalar(ratio))),
        Box::new(scalar(cap_value)),
    );
    let mut modifiers = Vec::new();
    for (index, purpose) in damage_purposes().into_iter().enumerate() {
        modifiers.push(ModifierDefinition {
            id: local::<ModifierDefinitionId>(
                LOCAL_MODIFIER_BASE,
                raw,
                u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?,
            )?,
            stat: StatKind::Hp,
            stage: FormulaStage::Vulnerability,
            purpose,
            value: value.clone(),
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
    let modifier_ids = modifiers.iter().map(|modifier| modifier.id).collect();
    let runtime = EffectRuntimeTemplate::new(
        EffectCategory::Buff,
        DispelCategory::NonDispellable,
        1,
        None,
        DurationClock::Permanent,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], vec![effect], Vec::new())
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
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::EveryEnemy,
        modifier_groups: vec![ModifierStackingGroup {
            id: group,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        }]
        .into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)]
            .into_boxed_slice(),
        effects: vec![
            EffectDefinition::new(effect, Vec::new(), modifier_ids).with_runtime_template(runtime),
        ]
        .into_boxed_slice(),
        programs: vec![program_definition].into_boxed_slice(),
        definition: RuleDefinition::new(binding.rule(), vec![program], vec![owner]).with_runtime(
            BattleRuleDefinition::new(
                binding.source().clone(),
                Vec::new(),
                vec![trigger(
                    trigger_id,
                    RuleEventPoint::BattleStarted,
                    OnceScope::Battle,
                    EventFilter::default(),
                    ConditionExpr::Literal(true),
                    program,
                )],
                None,
            ),
        ),
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
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
