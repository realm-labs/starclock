//! Executable combat rules for Goal 07 negative Curio partition M12-S01.

use starclock_combat::{
    ModifierDefinitionId, ModifierStackingGroupId, ProgramId, Rounding, SelectorId, TriggerId,
    catalog::action::AbilityTag,
    catalog::definition::{ProgramDefinition, SelectorDefinition},
    modifier::model::{
        FormulaPurpose, FormulaStage, ModifierDefinition, SnapshotPolicy, StatKind,
        StatQuerySubject,
    },
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority, ResourceUpdateKind,
        RuleEventPoint, RuleOperationTemplate, RuleResourceKind, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio::CurioStateKind,
    curio_activity::negative::FISSION_EXTRA_COPY_KEY,
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, id, multiply, owner_selector,
    parameter, propagation_s01, scalar,
};

const ERROR_ENERGY: &str = "45";
const ERROR_HP: &str = "47";
const ERROR_DAMAGE_TAKEN: &str = "49";
const FISSION: &str = "78";
const PROGRAM_BASE: u32 = 0x77b1_0000;
const SELECTOR_BASE: u32 = 0x77b2_0000;
const TRIGGER_BASE: u32 = 0x77b3_0000;
const MODIFIER_BASE: u32 = 0x77b4_0000;
const GROUP_BASE: u32 = 0x77b5_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some((binding, contribution)) = state_binding(bindings, curios, ERROR_ENERGY)? {
        output.push(energy_code(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, ERROR_HP)? {
        output.push(hp_code(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, ERROR_DAMAGE_TAKEN)?
        && contribution.state().kind() == CurioStateKind::Fixed
    {
        output.push(fixed_damage_reduction(binding, contribution)?);
    }
    if let Some((binding, contribution)) = state_binding(bindings, curios, FISSION)? {
        output.push(fission_attack_penalty(binding, contribution, curios)?);
    }
    Ok(output)
}

fn energy_code(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(SELECTOR_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_BASE, raw)?;
    let amount = match contribution.state().kind() {
        CurioStateKind::Repairing => scalar(0),
        CurioStateKind::Fixed => ValueExpr::QueryMaximumEnergy(StatQuerySubject::Owner),
        CurioStateKind::Active => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::ModifyResource {
                    selector: owner,
                    resource: RuleResourceKind::Energy,
                    update: ResourceUpdateKind::Set,
                    amount,
                    scales_with_regeneration: false,
                    rounding: Rounding::Floor,
                },
            )]);
    finish_event_rule(
        binding,
        owner,
        program_definition,
        TriggerDef {
            id: trigger,
            event: RuleEventPoint::WeaknessBroken.kind(),
            event_point: RuleEventPoint::WeaknessBroken,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(owner),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Event,
            priority: ReactionPriority::new(0),
            program,
        },
    )
}

fn hp_code(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let owner = id::<SelectorId>(SELECTOR_BASE, raw)?;
    let program = id::<ProgramId>(PROGRAM_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_BASE, raw)?;
    let ratio = parameter(contribution.state().parameters(), 0)?;
    let amount = multiply(
        ValueExpr::QueryHp {
            subject: StatQuerySubject::Owner,
        },
        scalar(ratio),
    );
    let operation = match contribution.state().kind() {
        CurioStateKind::Repairing => RuleOperationTemplate::ConsumeHp {
            selector: owner,
            amount,
            floor: scalar(1_000_000),
        },
        CurioStateKind::Fixed => RuleOperationTemplate::Heal {
            selector: owner,
            amount,
            apply_formula_modifiers: false,
        },
        CurioStateKind::Active => return Err(BattleRuleLoweringError::InvalidParameter),
    };
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![owner], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(operation)]);
    finish_event_rule(
        binding,
        owner,
        program_definition,
        TriggerDef {
            id: trigger,
            event: RuleEventPoint::ActionResolved.kind(),
            event_point: RuleEventPoint::ActionResolved,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter {
                actor_selector: Some(owner),
                ability_tag: Some(AbilityTag::Ultimate),
                ..EventFilter::default()
            },
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Action,
            priority: ReactionPriority::new(0),
            program,
        },
    )
}

fn finish_event_rule(
    binding: &UniverseBattleRuleBinding,
    owner: SelectorId,
    program: ProgramDefinition,
    trigger: TriggerDef,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    Ok(propagation_s01::finish_rule(
        binding,
        RuleAttachment::EveryPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(owner).with_rule_units(owner_selector()?)],
        Vec::new(),
        vec![program],
        vec![trigger],
        Vec::new(),
    ))
}

fn fixed_damage_reduction(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let ratio = parameter(contribution.state().parameters(), 2)?;
    let raw = binding.rule().get();
    let modifiers = damage_purpose_modifiers(raw, FormulaStage::Mitigation, scalar(ratio))?;
    super::curio_s01::permanent_team_modifiers(binding, 7, modifiers)
}

fn fission_attack_penalty(
    binding: &UniverseBattleRuleBinding,
    contribution: &CurioContribution,
    curios: &CurioContributionSet,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let copies = 1_i64
        .checked_add(curios.runtime_value(FISSION_EXTRA_COPY_KEY).unwrap_or(0))
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let penalty = parameter(contribution.state().parameters(), 0)?
        .checked_mul(copies)
        .and_then(i64::checked_neg)
        .ok_or(BattleRuleLoweringError::InvalidParameter)?;
    let raw = binding.rule().get();
    let modifier = ModifierDefinition {
        id: id::<ModifierDefinitionId>(MODIFIER_BASE, raw)?,
        stat: StatKind::Atk,
        stage: FormulaStage::PercentOfBase,
        purpose: FormulaPurpose::Stat,
        value: scalar(penalty),
        stacking_group: id::<ModifierStackingGroupId>(GROUP_BASE, raw)?,
        priority: 0,
        floor: None,
        cap: None,
        cap_stage: FormulaStage::PercentOfBase,
        snapshot: SnapshotPolicy::Dynamic,
        source_stack_slot: None,
        filters: Box::new([]),
    };
    super::curio_s01::permanent_team_modifiers(binding, 8, vec![modifier])
}

fn damage_purpose_modifiers(
    raw: u32,
    stage: FormulaStage,
    value: ValueExpr,
) -> Result<Vec<ModifierDefinition>, BattleRuleLoweringError> {
    [
        FormulaPurpose::OrdinaryDamage,
        FormulaPurpose::Dot,
        FormulaPurpose::Break,
        FormulaPurpose::SuperBreak,
        FormulaPurpose::AdditionalDamage,
        FormulaPurpose::JointDamage,
        FormulaPurpose::ElationDamage,
        FormulaPurpose::TrueDamage,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, purpose)| {
        let offset =
            u32::try_from(index).map_err(|_| BattleRuleLoweringError::InvalidDefinition)?;
        Ok(ModifierDefinition {
            id: local::<ModifierDefinitionId>(MODIFIER_BASE, raw, offset)?,
            stat: StatKind::Hp,
            stage,
            purpose,
            value: value.clone(),
            stacking_group: local::<ModifierStackingGroupId>(GROUP_BASE, raw, offset)?,
            priority: 0,
            floor: None,
            cap: Some(starclock_combat::Scalar::ONE),
            cap_stage: stage,
            snapshot: SnapshotPolicy::Dynamic,
            source_stack_slot: None,
            filters: Box::new([]),
        })
    })
    .collect()
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

fn local<T: TryFrom<u32>>(base: u32, raw: u32, offset: u32) -> Result<T, BattleRuleLoweringError> {
    base.checked_add((raw & 0xffff).saturating_mul(16))
        .and_then(|value| value.checked_add(offset))
        .and_then(|value| T::try_from(value).ok())
        .ok_or(BattleRuleLoweringError::InvalidDefinition)
}
