//! Aggregate Standard Universe state into one immutable battle contribution set.

use crate::{
    ability_runtime::{AbilityBoundary, AbilityExecutionContext, AbilityProjectionScope},
    battle_contribution::UniverseBattleContributionSet,
    battle_snapshot::StandardUniverseBattleSnapshot,
};

use super::{StandardUniverseActivity, StandardUniverseBattleContributionError};

impl StandardUniverseActivity {
    pub fn battle_start_snapshot(
        &self,
    ) -> Result<StandardUniverseBattleSnapshot, StandardUniverseBattleContributionError> {
        let view = self.graph.player_view();
        let path = self
            .path_contributions()
            .map_err(StandardUniverseBattleContributionError::Path)?;
        let context = AbilityExecutionContext::new(
            AbilityProjectionScope::Battle,
            AbilityBoundary::BattleStart,
            path.selected_path_blessings(),
            view.completed_battle_count() > 0,
        );
        self.compile_battle_snapshot(view, path, context)
    }

    pub fn battle_contribution_snapshot(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<StandardUniverseBattleSnapshot, StandardUniverseBattleContributionError> {
        if context.scope() != AbilityProjectionScope::Battle {
            return Err(StandardUniverseBattleContributionError::InvalidScope);
        }
        let view = self.graph.player_view();
        let path = self
            .path_contributions()
            .map_err(StandardUniverseBattleContributionError::Path)?;
        let expected_context = AbilityExecutionContext::new(
            context.scope(),
            context.boundary(),
            path.selected_path_blessings(),
            view.completed_battle_count() > 0,
        );
        if context != expected_context {
            return Err(StandardUniverseBattleContributionError::ContextMismatch);
        }
        self.compile_battle_snapshot(view, path, context)
    }

    fn compile_battle_snapshot(
        &self,
        view: starclock_activity::ActivityPlayerView,
        path: crate::path_runtime::PathContributionSet,
        context: AbilityExecutionContext,
    ) -> Result<StandardUniverseBattleSnapshot, StandardUniverseBattleContributionError> {
        let blessings = self
            .blessing_contributions()
            .map_err(StandardUniverseBattleContributionError::Blessing)?;
        let curios = self
            .curio_contributions()
            .map_err(StandardUniverseBattleContributionError::Curio)?;
        let abilities = self
            .ability_tree_contributions()
            .map_err(StandardUniverseBattleContributionError::Ability)?;
        let projection = self
            .ability_runtime
            .project(&self.ability_tree, context)
            .map_err(StandardUniverseBattleContributionError::Projection)?;
        let contributions = self
            .battle_contribution_compiler
            .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
            .map_err(StandardUniverseBattleContributionError::Compile)?;
        StandardUniverseBattleSnapshot::new(
            view.state_hash(),
            &self.participants,
            context,
            path,
            blessings,
            curios,
            abilities,
            projection,
            contributions,
            view.participant_carry(),
        )
        .map_err(StandardUniverseBattleContributionError::Snapshot)
    }

    pub fn battle_contributions(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<UniverseBattleContributionSet, StandardUniverseBattleContributionError> {
        self.battle_contribution_snapshot(context)
            .map(StandardUniverseBattleSnapshot::into_contributions)
    }
}
