//! Immutable spatial-free encounter, room-binding and content-pool definitions.

use crate::definition::{DomainKind, LocalizedText};
use crate::id::{
    ContentPoolId, DifficultyId, EncounterGroupId, EncounterMemberId, EncounterPoolId,
    EncounterWaveId, RoomId,
};
use crate::path::ExactParameter;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum WavePolicy {
    SingleWave = 0,
    AuthoredSequentialWaves = 1,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum BossPhasePolicy {
    EnemyAuthoredLifecycle = 0,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EnemyRole {
    Boss = 0,
    Elite = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterEnemySlot {
    slot_key: Box<str>,
    source_monster_id: Box<str>,
    enemy_variant_key: Box<str>,
}
impl EncounterEnemySlot {
    pub(crate) fn new(slot: &str, source: &str, enemy: &str) -> Self {
        Self {
            slot_key: slot.into(),
            source_monster_id: source.into(),
            enemy_variant_key: enemy.into(),
        }
    }
    #[must_use]
    pub fn slot_key(&self) -> &str {
        &self.slot_key
    }
    #[must_use]
    pub fn source_monster_id(&self) -> &str {
        &self.source_monster_id
    }
    #[must_use]
    pub fn enemy_variant_key(&self) -> &str {
        &self.enemy_variant_key
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterWaveDefinition {
    id: EncounterWaveId,
    enemies: Box<[EncounterEnemySlot]>,
}
impl EncounterWaveDefinition {
    pub(crate) fn new(id: EncounterWaveId, enemies: Box<[EncounterEnemySlot]>) -> Self {
        Self { id, enemies }
    }
    #[must_use]
    pub const fn id(&self) -> EncounterWaveId {
        self.id
    }
    #[must_use]
    pub fn enemies(&self) -> &[EncounterEnemySlot] {
        &self.enemies
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterMemberDefinition {
    id: EncounterMemberId,
    source_rogue_monster_id: Box<str>,
    source_primary_monster_id: Box<str>,
    source_stage_id: Box<str>,
    weight: ExactParameter,
    stage_level: u32,
    hard_level_group: u32,
    stage_ability_ids: Box<[Box<str>]>,
    drop_type: Option<Box<str>>,
    waves: Box<[EncounterWaveDefinition]>,
}
impl EncounterMemberDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: EncounterMemberId,
        rogue: &str,
        primary: &str,
        stage: &str,
        weight: ExactParameter,
        stage_level: u32,
        hard_level_group: u32,
        abilities: Box<[Box<str>]>,
        drop_type: Option<Box<str>>,
        waves: Box<[EncounterWaveDefinition]>,
    ) -> Self {
        Self {
            id,
            source_rogue_monster_id: rogue.into(),
            source_primary_monster_id: primary.into(),
            source_stage_id: stage.into(),
            weight,
            stage_level,
            hard_level_group,
            stage_ability_ids: abilities,
            drop_type,
            waves,
        }
    }
    #[must_use]
    pub const fn id(&self) -> EncounterMemberId {
        self.id
    }
    #[must_use]
    pub fn source_rogue_monster_id(&self) -> &str {
        &self.source_rogue_monster_id
    }
    #[must_use]
    pub fn source_primary_monster_id(&self) -> &str {
        &self.source_primary_monster_id
    }
    #[must_use]
    pub fn source_stage_id(&self) -> &str {
        &self.source_stage_id
    }
    #[must_use]
    pub const fn weight(&self) -> ExactParameter {
        self.weight
    }
    #[must_use]
    pub const fn stage_level(&self) -> u32 {
        self.stage_level
    }
    #[must_use]
    pub const fn hard_level_group(&self) -> u32 {
        self.hard_level_group
    }
    #[must_use]
    pub fn stage_ability_ids(&self) -> &[Box<str>] {
        &self.stage_ability_ids
    }
    #[must_use]
    pub fn drop_type(&self) -> Option<&str> {
        self.drop_type.as_deref()
    }
    #[must_use]
    pub fn waves(&self) -> &[EncounterWaveDefinition] {
        &self.waves
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncounterGroupDefinition {
    id: EncounterGroupId,
    stable_key: Box<str>,
    source_group_id: Box<str>,
    wave_policy: WavePolicy,
    boss_phase_policy: BossPhasePolicy,
    text: LocalizedText,
    members: Box<[EncounterMemberDefinition]>,
}
impl EncounterGroupDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: EncounterGroupId,
        stable_key: &str,
        source_group_id: &str,
        wave_policy: WavePolicy,
        boss_phase_policy: BossPhasePolicy,
        text: LocalizedText,
        members: Box<[EncounterMemberDefinition]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            source_group_id: source_group_id.into(),
            wave_policy,
            boss_phase_policy,
            text,
            members,
        }
    }
    #[must_use]
    pub const fn id(&self) -> EncounterGroupId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub fn source_group_id(&self) -> &str {
        &self.source_group_id
    }
    #[must_use]
    pub const fn wave_policy(&self) -> WavePolicy {
        self.wave_policy
    }
    #[must_use]
    pub const fn boss_phase_policy(&self) -> BossPhasePolicy {
        self.boss_phase_policy
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn members(&self) -> &[EncounterMemberDefinition] {
        &self.members
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DifficultyEnemyBinding {
    difficulty: DifficultyId,
    role: EnemyRole,
    source_monster_id: Box<str>,
    enemy_variant_key: Box<str>,
    level: u32,
}
impl DifficultyEnemyBinding {
    pub(crate) fn new(
        difficulty: DifficultyId,
        role: EnemyRole,
        source: &str,
        enemy: &str,
        level: u32,
    ) -> Self {
        Self {
            difficulty,
            role,
            source_monster_id: source.into(),
            enemy_variant_key: enemy.into(),
            level,
        }
    }
    #[must_use]
    pub const fn difficulty(&self) -> DifficultyId {
        self.difficulty
    }
    #[must_use]
    pub const fn role(&self) -> EnemyRole {
        self.role
    }
    #[must_use]
    pub fn source_monster_id(&self) -> &str {
        &self.source_monster_id
    }
    #[must_use]
    pub fn enemy_variant_key(&self) -> &str {
        &self.enemy_variant_key
    }
    #[must_use]
    pub const fn level(&self) -> u32 {
        self.level
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum EncounterSelectionPolicy {
    ExactConditionThenWeightedStableOrder = 0,
    WorldDifficultyBossEliteBinding = 1,
    ConditionThenGroupOrDifficultyBinding = 2,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FixedEncounterBinding {
    condition_key: Box<str>,
    source_content_id: Box<str>,
}
impl FixedEncounterBinding {
    pub(crate) fn new(condition: &str, source: &str) -> Self {
        Self {
            condition_key: condition.into(),
            source_content_id: source.into(),
        }
    }
    #[must_use]
    pub fn condition_key(&self) -> &str {
        &self.condition_key
    }
    #[must_use]
    pub fn source_content_id(&self) -> &str {
        &self.source_content_id
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedEncounterBinding {
    condition_key: Box<str>,
    group: EncounterGroupId,
    weight: ExactParameter,
}
impl WeightedEncounterBinding {
    pub(crate) fn new(condition: &str, group: EncounterGroupId, weight: ExactParameter) -> Self {
        Self {
            condition_key: condition.into(),
            group,
            weight,
        }
    }
    #[must_use]
    pub fn condition_key(&self) -> &str {
        &self.condition_key
    }
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
pub struct EncounterPoolDefinition {
    id: EncounterPoolId,
    stable_key: Box<str>,
    room: RoomId,
    domain_kind: DomainKind,
    map_entrance: Box<str>,
    selection_policy: EncounterSelectionPolicy,
    source_primary_condition_key: Box<str>,
    text: LocalizedText,
    fixed: Box<[FixedEncounterBinding]>,
    weighted: Box<[WeightedEncounterBinding]>,
}
impl EncounterPoolDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: EncounterPoolId,
        key: &str,
        room: RoomId,
        domain: DomainKind,
        entrance: &str,
        policy: EncounterSelectionPolicy,
        condition: &str,
        text: LocalizedText,
        fixed: Box<[FixedEncounterBinding]>,
        weighted: Box<[WeightedEncounterBinding]>,
    ) -> Self {
        Self {
            id,
            stable_key: key.into(),
            room,
            domain_kind: domain,
            map_entrance: entrance.into(),
            selection_policy: policy,
            source_primary_condition_key: condition.into(),
            text,
            fixed,
            weighted,
        }
    }
    #[must_use]
    pub const fn id(&self) -> EncounterPoolId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn room(&self) -> RoomId {
        self.room
    }
    #[must_use]
    pub const fn domain_kind(&self) -> DomainKind {
        self.domain_kind
    }
    #[must_use]
    pub fn map_entrance(&self) -> &str {
        &self.map_entrance
    }
    #[must_use]
    pub const fn selection_policy(&self) -> EncounterSelectionPolicy {
        self.selection_policy
    }
    #[must_use]
    pub fn source_primary_condition_key(&self) -> &str {
        &self.source_primary_condition_key
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn fixed(&self) -> &[FixedEncounterBinding] {
        &self.fixed
    }
    #[must_use]
    pub fn weighted(&self) -> &[WeightedEncounterBinding] {
        &self.weighted
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RoomContentKind {
    EncounterGroup = 0,
    FixedContent = 1,
    ExternalDecision = 2,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoomContentBinding {
    room: RoomId,
    condition_key: Box<str>,
    source_content_id: Box<str>,
    kind: RoomContentKind,
    encounter_group: Option<EncounterGroupId>,
}
impl RoomContentBinding {
    pub(crate) fn new(
        room: RoomId,
        condition: &str,
        source: &str,
        kind: RoomContentKind,
        group: Option<EncounterGroupId>,
    ) -> Self {
        Self {
            room,
            condition_key: condition.into(),
            source_content_id: source.into(),
            kind,
            encounter_group: group,
        }
    }
    #[must_use]
    pub const fn room(&self) -> RoomId {
        self.room
    }
    #[must_use]
    pub fn condition_key(&self) -> &str {
        &self.condition_key
    }
    #[must_use]
    pub fn source_content_id(&self) -> &str {
        &self.source_content_id
    }
    #[must_use]
    pub const fn kind(&self) -> RoomContentKind {
        self.kind
    }
    #[must_use]
    pub const fn encounter_group(&self) -> Option<EncounterGroupId> {
        self.encounter_group
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ContentPoolKind {
    Blessing = 0,
    Curio = 1,
    Occurrence = 2,
    Encounter = 3,
    Shop = 4,
    TrailblazeBonus = 5,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPoolEntry {
    content_key: Box<str>,
    weight: ExactParameter,
    condition: Option<Box<str>>,
}
impl ContentPoolEntry {
    pub(crate) fn new(key: &str, weight: ExactParameter, condition: Option<Box<str>>) -> Self {
        Self {
            content_key: key.into(),
            weight,
            condition,
        }
    }
    #[must_use]
    pub fn content_key(&self) -> &str {
        &self.content_key
    }
    #[must_use]
    pub const fn weight(&self) -> ExactParameter {
        self.weight
    }
    #[must_use]
    pub fn condition(&self) -> Option<&str> {
        self.condition.as_deref()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentPoolDefinition {
    id: ContentPoolId,
    stable_key: Box<str>,
    kind: ContentPoolKind,
    ordering_key: Box<str>,
    replacement: bool,
    entries: Box<[ContentPoolEntry]>,
}
impl ContentPoolDefinition {
    pub(crate) fn new(
        id: ContentPoolId,
        key: &str,
        kind: ContentPoolKind,
        ordering: &str,
        replacement: bool,
        entries: Box<[ContentPoolEntry]>,
    ) -> Self {
        Self {
            id,
            stable_key: key.into(),
            kind,
            ordering_key: ordering.into(),
            replacement,
            entries,
        }
    }
    #[must_use]
    pub const fn id(&self) -> ContentPoolId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn kind(&self) -> ContentPoolKind {
        self.kind
    }
    #[must_use]
    pub fn ordering_key(&self) -> &str {
        &self.ordering_key
    }
    #[must_use]
    pub const fn replacement(&self) -> bool {
        self.replacement
    }
    #[must_use]
    pub fn entries(&self) -> &[ContentPoolEntry] {
        &self.entries
    }
}
