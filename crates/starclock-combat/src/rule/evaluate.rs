//! Deterministic, budgeted and mutation-free Rule IR evaluation.
mod arithmetic;
mod event_property;
mod helpers;

use super::model::{
    ConditionExpr, ProgramStep, RuleEmission, RuleEvaluationInput, RuleOperationTemplate,
    RuleReplacementProposal, RuleResourceKind, RuleValue, RuleValueKind, ShieldObservation,
    TriggerDef, ValueExpr, once_key,
};

use crate::formula::model::CombatElement;
use crate::formula::toughness::EnemyRank;
use crate::modifier::model::{FormulaPurpose, StatKind, StatQuerySubject};
use crate::{
    AbilityId, EffectCategory, EffectDefinitionId, EventId, LifeState, PresenceState, ProgramId,
    RuleId, Scalar, SourceDefinitionId, UnitId,
};
use arithmetic::{Arithmetic, arithmetic, convert, extremum};
use event_property::event_property;
use helpers::{
    add_values, budget_error, compare_ordering, numeric_error, optional_unit,
    query_effect_category_stacks, query_subject, require_current_target_broken, selector_units,
    slot_value, type_error,
};
pub(crate) use helpers::{compare, compare_values, stat_query_error};
use std::collections::BTreeSet;

/// Applies the cheap indexed cause filter without inferring cause roles.
#[must_use]
pub fn matches_filter(filter: &super::model::EventFilter, input: RuleEvaluationInput<'_>) -> bool {
    helpers::matches_filter(filter, input)
}

/// Immutable program lookup used by the evaluator and static handler tests.
pub trait ProgramLookup {
    /// Returns the finite ordered steps for one validated program.
    fn program_steps(&self, id: ProgramId) -> Option<&[ProgramStep]>;
}

/// Read-only bridge used by the Rule IR `QueryStat` leaf.
pub trait StatQueryReader {
    fn query_stat(
        &self,
        origin: StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
        purpose: FormulaPurpose,
    ) -> Result<Scalar, RuleEvaluationError>;

    /// Reads the authored stat base without applying any modifier stage.
    fn query_base_stat(
        &self,
        origin: StatQuerySubject,
        subject: UnitId,
        stat: StatKind,
    ) -> Result<Scalar, RuleEvaluationError> {
        self.query_stat(origin, subject, stat, FormulaPurpose::Stat)
    }
}

/// Read-only bridge used by the Rule IR `AbilityParameter` leaf.
pub trait AbilityParameterReader {
    /// Returns one parameter selected by the exact resolved ability and semantic key.
    fn ability_parameter(&self, ability: AbilityId, key: &str) -> Option<RuleValue>;
}

/// Read-only bridge used by the Rule IR `ReadResource` leaf.
pub trait ResourceQueryReader {
    fn query_resource(&self, subject: UnitId, resource: &RuleResourceKind) -> Option<RuleValue>;
}

/// Read-only battlefield predicates used by authored contextual conditions.
pub trait BattleQueryReader {
    fn life_presence(&self, subject: UnitId) -> Option<(LifeState, PresenceState)>;
    fn has_effect(&self, subject: UnitId, effect: EffectDefinitionId) -> bool;
    fn is_frozen(&self, subject: UnitId) -> bool;
    fn has_weakness(&self, subject: UnitId, element: CombatElement) -> bool;
    fn is_broken(&self, subject: UnitId) -> bool;
    fn enemy_rank(&self, _subject: UnitId) -> Option<EnemyRank> {
        None
    }
    fn current_shield(&self, subject: UnitId) -> Option<Scalar>;
    fn current_hp(&self, _subject: UnitId) -> Option<Scalar> {
        None
    }
    fn maximum_energy(&self, _subject: UnitId) -> Option<Scalar> {
        None
    }
    fn effect_stacks(&self, subject: UnitId, effect: EffectDefinitionId) -> Option<i64>;
    fn effect_category_stacks(&self, _subject: UnitId, _category: EffectCategory) -> Option<i64> {
        Some(0)
    }
}

/// Stable evaluation failure category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuleEvaluationErrorKind {
    MissingProgram,
    MissingValue,
    TypeMismatch,
    Numeric,
    InvalidConversion,
    BudgetExceeded,
}

/// Deterministic Rule IR failure with numeric-only context.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleEvaluationError {
    kind: RuleEvaluationErrorKind,
    context: u32,
}

impl RuleEvaluationError {
    #[must_use]
    pub const fn kind(self) -> RuleEvaluationErrorKind {
        self.kind
    }
    #[must_use]
    pub const fn context(self) -> u32 {
        self.context
    }
}

impl core::fmt::Display for RuleEvaluationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "rule evaluation {:?} at {}",
            self.kind, self.context
        )
    }
}

impl std::error::Error for RuleEvaluationError {}

/// Per-trigger hard limits. Catalog policy supplies these fixed values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationBudget {
    pub maximum_steps: u32,
    pub maximum_emissions: u32,
    pub maximum_iterations: u32,
}

impl EvaluationBudget {
    /// Conservative generic defaults, never content-specific overrides.
    pub const STANDARD: Self = Self {
        maximum_steps: 1_024,
        maximum_emissions: 512,
        maximum_iterations: 256,
    };
}

#[derive(Clone, Copy, Debug)]
struct BudgetState {
    policy: EvaluationBudget,
    steps: u32,
    emissions: u32,
    iterations: u32,
}

impl BudgetState {
    const fn new(policy: EvaluationBudget) -> Self {
        Self {
            policy,
            steps: 0,
            emissions: 0,
            iterations: 0,
        }
    }
    fn step(&mut self) -> Result<(), RuleEvaluationError> {
        self.steps = self.steps.checked_add(1).ok_or_else(budget_error)?;
        if self.steps > self.policy.maximum_steps {
            return Err(budget_error());
        }
        Ok(())
    }
    fn emit(&mut self) -> Result<(), RuleEvaluationError> {
        self.emissions = self.emissions.checked_add(1).ok_or_else(budget_error)?;
        if self.emissions > self.policy.maximum_emissions {
            return Err(budget_error());
        }
        Ok(())
    }
    fn iterate(&mut self) -> Result<(), RuleEvaluationError> {
        self.iterations = self.iterations.checked_add(1).ok_or_else(budget_error)?;
        if self.iterations > self.policy.maximum_iterations {
            return Err(budget_error());
        }
        Ok(())
    }
}

/// Evaluates one validated program into resolver proposals without mutation.
pub fn evaluate_program(
    programs: &impl ProgramLookup,
    program: ProgramId,
    input: RuleEvaluationInput<'_>,
    budget: EvaluationBudget,
) -> Result<Vec<RuleEmission>, RuleEvaluationError> {
    let mut state = BudgetState::new(budget);
    let mut output = Vec::new();
    evaluate_program_inner(programs, program, input, None, &mut state, &mut output)?;
    Ok(output)
}

/// Evaluates one replacement program and rejects every mutating emission.
pub fn evaluate_replacement_program(
    programs: &impl ProgramLookup,
    program: ProgramId,
    input: RuleEvaluationInput<'_>,
    budget: EvaluationBudget,
) -> Result<Vec<RuleReplacementProposal>, RuleEvaluationError> {
    evaluate_program(programs, program, input, budget)?
        .into_iter()
        .map(|emission| match emission {
            RuleEmission::Replacement {
                code,
                value,
                current_target,
            } => Ok(RuleReplacementProposal {
                code,
                value,
                current_target,
            }),
            _ => Err(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::TypeMismatch,
                context: program.get(),
            }),
        })
        .collect()
}

/// Canonical authoritative once-key ledger owned by a future bound rule store.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TriggerLedger {
    keys: BTreeSet<super::model::OnceKey>,
}

impl TriggerLedger {
    /// Returns the number of committed once keys.
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns whether no once key has committed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
    pub(crate) fn canonical_keys(&self) -> impl ExactSizeIterator<Item = &super::model::OnceKey> {
        self.keys.iter()
    }

    pub(crate) fn reset_scope(&mut self, scope: super::model::OnceScope) -> usize {
        let before = self.keys.len();
        self.keys.retain(|key| key.scope != scope);
        before - self.keys.len()
    }

    pub(crate) fn reset_event(&mut self, event: EventId) -> usize {
        let before = self.keys.len();
        self.keys
            .retain(|key| key.scope != super::model::OnceScope::Event || key.first != event.get());
        before - self.keys.len()
    }

    /// Matches, evaluates, and only then commits the trigger's once key.
    pub fn evaluate(
        &mut self,
        programs: &impl ProgramLookup,
        trigger: &TriggerDef,
        input: RuleEvaluationInput<'_>,
        budget: EvaluationBudget,
        maximum_once_keys: usize,
    ) -> Result<Vec<RuleEmission>, RuleEvaluationError> {
        if input.event_kind != trigger.event
            || input.event_facts.point != Some(trigger.event_point)
            || !matches_filter(&trigger.filter, input)
            || !evaluate_condition(&trigger.condition, input, None)?
        {
            return Ok(Vec::new());
        }
        let Some(key) = once_key(trigger.id, trigger.once_scope, input.occurrence) else {
            return Ok(Vec::new());
        };
        if self.keys.contains(&key) {
            return Ok(Vec::new());
        }
        if self.keys.len() >= maximum_once_keys {
            return Err(budget_error());
        }
        let emissions = evaluate_program(programs, trigger.program, input, budget)?;
        self.keys.insert(key);
        Ok(emissions)
    }
}

fn evaluate_program_inner(
    programs: &impl ProgramLookup,
    program: ProgramId,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
    budget: &mut BudgetState,
    output: &mut Vec<RuleEmission>,
) -> Result<(), RuleEvaluationError> {
    let steps = programs.program_steps(program).ok_or(RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingProgram,
        context: program.get(),
    })?;
    for step in steps {
        budget.step()?;
        match step {
            ProgramStep::Operation(operation) => {
                output.push(evaluate_operation(operation, input, current_target)?);
                budget.emit()?;
            }
            ProgramStep::If {
                condition,
                then_program,
                else_program,
            } => {
                let selected = if evaluate_condition(condition, input, current_target)? {
                    Some(*then_program)
                } else {
                    *else_program
                };
                if let Some(selected) = selected {
                    evaluate_program_inner(
                        programs,
                        selected,
                        input,
                        current_target,
                        budget,
                        output,
                    )?;
                }
            }
            ProgramStep::ForEach {
                selector,
                body,
                maximum,
            } => {
                let units = selector_units(input, *selector).ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: selector.get(),
                })?;
                if units.len() > usize::from(*maximum) {
                    return Err(budget_error());
                }
                for unit in units {
                    budget.iterate()?;
                    evaluate_program_inner(programs, *body, input, Some(*unit), budget, output)?;
                }
            }
        }
    }
    Ok(())
}

fn evaluate_operation(
    operation: &RuleOperationTemplate,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<RuleEmission, RuleEvaluationError> {
    Ok(match operation {
        RuleOperationTemplate::SetSlot { slot, value } => RuleEmission::SetSlot {
            slot: *slot,
            value: evaluate_value(value, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::AddSlot { slot, value } => RuleEmission::AddSlot {
            slot: *slot,
            value: evaluate_value(value, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::Damage {
            selector,
            amount,
            class,
            element,
            can_crit,
            can_defeat,
        } => RuleEmission::Damage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            element: *element,
            can_crit: *can_crit,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::UnboostedDamage {
            selector,
            amount,
            class,
            element,
            can_defeat,
        } => RuleEmission::UnboostedDamage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            element: *element,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::UnboostedDamageFromEventElement {
            selector,
            amount,
            class,
            can_defeat,
        } => RuleEmission::UnboostedDamage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            element: input.event_facts.element.ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: 0x220,
            })?,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::DamageFromEventElement {
            selector,
            amount,
            class,
            can_crit,
            can_defeat,
        } => RuleEmission::Damage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            element: input.event_facts.element.ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: 0x21d,
            })?,
            can_crit: *can_crit,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::DamageFromActorBasicElement {
            selector,
            amount,
            class,
            can_crit,
            can_defeat,
        } => RuleEmission::DamageFromActorBasicElement {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            can_crit: *can_crit,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::UltimateDamageFromActorBasicElement {
            selector,
            amount,
            class,
            can_crit,
            can_defeat,
        } => RuleEmission::UltimateDamageFromActorBasicElement {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            can_crit: *can_crit,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::RandomRepeatedDamage {
            selector,
            amount,
            class,
            elements,
            minimum_hits,
            maximum_hits,
            count_rng_purpose,
            element_rng_purpose,
            exclude_event_element,
            can_crit,
            can_defeat,
        } => RuleEmission::RandomRepeatedDamage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            elements: elements.clone(),
            minimum_hits: *minimum_hits,
            maximum_hits: *maximum_hits,
            count_rng_purpose: *count_rng_purpose,
            element_rng_purpose: *element_rng_purpose,
            exclude_event_element: *exclude_event_element,
            can_crit: *can_crit,
            can_defeat: *can_defeat,
            current_target,
        },
        RuleOperationTemplate::TrueDamage { selector, amount } => RuleEmission::TrueDamage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::Heal {
            selector,
            amount,
            apply_formula_modifiers,
        } => RuleEmission::Heal {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            apply_formula_modifiers: *apply_formula_modifiers,
            current_target,
        },
        RuleOperationTemplate::Shield {
            selector,
            amount,
            effect,
        } => RuleEmission::Shield {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            effect: *effect,
            current_target,
        },
        RuleOperationTemplate::RemoveShield { selector, effect } => RuleEmission::RemoveShield {
            selector: *selector,
            effect: *effect,
            current_target,
        },
        RuleOperationTemplate::ConsumeHp {
            selector,
            amount,
            floor,
        } => RuleEmission::ConsumeHp {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            floor: evaluate_value(floor, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::ReduceToughness {
            selector,
            amount,
            element,
        } => RuleEmission::ReduceToughness {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            element: *element,
            current_target,
        },
        RuleOperationTemplate::Break { selector, element } => RuleEmission::Break {
            selector: *selector,
            element: *element,
            current_target,
        },
        RuleOperationTemplate::SuperBreak {
            selector,
            multiplier,
        } => RuleEmission::SuperBreak {
            selector: *selector,
            multiplier: evaluate_value(multiplier, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::AddWeakness {
            selector,
            element,
            duration_turns,
        } => RuleEmission::AddWeakness {
            selector: *selector,
            element: *element,
            duration_turns: duration_turns
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            current_target,
        },
        operation @ (RuleOperationTemplate::AddWeaknessFromAlliedElements { .. }
        | RuleOperationTemplate::RemoveWeakness { .. }) => {
            helpers::weakness_emission(operation, current_target)
        }
        RuleOperationTemplate::CreateToughnessLayer {
            selector,
            layer_key,
            maximum,
        } => RuleEmission::CreateToughnessLayer {
            selector: *selector,
            layer_key: layer_key.clone(),
            maximum: evaluate_value(maximum, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::RemoveToughnessLayer {
            selector,
            layer_key,
        } => RuleEmission::RemoveToughnessLayer {
            selector: *selector,
            layer_key: layer_key.clone(),
            current_target,
        },
        RuleOperationTemplate::ModifyResource {
            selector,
            resource,
            update,
            amount,
            scales_with_regeneration,
            rounding,
        } => RuleEmission::ModifyResource {
            selector: *selector,
            resource: resource.clone(),
            update: *update,
            amount: evaluate_value(amount, input, current_target)?,
            scales_with_regeneration: *scales_with_regeneration,
            rounding: *rounding,
            current_target,
        },
        RuleOperationTemplate::ApplyEffect {
            selector,
            effect,
            stacks,
            chance,
            base_chance,
            rng_purpose,
        } => RuleEmission::ApplyEffect {
            selector: *selector,
            effect: *effect,
            stacks: evaluate_value(stacks, input, current_target)?,
            chance: *chance,
            base_chance: base_chance
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            rng_purpose: *rng_purpose,
            current_target,
        },
        RuleOperationTemplate::ApplyRandomEffect {
            selector,
            effects,
            stacks,
            choice_rng_purpose,
            chance,
            base_chance,
            chance_rng_purpose,
        } => RuleEmission::ApplyRandomEffect {
            selector: *selector,
            effects: effects.clone(),
            stacks: evaluate_value(stacks, input, current_target)?,
            choice_rng_purpose: *choice_rng_purpose,
            chance: *chance,
            base_chance: base_chance
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            chance_rng_purpose: *chance_rng_purpose,
            current_target,
        },
        RuleOperationTemplate::RandomGroupedEffect {
            selector,
            effect,
            groups,
            applications_per_group,
            stacks,
            choice_rng_purpose,
            chance,
            base_chance,
            chance_rng_purpose,
        } => RuleEmission::RandomGroupedEffect {
            selector: *selector,
            effect: *effect,
            groups: evaluate_value(groups, input, current_target)?,
            applications_per_group: *applications_per_group,
            stacks: evaluate_value(stacks, input, current_target)?,
            choice_rng_purpose: *choice_rng_purpose,
            chance: *chance,
            base_chance: base_chance
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            chance_rng_purpose: *chance_rng_purpose,
            current_target,
        },
        RuleOperationTemplate::AdjustEffectStacks {
            selector,
            effect,
            delta,
        } => RuleEmission::AdjustEffectStacks {
            selector: *selector,
            effect: *effect,
            delta: evaluate_value(delta, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::RemoveEffect { selector, effect } => RuleEmission::RemoveEffect {
            selector: *selector,
            effect: *effect,
            current_target,
        },
        RuleOperationTemplate::Cleanse {
            selector,
            maximum,
            order,
        } => RuleEmission::Cleanse {
            selector: *selector,
            maximum: *maximum,
            order: *order,
            current_target,
        },
        RuleOperationTemplate::DetonateDot {
            selector,
            fraction,
            required_tag,
            selection,
        } => RuleEmission::DetonateDot {
            selector: *selector,
            fraction: evaluate_value(fraction, input, current_target)?,
            required_tag: *required_tag,
            selection: *selection,
            current_target,
        },
        RuleOperationTemplate::ModifyStateSlot {
            slot,
            update,
            value,
        } => RuleEmission::ModifyStateSlot {
            slot: *slot,
            update: *update,
            value: evaluate_value(value, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::AdvanceAction { selector, amount } => RuleEmission::AdvanceAction {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::DelayAction { selector, amount } => RuleEmission::DelayAction {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::QueueAction {
            actor_selector,
            target_selector,
            ability,
            priority,
            forced_use,
            boundary,
            owner,
            payment,
        } => RuleEmission::QueueAction {
            actor_selector: *actor_selector,
            target_selector: *target_selector,
            ability: *ability,
            priority: *priority,
            forced_use: *forced_use,
            boundary: *boundary,
            owner: *owner,
            payment: payment.clone(),
            current_target,
        },
        RuleOperationTemplate::GrantExtraTurn { actor_selector } => RuleEmission::GrantExtraTurn {
            actor_selector: *actor_selector,
            current_target,
        },
        RuleOperationTemplate::Summon {
            owner_selector,
            unit_definition,
        } => RuleEmission::Summon {
            owner_selector: *owner_selector,
            unit_definition: *unit_definition,
            current_target,
        },
        RuleOperationTemplate::Despawn { selector } => RuleEmission::Despawn {
            selector: *selector,
            current_target,
        },
        RuleOperationTemplate::Transform {
            selector,
            replacement_definition,
        } => RuleEmission::Transform {
            selector: *selector,
            replacement_definition: *replacement_definition,
            current_target,
        },
        RuleOperationTemplate::ReplaceAbility {
            selector,
            old_ability,
            new_ability,
        } => RuleEmission::ReplaceAbility {
            selector: *selector,
            old_ability: *old_ability,
            new_ability: *new_ability,
            current_target,
        },
        RuleOperationTemplate::ChangePresence { selector, presence } => {
            RuleEmission::ChangePresence {
                selector: *selector,
                presence: *presence,
                current_target,
            }
        }
        RuleOperationTemplate::CreateCountdown { code } => RuleEmission::CreateCountdown {
            code: *code,
            current_target,
        },
        RuleOperationTemplate::EmitRuleEvent { code, value } => RuleEmission::Informational {
            code: *code,
            value: value
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            current_target,
        },
        RuleOperationTemplate::ProposeReplacement { code, value } => RuleEmission::Replacement {
            code: *code,
            value: value
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            current_target,
        },
        RuleOperationTemplate::InvokeNative { handler, arguments } => RuleEmission::InvokeNative {
            handler: *handler,
            arguments: arguments
                .iter()
                .map(|value| evaluate_value(value, input, current_target))
                .collect::<Result<Vec<_>, _>>()?
                .into_boxed_slice(),
            current_target,
        },
    })
}

/// Stable definition-only total order for candidate triggers.
#[must_use]
pub fn trigger_definition_order(
    rule: RuleId,
    source: SourceDefinitionId,
    trigger: &super::model::TriggerDef,
) -> super::model::TriggerDefinitionOrder {
    helpers::trigger_definition_order(rule, source, trigger)
}

/// Evaluates a condition through the same read-only context used by programs.
pub fn evaluate_condition(
    condition: &ConditionExpr,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<bool, RuleEvaluationError> {
    Ok(match condition {
        ConditionExpr::Literal(value) => *value,
        ConditionExpr::Not(value) => !evaluate_condition(value, input, current_target)?,
        ConditionExpr::All(values) => {
            for value in values {
                if !evaluate_condition(value, input, current_target)? {
                    return Ok(false);
                }
            }
            true
        }
        ConditionExpr::Any(values) => {
            for value in values {
                if evaluate_condition(value, input, current_target)? {
                    return Ok(true);
                }
            }
            false
        }
        ConditionExpr::Compare { lhs, operator, rhs } => compare(
            &evaluate_value(lhs, input, current_target)?,
            *operator,
            &evaluate_value(rhs, input, current_target)?,
        )?,
        ConditionExpr::EventKind(kind) => input.event_kind == *kind,
        ConditionExpr::SourceTag(tag) => input.source_tags.binary_search(tag).is_ok(),
        ConditionExpr::SelectorCardinality {
            selector,
            operator,
            count,
        } => compare_ordering(
            selector_units(input, *selector)
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: selector.get(),
                })?
                .len()
                .cmp(&usize::from(*count)),
            *operator,
        ),
        ConditionExpr::LifePresence {
            selector,
            life,
            presence,
        } => selector_units(input, *selector)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?
            .iter()
            .copied()
            .all(|unit| {
                input
                    .battle_query_reader
                    .and_then(|reader| reader.life_presence(unit))
                    .is_some_and(|(actual_life, actual_presence)| {
                        life.is_none_or(|expected| expected == actual_life)
                            && presence.is_none_or(|expected| expected == actual_presence)
                    })
            }),
        ConditionExpr::EffectExists { selector, effect } => selector_units(input, *selector)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?
            .iter()
            .copied()
            .all(|unit| {
                input
                    .battle_query_reader
                    .is_some_and(|reader| reader.has_effect(unit, *effect))
            }),
        ConditionExpr::IsFrozen(selector) => selector_units(input, *selector)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?
            .iter()
            .copied()
            .all(|unit| {
                input
                    .battle_query_reader
                    .is_some_and(|reader| reader.is_frozen(unit))
            }),
        ConditionExpr::HasWeakness { selector, element } => selector_units(input, *selector)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?
            .iter()
            .copied()
            .all(|unit| {
                input
                    .battle_query_reader
                    .is_some_and(|reader| reader.has_weakness(unit, *element))
            }),
        ConditionExpr::IsBroken(selector) => selector_units(input, *selector)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?
            .iter()
            .copied()
            .all(|unit| {
                input
                    .battle_query_reader
                    .is_some_and(|reader| reader.is_broken(unit))
            }),
        ConditionExpr::CurrentTargetIsBroken => {
            require_current_target_broken(input, current_target)?
        }
        ConditionExpr::EnemyRank(selector, rank) => selector_units(input, *selector)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?
            .iter()
            .copied()
            .all(|unit| {
                input
                    .battle_query_reader
                    .and_then(|reader| reader.enemy_rank(unit))
                    == Some(*rank)
            }),
    })
}

pub fn evaluate_value(
    expression: &ValueExpr,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<RuleValue, RuleEvaluationError> {
    match expression {
        ValueExpr::Literal(value) => Ok(value.clone()),
        ValueExpr::Slot(slot) => slot_value(input, *slot)
            .cloned()
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: slot.get(),
            }),
        ValueExpr::AbilityParameter { key, .. } => input
            .occurrence
            .ability
            .and_then(|ability| {
                input
                    .ability_parameter_reader
                    .and_then(|reader| reader.ability_parameter(ability, key))
            })
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: 0x203,
            }),
        ValueExpr::ReadResource { selector, resource } => {
            let units = selector_units(input, *selector).ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?;
            let [unit] = units else {
                return Err(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: selector.get(),
                });
            };
            input
                .resource_reader
                .and_then(|reader| reader.query_resource(*unit, resource))
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: 0x204,
                })
        }
        ValueExpr::ReadEventProperty(property) => event_property(*property, input),
        ValueExpr::SelectorCount(selector) => selector_units(input, *selector)
            .and_then(|units| i64::try_from(units.len()).ok())
            .map(RuleValue::Integer)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            }),
        ValueExpr::SelectorSum { selector, value } => {
            let units = selector_units(input, *selector).ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            })?;
            let mut values = units
                .iter()
                .copied()
                .map(|unit| evaluate_value(value, input, Some(unit)));
            let Some(first) = values.next() else {
                return Err(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: selector.get(),
                });
            };
            values.try_fold(first?, |sum, value| add_values(sum, value?))
        }
        ValueExpr::EventId => Ok(RuleValue::StableId(input.occurrence.event.get())),
        ValueExpr::EventOwner => optional_unit(input.cause.owner),
        ValueExpr::EventActor => optional_unit(input.cause.actor),
        ValueExpr::EventApplier => optional_unit(input.cause.applier),
        ValueExpr::EventTarget => optional_unit(input.cause.target),
        ValueExpr::CurrentTarget => optional_unit(current_target),
        ValueExpr::QueryStat {
            subject,
            stat,
            purpose,
        } => {
            let origin = *subject;
            let subject = query_subject(origin, input, current_target)?;
            input
                .stat_reader
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: 0x201,
                })?
                .query_stat(origin, subject, *stat, *purpose)
                .map(RuleValue::Scalar)
        }
        ValueExpr::QueryBaseStat { subject, stat } => {
            let origin = *subject;
            let subject = query_subject(origin, input, current_target)?;
            input
                .stat_reader
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: 0x203,
                })?
                .query_base_stat(origin, subject, *stat)
                .map(RuleValue::Scalar)
        }
        ValueExpr::QueryShield {
            subject,
            observation,
        } => {
            let subject = query_subject(*subject, input, current_target)?;
            let current = input
                .battle_query_reader
                .and_then(|reader| reader.current_shield(subject))
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: 0x202,
                })?;
            let value = match observation {
                ShieldObservation::Current => current,
                ShieldObservation::BeforeEvent if input.cause.target == Some(subject) => {
                    input.event_facts.shield_before.unwrap_or(current)
                }
                ShieldObservation::BeforeEvent => current,
            };
            Ok(RuleValue::Scalar(value))
        }
        ValueExpr::QueryHp { subject } => {
            let subject = query_subject(*subject, input, current_target)?;
            input
                .battle_query_reader
                .and_then(|reader| reader.current_hp(subject))
                .map(RuleValue::Scalar)
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: 0x21e,
                })
        }
        ValueExpr::QueryMaximumEnergy(subject) => {
            let subject = query_subject(*subject, input, current_target)?;
            input
                .battle_query_reader
                .and_then(|reader| reader.maximum_energy(subject))
                .map(RuleValue::Scalar)
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: 0x21f,
                })
        }
        ValueExpr::QueryEffectStacks { subject, effect } => {
            let subject = query_subject(*subject, input, current_target)?;
            input
                .battle_query_reader
                .and_then(|reader| reader.effect_stacks(subject, *effect))
                .map(RuleValue::Integer)
                .ok_or(RuleEvaluationError {
                    kind: RuleEvaluationErrorKind::MissingValue,
                    context: effect.get(),
                })
        }
        ValueExpr::QueryEffectCategoryStacks { subject, category } => {
            query_effect_category_stacks(*subject, *category, input, current_target)
        }
        ValueExpr::Add(lhs, rhs) => arithmetic(lhs, rhs, input, current_target, Arithmetic::Add),
        ValueExpr::Subtract(lhs, rhs) => {
            arithmetic(lhs, rhs, input, current_target, Arithmetic::Subtract)
        }
        ValueExpr::Multiply { lhs, rhs, rounding } => arithmetic(
            lhs,
            rhs,
            input,
            current_target,
            Arithmetic::Multiply(*rounding),
        ),
        ValueExpr::Divide { lhs, rhs, rounding } => arithmetic(
            lhs,
            rhs,
            input,
            current_target,
            Arithmetic::Divide(*rounding),
        ),
        ValueExpr::Minimum(lhs, rhs) => extremum(lhs, rhs, input, current_target, true),
        ValueExpr::Maximum(lhs, rhs) => extremum(lhs, rhs, input, current_target, false),
        ValueExpr::Clamp {
            value,
            minimum,
            maximum,
        } => {
            let value = evaluate_value(value, input, current_target)?;
            let minimum = evaluate_value(minimum, input, current_target)?;
            let maximum = evaluate_value(maximum, input, current_target)?;
            if compare_values(&minimum, &maximum)? == core::cmp::Ordering::Greater {
                return Err(type_error(0x103));
            }
            if compare_values(&value, &minimum)? == core::cmp::Ordering::Less {
                Ok(minimum)
            } else if compare_values(&value, &maximum)? == core::cmp::Ordering::Greater {
                Ok(maximum)
            } else {
                Ok(value)
            }
        }
        ValueExpr::Negate(value) => match evaluate_value(value, input, current_target)? {
            RuleValue::Integer(value) => value
                .checked_neg()
                .map(RuleValue::Integer)
                .ok_or(numeric_error(0x104)),
            RuleValue::Scalar(value) => value
                .checked_neg()
                .map(RuleValue::Scalar)
                .map_err(|_| numeric_error(0x105)),
            _ => Err(type_error(0x106)),
        },
        ValueExpr::Choose {
            condition,
            when_true,
            when_false,
        } => {
            let selected = if evaluate_condition(condition, input, current_target)? {
                when_true
            } else {
                when_false
            };
            evaluate_value(selected, input, current_target)
        }
        ValueExpr::Convert {
            value,
            target,
            rounding,
        } => convert(
            evaluate_value(value, input, current_target)?,
            *target,
            *rounding,
        ),
    }
}
