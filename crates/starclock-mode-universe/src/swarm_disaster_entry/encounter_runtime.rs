//! Deterministic Swarm encounter-family and effective-level resolution.

use std::{collections::BTreeSet, sync::Arc};

use starclock_activity::{ActivityRngLabel, ActivityRngStreams, ActivityTransactionState, NodeId};

use crate::{
    digest::Encoder,
    error::{UniverseCatalogLoadError, UniverseCatalogLoadErrorKind},
    swarm_disaster_content::encounter_access::{
        BossPoolInput, EncounterGroupInput, EncounterRuntimeInput, EncounterWaveInput,
        EnemySlotInput,
    },
    swarm_disaster_structural::entry_access::SwarmDisasterEncounterStructuralInput,
};

use super::SwarmDisasterRuntimeInstance;

pub(crate) const SWARM_DISASTER_ENCOUNTER_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";
pub(crate) const SWARM_DISASTER_ENCOUNTER_POLICY_REPLACEMENT_CONDITION: &str = "released engine code or pinned tables expose the exact ChessRogue room/domain/group join and effective battle-level selection sequence";

const EXPECTED_GROUPS: usize = 179;
const EXPECTED_MEMBERS: usize = 347;
const EXPECTED_WAVES: usize = 347;
const EXPECTED_SLOTS: usize = 1_070;
const EXPECTED_BOSS_POOLS: usize = 15;
const EXPECTED_DISTINCT_ENEMIES: usize = 71;
const EXPECTED_ROOMS_WITHOUT_JOIN: usize = 861;
const GROUP_PURPOSE: u16 = 0x6d01;
const MEMBER_PURPOSE: u16 = 0x6d02;

const NORMAL_DOMAIN: &str = "swarm-disaster.domain.monsternormal";
const ELITE_DOMAIN: &str = "swarm-disaster.domain.monsterelite";
const BOSS_DOMAIN: &str = "swarm-disaster.domain.monsterboss";
const SWARM_DOMAIN: &str = "swarm-disaster.domain.monsterswarm";
const SWARM_BOSS_DOMAIN: &str = "swarm-disaster.domain.monsterswarmboss";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EncounterRole {
    Combat,
    Elite,
    FirstPlaneBoss,
    SecondPlaneBoss,
    FinalBoss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EncounterEnemySlot {
    pub(super) key: Box<str>,
    pub(super) formation_index: u8,
    pub(super) enemy_variant: Box<str>,
    pub(super) boss_choices: Box<[Box<str>]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EncounterWave {
    pub(super) key: Box<str>,
    pub(super) ordinal: u16,
    pub(super) stage_type: Box<str>,
    pub(super) authored_level: u16,
    pub(super) hard_level_group: u16,
    pub(super) slots: Box<[EncounterEnemySlot]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct EncounterSelection {
    pub(super) group: Box<str>,
    pub(super) source_group_id: u32,
    pub(super) source_rogue_monster_id: Box<str>,
    pub(super) source_primary_monster_id: Box<str>,
    pub(super) source_stage_id: Box<str>,
    pub(super) role: EncounterRole,
    pub(super) difficulty_segment: Box<str>,
    pub(super) effective_level: u16,
    pub(super) waves: Box<[EncounterWave]>,
}

#[derive(Debug)]
pub(super) struct EncounterRuntimeCatalog {
    groups: Box<[RuntimeGroup]>,
    boss_pools: Box<[RuntimeBossPool]>,
}

#[derive(Debug)]
struct RuntimeGroup {
    key: Box<str>,
    source_group_id: u32,
    role: EncounterRole,
    areas: Box<[Box<str>]>,
    members: Box<[RuntimeMember]>,
}

#[derive(Debug)]
struct RuntimeMember {
    source_rogue_monster_id: Box<str>,
    source_primary_monster_id: Box<str>,
    source_stage_id: Box<str>,
    weight: u64,
    waves: Box<[EncounterWave]>,
}

#[derive(Debug)]
struct RuntimeBossPool {
    _id: u32,
    _key: Box<str>,
    difficulty: u8,
    area_id: u32,
    role: EncounterRole,
    candidates: Box<[Box<str>]>,
}

#[derive(Clone, Debug)]
struct DifficultyBand {
    key: Box<str>,
    cuts: Box<[u16]>,
    levels: Box<[u16]>,
}

#[derive(Clone, Copy, Debug)]
struct NodeBinding {
    node: NodeId,
    plane: u8,
    position: u16,
}

#[derive(Clone, Debug)]
pub(super) struct CompiledEncounterRuntime {
    catalog: Arc<EncounterRuntimeCatalog>,
    area_id: u32,
    area: Box<str>,
    difficulty: u8,
    bands: Box<[DifficultyBand]>,
    nodes: Box<[NodeBinding]>,
}

impl EncounterRuntimeCatalog {
    pub(super) fn compile(input: EncounterRuntimeInput) -> Result<Self, UniverseCatalogLoadError> {
        if input.groups.len() != EXPECTED_GROUPS
            || input.waves.len() != EXPECTED_WAVES
            || input.slots.len() != EXPECTED_SLOTS
            || input.boss_pools.len() != EXPECTED_BOSS_POOLS
        {
            return Err(invalid("Swarm encounter denominator drift"));
        }
        let mut used_waves = BTreeSet::new();
        let mut used_slots = BTreeSet::new();
        let mut enemies = BTreeSet::new();
        let mut groups = input
            .groups
            .iter()
            .map(|group| {
                compile_group(
                    group,
                    &input.waves,
                    &input.slots,
                    &mut used_waves,
                    &mut used_slots,
                    &mut enemies,
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        groups.sort_unstable_by_key(|group| group.source_group_id);
        let member_count = groups
            .iter()
            .map(|group| group.members.len())
            .sum::<usize>();
        if member_count != EXPECTED_MEMBERS
            || used_waves.len() != EXPECTED_WAVES
            || used_slots.len() != EXPECTED_SLOTS
            || enemies.len() != EXPECTED_DISTINCT_ENEMIES
            || groups
                .windows(2)
                .any(|pair| pair[0].source_group_id >= pair[1].source_group_id)
            || role_counts(&groups) != [103, 40, 30, 5, 1]
        {
            return Err(invalid("Swarm encounter closure drift"));
        }
        let mut boss_pools = input
            .boss_pools
            .iter()
            .map(|pool| compile_boss_pool(pool, &groups))
            .collect::<Result<Vec<_>, _>>()?;
        boss_pools.sort_unstable_by_key(|pool| (pool.difficulty, role_order(pool.role)));
        if !valid_boss_pool_matrix(&boss_pools) {
            return Err(invalid("Swarm boss-pool matrix drift"));
        }
        Ok(Self {
            groups: groups.into_boxed_slice(),
            boss_pools: boss_pools.into_boxed_slice(),
        })
    }

    pub(super) fn enemy_keys(&self) -> Box<[&str]> {
        let mut keys = self
            .groups
            .iter()
            .flat_map(|group| &group.members)
            .flat_map(|member| &member.waves)
            .flat_map(|wave| &wave.slots)
            .map(|slot| slot.enemy_variant.as_ref())
            .collect::<Vec<_>>();
        keys.sort_unstable();
        keys.dedup();
        keys.into_boxed_slice()
    }
}

impl CompiledEncounterRuntime {
    pub(super) fn compile(
        catalog: Arc<EncounterRuntimeCatalog>,
        input: SwarmDisasterEncounterStructuralInput,
    ) -> Result<Self, UniverseCatalogLoadError> {
        if input.bands.len() != 20
            || input.selected_band_keys.len() != 3
            || input.nodes.is_empty()
            || input.room_count != EXPECTED_ROOMS_WITHOUT_JOIN
            || input.difficulty == 0
            || input.difficulty > 5
        {
            return Err(invalid("Swarm encounter structural denominator drift"));
        }
        let catalog_bands = input
            .bands
            .into_vec()
            .into_iter()
            .map(|band| {
                if band.levels.len() != band.cuts.len() + 1
                    || band.cuts.windows(2).any(|pair| pair[0] >= pair[1])
                {
                    return Err(invalid("invalid Swarm difficulty segment"));
                }
                Ok(DifficultyBand {
                    key: band.key,
                    cuts: band.cuts,
                    levels: band.levels,
                })
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        let bands = input
            .selected_band_keys
            .iter()
            .map(|key| {
                catalog_bands
                    .iter()
                    .find(|band| band.key.as_ref() == key.as_ref())
                    .cloned()
                    .ok_or_else(|| reference("unknown Swarm selected difficulty segment"))
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        let nodes = input
            .nodes
            .iter()
            .map(|node| {
                Ok(NodeBinding {
                    node: NodeId::new(node.id)
                        .ok_or_else(|| invalid("invalid Swarm encounter node"))?,
                    plane: node.plane,
                    position: node.position,
                })
            })
            .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
        if nodes.windows(2).any(|pair| pair[0].node >= pair[1].node) {
            return Err(invalid("duplicate Swarm encounter node"));
        }
        Ok(Self {
            catalog,
            area_id: input.area_id,
            area: input.area_key,
            difficulty: input.difficulty,
            bands: bands.into_boxed_slice(),
            nodes: nodes.into_boxed_slice(),
        })
    }

    pub(super) fn select(
        &self,
        node: NodeId,
        domain: &str,
        rng: &mut ActivityRngStreams,
    ) -> Result<EncounterSelection, UniverseCatalogLoadError> {
        let binding = self
            .nodes
            .binary_search_by_key(&node, |binding| binding.node)
            .ok()
            .map(|index| self.nodes[index])
            .ok_or_else(|| reference("unknown Swarm encounter node"))?;
        let role = role_for_domain(domain, binding.plane)?;
        let band = self
            .bands
            .get(usize::from(binding.plane - 1))
            .ok_or_else(|| invalid("invalid Swarm encounter plane"))?;
        let bucket = band.cuts.partition_point(|cut| *cut <= binding.position);
        let effective_level = *band
            .levels
            .get(bucket)
            .ok_or_else(|| invalid("invalid Swarm effective encounter level"))?;
        let candidates = self.candidates(role)?;
        rng.transact(|rng| {
            let group_index = select_uniform(rng, GROUP_PURPOSE, candidates.len())?;
            let group = candidates[group_index];
            let member_index = select_weighted(rng, &group.members)?;
            let member = &group.members[member_index];
            Ok(EncounterSelection {
                group: group.key.clone(),
                source_group_id: group.source_group_id,
                source_rogue_monster_id: member.source_rogue_monster_id.clone(),
                source_primary_monster_id: member.source_primary_monster_id.clone(),
                source_stage_id: member.source_stage_id.clone(),
                role,
                difficulty_segment: band.key.clone(),
                effective_level,
                waves: member.waves.clone(),
            })
        })
    }

    fn candidates(
        &self,
        role: EncounterRole,
    ) -> Result<Vec<&RuntimeGroup>, UniverseCatalogLoadError> {
        let groups = if matches!(role, EncounterRole::Combat | EncounterRole::Elite) {
            self.catalog
                .groups
                .iter()
                .filter(|group| {
                    group.role == role
                        && group
                            .areas
                            .iter()
                            .any(|area| area.as_ref() == self.area.as_ref())
                })
                .collect::<Vec<_>>()
        } else {
            let pool = self
                .catalog
                .boss_pools
                .iter()
                .find(|pool| {
                    pool.area_id == self.area_id
                        && pool.difficulty == self.difficulty
                        && pool.role == role
                })
                .ok_or_else(|| reference("unresolved Swarm boss encounter pool"))?;
            pool.candidates
                .iter()
                .map(|key| {
                    self.catalog
                        .groups
                        .iter()
                        .find(|group| group.key.as_ref() == key.as_ref())
                        .ok_or_else(|| reference("unknown Swarm boss encounter candidate"))
                })
                .collect::<Result<Vec<_>, _>>()?
        };
        if groups.is_empty() {
            return Err(reference("no Swarm encounter candidates"));
        }
        Ok(groups)
    }

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize, usize, usize) {
        (
            EXPECTED_GROUPS,
            EXPECTED_MEMBERS,
            EXPECTED_WAVES,
            EXPECTED_SLOTS,
            EXPECTED_BOSS_POOLS,
        )
    }

    #[cfg(test)]
    pub(super) fn effective_level_at(&self, plane: u8, position: u16) -> Option<(&str, u16)> {
        let band = self.bands.get(usize::from(plane.checked_sub(1)?))?;
        let bucket = band.cuts.partition_point(|cut| *cut <= position);
        Some((band.key.as_ref(), *band.levels.get(bucket)?))
    }
}

impl SwarmDisasterRuntimeInstance {
    /// Selects the current resolved combat encounter and returns its canonical digest.
    ///
    /// This is the generated-type-free Phase 6 boundary before immutable
    /// `BattleSpec` materialization. It consumes only the labeled Encounter
    /// stream; unresolved nodes, non-combat domains and missing boss pools fail
    /// before any draw. The digest binds every selected 81-series wave and slot.
    pub fn select_current_encounter_digest(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<[u8; 32], UniverseCatalogLoadError> {
        self.select_current_encounter(state, rng)
            .map(|selection| selection_digest(&selection))
    }

    pub(super) fn select_current_encounter(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<EncounterSelection, UniverseCatalogLoadError> {
        let node = state.current_node();
        let domain = self
            .map
            .node_domain_key(state, node)?
            .ok_or_else(|| reference("unresolved Swarm encounter domain"))?;
        self.encounter_runtime.select(node, domain, rng)
    }
}

pub(super) fn selection_digest(selection: &EncounterSelection) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.encounter-selection");
    encoder.text(SWARM_DISASTER_ENCOUNTER_POLICY_ACCURACY);
    encoder.text(SWARM_DISASTER_ENCOUNTER_POLICY_REPLACEMENT_CONDITION);
    encoder.text(&selection.group);
    encoder.u32(selection.source_group_id);
    encoder.text(&selection.source_rogue_monster_id);
    encoder.text(&selection.source_primary_monster_id);
    encoder.text(&selection.source_stage_id);
    encoder.u8(role_order(selection.role));
    encoder.text(&selection.difficulty_segment);
    encoder.u32(u32::from(selection.effective_level));
    encoder.u32(u32::try_from(selection.waves.len()).expect("validated wave count fits u32"));
    for wave in &selection.waves {
        encoder.text(&wave.key);
        encoder.u32(u32::from(wave.ordinal));
        encoder.text(&wave.stage_type);
        encoder.u32(u32::from(wave.authored_level));
        encoder.u32(u32::from(wave.hard_level_group));
        encoder.u32(u32::try_from(wave.slots.len()).expect("validated slot count fits u32"));
        for slot in &wave.slots {
            encoder.text(&slot.key);
            encoder.u8(slot.formation_index);
            encoder.text(&slot.enemy_variant);
            encoder.u32(
                u32::try_from(slot.boss_choices.len())
                    .expect("validated boss-choice count fits u32"),
            );
            for choice in &slot.boss_choices {
                encoder.text(choice);
            }
        }
    }
    encoder.finish()
}

fn compile_group(
    group: &EncounterGroupInput,
    waves: &[EncounterWaveInput],
    slots: &[EnemySlotInput],
    used_waves: &mut BTreeSet<u32>,
    used_slots: &mut BTreeSet<u32>,
    enemies: &mut BTreeSet<Box<str>>,
) -> Result<RuntimeGroup, UniverseCatalogLoadError> {
    if group.room_key.is_some() || group.members.is_empty() || group.area_keys.len() != 5 {
        return Err(invalid("invalid Swarm encounter room/area binding"));
    }
    let role = parse_role(&group.role)?;
    let mut group_wave_keys = BTreeSet::new();
    let mut group_boss_choices = BTreeSet::new();
    let members = group
        .members
        .iter()
        .enumerate()
        .map(|(member_index, member)| {
            if usize::from(member.order) != member_index || member.wave_keys.is_empty() {
                return Err(invalid("invalid Swarm encounter member order"));
            }
            let member_waves = member
                .wave_keys
                .iter()
                .map(|wave_key| {
                    group_wave_keys.insert(wave_key.clone());
                    let wave = waves
                        .iter()
                        .find(|wave| wave.key.as_ref() == wave_key.as_ref())
                        .ok_or_else(|| reference("unknown Swarm encounter wave"))?;
                    if wave.group_id != group.id
                        || usize::from(wave.ordinal) != group_wave_keys.len()
                        || wave.stage_type.as_ref() != "VerseSimulation"
                        || !used_waves.insert(wave.id)
                    {
                        return Err(invalid("invalid Swarm encounter wave closure"));
                    }
                    compile_wave(wave, slots, used_slots, enemies, &mut group_boss_choices)
                })
                .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
            Ok(RuntimeMember {
                source_rogue_monster_id: member.source_rogue_monster_id.clone(),
                source_primary_monster_id: member.source_primary_monster_id.clone(),
                source_stage_id: member.source_stage_id.clone(),
                weight: member.weight,
                waves: member_waves.into_boxed_slice(),
            })
        })
        .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
    let declared_waves = group.wave_keys.iter().cloned().collect::<BTreeSet<_>>();
    let declared_bosses = group
        .boss_choice_keys
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if group_wave_keys != declared_waves || group_boss_choices != declared_bosses {
        return Err(invalid("Swarm encounter aggregate membership drift"));
    }
    Ok(RuntimeGroup {
        key: group.key.clone(),
        source_group_id: group
            .key
            .rsplit('.')
            .next()
            .and_then(|value| value.parse::<u32>().ok())
            .ok_or_else(|| invalid("invalid Swarm source encounter-group ID"))?,
        role,
        areas: group.area_keys.clone(),
        members: members.into_boxed_slice(),
    })
}

fn compile_wave(
    wave: &EncounterWaveInput,
    slots: &[EnemySlotInput],
    used_slots: &mut BTreeSet<u32>,
    enemies: &mut BTreeSet<Box<str>>,
    boss_choices: &mut BTreeSet<Box<str>>,
) -> Result<EncounterWave, UniverseCatalogLoadError> {
    let runtime_slots = wave
        .slot_keys
        .iter()
        .enumerate()
        .map(|(slot_index, slot_key)| {
            let slot = slots
                .iter()
                .find(|slot| slot.key.as_ref() == slot_key.as_ref())
                .ok_or_else(|| reference("unknown Swarm enemy slot"))?;
            if slot.wave_id != wave.id
                || usize::from(slot.formation_index) != slot_index + 1
                || !used_slots.insert(slot.id)
            {
                return Err(invalid("invalid Swarm enemy-slot closure"));
            }
            enemies.insert(slot.enemy_variant_key.clone());
            boss_choices.extend(slot.boss_choice_keys.iter().cloned());
            Ok(EncounterEnemySlot {
                key: slot.key.clone(),
                formation_index: slot.formation_index,
                enemy_variant: slot.enemy_variant_key.clone(),
                boss_choices: slot.boss_choice_keys.clone(),
            })
        })
        .collect::<Result<Vec<_>, UniverseCatalogLoadError>>()?;
    Ok(EncounterWave {
        key: wave.key.clone(),
        ordinal: wave.ordinal,
        stage_type: wave.stage_type.clone(),
        authored_level: wave.authored_level,
        hard_level_group: wave.hard_level_group,
        slots: runtime_slots.into_boxed_slice(),
    })
}

fn compile_boss_pool(
    pool: &BossPoolInput,
    groups: &[RuntimeGroup],
) -> Result<RuntimeBossPool, UniverseCatalogLoadError> {
    let difficulty = pool
        .difficulty_key
        .strip_prefix("Difficulty_")
        .and_then(|value| value.parse::<u8>().ok())
        .filter(|value| (1..=5).contains(value))
        .ok_or_else(|| invalid("invalid Swarm boss-pool difficulty"))?;
    let role = parse_role(&pool.tier)?;
    if pool.area_id != u32::from(difficulty)
        || !matches!(
            role,
            EncounterRole::FirstPlaneBoss
                | EncounterRole::SecondPlaneBoss
                | EncounterRole::FinalBoss
        )
        || pool.candidate_keys.is_empty()
    {
        return Err(invalid("invalid Swarm boss-pool tier"));
    }
    let mut previous = None;
    for key in &pool.candidate_keys {
        let group = groups
            .iter()
            .find(|group| group.key.as_ref() == key.as_ref())
            .ok_or_else(|| reference("unknown Swarm boss-pool group"))?;
        if group.role != role || previous.is_some_and(|id| id >= group.source_group_id) {
            return Err(invalid("invalid Swarm boss-pool candidate order"));
        }
        previous = Some(group.source_group_id);
    }
    Ok(RuntimeBossPool {
        _id: pool.id,
        _key: pool.key.clone(),
        difficulty,
        area_id: pool.area_id,
        role,
        candidates: pool.candidate_keys.clone(),
    })
}

fn valid_boss_pool_matrix(pools: &[RuntimeBossPool]) -> bool {
    pools.len() == EXPECTED_BOSS_POOLS
        && (1..=5).all(|difficulty| {
            pools
                .iter()
                .filter(|pool| pool.difficulty == difficulty)
                .count()
                == 3
                && [
                    EncounterRole::FirstPlaneBoss,
                    EncounterRole::SecondPlaneBoss,
                    EncounterRole::FinalBoss,
                ]
                .into_iter()
                .all(|role| {
                    pools
                        .iter()
                        .filter(|pool| pool.difficulty == difficulty && pool.role == role)
                        .count()
                        == 1
                })
        })
}

fn role_for_domain(domain: &str, plane: u8) -> Result<EncounterRole, UniverseCatalogLoadError> {
    match (domain, plane) {
        (NORMAL_DOMAIN | SWARM_DOMAIN, 1..=3) => Ok(EncounterRole::Combat),
        (ELITE_DOMAIN, 1..=3) => Ok(EncounterRole::Elite),
        (BOSS_DOMAIN, 1) => Ok(EncounterRole::FirstPlaneBoss),
        (BOSS_DOMAIN, 2) => Ok(EncounterRole::SecondPlaneBoss),
        (SWARM_BOSS_DOMAIN, 3) => Ok(EncounterRole::FinalBoss),
        (NORMAL_DOMAIN | SWARM_DOMAIN | ELITE_DOMAIN | BOSS_DOMAIN | SWARM_BOSS_DOMAIN, _) => {
            Err(invalid("combat domain is invalid for Swarm plane"))
        }
        _ => Err(reference("non-combat Swarm encounter domain")),
    }
}

fn parse_role(value: &str) -> Result<EncounterRole, UniverseCatalogLoadError> {
    match value {
        "CombatPool" => Ok(EncounterRole::Combat),
        "ElitePool" => Ok(EncounterRole::Elite),
        "FirstPlaneBossAlternative" => Ok(EncounterRole::FirstPlaneBoss),
        "SecondPlaneBossAlternative" => Ok(EncounterRole::SecondPlaneBoss),
        "FinalBoss" => Ok(EncounterRole::FinalBoss),
        _ => Err(invalid("unknown Swarm encounter role")),
    }
}

const fn role_order(role: EncounterRole) -> u8 {
    match role {
        EncounterRole::Combat => 0,
        EncounterRole::Elite => 1,
        EncounterRole::FirstPlaneBoss => 2,
        EncounterRole::SecondPlaneBoss => 3,
        EncounterRole::FinalBoss => 4,
    }
}

fn role_counts(groups: &[RuntimeGroup]) -> [usize; 5] {
    let mut counts = [0; 5];
    for group in groups {
        counts[usize::from(role_order(group.role))] += 1;
    }
    counts
}

fn select_uniform(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    count: usize,
) -> Result<usize, UniverseCatalogLoadError> {
    if count == 1 {
        return Ok(0);
    }
    let count = u32::try_from(count).map_err(|_| invalid("Swarm encounter candidate overflow"))?;
    rng.choose_index(ActivityRngLabel::Encounter, purpose, count)
        .map_err(|_| invalid("Swarm encounter RNG failure"))?
        .map(|draw| draw.value() as usize)
        .ok_or_else(|| reference("no Swarm encounter candidates"))
}

fn select_weighted(
    rng: &mut ActivityRngStreams,
    members: &[RuntimeMember],
) -> Result<usize, UniverseCatalogLoadError> {
    if members.len() == 1 {
        return Ok(0);
    }
    let weights = members
        .iter()
        .map(|member| member.weight)
        .collect::<Vec<_>>();
    rng.choose_weighted(ActivityRngLabel::Encounter, MEMBER_PURPOSE, &weights)
        .map_err(|_| invalid("Swarm encounter RNG failure"))?
        .map(|(index, _)| index as usize)
        .ok_or_else(|| reference("no Swarm encounter members"))
}

fn invalid(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidDefinition, message)
}

fn reference(message: &'static str) -> UniverseCatalogLoadError {
    UniverseCatalogLoadError::new(UniverseCatalogLoadErrorKind::InvalidReference, message)
}
