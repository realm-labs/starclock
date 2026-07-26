//! Nested battle handoff and settlement accessors for the runtime facade.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityBattleResultContract, ActivityProgramDefinition,
    ActivityProgramId, ActivityStateHash, BattleBinding, BattleOutcome, BattleResult,
    GraphActivityBattleError, GraphActivityBattleResolution, GraphActivityRuntimeError,
    ProjectedValue, TechniqueContributionDigest,
};

use crate::ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope};
use crate::{
    curio_activity::event_key,
    curio_effect_runtime::{CurioEffectFacts, CurioEvent},
};

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
            let mut operations = projection.operations().to_vec();
            let contributions = self.curio_contributions().map_err(|_| invalid_boundary())?;
            if let Some(curio) = contributions
                .entries()
                .iter()
                .find(|entry| entry.state().source_effect_id() == "76")
            {
                let full_hp_allies = result
                    .values()
                    .iter()
                    .filter_map(|value| match value {
                        ProjectedValue::ParticipantState(state) => Some(*state),
                        _ => None,
                    })
                    .filter(|state| state.current_hp() == state.maximum_hp())
                    .count();
                let event = CurioEvent::BattleWon;
                let projection = self
                    .curio_activity_projection(
                        curio.curio(),
                        event,
                        CurioEffectFacts {
                            full_hp_allies: u32::try_from(full_hp_allies)
                                .map_err(|_| invalid_boundary())?,
                            destroyed_curios: contributions.destroyed_curios(),
                            ..CurioEffectFacts::default()
                        },
                    )
                    .map_err(|_| invalid_boundary())?;
                operations.push(starclock_activity::ActivityOperation::AddCounter {
                    slot: self.curio_event_slot,
                    key: event_key(curio.curio(), event),
                    delta: starclock_activity::ActivityExpression::Literal(
                        starclock_activity::ActivityValue::BoundedInteger(1),
                    ),
                });
                operations.extend_from_slice(projection.operations());
            }
            (!operations.is_empty())
                .then(|| {
                    ActivityProgramDefinition::new(
                        ActivityProgramId::new(AFTER_BATTLE_ABILITY_PROGRAM)
                            .expect("static boundary program ID is non-zero"),
                        operations,
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
