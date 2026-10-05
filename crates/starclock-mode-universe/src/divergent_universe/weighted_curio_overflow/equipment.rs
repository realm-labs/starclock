//! Normal immutable equipment handoff; no live Activity or battle queries.
use crate::divergent_universe::{
    DivergentUniverseBattleAssemblyError,
    contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
    weighted_curio::{WeightedCurioRuntime, WeightedCurioSnapshot},
    weighted_curio_overflow::{
        OverflowBaseDamagePolicy, attack_increase::bind_attack_increase_policy,
        bind_death_conversion_policy,
    },
};
use starclock_build::light_cone::CombatPath;
use starclock_combat::{ParticipantSpec, TeamSide, catalog::builder::CombatCatalogBuilder};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_overflows::WeightedCurioOverflowStatus,
};

impl WeightedCurioRuntime {
    pub(in crate::divergent_universe) fn assemble_overflows(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        protocol: &DivergentUniverseDifficultyProtocolSnapshot,
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for definition in &self.overflows {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            match definition.status {
                WeightedCurioOverflowStatus::NativeProjectPolicyHitEnded => {}
            }
            let base = OverflowBaseDamagePolicy::from_authored(definition, protocol)
                .map_err(|_| DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants)?;
            for player in &mut *players {
                if player.side() != TeamSide::Player || player.formation().get() > 3 {
                    return Err(DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants);
                }
                let character = core
                    .build_catalog()
                    .character(player.combatant().form())
                    .ok_or(DivergentUniverseBattleAssemblyError::InvalidPlayerParticipants)?;
                if matches!(character.path(), CombatPath::Hunt | CombatPath::Erudition) {
                    let with_attack = bind_attack_increase_policy(
                        builder,
                        definition,
                        core,
                        player,
                        assembly_digest,
                    )?;
                    *player = bind_death_conversion_policy(
                        builder,
                        definition,
                        &with_attack,
                        &base,
                        assembly_digest,
                    )?;
                }
            }
        }
        Ok(())
    }
}
