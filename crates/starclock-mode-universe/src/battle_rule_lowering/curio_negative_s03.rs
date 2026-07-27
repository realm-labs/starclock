//! Executable combat rules for Goal 07 negative Curio partition M12-S03.

use starclock_combat::{
    DispelCategory, DurationClock, EffectCategory, EffectDefinitionId, EffectRuntimeTemplate,
    EffectStackPolicy, EffectTickPhase, ModifierDefinitionId, ModifierStackingGroupId, ProgramId,
    Rounding, SelectorId, TriggerId,
    catalog::{
        definition::{EffectDefinition, ProgramDefinition, SelectorDefinition},
        selector::{
            RuleEmptyPoolPolicy, RuleLifePredicate, RulePresencePredicate, RuleSelectorChoice,
            RuleSelectorOrdering, RuleSelectorOrigin, RuleSelectorReference, RuleSelectorSide,
            RuleUnitSelector,
        },
    },
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierAggregation, ModifierDefinition,
        ModifierStackingGroup, SnapshotPolicy, StatKind,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind,
        RuleEffectChancePolicy, RuleEventPoint, RuleOperationTemplate, RuleResourceKind, RuleValue,
        TriggerDef, TriggerPhase, ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, owner_selector, parameter,
    propagation_s01, scalar,
};

const BLACK_FOREST: &str = "66";
const MECHANICAL: &str = "71";
const PROGRAM_BASE: u32 = 0xf710_0000;
const SELECTOR_BASE: u32 = 0xf720_0000;
const EFFECT_BASE: u32 = 0xf730_0000;
const TRIGGER_BASE: u32 = 0xf740_0000;
const MODIFIER_BASE: u32 = 0xf750_0000;
const GROUP_BASE: u32 = 0xf760_0000;

// The released text publishes only "greatly increases." This project policy
// matches other major-aggro effects and remains explicitly replaceable.
const RELEASED_MAJOR_AGGRO_RATIO: i64 = 5_000_000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some((binding, contribution)) = state_binding(bindings, curios, BLACK_FOREST)? {
        output.push(black_forest(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, MECHANICAL)? {
        output.push(mechanical(binding, contribution)?);
    }
    Ok(output)
}

fn black_forest(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let target = local::<SelectorId>(SELECTOR_BASE, raw, 0)?;
    let effect = local::<EffectDefinitionId>(EFFECT_BASE, raw, 0)?;
    let program = local::<ProgramId>(PROGRAM_BASE, raw, 0)?;
    let trigger = local::<TriggerId>(TRIGGER_BASE, raw, 0)?;
    let modifier_id = local::<ModifierDefinitionId>(MODIFIER_BASE, raw, 0)?;
    let group_id = local::<ModifierStackingGroupId>(GROUP_BASE, raw, 0)?;
    let target_count = whole(parameter(contribution.state().parameters(), 0)?)?;
    let duration_turns = whole(parameter(contribution.state().parameters(), 1)?)?;
    if target_count != 1 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let modifier = ModifierDefinition {
        id: modifier_id,
        stat: StatKind::Aggro,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Aggro,
        value: scalar(RELEASED_MAJOR_AGGRO_RATIO),
        stacking_group: group_id,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    };
    let group = ModifierStackingGroup {
        id: group_id,
        aggregation: ModifierAggregation::UniquePerSource,
        comparator: None,
    };
    let effect_definition = EffectDefinition::new(effect, Vec::new(), vec![modifier_id])
        .with_runtime_template(timed_aggro_effect(duration_turns)?);
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![target], vec![effect], Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ApplyEffect {
                    selector: target,
                    effect,
                    stacks: integer(1),
                    chance: RuleEffectChancePolicy::Guaranteed,
                    base_chance: None,
                    rng_purpose: None,
                },
            )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        vec![group],
        vec![modifier],
        vec![SelectorDefinition::new(target).with_rule_units(random_ally_selector()?)],
        vec![effect_definition],
        vec![program_definition],
        vec![battle_started_trigger(trigger, program)],
        Vec::new(),
    ))
}

fn mechanical(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = local::<SelectorId>(SELECTOR_BASE, raw, 1)?;
    let program = local::<ProgramId>(PROGRAM_BASE, raw, 1)?;
    let trigger = local::<TriggerId>(TRIGGER_BASE, raw, 1)?;
    let amount = whole(parameter(contribution.state().parameters(), 0)?)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::SkillPoints,
                    update: ResourceUpdateKind::Spend,
                    amount: ValueExpr::Minimum(
                        Box::new(scalar(i64::from(amount) * 1_000_000)),
                        Box::new(ValueExpr::ReadResource {
                            selector: owner,
                            resource: RuleResourceKind::SkillPoints,
                        }),
                    ),
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![program_definition],
        vec![battle_started_trigger(trigger, program)],
        Vec::new(),
    ))
}

fn battle_started_trigger(id: TriggerId, program: ProgramId) -> TriggerDef {
    TriggerDef {
        id,
        event: RuleEventPoint::BattleStarted.kind(),
        event_point: RuleEventPoint::BattleStarted,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Battle,
        priority: ReactionPriority::new(0),
        program,
    }
}

fn random_ally_selector() -> Result<RuleUnitSelector, BattleRuleLoweringError> {
    RuleUnitSelector::new(
        RuleSelectorOrigin::Owner,
        RuleSelectorSide::Same,
        RuleLifePredicate::Alive,
        RulePresencePredicate::Present,
        RuleSelectorReference::CurrentState,
        RuleSelectorOrdering::StableId,
        0,
        1,
        RuleEmptyPoolPolicy::NoOp,
        RuleSelectorChoice::RngUniform,
        Some("behavior-choice".into()),
        false,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn timed_aggro_effect(turns: u16) -> Result<EffectRuntimeTemplate, BattleRuleLoweringError> {
    EffectRuntimeTemplate::new(
        EffectCategory::NeutralState,
        DispelCategory::NonDispellable,
        1,
        Some(integer(i64::from(turns))),
        DurationClock::TargetTurnEnd,
        EffectTickPhase::None,
        EffectStackPolicy::Replace,
    )
    .ok_or(BattleRuleLoweringError::InvalidDefinition)
}

fn state_binding<'a>(
    bindings: &'a [UniverseBattleRuleBinding],
    curios: &'a CurioContributionSet,
    effect: &str,
) -> Result<Option<(&'a UniverseBattleRuleBinding, &'a CurioContribution)>, BattleRuleLoweringError>
{
    let Some(contribution) = curios
        .entries()
        .iter()
        .find(|entry| entry.state().source_effect_id() == effect)
    else {
        return Ok(None);
    };
    let binding = bindings
        .iter()
        .find(|binding| {
            binding.role() == UniverseBattleRuleRole::CurioState
                && binding.source_binding_key() == Some(effect)
        })
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    Ok(Some((binding, contribution)))
}

fn whole(value: i64) -> Result<u16, BattleRuleLoweringError> {
    if value <= 0 || value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    u16::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}

fn integer(value: i64) -> ValueExpr {
    ValueExpr::Literal(RuleValue::Integer(value))
}

fn local<T: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<T, BattleRuleLoweringError> {
    base.checked_add((raw & 0xffff).saturating_mul(16))
        .and_then(|value| value.checked_add(offset))
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}
