//! Exact Swarm enemy identities composed over released combat definitions.

use std::sync::Arc;

use starclock_combat::{
    EnemyDefinitionId,
    catalog::{CombatCatalog, builder::CombatCatalogBuilder, definition::EnemyDefinition},
};

use crate::{catalog::UniverseCatalog, digest::Encoder, error::UniverseCatalogLoadError};

use super::{encounter_runtime::EncounterRuntimeCatalog, validate::reference};

pub(super) const SWARM_DISASTER_ENEMY_DEFINITION_REVISION: &str =
    "swarm-disaster-enemy-definition-composition-v1";

const MODE_ENEMY_BASE: u32 = 0x7f70_0000;
const EXPECTED_ENEMIES: usize = 71;

// These released variants are absent from the shared Standard combat catalog.
// Each keeps its exact Swarm identity while borrowing the reviewed same-family
// behavior definition shown here. The True Sting row has no released shared
// implementation and therefore uses the explicit reviewed boss-rank donor.
const MODE_ENEMIES: [(&str, &str); 12] = [
    (
        "enemy.automaton-direwolf-bug.elite.variant.01",
        "enemy.automaton-direwolf.elite.variant.01",
    ),
    (
        "enemy.automaton-grizzly-bug.elite.variant.01",
        "enemy.automaton-grizzly.elite.variant.01",
    ),
    (
        "enemy.blaze-out-of-space-bug.elite.variant.01",
        "enemy.blaze-out-of-space.elite.variant.01",
    ),
    (
        "enemy.decaying-shadow-bug.elite.variant.01",
        "enemy.decaying-shadow.elite.variant.01",
    ),
    (
        "enemy.frigid-prowler-bug.elite.variant.01",
        "enemy.frigid-prowler.elite.variant.01",
    ),
    (
        "enemy.guardian-shadow-bug.elite.variant.01",
        "enemy.guardian-shadow.elite.variant.01",
    ),
    (
        "enemy.ice-out-of-space-bug.elite.variant.01",
        "enemy.ice-out-of-space.elite.variant.01",
    ),
    (
        "enemy.searing-prowler-bug.elite.variant.01",
        "enemy.searing-prowler.elite.variant.01",
    ),
    (
        "enemy.sequence-trotter.minionlv2.05.variant.01",
        "enemy.trotter-of-preservation.minionlv2.variant.01",
    ),
    (
        "enemy.silvermane-lieutenant.elite.variant.01",
        "enemy.silvermane-lieutenant-bug.elite.variant.01",
    ),
    (
        "enemy.swarm-true-sting-complete.littleboss.variant.01",
        "enemy.memory-zone-meme-something-unto-death-complete.littleboss.variant.01",
    ),
    (
        "enemy.the-ascended-bug.elite.variant.01",
        "enemy.the-ascended.elite.variant.01",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SwarmEnemyDefinitionBinding {
    stable_key: Box<str>,
    combat_enemy: EnemyDefinitionId,
    stat_source: EnemyDefinitionId,
    behavior_source_key: Box<str>,
    mode_owned: bool,
}

impl SwarmEnemyDefinitionBinding {
    pub(super) const fn combat_enemy(&self) -> EnemyDefinitionId {
        self.combat_enemy
    }

    pub(super) const fn stat_source(&self) -> EnemyDefinitionId {
        self.stat_source
    }
}

#[derive(Clone, Debug)]
pub(super) struct SwarmBattleCatalogComposition {
    combat: Arc<CombatCatalog>,
    enemies: Box<[SwarmEnemyDefinitionBinding]>,
    digest: [u8; 32],
}

impl SwarmBattleCatalogComposition {
    pub(super) fn compile(
        encounters: &EncounterRuntimeCatalog,
        standard: &UniverseCatalog,
        base: &CombatCatalog,
    ) -> Result<Self, UniverseCatalogLoadError> {
        let keys = encounters.enemy_keys();
        if keys.len() != EXPECTED_ENEMIES {
            return Err(reference("Swarm battle enemy denominator drift"));
        }
        let mut bindings = Vec::with_capacity(keys.len());
        let mut aliases = Vec::with_capacity(MODE_ENEMIES.len());
        for key in keys {
            if let Some(enemy) = standard.simulation_catalog().enemy_by_stable_key(key) {
                bindings.push(SwarmEnemyDefinitionBinding {
                    stable_key: key.into(),
                    combat_enemy: enemy.id(),
                    stat_source: enemy.id(),
                    behavior_source_key: key.into(),
                    mode_owned: false,
                });
                continue;
            }
            let (ordinal, (_, donor_key)) = MODE_ENEMIES
                .iter()
                .enumerate()
                .find(|(_, (candidate, _))| *candidate == key)
                .ok_or_else(|| reference("unmapped Swarm enemy definition"))?;
            let donor = standard
                .simulation_catalog()
                .enemy_by_stable_key(donor_key)
                .ok_or_else(|| reference("missing Swarm enemy behavior donor"))?;
            let donor_definition = base
                .enemy(donor.id())
                .ok_or_else(|| reference("missing Swarm donor combat definition"))?;
            let raw = u32::try_from(ordinal + 1)
                .map_err(|_| reference("Swarm enemy identity overflow"))?;
            let id = EnemyDefinitionId::new(MODE_ENEMY_BASE + raw)
                .ok_or_else(|| reference("invalid Swarm enemy identity"))?;
            aliases.push(clone_definition(donor_definition, id)?);
            bindings.push(SwarmEnemyDefinitionBinding {
                stable_key: key.into(),
                combat_enemy: id,
                stat_source: donor.id(),
                behavior_source_key: (*donor_key).into(),
                mode_owned: true,
            });
        }
        bindings.sort_unstable_by(|left, right| left.stable_key.cmp(&right.stable_key));
        if bindings.len() != EXPECTED_ENEMIES
            || bindings.iter().filter(|binding| binding.mode_owned).count() != MODE_ENEMIES.len()
            || bindings
                .windows(2)
                .any(|pair| pair[0].stable_key >= pair[1].stable_key)
        {
            return Err(reference("Swarm enemy composition closure drift"));
        }
        let digest = composition_digest(&bindings);
        let mut builder = CombatCatalogBuilder::from_catalog(base, digest);
        for alias in aliases {
            builder.add_enemy(alias);
        }
        let combat = builder
            .build()
            .map_err(|_| reference("invalid Swarm battle combat catalog"))?;
        Ok(Self {
            combat,
            enemies: bindings.into_boxed_slice(),
            digest,
        })
    }

    pub(super) const fn combat(&self) -> &Arc<CombatCatalog> {
        &self.combat
    }

    pub(super) fn enemy(&self, key: &str) -> Option<&SwarmEnemyDefinitionBinding> {
        self.enemies
            .binary_search_by(|binding| binding.stable_key.as_ref().cmp(key))
            .ok()
            .map(|index| &self.enemies[index])
    }

    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    #[cfg(test)]
    pub(super) fn summary(&self) -> (usize, usize, [u8; 32]) {
        (
            self.enemies.len(),
            self.enemies
                .iter()
                .filter(|binding| binding.mode_owned)
                .count(),
            self.digest,
        )
    }

    #[cfg(test)]
    pub(super) fn runtime_stat_summary(
        &self,
        standard: &UniverseCatalog,
        level: starclock_combat::UnitLevel,
    ) -> (usize, usize) {
        let reviewed = self
            .enemies
            .iter()
            .filter(|binding| {
                standard
                    .simulation_catalog()
                    .enemy_runtime_stat(binding.stat_source, level, "standard-universe-v1")
                    .is_some()
            })
            .count();
        (reviewed, self.enemies.len() - reviewed)
    }
}

fn clone_definition(
    donor: &EnemyDefinition,
    id: EnemyDefinitionId,
) -> Result<EnemyDefinition, UniverseCatalogLoadError> {
    let mut definition = EnemyDefinition::new(id, donor.unit(), donor.abilities().to_vec());
    if !donor.links().is_empty() {
        definition = definition
            .with_links(donor.links().to_vec())
            .ok_or_else(|| reference("invalid Swarm enemy links"))?;
    }
    if let Some(ai) = donor.ai_graph() {
        definition = definition
            .with_orchestration(ai, donor.phases().to_vec())
            .ok_or_else(|| reference("invalid Swarm enemy orchestration"))?;
    }
    Ok(definition)
}

fn composition_digest(bindings: &[SwarmEnemyDefinitionBinding]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.enemy-composition.v1");
    encoder.text(SWARM_DISASTER_ENEMY_DEFINITION_REVISION);
    encoder.u32(u32::try_from(bindings.len()).expect("enemy identity count is bounded"));
    for binding in bindings {
        encoder.text(&binding.stable_key);
        encoder.u32(binding.combat_enemy.get());
        encoder.u32(binding.stat_source.get());
        encoder.text(&binding.behavior_source_key);
        encoder.u8(u8::from(binding.mode_owned));
    }
    encoder.finish()
}
