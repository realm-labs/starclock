use super::*;

impl GraphActivity {
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
            let mut events = committed_runtime(self.state.apply_program(
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
}
