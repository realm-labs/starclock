//! Executable combat rules for Goal 07 Curio partition M11-S06.

use starclock_combat::{
    ProgramId, SelectorId, TriggerId,
    catalog::definition::{ProgramDefinition, RuleBundle, RuleDefinition, SelectorDefinition},
    modifier::model::{FormulaPurpose, StatKind, StatQuerySubject},
    rule::model::{
        BattleRuleDefinition, ConditionExpr, EventFilter, OnceScope, ProgramStep, ReactionPriority,
        RuleEventKind, RuleEventPoint, RuleOperationTemplate, RuleValue, TriggerDef, TriggerPhase,
        ValueExpr,
    },
};

use crate::{
    battle_contribution::{UniverseBattleRuleBinding, UniverseBattleRuleRole},
    curio_runtime::CurioContributionSet,
};

use super::{
    ALL_TARGET_SELECTOR_ID_BASE, BODY_PROGRAM_ID_BASE, BattleRuleLoweringError,
    CURRENT_TARGET_SELECTOR_ID_BASE, ExecutableBattleRule, PROGRAM_ID_BASE, RuleAttachment,
    TRIGGER_ID_BASE, all_enemy_selector, current_subject_selector, id, parameter,
};

const PARCHMENT_EFFECT: &str = "8";

pub(super) fn lower(
    bindings: &[UniverseBattleRuleBinding],
    curios: &CurioContributionSet,
) -> Result<Vec<ExecutableBattleRule>, BattleRuleLoweringError> {
    let Some(binding) = bindings.iter().find(|binding| {
        binding.role() == UniverseBattleRuleRole::CurioState
            && binding.source_binding_key() == Some(PARCHMENT_EFFECT)
    }) else {
        return Ok(Vec::new());
    };
    let contribution = curios
        .entries()
        .iter()
        .find(|entry| entry.state().source_effect_id() == PARCHMENT_EFFECT)
        .ok_or(BattleRuleLoweringError::SnapshotMismatch)?;
    let ratio = parameter(contribution.state().parameters(), 0)?;
    Ok(vec![entry_enemy_damage(binding, ratio)?])
}

fn entry_enemy_damage(
    binding: &UniverseBattleRuleBinding,
    ratio: i64,
) -> Result<ExecutableBattleRule, BattleRuleLoweringError> {
    let raw = binding.rule().get();
    let root = id::<ProgramId>(PROGRAM_ID_BASE, raw)?;
    let body = id::<ProgramId>(BODY_PROGRAM_ID_BASE, raw)?;
    let all_targets = id::<SelectorId>(ALL_TARGET_SELECTOR_ID_BASE, raw)?;
    let current_target = id::<SelectorId>(CURRENT_TARGET_SELECTOR_ID_BASE, raw)?;
    let trigger = id::<TriggerId>(TRIGGER_ID_BASE, raw)?;
    let selectors = vec![
        SelectorDefinition::new(all_targets).with_rule_units(all_enemy_selector()?),
        SelectorDefinition::new(current_target).with_rule_units(current_subject_selector()?),
    ];
    let root_definition =
        ProgramDefinition::new(root, Vec::new(), vec![all_targets], Vec::new(), Vec::new())
            .with_steps(vec![ProgramStep::ForEach {
                selector: all_targets,
                body,
                maximum: 16,
            }]);
    let amount = ValueExpr::Multiply {
        lhs: Box::new(ValueExpr::QueryStat {
            subject: StatQuerySubject::CurrentTarget,
            stat: StatKind::Hp,
            purpose: FormulaPurpose::Stat,
        }),
        rhs: Box::new(ValueExpr::Literal(RuleValue::Scalar(
            starclock_combat::Scalar::from_scaled(ratio),
        ))),
        rounding: starclock_combat::Rounding::NearestTiesEven,
    };
    let body_definition = ProgramDefinition::new(
        body,
        Vec::new(),
        vec![current_target],
        Vec::new(),
        Vec::new(),
    )
    .with_steps(vec![ProgramStep::Operation(
        RuleOperationTemplate::TrueDamage {
            selector: current_target,
            amount,
        },
    )]);
    let definition = RuleDefinition::new(
        binding.rule(),
        vec![root, body],
        vec![all_targets, current_target],
    )
    .with_runtime(BattleRuleDefinition::new(
        binding.source().clone(),
        Vec::new(),
        vec![TriggerDef {
            id: trigger,
            event: RuleEventKind::Battle,
            event_point: RuleEventPoint::BattleStarted,
            phase: TriggerPhase::AfterEvent,
            filter: EventFilter::default(),
            condition: ConditionExpr::Literal(true),
            once_scope: OnceScope::Battle,
            priority: ReactionPriority::new(-100),
            program: root,
        }],
        None,
    ));
    Ok(ExecutableBattleRule {
        attachment: RuleAttachment::FirstPlayer,
        modifier_groups: Box::new([]),
        modifiers: Box::new([]),
        selectors: selectors.into_boxed_slice(),
        effects: Box::new([]),
        programs: vec![root_definition, body_definition].into_boxed_slice(),
        definition,
        bundle: RuleBundle::new(binding.bundle(), vec![binding.rule()]),
    })
}
