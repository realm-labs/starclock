use std::sync::Arc;

use crate::{
    ActivityBattleHandoff, ActivityBattlePreparationRequest, ActivityBattleResultContract,
    ActivityBattleResultSubmission, ActivityBattleSettlement, ActivityBattleSettlementError,
    ActivityBattleStartRequest, ActivityBootstrapSelection, ActivityCause, ActivityDebugView,
    ActivityDecisionId, ActivityDecisionKind, ActivityDefinitionIdentity, ActivityExpression,
    ActivityExternalOutcomeId, ActivityGraphDefinition, ActivityHandlerInput, ActivityInstanceId,
    ActivityInteractionBinding, ActivityInteractionBindings, ActivityMasterSeed, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityPendingBattleView, ActivityPlayerView,
    ActivityPreparationBoundary, ActivityPreparationView, ActivityProgramDefinition,
    ActivityRandomCheckpoint, ActivityRandomOffer, ActivityRandomPolicies, ActivityRngContext,
    ActivityRngStreams, ActivityStateDefinition, ActivityStateHash, ActivityTransactionEvent,
    ActivityTransactionOutcome, ActivityTransactionState, ActivityValue, BattleResult,
    GraphActivityBattleError, GraphActivityCommandError, GraphActivityDefinitionError,
    GraphActivityEncounterError, GraphActivityRandomOfferError, GraphActivityRuntimeError,
    GraphActivityStartError, NodeId, ParticipantLock, PendingBattleSpec, SlotValueKind,
};

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
    graph: Arc<ActivityGraphDefinition>,
    state: ActivityStateDefinition,
    participants: Arc<ParticipantLock>,
    programs: Arc<[GraphActivityNodeProgram]>,
    bootstrap: Option<ActivityBootstrapSelection>,
    random_checkpoints: Arc<[ActivityRandomCheckpoint]>,
    random_offers: Arc<[ActivityRandomOffer]>,
    interactions: Option<Arc<ActivityInteractionBindings>>,
}

impl GraphActivityDefinition {
    pub fn new(
        identity: ActivityDefinitionIdentity,
        graph: ActivityGraphDefinition,
        state: ActivityStateDefinition,
        participants: Arc<ParticipantLock>,
        mut programs: Vec<GraphActivityNodeProgram>,
        bootstrap: Option<ActivityBootstrapSelection>,
        random_policies: ActivityRandomPolicies,
    ) -> Result<Self, GraphActivityDefinitionError> {
        let ActivityRandomPolicies {
            mut checkpoints,
            mut offers,
        } = random_policies;
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
        state
            .logical_scopes()
            .validate_graph(&graph)
            .map_err(|_| GraphActivityDefinitionError::InvalidLogicalScopes)?;
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
        checkpoints.sort_by_key(ActivityRandomCheckpoint::node);
        if checkpoints
            .windows(2)
            .any(|pair| pair[0].node == pair[1].node)
        {
            return Err(GraphActivityDefinitionError::DuplicateRandomCheckpoint);
        }
        for checkpoint in &checkpoints {
            let binding = programs
                .binary_search_by_key(&checkpoint.node, GraphActivityNodeProgram::node)
                .ok()
                .map(|index| &programs[index])
                .ok_or(GraphActivityDefinitionError::InvalidRandomCheckpoint)?;
            let offered = checkpoint_options(binding.program.operations())
                .ok_or(GraphActivityDefinitionError::InvalidRandomCheckpoint)?;
            if checkpoint
                .weights
                .iter()
                .any(|(option, _)| !offered.contains(option))
            {
                return Err(GraphActivityDefinitionError::InvalidRandomCheckpoint);
            }
        }
        offers.sort_by_key(ActivityRandomOffer::node);
        if offers.windows(2).any(|pair| pair[0].node == pair[1].node) {
            return Err(GraphActivityDefinitionError::DuplicateRandomOffer);
        }
        for offer in &offers {
            let binding = programs
                .binary_search_by_key(&offer.node, GraphActivityNodeProgram::node)
                .ok()
                .map(|index| &programs[index])
                .ok_or(GraphActivityDefinitionError::InvalidRandomOffer)?;
            let offered = player_offer_options(binding.program.operations())
                .ok_or(GraphActivityDefinitionError::InvalidRandomOffer)?;
            if offered.len() != offer.weights.len()
                || offered.iter().any(|option| {
                    offer
                        .weights
                        .binary_search_by_key(option, |item| item.0)
                        .is_err()
                })
            {
                return Err(GraphActivityDefinitionError::InvalidRandomOffer);
            }
            if let Some((slot, _)) = offer.reroll_counter {
                let valid = state.slots().iter().any(|definition| {
                    definition.id() == slot && definition.kind() == SlotValueKind::BoundedCounterMap
                });
                if !valid {
                    return Err(GraphActivityDefinitionError::InvalidRandomOffer);
                }
            }
        }
        Ok(Self {
            identity,
            graph: Arc::new(graph),
            state,
            participants,
            programs: programs.into(),
            bootstrap,
            random_checkpoints: checkpoints.into(),
            random_offers: offers.into(),
            interactions: None,
        })
    }

    pub fn with_interactions(
        mut self,
        registry: crate::ActivityHandlerRegistry,
        bindings: Vec<ActivityInteractionBinding>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        self.interactions = Some(Arc::new(
            ActivityInteractionBindings::new(registry, bindings, &self.graph, &self.programs)
                .map_err(GraphActivityDefinitionError::InvalidInteractionBindings)?,
        ));
        Ok(self)
    }

    pub fn rebind(
        &self,
        identity: ActivityDefinitionIdentity,
        state: ActivityStateDefinition,
        participants: Arc<ParticipantLock>,
    ) -> Result<Self, GraphActivityDefinitionError> {
        if !compatible_state_shape(&self.state, &state) {
            return Err(GraphActivityDefinitionError::IncompatibleStateShape);
        }
        Ok(Self {
            identity,
            graph: Arc::clone(&self.graph),
            state,
            participants,
            programs: Arc::clone(&self.programs),
            bootstrap: self.bootstrap.clone(),
            random_checkpoints: Arc::clone(&self.random_checkpoints),
            random_offers: Arc::clone(&self.random_offers),
            interactions: self.interactions.clone(),
        })
    }

    #[must_use]
    pub const fn identity(&self) -> ActivityDefinitionIdentity {
        self.identity
    }
    #[must_use]
    pub fn graph(&self) -> &ActivityGraphDefinition {
        self.graph.as_ref()
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

    #[must_use]
    pub fn random_checkpoints(&self) -> &[ActivityRandomCheckpoint] {
        &self.random_checkpoints
    }

    #[must_use]
    pub fn random_offers(&self) -> &[ActivityRandomOffer] {
        &self.random_offers
    }

    #[must_use]
    pub fn interactions(&self) -> Option<&ActivityInteractionBindings> {
        self.interactions.as_deref()
    }

    fn program(&self, node: NodeId) -> Option<&ActivityProgramDefinition> {
        self.programs
            .binary_search_by_key(&node, GraphActivityNodeProgram::node)
            .ok()
            .map(|index| &self.programs[index].program)
    }

    fn random_checkpoint(&self, node: NodeId) -> Option<&ActivityRandomCheckpoint> {
        self.random_checkpoints
            .binary_search_by_key(&node, ActivityRandomCheckpoint::node)
            .ok()
            .map(|index| &self.random_checkpoints[index])
    }

    fn random_offer(&self, node: NodeId) -> Option<&ActivityRandomOffer> {
        self.random_offers
            .binary_search_by_key(&node, ActivityRandomOffer::node)
            .ok()
            .map(|index| &self.random_offers[index])
    }
}

fn compatible_state_shape(
    expected: &ActivityStateDefinition,
    actual: &ActivityStateDefinition,
) -> bool {
    expected.slots().len() == actual.slots().len()
        && expected
            .slots()
            .iter()
            .zip(actual.slots())
            .all(|(left, right)| left.id() == right.id() && left.kind() == right.kind())
        && expected.inventories().len() == actual.inventories().len()
        && expected
            .inventories()
            .iter()
            .zip(actual.inventories())
            .all(|(left, right)| left.id() == right.id())
        && expected.modifiers().len() == actual.modifiers().len()
        && expected
            .modifiers()
            .iter()
            .zip(actual.modifiers())
            .all(|(left, right)| left.id() == right.id())
        && expected.logical_scopes() == actual.logical_scopes()
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
    pub const fn instance(&self) -> ActivityInstanceId {
        self.instance
    }
    #[must_use]
    pub const fn current_node(&self) -> NodeId {
        self.state.current_node()
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
        if offered.kind() == ActivityDecisionKind::ExternalOutcome {
            return Err(GraphActivityCommandError::InteractionNotBound);
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

    /// Accepts an externally resolved, non-spatial interaction outcome through
    /// the same checked option transaction used by ordinary decisions.
    pub fn submit_external_outcome(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        outcome: ActivityExternalOutcomeId,
    ) -> Result<Box<[ActivityTransactionEvent]>, GraphActivityCommandError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let view = self.player_view();
        let offered = view
            .decision()
            .filter(|offered| {
                offered.id() == decision && offered.kind() == ActivityDecisionKind::ExternalOutcome
            })
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        let option = ActivityOptionId::new(outcome.get())
            .expect("external outcome and option IDs share the same non-zero width");
        if !offered
            .options()
            .iter()
            .any(|candidate| candidate.id() == option)
        {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        let interactions = self
            .definition
            .interactions()
            .ok_or(GraphActivityCommandError::InteractionNotBound)?;
        let binding = interactions
            .binding(self.state.current_node(), outcome)
            .ok_or(GraphActivityCommandError::InteractionNotBound)?;
        let registration = interactions
            .registry()
            .handler(binding.handler())
            .ok_or(GraphActivityCommandError::HandlerUnavailable)?;
        let mut working_rng = self.rng.transaction_copy();
        let random_index = binding
            .random_policy()
            .map(|policy| {
                working_rng
                    .choose_index(policy.label(), policy.purpose(), policy.candidate_count())
                    .map(|draw| {
                        draw.expect("non-zero random candidate count")
                            .value()
                            .try_into()
                            .expect("candidate count is bounded to u32")
                    })
            })
            .transpose()
            .map_err(GraphActivityCommandError::Rng)?;
        let output = registration
            .execute(
                ActivityHandlerInput::new(&view, binding.payload())
                    .map_err(|fault| GraphActivityCommandError::HandlerFault(fault.kind()))?
                    .with_random_index(random_index),
            )
            .map_err(|fault| GraphActivityCommandError::HandlerFault(fault.kind()))?;
        let program = self
            .definition
            .program(self.state.current_node())
            .expect("pending decision came from the current node");
        let selected_operations = program
            .operations()
            .iter()
            .find_map(|operation| match operation {
                ActivityOperation::Offer { options, .. } => options
                    .iter()
                    .find(|candidate| candidate.id() == option)
                    .map(|candidate| candidate.operations().len()),
                _ => None,
            })
            .expect("validated offered option exists in its source program");
        if selected_operations
            .checked_add(output.operations().len())
            .is_none_or(|count| count > crate::MAX_ACTIVITY_PROGRAM_OPERATIONS)
        {
            return Err(GraphActivityCommandError::InteractionOperationLimit);
        }
        let cause = ActivityCause::new(
            self.state.command_sequence().saturating_add(1),
            program.id(),
            self.state.current_node(),
        )
        .expect("next command sequence is non-zero");
        let mut working_state = self.state.transaction_copy();
        let mut events = match working_state.apply_option_with_prefix(
            option,
            output.operations(),
            cause,
            &self.definition.graph,
        ) {
            ActivityTransactionOutcome::Committed(events) => events.into_vec(),
            ActivityTransactionOutcome::Rejected(error) => {
                return Err(GraphActivityCommandError::Rejected(error));
            }
            ActivityTransactionOutcome::Faulted(_, fault) => {
                return Err(GraphActivityCommandError::InteractionFault(fault));
            }
        };
        self.state = working_state;
        self.rng = working_rng;
        events.extend(self.pump().map_err(GraphActivityCommandError::Runtime)?);
        Ok(events.into_boxed_slice())
    }

    pub fn reroll_random_offer(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<Box<[ActivityTransactionEvent]>, GraphActivityRandomOfferError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityRandomOfferError::StaleStateHash);
        }
        let node = self.state.current_node();
        let policy = self
            .definition
            .random_offer(node)
            .ok_or(GraphActivityRandomOfferError::NotOffered)?;
        if self.state.pending_option_ids().is_none() {
            return Err(GraphActivityRandomOfferError::NotOffered);
        }
        let (counter, maximum) = policy
            .reroll_counter
            .ok_or(GraphActivityRandomOfferError::RerollDisabled)?;
        let current = self
            .state
            .counter_value(counter, u64::from(node.get()))
            .ok_or(GraphActivityRandomOfferError::InvalidCounter)?;
        if current >= i64::from(maximum) {
            return Err(GraphActivityRandomOfferError::RerollLimitReached);
        }
        let source = self
            .definition
            .program(node)
            .ok_or(GraphActivityRandomOfferError::NotOffered)?;
        let mut operations = Vec::with_capacity(source.operations().len() + 1);
        operations.push(ActivityOperation::AddCounter {
            slot: counter,
            key: u64::from(node.get()),
            delta: ActivityExpression::Literal(ActivityValue::BoundedInteger(1)),
        });
        operations.extend_from_slice(source.operations());
        let program = ActivityProgramDefinition::new(source.id(), operations)
            .map_err(|_| GraphActivityRandomOfferError::InvalidProgram)?;
        let cause = ActivityCause::new(
            self.state.command_sequence().saturating_add(1),
            program.id(),
            node,
        )
        .ok_or(GraphActivityRandomOfferError::InvalidProgram)?;
        let mut working_state = self.state.transaction_copy();
        let mut working_rng = self.rng.transaction_copy();
        let events = match working_state.replace_pending_with_program(
            &program,
            cause,
            &self.definition.graph,
        ) {
            ActivityTransactionOutcome::Committed(events) => events,
            ActivityTransactionOutcome::Rejected(error) => {
                return Err(GraphActivityRandomOfferError::Rejected(error));
            }
            ActivityTransactionOutcome::Faulted(_, fault) => {
                return Err(GraphActivityRandomOfferError::Faulted(fault));
            }
        };
        restrict_random_offer(&mut working_state, &mut working_rng, policy)
            .map_err(GraphActivityRandomOfferError::Runtime)?;
        self.state = working_state;
        self.rng = working_rng;
        Ok(events)
    }

    pub fn engage_encounter(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
        request: ActivityBattlePreparationRequest,
    ) -> Result<GraphActivityPreparationResolution, GraphActivityEncounterError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityEncounterError::StaleStateHash);
        }
        let view = self.player_view();
        let offered = view
            .decision()
            .ok_or(GraphActivityEncounterError::DecisionNotOffered)?;
        if offered.id() != decision || offered.kind() != ActivityDecisionKind::Encounter {
            return Err(GraphActivityEncounterError::DecisionNotOffered);
        }
        let program = self.definition.program(self.state.current_node()).ok_or(
            GraphActivityEncounterError::Runtime(GraphActivityRuntimeError::MissingNodeProgram),
        )?;
        let cause = ActivityCause::new(
            self.state.command_sequence().saturating_add(1),
            program.id(),
            self.state.current_node(),
        )
        .ok_or(GraphActivityEncounterError::Runtime(
            GraphActivityRuntimeError::InvalidCause,
        ))?;
        let mut working = self.state.transaction_copy();
        let events = match working.apply_option(option, cause, &self.definition.graph) {
            ActivityTransactionOutcome::Committed(events) => events,
            ActivityTransactionOutcome::Rejected(error) => {
                return Err(GraphActivityEncounterError::Rejected(error));
            }
            ActivityTransactionOutcome::Faulted(_, fault) => {
                return Err(GraphActivityEncounterError::Faulted(fault));
            }
        };
        let boundary = working
            .begin_battle_preparation(self.instance, &self.definition.graph, request)
            .map_err(GraphActivityEncounterError::Preparation)?;
        self.state = working;
        Ok(GraphActivityPreparationResolution {
            boundary,
            events,
            state_hash: self.state_hash(),
        })
    }

    pub fn choose_preparation_option(
        &mut self,
        expected_state_hash: ActivityStateHash,
        option: ActivityOptionId,
    ) -> Result<ActivityPreparationBoundary, GraphActivityEncounterError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityEncounterError::StaleStateHash);
        }
        let mut working = self.state.transaction_copy();
        let boundary = working
            .choose_preparation_option(option)
            .map_err(GraphActivityEncounterError::Preparation)?;
        self.state = working;
        Ok(boundary)
    }

    #[must_use]
    pub fn preparation_view(&self) -> Option<ActivityPreparationView> {
        self.state.preparation_view()
    }

    #[must_use]
    pub fn pending_battle(&self) -> Option<&PendingBattleSpec> {
        self.state.pending_battle()
    }

    #[must_use]
    pub fn pending_battle_view(&self) -> Option<ActivityPendingBattleView> {
        self.state.pending_battle_view()
    }

    pub fn start_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
        contract: Arc<ActivityBattleResultContract>,
    ) -> Result<ActivityBattleHandoff, ActivityBattleSettlementError> {
        self.state.start_pending_battle(
            &self.definition.graph,
            &self.rng,
            ActivityBattleStartRequest::new(
                expected_state_hash,
                self.definition.identity,
                self.instance,
                contract,
            ),
        )
    }

    pub fn submit_pending_battle_result(
        &mut self,
        expected_state_hash: ActivityStateHash,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        let settlement = self
            .state
            .submit_pending_battle_result(
                self.definition.identity,
                &self.definition.graph,
                self.instance,
                &self.rng,
                ActivityBattleResultSubmission::new(expected_state_hash, result),
            )
            .map_err(GraphActivityBattleError::Settlement)?;
        let events = self.pump().map_err(GraphActivityBattleError::Runtime)?;
        Ok(GraphActivityBattleResolution {
            settlement,
            events: events.into_boxed_slice(),
            state_hash: self.state_hash(),
        })
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
                if decision.kind() != ActivityDecisionKind::Checkpoint {
                    return Ok(events);
                }
                let option = if let Some(policy) =
                    self.definition.random_checkpoint(self.state.current_node())
                {
                    let mut weights = Vec::with_capacity(decision.options().len());
                    for offered in decision.options() {
                        let weight = policy
                            .weights
                            .binary_search_by_key(&offered.id(), |item| item.0)
                            .ok()
                            .map(|index| policy.weights[index].1)
                            .ok_or(GraphActivityRuntimeError::InvalidRandomCheckpoint)?;
                        weights.push(weight);
                    }
                    let (index, _) = self
                        .rng
                        .choose_weighted(policy.label, policy.purpose, &weights)
                        .map_err(GraphActivityRuntimeError::Rng)?
                        .ok_or(GraphActivityRuntimeError::InvalidRandomCheckpoint)?;
                    decision.options()[index as usize].id()
                } else if decision.options().len() == 1 {
                    decision.options()[0].id()
                } else {
                    return Err(GraphActivityRuntimeError::InvalidRandomCheckpoint);
                };
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
                    option,
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
            if let Some(policy) = self.definition.random_offer(node).cloned() {
                let mut working_state = self.state.transaction_copy();
                let mut working_rng = self.rng.transaction_copy();
                match working_state.apply_program(&program, cause, &self.definition.graph) {
                    ActivityTransactionOutcome::Committed(committed_events) => {
                        restrict_random_offer(&mut working_state, &mut working_rng, &policy)?;
                        self.state = working_state;
                        self.rng = working_rng;
                        events.extend(committed_events);
                    }
                    ActivityTransactionOutcome::Faulted(fault_events, _) => {
                        self.state = working_state;
                        events.extend(fault_events);
                    }
                    ActivityTransactionOutcome::Rejected(error) => {
                        return Err(GraphActivityRuntimeError::Rejected(error));
                    }
                }
            } else {
                events.extend(committed_runtime(self.state.apply_program(
                    &program,
                    cause,
                    &self.definition.graph,
                ))?);
            }
        }
        Err(GraphActivityRuntimeError::AutomaticStepLimit)
    }
}

fn checkpoint_options(operations: &[crate::ActivityOperation]) -> Option<Vec<ActivityOptionId>> {
    operations.iter().find_map(|operation| match operation {
        crate::ActivityOperation::Offer { kind, options }
            if *kind == ActivityDecisionKind::Checkpoint =>
        {
            Some(options.iter().map(ActivityOptionDefinition::id).collect())
        }
        _ => None,
    })
}

fn player_offer_options(operations: &[ActivityOperation]) -> Option<Vec<ActivityOptionId>> {
    if operations.len() != 1 {
        return None;
    }
    match &operations[0] {
        ActivityOperation::Offer { kind, options } if *kind != ActivityDecisionKind::Checkpoint => {
            Some(options.iter().map(ActivityOptionDefinition::id).collect())
        }
        _ => None,
    }
}

fn restrict_random_offer(
    state: &mut ActivityTransactionState,
    rng: &mut ActivityRngStreams,
    policy: &ActivityRandomOffer,
) -> Result<(), GraphActivityRuntimeError> {
    let offered = state
        .pending_option_ids()
        .ok_or(GraphActivityRuntimeError::InvalidRandomOffer)?;
    let mut weights = Vec::with_capacity(offered.len());
    for option in &offered {
        let weight = policy
            .weights
            .binary_search_by_key(option, |item| item.0)
            .ok()
            .map(|index| policy.weights[index].1)
            .ok_or(GraphActivityRuntimeError::InvalidRandomOffer)?;
        weights.push(weight);
    }
    let selected = rng
        .choose_weighted_without_replacement(
            policy.label,
            policy.purpose,
            &weights,
            policy.maximum_options,
        )
        .map_err(GraphActivityRuntimeError::Rng)?;
    let ids = selected
        .iter()
        .map(|index| offered[*index as usize])
        .collect::<Vec<_>>();
    state
        .restrict_pending_options(ids)
        .map_err(|_| GraphActivityRuntimeError::InvalidRandomOffer)
}

pub struct GraphActivityResolution {
    activity: GraphActivity,
    events: Box<[ActivityTransactionEvent]>,
}

pub struct GraphActivityPreparationResolution {
    boundary: ActivityPreparationBoundary,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl GraphActivityPreparationResolution {
    #[must_use]
    pub const fn boundary(&self) -> ActivityPreparationBoundary {
        self.boundary
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

pub struct GraphActivityBattleResolution {
    settlement: ActivityBattleSettlement,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl GraphActivityBattleResolution {
    #[must_use]
    pub const fn settlement(&self) -> ActivityBattleSettlement {
        self.settlement
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
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
