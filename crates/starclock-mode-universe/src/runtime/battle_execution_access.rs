//! Nested battle handoff and settlement accessors for the runtime facade.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattleResultContract, ActivityProgramDefinition,
    ActivityProgramId, ActivityStateHash, BattleBinding, BattleOutcome, BattleResult,
    GraphActivityBattleError, GraphActivityBattleResolution, GraphActivityRuntimeError,
    ProjectedValue, TechniqueContributionDigest,
};

use crate::ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope};

use super::{StandardUniverseActivity, StandardUniverseBattleStartError};
const AFTER_BATTLE_ABILITY_PROGRAM: u32 = 9_700_001;

impl StandardUniverseActivity {
    pub(crate) fn start_assembled_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
        binding: BattleBinding,
        contribution: TechniqueContributionDigest,
        contract: Arc<ActivityBattleResultContract>,
    ) -> Result<ActivityBattleHandoff, StandardUniverseBattleStartError> {
        self.graph
            .start_assembled_pending_battle(expected_state_hash, binding, contribution, contract)
            .map_err(StandardUniverseBattleStartError::Activity)
    }

    pub fn start_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<ActivityBattleHandoff, StandardUniverseBattleStartError> {
        let digest = self
            .graph
            .pending_battle()
            .ok_or(StandardUniverseBattleStartError::MissingPendingBattle)?
            .battle_spec_digest();
        let binding = self
            .overlay
            .binding_for_spec(digest.bytes())
            .ok_or(StandardUniverseBattleStartError::MissingBattleOverlay)?;
        self.graph
            .start_pending_battle(expected_state_hash, Arc::clone(binding.contract()))
            .map_err(StandardUniverseBattleStartError::Activity)
    }

    pub(crate) fn rollback_pending_battle_start(&mut self) -> bool {
        self.graph.rollback_pending_battle_start()
    }

    pub fn submit_pending_battle_result(
        &mut self,
        expected_state_hash: ActivityStateHash,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        let won = result
            .values()
            .iter()
            .any(|value| matches!(value, ProjectedValue::Outcome(BattleOutcome::Won)));
        let boundary_program = if won {
            let first_battle_won = self.view().completed_battle_count() == 0;
            let chosen_path_blessings = self
                .path_contributions()
                .map_err(|_| invalid_boundary())?
                .selected_path_blessings();
            let projection = self
                .ability_activity_delta_projection(AbilityExecutionContext::new(
                    AbilityProjectionScope::Run,
                    AbilityBoundary::AfterBattle,
                    chosen_path_blessings,
                    first_battle_won,
                ))
                .map_err(|_| invalid_boundary())?;
            (!projection.operations().is_empty())
                .then(|| {
                    ActivityProgramDefinition::new(
                        ActivityProgramId::new(AFTER_BATTLE_ABILITY_PROGRAM)
                            .expect("static boundary program ID is non-zero"),
                        projection.operations().to_vec(),
                    )
                })
                .transpose()
                .map_err(|_| invalid_boundary())?
        } else {
            None
        };
        self.graph
            .submit_pending_battle_result_with_boundary_program(
                expected_state_hash,
                result,
                boundary_program.as_ref(),
            )
    }
}

const fn invalid_boundary() -> GraphActivityBattleError {
    GraphActivityBattleError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
