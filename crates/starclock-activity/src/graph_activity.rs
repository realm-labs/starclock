use std::sync::Arc;

use crate::{
    ActivityCause, ActivityDebugView, ActivityDecisionId, ActivityDecisionKind,
    ActivityDefinitionIdentity, ActivityFault, ActivityGraphDefinition, ActivityInstanceId,
    ActivityMasterSeed, ActivityOptionId, ActivityPlayerView, ActivityProgramDefinition,
    ActivityRngContext, ActivityRngError, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityStateDefinition, ActivityStateHash, ActivityTransactionEvent,
    ActivityTransactionOutcome, ActivityTransactionRejection, ActivityTransactionState,
    ActivityValue, NodeId, ParticipantLock, SlotValueKind,
};

/// One deterministic bootstrap draw applied before the entry-node program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActivityBootstrapSelection {
    slot: ActivitySlotId,
    label: ActivityRngLabel,
    purpose: u16,
    candidates: Box<[u64]>,
}

impl ActivityBootstrapSelection {
    pub fn new(
        slot: ActivitySlotId,
        label: ActivityRngLabel,
        purpose: u16,
        candidates: Vec<u64>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        if purpose == 0
            || candidates.is_empty()
            || candidates.len() > 256
            || candidates.contains(&0)
            || candidates.windows(2).any(|pair| pair[0] >= pair[1])
        {
            return Err(GraphActivityDefinitionError::InvalidBootstrapSelection);
        }
        Ok(Self {
            slot,
            label,
            purpose,
            candidates: candidates.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn slot(&self) -> ActivitySlotId {
        self.slot
    }
    #[must_use]
    pub const fn label(&self) -> ActivityRngLabel {
        self.label
    }
    #[must_use]
    pub const fn purpose(&self) -> u16 {
        self.purpose
    }
    #[must_use]
    pub fn candidates(&self) -> &[u64] {
        &self.candidates
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphActivityNodeProgram {
    node: NodeId,
    program: ActivityProgramDefinition,
}

impl GraphActivityNodeProgram {
    #[must_use]
    pub const fn new(node: NodeId, program: ActivityProgramDefinition) -> Self {
        Self { node, program }
    }
    #[must_use]
    pub const fn node(&self) -> NodeId {
        self.node
    }
    #[must_use]
    pub const fn program(&self) -> &ActivityProgramDefinition {
        &self.program
    }
}

/// Immutable generic graph-Activity definition shared by isolated runs.
#[derive(Clone, Debug)]
pub struct GraphActivityDefinition {
    identity: ActivityDefinitionIdentity,
    graph: ActivityGraphDefinition,
    state: ActivityStateDefinition,
    participants: Arc<ParticipantLock>,
    programs: Box<[GraphActivityNodeProgram]>,
    bootstrap: Option<ActivityBootstrapSelection>,
}

impl GraphActivityDefinition {
    pub fn new(
        identity: ActivityDefinitionIdentity,
        graph: ActivityGraphDefinition,
        state: ActivityStateDefinition,
        participants: Arc<ParticipantLock>,
        mut programs: Vec<GraphActivityNodeProgram>,
        bootstrap: Option<ActivityBootstrapSelection>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        programs.sort_by_key(GraphActivityNodeProgram::node);
        if programs.windows(2).any(|pair| pair[0].node == pair[1].node) {
            return Err(GraphActivityDefinitionError::DuplicateNodeProgram);
        }
        for node in graph.nodes() {
            let program = programs
                .binary_search_by_key(&node.id(), GraphActivityNodeProgram::node)
                .ok()
                .map(|index| &programs[index]);
            if node.kind().terminal().is_none() && program.is_none() {
                return Err(GraphActivityDefinitionError::MissingNodeProgram(node.id()));
            }
            if node.kind().terminal().is_some() && program.is_some() {
                return Err(GraphActivityDefinitionError::TerminalNodeProgram(node.id()));
            }
        }
        for binding in &programs {
            binding
                .program
                .validate_against(&state, &graph)
                .map_err(|_| GraphActivityDefinitionError::InvalidProgramBinding(binding.node))?;
            validate_edge_ownership(binding.node, binding.program.operations(), &graph)?;
        }
        if let Some(selection) = &bootstrap {
            let slot = state
                .slots()
                .iter()
                .find(|definition| definition.id() == selection.slot)
                .ok_or(GraphActivityDefinitionError::InvalidBootstrapSlot)?;
            if !matches!(
                slot.kind(),
                SlotValueKind::StableId | SlotValueKind::OptionalId
            ) {
                return Err(GraphActivityDefinitionError::InvalidBootstrapSlot);
            }
        }
        Ok(Self {
            identity,
            graph,
            state,
            participants,
            programs: programs.into_boxed_slice(),
            bootstrap,
        })
    }

    #[must_use]
    pub const fn identity(&self) -> ActivityDefinitionIdentity {
        self.identity
    }
    #[must_use]
    pub const fn graph(&self) -> &ActivityGraphDefinition {
        &self.graph
    }
    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }
    #[must_use]
    pub const fn participants(&self) -> &Arc<ParticipantLock> {
        &self.participants
    }
    #[must_use]
    pub fn programs(&self) -> &[GraphActivityNodeProgram] {
        &self.programs
    }
    #[must_use]
    pub const fn bootstrap(&self) -> Option<&ActivityBootstrapSelection> {
        self.bootstrap.as_ref()
    }

    fn program(&self, node: NodeId) -> Option<&ActivityProgramDefinition> {
        self.programs
            .binary_search_by_key(&node, GraphActivityNodeProgram::node)
            .ok()
            .map(|index| &self.programs[index].program)
    }
}

/// Mutable generic graph execution. Mode crates only compile definitions.
#[derive(Debug)]
pub struct GraphActivity {
    definition: Arc<GraphActivityDefinition>,
    instance: ActivityInstanceId,
    rng: ActivityRngStreams,
    state: ActivityTransactionState,
}

impl GraphActivity {
    pub fn start(
        definition: Arc<GraphActivityDefinition>,
        instance: ActivityInstanceId,
        master_seed: ActivityMasterSeed,
    ) -> Result<GraphActivityResolution, GraphActivityStartError> {
        let entry = definition.graph.entry();
        let section = definition
            .graph
            .node(entry)
            .expect("validated graph contains entry")
            .section();
        let context = ActivityRngContext::new(
            master_seed,
            definition.identity.id(),
            definition.identity.definition_digest(),
            definition.identity.config_digest(),
            definition.graph.digest(),
            instance,
            Some(section),
            Some(entry),
            None,
            0,
        );
        let mut rng = ActivityRngStreams::new(context);
        let mut overrides = Vec::new();
        if let Some(selection) = &definition.bootstrap {
            let draw = rng
                .choose_index(
                    selection.label,
                    selection.purpose,
                    selection.candidates.len() as u32,
                )
                .map_err(GraphActivityStartError::Rng)?
                .expect("non-empty candidates produce a draw");
            let value = selection.candidates[draw.value() as usize];
            let kind = definition
                .state
                .slots()
                .iter()
                .find(|slot| slot.id() == selection.slot)
                .expect("bootstrap slot validated")
                .kind();
            overrides.push((
                selection.slot,
                if kind == SlotValueKind::OptionalId {
                    ActivityValue::OptionalId(Some(value))
                } else {
                    ActivityValue::StableId(value)
                },
            ));
        }
        let state = ActivityTransactionState::new_with_initial_values(
            definition.state.clone(),
            entry,
            overrides,
        )
        .map_err(GraphActivityStartError::State)?;
        let mut activity = Self {
            definition,
            instance,
            rng,
            state,
        };
        let events = activity.pump().map_err(GraphActivityStartError::Runtime)?;
        Ok(GraphActivityResolution {
            activity,
            events: events.into_boxed_slice(),
        })
    }

    #[must_use]
    pub const fn definition(&self) -> &Arc<GraphActivityDefinition> {
        &self.definition
    }
    #[must_use]
    pub fn state_hash(&self) -> ActivityStateHash {
        self.state.state_hash(
            self.definition.identity,
            &self.definition.graph,
            self.instance,
            &self.rng,
        )
    }
    #[must_use]
    pub fn canonical_state_bytes(&self) -> Box<[u8]> {
        self.state.canonical_state_bytes(
            self.definition.identity,
            &self.definition.graph,
            self.instance,
            &self.rng,
        )
    }
    #[must_use]
    pub fn player_view(&self) -> ActivityPlayerView {
        self.state.player_view(
            self.definition.identity,
            &self.definition.graph,
            self.instance,
            &self.rng,
        )
    }
    #[must_use]
    pub fn debug_view(&self) -> ActivityDebugView {
        self.state.debug_view(
            self.definition.identity,
            &self.definition.graph,
            self.instance,
            &self.rng,
        )
    }

    pub fn choose_option(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<Box<[ActivityTransactionEvent]>, GraphActivityCommandError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let view = self.player_view();
        let offered = view
            .decision()
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        if offered.id() != decision {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        let program = self
            .definition
            .program(self.state.current_node())
            .expect("pending decision came from the current node");
        let cause = ActivityCause::new(
            self.state.command_sequence().saturating_add(1),
            program.id(),
            self.state.current_node(),
        )
        .expect("next command sequence is non-zero");
        let mut events = committed(
            self.state
                .apply_option(option, cause, &self.definition.graph),
        )?;
        events.extend(self.pump().map_err(GraphActivityCommandError::Runtime)?);
        Ok(events.into_boxed_slice())
    }

    fn pump(&mut self) -> Result<Vec<ActivityTransactionEvent>, GraphActivityRuntimeError> {
        let mut events = Vec::new();
        let maximum_steps = usize::try_from(self.definition.graph.maximum_total_visits())
            .unwrap_or(usize::MAX)
            .saturating_mul(3);
        for _ in 0..maximum_steps {
            if self.state.terminal().is_some() {
                return Ok(events);
            }
            let view = self.player_view();
            if let Some(decision) = view.decision() {
                if decision.kind() != ActivityDecisionKind::Checkpoint
                    || decision.options().len() != 1
                {
                    return Ok(events);
                }
                let program = self
                    .definition
                    .program(self.state.current_node())
                    .ok_or(GraphActivityRuntimeError::MissingNodeProgram)?;
                let cause = ActivityCause::new(
                    self.state.command_sequence().saturating_add(1),
                    program.id(),
                    self.state.current_node(),
                )
                .ok_or(GraphActivityRuntimeError::InvalidCause)?;
                events.extend(committed_runtime(self.state.apply_option(
                    decision.options()[0].id(),
                    cause,
                    &self.definition.graph,
                ))?);
                continue;
            }
            let node = self.state.current_node();
            let definition = self
                .definition
                .graph
                .node(node)
                .ok_or(GraphActivityRuntimeError::MissingNodeProgram)?;
            if let Some(terminal) = definition.kind().terminal() {
                self.state.settle_terminal(terminal);
                return Ok(events);
            }
            let program = self
                .definition
                .program(node)
                .ok_or(GraphActivityRuntimeError::MissingNodeProgram)?
                .clone();
            let cause = ActivityCause::new(
                self.state.command_sequence().saturating_add(1),
                program.id(),
                node,
            )
            .ok_or(GraphActivityRuntimeError::InvalidCause)?;
            events.extend(committed_runtime(self.state.apply_program(
                &program,
                cause,
                &self.definition.graph,
            ))?);
        }
        Err(GraphActivityRuntimeError::AutomaticStepLimit)
    }
}

pub struct GraphActivityResolution {
    activity: GraphActivity,
    events: Box<[ActivityTransactionEvent]>,
}

impl GraphActivityResolution {
    #[must_use]
    pub fn into_activity(self) -> GraphActivity {
        self.activity
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub fn view(&self) -> ActivityPlayerView {
        self.activity.player_view()
    }
}

fn validate_edge_ownership(
    node: NodeId,
    operations: &[crate::ActivityOperation],
    graph: &ActivityGraphDefinition,
) -> Result<(), GraphActivityDefinitionError> {
    for operation in operations {
        match operation {
            crate::ActivityOperation::Traverse(edge) => {
                if !graph
                    .edges()
                    .iter()
                    .any(|item| item.id() == *edge && item.from() == node)
                {
                    return Err(GraphActivityDefinitionError::InvalidProgramBinding(node));
                }
            }
            crate::ActivityOperation::Offer { options, .. } => {
                for option in options.iter() {
                    validate_edge_ownership(node, option.operations(), graph)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn committed(
    outcome: ActivityTransactionOutcome,
) -> Result<Vec<ActivityTransactionEvent>, GraphActivityCommandError> {
    match outcome {
        ActivityTransactionOutcome::Committed(events)
        | ActivityTransactionOutcome::Faulted(events, _) => Ok(events.into_vec()),
        ActivityTransactionOutcome::Rejected(error) => {
            Err(GraphActivityCommandError::Rejected(error))
        }
    }
}

fn committed_runtime(
    outcome: ActivityTransactionOutcome,
) -> Result<Vec<ActivityTransactionEvent>, GraphActivityRuntimeError> {
    match outcome {
        ActivityTransactionOutcome::Committed(events)
        | ActivityTransactionOutcome::Faulted(events, _) => Ok(events.into_vec()),
        ActivityTransactionOutcome::Rejected(error) => {
            Err(GraphActivityRuntimeError::Rejected(error))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityDefinitionError {
    InvalidBootstrapSelection,
    InvalidBootstrapSlot,
    DuplicateNodeProgram,
    MissingNodeProgram(NodeId),
    TerminalNodeProgram(NodeId),
    InvalidProgramBinding(NodeId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityStartError {
    Rng(ActivityRngError),
    State(ActivityFault),
    Runtime(GraphActivityRuntimeError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityRuntimeError {
    MissingNodeProgram,
    InvalidCause,
    Rejected(ActivityTransactionRejection),
    AutomaticStepLimit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphActivityCommandError {
    StaleStateHash,
    DecisionNotOffered,
    Rejected(ActivityTransactionRejection),
    Runtime(GraphActivityRuntimeError),
}
