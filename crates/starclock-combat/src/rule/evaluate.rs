//! Deterministic, budgeted and mutation-free Rule IR evaluation.

use core::cmp::Ordering;
use std::collections::BTreeSet;

use crate::modifier::model::{FormulaPurpose, StatKind, StatQuerySubject};
use crate::{NumericError, ProgramId, RuleId, Scalar, StateSlotDefinitionId, UnitId};

use super::model::{
    Comparison, ConditionExpr, EventFilter, ProgramStep, RuleEmission, RuleEvaluationInput,
    RuleOperationTemplate, RuleValue, RuleValueKind, TriggerDef, ValueExpr, once_key,
};

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

pub(crate) const fn stat_query_error(context: u32) -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingValue,
        context,
    }
}

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
            || !matches_filter(&trigger.filter, input)
            || !evaluate_condition(&trigger.condition, input, None)?
        {
            return Ok(Vec::new());
        }
        let key = once_key(trigger.id, trigger.once_scope, input.occurrence).ok_or(
            RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: trigger.id.get(),
            },
        )?;
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
        } => RuleEmission::Damage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            class: *class,
            element: *element,
            can_crit: *can_crit,
            current_target,
        },
        RuleOperationTemplate::TrueDamage { selector, amount } => RuleEmission::TrueDamage {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
            current_target,
        },
        RuleOperationTemplate::Heal { selector, amount } => RuleEmission::Heal {
            selector: *selector,
            amount: evaluate_value(amount, input, current_target)?,
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
        RuleOperationTemplate::AddWeakness { selector, element } => RuleEmission::AddWeakness {
            selector: *selector,
            element: *element,
            current_target,
        },
        RuleOperationTemplate::RemoveWeakness { selector, element } => {
            RuleEmission::RemoveWeakness {
                selector: *selector,
                element: *element,
                current_target,
            }
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
            chance,
            base_chance,
            rng_purpose,
        } => RuleEmission::ApplyEffect {
            selector: *selector,
            effect: *effect,
            chance: *chance,
            base_chance: base_chance
                .as_ref()
                .map(|value| evaluate_value(value, input, current_target))
                .transpose()?,
            rng_purpose: *rng_purpose,
            current_target,
        },
        RuleOperationTemplate::RemoveEffect { selector, effect } => RuleEmission::RemoveEffect {
            selector: *selector,
            effect: *effect,
            current_target,
        },
        RuleOperationTemplate::DetonateDot {
            selector,
            fraction,
            required_tag,
        } => RuleEmission::DetonateDot {
            selector: *selector,
            fraction: evaluate_value(fraction, input, current_target)?,
            required_tag: *required_tag,
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
        } => RuleEmission::QueueAction {
            actor_selector: *actor_selector,
            target_selector: *target_selector,
            ability: *ability,
            priority: *priority,
            current_target,
        },
        RuleOperationTemplate::GrantExtraTurn { actor_selector } => RuleEmission::GrantExtraTurn {
            actor_selector: *actor_selector,
            current_target,
        },
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
    })
}

/// Applies the cheap indexed cause filter without inferring cause roles.
#[must_use]
pub fn matches_filter(filter: &EventFilter, input: RuleEvaluationInput<'_>) -> bool {
    filter
        .owner
        .is_none_or(|value| input.cause.owner == Some(value))
        && filter
            .actor
            .is_none_or(|value| input.cause.actor == Some(value))
        && filter
            .applier
            .is_none_or(|value| input.cause.applier == Some(value))
        && filter
            .target
            .is_none_or(|value| input.cause.target == Some(value))
        && filter
            .source
            .is_none_or(|value| input.cause.source == Some(value))
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
        ValueExpr::SelectorCount(selector) => selector_units(input, *selector)
            .and_then(|units| i64::try_from(units.len()).ok())
            .map(RuleValue::Integer)
            .ok_or(RuleEvaluationError {
                kind: RuleEvaluationErrorKind::MissingValue,
                context: selector.get(),
            }),
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
            if compare_values(&minimum, &maximum)? == Ordering::Greater {
                return Err(type_error(0x103));
            }
            if compare_values(&value, &minimum)? == Ordering::Less {
                Ok(minimum)
            } else if compare_values(&value, &maximum)? == Ordering::Greater {
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

fn query_subject(
    subject: StatQuerySubject,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
) -> Result<UnitId, RuleEvaluationError> {
    let value = match subject {
        StatQuerySubject::Owner => input.cause.owner,
        StatQuerySubject::Actor => input.cause.actor,
        StatQuerySubject::Applier => input.cause.applier,
        StatQuerySubject::EventTarget => input.cause.target,
        StatQuerySubject::CurrentTarget => current_target,
    };
    value.ok_or(RuleEvaluationError {
        kind: RuleEvaluationErrorKind::MissingValue,
        context: 0x202,
    })
}

#[derive(Clone, Copy)]
enum Arithmetic {
    Add,
    Subtract,
    Multiply(crate::Rounding),
    Divide(crate::Rounding),
}

fn arithmetic(
    lhs: &ValueExpr,
    rhs: &ValueExpr,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
    operation: Arithmetic,
) -> Result<RuleValue, RuleEvaluationError> {
    let lhs = evaluate_value(lhs, input, current_target)?;
    let rhs = evaluate_value(rhs, input, current_target)?;
    match (lhs, rhs) {
        (RuleValue::Integer(lhs), RuleValue::Integer(rhs)) => {
            let value = match operation {
                Arithmetic::Add => lhs.checked_add(rhs),
                Arithmetic::Subtract => lhs.checked_sub(rhs),
                Arithmetic::Multiply(_) => lhs.checked_mul(rhs),
                Arithmetic::Divide(_) if rhs == 0 => None,
                Arithmetic::Divide(_) => lhs.checked_div(rhs),
            };
            value.map(RuleValue::Integer).ok_or(numeric_error(0x110))
        }
        (RuleValue::Scalar(lhs), RuleValue::Scalar(rhs)) => {
            let value = match operation {
                Arithmetic::Add => lhs.checked_add(rhs),
                Arithmetic::Subtract => lhs.checked_sub(rhs),
                Arithmetic::Multiply(rounding) => lhs.checked_mul(rhs, rounding),
                Arithmetic::Divide(rounding) => lhs.checked_div(rhs, rounding),
            };
            value
                .map(RuleValue::Scalar)
                .map_err(|_| numeric_error(0x111))
        }
        _ => Err(type_error(0x112)),
    }
}

fn extremum(
    lhs: &ValueExpr,
    rhs: &ValueExpr,
    input: RuleEvaluationInput<'_>,
    current_target: Option<UnitId>,
    minimum: bool,
) -> Result<RuleValue, RuleEvaluationError> {
    let lhs = evaluate_value(lhs, input, current_target)?;
    let rhs = evaluate_value(rhs, input, current_target)?;
    let ordering = compare_values(&lhs, &rhs)?;
    Ok(
        if (minimum && ordering != Ordering::Greater) || (!minimum && ordering != Ordering::Less) {
            lhs
        } else {
            rhs
        },
    )
}

fn convert(
    value: RuleValue,
    target: RuleValueKind,
    rounding: crate::Rounding,
) -> Result<RuleValue, RuleEvaluationError> {
    if value.kind() == target {
        return Ok(value);
    }
    match (value, target) {
        (RuleValue::Integer(value), RuleValueKind::Scalar) => {
            crate::Scalar::checked_from_integer(value)
                .map(RuleValue::Scalar)
                .map_err(|_| numeric_error(0x120))
        }
        (RuleValue::Scalar(value), RuleValueKind::Integer) => value
            .rounded_integer(rounding)
            .map(RuleValue::Integer)
            .map_err(|_| numeric_error(0x121)),
        _ => Err(RuleEvaluationError {
            kind: RuleEvaluationErrorKind::InvalidConversion,
            context: 0x122,
        }),
    }
}

fn compare(
    lhs: &RuleValue,
    operator: Comparison,
    rhs: &RuleValue,
) -> Result<bool, RuleEvaluationError> {
    Ok(compare_ordering(compare_values(lhs, rhs)?, operator))
}

fn compare_ordering(ordering: Ordering, operator: Comparison) -> bool {
    match operator {
        Comparison::Equal => ordering == Ordering::Equal,
        Comparison::NotEqual => ordering != Ordering::Equal,
        Comparison::Less => ordering == Ordering::Less,
        Comparison::LessOrEqual => ordering != Ordering::Greater,
        Comparison::Greater => ordering == Ordering::Greater,
        Comparison::GreaterOrEqual => ordering != Ordering::Less,
    }
}

fn compare_values(lhs: &RuleValue, rhs: &RuleValue) -> Result<Ordering, RuleEvaluationError> {
    match (lhs, rhs) {
        (RuleValue::Integer(lhs), RuleValue::Integer(rhs)) => Ok(lhs.cmp(rhs)),
        (RuleValue::Scalar(lhs), RuleValue::Scalar(rhs)) => Ok(lhs.cmp(rhs)),
        (RuleValue::Boolean(lhs), RuleValue::Boolean(rhs)) => Ok(lhs.cmp(rhs)),
        (RuleValue::StableId(lhs), RuleValue::StableId(rhs)) => Ok(lhs.cmp(rhs)),
        (RuleValue::OptionalStableId(lhs), RuleValue::OptionalStableId(rhs)) => Ok(lhs.cmp(rhs)),
        (RuleValue::OrderedStableIdSet(lhs), RuleValue::OrderedStableIdSet(rhs)) => {
            Ok(lhs.cmp(rhs))
        }
        _ => Err(type_error(0x130)),
    }
}

fn selector_units(
    input: RuleEvaluationInput<'_>,
    selector: crate::SelectorId,
) -> Option<&[UnitId]> {
    input
        .selectors
        .binary_search_by_key(&selector, |result| result.selector)
        .ok()
        .map(|index| input.selectors[index].units)
}

fn slot_value(input: RuleEvaluationInput<'_>, slot: StateSlotDefinitionId) -> Option<&RuleValue> {
    input
        .slots
        .binary_search_by_key(&slot, |(id, _)| *id)
        .ok()
        .map(|index| &input.slots[index].1)
}

fn optional_unit(value: Option<UnitId>) -> Result<RuleValue, RuleEvaluationError> {
    Ok(RuleValue::OptionalStableId(value.map(UnitId::get)))
}

fn type_error(context: u32) -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::TypeMismatch,
        context,
    }
}

fn numeric_error(context: u32) -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::Numeric,
        context,
    }
}

fn budget_error() -> RuleEvaluationError {
    RuleEvaluationError {
        kind: RuleEvaluationErrorKind::BudgetExceeded,
        context: 0x1ff,
    }
}

impl From<NumericError> for RuleEvaluationError {
    fn from(_: NumericError) -> Self {
        numeric_error(0x1fe)
    }
}

/// Stable definition-only total order for candidate triggers.
#[must_use]
pub fn trigger_definition_order(
    rule: RuleId,
    source: crate::SourceDefinitionId,
    trigger: &super::model::TriggerDef,
) -> super::model::TriggerDefinitionOrder {
    super::model::TriggerDefinitionOrder {
        phase: trigger.phase,
        priority: trigger.priority,
        source,
        rule,
        trigger: trigger.id,
    }
}
