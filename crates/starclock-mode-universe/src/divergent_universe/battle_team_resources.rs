//! Original-roster resource assembly. All live mutations remain combat-owned.

use crate::divergent_universe::DivergentUniverseBattleAssemblyError;
use starclock_build::light_cone::CombatPath;
use starclock_combat::{
    KeyedTeamResourceSpec, ParticipantSpec, SourceDefinitionId, TeamResourceSpec,
    TeamResourceWavePolicy, TeamSide,
};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::battle_team_resources::{
        BattleTeamResourceDefinition, BattleTeamResourcePolicy,
    },
};

// Mode-owned keyed-resource address; unrelated to source row locators or Curios.
pub(super) const PUNCHLINE: SourceDefinitionId =
    SourceDefinitionId::new(0x7ead_0001).expect("reserved resource identity is non-zero");

pub(super) fn player_resources(
    definitions: &[BattleTeamResourceDefinition],
    core: &SimulationCatalog,
    players: &[ParticipantSpec],
) -> Result<TeamResourceSpec, DivergentUniverseBattleAssemblyError> {
    let invalid = || DivergentUniverseBattleAssemblyError::InvalidTeamResources;
    let [definition] = definitions else {
        return Err(invalid());
    };
    match definition.policy {
        BattleTeamResourcePolicy::VersionedProjectPolicyOriginalElationZeroClampPersist => {}
    }
    let mut eligible = false;
    // Evaluate every original participant, even after finding an Elation form:
    // an invalid roster must not become valid through early-return ordering.
    for player in players {
        if player.side() != TeamSide::Player {
            return Err(invalid());
        }
        let character = core
            .build_catalog()
            .character(player.combatant().form())
            .ok_or_else(invalid)?;
        eligible |= character.path() == CombatPath::Elation;
    }
    let keyed = if eligible {
        vec![
            KeyedTeamResourceSpec::new(
                PUNCHLINE,
                definition.initial_value,
                definition.maximum_value,
                TeamResourceWavePolicy::Persist,
            )
            .and_then(|resource| resource.with_stable_key(definition.resource_key.clone()))
            .ok_or_else(invalid)?,
        ]
    } else {
        Vec::new()
    };
    TeamResourceSpec::new(3, 5)
        .and_then(|resources| resources.with_keyed(keyed))
        .ok_or_else(invalid)
}
