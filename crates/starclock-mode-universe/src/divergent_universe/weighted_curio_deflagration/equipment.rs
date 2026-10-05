//! Normal assembly binds the authored policy only for explicitly equipped Curios.
use crate::divergent_universe::{
    DivergentUniverseBattleAssemblyError,
    contribution_snapshot::DivergentUniverseDifficultyProtocolSnapshot,
    weighted_curio::{
        WeightedCurioRuntime, WeightedCurioSnapshot,
        deflagration::native::bind_mapped_deflagration_policy,
    },
};
use starclock_combat::{ParticipantSpec, catalog::builder::CombatCatalogBuilder};
use starclock_data::catalog::SimulationCatalog;

impl WeightedCurioRuntime {
    pub(in crate::divergent_universe) fn assemble_deflagrations(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        protocol: &DivergentUniverseDifficultyProtocolSnapshot,
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for definition in &self.deflagrations {
            if snapshot.equipped().contains(&definition.weighted_curio) {
                let bound = bind_mapped_deflagration_policy(
                    builder,
                    definition,
                    core,
                    players,
                    protocol,
                    assembly_digest,
                )?;
                players.clone_from_slice(&bound);
            }
        }
        Ok(())
    }
}
