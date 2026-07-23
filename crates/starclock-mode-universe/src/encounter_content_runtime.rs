//! Executable encounter-selection, composition and overlay-closure contract.

use std::collections::BTreeSet;

use crate::{
    battle_overlay::UniverseEncounterOverlay,
    catalog::UniverseCatalog,
    definition::DomainKind,
    digest::Encoder,
    encounter::{EncounterGroupDefinition, EncounterSelectionPolicy, EnemyRole, WavePolicy},
    id::{DifficultyId, EncounterGroupId, EncounterPoolId},
    path::ExactParameter,
};

pub const ENCOUNTER_CONTENT_RUNTIME_REVISION: &str =
    "standard-universe-encounter-content-runtime-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedEncounterGroup {
    group: EncounterGroupId,
    weight: ExactParameter,
}

impl WeightedEncounterGroup {
    #[must_use]
    pub const fn group(&self) -> EncounterGroupId {
        self.group
    }
    #[must_use]
    pub const fn weight(&self) -> ExactParameter {
        self.weight
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EncounterSelection {
    WeightedGroups(Box<[WeightedEncounterGroup]>),
    FixedContent {
        source_content_id: Box<str>,
    },
    DifficultyEnemy {
        role: EnemyRole,
        enemy_variant_key: Box<str>,
        level: u32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterContentRuntimeCatalog {
    enemy_variant_keys: Box<[Box<str>]>,
    member_ids: Box<[crate::id::EncounterMemberId]>,
    digest: [u8; 32],
}

impl EncounterContentRuntimeCatalog {
    pub fn compile(catalog: &UniverseCatalog) -> Result<Self, EncounterContentRuntimeError> {
        let mut member_ids = catalog
            .encounter_groups()
            .iter()
            .flat_map(|group| group.members())
            .map(|member| member.id())
            .collect::<Vec<_>>();
        member_ids.sort_unstable();
        let member_count = member_ids.len();
        let wave_count = catalog
            .encounter_groups()
            .iter()
            .flat_map(|group| group.members())
            .map(|member| member.waves().len())
            .sum::<usize>();
        let enemy_slot_count = catalog
            .encounter_groups()
            .iter()
            .flat_map(|group| group.members())
            .flat_map(|member| member.waves())
            .map(|wave| wave.enemies().len())
            .sum::<usize>();
        let mut enemy_variant_keys = BTreeSet::new();
        for slot in catalog
            .encounter_groups()
            .iter()
            .flat_map(|group| group.members())
            .flat_map(|member| member.waves())
            .flat_map(|wave| wave.enemies())
        {
            enemy_variant_keys.insert(Box::<str>::from(slot.enemy_variant_key()));
        }
        for binding in catalog.difficulty_enemy_bindings() {
            enemy_variant_keys.insert(Box::<str>::from(binding.enemy_variant_key()));
        }
        let topology_node_count = catalog
            .topologies()
            .iter()
            .map(|topology| topology.nodes().len())
            .sum::<usize>();
        let content_pool_entry_count = catalog
            .content_pools()
            .iter()
            .map(|pool| pool.entries().len())
            .sum::<usize>();
        if catalog.domains().len() != 9
            || catalog.encounter_groups().len() != 74
            || catalog.encounter_pools().len() != 92
            || topology_node_count != 579
            || catalog.rooms().len() != 163
            || catalog.worlds().len() != 9
            || catalog.difficulties().len() != 33
            || member_count != 173
            || wave_count != 173
            || enemy_slot_count != 538
            || catalog.difficulty_enemy_bindings().len() != 182
            || catalog.room_content().len() != 380
            || catalog.content_pools().len() != 23
            || content_pool_entry_count != 1_651
            || enemy_variant_keys.len() != 86
            || member_ids.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(EncounterContentRuntimeError::InvalidDenominator);
        }
        validate_groups(catalog.encounter_groups())?;
        let enemy_variant_keys = enemy_variant_keys.into_iter().collect::<Vec<_>>();
        let digest = catalog_digest(catalog, &enemy_variant_keys);
        Ok(Self {
            enemy_variant_keys: enemy_variant_keys.into_boxed_slice(),
            member_ids: member_ids.into_boxed_slice(),
            digest,
        })
    }

    #[must_use]
    pub const fn content_count(&self) -> usize {
        959
    }
    #[must_use]
    pub const fn rule_count(&self) -> usize {
        0
    }
    #[must_use]
    pub const fn semantic_fixture_count(&self) -> usize {
        4
    }
    #[must_use]
    pub const fn bundled_enemy_definition_count(&self) -> usize {
        13
    }
    #[must_use]
    pub const fn extension_enemy_definition_count(&self) -> usize {
        73
    }
    #[must_use]
    pub fn enemy_variant_keys(&self) -> &[Box<str>] {
        &self.enemy_variant_keys
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub fn validate_overlay(
        &self,
        overlay: &UniverseEncounterOverlay,
    ) -> Result<(), EncounterContentRuntimeError> {
        if overlay.bindings().len() != self.member_ids.len()
            || !overlay
                .bindings()
                .iter()
                .map(|binding| binding.member())
                .eq(self.member_ids.iter().copied())
        {
            return Err(EncounterContentRuntimeError::IncompleteBattleOverlay);
        }
        Ok(())
    }

    pub fn resolve(
        &self,
        catalog: &UniverseCatalog,
        pool: EncounterPoolId,
        condition_key: &str,
        difficulty: DifficultyId,
    ) -> Result<EncounterSelection, EncounterContentRuntimeError> {
        let pool = catalog
            .encounter_pool(pool)
            .ok_or(EncounterContentRuntimeError::UnknownPool)?;
        if catalog.difficulty(difficulty).is_none() {
            return Err(EncounterContentRuntimeError::UnknownDifficulty);
        }
        match pool.selection_policy() {
            EncounterSelectionPolicy::WorldDifficultyBossEliteBinding => {
                let role = role_for_domain(pool.domain_kind())?;
                let binding = catalog
                    .difficulty_enemy_bindings()
                    .iter()
                    .find(|binding| binding.difficulty() == difficulty && binding.role() == role)
                    .ok_or(EncounterContentRuntimeError::MissingDifficultyEnemy)?;
                Ok(EncounterSelection::DifficultyEnemy {
                    role,
                    enemy_variant_key: binding.enemy_variant_key().into(),
                    level: binding.level(),
                })
            }
            EncounterSelectionPolicy::ExactConditionThenWeightedStableOrder => {
                weighted_groups(pool.weighted(), condition_key)
            }
            EncounterSelectionPolicy::ConditionThenGroupOrDifficultyBinding => {
                let weighted = pool
                    .weighted()
                    .iter()
                    .filter(|binding| binding.condition_key() == condition_key)
                    .map(|binding| WeightedEncounterGroup {
                        group: binding.group(),
                        weight: binding.weight(),
                    })
                    .collect::<Vec<_>>();
                if !weighted.is_empty() {
                    return Ok(EncounterSelection::WeightedGroups(
                        weighted.into_boxed_slice(),
                    ));
                }
                let fixed = pool
                    .fixed()
                    .iter()
                    .find(|binding| binding.condition_key() == condition_key)
                    .ok_or(EncounterContentRuntimeError::ConditionNotOffered)?;
                Ok(EncounterSelection::FixedContent {
                    source_content_id: fixed.source_content_id().into(),
                })
            }
        }
    }
}

fn weighted_groups(
    bindings: &[crate::encounter::WeightedEncounterBinding],
    condition_key: &str,
) -> Result<EncounterSelection, EncounterContentRuntimeError> {
    let values = bindings
        .iter()
        .filter(|binding| binding.condition_key() == condition_key)
        .map(|binding| WeightedEncounterGroup {
            group: binding.group(),
            weight: binding.weight(),
        })
        .collect::<Vec<_>>();
    if values.is_empty() {
        return Err(EncounterContentRuntimeError::ConditionNotOffered);
    }
    Ok(EncounterSelection::WeightedGroups(
        values.into_boxed_slice(),
    ))
}

fn role_for_domain(domain: DomainKind) -> Result<EnemyRole, EncounterContentRuntimeError> {
    match domain {
        DomainKind::Boss => Ok(EnemyRole::Boss),
        DomainKind::Elite => Ok(EnemyRole::Elite),
        _ => Err(EncounterContentRuntimeError::InvalidDifficultyPolicy),
    }
}

fn validate_groups(
    groups: &[EncounterGroupDefinition],
) -> Result<(), EncounterContentRuntimeError> {
    for group in groups {
        if group.members().is_empty() {
            return Err(EncounterContentRuntimeError::InvalidEncounterGroup);
        }
        for member in group.members() {
            if member.waves().is_empty()
                || member.waves().iter().any(|wave| wave.enemies().is_empty())
                || (group.wave_policy() == WavePolicy::SingleWave && member.waves().len() != 1)
            {
                return Err(EncounterContentRuntimeError::InvalidEncounterGroup);
            }
        }
    }
    Ok(())
}

fn catalog_digest(catalog: &UniverseCatalog, enemy_keys: &[Box<str>]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock-universe-encounter-content-runtime-catalog-v1");
    encoder.text(ENCOUNTER_CONTENT_RUNTIME_REVISION);
    encoder.digest(catalog.identity().definitions_digest().bytes());
    encoder.digest(catalog.identity().encounter_definitions_digest().bytes());
    encoder.u32(enemy_keys.len() as u32);
    for key in enemy_keys {
        encoder.text(key);
    }
    encoder.finish()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncounterContentRuntimeError {
    InvalidDenominator,
    InvalidEncounterGroup,
    IncompleteBattleOverlay,
    UnknownPool,
    UnknownDifficulty,
    ConditionNotOffered,
    MissingDifficultyEnemy,
    InvalidDifficultyPolicy,
}
