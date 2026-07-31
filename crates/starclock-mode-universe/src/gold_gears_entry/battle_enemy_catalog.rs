//! Mode-owned exact enemy identities composed over released combat definitions.

use std::{collections::BTreeSet, sync::Arc};

use starclock_combat::{
    EnemyDefinitionId,
    catalog::{CombatCatalog, builder::CombatCatalogBuilder, definition::EnemyDefinition},
};

use crate::{
    catalog::UniverseCatalog, digest::Encoder, gold_gears_content::GoldAndGearsContentCatalog,
};

use super::GoldAndGearsEntryError;

pub const GOLD_AND_GEARS_ENEMY_DEFINITION_REVISION: &str =
    "gold-and-gears-enemy-definition-composition-v1";

const MODE_ENEMY_BASE: u32 = 0x7f40_0000;
const EXPECTED_ENEMIES: usize = 90;
const EXPECTED_MODE_OWNED: usize = 23;

// The released structured variants retain distinct source identities. Their
// executable behavior is the reviewed same-family definition where available;
// identities without a same-variant definition use the explicit reviewed
// behavior source listed here. This mapping is mode-owned and visible.
const MODE_ENEMIES: [(&str, &str); EXPECTED_MODE_OWNED] = [
    (
        "enemy.abundant-ebon-deer-complete.littleboss.02.variant.01",
        "enemy.abundant-ebon-deer-complete.littleboss.variant.01",
    ),
    (
        "enemy.argenti-complete.littleboss.variant.01",
        "enemy.cocolia-complete.littleboss.variant.01",
    ),
    (
        "enemy.aurumaton-spectral-envoy-bug.elite.variant.01",
        "enemy.aurumaton-spectral-envoy.elite.variant.01",
    ),
    (
        "enemy.automaton-direwolf-bug.elite.variant.01",
        "enemy.automaton-direwolf.elite.variant.01",
    ),
    (
        "enemy.automaton-direwolf-complete.elite.03.variant.01",
        "enemy.automaton-direwolf-complete.elite.variant.01",
    ),
    (
        "enemy.automaton-grizzly-bug.elite.variant.01",
        "enemy.automaton-grizzly.elite.variant.01",
    ),
    (
        "enemy.automaton-grizzly-complete.elite.03.variant.01",
        "enemy.automaton-grizzly-complete.elite.variant.01",
    ),
    (
        "enemy.blaze-out-of-space-bug.elite.variant.01",
        "enemy.blaze-out-of-space.elite.variant.01",
    ),
    (
        "enemy.cloud-knight-lieutenant-yanqing-complete.littleboss.02.variant.01",
        "enemy.cloud-knight-lieutenant-yanqing-complete.littleboss.variant.01",
    ),
    (
        "enemy.cocolia-complete.littleboss.02.variant.01",
        "enemy.cocolia-complete.littleboss.variant.01",
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
        "enemy.gepard-complete.littleboss.02.variant.01",
        "enemy.gepard-complete.littleboss.variant.01",
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
        "enemy.stellaron-hunter-kafka-complete.littleboss.02.variant.01",
        "enemy.stellaron-hunter-kafka-complete.littleboss.variant.01",
    ),
    (
        "enemy.swarm-true-sting-complete.littleboss.02.variant.01",
        "enemy.memory-zone-meme-something-unto-death-complete.littleboss.variant.01",
    ),
    (
        "enemy.swarm-true-sting-complete.littleboss.variant.01",
        "enemy.memory-zone-meme-something-unto-death-complete.littleboss.variant.01",
    ),
    (
        "enemy.svarog-complete.littleboss.02.variant.01",
        "enemy.svarog-complete.littleboss.variant.01",
    ),
    (
        "enemy.the-ascended-bug.elite.variant.01",
        "enemy.the-ascended.elite.variant.01",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEnemyDefinitionBinding {
    stable_key: Box<str>,
    combat_enemy: EnemyDefinitionId,
    stat_source: EnemyDefinitionId,
    behavior_source_key: Box<str>,
    mode_owned: bool,
}

impl GoldAndGearsEnemyDefinitionBinding {
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    pub const fn combat_enemy(&self) -> EnemyDefinitionId {
        self.combat_enemy
    }
    pub const fn stat_source(&self) -> EnemyDefinitionId {
        self.stat_source
    }
    pub fn behavior_source_key(&self) -> &str {
        &self.behavior_source_key
    }
    pub const fn mode_owned(&self) -> bool {
        self.mode_owned
    }
}

#[derive(Clone, Debug)]
pub(super) struct GoldAndGearsBattleCatalogComposition {
    combat: Arc<CombatCatalog>,
    enemies: Box<[GoldAndGearsEnemyDefinitionBinding]>,
    digest: [u8; 32],
}

impl GoldAndGearsBattleCatalogComposition {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
        standard: &UniverseCatalog,
        base: &CombatCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        let keys = content
            .enemy_slots
            .iter()
            .map(|slot| slot.enemy.as_str())
            .collect::<BTreeSet<_>>();
        if keys.len() != EXPECTED_ENEMIES {
            return Err(GoldAndGearsEntryError::InvalidBattleMaterialization);
        }
        let mut bindings = Vec::with_capacity(keys.len());
        let mut aliases = Vec::with_capacity(EXPECTED_MODE_OWNED);
        for key in keys.iter().copied() {
            if let Some(enemy) = standard.simulation_catalog().enemy_by_stable_key(key) {
                bindings.push(GoldAndGearsEnemyDefinitionBinding {
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
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let donor = standard
                .simulation_catalog()
                .enemy_by_stable_key(donor_key)
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let donor_definition = base
                .enemy(donor.id())
                .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            let id = EnemyDefinitionId::new(
                MODE_ENEMY_BASE
                    + u32::try_from(ordinal + 1)
                        .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?,
            )
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
            aliases.push(clone_definition(donor_definition, id)?);
            bindings.push(GoldAndGearsEnemyDefinitionBinding {
                stable_key: key.into(),
                combat_enemy: id,
                stat_source: donor.id(),
                behavior_source_key: (*donor_key).into(),
                mode_owned: true,
            });
        }
        bindings.sort_unstable_by(|left, right| left.stable_key.cmp(&right.stable_key));
        if bindings.len() != EXPECTED_ENEMIES
            || bindings.iter().filter(|binding| binding.mode_owned).count() != EXPECTED_MODE_OWNED
            || bindings
                .windows(2)
                .any(|pair| pair[0].stable_key >= pair[1].stable_key)
        {
            return Err(GoldAndGearsEntryError::InvalidBattleMaterialization);
        }
        let digest = composition_digest(&bindings);
        let mut builder = CombatCatalogBuilder::from_catalog(
            base,
            format!(
                "{}+{}",
                base.revision().as_str(),
                GOLD_AND_GEARS_ENEMY_DEFINITION_REVISION
            ),
            digest,
        );
        for alias in aliases {
            builder.add_enemy(alias);
        }
        let combat = builder
            .build()
            .map_err(|_| GoldAndGearsEntryError::InvalidBattleMaterialization)?;
        Ok(Self {
            combat,
            enemies: bindings.into_boxed_slice(),
            digest,
        })
    }

    pub(super) const fn combat(&self) -> &Arc<CombatCatalog> {
        &self.combat
    }
    pub(super) fn enemies(&self) -> &[GoldAndGearsEnemyDefinitionBinding] {
        &self.enemies
    }
    pub(super) fn enemy(&self, key: &str) -> Option<&GoldAndGearsEnemyDefinitionBinding> {
        self.enemies
            .binary_search_by(|binding| binding.stable_key.as_ref().cmp(key))
            .ok()
            .map(|index| &self.enemies[index])
    }
    pub(super) const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

fn clone_definition(
    donor: &EnemyDefinition,
    id: EnemyDefinitionId,
) -> Result<EnemyDefinition, GoldAndGearsEntryError> {
    let mut definition = EnemyDefinition::new(id, donor.unit(), donor.abilities().to_vec());
    if !donor.links().is_empty() {
        definition = definition
            .with_links(donor.links().to_vec())
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    }
    if let Some(ai) = donor.ai_graph() {
        definition = definition
            .with_orchestration(ai, donor.phases().to_vec())
            .ok_or(GoldAndGearsEntryError::InvalidBattleMaterialization)?;
    }
    Ok(definition)
}

fn composition_digest(bindings: &[GoldAndGearsEnemyDefinitionBinding]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.enemy-composition.v1");
    encoder.u32(bindings.len() as u32);
    for binding in bindings {
        encoder.text(&binding.stable_key);
        encoder.u32(binding.combat_enemy.get());
        encoder.u32(binding.stat_source.get());
        encoder.text(&binding.behavior_source_key);
        encoder.u8(u8::from(binding.mode_owned));
    }
    encoder.finish()
}
