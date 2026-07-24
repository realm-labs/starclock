//! Aggregate Standard Universe state into one immutable battle contribution set.

use crate::{
    ability_runtime::{AbilityExecutionContext, AbilityProjectionScope},
    battle_contribution::UniverseBattleContributionSet,
};

use super::{StandardUniverseActivity, StandardUniverseBattleContributionError};

impl StandardUniverseActivity {
    pub fn battle_contributions(
        &self,
        context: AbilityExecutionContext,
    ) -> Result<UniverseBattleContributionSet, StandardUniverseBattleContributionError> {
        if context.scope() != AbilityProjectionScope::Battle {
            return Err(StandardUniverseBattleContributionError::InvalidScope);
        }
        let blessings = self
            .blessing_contributions()
            .map_err(StandardUniverseBattleContributionError::Blessing)?;
        let path = self
            .path_contributions()
            .map_err(StandardUniverseBattleContributionError::Path)?;
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
        self.battle_contribution_compiler
            .compile_snapshot(&path, &blessings, &curios, &abilities, &projection)
            .map_err(StandardUniverseBattleContributionError::Compile)
    }
}
