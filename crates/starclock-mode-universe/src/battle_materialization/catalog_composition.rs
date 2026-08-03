//! Immutable encounter and enemy definition composition shared by assemblies.

use crate::occurrence_battle::compile as occurrence_battle_compile;
use std::{collections::BTreeMap, sync::Arc};

use starclock_combat::catalog::{CombatCatalog, builder::CombatCatalogBuilder};

use crate::{catalog::UniverseCatalog, encounter_content_runtime::EncounterContentRuntimeCatalog};

use super::{
    DIFFICULTY_BINDING_COUNT, ENEMY_VARIANT_COUNT, EXACT_ENEMY_VARIANT_COUNT, MEMBER_COUNT,
    MEMBER_ENEMY_SLOT_COUNT, UniverseBattleMaterializationError, UniverseEnemyMaterialization,
    difficulty_encounter, materialization_digest::catalog_composition_digest, materialize_enemies,
    member_encounter, members,
};
use super::{EnemyDefinitionMatch, proxy_key};

/// Definitions that depend only on the released catalog, never on one run.
///
/// Construction performs the expensive encounter/enemy composition once.
/// Per-battle assembly starts from this validated immutable catalog and only
/// selects contribution definitions and runtime values.
#[derive(Clone, Debug)]
pub struct UniverseBattleCatalogComposition {
    combat_catalog: Arc<CombatCatalog>,
    content: EncounterContentRuntimeCatalog,
    enemies: Box<[UniverseEnemyMaterialization]>,
    digest: [u8; 32],
}

impl UniverseBattleCatalogComposition {
    pub fn compile(universe: &UniverseCatalog) -> Result<Self, UniverseBattleMaterializationError> {
        let content = EncounterContentRuntimeCatalog::compile(universe)
            .map_err(|_| UniverseBattleMaterializationError::InvalidEncounterContent)?;
        let enemies = materialize_enemies(universe, &content)?;
        validate_denominators(universe, &enemies)?;
        let occurrence_battles = occurrence_battle_compile(universe)
            .map_err(|_| UniverseBattleMaterializationError::InvalidEncounterContent)?;
        let mut enemy_map = enemies
            .iter()
            .map(|enemy| (enemy.stable_key(), enemy.combat_enemy()))
            .collect::<BTreeMap<_, _>>();
        for battle in &occurrence_battles {
            for slot in battle
                .member()
                .waves()
                .iter()
                .flat_map(|wave| wave.enemies())
            {
                let enemy = universe
                    .simulation_catalog()
                    .enemy_by_stable_key(slot.enemy_variant_key())
                    .or_else(|| {
                        universe
                            .simulation_catalog()
                            .enemy_by_stable_key(proxy_key(slot.enemy_variant_key()))
                    })
                    .ok_or(UniverseBattleMaterializationError::MissingEnemyMapping)?;
                enemy_map.insert(slot.enemy_variant_key(), enemy.id());
            }
        }
        let digest = catalog_composition_digest(universe, content.digest(), &enemies);
        let mut builder = CombatCatalogBuilder::from_catalog(
            universe.simulation_catalog().combat_catalog(),
            digest,
        );
        for member in members(universe) {
            builder.add_encounter(member_encounter(
                member,
                &enemy_map,
                universe.simulation_catalog().combat_catalog(),
            )?);
        }
        for battle in &occurrence_battles {
            builder.add_encounter(member_encounter(
                battle.member(),
                &enemy_map,
                universe.simulation_catalog().combat_catalog(),
            )?);
        }
        for (index, binding) in universe.difficulty_enemy_bindings().iter().enumerate() {
            builder.add_encounter(difficulty_encounter(
                index,
                binding,
                &enemy_map,
                universe.simulation_catalog().combat_catalog(),
            )?);
        }
        let combat_catalog = builder
            .build()
            .map_err(|_| UniverseBattleMaterializationError::InvalidCompositeCatalog)?;
        Ok(Self {
            combat_catalog,
            content,
            enemies: enemies.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn combat_catalog(&self) -> &Arc<CombatCatalog> {
        &self.combat_catalog
    }

    #[must_use]
    pub const fn content(&self) -> &EncounterContentRuntimeCatalog {
        &self.content
    }

    #[must_use]
    pub fn enemies(&self) -> &[UniverseEnemyMaterialization] {
        &self.enemies
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

fn validate_denominators(
    universe: &UniverseCatalog,
    enemies: &[UniverseEnemyMaterialization],
) -> Result<(), UniverseBattleMaterializationError> {
    let member_count = members(universe).count();
    let enemy_slot_count = members(universe)
        .flat_map(|member| member.waves())
        .map(|wave| wave.enemies().len())
        .sum::<usize>();
    let exact = enemies
        .iter()
        .filter(|enemy| enemy.definition_match() == EnemyDefinitionMatch::Exact)
        .count();
    if member_count != MEMBER_COUNT
        || enemy_slot_count != MEMBER_ENEMY_SLOT_COUNT
        || universe.difficulty_enemy_bindings().len() != DIFFICULTY_BINDING_COUNT
        || enemies.len() != ENEMY_VARIANT_COUNT
        || exact != EXACT_ENEMY_VARIANT_COUNT
    {
        return Err(UniverseBattleMaterializationError::InvalidDenominator);
    }
    Ok(())
}
