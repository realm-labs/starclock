//! Generated-row-free immutable Standard Universe definitions.

use crate::digest::{UniverseDefinitionsDigest, UniversePathDefinitionsDigest};
use crate::id::{
    ActivityBindingId, DifficultyId, DomainId, RoomId, TopologyId, TopologyNodeId,
    UniverseProfileId, WorldId,
};
use crate::path::{
    BlessingDefinition, BlessingLevelDefinition, PathDefinition, ResonanceDefinition,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalizedText {
    name_en: Box<str>,
    name_zh_cn: Box<str>,
    summary_en: Box<str>,
    summary_zh_cn: Box<str>,
}

impl LocalizedText {
    pub(crate) fn new(
        name_en: &str,
        name_zh_cn: &str,
        summary_en: &str,
        summary_zh_cn: &str,
    ) -> Self {
        Self {
            name_en: name_en.into(),
            name_zh_cn: name_zh_cn.into(),
            summary_en: summary_en.into(),
            summary_zh_cn: summary_zh_cn.into(),
        }
    }

    #[must_use]
    pub fn name_en(&self) -> &str {
        &self.name_en
    }
    #[must_use]
    pub fn name_zh_cn(&self) -> &str {
        &self.name_zh_cn
    }
    #[must_use]
    pub fn summary_en(&self) -> &str {
        &self.summary_en
    }
    #[must_use]
    pub fn summary_zh_cn(&self) -> &str {
        &self.summary_zh_cn
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseProfileDefinition {
    id: UniverseProfileId,
    stable_key: Box<str>,
    game_version: Box<str>,
    snapshot_date: Box<str>,
    content_manifest_digest: [u8; 32],
    normalized_pack_digest: [u8; 32],
}

impl UniverseProfileDefinition {
    pub(crate) fn new(
        id: UniverseProfileId,
        stable_key: &str,
        game_version: &str,
        snapshot_date: &str,
        content_manifest_digest: [u8; 32],
        normalized_pack_digest: [u8; 32],
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            game_version: game_version.into(),
            snapshot_date: snapshot_date.into(),
            content_manifest_digest,
            normalized_pack_digest,
        }
    }

    #[must_use]
    pub const fn id(&self) -> UniverseProfileId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub fn game_version(&self) -> &str {
        &self.game_version
    }
    #[must_use]
    pub fn snapshot_date(&self) -> &str {
        &self.snapshot_date
    }
    #[must_use]
    pub const fn content_manifest_digest(&self) -> [u8; 32] {
        self.content_manifest_digest
    }
    #[must_use]
    pub const fn normalized_pack_digest(&self) -> [u8; 32] {
        self.normalized_pack_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldDefinition {
    id: WorldId,
    profile: UniverseProfileId,
    stable_key: Box<str>,
    number: u8,
    text: LocalizedText,
    entry_rule_key: Box<str>,
    terminal_rule_key: Box<str>,
    difficulties: Box<[DifficultyId]>,
}

impl WorldDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: WorldId,
        profile: UniverseProfileId,
        stable_key: &str,
        number: u8,
        text: LocalizedText,
        entry_rule_key: &str,
        terminal_rule_key: &str,
        difficulties: Box<[DifficultyId]>,
    ) -> Self {
        Self {
            id,
            profile,
            stable_key: stable_key.into(),
            number,
            text,
            entry_rule_key: entry_rule_key.into(),
            terminal_rule_key: terminal_rule_key.into(),
            difficulties,
        }
    }

    #[must_use]
    pub const fn id(&self) -> WorldId {
        self.id
    }
    #[must_use]
    pub const fn profile(&self) -> UniverseProfileId {
        self.profile
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn number(&self) -> u8 {
        self.number
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
    #[must_use]
    pub fn entry_rule_key(&self) -> &str {
        &self.entry_rule_key
    }
    #[must_use]
    pub fn terminal_rule_key(&self) -> &str {
        &self.terminal_rule_key
    }
    #[must_use]
    pub fn difficulties(&self) -> &[DifficultyId] {
        &self.difficulties
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DifficultyKind {
    Tutorial = 0,
    Standard = 1,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum RecommendedElement {
    Physical = 0,
    Fire = 1,
    Ice = 2,
    Lightning = 3,
    Wind = 4,
    Quantum = 5,
    Imaginary = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScoreThreshold {
    tier: u8,
    score: u32,
}

impl ScoreThreshold {
    pub(crate) const fn new(tier: u8, score: u32) -> Self {
        Self { tier, score }
    }
    #[must_use]
    pub const fn tier(self) -> u8 {
        self.tier
    }
    #[must_use]
    pub const fn score(self) -> u32 {
        self.score
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DifficultyDefinition {
    id: DifficultyId,
    stable_key: Box<str>,
    world: WorldId,
    source_area_id: Box<str>,
    ordinal: u8,
    kind: DifficultyKind,
    recommended_level: u8,
    recommended_elements: Box<[RecommendedElement]>,
    score_curve: Box<[ScoreThreshold]>,
    unlock_source_id: Option<Box<str>>,
}

impl DifficultyDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: DifficultyId,
        stable_key: &str,
        world: WorldId,
        source_area_id: &str,
        ordinal: u8,
        kind: DifficultyKind,
        recommended_level: u8,
        recommended_elements: Box<[RecommendedElement]>,
        score_curve: Box<[ScoreThreshold]>,
        unlock_source_id: Option<Box<str>>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            world,
            source_area_id: source_area_id.into(),
            ordinal,
            kind,
            recommended_level,
            recommended_elements,
            score_curve,
            unlock_source_id,
        }
    }

    #[must_use]
    pub const fn id(&self) -> DifficultyId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn world(&self) -> WorldId {
        self.world
    }
    #[must_use]
    pub fn source_area_id(&self) -> &str {
        &self.source_area_id
    }
    #[must_use]
    pub const fn ordinal(&self) -> u8 {
        self.ordinal
    }
    #[must_use]
    pub const fn kind(&self) -> DifficultyKind {
        self.kind
    }
    #[must_use]
    pub const fn recommended_level(&self) -> u8 {
        self.recommended_level
    }
    #[must_use]
    pub fn recommended_elements(&self) -> &[RecommendedElement] {
        &self.recommended_elements
    }
    #[must_use]
    pub fn score_curve(&self) -> &[ScoreThreshold] {
        &self.score_curve
    }
    #[must_use]
    pub fn unlock_source_id(&self) -> Option<&str> {
        self.unlock_source_id.as_deref()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DomainKind {
    CombatPrimary = 0,
    CombatSecondary = 1,
    Occurrence = 2,
    Encounter = 3,
    Respite = 4,
    Elite = 5,
    Boss = 6,
    Transaction = 7,
    Adventure = 8,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DomainDecisionPolicy {
    BattleHandoff = 0,
    ExternalCommand = 1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainDefinition {
    id: DomainId,
    stable_key: Box<str>,
    source_type: u32,
    kind: DomainKind,
    decision_policy: DomainDecisionPolicy,
    terminal: bool,
    text: LocalizedText,
}

impl DomainDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: DomainId,
        stable_key: &str,
        source_type: u32,
        kind: DomainKind,
        decision_policy: DomainDecisionPolicy,
        terminal: bool,
        text: LocalizedText,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            source_type,
            kind,
            decision_policy,
            terminal,
            text,
        }
    }
    #[must_use]
    pub const fn id(&self) -> DomainId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn source_type(&self) -> u32 {
        self.source_type
    }
    #[must_use]
    pub const fn kind(&self) -> DomainKind {
        self.kind
    }
    #[must_use]
    pub const fn decision_policy(&self) -> DomainDecisionPolicy {
        self.decision_policy
    }
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.terminal
    }
    #[must_use]
    pub const fn text(&self) -> &LocalizedText {
        &self.text
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyNodeDefinition {
    id: TopologyNodeId,
    stable_key: Box<str>,
    source_node_id: u32,
    outgoing: Box<[TopologyNodeId]>,
}

impl TopologyNodeDefinition {
    pub(crate) fn new(
        id: TopologyNodeId,
        stable_key: &str,
        source_node_id: u32,
        outgoing: Box<[TopologyNodeId]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            source_node_id,
            outgoing,
        }
    }
    #[must_use]
    pub const fn id(&self) -> TopologyNodeId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn source_node_id(&self) -> u32 {
        self.source_node_id
    }
    #[must_use]
    pub fn outgoing(&self) -> &[TopologyNodeId] {
        &self.outgoing
    }
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.outgoing.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyDefinition {
    id: TopologyId,
    source_map_id: u32,
    start: TopologyNodeId,
    terminals: Box<[TopologyNodeId]>,
    nodes: Box<[TopologyNodeDefinition]>,
}

impl TopologyDefinition {
    pub(crate) fn new(
        id: TopologyId,
        source_map_id: u32,
        start: TopologyNodeId,
        terminals: Box<[TopologyNodeId]>,
        nodes: Box<[TopologyNodeDefinition]>,
    ) -> Self {
        Self {
            id,
            source_map_id,
            start,
            terminals,
            nodes,
        }
    }
    #[must_use]
    pub const fn id(&self) -> TopologyId {
        self.id
    }
    #[must_use]
    pub const fn source_map_id(&self) -> u32 {
        self.source_map_id
    }
    #[must_use]
    pub const fn start(&self) -> TopologyNodeId {
        self.start
    }
    #[must_use]
    pub fn terminals(&self) -> &[TopologyNodeId] {
        &self.terminals
    }
    #[must_use]
    pub fn nodes(&self) -> &[TopologyNodeDefinition] {
        &self.nodes
    }
    #[must_use]
    pub fn node(&self, id: TopologyNodeId) -> Option<&TopologyNodeDefinition> {
        self.nodes
            .binary_search_by_key(&id, TopologyNodeDefinition::id)
            .ok()
            .map(|index| &self.nodes[index])
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoomDefinition {
    id: RoomId,
    stable_key: Box<str>,
    domain: DomainId,
    source_room_id: Box<str>,
    map_entrance: Box<str>,
    source_group_id: Box<str>,
    section_ids: Box<[u32]>,
}

impl RoomDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: RoomId,
        stable_key: &str,
        domain: DomainId,
        source_room_id: &str,
        map_entrance: &str,
        source_group_id: &str,
        section_ids: Box<[u32]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            domain,
            source_room_id: source_room_id.into(),
            map_entrance: map_entrance.into(),
            source_group_id: source_group_id.into(),
            section_ids,
        }
    }
    #[must_use]
    pub const fn id(&self) -> RoomId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn domain(&self) -> DomainId {
        self.domain
    }
    #[must_use]
    pub fn source_room_id(&self) -> &str {
        &self.source_room_id
    }
    #[must_use]
    pub fn map_entrance(&self) -> &str {
        &self.map_entrance
    }
    #[must_use]
    pub fn source_group_id(&self) -> &str {
        &self.source_group_id
    }
    #[must_use]
    pub fn section_ids(&self) -> &[u32] {
        &self.section_ids
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum ActivityDomainDecision {
    BattleCommand = 0,
    RunCommand = 1,
    ExternalOutcome = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActivityDomainBinding {
    domain: DomainId,
    decision: ActivityDomainDecision,
}

impl ActivityDomainBinding {
    pub(crate) const fn new(domain: DomainId, decision: ActivityDomainDecision) -> Self {
        Self { domain, decision }
    }
    #[must_use]
    pub const fn domain(self) -> DomainId {
        self.domain
    }
    #[must_use]
    pub const fn decision(self) -> ActivityDomainDecision {
        self.decision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UniverseActivityBindingDefinition {
    id: ActivityBindingId,
    stable_key: Box<str>,
    profile: UniverseProfileId,
    activity_key: Box<str>,
    battle_handoff_contract: Box<str>,
    external_outcome_contract: Box<str>,
    domains: Box<[ActivityDomainBinding]>,
}

impl UniverseActivityBindingDefinition {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        id: ActivityBindingId,
        stable_key: &str,
        profile: UniverseProfileId,
        activity_key: &str,
        battle_handoff_contract: &str,
        external_outcome_contract: &str,
        domains: Box<[ActivityDomainBinding]>,
    ) -> Self {
        Self {
            id,
            stable_key: stable_key.into(),
            profile,
            activity_key: activity_key.into(),
            battle_handoff_contract: battle_handoff_contract.into(),
            external_outcome_contract: external_outcome_contract.into(),
            domains,
        }
    }
    #[must_use]
    pub const fn id(&self) -> ActivityBindingId {
        self.id
    }
    #[must_use]
    pub fn stable_key(&self) -> &str {
        &self.stable_key
    }
    #[must_use]
    pub const fn profile(&self) -> UniverseProfileId {
        self.profile
    }
    #[must_use]
    pub fn activity_key(&self) -> &str {
        &self.activity_key
    }
    #[must_use]
    pub fn battle_handoff_contract(&self) -> &str {
        &self.battle_handoff_contract
    }
    #[must_use]
    pub fn external_outcome_contract(&self) -> &str {
        &self.external_outcome_contract
    }
    #[must_use]
    pub fn domains(&self) -> &[ActivityDomainBinding] {
        &self.domains
    }
}

#[derive(Debug)]
pub(crate) struct UniverseDefinitions {
    pub(crate) digest: UniverseDefinitionsDigest,
    pub(crate) path_digest: UniversePathDefinitionsDigest,
    pub(crate) profile: UniverseProfileDefinition,
    pub(crate) worlds: Box<[WorldDefinition]>,
    pub(crate) difficulties: Box<[DifficultyDefinition]>,
    pub(crate) domains: Box<[DomainDefinition]>,
    pub(crate) topologies: Box<[TopologyDefinition]>,
    pub(crate) rooms: Box<[RoomDefinition]>,
    pub(crate) activity: UniverseActivityBindingDefinition,
    pub(crate) paths: Box<[PathDefinition]>,
    pub(crate) blessings: Box<[BlessingDefinition]>,
    pub(crate) blessing_levels: Box<[BlessingLevelDefinition]>,
    pub(crate) resonances: Box<[ResonanceDefinition]>,
}
