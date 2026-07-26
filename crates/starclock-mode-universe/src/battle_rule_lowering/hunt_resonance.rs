use super::*;
use starclock_combat::catalog::{
    action::HitTargetGroup,
    selector::{
        RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
        RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
        RuleUnitSelector,
    },
};

const BOW_AND_ARROW: &str = "StageAbility_612422";
const PERFECT_AIM: &str = "StageAbility_612423";
const HIGHEST_ATTACK: SelectorId = SelectorId::new(0x7980_0001).expect("reserved selector ID");
const RESONANCE_ACTOR: SelectorId = SelectorId::new(0x7980_0002).expect("reserved selector ID");
const CRIT_BEFORE: ProgramId = ProgramId::new(0x7980_0003).expect("reserved program ID");
const CRIT_AFTER: ProgramId = ProgramId::new(0x7980_0004).expect("reserved program ID");
const CRIT_EFFECT: EffectDefinitionId =
    EffectDefinitionId::new(0x7980_0005).expect("reserved effect ID");
const CRIT_MODIFIER: ModifierDefinitionId =
    ModifierDefinitionId::new(0x7980_0006).expect("reserved modifier ID");
const CRIT_GROUP: ModifierStackingGroupId =
    ModifierStackingGroupId::new(0x7980_0007).expect("reserved group ID");

pub(super) fn lower(
    catalog: &UniverseCatalog,
    bindings: &[UniverseBattleRuleBinding],
    binding: &UniverseBattleRuleBinding,
    initial_energy: u16,
    damage_ratio: i64,
) -> Result<ExecutableResonance, BattleRuleLoweringError> {
    let resonance = catalog
        .resonances()
        .iter()
        .find(|definition| definition.stable_key() == binding.source_record_key())
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    let ratio = Ratio::from_scaled(parameter(resonance.parameters(), 1)?)
        .checked_mul(
            Ratio::ONE
                .checked_add(Ratio::from_scaled(damage_ratio))
                .map_err(|_| BattleRuleLoweringError::InvalidParameter)?,
            starclock_combat::Rounding::NearestTiesEven,
        )
        .map_err(|_| BattleRuleLoweringError::InvalidParameter)?;
    let bow_and_arrow = resonance_binding(bindings, BOW_AND_ARROW).is_some();
    let perfect_aim = resonance_binding(bindings, PERFECT_AIM).is_some();
    let maximum_energy = if perfect_aim { 200 } else { 100 };
    let highest_attack = RuleUnitSelector::new(
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
        purpose: FormulaPurpose::Stat,
    }));
    let actor = RuleUnitSelector::new(
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
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let amount = multiply(
        ValueExpr::SelectorSum {
            selector: HIGHEST_ATTACK,
            value: Box::new(ValueExpr::QueryStat {
                subject: StatQuerySubject::CurrentTarget,
                stat: StatKind::Atk,
                purpose: FormulaPurpose::AdditionalDamage,
            }),
        },
        scalar(ratio.scaled()),
    );
    let main = ProgramDefinition::new(
        RESONANCE_PROGRAM_ID,
        Vec::new(),
        vec![RESONANCE_ENEMY_SELECTOR_ID, HIGHEST_ATTACK],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::Damage {
            selector: RESONANCE_ENEMY_SELECTOR_ID,
            amount,
            class: DamageClass::Additional,
            element: CombatElement::Wind,
            can_crit: true,
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
        if bow_and_arrow {
            HitCritPolicy::GuaranteedBelowHpRatio(Ratio::from_scaled(parameter(
                resonance.parameters(),
                2,
            )?))
        } else {
            HitCritPolicy::PerTarget
        },
    )])
    .ok_or(BattleRuleLoweringError::InvalidDefinition)?;
    let selector = SelectorDefinition::new(RESONANCE_SELECTOR_ID).with_unit_targets(
        UnitTargetSelector::new(TargetRelation::Opposing, TargetPattern::All)
            .ok_or(BattleRuleLoweringError::InvalidDefinition)?,
    );
    let mut programs = vec![main];
    let mut effects = Vec::new();
    let mut modifiers = Vec::new();
    let mut groups = Vec::new();
    let mut bindings = vec![
        AbilityProgramBinding::new(2, AbilityProgramTiming::Hits, RESONANCE_PROGRAM_ID)
            .expect("non-zero sequence"),
    ];
    if bow_and_arrow {
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
        groups.push(ModifierStackingGroup {
            id: CRIT_GROUP,
            aggregation: ModifierAggregation::UniquePerSource,
            comparator: None,
        });
        modifiers.push(ModifierDefinition {
            id: CRIT_MODIFIER,
            stat: StatKind::CritDamage,
            stage: FormulaStage::Flat,
            purpose: FormulaPurpose::Stat,
            value: scalar(parameter(resonance.parameters(), 4)?),
            stacking_group: CRIT_GROUP,
            priority: 0,
            floor: None,
            cap: None,
            cap_stage: FormulaStage::Flat,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        });
        effects.push(
            EffectDefinition::new(CRIT_EFFECT, Vec::new(), vec![CRIT_MODIFIER])
                .with_runtime_template(runtime),
        );
        programs.extend([
            ProgramDefinition::new(
                CRIT_BEFORE,
                Vec::new(),
                vec![RESONANCE_ACTOR],
                vec![CRIT_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: RESONANCE_ACTOR,
                    effect: CRIT_EFFECT,
                    stacks: ValueExpr::Literal(RuleValue::Integer(1)),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]),
            ProgramDefinition::new(
                CRIT_AFTER,
                Vec::new(),
                vec![RESONANCE_ACTOR],
                vec![CRIT_EFFECT],
                Vec::new(),
            )
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::RemoveEffect {
                    selector: RESONANCE_ACTOR,
                    effect: CRIT_EFFECT,
                },
            )]),
        ]);
        bindings.extend([
            AbilityProgramBinding::new(1, AbilityProgramTiming::BeforeHits, CRIT_BEFORE)
                .expect("non-zero sequence"),
            AbilityProgramBinding::new(3, AbilityProgramTiming::AfterHits, CRIT_AFTER)
                .expect("non-zero sequence"),
        ]);
    }
    bindings.sort_unstable_by_key(|binding| binding.sequence());
    programs.sort_unstable_by_key(ProgramDefinition::id);
    let ability = AbilityDefinition::new(
        RESONANCE_ABILITY_ID,
        RESONANCE_PROGRAM_ID,
        RESONANCE_SELECTOR_ID,
        Vec::new(),
    )
    .with_action(action)
    .with_programs(bindings);
    Ok(ExecutableResonance {
        modifier_groups: groups.into_boxed_slice(),
        modifiers: modifiers.into_boxed_slice(),
        selectors: vec![
            selector,
            SelectorDefinition::new(RESONANCE_ENEMY_SELECTOR_ID)
                .with_rule_units(all_enemy_selector()?),
            SelectorDefinition::new(HIGHEST_ATTACK).with_rule_units(highest_attack),
            SelectorDefinition::new(RESONANCE_ACTOR).with_rule_units(actor),
        ]
        .into_boxed_slice(),
        effects: effects.into_boxed_slice(),
        programs: programs.into_boxed_slice(),
        ability,
        auxiliary_abilities: Box::new([]),
        countdowns: Box::new([]),
        initial_energy: initial_energy.min(maximum_energy),
        maximum_energy,
    })
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
