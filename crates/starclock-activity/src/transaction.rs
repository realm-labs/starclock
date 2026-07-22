use std::collections::BTreeMap;

use crate::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId,
    ActivityExpression, ActivityInventoryId, ActivityModifierId, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivitySlotId,
    ActivityStateDefinition, ActivityTerminalOutcome, ActivityValue, NodeId,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityCause {
    command_sequence: u64,
    program: ActivityProgramId,
    node: NodeId,
    option: Option<ActivityOptionId>,
}

impl ActivityCause {
    #[must_use]
    pub const fn new(
        command_sequence: u64,
        program: ActivityProgramId,
        node: NodeId,
    ) -> Option<Self> {
        if command_sequence == 0 {
            None
        } else {
            Some(Self {
                command_sequence,
                program,
                node,
                option: None,
            })
        }
    }
    #[must_use]
    pub const fn command_sequence(self) -> u64 {
        self.command_sequence
    }
    #[must_use]
    pub const fn program(self) -> ActivityProgramId {
        self.program
    }
    #[must_use]
    pub const fn node(self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn option(self) -> Option<ActivityOptionId> {
        self.option
    }
    #[must_use]
    pub const fn with_option(mut self, option: ActivityOptionId) -> Self {
        self.option = Some(option);
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityTransactionEventKind {
    SlotChanged(ActivitySlotId),
    CounterChanged {
        slot: ActivitySlotId,
        key: u64,
    },
    InventoryChanged {
        inventory: ActivityInventoryId,
        content: u64,
    },
    ModifierChanged(ActivityModifierId),
    EdgeTraversed(ActivityEdgeId),
    DecisionOffered(ActivityDecisionId),
    Terminal(ActivityTerminalOutcome),
    Faulted(ActivityFault),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityTransactionEvent {
    cause: ActivityCause,
    kind: ActivityTransactionEventKind,
}

impl ActivityTransactionEvent {
    #[must_use]
    pub const fn cause(&self) -> ActivityCause {
        self.cause
    }
    #[must_use]
    pub const fn kind(&self) -> &ActivityTransactionEventKind {
        &self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityFault {
    ArithmeticOverflow,
    TypeMismatch,
    MissingSlot(ActivitySlotId),
    SlotBounds(ActivitySlotId),
    MissingInventory(ActivityInventoryId),
    InventoryBounds(ActivityInventoryId),
    MissingModifier(ActivityModifierId),
    ModifierBounds(ActivityModifierId),
    InvalidGraphEdge(ActivityEdgeId),
    VisitLimitExceeded,
    InvalidProgramBoundary,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityTransactionRejection {
    ConditionNotSatisfied,
    StateAlreadyAtBoundary,
    DecisionNotOffered,
    UnknownOption,
    CauseMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActivityTransactionOutcome {
    Committed(Box<[ActivityTransactionEvent]>),
    Rejected(ActivityTransactionRejection),
    Faulted(Box<[ActivityTransactionEvent]>, ActivityFault),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingDecision {
    id: ActivityDecisionId,
    kind: ActivityDecisionKind,
    options: Box<[ActivityOptionDefinition]>,
}

/// Mutable transaction substrate; adapters never receive mutable access to it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityTransactionState {
    definition: ActivityStateDefinition,
    slots: BTreeMap<ActivitySlotId, ActivityValue>,
    inventories: BTreeMap<ActivityInventoryId, BTreeMap<u64, u32>>,
    modifiers: BTreeMap<ActivityModifierId, u32>,
    current_node: NodeId,
    node_visits: BTreeMap<NodeId, u32>,
    edge_traversals: BTreeMap<ActivityEdgeId, u32>,
    total_visits: u32,
    pending: Option<PendingDecision>,
    terminal: Option<ActivityTerminalOutcome>,
}

impl ActivityTransactionState {
    #[must_use]
    pub fn new(definition: ActivityStateDefinition, current_node: NodeId) -> Self {
        let slots = definition
            .slots()
            .iter()
            .map(|item| (item.id(), item.initial().clone()))
            .collect();
        let inventories = definition
            .inventories()
            .iter()
            .map(|item| (item.id(), BTreeMap::new()))
            .collect();
        let modifiers = definition
            .modifiers()
            .iter()
            .map(|item| (item.id(), 0))
            .collect();
        Self {
            definition,
            slots,
            inventories,
            modifiers,
            current_node,
            node_visits: BTreeMap::from([(current_node, 1)]),
            edge_traversals: BTreeMap::new(),
            total_visits: 1,
            pending: None,
            terminal: None,
        }
    }

    #[must_use]
    pub fn slot(&self, id: ActivitySlotId) -> Option<&ActivityValue> {
        self.slots.get(&id)
    }
    #[must_use]
    pub const fn current_node(&self) -> NodeId {
        self.current_node
    }
    #[must_use]
    pub const fn terminal(&self) -> Option<ActivityTerminalOutcome> {
        self.terminal
    }
    #[must_use]
    pub fn node_visits(&self, node: NodeId) -> u32 {
        self.node_visits.get(&node).copied().unwrap_or(0)
    }
    #[must_use]
    pub fn edge_traversals(&self, edge: ActivityEdgeId) -> u32 {
        self.edge_traversals.get(&edge).copied().unwrap_or(0)
    }

    pub fn apply_program(
        &mut self,
        program: &ActivityProgramDefinition,
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        if self.pending.is_some() || self.terminal.is_some() {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::StateAlreadyAtBoundary,
            );
        }
        if cause.program != program.id() || cause.node != self.current_node {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::CauseMismatch,
            );
        }
        let mut working = self.clone();
        let mut events = Vec::new();
        match working.execute(program.operations(), cause, graph, &mut events) {
            Ok(()) => {
                *self = working;
                ActivityTransactionOutcome::Committed(events.into_boxed_slice())
            }
            Err(ExecutionFailure::Rejected(error)) => ActivityTransactionOutcome::Rejected(error),
            Err(ExecutionFailure::Fault(fault)) => {
                let mut faulted = self.clone();
                faulted.terminal = Some(ActivityTerminalOutcome::Faulted);
                events.clear();
                events.push(ActivityTransactionEvent {
                    cause,
                    kind: ActivityTransactionEventKind::Faulted(fault),
                });
                *self = faulted;
                ActivityTransactionOutcome::Faulted(events.into_boxed_slice(), fault)
            }
        }
    }

    pub fn apply_option(
        &mut self,
        option: ActivityOptionId,
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        let Some(pending) = &self.pending else {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::DecisionNotOffered,
            );
        };
        if cause.node != self.current_node {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::CauseMismatch,
            );
        }
        let Some(selected) = pending.options.iter().find(|item| item.id() == option) else {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::UnknownOption,
            );
        };
        let operations = selected.operations().to_vec();
        let cause = cause.with_option(option);
        let mut working = self.clone();
        working.pending = None;
        let mut events = Vec::new();
        match working.execute(&operations, cause, graph, &mut events) {
            Ok(()) => {
                *self = working;
                ActivityTransactionOutcome::Committed(events.into_boxed_slice())
            }
            Err(ExecutionFailure::Rejected(error)) => ActivityTransactionOutcome::Rejected(error),
            Err(ExecutionFailure::Fault(fault)) => {
                let mut faulted = self.clone();
                faulted.pending = None;
                faulted.terminal = Some(ActivityTerminalOutcome::Faulted);
                events.clear();
                events.push(ActivityTransactionEvent {
                    cause,
                    kind: ActivityTransactionEventKind::Faulted(fault),
                });
                *self = faulted;
                ActivityTransactionOutcome::Faulted(events.into_boxed_slice(), fault)
            }
        }
    }

    fn execute(
        &mut self,
        operations: &[crate::ActivityOperation],
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ExecutionFailure> {
        for operation in operations {
            self.execute_one(operation, cause, graph, events)?;
        }
        Ok(())
    }

    fn execute_one(
        &mut self,
        operation: &crate::ActivityOperation,
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ExecutionFailure> {
        use crate::ActivityOperation as Op;
        match operation {
            Op::SetSlot { slot, value } => {
                let value = self.evaluate(value)?;
                self.set_slot(*slot, value)?;
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::SlotChanged(*slot),
                );
            }
            Op::AddToSlot { slot, delta } => {
                let current = integer(
                    self.slots
                        .get(slot)
                        .ok_or(ActivityFault::MissingSlot(*slot))?,
                )?;
                let value = current
                    .checked_add(integer(&self.evaluate(delta)?)?)
                    .ok_or(ActivityFault::ArithmeticOverflow)?;
                self.set_slot(*slot, ActivityValue::BoundedInteger(value))?;
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::SlotChanged(*slot),
                );
            }
            Op::AddCounter { slot, key, delta } => {
                self.add_counter(*slot, *key, integer(&self.evaluate(delta)?)?, cause, events)?
            }
            Op::AddInventory {
                inventory,
                content,
                count,
            } => self.change_inventory(
                *inventory,
                *content,
                integer(&self.evaluate(count)?)?,
                cause,
                events,
            )?,
            Op::RemoveInventory {
                inventory,
                content,
                count,
            } => self.change_inventory(
                *inventory,
                *content,
                integer(&self.evaluate(count)?)?
                    .checked_neg()
                    .ok_or(ActivityFault::ArithmeticOverflow)?,
                cause,
                events,
            )?,
            Op::AddModifier { modifier, stacks } => {
                self.change_modifier(*modifier, integer(&self.evaluate(stacks)?)?, cause, events)?
            }
            Op::RemoveModifier { modifier } => {
                self.change_modifier(*modifier, i64::MIN, cause, events)?
            }
            Op::Traverse(edge) => {
                let edge_def = graph
                    .edges()
                    .iter()
                    .find(|item| item.id() == *edge)
                    .ok_or(ActivityFault::InvalidGraphEdge(*edge))?;
                if edge_def.from() != self.current_node {
                    return Err(ActivityFault::InvalidGraphEdge(*edge).into());
                }
                let next_edge_count = self
                    .edge_traversals
                    .get(edge)
                    .copied()
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(ActivityFault::VisitLimitExceeded)?;
                let next_node = graph
                    .node(edge_def.to())
                    .ok_or(ActivityFault::InvalidGraphEdge(*edge))?;
                let next_node_count = self
                    .node_visits
                    .get(&edge_def.to())
                    .copied()
                    .unwrap_or(0)
                    .checked_add(1)
                    .ok_or(ActivityFault::VisitLimitExceeded)?;
                let next_total = self
                    .total_visits
                    .checked_add(1)
                    .ok_or(ActivityFault::VisitLimitExceeded)?;
                if next_edge_count > edge_def.maximum_traversals()
                    || next_node_count > next_node.maximum_visits()
                    || next_total > graph.maximum_total_visits()
                {
                    return Err(ActivityFault::VisitLimitExceeded.into());
                }
                self.edge_traversals.insert(*edge, next_edge_count);
                self.node_visits.insert(edge_def.to(), next_node_count);
                self.total_visits = next_total;
                self.current_node = edge_def.to();
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::EdgeTraversed(*edge),
                );
            }
            Op::Offer { kind, options } => {
                let mut enabled = Vec::new();
                for option in options.iter() {
                    if self.condition(option.enabled())? {
                        enabled.push(option.clone());
                    }
                }
                let options = enabled;
                if options.is_empty() {
                    return Err(ActivityFault::InvalidProgramBoundary.into());
                }
                let id = ActivityDecisionId::new(cause.command_sequence)
                    .ok_or(ActivityFault::InvalidProgramBoundary)?;
                self.pending = Some(PendingDecision {
                    id,
                    kind: *kind,
                    options: options.into_boxed_slice(),
                });
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::DecisionOffered(id),
                );
            }
            Op::Terminal(outcome) => {
                self.terminal = Some(*outcome);
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::Terminal(*outcome),
                );
            }
            Op::Require(condition) => {
                if !self.condition(condition)? {
                    return Err(ExecutionFailure::Rejected(
                        ActivityTransactionRejection::ConditionNotSatisfied,
                    ));
                }
            }
        }
        Ok(())
    }

    fn evaluate(&self, expression: &ActivityExpression) -> Result<ActivityValue, ActivityFault> {
        match expression {
            ActivityExpression::Literal(value) => Ok(value.clone()),
            ActivityExpression::Slot(slot) => self
                .slots
                .get(slot)
                .cloned()
                .ok_or(ActivityFault::MissingSlot(*slot)),
            ActivityExpression::Add(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_add)
            }
            ActivityExpression::Subtract(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_sub)
            }
            ActivityExpression::Minimum(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, |a, b| Some(a.min(b)))
            }
            ActivityExpression::Maximum(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, |a, b| Some(a.max(b)))
            }
            ActivityExpression::Negate(value) => match self.evaluate(value)? {
                ActivityValue::BoundedInteger(value) => value
                    .checked_neg()
                    .map(ActivityValue::BoundedInteger)
                    .ok_or(ActivityFault::ArithmeticOverflow),
                ActivityValue::FixedScalar(value) => value
                    .checked_neg()
                    .map(ActivityValue::FixedScalar)
                    .ok_or(ActivityFault::ArithmeticOverflow),
                _ => Err(ActivityFault::TypeMismatch),
            },
        }
    }

    fn condition(&self, condition: &ActivityCondition) -> Result<bool, ActivityFault> {
        match condition {
            ActivityCondition::Boolean(value) => match self.evaluate(value)? {
                ActivityValue::Boolean(value) => Ok(value),
                _ => Err(ActivityFault::TypeMismatch),
            },
            ActivityCondition::Equal(a, b) => Ok(self.evaluate(a)? == self.evaluate(b)?),
            ActivityCondition::LessThan(a, b) => match (self.evaluate(a)?, self.evaluate(b)?) {
                (ActivityValue::BoundedInteger(a), ActivityValue::BoundedInteger(b))
                | (ActivityValue::FixedScalar(a), ActivityValue::FixedScalar(b)) => Ok(a < b),
                _ => Err(ActivityFault::TypeMismatch),
            },
            ActivityCondition::Not(value) => Ok(!self.condition(value)?),
            ActivityCondition::All(values) => {
                for value in values.iter() {
                    if !self.condition(value)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ActivityCondition::Any(values) => {
                for value in values.iter() {
                    if self.condition(value)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
        }
    }

    fn set_slot(&mut self, id: ActivitySlotId, value: ActivityValue) -> Result<(), ActivityFault> {
        let definition = self
            .definition
            .slots()
            .iter()
            .find(|item| item.id() == id)
            .ok_or(ActivityFault::MissingSlot(id))?;
        if !definition.accepts(&value) {
            return Err(ActivityFault::SlotBounds(id));
        }
        self.slots.insert(id, value);
        Ok(())
    }

    fn add_counter(
        &mut self,
        id: ActivitySlotId,
        key: u64,
        delta: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        if key == 0 {
            return Err(ActivityFault::TypeMismatch);
        }
        let mut values = match self.slots.get(&id).ok_or(ActivityFault::MissingSlot(id))? {
            ActivityValue::BoundedCounterMap(values) => values.to_vec(),
            _ => return Err(ActivityFault::TypeMismatch),
        };
        match values.binary_search_by_key(&key, |item| item.0) {
            Ok(index) => {
                values[index].1 = values[index]
                    .1
                    .checked_add(delta)
                    .ok_or(ActivityFault::ArithmeticOverflow)?
            }
            Err(index) => values.insert(index, (key, delta)),
        }
        self.set_slot(
            id,
            ActivityValue::BoundedCounterMap(values.into_boxed_slice()),
        )?;
        push(
            events,
            cause,
            ActivityTransactionEventKind::CounterChanged { slot: id, key },
        );
        Ok(())
    }

    fn change_inventory(
        &mut self,
        id: ActivityInventoryId,
        content: u64,
        delta: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        if content == 0 {
            return Err(ActivityFault::TypeMismatch);
        }
        let definition = self
            .definition
            .inventories()
            .iter()
            .find(|item| item.id() == id)
            .ok_or(ActivityFault::MissingInventory(id))?;
        let inventory = self
            .inventories
            .get_mut(&id)
            .ok_or(ActivityFault::MissingInventory(id))?;
        let current = i64::from(*inventory.get(&content).unwrap_or(&0));
        let next = current
            .checked_add(delta)
            .ok_or(ActivityFault::ArithmeticOverflow)?;
        if next < 0
            || next > i64::from(definition.maximum_stack())
            || (current == 0
                && next > 0
                && inventory.len() >= definition.maximum_entries() as usize)
        {
            return Err(ActivityFault::InventoryBounds(id));
        }
        if next == 0 {
            inventory.remove(&content);
        } else {
            inventory.insert(content, next as u32);
        }
        push(
            events,
            cause,
            ActivityTransactionEventKind::InventoryChanged {
                inventory: id,
                content,
            },
        );
        Ok(())
    }

    fn change_modifier(
        &mut self,
        id: ActivityModifierId,
        delta: i64,
        cause: ActivityCause,
        events: &mut Vec<ActivityTransactionEvent>,
    ) -> Result<(), ActivityFault> {
        let definition = self
            .definition
            .modifiers()
            .iter()
            .find(|item| item.id() == id)
            .ok_or(ActivityFault::MissingModifier(id))?;
        let current = i64::from(
            *self
                .modifiers
                .get(&id)
                .ok_or(ActivityFault::MissingModifier(id))?,
        );
        let next = if delta == i64::MIN {
            0
        } else {
            current
                .checked_add(delta)
                .ok_or(ActivityFault::ArithmeticOverflow)?
        };
        if next < 0 || next > i64::from(definition.maximum_stacks()) {
            return Err(ActivityFault::ModifierBounds(id));
        }
        self.modifiers.insert(id, next as u32);
        push(
            events,
            cause,
            ActivityTransactionEventKind::ModifierChanged(id),
        );
        Ok(())
    }
}

fn integer(value: &ActivityValue) -> Result<i64, ActivityFault> {
    match value {
        ActivityValue::BoundedInteger(value) => Ok(*value),
        _ => Err(ActivityFault::TypeMismatch),
    }
}
fn numeric_binary(
    left: ActivityValue,
    right: ActivityValue,
    operation: impl FnOnce(i64, i64) -> Option<i64>,
) -> Result<ActivityValue, ActivityFault> {
    let (left, right, fixed) = match (left, right) {
        (ActivityValue::BoundedInteger(left), ActivityValue::BoundedInteger(right)) => {
            (left, right, false)
        }
        (ActivityValue::FixedScalar(left), ActivityValue::FixedScalar(right)) => {
            (left, right, true)
        }
        _ => return Err(ActivityFault::TypeMismatch),
    };
    let value = operation(left, right).ok_or(ActivityFault::ArithmeticOverflow)?;
    Ok(if fixed {
        ActivityValue::FixedScalar(value)
    } else {
        ActivityValue::BoundedInteger(value)
    })
}
fn push(
    events: &mut Vec<ActivityTransactionEvent>,
    cause: ActivityCause,
    kind: ActivityTransactionEventKind,
) {
    events.push(ActivityTransactionEvent { cause, kind });
}

enum ExecutionFailure {
    Rejected(ActivityTransactionRejection),
    Fault(ActivityFault),
}
impl From<ActivityFault> for ExecutionFailure {
    fn from(value: ActivityFault) -> Self {
        Self::Fault(value)
    }
}
