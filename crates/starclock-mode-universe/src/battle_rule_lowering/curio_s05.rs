//! Executable combat rules for Goal 07 Curio partition M11-S05.

use starclock_combat::{
    ProgramId, SelectorId, TriggerId,
    catalog::definition::{ProgramDefinition, SelectorDefinition},
    rule::model::{
        ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority, RuleEventPoint,
        RuleOperationTemplate, TriggerDef, TriggerPhase,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio_runtime::{CurioContribution, CurioContributionSet},
};

use super::{
    BattleRuleLoweringError, ExecutableBattleRule, RuleAttachment, all_enemy_selector, curio_s01,
    id, parameter, propagation_s01, scalar,
};

const WICK_TRIMMER_EFFECT: &str = "58";
const PUNKLORDE_EFFECT: &str = "68";
const PROGRAM_BASE: u32 = 0x77c1_0000;
const SELECTOR_BASE: u32 = 0x77c2_0000;
const TRIGGER_BASE: u32 = 0x77c3_0000;

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let mut output = Vec::new();
    if let Some(contribution) = curios
        .entries()
        .iter()
        .find(|entry| entry.state().source_effect_id() == WICK_TRIMMER_EFFECT)
    {
        let count = i64::from(curios.destructibles_destroyed());
        if count != 0 {
            let binding = state_binding(bindings, contribution)?;
            let ratio = parameter(contribution.state().parameters(), 0)?
                .checked_mul(count)
                .ok_or(BattleRuleLoweringError::InvalidParameter)?;
            output.push(curio_s01::permanent_team_modifiers(
                binding,
                1,
                curio_s01::damage_modifiers(binding.rule().get(), scalar(ratio), &[])?,
            )?);
        }
    }
    if let Some(rule) = punklorde(bindings, curios)? {
        output.push(rule);
    }
    Ok(output)
}

fn punklorde(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Option<ExecutableBattleRule>, BattleRuleLoweringError> {
    let Some(contribution) = curios
        .entries()
        .iter()
        .find(|entry| entry.state().source_effect_id() == PUNKLORDE_EFFECT)
    else {
        return Ok(None);
    };
    let binding = state_binding(bindings, contribution)?;
    if parameter(contribution.state().parameters(), 0)? != 1_000_000 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    let count = whole(parameter(contribution.state().parameters(), 1)?)?;
    let duration = whole(parameter(contribution.state().parameters(), 2)?)?;
    let raw = binding.rule().get();
    let program = id::<ProgramId>(PROGRAM_BASE, raw)?;
    let enemies = id::<SelectorId>(SELECTOR_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_BASE, raw)?;
    let program_definition =
        ProgramDefinition::new(program, Vec::new(), vec![enemies], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::Operation(
                RuleOperationTemplate::AddWeaknessFromAlliedElements {
                    selector: enemies,
                    count,
                    duration_turns: duration,
                },
            )]);
    let trigger_definition = TriggerDef {
        id: trigger,
        event: RuleEventPoint::BattleStarted.kind(),
        event_point: RuleEventPoint::BattleStarted,
        phase: TriggerPhase::AfterEvent,
        filter: EventFilter::default(),
        condition: ConditionExpr::Literal(true),
        once_scope: OnceScope::Battle,
        priority: ReactionPriority::new(0),
        program,
    };
    Ok(Some(propagation_s01::finish_rule(
        binding,
        RuleAttachment::FirstPlayer,
        Vec::new(),
        Vec::new(),
        vec![SelectorDefinition::new(enemies).with_rule_units(all_enemy_selector()?)],
        Vec::new(),
        vec![program_definition],
        vec![trigger_definition],
        Vec::new(),
    )))
}

fn state_binding<'a>(
    bindings: &'a [UniverseBattleRuleBinding],
    contribution: &CurioContribution,
) -> Result<&'a UniverseBattleRuleBinding, BattleRuleLoweringError> {
    bindings
        .iter()
        .find(|binding| {
            binding.role() == UniverseBattleRuleRole::CurioState
                && binding.source_binding_key() == Some(contribution.state().source_effect_id())
        })
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)
}

fn whole(value: i64) -> Result<u8, BattleRuleLoweringError> {
    if value < 0 || value % 1_000_000 != 0 {
        return Err(BattleRuleLoweringError::InvalidParameter);
    }
    u8::try_from(value / 1_000_000).map_err(|_| BattleRuleLoweringError::InvalidParameter)
}
