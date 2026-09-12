//! Immutable room, stage, weekly-display and encounter reachability closure.

use std::{collections::BTreeMap, sync::Arc};

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{ActivityStateHash, GraphActivity};
use starclock_data::{
    divergent_universe_catalog::{
        DivergentUniverseFlowCatalog, DivergentUniverseFlowId, DivergentUniverseRoomCandidateId,
        DivergentUniverseRoomReachability,
    },
    divergent_universe_encounter_catalog::{
        DivergentUniverseBossPoolId, DivergentUniverseEncounterCatalog,
        DivergentUniverseEncounterGroupId, DivergentUniverseEncounterSourceId,
        DivergentUniverseEncounterWaveId, DivergentUniverseEnemySlotId,
    },
    divergent_universe_progression_catalog::{
        DivergentUniverseProgressionCatalog, DivergentUniverseWeeklyModifierId,
    },
};

use super::DivergentUniverseRuntimeFactory;

const STAGE_CONFIG_SOURCE: &str = "divergent-universe.encounter-source.stageconfig";
const GROUP_PREFIX: &str = "divergent-universe.encounter-group.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEncounterReachabilityAccuracy {
    ExactReleasedRoomStageFlowWeeklyDisplayAndStageConfigClosure,
    VersionedProjectPolicyAreaAndRoomSelectorsRejectWithoutMutation,
    VersionedProjectPolicyExplicitDisplayCandidateSelection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEncounterSourceKind {
    AreaEntry,
    RoomCandidate,
    SharedStageConfigRoot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseEncounterSourcePolicy {
    AreaEntryStageBoundary,
    EncounterRoomSelector,
    StageConfigCandidateBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterSourceRuntimeDefinition {
    id: DivergentUniverseEncounterSourceId,
    parent_id: Box<str>,
    kind: DivergentUniverseEncounterSourceKind,
    policy: DivergentUniverseEncounterSourcePolicy,
    resolution_state: Box<str>,
    encounter_group_count: usize,
    stage_count: usize,
}

impl DivergentUniverseEncounterSourceRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEncounterSourceId {
        &self.id
    }
    #[must_use]
    pub fn parent_id(&self) -> &str {
        &self.parent_id
    }
    #[must_use]
    pub const fn kind(&self) -> DivergentUniverseEncounterSourceKind {
        self.kind
    }
    #[must_use]
    pub const fn policy(&self) -> DivergentUniverseEncounterSourcePolicy {
        self.policy
    }
    #[must_use]
    pub fn resolution_state(&self) -> &str {
        &self.resolution_state
    }
    #[must_use]
    pub const fn encounter_group_count(&self) -> usize {
        self.encounter_group_count
    }
    #[must_use]
    pub const fn stage_count(&self) -> usize {
        self.stage_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseRoomReachabilityRuntimeDefinition {
    id: DivergentUniverseRoomCandidateId,
    source_id: Box<str>,
    room_type: Box<str>,
}

impl DivergentUniverseRoomReachabilityRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseRoomCandidateId {
        &self.id
    }
    #[must_use]
    pub fn source_id(&self) -> &str {
        &self.source_id
    }
    #[must_use]
    pub fn room_type(&self) -> &str {
        &self.room_type
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseStageFlowRuntimeDefinition {
    id: DivergentUniverseFlowId,
    policy_id: Box<str>,
}

impl DivergentUniverseStageFlowRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseFlowId {
        &self.id
    }
    #[must_use]
    pub fn policy_id(&self) -> &str {
        &self.policy_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEnemySlotRuntimeDefinition {
    id: DivergentUniverseEnemySlotId,
    slot_index: u16,
    monster_id: Box<str>,
    enemy_id: Box<str>,
    level: Box<str>,
    ability_refs: Box<[Box<str>]>,
}

impl DivergentUniverseEnemySlotRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEnemySlotId {
        &self.id
    }
    #[must_use]
    pub const fn slot_index(&self) -> u16 {
        self.slot_index
    }
    #[must_use]
    pub fn monster_id(&self) -> &str {
        &self.monster_id
    }
    #[must_use]
    pub fn enemy_id(&self) -> &str {
        &self.enemy_id
    }
    #[must_use]
    pub fn level(&self) -> &str {
        &self.level
    }
    #[must_use]
    pub fn ability_refs(&self) -> &[Box<str>] {
        &self.ability_refs
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterWaveRuntimeDefinition {
    id: DivergentUniverseEncounterWaveId,
    wave_index: u16,
    trigger: Box<str>,
    slots: Arc<[DivergentUniverseEnemySlotRuntimeDefinition]>,
    hard_level_group: Box<str>,
    level: Box<str>,
    stage_ability_refs: Box<[Box<str>]>,
}

impl DivergentUniverseEncounterWaveRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEncounterWaveId {
        &self.id
    }
    #[must_use]
    pub const fn wave_index(&self) -> u16 {
        self.wave_index
    }
    #[must_use]
    pub fn trigger(&self) -> &str {
        &self.trigger
    }
    #[must_use]
    pub fn slots(&self) -> &[DivergentUniverseEnemySlotRuntimeDefinition] {
        &self.slots
    }
    #[must_use]
    pub fn hard_level_group(&self) -> &str {
        &self.hard_level_group
    }
    #[must_use]
    pub fn level(&self) -> &str {
        &self.level
    }
    #[must_use]
    pub fn stage_ability_refs(&self) -> &[Box<str>] {
        &self.stage_ability_refs
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterStageRuntimeDefinition {
    stage_id: Box<str>,
    waves: Arc<[DivergentUniverseEncounterWaveRuntimeDefinition]>,
}

impl DivergentUniverseEncounterStageRuntimeDefinition {
    #[must_use]
    pub fn stage_id(&self) -> &str {
        &self.stage_id
    }
    #[must_use]
    pub fn waves(&self) -> &[DivergentUniverseEncounterWaveRuntimeDefinition] {
        &self.waves
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterGroupRuntimeDefinition {
    id: DivergentUniverseEncounterGroupId,
    candidate_stage_ids: Box<[Box<str>]>,
    display_binding_ids: Box<[Box<str>]>,
    display_roles: Box<[Box<str>]>,
}

impl DivergentUniverseEncounterGroupRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseEncounterGroupId {
        &self.id
    }
    #[must_use]
    pub fn candidate_stage_ids(&self) -> &[Box<str>] {
        &self.candidate_stage_ids
    }
    #[must_use]
    pub fn display_binding_ids(&self) -> &[Box<str>] {
        &self.display_binding_ids
    }
    #[must_use]
    pub fn display_roles(&self) -> &[Box<str>] {
        &self.display_roles
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWeeklyDisplayRuntimeDefinition {
    id: DivergentUniverseWeeklyModifierId,
    encounter_groups: Box<[DivergentUniverseEncounterGroupId]>,
    effect_ids: Box<[Box<str>]>,
}

impl DivergentUniverseWeeklyDisplayRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseWeeklyModifierId {
        &self.id
    }
    #[must_use]
    pub fn encounter_groups(&self) -> &[DivergentUniverseEncounterGroupId] {
        &self.encounter_groups
    }
    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseBossPoolRuntimeDefinition {
    id: DivergentUniverseBossPoolId,
    weekly_modifier: DivergentUniverseWeeklyModifierId,
    encounter_group: DivergentUniverseEncounterGroupId,
    display_slot: Box<str>,
    display_variant: Box<str>,
    candidate_monster_ids: Box<[Box<str>]>,
    candidate_stage_ids: Box<[Box<str>]>,
}

impl DivergentUniverseBossPoolRuntimeDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseBossPoolId {
        &self.id
    }
    #[must_use]
    pub const fn weekly_modifier(&self) -> &DivergentUniverseWeeklyModifierId {
        &self.weekly_modifier
    }
    #[must_use]
    pub const fn encounter_group(&self) -> &DivergentUniverseEncounterGroupId {
        &self.encounter_group
    }
    #[must_use]
    pub fn display_slot(&self) -> &str {
        &self.display_slot
    }
    #[must_use]
    pub fn display_variant(&self) -> &str {
        &self.display_variant
    }
    #[must_use]
    pub fn candidate_monster_ids(&self) -> &[Box<str>] {
        &self.candidate_monster_ids
    }
    #[must_use]
    pub fn candidate_stage_ids(&self) -> &[Box<str>] {
        &self.candidate_stage_ids
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniverseEncounterSelectionDigest([u8; 32]);

impl DivergentUniverseEncounterSelectionDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseEncounterSelection {
    component_digest: [u8; 32],
    source_state_hash: ActivityStateHash,
    encounter_group: DivergentUniverseEncounterGroupId,
    stage_id: Box<str>,
    waves: Arc<[DivergentUniverseEncounterWaveRuntimeDefinition]>,
    digest: DivergentUniverseEncounterSelectionDigest,
}

impl DivergentUniverseEncounterSelection {
    #[must_use]
    pub const fn component_digest(&self) -> [u8; 32] {
        self.component_digest
    }
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
    #[must_use]
    pub const fn encounter_group(&self) -> &DivergentUniverseEncounterGroupId {
        &self.encounter_group
    }
    #[must_use]
    pub fn stage_id(&self) -> &str {
        &self.stage_id
    }
    #[must_use]
    pub fn waves(&self) -> &[DivergentUniverseEncounterWaveRuntimeDefinition] {
        &self.waves
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniverseEncounterSelectionDigest {
        self.digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWeeklyBossSelection {
    weekly_modifier: DivergentUniverseWeeklyModifierId,
    boss_pool: DivergentUniverseBossPoolId,
    monster_id: Box<str>,
    encounter: DivergentUniverseEncounterSelection,
}

impl DivergentUniverseWeeklyBossSelection {
    #[must_use]
    pub const fn weekly_modifier(&self) -> &DivergentUniverseWeeklyModifierId {
        &self.weekly_modifier
    }
    #[must_use]
    pub const fn boss_pool(&self) -> &DivergentUniverseBossPoolId {
        &self.boss_pool
    }
    #[must_use]
    pub fn monster_id(&self) -> &str {
        &self.monster_id
    }
    #[must_use]
    pub const fn encounter(&self) -> &DivergentUniverseEncounterSelection {
        &self.encounter
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseEncounterReachabilityRuntime {
    component_digest: [u8; 32],
    sources: Arc<[DivergentUniverseEncounterSourceRuntimeDefinition]>,
    rooms: Arc<[DivergentUniverseRoomReachabilityRuntimeDefinition]>,
    stage_flows: Arc<[DivergentUniverseStageFlowRuntimeDefinition]>,
    weekly_displays: Arc<[DivergentUniverseWeeklyDisplayRuntimeDefinition]>,
    groups: Arc<[DivergentUniverseEncounterGroupRuntimeDefinition]>,
    stages: Arc<[DivergentUniverseEncounterStageRuntimeDefinition]>,
    boss_pools: Arc<[DivergentUniverseBossPoolRuntimeDefinition]>,
}

impl DivergentUniverseRuntimeFactory {
    pub fn encounter_reachability_runtime(
        &self,
    ) -> Result<
        DivergentUniverseEncounterReachabilityRuntime,
        DivergentUniverseEncounterReachabilityError,
    > {
        DivergentUniverseEncounterReachabilityRuntime::compile(
            self.bundle.catalog(),
            self.bundle.progression_catalog(),
            self.bundle.encounter_catalog(),
            self.bundle.identity().component_digest().bytes(),
        )
    }
}

impl DivergentUniverseEncounterReachabilityRuntime {
    fn compile(
        flow: &DivergentUniverseFlowCatalog,
        progression: &DivergentUniverseProgressionCatalog,
        encounter: &DivergentUniverseEncounterCatalog,
        component_digest: [u8; 32],
    ) -> Result<Self, DivergentUniverseEncounterReachabilityError> {
        let groups = lower_groups(encounter)?;
        let stages = lower_stages(encounter)?;
        validate_group_stage_closure(&groups, &stages)?;
        let sources = lower_sources(encounter, &groups, &stages)?;
        let rooms = lower_rooms(flow)?;
        let stage_flows = flow
            .flows()
            .iter()
            .map(|value| DivergentUniverseStageFlowRuntimeDefinition {
                id: value.id.clone(),
                policy_id: value.policy_id.clone(),
            })
            .collect::<Vec<_>>();
        let weekly_displays = lower_weekly(progression, &groups)?;
        let boss_pools = lower_boss_pools(encounter, &weekly_displays, &groups)?;
        if sources.len() != 877
            || rooms.len() != 848
            || stage_flows.len() != 111
            || weekly_displays.len() != 103
            || groups.len() != 43
            || stages.len() != 118
            || stages.iter().map(|value| value.waves.len()).sum::<usize>() != 176
            || stages
                .iter()
                .flat_map(|value| value.waves.iter())
                .map(|value| value.slots.len())
                .sum::<usize>()
                != 385
            || boss_pools.len() != 618
        {
            return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
        }
        Ok(Self {
            component_digest,
            sources: sources.into(),
            rooms: rooms.into(),
            stage_flows: stage_flows.into(),
            weekly_displays: weekly_displays.into(),
            groups: groups.into(),
            stages: stages.into(),
            boss_pools: boss_pools.into(),
        })
    }

    #[must_use]
    pub const fn accuracies(&self) -> [DivergentUniverseEncounterReachabilityAccuracy; 3] {
        [
            DivergentUniverseEncounterReachabilityAccuracy::ExactReleasedRoomStageFlowWeeklyDisplayAndStageConfigClosure,
            DivergentUniverseEncounterReachabilityAccuracy::VersionedProjectPolicyAreaAndRoomSelectorsRejectWithoutMutation,
            DivergentUniverseEncounterReachabilityAccuracy::VersionedProjectPolicyExplicitDisplayCandidateSelection,
        ]
    }
    #[must_use]
    pub fn sources(&self) -> &[DivergentUniverseEncounterSourceRuntimeDefinition] {
        &self.sources
    }
    #[must_use]
    pub fn rooms(&self) -> &[DivergentUniverseRoomReachabilityRuntimeDefinition] {
        &self.rooms
    }
    #[must_use]
    pub fn stage_flows(&self) -> &[DivergentUniverseStageFlowRuntimeDefinition] {
        &self.stage_flows
    }
    #[must_use]
    pub fn weekly_displays(&self) -> &[DivergentUniverseWeeklyDisplayRuntimeDefinition] {
        &self.weekly_displays
    }
    #[must_use]
    pub fn groups(&self) -> &[DivergentUniverseEncounterGroupRuntimeDefinition] {
        &self.groups
    }
    #[must_use]
    pub fn stages(&self) -> &[DivergentUniverseEncounterStageRuntimeDefinition] {
        &self.stages
    }
    #[must_use]
    pub fn boss_pools(&self) -> &[DivergentUniverseBossPoolRuntimeDefinition] {
        &self.boss_pools
    }

    pub fn resolve_source(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        source: &DivergentUniverseEncounterSourceId,
    ) -> Result<
        &DivergentUniverseEncounterSourceRuntimeDefinition,
        DivergentUniverseEncounterReachabilityError,
    > {
        validate_activity(activity, expected)?;
        let source = lookup(&self.sources, |value| &value.id, source)?;
        if source.kind == DivergentUniverseEncounterSourceKind::SharedStageConfigRoot {
            Ok(source)
        } else {
            Err(DivergentUniverseEncounterReachabilityError::SelectorUnavailable)
        }
    }

    pub fn select_stage_candidate(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        group: &DivergentUniverseEncounterGroupId,
        stage_id: &str,
    ) -> Result<DivergentUniverseEncounterSelection, DivergentUniverseEncounterReachabilityError>
    {
        validate_activity(activity, expected)?;
        let group = lookup(&self.groups, |value| &value.id, group)?;
        if group
            .candidate_stage_ids
            .binary_search_by(|value| value.as_ref().cmp(stage_id))
            .is_err()
        {
            return Err(DivergentUniverseEncounterReachabilityError::StageNotInGroup);
        }
        let stage = lookup(&self.stages, |value| value.stage_id.as_ref(), stage_id)?;
        Ok(selection(
            self.component_digest,
            activity.state_hash(),
            &group.id,
            stage,
        ))
    }

    pub fn select_weekly_boss_candidate(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        weekly: &DivergentUniverseWeeklyModifierId,
        pool: &DivergentUniverseBossPoolId,
        stage_id: &str,
    ) -> Result<DivergentUniverseWeeklyBossSelection, DivergentUniverseEncounterReachabilityError>
    {
        validate_activity(activity, expected)?;
        let display = lookup(&self.weekly_displays, |value| &value.id, weekly)?;
        let pool = lookup(&self.boss_pools, |value| &value.id, pool)?;
        if &pool.weekly_modifier != weekly
            || display
                .encounter_groups
                .binary_search(&pool.encounter_group)
                .is_err()
        {
            return Err(DivergentUniverseEncounterReachabilityError::WeeklyPoolMismatch);
        }
        let index = pool
            .candidate_stage_ids
            .binary_search_by(|value| value.as_ref().cmp(stage_id))
            .map_err(|_| DivergentUniverseEncounterReachabilityError::StageNotInGroup)?;
        let encounter =
            self.select_stage_candidate(activity, expected, &pool.encounter_group, stage_id)?;
        Ok(DivergentUniverseWeeklyBossSelection {
            weekly_modifier: weekly.clone(),
            boss_pool: pool.id.clone(),
            monster_id: pool.candidate_monster_ids[index].clone(),
            encounter,
        })
    }
}

fn lower_sources(
    encounter: &DivergentUniverseEncounterCatalog,
    groups: &[DivergentUniverseEncounterGroupRuntimeDefinition],
    stages: &[DivergentUniverseEncounterStageRuntimeDefinition],
) -> Result<
    Vec<DivergentUniverseEncounterSourceRuntimeDefinition>,
    DivergentUniverseEncounterReachabilityError,
> {
    encounter
        .sources()
        .iter()
        .map(|value| {
            if value.runtime_lowered || value.blocking {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            let (kind, policy) = match value.parent_kind.as_ref() {
                "AreaEntry" if value.encounter_groups.is_empty() && value.stage_ids.is_empty() => (
                    DivergentUniverseEncounterSourceKind::AreaEntry,
                    DivergentUniverseEncounterSourcePolicy::AreaEntryStageBoundary,
                ),
                "RoomCandidate"
                    if value.encounter_groups.is_empty() && value.stage_ids.is_empty() =>
                {
                    (
                        DivergentUniverseEncounterSourceKind::RoomCandidate,
                        DivergentUniverseEncounterSourcePolicy::EncounterRoomSelector,
                    )
                }
                "SharedStageConfigRoot"
                    if value.id.as_str() == STAGE_CONFIG_SOURCE
                        && value.encounter_groups
                            == groups
                                .iter()
                                .map(|group| group.id.clone())
                                .collect::<Vec<_>>()
                                .into_boxed_slice()
                        && value.stage_ids
                            == stages
                                .iter()
                                .map(|stage| stage.stage_id.clone())
                                .collect::<Vec<_>>()
                                .into_boxed_slice() =>
                {
                    (
                        DivergentUniverseEncounterSourceKind::SharedStageConfigRoot,
                        DivergentUniverseEncounterSourcePolicy::StageConfigCandidateBoundary,
                    )
                }
                _ => return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog),
            };
            Ok(DivergentUniverseEncounterSourceRuntimeDefinition {
                id: value.id.clone(),
                parent_id: value.parent_id.clone(),
                kind,
                policy,
                resolution_state: value.resolution_state.clone(),
                encounter_group_count: value.encounter_groups.len(),
                stage_count: value.stage_ids.len(),
            })
        })
        .collect()
}

fn lower_rooms(
    flow: &DivergentUniverseFlowCatalog,
) -> Result<
    Vec<DivergentUniverseRoomReachabilityRuntimeDefinition>,
    DivergentUniverseEncounterReachabilityError,
> {
    flow.room_candidates()
        .iter()
        .map(|value| {
            if value.reachability != DivergentUniverseRoomReachability::UnprovenSharedCandidate
                || !value.stage_refs.is_empty()
                || !value.offered_pool_ids.is_empty()
            {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            Ok(DivergentUniverseRoomReachabilityRuntimeDefinition {
                id: value.id.clone(),
                source_id: value.source_id.clone(),
                room_type: value.room_type.clone(),
            })
        })
        .collect()
}

fn lower_groups(
    encounter: &DivergentUniverseEncounterCatalog,
) -> Result<
    Vec<DivergentUniverseEncounterGroupRuntimeDefinition>,
    DivergentUniverseEncounterReachabilityError,
> {
    encounter
        .groups()
        .iter()
        .map(|value| {
            if value.runtime_lowered
                || value.selection_policy.as_ref() != "DisplayOnlyNoEnabledWeeklySelector"
                || value.reachability_disposition.as_ref() != "UnprovenWeeklyDisplayCandidate"
            {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            Ok(DivergentUniverseEncounterGroupRuntimeDefinition {
                id: value.id.clone(),
                candidate_stage_ids: value.candidate_stage_ids.clone(),
                display_binding_ids: value.display_binding_ids.clone(),
                display_roles: value.display_roles.clone(),
            })
        })
        .collect()
}

fn lower_stages(
    encounter: &DivergentUniverseEncounterCatalog,
) -> Result<
    Vec<DivergentUniverseEncounterStageRuntimeDefinition>,
    DivergentUniverseEncounterReachabilityError,
> {
    let slots = encounter
        .slots()
        .iter()
        .map(|value| {
            (
                value.id.clone(),
                DivergentUniverseEnemySlotRuntimeDefinition {
                    id: value.id.clone(),
                    slot_index: value.slot_index,
                    monster_id: value.monster_id.clone(),
                    enemy_id: value.enemy_id.clone(),
                    level: value.level.clone(),
                    ability_refs: value.ability_refs.clone(),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    if slots.len() != 385 {
        return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
    }
    let mut by_stage = BTreeMap::<Box<str>, Vec<_>>::new();
    for wave in encounter.waves() {
        let lowered_slots = wave
            .enemy_slots
            .iter()
            .map(|id| {
                slots
                    .get(id)
                    .cloned()
                    .ok_or(DivergentUniverseEncounterReachabilityError::InvalidCatalog)
            })
            .collect::<Result<Vec<_>, _>>()?;
        if lowered_slots
            .iter()
            .enumerate()
            .any(|(index, slot)| usize::from(slot.slot_index) != index + 1)
        {
            return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
        }
        by_stage.entry(wave.stage_id.clone()).or_default().push(
            DivergentUniverseEncounterWaveRuntimeDefinition {
                id: wave.id.clone(),
                wave_index: wave.wave_index,
                trigger: wave.trigger.clone(),
                slots: lowered_slots.into(),
                hard_level_group: wave.hard_level_group.clone(),
                level: wave.level.clone(),
                stage_ability_refs: wave.stage_ability_refs.clone(),
            },
        );
    }
    by_stage
        .into_iter()
        .map(|(stage_id, mut waves)| {
            waves.sort_unstable_by_key(|wave| wave.wave_index);
            if waves
                .iter()
                .enumerate()
                .any(|(index, wave)| usize::from(wave.wave_index) != index + 1)
            {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            Ok(DivergentUniverseEncounterStageRuntimeDefinition {
                stage_id,
                waves: waves.into(),
            })
        })
        .collect()
}

fn validate_group_stage_closure(
    groups: &[DivergentUniverseEncounterGroupRuntimeDefinition],
    stages: &[DivergentUniverseEncounterStageRuntimeDefinition],
) -> Result<(), DivergentUniverseEncounterReachabilityError> {
    if groups.iter().any(|group| {
        group.candidate_stage_ids.is_empty()
            || group.candidate_stage_ids.iter().any(|stage| {
                stages
                    .binary_search_by(|value| value.stage_id.cmp(stage))
                    .is_err()
            })
    }) {
        Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog)
    } else {
        Ok(())
    }
}

fn lower_weekly(
    progression: &DivergentUniverseProgressionCatalog,
    groups: &[DivergentUniverseEncounterGroupRuntimeDefinition],
) -> Result<
    Vec<DivergentUniverseWeeklyDisplayRuntimeDefinition>,
    DivergentUniverseEncounterReachabilityError,
> {
    progression
        .weekly_modifiers()
        .iter()
        .map(|value| {
            if value.runtime_lowered
                || value.reachability.as_ref() != "UnprovenCurrentWeeklyCandidate"
                || value.enemy_groups.len() != 6
            {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            let mut encounter_groups = value
                .enemy_groups
                .iter()
                .map(|entry| {
                    DivergentUniverseEncounterGroupId::new(format!(
                        "{GROUP_PREFIX}{}",
                        entry.source_group_id
                    ))
                    .map_err(|_| DivergentUniverseEncounterReachabilityError::InvalidCatalog)
                })
                .collect::<Result<Vec<_>, _>>()?;
            encounter_groups.sort_unstable();
            encounter_groups.dedup();
            if encounter_groups
                .iter()
                .any(|id| groups.binary_search_by(|group| group.id.cmp(id)).is_err())
            {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            Ok(DivergentUniverseWeeklyDisplayRuntimeDefinition {
                id: value.id.clone(),
                encounter_groups: encounter_groups.into(),
                effect_ids: value.effect_ids.clone(),
            })
        })
        .collect()
}

fn lower_boss_pools(
    encounter: &DivergentUniverseEncounterCatalog,
    weekly: &[DivergentUniverseWeeklyDisplayRuntimeDefinition],
    groups: &[DivergentUniverseEncounterGroupRuntimeDefinition],
) -> Result<
    Vec<DivergentUniverseBossPoolRuntimeDefinition>,
    DivergentUniverseEncounterReachabilityError,
> {
    encounter
        .boss_pools()
        .iter()
        .map(|value| {
            let display = lookup(
                weekly,
                |entry| &entry.id,
                &DivergentUniverseWeeklyModifierId::new(value.weekly_modifier_id.clone())
                    .map_err(|_| DivergentUniverseEncounterReachabilityError::InvalidCatalog)?,
            )?;
            let group = lookup(groups, |entry| &entry.id, &value.encounter_group)?;
            if value.runtime_lowered
                || value.selection_policy.as_ref() != "DisplayOnlyNoEnabledWeeklySelector"
                || value.fallback.as_ref() != "FailClosedWithoutCurrentWeeklySelector"
                || value.candidate_stage_ids != group.candidate_stage_ids
                || value.candidate_monster_ids.len() != value.candidate_stage_ids.len()
                || display
                    .encounter_groups
                    .binary_search(&value.encounter_group)
                    .is_err()
            {
                return Err(DivergentUniverseEncounterReachabilityError::InvalidCatalog);
            }
            Ok(DivergentUniverseBossPoolRuntimeDefinition {
                id: value.id.clone(),
                weekly_modifier: display.id.clone(),
                encounter_group: value.encounter_group.clone(),
                display_slot: value.display_slot.clone(),
                display_variant: value.display_variant.clone(),
                candidate_monster_ids: value.candidate_monster_ids.clone(),
                candidate_stage_ids: value.candidate_stage_ids.clone(),
            })
        })
        .collect()
}

fn selection(
    component_digest: [u8; 32],
    source_state_hash: ActivityStateHash,
    group: &DivergentUniverseEncounterGroupId,
    stage: &DivergentUniverseEncounterStageRuntimeDefinition,
) -> DivergentUniverseEncounterSelection {
    let mut hasher = CanonicalDigestBuilder::new();
    hasher.update(b"starclock.divergent-universe.encounter-selection.v1");
    hasher.update(component_digest);
    hasher.update(source_state_hash.bytes());
    hash_text(&mut hasher, group.as_str());
    hash_text(&mut hasher, &stage.stage_id);
    for wave in stage.waves.iter() {
        hash_text(&mut hasher, wave.id.as_str());
        hasher.update(wave.wave_index.to_be_bytes());
        for slot in wave.slots.iter() {
            hash_text(&mut hasher, slot.id.as_str());
            hash_text(&mut hasher, &slot.monster_id);
        }
    }
    DivergentUniverseEncounterSelection {
        component_digest,
        source_state_hash,
        encounter_group: group.clone(),
        stage_id: stage.stage_id.clone(),
        waves: Arc::clone(&stage.waves),
        digest: DivergentUniverseEncounterSelectionDigest(hasher.finalize()),
    }
}

fn hash_text(hasher: &mut CanonicalDigestBuilder, value: &str) {
    hasher.update(
        u64::try_from(value.len())
            .expect("stable ID length fits u64")
            .to_be_bytes(),
    );
    hasher.update(value.as_bytes());
}

fn validate_activity(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniverseEncounterReachabilityError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniverseEncounterReachabilityError::StaleStateHash)
    }
}

fn lookup<'a, T, I: Ord + ?Sized>(
    values: &'a [T],
    id: impl Fn(&T) -> &I,
    expected: &I,
) -> Result<&'a T, DivergentUniverseEncounterReachabilityError> {
    values
        .binary_search_by(|value| id(value).cmp(expected))
        .ok()
        .map(|index| &values[index])
        .ok_or(DivergentUniverseEncounterReachabilityError::UnknownIdentity)
}

#[derive(Debug, Eq, PartialEq)]
pub enum DivergentUniverseEncounterReachabilityError {
    InvalidCatalog,
    UnknownIdentity,
    SelectorUnavailable,
    StageNotInGroup,
    WeeklyPoolMismatch,
    StaleStateHash,
}

impl core::fmt::Display for DivergentUniverseEncounterReachabilityError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe encounter reachability error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseEncounterReachabilityError {}
