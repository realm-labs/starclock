//! Normal equipment composes the independently authored clauses immutably.
use super::{HpLossPointPolicy, bind_hp_loss_point_policy, invalid};
use crate::{
    digest::Encoder,
    divergent_universe::{
        DivergentUniverseBattleAssemblyError,
        weighted_curio::{WeightedCurioRuntime, WeightedCurioSnapshot},
        weighted_curio_footstep::skill_damage::{SkillDamagePolicy, bind_skill_damage_policy},
    },
};
use starclock_combat::{ParticipantSpec, Scalar, catalog::builder::CombatCatalogBuilder};
use starclock_data::{
    catalog::SimulationCatalog,
    divergent_universe_decisions::weighted_curio_footsteps::WeightedCurioFootstepPolicy,
};

impl WeightedCurioRuntime {
    pub(in crate::divergent_universe) fn assemble_footsteps(
        &self,
        builder: &mut CombatCatalogBuilder,
        snapshot: &WeightedCurioSnapshot,
        core: &SimulationCatalog,
        players: &mut [ParticipantSpec],
        assembly_digest: [u8; 32],
    ) -> Result<(), DivergentUniverseBattleAssemblyError> {
        for definition in &self.footsteps {
            if !snapshot.equipped().contains(&definition.weighted_curio) {
                continue;
            }
            let WeightedCurioFootstepPolicy::VersionedProjectPolicyEffectiveHpLossAfterSkillOriginalDamage = definition.policy;
            let mut digest = Encoder::new(b"starclock.divergent-universe.footstep.authored-policy");
            digest.digest(assembly_digest);
            digest.text(&definition.key);
            digest.text(&definition.hp_policy_note);
            digest.text(&definition.hp_replacement_condition);
            digest.text(&definition.skill_policy_note);
            digest.text(&definition.skill_replacement_condition);
            for source in &definition.sources {
                digest.text(source);
            }
            let identity = digest.finish();
            let hp = HpLossPointPolicy::new(
                Scalar::from_scaled(definition.loss_fraction_millionths),
                identity,
            )
            .map_err(|_| invalid())?;
            let skill = SkillDamagePolicy::new(
                Scalar::from_scaled(definition.damage_per_stack_millionths),
                definition.maximum_stacks,
                identity,
            )
            .map_err(|_| invalid())?;
            if players.is_empty() || players.len() > 4 {
                return Err(invalid());
            }
            for player in &mut *players {
                let with_hp =
                    bind_hp_loss_point_policy(builder, core, player, &hp, assembly_digest)?;
                *player =
                    bind_skill_damage_policy(builder, core, &with_hp, &skill, assembly_digest)?;
            }
        }
        Ok(())
    }
}
