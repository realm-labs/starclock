use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

mod condition;
mod decision;
mod extension;
mod movement;
mod ordered_id_set;
mod participant_carry;
mod support;

use self::support::{
    ExecutionFailure, decision_view, encode_value, integer, inventory_view, numeric_binary, push,
    settle_integer,
};

use crate::{
    ACTIVITY_RNG_REVISION, ACTIVITY_STATE_CODEC_REVISION, ACTIVITY_STATE_HASH_REVISION,
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityDefinitionIdentity,
    ActivityEdgeId, ActivityExpression, ActivityGraphDefinition, ActivityInstanceId,
    ActivityInventoryId, ActivityModifierId, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityRngStreams, ActivitySlotId,
    ActivityStateDefinition, ActivityStateHash, ActivityStateVisibility, ActivityTerminalOutcome,
    ActivityValue, LogicalScopeInstance, NodeId, ParticipantId, SlotResetPoint,
    battle_preparation::ActivityAttemptState,
    battle_settlement::{ActivityAwaitingBattle, ActivityCarryLedger, MetricSettlementPolicy},
    codec::ActivityStateEncoder,
    logical_scope::LogicalScopeRuntimeState,
    view::{ActivityDebugView, ActivityPlayerView, ActivitySlotView},
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
    SlotReset {
        slot: ActivitySlotId,
        point: SlotResetPoint,
    },
    CounterChanged {
        slot: ActivitySlotId,
        key: u64,
    },
    InventoryChanged {
        inventory: ActivityInventoryId,
        content: u64,
    },
    ModifierChanged(ActivityModifierId),
    ParticipantCarryChanged(ParticipantId),
    EdgeTraversed(ActivityEdgeId),
    NodeRelocated(NodeId),
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
    MissingParticipant(ParticipantId),
    InvalidParticipantState(ParticipantId),
    InvalidGraphEdge(ActivityEdgeId),
    InvalidGraphNode(NodeId),
    VisitLimitExceeded,
    LogicalScopeLimitExceeded,
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
#[derive(Debug, Eq, PartialEq)]
pub struct ActivityTransactionState {
    definition: ActivityStateDefinition,
    slots: BTreeMap<ActivitySlotId, ActivityValue>,
    inventories: BTreeMap<ActivityInventoryId, BTreeMap<u64, u32>>,
    modifiers: BTreeMap<ActivityModifierId, u32>,
    current_node: NodeId,
    command_sequence: u64,
    node_visits: BTreeMap<NodeId, u32>,
    edge_traversals: BTreeMap<ActivityEdgeId, u32>,
    total_visits: u32,
    logical_scopes: LogicalScopeRuntimeState,
    pub(crate) attempt: Option<ActivityAttemptState>,
    pub(crate) awaiting_battle: Option<ActivityAwaitingBattle>,
    pub(crate) carry: ActivityCarryLedger,
    pub(crate) completed_battles: Vec<crate::BattleResultDigest>,
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
        let logical_scopes =
            LogicalScopeRuntimeState::new(definition.logical_scopes(), current_node)
                .expect("validated logical scope entry fits declared instance limits");
        Self {
            definition,
            slots,
            inventories,
            modifiers,
            current_node,
            command_sequence: 0,
            node_visits: BTreeMap::from([(current_node, 1)]),
            edge_traversals: BTreeMap::new(),
            total_visits: 1,
            logical_scopes,
            attempt: None,
            awaiting_battle: None,
            carry: ActivityCarryLedger::default(),
            completed_battles: Vec::new(),
            pending: None,
            terminal: None,
        }
    }

    /// Creates initial state with validated, canonical bootstrap overrides.
    /// Bootstrap values are definition input, not accepted commands, so the
    /// command sequence remains zero and no events are emitted.
    pub fn new_with_initial_values(
        definition: ActivityStateDefinition,
        current_node: NodeId,
        mut overrides: Vec<(ActivitySlotId, ActivityValue)>,
    ) -> Result<Self, ActivityFault> {
        overrides.sort_by_key(|item| item.0);
        if overrides.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(ActivityFault::InvalidProgramBoundary);
        }
        let mut state = Self::new(definition, current_node);
        for (slot, value) in overrides {
            state.set_slot(slot, value)?;
        }
        Ok(state)
    }

    #[must_use]
    pub fn slot(&self, id: ActivitySlotId) -> Option<&ActivityValue> {
        self.slots.get(&id)
    }
    /// Returns one inventory snapshot in canonical content-ID order.
    ///
    /// This is a read-only battle-snapshot boundary. Callers cannot mutate the
    /// inventory except through accepted Activity commands.
    #[must_use]
    pub fn inventory_entries(
        &self,
        id: ActivityInventoryId,
    ) -> Option<impl ExactSizeIterator<Item = (u64, u32)> + '_> {
        self.inventories
            .get(&id)
            .map(|entries| entries.iter().map(|(content, count)| (*content, *count)))
    }
    /// Whether the current node's most recent battle attempt has already
    /// committed a verified result.
    ///
    /// A mode may use this read-only boundary to prevent preparing the same
    /// encounter twice before an ordinary graph transition enters a new node.
    #[must_use]
    pub fn current_battle_attempt_is_settled(&self) -> bool {
        self.attempt
            .as_ref()
            .is_some_and(|attempt| attempt.is_settled_at(self.current_node))
    }
    pub(crate) const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.definition
    }
    #[must_use]
    pub(crate) fn counter_value(&self, id: ActivitySlotId, key: u64) -> Option<i64> {
        match self.slots.get(&id)? {
            ActivityValue::BoundedCounterMap(values) => Some(
                values
                    .binary_search_by_key(&key, |item| item.0)
                    .ok()
                    .map(|index| values[index].1)
                    .unwrap_or(0),
            ),
            _ => None,
        }
    }
    #[must_use]
    pub(crate) fn pending_option_ids(&self) -> Option<Box<[ActivityOptionId]>> {
        self.pending.as_ref().map(|pending| {
            pending
                .options
                .iter()
                .map(ActivityOptionDefinition::id)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
    }
    pub(crate) fn restrict_pending_options(
        &mut self,
        mut selected: Vec<ActivityOptionId>,
    ) -> Result<(), ActivityFault> {
        selected.sort_unstable();
        selected.dedup();
        if selected.is_empty() {
            return Err(ActivityFault::InvalidProgramBoundary);
        }
        let pending = self
            .pending
            .as_mut()
            .ok_or(ActivityFault::InvalidProgramBoundary)?;
        if selected
            .iter()
            .any(|id| !pending.options.iter().any(|option| option.id() == *id))
        {
            return Err(ActivityFault::InvalidProgramBoundary);
        }
        pending.options = pending
            .options
            .iter()
            .filter(|option| selected.binary_search(&option.id()).is_ok())
            .cloned()
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(())
    }
    pub(crate) fn replace_counter_keys(
        &mut self,
        slot: ActivitySlotId,
        mut keys: Vec<ActivityOptionId>,
    ) -> Result<(), ActivityFault> {
        keys.sort_unstable();
        keys.dedup();
        self.set_slot(
            slot,
            ActivityValue::BoundedCounterMap(
                keys.into_iter()
                    .map(|key| (key.get(), 1))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
        )
    }
    #[must_use]
    pub const fn current_node(&self) -> NodeId {
        self.current_node
    }
    #[must_use]
    pub const fn command_sequence(&self) -> u64 {
        self.command_sequence
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
    #[must_use]
    pub fn active_logical_scopes(&self) -> &[LogicalScopeInstance] {
        self.logical_scopes.active()
    }

    #[must_use]
    pub fn canonical_state_bytes(
        &self,
        identity: ActivityDefinitionIdentity,
        graph: &ActivityGraphDefinition,
        instance: ActivityInstanceId,
        rng: &ActivityRngStreams,
    ) -> Box<[u8]> {
        let mut writer = ActivityStateEncoder::new();
        writer.text(ACTIVITY_STATE_CODEC_REVISION);
        writer.text(ACTIVITY_STATE_HASH_REVISION);
        writer.text(ACTIVITY_RNG_REVISION);
        writer.u32(identity.id().get());
        writer.digest(identity.definition_digest().bytes());
        writer.digest(identity.config_digest().bytes());
        writer.digest(graph.digest().bytes());
        writer.u64(instance.get());
        writer.u64(self.command_sequence);
        writer.u32(self.current_node.get());
        let section = graph.node(self.current_node).map(|node| node.section());
        writer.bool(section.is_some());
        if let Some(section) = section {
            writer.u32(section.get());
        }
        writer.bool(self.attempt.is_some());
        if let Some(attempt) = &self.attempt {
            attempt.encode(&mut writer);
            writer.bool(self.awaiting_battle.is_some());
            if let Some(awaiting) = &self.awaiting_battle {
                awaiting.encode(&mut writer);
            }
            self.carry.encode(&mut writer);
        }
        writer.u32(self.total_visits);
        writer.u32(self.node_visits.len() as u32);
        for (node, count) in &self.node_visits {
            writer.u32(node.get());
            writer.u32(*count);
        }
        writer.u32(self.edge_traversals.len() as u32);
        for (edge, count) in &self.edge_traversals {
            writer.u32(edge.get());
            writer.u32(*count);
        }
        if !self.definition.logical_scopes().is_empty() {
            self.logical_scopes.encode(&mut writer);
        }
        writer.u32(self.slots.len() as u32);
        for (slot, value) in &self.slots {
            writer.u32(slot.get());
            encode_value(&mut writer, value);
        }
        writer.u32(self.inventories.len() as u32);
        for (inventory, entries) in &self.inventories {
            writer.u32(inventory.get());
            writer.u32(entries.len() as u32);
            for (content, count) in entries {
                writer.u64(*content);
                writer.u32(*count);
            }
        }
        writer.u32(self.modifiers.len() as u32);
        for (modifier, stacks) in &self.modifiers {
            writer.u32(modifier.get());
            writer.u32(*stacks);
        }
        writer.bool(self.pending.is_some());
        if let Some(pending) = &self.pending {
            writer.u64(pending.id.get());
            writer.byte(pending.kind as u8);
            writer.u32(pending.options.len() as u32);
            for option in pending.options.iter() {
                writer.u64(option.id().get());
                writer.i32(option.priority());
            }
        }
        writer.bool(self.terminal.is_some());
        if let Some(terminal) = self.terminal {
            writer.byte(terminal as u8);
        }
        let snapshots = rng.snapshots();
        writer.u32(snapshots.len() as u32);
        for snapshot in snapshots.iter() {
            writer.byte(snapshot.label() as u8);
            writer.digest(snapshot.seed());
            writer.u64(snapshot.draw_count());
        }
        writer.u32(0); // checkpoints
        writer.u32(self.completed_battles.len() as u32);
        for digest in &self.completed_battles {
            writer.digest(digest.bytes());
        }
        writer.finish()
    }

    #[must_use]
    pub fn state_hash(
        &self,
        identity: ActivityDefinitionIdentity,
        graph: &ActivityGraphDefinition,
        instance: ActivityInstanceId,
        rng: &ActivityRngStreams,
    ) -> ActivityStateHash {
        let bytes = self.canonical_state_bytes(identity, graph, instance, rng);
        ActivityStateHash::new(Sha256::digest(bytes).into())
            .expect("Activity state hash accepts zero")
    }

    #[must_use]
    pub fn player_view(
        &self,
        identity: ActivityDefinitionIdentity,
        graph: &ActivityGraphDefinition,
        instance: ActivityInstanceId,
        rng: &ActivityRngStreams,
    ) -> ActivityPlayerView {
        let slots = self
            .definition
            .slots()
            .iter()
            .filter(|item| item.visibility() == ActivityStateVisibility::Player)
            .map(|item| ActivitySlotView {
                id: item.id(),
                value: self.slots[&item.id()].clone(),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let inventories = self
            .definition
            .inventories()
            .iter()
            .filter(|item| item.visibility() == ActivityStateVisibility::Player)
            .map(|item| inventory_view(item.id(), &self.inventories[&item.id()]))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        ActivityPlayerView {
            current_node: self.current_node,
            command_sequence: self.command_sequence,
            logical_scopes: self.logical_scopes.active().to_vec().into_boxed_slice(),
            slots,
            inventories,
            decision: self.pending.as_ref().map(decision_view),
            preparation: self.preparation_view(),
            pending_battle: self.pending_battle_view(),
            participant_carry: self.carry.view(),
            completed_battle_count: u32::try_from(self.completed_battles.len())
                .expect("completed battle count is bounded"),
            terminal: self.terminal,
            state_hash: self.state_hash(identity, graph, instance, rng),
        }
    }

    #[must_use]
    pub fn debug_view(
        &self,
        identity: ActivityDefinitionIdentity,
        graph: &ActivityGraphDefinition,
        instance: ActivityInstanceId,
        rng: &ActivityRngStreams,
    ) -> ActivityDebugView {
        let player = self.player_view(identity, graph, instance, rng);
        ActivityDebugView {
            player,
            all_slots: self
                .slots
                .iter()
                .map(|(id, value)| ActivitySlotView {
                    id: *id,
                    value: value.clone(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            all_inventories: self
                .inventories
                .iter()
                .map(|(id, entries)| inventory_view(*id, entries))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            modifiers: self
                .modifiers
                .iter()
                .map(|(id, stacks)| (*id, *stacks))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            node_visits: self
                .node_visits
                .iter()
                .map(|(id, count)| (*id, *count))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            edge_traversals: self
                .edge_traversals
                .iter()
                .map(|(id, count)| (*id, *count))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            logical_scopes: self.logical_scopes.active().to_vec().into_boxed_slice(),
            rng: rng.snapshots(),
        }
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
        if cause.program != program.id()
            || cause.node != self.current_node
            || cause.command_sequence != self.command_sequence.saturating_add(1)
        {
            return ActivityTransactionOutcome::Rejected(
                ActivityTransactionRejection::CauseMismatch,
            );
        }
        let mut working = self.transaction_copy();
        let mut events = Vec::new();
        match working.execute(program.operations(), cause, graph, &mut events) {
            Ok(()) => {
                working.command_sequence = cause.command_sequence;
                *self = working;
                ActivityTransactionOutcome::Committed(events.into_boxed_slice())
            }
            Err(ExecutionFailure::Rejected(error)) => ActivityTransactionOutcome::Rejected(error),
            Err(ExecutionFailure::Fault(fault)) => {
                let mut faulted = self.transaction_copy();
                faulted.command_sequence = cause.command_sequence;
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

    pub(crate) fn replace_pending_with_program(
        &mut self,
        program: &ActivityProgramDefinition,
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        decision::replace_pending_with_program(self, program, cause, graph)
    }

    pub fn apply_option(
        &mut self,
        option: ActivityOptionId,
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        self.apply_option_with_prefix(option, &[], cause, graph)
    }

    pub(crate) fn apply_option_with_prefix(
        &mut self,
        option: ActivityOptionId,
        prefix: &[crate::ActivityOperation],
        cause: ActivityCause,
        graph: &crate::ActivityGraphDefinition,
    ) -> ActivityTransactionOutcome {
        decision::apply_option_with_prefix(self, option, prefix, cause, graph)
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

    /// Creates the private working copy used by the mutation transaction.
    ///
    /// The authoritative state intentionally does not implement `Clone`: callers
    /// cannot fork a run by accidentally duplicating it outside the transaction
    /// boundary.
    pub(crate) fn transaction_copy(&self) -> Self {
        Self {
            definition: self.definition.clone(),
            slots: self.slots.clone(),
            inventories: self.inventories.clone(),
            modifiers: self.modifiers.clone(),
            current_node: self.current_node,
            command_sequence: self.command_sequence,
            node_visits: self.node_visits.clone(),
            edge_traversals: self.edge_traversals.clone(),
            total_visits: self.total_visits,
            logical_scopes: self.logical_scopes.clone(),
            attempt: self.attempt.clone(),
            awaiting_battle: self.awaiting_battle.clone(),
            carry: self.carry.clone(),
            completed_battles: self.completed_battles.clone(),
            pending: self.pending.clone(),
            terminal: self.terminal,
        }
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
            Op::InsertOrderedId { slot, id } => {
                self.insert_ordered_id(*slot, *id, cause, events)?
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
            Op::RestoreParticipant {
                participant,
                hp_ratio,
            } => {
                participant_carry::restore(&mut self.carry, *participant, *hp_ratio)?;
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::ParticipantCarryChanged(*participant),
                );
            }
            Op::HealParticipantMaximumHpRatio {
                participant,
                hp_ratio,
            } => {
                participant_carry::heal_maximum_hp_ratio(&mut self.carry, *participant, *hp_ratio)?;
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::ParticipantCarryChanged(*participant),
                );
            }
            Op::LoseParticipantCurrentHpRatio {
                participant,
                hp_ratio,
                minimum_hp,
            } => {
                participant_carry::lose_current_hp_ratio(
                    &mut self.carry,
                    *participant,
                    *hp_ratio,
                    *minimum_hp,
                )?;
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::ParticipantCarryChanged(*participant),
                );
            }
            Op::SetParticipantEnergy {
                participant,
                energy,
            } => {
                participant_carry::set_energy(&mut self.carry, *participant, *energy)?;
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::ParticipantCarryChanged(*participant),
                );
            }
            Op::Traverse(edge) => {
                let (_, resets) = self.traverse_edge(*edge, graph)?;
                for (slot, point) in resets {
                    push(
                        events,
                        cause,
                        ActivityTransactionEventKind::SlotReset { slot, point },
                    );
                }
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::EdgeTraversed(*edge),
                );
            }
            Op::Relocate(node) => {
                let resets = self.relocate_node(*node, graph)?;
                for (slot, point) in resets {
                    push(
                        events,
                        cause,
                        ActivityTransactionEventKind::SlotReset { slot, point },
                    );
                }
                push(
                    events,
                    cause,
                    ActivityTransactionEventKind::NodeRelocated(*node),
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
            Op::Conditional {
                condition,
                if_true,
                if_false,
            } => {
                let branch = if self.condition(condition)? {
                    if_true
                } else {
                    if_false
                };
                self.execute(branch, cause, graph, events)?;
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
            ActivityExpression::CounterValue { slot, key } => {
                if *key == 0 {
                    return Err(ActivityFault::TypeMismatch);
                }
                match self
                    .slots
                    .get(slot)
                    .ok_or(ActivityFault::MissingSlot(*slot))?
                {
                    ActivityValue::BoundedCounterMap(values) => Ok(ActivityValue::BoundedInteger(
                        values
                            .binary_search_by_key(key, |item| item.0)
                            .ok()
                            .map(|index| values[index].1)
                            .unwrap_or(0),
                    )),
                    _ => Err(ActivityFault::TypeMismatch),
                }
            }
            ActivityExpression::InventoryCount { inventory, content } => {
                if *content == 0 {
                    return Err(ActivityFault::TypeMismatch);
                }
                let values = self
                    .inventories
                    .get(inventory)
                    .ok_or(ActivityFault::MissingInventory(*inventory))?;
                Ok(ActivityValue::BoundedInteger(i64::from(
                    *values.get(content).unwrap_or(&0),
                )))
            }
            ActivityExpression::Add(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_add)
            }
            ActivityExpression::Subtract(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_sub)
            }
            ActivityExpression::Multiply(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, i64::checked_mul)
            }
            ActivityExpression::Divide(a, b) => {
                numeric_binary(self.evaluate(a)?, self.evaluate(b)?, |a, b| {
                    (b != 0).then(|| a / b)
                })
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

    pub(crate) fn consume_bounded_integer_slot(
        &mut self,
        id: ActivitySlotId,
        amount: u16,
    ) -> Result<(), ActivityFault> {
        let current = match self.slot(id) {
            Some(ActivityValue::BoundedInteger(value)) => *value,
            Some(_) => return Err(ActivityFault::TypeMismatch),
            None => return Err(ActivityFault::MissingSlot(id)),
        };
        let next = current
            .checked_sub(i64::from(amount))
            .ok_or(ActivityFault::ArithmeticOverflow)?;
        self.set_slot(id, ActivityValue::BoundedInteger(next))
    }

    pub(crate) fn settle_metric(
        &mut self,
        id: ActivitySlotId,
        value: ActivityValue,
        policy: MetricSettlementPolicy,
    ) -> Result<(), ActivityFault> {
        let next = match (self.slots.get(&id), value, policy) {
            (_, value, MetricSettlementPolicy::Replace) => value,
            (
                Some(ActivityValue::BoundedInteger(current)),
                ActivityValue::BoundedInteger(value),
                policy,
            ) => ActivityValue::BoundedInteger(settle_integer(*current, value, policy)?),
            (
                Some(ActivityValue::FixedScalar(current)),
                ActivityValue::FixedScalar(value),
                policy,
            ) => ActivityValue::FixedScalar(settle_integer(*current, value, policy)?),
            _ => return Err(ActivityFault::TypeMismatch),
        };
        self.set_slot(id, next)
    }

    pub(crate) fn settle_terminal(&mut self, outcome: ActivityTerminalOutcome) {
        self.terminal = Some(outcome);
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
