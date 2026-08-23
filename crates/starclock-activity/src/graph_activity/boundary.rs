use super::*;
use crate::ActivityRngLabel;

impl GraphActivity {
    /// Atomically settles a battle, applies its state-only boundary program,
    /// advances the graph, then generates one RNG-backed follow-up program.
    ///
    /// The generator observes the post-transition player view. Any settlement,
    /// generation, follow-up mutation or pump error restores both state and RNG.
    pub fn submit_pending_battle_result_with_generated_follow_up(
        &mut self,
        expected_state_hash: ActivityStateHash,
        result: BattleResult,
        boundary_program: Option<&ActivityProgramDefinition>,
        follow_up_program_id: ActivityProgramId,
        generate: impl FnOnce(
            &ActivityPlayerView,
            &mut ActivityRngStreams,
        ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError>,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        if boundary_program.is_some_and(|program| {
            program
                .validate_against(&self.definition.state, &self.definition.graph)
                .is_err()
                || contains_boundary_operation(program.operations())
        }) {
            return Err(GraphActivityBattleError::Runtime(
                GraphActivityRuntimeError::InvalidBoundaryProgram,
            ));
        }
        let original_state = self.state.transaction_copy();
        let original_rng = self.rng.transaction_copy();
        let outcome = (|| {
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
            let mut events = Vec::new();
            if let Some(program) = boundary_program {
                let cause = ActivityCause::new(
                    self.state.command_sequence().saturating_add(1),
                    program.id(),
                    self.state.current_node(),
                )
                .ok_or(GraphActivityBattleError::Runtime(
                    GraphActivityRuntimeError::InvalidBoundaryProgram,
                ))?;
                events.extend(
                    committed_runtime(self.state.apply_settlement_extension_program(
                        program,
                        cause,
                        &self.definition.graph,
                    ))
                    .map_err(GraphActivityBattleError::Runtime)?,
                );
            }
            events.extend(self.pump().map_err(GraphActivityBattleError::Runtime)?);
            let operations = generate(&self.player_view(), &mut self.rng)
                .map_err(GraphActivityBattleError::GeneratedBoundary)?;
            if !operations.is_empty() {
                let program = ActivityProgramDefinition::new(follow_up_program_id, operations)
                    .map_err(|_| invalid_random_boundary())
                    .map_err(GraphActivityBattleError::GeneratedBoundary)?;
                if program
                    .validate_against(&self.definition.state, &self.definition.graph)
                    .is_err()
                    || contains_boundary_operation(program.operations())
                {
                    return Err(GraphActivityBattleError::Runtime(
                        GraphActivityRuntimeError::InvalidBoundaryProgram,
                    ));
                }
                let cause = ActivityCause::new(
                    self.state.command_sequence().saturating_add(1),
                    program.id(),
                    self.state.current_node(),
                )
                .ok_or(GraphActivityBattleError::Runtime(
                    GraphActivityRuntimeError::InvalidBoundaryProgram,
                ))?;
                events.extend(
                    committed_runtime(self.state.apply_extension_program(
                        &program,
                        cause,
                        &self.definition.graph,
                    ))
                    .map_err(GraphActivityBattleError::Runtime)?,
                );
                events.extend(self.pump().map_err(GraphActivityBattleError::Runtime)?);
            }
            Ok(GraphActivityBattleResolution {
                settlement,
                events: events.into_boxed_slice(),
                state_hash: self.state_hash(),
            })
        })();
        if outcome.is_err() {
            self.state = original_state;
            self.rng = original_rng;
        }
        outcome
    }

    /// Atomically generates and applies one RNG-backed state program.
    ///
    /// The generator receives the authoritative Activity RNG streams and must
    /// return both the ordered operations and its caller-owned result. Any
    /// generation, validation or mutation error restores state and RNG to the
    /// exact command-boundary snapshot.
    pub fn apply_generated_boundary<T>(
        &mut self,
        expected_state_hash: ActivityStateHash,
        program_id: ActivityProgramId,
        generate: impl FnOnce(
            &mut ActivityRngStreams,
        ) -> Result<(Vec<ActivityOperation>, T), GraphActivityCommandError>,
    ) -> Result<ActivityGeneratedBoundaryResolution<T>, GraphActivityCommandError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let original_state = self.state.transaction_copy();
        let original_rng = self.rng.transaction_copy();
        let outcome = (|| {
            let (operations, value) = generate(&mut self.rng)?;
            let program = ActivityProgramDefinition::new(program_id, operations)
                .map_err(|_| invalid_random_boundary())?;
            if program
                .validate_against(&self.definition.state, &self.definition.graph)
                .is_err()
                || contains_boundary_operation(program.operations())
            {
                return Err(invalid_random_boundary());
            }
            let cause = ActivityCause::new(
                self.state.command_sequence().saturating_add(1),
                program.id(),
                self.state.current_node(),
            )
            .ok_or_else(invalid_random_boundary)?;
            let mut events =
                match self
                    .state
                    .apply_extension_program(&program, cause, &self.definition.graph)
                {
                    ActivityTransactionOutcome::Committed(events) => events.into_vec(),
                    ActivityTransactionOutcome::Rejected(rejection) => {
                        return Err(GraphActivityCommandError::Rejected(rejection));
                    }
                    ActivityTransactionOutcome::Faulted(_, _) => {
                        return Err(invalid_random_boundary());
                    }
                };
            events.extend(self.pump().map_err(GraphActivityCommandError::Runtime)?);
            Ok(ActivityGeneratedBoundaryResolution {
                value,
                events: events.into_boxed_slice(),
                state_hash: self.state_hash(),
            })
        })();
        if outcome.is_err() {
            self.state = original_state;
            self.rng = original_rng;
        }
        outcome
    }

    /// Atomically applies one state-only extension program at the current
    /// command boundary and resumes automatic graph execution.
    pub fn apply_boundary_program(
        &mut self,
        expected_state_hash: ActivityStateHash,
        program: &ActivityProgramDefinition,
    ) -> Result<Box<[ActivityTransactionEvent]>, GraphActivityCommandError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if program
            .validate_against(&self.definition.state, &self.definition.graph)
            .is_err()
            || contains_boundary_operation(program.operations())
        {
            return Err(GraphActivityCommandError::Runtime(
                GraphActivityRuntimeError::InvalidBoundaryProgram,
            ));
        }
        let original_state = self.state.transaction_copy();
        let original_rng = self.rng.transaction_copy();
        let outcome = (|| {
            let cause = ActivityCause::new(
                self.state.command_sequence().saturating_add(1),
                program.id(),
                self.state.current_node(),
            )
            .ok_or(GraphActivityCommandError::Runtime(
                GraphActivityRuntimeError::InvalidBoundaryProgram,
            ))?;
            let mut events = committed_runtime(self.state.apply_extension_program(
                program,
                cause,
                &self.definition.graph,
            ))
            .map_err(GraphActivityCommandError::Runtime)?;
            events.extend(self.pump().map_err(GraphActivityCommandError::Runtime)?);
            Ok(events.into_boxed_slice())
        })();
        if outcome.is_err() {
            self.state = original_state;
            self.rng = original_rng;
        }
        outcome
    }

    /// Atomically samples enabled authored options and applies their ordinary
    /// state operations at a command boundary.
    ///
    /// This is the generic run-layer primitive for rewards whose count and
    /// contents are random but which do not create a player decision. Candidate
    /// order is canonicalized by option ID, all RNG draws are part of the
    /// Activity state hash, and any validation or mutation failure restores
    /// both state and RNG exactly.
    #[allow(clippy::too_many_arguments)]
    pub fn apply_random_option_boundary(
        &mut self,
        expected_state_hash: ActivityStateHash,
        program_id: ActivityProgramId,
        label: ActivityRngLabel,
        count_purpose: u16,
        choice_purpose: u16,
        minimum: u16,
        maximum: u16,
        prefix: &[ActivityOperation],
        candidates: &[(ActivityOptionDefinition, u64)],
    ) -> Result<ActivityRandomBoundaryResolution, GraphActivityCommandError> {
        if expected_state_hash != self.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if minimum > maximum || count_purpose == 0 || choice_purpose == 0 || candidates.is_empty() {
            return Err(invalid_random_boundary());
        }
        let mut candidates = candidates.to_vec();
        candidates.sort_by_key(|candidate| candidate.0.id());
        if candidates.iter().any(|candidate| candidate.1 == 0)
            || candidates
                .windows(2)
                .any(|pair| pair[0].0.id() == pair[1].0.id())
        {
            return Err(invalid_random_boundary());
        }
        let original_state = self.state.transaction_copy();
        let original_rng = self.rng.transaction_copy();
        let outcome = (|| {
            let mut eligible = Vec::new();
            for (option, weight) in candidates {
                if self
                    .state
                    .condition(option.enabled())
                    .map_err(|_| invalid_random_boundary())?
                {
                    eligible.push((option, weight));
                }
            }
            if eligible.len() < usize::from(minimum) {
                return Err(invalid_random_boundary());
            }
            let range = maximum
                .checked_sub(minimum)
                .and_then(|value| value.checked_add(1))
                .ok_or_else(invalid_random_boundary)?;
            let sampled_count = if maximum == 0 || range == 1 {
                minimum
            } else {
                let draw = self
                    .rng
                    .choose_index(label, count_purpose, u32::from(range))
                    .map_err(GraphActivityCommandError::Rng)?
                    .ok_or_else(invalid_random_boundary)?;
                minimum
                    .checked_add(
                        u16::try_from(draw.value()).map_err(|_| invalid_random_boundary())?,
                    )
                    .ok_or_else(invalid_random_boundary)?
            };
            let count = sampled_count
                .min(u16::try_from(eligible.len()).map_err(|_| invalid_random_boundary())?);
            let weights = eligible
                .iter()
                .map(|candidate| candidate.1)
                .collect::<Vec<_>>();
            let selected = if count == 0 {
                Vec::new().into_boxed_slice()
            } else {
                self.rng
                    .choose_weighted_without_replacement(label, choice_purpose, &weights, count)
                    .map_err(GraphActivityCommandError::Rng)?
            };
            let mut selected_options = Vec::with_capacity(selected.len());
            let mut operations = prefix.to_vec();
            for index in selected {
                let option = &eligible[index as usize].0;
                selected_options.push(option.id());
                operations.extend_from_slice(option.operations());
            }
            let program = ActivityProgramDefinition::new(program_id, operations)
                .map_err(|_| invalid_random_boundary())?;
            if program
                .validate_against(&self.definition.state, &self.definition.graph)
                .is_err()
                || contains_boundary_operation(program.operations())
            {
                return Err(invalid_random_boundary());
            }
            let cause = ActivityCause::new(
                self.state.command_sequence().saturating_add(1),
                program.id(),
                self.state.current_node(),
            )
            .ok_or_else(invalid_random_boundary)?;
            let mut events = committed_runtime(self.state.apply_extension_program(
                &program,
                cause,
                &self.definition.graph,
            ))
            .map_err(GraphActivityCommandError::Runtime)?;
            events.extend(self.pump().map_err(GraphActivityCommandError::Runtime)?);
            Ok(ActivityRandomBoundaryResolution {
                selected_options: selected_options.into_boxed_slice(),
                events: events.into_boxed_slice(),
                state_hash: self.state_hash(),
            })
        })();
        if outcome.is_err() {
            self.state = original_state;
            self.rng = original_rng;
        }
        outcome
    }
}

fn invalid_random_boundary() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
