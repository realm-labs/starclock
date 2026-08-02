//! Deterministic Gold and Gears encounter-family and difficulty selection.

use std::{collections::BTreeSet, sync::Arc};

use starclock_activity::{ActivityRngLabel, ActivityRngStreams, ActivityTransactionState, NodeId};

use crate::{
    gold_gears_content::{
        GoldAndGearsContentCatalog,
        types::{EncounterRole, StableKey},
    },
    gold_gears_structural::{
        AreaDefinition, DifficultySegmentDefinition, GoldAndGearsStructuralCatalog,
    },
};

use super::{GoldAndGearsEntryError, GoldAndGearsRuntimeInstance, topology::CompiledTopology};

/// This policy is deterministic project behavior, not observed game parity.
pub const GOLD_AND_GEARS_ENCOUNTER_POLICY_ACCURACY: &str =
    "DeterministicProjectPolicyNotObservedParity";

/// Released engine code or a pinned table must expose both hidden joins.
pub const GOLD_AND_GEARS_ENCOUNTER_POLICY_REPLACEMENT_CONDITION: &str = "released engine code or pinned tables expose the exact room/domain/group join and effective battle-level selection sequence";

const EXPECTED_GROUPS: usize = 181;
const EXPECTED_WAVES: usize = 478;
const EXPECTED_SLOTS: usize = 1_513;
const EXPECTED_DISTINCT_ENEMIES: usize = 90;
const GROUP_PURPOSE: u16 = 0x6101;
const MEMBER_PURPOSE: u16 = 0x6102;

const NORMAL_DOMAIN: &str = "gold-gears.domain.monsternormal";
const ELITE_DOMAIN: &str = "gold-gears.domain.monsterelite";
const BOSS_DOMAIN: &str = "gold-gears.domain.monsterboss";
const NOUS_BOSS_DOMAIN: &str = "gold-gears.domain.monsternousboss";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsEncounterRole {
    Combat,
    Elite,
    FirstPlaneBoss,
    SecondPlaneBoss,
    FinalBoss,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEncounterEnemySlot {
    key: Box<str>,
    source_slot: Box<str>,
    source_monster_id: Box<str>,
    enemy: Box<str>,
    boss_choices: Box<[Box<str>]>,
}

impl GoldAndGearsEncounterEnemySlot {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn source_slot(&self) -> &str {
        &self.source_slot
    }
    pub fn source_monster_id(&self) -> &str {
        &self.source_monster_id
    }
    pub fn enemy(&self) -> &str {
        &self.enemy
    }
    pub fn boss_choices(&self) -> impl ExactSizeIterator<Item = &str> {
        self.boss_choices.iter().map(Box::as_ref)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEncounterWave {
    key: Box<str>,
    source_stage_id: Box<str>,
    wave_index: u16,
    stage_type: Box<str>,
    authored_stage_level: u16,
    hard_level_group: u16,
    stage_ability_ids: Box<[Box<str>]>,
    slots: Box<[GoldAndGearsEncounterEnemySlot]>,
}

impl GoldAndGearsEncounterWave {
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn source_stage_id(&self) -> &str {
        &self.source_stage_id
    }
    pub const fn wave_index(&self) -> u16 {
        self.wave_index
    }
    pub fn stage_type(&self) -> &str {
        &self.stage_type
    }
    pub const fn authored_stage_level(&self) -> u16 {
        self.authored_stage_level
    }
    pub const fn hard_level_group(&self) -> u16 {
        self.hard_level_group
    }
    pub fn stage_ability_ids(&self) -> impl ExactSizeIterator<Item = &str> {
        self.stage_ability_ids.iter().map(Box::as_ref)
    }
    pub fn slots(&self) -> &[GoldAndGearsEncounterEnemySlot] {
        &self.slots
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsEncounterSelection {
    group: Box<str>,
    source_group_id: u32,
    source_rogue_monster_id: Box<str>,
    source_primary_monster_id: Box<str>,
    source_stage_id: Box<str>,
    role: GoldAndGearsEncounterRole,
    difficulty_segment: Box<str>,
    effective_level: u16,
    waves: Box<[GoldAndGearsEncounterWave]>,
}

impl GoldAndGearsEncounterSelection {
    pub fn group(&self) -> &str {
        &self.group
    }
    pub const fn source_group_id(&self) -> u32 {
        self.source_group_id
    }
    pub fn source_rogue_monster_id(&self) -> &str {
        &self.source_rogue_monster_id
    }
    pub fn source_primary_monster_id(&self) -> &str {
        &self.source_primary_monster_id
    }
    pub fn source_stage_id(&self) -> &str {
        &self.source_stage_id
    }
    pub const fn role(&self) -> GoldAndGearsEncounterRole {
        self.role
    }
    pub fn difficulty_segment(&self) -> &str {
        &self.difficulty_segment
    }
    pub const fn effective_level(&self) -> u16 {
        self.effective_level
    }
    pub fn waves(&self) -> &[GoldAndGearsEncounterWave] {
        &self.waves
    }
}

#[derive(Debug)]
pub(super) struct EncounterRuntimeCatalog {
    groups: Box<[RuntimeGroup]>,
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
    waves: Box<[GoldAndGearsEncounterWave]>,
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
    area: Box<str>,
    bands: Box<[DifficultyBand]>,
    nodes: Box<[NodeBinding]>,
}

impl EncounterRuntimeCatalog {
    pub(super) fn compile(
        content: &GoldAndGearsContentCatalog,
    ) -> Result<Self, GoldAndGearsEntryError> {
        if content.encounter_groups.len() != EXPECTED_GROUPS
            || content.encounter_waves.len() != EXPECTED_WAVES
            || content.enemy_slots.len() != EXPECTED_SLOTS
        {
            return Err(GoldAndGearsEntryError::InvalidEncounterRuntime);
        }
        let mut used_waves = BTreeSet::new();
        let mut used_slots = BTreeSet::new();
        let mut enemies = BTreeSet::new();
        let mut groups = Vec::with_capacity(content.encounter_groups.len());
        for group in &content.encounter_groups {
            let source_group_id = group
                .source_group_id
                .parse::<u32>()
                .map_err(|_| GoldAndGearsEntryError::InvalidEncounterRuntime)?;
            if group.members.is_empty()
                || !matches!(
                    group.source_namespace.as_ref(),
                    "GoldAndGears82Series" | "SharedDlcGuide"
                )
            {
                return Err(GoldAndGearsEntryError::InvalidEncounterRuntime);
            }
            let mut members = Vec::with_capacity(group.members.len());
            for member in &group.members {
                let mut waves = Vec::with_capacity(member.waves.len());
                for (wave_offset, wave_key) in member.waves.iter().enumerate() {
                    let wave = content
                        .encounter_waves
                        .iter()
                        .find(|wave| wave.key.as_str() == wave_key.as_str())
                        .ok_or(GoldAndGearsEntryError::InvalidEncounterRuntime)?;
                    if wave.group_id != group.id
                        || wave.source_rogue_monster_id.as_ref()
                            != member.source_rogue_monster_id.as_ref()
                        || wave.source_stage_id.as_ref() != member.source_stage_id.as_ref()
                        || usize::from(wave.wave_index) != wave_offset + 1
                        || wave.stage_type.as_ref() != "VerseSimulation"
                        || !used_waves.insert(wave.id)
                    {
                        return Err(GoldAndGearsEntryError::InvalidEncounterRuntime);
                    }
                    let mut slots = Vec::with_capacity(wave.slots.len());
                    for (slot_offset, slot_key) in wave.slots.iter().enumerate() {
                        let slot = content
                            .enemy_slots
                            .iter()
                            .find(|slot| slot.key.as_str() == slot_key.as_str())
                            .ok_or(GoldAndGearsEntryError::InvalidEncounterRuntime)?;
                        if slot.wave_id != wave.id
                            || usize::from(slot.slot_index) != slot_offset + 1
                            || !used_slots.insert(slot.id)
                        {
                            return Err(GoldAndGearsEntryError::InvalidEncounterRuntime);
                        }
                        enemies.insert(slot.enemy.as_str());
                        slots.push(GoldAndGearsEncounterEnemySlot {
                            key: slot.key.as_str().into(),
                            source_slot: slot.source_slot.clone(),
                            source_monster_id: slot.source_monster_id.clone(),
                            enemy: slot.enemy.as_str().into(),
                            boss_choices: copy_keys(&slot.boss_choices),
                        });
                    }
                    waves.push(GoldAndGearsEncounterWave {
                        key: wave.key.as_str().into(),
                        source_stage_id: wave.source_stage_id.clone(),
                        wave_index: wave.wave_index,
                        stage_type: wave.stage_type.clone(),
                        authored_stage_level: wave.authored_stage_level,
                        hard_level_group: wave.hard_level_group,
                        stage_ability_ids: wave.stage_ability_ids.clone(),
                        slots: slots.into_boxed_slice(),
                    });
                }
                if waves.is_empty() {
                    return Err(GoldAndGearsEntryError::InvalidEncounterRuntime);
                }
                members.push(RuntimeMember {
                    source_rogue_monster_id: member.source_rogue_monster_id.clone(),
                    source_primary_monster_id: member.source_primary_monster_id.clone(),
                    source_stage_id: member.source_stage_id.clone(),
                    weight: member.weight,
                    waves: waves.into_boxed_slice(),
                });
            }
            groups.push(RuntimeGroup {
                key: group.key.as_str().into(),
                source_group_id,
                role: group.role,
                areas: copy_keys(&group.areas),
                members: members.into_boxed_slice(),
            });
        }
        groups.sort_by_key(|group| group.source_group_id);
        if used_waves.len() != EXPECTED_WAVES
            || used_slots.len() != EXPECTED_SLOTS
            || enemies.len() != EXPECTED_DISTINCT_ENEMIES
            || groups
                .windows(2)
                .any(|pair| pair[0].source_group_id == pair[1].source_group_id)
            || role_counts(&groups) != [2, 123, 6, 35, 12, 3]
        {
            return Err(GoldAndGearsEntryError::InvalidEncounterRuntime);
        }
        Ok(Self {
            groups: groups.into_boxed_slice(),
        })
    }
}

impl CompiledEncounterRuntime {
    pub(super) fn compile(
        catalog: Arc<EncounterRuntimeCatalog>,
        structural: &GoldAndGearsStructuralCatalog,
        area: &AreaDefinition,
        topology: &CompiledTopology,
    ) -> Result<Self, GoldAndGearsEntryError> {
        if area.difficulty_segment_sources.len() != topology.planes.len() {
            return Err(GoldAndGearsEntryError::InvalidEncounterDifficulty);
        }
        let bands = area
            .difficulty_segment_sources
            .iter()
            .map(|source| difficulty_band(structural, source))
            .collect::<Result<Vec<_>, _>>()?;
        let mut nodes = Vec::new();
        for (plane_index, plane) in topology.planes.iter().enumerate() {
            let board = structural
                .chessboards
                .iter()
                .find(|board| board.stable_key.as_ref() == plane.chessboard_key.as_ref())
                .ok_or(GoldAndGearsEntryError::InvalidEncounterDifficulty)?;
            for node in structural
                .nodes
                .iter()
                .filter(|node| node.chessboard == board.id)
            {
                let column = structural
                    .columns
                    .iter()
                    .find(|column| column.id == node.column)
                    .ok_or(GoldAndGearsEntryError::InvalidEncounterDifficulty)?;
                nodes.push(NodeBinding {
                    node: NodeId::new(node.id.0)
                        .ok_or(GoldAndGearsEntryError::InvalidEncounterDifficulty)?,
                    plane: u8::try_from(plane_index + 1)
                        .map_err(|_| GoldAndGearsEntryError::InvalidEncounterDifficulty)?,
                    position: column.index,
                });
            }
        }
        nodes.sort_by_key(|binding| binding.node);
        if nodes.windows(2).any(|pair| pair[0].node == pair[1].node) {
            return Err(GoldAndGearsEntryError::InvalidEncounterDifficulty);
        }
        Ok(Self {
            catalog,
            area: area.stable_key.clone(),
            bands: bands.into_boxed_slice(),
            nodes: nodes.into_boxed_slice(),
        })
    }

    pub(super) fn select(
        &self,
        node: NodeId,
        domain: &str,
        boss_choice: Option<&str>,
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsEncounterSelection, GoldAndGearsEntryError> {
        let binding = self
            .nodes
            .binary_search_by_key(&node, |binding| binding.node)
            .ok()
            .map(|index| self.nodes[index])
            .ok_or(GoldAndGearsEntryError::UnknownEncounterNode(node.get()))?;
        let role = role_for_domain(domain, binding.plane)?;
        let band = self
            .bands
            .get(usize::from(binding.plane - 1))
            .ok_or(GoldAndGearsEntryError::InvalidEncounterDifficulty)?;
        let bucket = band.cuts.partition_point(|cut| *cut <= binding.position);
        let effective_level = *band
            .levels
            .get(bucket)
            .ok_or(GoldAndGearsEntryError::InvalidEncounterDifficulty)?;
        let candidates = self
            .catalog
            .groups
            .iter()
            .filter(|group| group.role == content_role(role))
            .filter(|group| {
                group
                    .areas
                    .iter()
                    .any(|area| area.as_ref() == self.area.as_ref())
            })
            .filter(|group| {
                role != GoldAndGearsEncounterRole::FinalBoss
                    || boss_choice.is_some_and(|choice| group_has_boss_choice(group, choice))
            })
            .collect::<Vec<_>>();
        if role == GoldAndGearsEncounterRole::FinalBoss && boss_choice.is_none() {
            return Err(GoldAndGearsEntryError::MissingEncounterBossChoice);
        }
        if candidates.is_empty() {
            return Err(GoldAndGearsEntryError::NoEncounterCandidates);
        }
        rng.transact(|rng| {
            let group_index = select_uniform(rng, GROUP_PURPOSE, candidates.len())?;
            let group = candidates[group_index];
            let member_index = select_weighted(rng, &group.members)?;
            let member = &group.members[member_index];
            Ok(GoldAndGearsEncounterSelection {
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

    #[cfg(test)]
    pub(super) fn denominators(&self) -> (usize, usize, usize) {
        (EXPECTED_GROUPS, EXPECTED_WAVES, EXPECTED_SLOTS)
    }

    #[cfg(any(test, feature = "benchmark-harness"))]
    pub(super) fn node_at(&self, plane: u8, position: u16) -> Option<NodeId> {
        self.nodes
            .iter()
            .find(|binding| binding.plane == plane && binding.position == position)
            .map(|binding| binding.node)
    }
}

impl GoldAndGearsRuntimeInstance {
    pub(super) fn encounter_role_for_node(
        &self,
        state: &ActivityTransactionState,
        node: NodeId,
    ) -> Option<GoldAndGearsEncounterRole> {
        let plane = self
            .graph
            .node(node)
            .and_then(|definition| u8::try_from(definition.section().get()).ok())?;
        let domain = self.map.node_domain_key(state, node)?;
        role_for_domain(domain, plane).ok()
    }

    /// Resolves the current combat-capable domain into one immutable encounter.
    ///
    /// Group and weighted-member draws are transactional and use only the
    /// Activity Encounter stream. A final-plane boss requires the explicit
    /// boss choice already committed for plane three.
    pub fn select_current_encounter(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<GoldAndGearsEncounterSelection, GoldAndGearsEntryError> {
        let node = state.current_node();
        let plane = self
            .graph
            .node(node)
            .and_then(|definition| u8::try_from(definition.section().get()).ok())
            .ok_or(GoldAndGearsEntryError::UnknownEncounterNode(node.get()))?;
        let domain = self.map.node_domain_key(state, node).ok_or(
            GoldAndGearsEntryError::UnresolvedEncounterDomain(node.get()),
        )?;
        let boss = (plane == 3)
            .then(|| self.transitions.selected_boss(state, plane))
            .flatten();
        self.encounter_runtime.select(node, domain, boss, rng)
    }
}

fn difficulty_band(
    structural: &GoldAndGearsStructuralCatalog,
    source: &str,
) -> Result<DifficultyBand, GoldAndGearsEntryError> {
    let segment: &DifficultySegmentDefinition = structural
        .difficulty_segments
        .iter()
        .find(|segment| segment.source_id.as_ref() == source)
        .ok_or(GoldAndGearsEntryError::InvalidEncounterDifficulty)?;
    if segment.levels.len() != segment.cut_positions.len() + 1
        || segment
            .cut_positions
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(GoldAndGearsEntryError::InvalidEncounterDifficulty);
    }
    Ok(DifficultyBand {
        key: segment.stable_key.clone(),
        cuts: segment.cut_positions.clone(),
        levels: segment.levels.clone(),
    })
}

fn role_for_domain(
    domain: &str,
    plane: u8,
) -> Result<GoldAndGearsEncounterRole, GoldAndGearsEntryError> {
    match domain {
        NORMAL_DOMAIN => Ok(GoldAndGearsEncounterRole::Combat),
        ELITE_DOMAIN => Ok(GoldAndGearsEncounterRole::Elite),
        BOSS_DOMAIN | NOUS_BOSS_DOMAIN => match plane {
            1 => Ok(GoldAndGearsEncounterRole::FirstPlaneBoss),
            2 => Ok(GoldAndGearsEncounterRole::SecondPlaneBoss),
            3 => Ok(GoldAndGearsEncounterRole::FinalBoss),
            _ => Err(GoldAndGearsEntryError::InvalidEncounterDifficulty),
        },
        _ => Err(GoldAndGearsEntryError::NonCombatEncounterDomain(
            domain.into(),
        )),
    }
}

fn content_role(role: GoldAndGearsEncounterRole) -> EncounterRole {
    match role {
        GoldAndGearsEncounterRole::Combat => EncounterRole::CombatPool,
        GoldAndGearsEncounterRole::Elite => EncounterRole::ElitePool,
        GoldAndGearsEncounterRole::FirstPlaneBoss => EncounterRole::FirstPlaneBossAlternative,
        GoldAndGearsEncounterRole::SecondPlaneBoss => EncounterRole::SecondPlaneBossAlternative,
        GoldAndGearsEncounterRole::FinalBoss => EncounterRole::FinalBoss,
    }
}

fn group_has_boss_choice(group: &RuntimeGroup, choice: &str) -> bool {
    group
        .members
        .iter()
        .flat_map(|member| member.waves.iter())
        .flat_map(|wave| wave.slots.iter())
        .flat_map(|slot| slot.boss_choices.iter())
        .any(|candidate| candidate.as_ref() == choice)
}

fn select_uniform(
    rng: &mut ActivityRngStreams,
    purpose: u16,
    count: usize,
) -> Result<usize, GoldAndGearsEntryError> {
    if count == 1 {
        return Ok(0);
    }
    let count = u32::try_from(count).map_err(|_| GoldAndGearsEntryError::EncounterRng)?;
    rng.choose_index(ActivityRngLabel::Encounter, purpose, count)
        .map_err(|_| GoldAndGearsEntryError::EncounterRng)?
        .map(|draw| draw.value() as usize)
        .ok_or(GoldAndGearsEntryError::NoEncounterCandidates)
}

fn select_weighted(
    rng: &mut ActivityRngStreams,
    members: &[RuntimeMember],
) -> Result<usize, GoldAndGearsEntryError> {
    if members.len() == 1 {
        return Ok(0);
    }
    let weights = members
        .iter()
        .map(|member| member.weight)
        .collect::<Vec<_>>();
    rng.choose_weighted(ActivityRngLabel::Encounter, MEMBER_PURPOSE, &weights)
        .map_err(|_| GoldAndGearsEntryError::EncounterRng)?
        .map(|(index, _)| index as usize)
        .ok_or(GoldAndGearsEntryError::NoEncounterCandidates)
}

fn copy_keys(values: &[StableKey]) -> Box<[Box<str>]> {
    values
        .iter()
        .map(|value| value.as_str().into())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn role_counts(groups: &[RuntimeGroup]) -> [usize; 6] {
    let mut counts = [0; 6];
    for group in groups {
        let index = match group.role {
            EncounterRole::GuideBoss => 0,
            EncounterRole::CombatPool => 1,
            EncounterRole::ElitePool => 2,
            EncounterRole::FirstPlaneBossAlternative => 3,
            EncounterRole::SecondPlaneBossAlternative => 4,
            EncounterRole::FinalBoss => 5,
        };
        counts[index] += 1;
    }
    counts
}
