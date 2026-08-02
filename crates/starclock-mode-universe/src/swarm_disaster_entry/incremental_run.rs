//! Resumable external-control adapter over the canonical Swarm run executor.

use std::collections::BTreeSet;

use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityDefinitionIdentity, ActivityInstanceId,
    ActivityMasterSeed, ActivityPlayerView, ActivityRngContext, ActivityRngStreams,
    ActivityStateHash, ActivityTerminalOutcome, ActivityTransactionState, AttemptId,
    BattleSequence, GraphActivityCommand, GraphActivityCommandKind, NodeId,
};

use crate::battle_materialization::UniverseBattleRoster;

use super::{
    SwarmDisasterRuntimeInstance,
    baseline_controller::{SwarmOfferedAction, SwarmOfferedCommand, route_offers},
    encounter_runtime::EncounterRole,
    replay::SwarmReplayError,
    replay_action::SwarmSeededRunAction,
    seeded_run::{
        MAXIMUM_STEPS, PLANE_ONE_DECAY, PLANE_TWO_DECAY, SwarmRecordedExecution,
        SwarmSeededBattleRecord, SwarmSeededBoundary, SwarmSeededReplayStep, SwarmSeededRunError,
        SwarmSeededRunReport, SwarmSeededRunRequest, SwarmSeededRunStep, SwarmSeededStepKind,
        apply_and_record, configure_boundary, create_plane, is_battle_domain, is_boss,
        preview_encounter, step, validate_boundary,
    },
    seeded_run_digest::transcript_digest,
    seeded_run_route::{explicit_face_target, longest_legal_route, movement_program},
};

/// Generic command plus authored presentation metadata for one exact offer.
#[doc(hidden)]
pub type SwarmDisasterIncrementalOffer = (GraphActivityCommand, i32, ActivityDecisionKind);

/// Resumable external-control adapter. Its public commands are the existing
/// generic Activity command envelope; all Swarm mutations remain private.
#[doc(hidden)]
pub struct SwarmDisasterIncrementalRun {
    request: SwarmSeededRunRequest,
    state: ActivityTransactionState,
    rng: ActivityRngStreams,
    initialization_step: u8,
    plane: usize,
    prepared_boss_nodes: BTreeSet<NodeId>,
    selected_boss_nodes: BTreeSet<NodeId>,
    steps: Vec<SwarmSeededRunStep>,
    replay: Vec<SwarmSeededReplayStep>,
    battle_count: u32,
    maximum_disarray_level: i64,
    observed_one_to_zero: bool,
    observed_entry_one: bool,
    cross_plane_countdown_carried: bool,
}

impl SwarmDisasterIncrementalRun {
    /// Starts the baseline external-control fixture from immutable inputs.
    #[must_use]
    pub fn start(
        instance: &SwarmDisasterRuntimeInstance,
        seed: u64,
        identity: ActivityDefinitionIdentity,
        activity_instance: ActivityInstanceId,
    ) -> Self {
        Self::start_request(
            instance,
            SwarmSeededRunRequest {
                seed,
                identity,
                activity_instance,
                config_digest: identity.config_digest(),
                boundary: SwarmSeededBoundary::Baseline,
            },
        )
    }

    pub(super) fn start_request(
        instance: &SwarmDisasterRuntimeInstance,
        request: SwarmSeededRunRequest,
    ) -> Self {
        let state = ActivityTransactionState::new(
            instance.state_definition().clone(),
            instance.graph_definition().entry(),
        );
        let maximum_disarray_level = instance.disarray_level(&state).unwrap_or(0);
        Self {
            request,
            state,
            rng: ActivityRngStreams::new(ActivityRngContext::new(
                ActivityMasterSeed::from_u64(request.seed),
                request.identity.id(),
                request.identity.definition_digest(),
                request.config_digest,
                instance.graph_definition().digest(),
                request.activity_instance,
                None,
                Some(instance.graph_definition().entry()),
                None,
                0,
            )),
            initialization_step: 0,
            plane: 0,
            prepared_boss_nodes: BTreeSet::new(),
            selected_boss_nodes: BTreeSet::new(),
            steps: Vec::new(),
            replay: Vec::new(),
            battle_count: 0,
            maximum_disarray_level,
            observed_one_to_zero: false,
            observed_entry_one: false,
            cross_plane_countdown_carried: false,
        }
    }

    /// Current player-visible Activity projection.
    #[must_use]
    pub fn player_view(&self, instance: &SwarmDisasterRuntimeInstance) -> ActivityPlayerView {
        self.state.player_view(
            self.request.identity,
            instance.graph_definition(),
            self.request.activity_instance,
            &self.rng,
        )
    }

    /// Current canonical Activity state hash.
    #[must_use]
    pub fn state_hash(&self, instance: &SwarmDisasterRuntimeInstance) -> ActivityStateHash {
        self.state.state_hash(
            self.request.identity,
            instance.graph_definition(),
            self.request.activity_instance,
            &self.rng,
        )
    }

    /// Terminal outcome, when settled.
    #[must_use]
    pub const fn terminal(&self) -> Option<ActivityTerminalOutcome> {
        self.state.terminal()
    }

    /// Number of accepted Activity actions retained for replay.
    #[must_use]
    pub fn action_count(&self) -> usize {
        self.replay.len()
    }

    /// Number of real nested battles settled so far.
    #[must_use]
    pub const fn battle_count(&self) -> u32 {
        self.battle_count
    }

    /// Advances system-owned work until the next player decision or terminal.
    pub fn settle_automatic(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
        roster: &UniverseBattleRoster,
    ) -> Result<(u32, u32), SwarmReplayError> {
        self.settle_automatic_internal(instance, roster)
            .map_err(Into::into)
    }

    /// Returns the exact current offer as generic Activity commands paired
    /// with authored priority and decision kind.
    pub fn offered_commands(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
    ) -> Result<Box<[SwarmDisasterIncrementalOffer]>, SwarmReplayError> {
        let state_hash = self.state_hash(instance);
        let decision = self.decision_id().map_err(SwarmReplayError::from)?;
        self.offered_swarm_commands(instance)
            .map(|offers| {
                offers
                    .into_iter()
                    .map(|offer| {
                        (
                            GraphActivityCommand::new(
                                state_hash,
                                decision,
                                GraphActivityCommandKind::ChooseOption { option: offer.id() },
                            ),
                            offer.authored_priority(),
                            offer.family().decision_kind(),
                        )
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .map_err(Into::into)
    }

    /// Applies only an exact generic command regenerated at this boundary.
    pub fn apply_offered_command(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
        command: &GraphActivityCommand,
    ) -> Result<(), SwarmReplayError> {
        let state_hash = self.state_hash(instance);
        let decision = self.decision_id().map_err(SwarmReplayError::from)?;
        if command.expected_state_hash() != state_hash || command.decision() != decision {
            return Err(SwarmReplayError::Execution);
        }
        let option = match command.kind() {
            GraphActivityCommandKind::ChooseOption { option } => *option,
            _ => return Err(SwarmReplayError::Execution),
        };
        let selected = self
            .offered_swarm_commands(instance)
            .map_err(SwarmReplayError::from)?
            .into_iter()
            .find(|offer| offer.id() == option)
            .ok_or(SwarmReplayError::Execution)?;
        self.apply_swarm_command(instance, &selected)
            .map_err(Into::into)
    }

    pub(super) fn request(&self) -> SwarmSeededRunRequest {
        self.request
    }

    pub(super) fn decision_id(&self) -> Result<ActivityDecisionId, SwarmSeededRunError> {
        self.state
            .command_sequence()
            .checked_add(1)
            .and_then(ActivityDecisionId::new)
            .ok_or(SwarmSeededRunError::StepBudgetExceeded)
    }

    pub(super) fn settle_automatic_internal(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
        roster: &UniverseBattleRoster,
    ) -> Result<(u32, u32), SwarmSeededRunError> {
        let initial_actions = self.replay.len();
        let initial_battles = self.battle_count;
        loop {
            if let Some(terminal) = self.state.terminal() {
                if terminal != ActivityTerminalOutcome::Completed {
                    return Err(SwarmSeededRunError::UnexpectedTerminal(terminal));
                }
                break;
            }
            self.ensure_budget()?;
            if self.initialize_next(instance)? {
                continue;
            }
            let node = self.state.current_node();
            let domain = instance.map.node_domain_key(&self.state, node)?;
            if is_battle_domain(domain) && !self.state.current_battle_attempt_is_settled() {
                let selection = preview_encounter(instance, &self.state, &mut self.rng)?;
                if is_boss(selection.role) && !self.selected_boss_nodes.contains(&node) {
                    if self.prepared_boss_nodes.insert(node) {
                        self.prepare_boss_decay(instance, selection.role)?;
                        continue;
                    }
                    break;
                }
                self.execute_battle(instance, roster, selection.role)?;
                continue;
            }
            break;
        }
        Ok((
            u32::try_from(self.replay.len() - initial_actions)
                .map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?,
            self.battle_count
                .checked_sub(initial_battles)
                .ok_or(SwarmSeededRunError::StepBudgetExceeded)?,
        ))
    }

    pub(super) fn offered_swarm_commands(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
    ) -> Result<Vec<SwarmOfferedCommand>, SwarmSeededRunError> {
        if self.state.terminal().is_some() || self.initialization_step < 5 {
            return Ok(Vec::new());
        }
        let node = self.state.current_node();
        let domain = instance.map.node_domain_key(&self.state, node)?;
        if is_battle_domain(domain) && !self.state.current_battle_attempt_is_settled() {
            let preview = preview_encounter(instance, &self.state, &mut self.rng)?;
            if !is_boss(preview.role) || self.selected_boss_nodes.contains(&node) {
                return Ok(Vec::new());
            }
            let layer = u8::try_from(self.plane + 1)
                .map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
            let mut choices = preview
                .waves
                .iter()
                .flat_map(|wave| wave.slots.iter())
                .flat_map(|slot| slot.boss_choices.iter())
                .map(AsRef::as_ref)
                .collect::<Vec<_>>();
            if choices.is_empty() {
                choices.extend(instance.boss_choices());
            }
            let preferred = choices
                .first()
                .copied()
                .ok_or(SwarmSeededRunError::MissingBossChoice(node))?;
            return Ok(choices
                .iter()
                .enumerate()
                .map(|(ordinal, boss)| {
                    SwarmOfferedCommand::boss(layer, ordinal, boss, *boss == preferred)
                })
                .collect());
        }
        let plane_end = instance
            .plane_ends()
            .nth(self.plane)
            .ok_or(SwarmSeededRunError::Incomplete)?;
        if node == plane_end {
            return Err(SwarmSeededRunError::Incomplete);
        }
        let preferred = longest_legal_route(instance, &self.state, node, plane_end)?;
        route_offers(instance, &self.state, node, preferred)
    }

    pub(super) fn apply_swarm_command(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
        selected: &SwarmOfferedCommand,
    ) -> Result<(), SwarmSeededRunError> {
        self.ensure_budget()?;
        let selected = self
            .offered_swarm_commands(instance)?
            .into_iter()
            .find(|candidate| candidate == selected)
            .ok_or(SwarmSeededRunError::CommandNotOffered)?;
        match selected.action() {
            SwarmOfferedAction::SelectBoss { plane, boss } => {
                let expected_plane = u8::try_from(self.plane + 1)
                    .map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
                if *plane != expected_plane {
                    return Err(SwarmSeededRunError::CommandNotOffered);
                }
                let source = self.state.current_node();
                let program = instance.compile_boss_selection(*plane, boss)?;
                apply_and_record(
                    instance,
                    &mut self.state,
                    &self.rng,
                    self.request,
                    program,
                    SwarmSeededStepKind::BossSelection(*plane),
                    SwarmSeededRunAction::BossSelection {
                        source_node: source,
                        plane: *plane,
                        boss: boss.clone(),
                    },
                    &mut self.steps,
                    &mut self.replay,
                )?;
                self.selected_boss_nodes.insert(source);
            }
            SwarmOfferedAction::Traverse { edge, target } => {
                let node = self.state.current_node();
                let before = (
                    instance.countdown(&self.state)?,
                    instance.disarray_level(&self.state)?,
                );
                let program = if instance.dice_roll_available(&self.state)? {
                    let roll = instance.compile_dice_roll(&self.state, &mut self.rng)?;
                    apply_and_record(
                        instance,
                        &mut self.state,
                        &self.rng,
                        self.request,
                        roll,
                        SwarmSeededStepKind::DiceRoll,
                        SwarmSeededRunAction::DiceRoll { source_node: node },
                        &mut self.steps,
                        &mut self.replay,
                    )?;
                    let face_target = explicit_face_target(instance, &self.state, &mut self.rng)?;
                    instance.compile_simultaneous_resolution(
                        &self.state,
                        Some((*target, &[])),
                        face_target,
                        None,
                        (None, None),
                        &mut self.rng,
                    )?
                } else {
                    movement_program(instance, &self.state, *target)?
                };
                apply_and_record(
                    instance,
                    &mut self.state,
                    &self.rng,
                    self.request,
                    program,
                    SwarmSeededStepKind::Traverse,
                    SwarmSeededRunAction::Traverse {
                        source_node: node,
                        edge: *edge,
                    },
                    &mut self.steps,
                    &mut self.replay,
                )?;
                let after = (
                    instance.countdown(&self.state)?,
                    instance.disarray_level(&self.state)?,
                );
                self.observed_one_to_zero |= before == (1, 0) && after == (0, 0);
                self.observed_entry_one |= before == (0, 0) && after == (-1, 1);
                self.maximum_disarray_level = self.maximum_disarray_level.max(after.1);
            }
            _ => return Err(SwarmSeededRunError::CommandNotOffered),
        }
        Ok(())
    }

    pub(super) fn recorded_execution(
        &self,
        instance: &SwarmDisasterRuntimeInstance,
    ) -> Result<SwarmRecordedExecution, SwarmSeededRunError> {
        let terminal = self
            .state
            .terminal()
            .ok_or(SwarmSeededRunError::Incomplete)?;
        if terminal != ActivityTerminalOutcome::Completed {
            return Err(SwarmSeededRunError::UnexpectedTerminal(terminal));
        }
        validate_boundary(
            self.request.boundary,
            self.maximum_disarray_level,
            self.observed_one_to_zero,
            self.observed_entry_one,
            self.cross_plane_countdown_carried,
        )?;
        let final_state_hash = self.state_hash(instance);
        let step_count =
            u32::try_from(self.steps.len()).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
        let digest = transcript_digest(
            self.request.seed,
            terminal,
            final_state_hash,
            self.battle_count,
            self.maximum_disarray_level,
            self.cross_plane_countdown_carried,
            &self.steps,
        );
        Ok(SwarmRecordedExecution {
            report: SwarmSeededRunReport {
                terminal,
                final_state_hash,
                transcript_digest: digest,
                battle_count: self.battle_count,
                step_count,
                maximum_disarray_level: self.maximum_disarray_level,
                cross_plane_countdown_carried: self.cross_plane_countdown_carried,
                steps: self.steps.clone().into_boxed_slice(),
            },
            replay: self.replay.to_vec().into_boxed_slice(),
        })
    }

    fn initialize_next(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
    ) -> Result<bool, SwarmSeededRunError> {
        match self.initialization_step {
            0 => {
                if instance.countdown(&self.state)? != 20 {
                    return Err(SwarmSeededRunError::BoundaryNotObserved(
                        SwarmSeededBoundary::InitialCountdown,
                    ));
                }
                let source = self.state.current_node();
                let program = instance.compile_profile_entry_rule(&self.state)?;
                apply_and_record(
                    instance,
                    &mut self.state,
                    &self.rng,
                    self.request,
                    program,
                    SwarmSeededStepKind::ProfileEntry,
                    SwarmSeededRunAction::ProfileEntry {
                        source_node: source,
                    },
                    &mut self.steps,
                    &mut self.replay,
                )?;
            }
            1 => {
                let source = self.state.current_node();
                let program = instance.compile_audience_initialization(&self.state)?;
                apply_and_record(
                    instance,
                    &mut self.state,
                    &self.rng,
                    self.request,
                    program,
                    SwarmSeededStepKind::AudienceInitialization,
                    SwarmSeededRunAction::AudienceInitialization {
                        source_node: source,
                    },
                    &mut self.steps,
                    &mut self.replay,
                )?;
            }
            2 => {
                let source = self.state.current_node();
                let program = instance.compile_trail_run_start(&self.state)?;
                apply_and_record(
                    instance,
                    &mut self.state,
                    &self.rng,
                    self.request,
                    program,
                    SwarmSeededStepKind::TrailRunStart,
                    SwarmSeededRunAction::TrailRunStart {
                        source_node: source,
                    },
                    &mut self.steps,
                    &mut self.replay,
                )?;
            }
            3 => configure_boundary(
                instance,
                &mut self.state,
                &self.rng,
                self.request,
                &mut self.steps,
                &mut self.replay,
            )?,
            4 => create_plane(
                instance,
                &mut self.state,
                &mut self.rng,
                self.request,
                0,
                &mut self.steps,
                &mut self.replay,
            )?,
            _ => return Ok(false),
        }
        self.initialization_step = self
            .initialization_step
            .checked_add(1)
            .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
        Ok(true)
    }

    fn prepare_boss_decay(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
        role: EncounterRole,
    ) -> Result<(), SwarmSeededRunError> {
        let layer =
            u8::try_from(self.plane + 1).map_err(|_| SwarmSeededRunError::StepBudgetExceeded)?;
        let decay = match layer {
            1 => Some(PLANE_ONE_DECAY),
            2 => Some(PLANE_TWO_DECAY),
            _ => None,
        };
        if let Some(decay) = decay {
            let source = self.state.current_node();
            let program = instance.compile_boss_decay_selection(&self.state, &[decay])?;
            apply_and_record(
                instance,
                &mut self.state,
                &self.rng,
                self.request,
                program,
                SwarmSeededStepKind::BossSelection(layer),
                SwarmSeededRunAction::BossDecaySelection {
                    source_node: source,
                    plane: layer,
                    decay: decay.into(),
                },
                &mut self.steps,
                &mut self.replay,
            )?;
        } else if instance.countdown.selected_boss_decay(&self.state)?.len() != 2 {
            return Err(SwarmSeededRunError::BoundaryNotObserved(
                SwarmSeededBoundary::FinalBossDecay,
            ));
        }
        let preview = preview_encounter(instance, &self.state, &mut self.rng)?;
        if preview.role != role {
            return Err(SwarmSeededRunError::Incomplete);
        }
        Ok(())
    }

    fn execute_battle(
        &mut self,
        instance: &SwarmDisasterRuntimeInstance,
        roster: &UniverseBattleRoster,
        role: EncounterRole,
    ) -> Result<(), SwarmSeededRunError> {
        let node = self.state.current_node();
        let selection = preview_encounter(instance, &self.state, &mut self.rng)?;
        let before_transition = (
            instance.countdown(&self.state)?,
            instance.disarray_level(&self.state)?,
        );
        let expected = self.state_hash(instance);
        let sequence = self
            .battle_count
            .checked_add(1)
            .and_then(BattleSequence::new)
            .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
        let start = instance.start_current_battle(
            &mut self.state,
            &mut self.rng,
            expected,
            self.request.identity,
            self.request.activity_instance,
            AttemptId::new(1).expect("the fixed seeded Attempt is non-zero"),
            sequence,
            roster,
        )?;
        let start_identity = start.handoff().identity();
        let (result, report, _) = instance.execute_started_battle(
            &mut self.state,
            &self.rng,
            self.request.identity,
            self.request.activity_instance,
            &start,
            false,
        )?;
        if report.outcome() != starclock_activity::BattleOutcome::Won {
            return Err(SwarmSeededRunError::BattleNotWon(role));
        }
        self.battle_count = sequence.get();
        let accepted = step(
            instance,
            &self.state,
            &self.rng,
            self.request,
            SwarmSeededStepKind::Battle(role),
            node,
            Some(result.actual_digest()),
        );
        self.replay.push(SwarmSeededReplayStep {
            action: SwarmSeededRunAction::Battle {
                source_node: node,
                role,
                group: selection.group,
                member: selection.source_rogue_monster_id,
                effective_level: selection.effective_level,
            },
            state_hash: accepted.state_hash,
            battle: Some(SwarmSeededBattleRecord {
                start_identity,
                result,
                report,
            }),
        });
        self.steps.push(accepted);
        if is_boss(role) && self.state.terminal().is_none() {
            let after_transition = (
                instance.countdown(&self.state)?,
                instance.disarray_level(&self.state)?,
            );
            self.cross_plane_countdown_carried |= before_transition == after_transition;
            self.plane = self
                .plane
                .checked_add(1)
                .ok_or(SwarmSeededRunError::StepBudgetExceeded)?;
            let next = instance
                .plane_starts()
                .nth(self.plane)
                .ok_or(SwarmSeededRunError::Incomplete)?;
            if self.state.current_node() != next {
                return Err(SwarmSeededRunError::Incomplete);
            }
            create_plane(
                instance,
                &mut self.state,
                &mut self.rng,
                self.request,
                self.plane,
                &mut self.steps,
                &mut self.replay,
            )?;
        }
        self.maximum_disarray_level = self
            .maximum_disarray_level
            .max(instance.disarray_level(&self.state)?);
        Ok(())
    }

    fn ensure_budget(&self) -> Result<(), SwarmSeededRunError> {
        if self.steps.len() >= MAXIMUM_STEPS {
            Err(SwarmSeededRunError::StepBudgetExceeded)
        } else {
            Ok(())
        }
    }
}
