//! Nested battle handoff and settlement accessors for the runtime facade.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityStateHash, BattleResult, GraphActivityBattleError,
    GraphActivityBattleResolution,
};

use super::{StandardUniverseActivity, StandardUniverseBattleStartError};

impl StandardUniverseActivity {
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
        self.graph
            .submit_pending_battle_result(expected_state_hash, result)
    }
}
