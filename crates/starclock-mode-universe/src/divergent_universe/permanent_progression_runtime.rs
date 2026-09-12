//! Permanent talents, finish unlocks, weekly modifiers and room marks.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::{
    divergent_universe_catalog::{DivergentUniverseAreaId, DivergentUniverseRunFamily},
    divergent_universe_progression_catalog::{
        DivergentUniversePermanentTalentId, DivergentUniverseProgressionCatalog,
        DivergentUniverseProgressionEffectId, DivergentUniverseRoomMarkId,
        DivergentUniverseUnlockId, DivergentUniverseWeeklyModifierId,
    },
    divergent_universe_service_catalog::DivergentUniverseModeServiceId,
};

use super::state::{
    PERMANENT_TALENT_CURRENCY_SLOT, PERMANENT_UNLOCKS_SLOT, ROOM_MARK_SLOT, WEEKLY_MODIFIER_SLOT,
};
use super::{DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory};

const TALENT_KEY_BASE: u64 = 0x2254_0000;
const UNLOCK_KEY_BASE: u64 = 0x2255_0000;
const CREDIT_TALENT_CURRENCY_PROGRAM: u32 = 22_541;
const UNLOCK_TALENT_PROGRAM: u32 = 22_542;
const RECORD_FINISH_UNLOCK_PROGRAM: u32 = 22_543;
const ACTIVATE_WEEKLY_PROGRAM: u32 = 22_544;
const APPLY_ROOM_MARK_PROGRAM: u32 = 22_545;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniversePermanentProgressionAccuracy {
    ExactReleasedCostsEffectsAdjacencyAndAreaConsumers,
    VersionedProjectPolicyNoInventedPrerequisiteDirection,
    VersionedProjectPolicyAcceptedWeeklySelectionNotObservedParity,
    VersionedProjectPolicyPreserveRoomMarkWithoutTransitionProgram,
    VersionedProjectPolicyRejectMissingServiceGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseProgressionLifetime {
    ProfileResetOnly,
    CyclicalEpoch,
    Node,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseProgressionContributionScope {
    Activity,
    Battle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseProgressionContribution {
    id: DivergentUniverseProgressionEffectId,
    talent: DivergentUniversePermanentTalentId,
    scope: DivergentUniverseProgressionContributionScope,
    operation: Box<str>,
    metric: Box<str>,
    condition: Box<str>,
    parameters: Box<[Box<str>]>,
}

impl DivergentUniverseProgressionContribution {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseProgressionEffectId {
        &self.id
    }
    #[must_use]
    pub const fn talent(&self) -> &DivergentUniversePermanentTalentId {
        &self.talent
    }
    #[must_use]
    pub const fn scope(&self) -> DivergentUniverseProgressionContributionScope {
        self.scope
    }
    #[must_use]
    pub fn operation(&self) -> &str {
        &self.operation
    }
    #[must_use]
    pub fn metric(&self) -> &str {
        &self.metric
    }
    #[must_use]
    pub fn condition(&self) -> &str {
        &self.condition
    }
    #[must_use]
    pub fn parameters(&self) -> &[Box<str>] {
        &self.parameters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePermanentTalentRuntime {
    id: DivergentUniversePermanentTalentId,
    state_key: u64,
    adjacent: Box<[DivergentUniversePermanentTalentId]>,
    prerequisite_resolution: Box<str>,
    cost_item_id: Box<str>,
    cost: u32,
    contribution: DivergentUniverseProgressionContribution,
    important: bool,
}

impl DivergentUniversePermanentTalentRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniversePermanentTalentId {
        &self.id
    }
    #[must_use]
    pub const fn state_key(&self) -> u64 {
        self.state_key
    }
    #[must_use]
    pub fn adjacent(&self) -> &[DivergentUniversePermanentTalentId] {
        &self.adjacent
    }
    #[must_use]
    pub fn prerequisite_resolution(&self) -> &str {
        &self.prerequisite_resolution
    }
    #[must_use]
    pub fn cost_item_id(&self) -> &str {
        &self.cost_item_id
    }
    #[must_use]
    pub const fn cost(&self) -> u32 {
        self.cost
    }
    #[must_use]
    pub const fn contribution(&self) -> &DivergentUniverseProgressionContribution {
        &self.contribution
    }
    #[must_use]
    pub const fn important(&self) -> bool {
        self.important
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseUnlockConsumerResolution {
    ExactCurrentTourn3AreaAvailability,
    MissingPublishedConsumer,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseUnlockRuntime {
    id: DivergentUniverseUnlockId,
    state_key: u64,
    finish_condition_id: Box<str>,
    areas: Box<[DivergentUniverseAreaId]>,
    resolution: DivergentUniverseUnlockConsumerResolution,
}

impl DivergentUniverseUnlockRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseUnlockId {
        &self.id
    }
    #[must_use]
    pub const fn state_key(&self) -> u64 {
        self.state_key
    }
    #[must_use]
    pub fn finish_condition_id(&self) -> &str {
        &self.finish_condition_id
    }
    #[must_use]
    pub fn areas(&self) -> &[DivergentUniverseAreaId] {
        &self.areas
    }
    #[must_use]
    pub const fn resolution(&self) -> DivergentUniverseUnlockConsumerResolution {
        self.resolution
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseWeeklyModifierRuntime {
    id: DivergentUniverseWeeklyModifierId,
    key: u64,
    content_ids: Box<[Box<str>]>,
    detail_ids: Box<[Box<str>]>,
    effect_ids: Box<[Box<str>]>,
    enemy_group_count: u16,
    reachability: Box<str>,
}

impl DivergentUniverseWeeklyModifierRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseWeeklyModifierId {
        &self.id
    }
    #[must_use]
    pub fn content_ids(&self) -> &[Box<str>] {
        &self.content_ids
    }
    #[must_use]
    pub fn detail_ids(&self) -> &[Box<str>] {
        &self.detail_ids
    }
    #[must_use]
    pub fn effect_ids(&self) -> &[Box<str>] {
        &self.effect_ids
    }
    #[must_use]
    pub const fn enemy_group_count(&self) -> u16 {
        self.enemy_group_count
    }
    #[must_use]
    pub fn reachability(&self) -> &str {
        &self.reachability
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseRoomMarkRuntime {
    id: DivergentUniverseRoomMarkId,
    key: u64,
    room_type: Box<str>,
    mark_kind: Box<str>,
    fallback: Box<str>,
}

impl DivergentUniverseRoomMarkRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseRoomMarkId {
        &self.id
    }
    #[must_use]
    pub fn room_type(&self) -> &str {
        &self.room_type
    }
    #[must_use]
    pub fn mark_kind(&self) -> &str {
        &self.mark_kind
    }
    #[must_use]
    pub fn fallback(&self) -> &str {
        &self.fallback
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseMissingServiceRuntime {
    id: DivergentUniverseModeServiceId,
    graph_path: Box<str>,
}

impl DivergentUniverseMissingServiceRuntime {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseModeServiceId {
        &self.id
    }
    #[must_use]
    pub fn graph_path(&self) -> &str {
        &self.graph_path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePermanentProgressionResolution {
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniversePermanentProgressionResolution {
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DivergentUniversePermanentProgressionSnapshotDigest([u8; 32]);

impl DivergentUniversePermanentProgressionSnapshotDigest {
    #[must_use]
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePermanentProgressionSnapshot {
    source_state_hash: ActivityStateHash,
    talents: Box<[DivergentUniversePermanentTalentId]>,
    finish_unlocks: Box<[DivergentUniverseUnlockId]>,
    unlocked_areas: Box<[DivergentUniverseAreaId]>,
    weekly_modifier: Option<DivergentUniverseWeeklyModifierId>,
    room_mark: Option<DivergentUniverseRoomMarkId>,
    contributions: Box<[DivergentUniverseProgressionContribution]>,
    digest: DivergentUniversePermanentProgressionSnapshotDigest,
}

impl DivergentUniversePermanentProgressionSnapshot {
    #[must_use]
    pub fn talents(&self) -> &[DivergentUniversePermanentTalentId] {
        &self.talents
    }
    #[must_use]
    pub fn finish_unlocks(&self) -> &[DivergentUniverseUnlockId] {
        &self.finish_unlocks
    }
    #[must_use]
    pub fn unlocked_areas(&self) -> &[DivergentUniverseAreaId] {
        &self.unlocked_areas
    }
    #[must_use]
    pub const fn weekly_modifier(&self) -> Option<&DivergentUniverseWeeklyModifierId> {
        self.weekly_modifier.as_ref()
    }
    #[must_use]
    pub const fn room_mark(&self) -> Option<&DivergentUniverseRoomMarkId> {
        self.room_mark.as_ref()
    }
    #[must_use]
    pub fn contributions(&self) -> &[DivergentUniverseProgressionContribution] {
        &self.contributions
    }
    #[must_use]
    pub const fn digest(&self) -> DivergentUniversePermanentProgressionSnapshotDigest {
        self.digest
    }
    #[must_use]
    pub const fn source_state_hash(&self) -> ActivityStateHash {
        self.source_state_hash
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniversePermanentProgressionRuntime {
    talents: Arc<[DivergentUniversePermanentTalentRuntime]>,
    unlocks: Arc<[DivergentUniverseUnlockRuntime]>,
    weekly_modifiers: Arc<[DivergentUniverseWeeklyModifierRuntime]>,
    room_marks: Arc<[DivergentUniverseRoomMarkRuntime]>,
    missing_services: Arc<[DivergentUniverseMissingServiceRuntime]>,
    component_digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn permanent_progression_runtime(
        &self,
    ) -> Result<
        DivergentUniversePermanentProgressionRuntime,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        DivergentUniversePermanentProgressionRuntime::compile(
            self.bundle.progression_catalog(),
            self.bundle.service_catalog(),
            self.bundle.identity().component_digest().bytes(),
        )
    }
}

impl DivergentUniversePermanentProgressionRuntime {
    fn compile(
        catalog: &DivergentUniverseProgressionCatalog,
        services: &starclock_data::divergent_universe_service_catalog::DivergentUniverseServiceCatalog,
        component_digest: [u8; 32],
    ) -> Result<Self, DivergentUniversePermanentProgressionRuntimeError> {
        let talents = catalog
            .talents()
            .iter()
            .enumerate()
            .map(|(index, talent)| {
                let effect = catalog
                    .effects()
                    .binary_search_by(|value| value.id.cmp(&talent.effects[0]))
                    .ok()
                    .and_then(|effect_index| catalog.effects().get(effect_index))
                    .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog)?;
                let program = &effect.contributions[0];
                if program.operation != talent.program.operation
                    || program.metric != talent.program.metric
                    || program.condition != talent.program.condition
                    || program.parameters != talent.program.parameters
                    || effect.scope != talent.scope
                {
                    return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog);
                }
                let scope = contribution_scope(&effect.scope)?;
                let cost = positive_u32(&talent.cost[0].amount)?;
                Ok(DivergentUniversePermanentTalentRuntime {
                    id: talent.id.clone(),
                    state_key: reserved_key(TALENT_KEY_BASE, index)?,
                    adjacent: talent.adjacent.clone(),
                    prerequisite_resolution: talent.prerequisite_resolution.clone(),
                    cost_item_id: talent.cost[0].item_id.clone(),
                    cost,
                    contribution: DivergentUniverseProgressionContribution {
                        id: effect.id.clone(),
                        talent: talent.id.clone(),
                        scope,
                        operation: program.operation.clone(),
                        metric: program.metric.clone(),
                        condition: program.condition.clone(),
                        parameters: program.parameters.clone(),
                    },
                    important: talent.important,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let unlocks = catalog
            .unlocks()
            .iter()
            .enumerate()
            .map(|(index, unlock)| {
                let areas = unlock
                    .unlocked_content_ids
                    .iter()
                    .map(|id| {
                        DivergentUniverseAreaId::new(id.as_ref()).map_err(|_| {
                            DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let resolution = if areas.is_empty() {
                    DivergentUniverseUnlockConsumerResolution::MissingPublishedConsumer
                } else {
                    DivergentUniverseUnlockConsumerResolution::ExactCurrentTourn3AreaAvailability
                };
                Ok(DivergentUniverseUnlockRuntime {
                    id: unlock.id.clone(),
                    state_key: reserved_key(UNLOCK_KEY_BASE, index)?,
                    finish_condition_id: unlock.finish_condition_id.clone(),
                    areas: areas.into_boxed_slice(),
                    resolution,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let weekly_modifiers = catalog
            .weekly_modifiers()
            .iter()
            .enumerate()
            .map(|(index, modifier)| {
                Ok(DivergentUniverseWeeklyModifierRuntime {
                    id: modifier.id.clone(),
                    key: ordinal(index)?,
                    content_ids: modifier.content_ids.clone(),
                    detail_ids: modifier.detail_ids.clone(),
                    effect_ids: modifier.effect_ids.clone(),
                    enemy_group_count: u16::try_from(modifier.enemy_groups.len()).map_err(
                        |_| DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog,
                    )?,
                    reachability: modifier.reachability.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let room_marks = catalog
            .room_marks()
            .iter()
            .enumerate()
            .map(|(index, mark)| {
                Ok(DivergentUniverseRoomMarkRuntime {
                    id: mark.id.clone(),
                    key: ordinal(index)?,
                    room_type: mark.room_type.clone(),
                    mark_kind: mark.mark_kind.clone(),
                    fallback: mark.fallback.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let missing_services = services
            .mode_services()
            .iter()
            .map(|service| DivergentUniverseMissingServiceRuntime {
                id: service.id.clone(),
                graph_path: service.graph_path.clone(),
            })
            .collect::<Vec<_>>();
        if talents.len() != 38
            || talents.iter().any(|talent| {
                talent.prerequisite_resolution.as_ref() != "UnavailableInBidirectionalAdjacency"
                    || talent.cost_item_id.as_ref() != "281018"
                    || talent.adjacent.is_empty()
            })
            || unlocks.len() != 97
            || unlocks
                .iter()
                .filter(|unlock| !unlock.areas.is_empty())
                .count()
                != 8
            || weekly_modifiers.len() != 103
            || weekly_modifiers.iter().any(|modifier| {
                modifier.reachability.as_ref() != "UnprovenCurrentWeeklyCandidate"
                    || modifier.enemy_group_count != 6
            })
            || room_marks.len() != 24
            || room_marks
                .iter()
                .any(|mark| mark.fallback.as_ref() != "PreserveCurrentMark")
            || missing_services.len() != 23
        {
            return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog);
        }
        Ok(Self {
            talents: talents.into(),
            unlocks: unlocks.into(),
            weekly_modifiers: weekly_modifiers.into(),
            room_marks: room_marks.into(),
            missing_services: missing_services.into(),
            component_digest,
        })
    }

    #[must_use]
    pub const fn accuracy(&self) -> [DivergentUniversePermanentProgressionAccuracy; 5] {
        [
            DivergentUniversePermanentProgressionAccuracy::ExactReleasedCostsEffectsAdjacencyAndAreaConsumers,
            DivergentUniversePermanentProgressionAccuracy::VersionedProjectPolicyNoInventedPrerequisiteDirection,
            DivergentUniversePermanentProgressionAccuracy::VersionedProjectPolicyAcceptedWeeklySelectionNotObservedParity,
            DivergentUniversePermanentProgressionAccuracy::VersionedProjectPolicyPreserveRoomMarkWithoutTransitionProgram,
            DivergentUniversePermanentProgressionAccuracy::VersionedProjectPolicyRejectMissingServiceGraph,
        ]
    }
    #[must_use]
    pub fn talents(&self) -> &[DivergentUniversePermanentTalentRuntime] {
        &self.talents
    }
    #[must_use]
    pub fn unlocks(&self) -> &[DivergentUniverseUnlockRuntime] {
        &self.unlocks
    }
    #[must_use]
    pub fn weekly_modifiers(&self) -> &[DivergentUniverseWeeklyModifierRuntime] {
        &self.weekly_modifiers
    }
    #[must_use]
    pub fn room_marks(&self) -> &[DivergentUniverseRoomMarkRuntime] {
        &self.room_marks
    }
    #[must_use]
    pub fn missing_services(&self) -> &[DivergentUniverseMissingServiceRuntime] {
        &self.missing_services
    }
    #[must_use]
    pub const fn talent_lifetime(&self) -> DivergentUniverseProgressionLifetime {
        DivergentUniverseProgressionLifetime::ProfileResetOnly
    }
    #[must_use]
    pub const fn finish_unlock_lifetime(&self) -> DivergentUniverseProgressionLifetime {
        DivergentUniverseProgressionLifetime::ProfileResetOnly
    }
    #[must_use]
    pub const fn weekly_modifier_lifetime(&self) -> DivergentUniverseProgressionLifetime {
        DivergentUniverseProgressionLifetime::CyclicalEpoch
    }
    #[must_use]
    pub const fn room_mark_lifetime(&self) -> DivergentUniverseProgressionLifetime {
        DivergentUniverseProgressionLifetime::Node
    }

    pub fn credit_talent_currency_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        amount: u32,
    ) -> Result<
        DivergentUniversePermanentProgressionResolution,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        validate_hash(activity, expected)?;
        if amount == 0 {
            return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidAmount);
        }
        permanent_currency(activity)?
            .checked_add(i64::from(amount))
            .ok_or(DivergentUniversePermanentProgressionRuntimeError::CurrencyOutOfRange)?;
        self.apply(
            activity,
            expected,
            CREDIT_TALENT_CURRENCY_PROGRAM,
            vec![ActivityOperation::AddToSlot {
                slot: PERMANENT_TALENT_CURRENCY_SLOT,
                delta: literal(ActivityValue::BoundedInteger(i64::from(amount))),
            }],
        )
    }

    pub fn unlock_talent_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        talent: &DivergentUniversePermanentTalentId,
    ) -> Result<
        DivergentUniversePermanentProgressionResolution,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        validate_hash(activity, expected)?;
        let definition = self.talent(talent)?;
        let state = permanent_keys(activity)?;
        if state.binary_search(&definition.state_key).is_ok() {
            return Err(DivergentUniversePermanentProgressionRuntimeError::TalentAlreadyUnlocked);
        }
        if permanent_currency(activity)? < i64::from(definition.cost) {
            return Err(
                DivergentUniversePermanentProgressionRuntimeError::InsufficientTalentCurrency,
            );
        }
        self.apply(
            activity,
            expected,
            UNLOCK_TALENT_PROGRAM,
            vec![
                ActivityOperation::AddToSlot {
                    slot: PERMANENT_TALENT_CURRENCY_SLOT,
                    delta: literal(ActivityValue::BoundedInteger(-i64::from(definition.cost))),
                },
                ActivityOperation::InsertOrderedId {
                    slot: PERMANENT_UNLOCKS_SLOT,
                    id: definition.state_key,
                },
            ],
        )
    }

    pub fn reject_unproven_prerequisite(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        talent: &DivergentUniversePermanentTalentId,
        proposed_prerequisite: &DivergentUniversePermanentTalentId,
    ) -> Result<(), DivergentUniversePermanentProgressionRuntimeError> {
        validate_hash(activity, expected)?;
        let definition = self.talent(talent)?;
        if !definition.adjacent.contains(proposed_prerequisite) {
            return Err(DivergentUniversePermanentProgressionRuntimeError::NotAdjacentTalent);
        }
        Err(DivergentUniversePermanentProgressionRuntimeError::UnresolvedPrerequisiteDirection)
    }

    pub fn record_finish_unlock_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        finish_condition_id: &str,
    ) -> Result<
        DivergentUniversePermanentProgressionResolution,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        validate_hash(activity, expected)?;
        let unlock = self
            .unlocks
            .iter()
            .find(|unlock| unlock.finish_condition_id.as_ref() == finish_condition_id)
            .ok_or(DivergentUniversePermanentProgressionRuntimeError::UnknownIdentity)?;
        if permanent_keys(activity)?
            .binary_search(&unlock.state_key)
            .is_ok()
        {
            return Err(DivergentUniversePermanentProgressionRuntimeError::UnlockAlreadyRecorded);
        }
        self.apply(
            activity,
            expected,
            RECORD_FINISH_UNLOCK_PROGRAM,
            vec![ActivityOperation::InsertOrderedId {
                slot: PERMANENT_UNLOCKS_SLOT,
                id: unlock.state_key,
            }],
        )
    }

    pub fn activate_weekly_modifier_policy_accepted(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        modifier: &DivergentUniverseWeeklyModifierId,
    ) -> Result<
        DivergentUniversePermanentProgressionResolution,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        validate_hash(activity, expected)?;
        if flow.run_family() != DivergentUniverseRunFamily::Cyclical
            || flow.progression().cyclical_epoch().is_none()
        {
            return Err(
                DivergentUniversePermanentProgressionRuntimeError::WeeklyModifierOutsideCyclical,
            );
        }
        if optional_key(activity, WEEKLY_MODIFIER_SLOT)?.is_some() {
            return Err(
                DivergentUniversePermanentProgressionRuntimeError::WeeklyModifierAlreadyActive,
            );
        }
        let definition = self.weekly_modifier(modifier)?;
        self.apply(
            activity,
            expected,
            ACTIVATE_WEEKLY_PROGRAM,
            vec![ActivityOperation::SetSlot {
                slot: WEEKLY_MODIFIER_SLOT,
                value: literal(ActivityValue::OptionalId(Some(definition.key))),
            }],
        )
    }

    pub fn apply_room_mark_policy_accepted(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        room_type: &str,
        mark: &DivergentUniverseRoomMarkId,
    ) -> Result<
        DivergentUniversePermanentProgressionResolution,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        validate_hash(activity, expected)?;
        if optional_key(activity, ROOM_MARK_SLOT)?.is_some() {
            return Err(DivergentUniversePermanentProgressionRuntimeError::PreserveCurrentRoomMark);
        }
        let definition = self.room_mark(mark)?;
        if definition.room_type.as_ref() != room_type {
            return Err(DivergentUniversePermanentProgressionRuntimeError::RoomTypeMismatch);
        }
        self.apply(
            activity,
            expected,
            APPLY_ROOM_MARK_PROGRAM,
            vec![ActivityOperation::SetSlot {
                slot: ROOM_MARK_SLOT,
                value: literal(ActivityValue::OptionalId(Some(definition.key))),
            }],
        )
    }

    pub fn reject_missing_service_graph(
        &self,
        activity: &GraphActivity,
        expected: ActivityStateHash,
        service: &DivergentUniverseModeServiceId,
    ) -> Result<(), DivergentUniversePermanentProgressionRuntimeError> {
        validate_hash(activity, expected)?;
        lookup(&self.missing_services, |value| &value.id, service)?;
        Err(DivergentUniversePermanentProgressionRuntimeError::MissingServiceGraph)
    }

    pub fn snapshot(
        &self,
        activity: &GraphActivity,
    ) -> Result<
        DivergentUniversePermanentProgressionSnapshot,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        let keys = permanent_keys(activity)?;
        let talents = self
            .talents
            .iter()
            .filter(|talent| keys.binary_search(&talent.state_key).is_ok())
            .map(|talent| talent.id.clone())
            .collect::<Vec<_>>();
        let finish_unlocks = self
            .unlocks
            .iter()
            .filter(|unlock| keys.binary_search(&unlock.state_key).is_ok())
            .map(|unlock| unlock.id.clone())
            .collect::<Vec<_>>();
        let mut unlocked_areas = self
            .unlocks
            .iter()
            .filter(|unlock| keys.binary_search(&unlock.state_key).is_ok())
            .flat_map(|unlock| unlock.areas.iter().cloned())
            .collect::<Vec<_>>();
        unlocked_areas.sort_unstable();
        unlocked_areas.dedup();
        let weekly_modifier = optional_key(activity, WEEKLY_MODIFIER_SLOT)?
            .map(|key| self.weekly_by_key(key).map(|value| value.id.clone()))
            .transpose()?;
        let room_mark = optional_key(activity, ROOM_MARK_SLOT)?
            .map(|key| self.room_mark_by_key(key).map(|value| value.id.clone()))
            .transpose()?;
        let contributions = talents
            .iter()
            .map(|id| self.talent(id).map(|value| value.contribution.clone()))
            .collect::<Result<Vec<_>, _>>()?;
        let state_hash = activity.state_hash();
        let digest = snapshot_digest(
            state_hash,
            self.component_digest,
            &talents,
            &finish_unlocks,
            &unlocked_areas,
            weekly_modifier.as_ref(),
            room_mark.as_ref(),
            &contributions,
        );
        Ok(DivergentUniversePermanentProgressionSnapshot {
            source_state_hash: state_hash,
            talents: talents.into_boxed_slice(),
            finish_unlocks: finish_unlocks.into_boxed_slice(),
            unlocked_areas: unlocked_areas.into_boxed_slice(),
            weekly_modifier,
            room_mark,
            contributions: contributions.into_boxed_slice(),
            digest: DivergentUniversePermanentProgressionSnapshotDigest(digest),
        })
    }

    fn talent(
        &self,
        id: &DivergentUniversePermanentTalentId,
    ) -> Result<
        &DivergentUniversePermanentTalentRuntime,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        lookup(&self.talents, |value| &value.id, id)
    }
    fn weekly_modifier(
        &self,
        id: &DivergentUniverseWeeklyModifierId,
    ) -> Result<
        &DivergentUniverseWeeklyModifierRuntime,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        lookup(&self.weekly_modifiers, |value| &value.id, id)
    }
    fn room_mark(
        &self,
        id: &DivergentUniverseRoomMarkId,
    ) -> Result<&DivergentUniverseRoomMarkRuntime, DivergentUniversePermanentProgressionRuntimeError>
    {
        lookup(&self.room_marks, |value| &value.id, id)
    }
    fn weekly_by_key(
        &self,
        key: u64,
    ) -> Result<
        &DivergentUniverseWeeklyModifierRuntime,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        self.weekly_modifiers
            .iter()
            .find(|value| value.key == key)
            .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidState)
    }
    fn room_mark_by_key(
        &self,
        key: u64,
    ) -> Result<&DivergentUniverseRoomMarkRuntime, DivergentUniversePermanentProgressionRuntimeError>
    {
        self.room_marks
            .iter()
            .find(|value| value.key == key)
            .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidState)
    }
    fn apply(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        raw_program: u32,
        operations: Vec<ActivityOperation>,
    ) -> Result<
        DivergentUniversePermanentProgressionResolution,
        DivergentUniversePermanentProgressionRuntimeError,
    > {
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(raw_program)
                .expect("static permanent progression program ID is non-zero"),
            operations,
        )
        .map_err(|_| DivergentUniversePermanentProgressionRuntimeError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected, &program)
            .map_err(DivergentUniversePermanentProgressionRuntimeError::Activity)?;
        Ok(DivergentUniversePermanentProgressionResolution {
            events,
            state_hash: activity.state_hash(),
        })
    }
}

fn contribution_scope(
    value: &str,
) -> Result<
    DivergentUniverseProgressionContributionScope,
    DivergentUniversePermanentProgressionRuntimeError,
> {
    match value {
        "Activity" => Ok(DivergentUniverseProgressionContributionScope::Activity),
        "Battle" => Ok(DivergentUniverseProgressionContributionScope::Battle),
        _ => Err(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog),
    }
}

fn lookup<'a, T, I: Ord>(
    values: &'a [T],
    id: impl Fn(&T) -> &I,
    expected: &I,
) -> Result<&'a T, DivergentUniversePermanentProgressionRuntimeError> {
    values
        .binary_search_by(|value| id(value).cmp(expected))
        .ok()
        .and_then(|index| values.get(index))
        .ok_or(DivergentUniversePermanentProgressionRuntimeError::UnknownIdentity)
}

fn permanent_keys(
    activity: &GraphActivity,
) -> Result<Vec<u64>, DivergentUniversePermanentProgressionRuntimeError> {
    let value = slot_value(activity, PERMANENT_UNLOCKS_SLOT)?;
    let ActivityValue::OrderedIdSet(values) = value else {
        return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidState);
    };
    Ok(values.into_vec())
}

fn permanent_currency(
    activity: &GraphActivity,
) -> Result<i64, DivergentUniversePermanentProgressionRuntimeError> {
    let value = slot_value(activity, PERMANENT_TALENT_CURRENCY_SLOT)?;
    let ActivityValue::BoundedInteger(value) = value else {
        return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidState);
    };
    if value < 0 {
        return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidState);
    }
    Ok(value)
}

fn optional_key(
    activity: &GraphActivity,
    slot: starclock_activity::ActivitySlotId,
) -> Result<Option<u64>, DivergentUniversePermanentProgressionRuntimeError> {
    let value = slot_value(activity, slot)?;
    let ActivityValue::OptionalId(value) = value else {
        return Err(DivergentUniversePermanentProgressionRuntimeError::InvalidState);
    };
    Ok(value)
}

fn slot_value(
    activity: &GraphActivity,
    id: starclock_activity::ActivitySlotId,
) -> Result<ActivityValue, DivergentUniversePermanentProgressionRuntimeError> {
    activity
        .player_view()
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .map(|slot| slot.value().clone())
        .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidState)
}

fn validate_hash(
    activity: &GraphActivity,
    expected: ActivityStateHash,
) -> Result<(), DivergentUniversePermanentProgressionRuntimeError> {
    if activity.state_hash() == expected {
        Ok(())
    } else {
        Err(DivergentUniversePermanentProgressionRuntimeError::Activity(
            GraphActivityCommandError::StaleStateHash,
        ))
    }
}

fn positive_u32(value: &str) -> Result<u32, DivergentUniversePermanentProgressionRuntimeError> {
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog)
}

fn ordinal(index: usize) -> Result<u64, DivergentUniversePermanentProgressionRuntimeError> {
    u64::try_from(index + 1)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog)
}

fn reserved_key(
    base: u64,
    index: usize,
) -> Result<u64, DivergentUniversePermanentProgressionRuntimeError> {
    base.checked_add(ordinal(index)?)
        .ok_or(DivergentUniversePermanentProgressionRuntimeError::InvalidCatalog)
}

fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}

#[allow(clippy::too_many_arguments)]
fn snapshot_digest(
    state_hash: ActivityStateHash,
    component_digest: [u8; 32],
    talents: &[DivergentUniversePermanentTalentId],
    unlocks: &[DivergentUniverseUnlockId],
    areas: &[DivergentUniverseAreaId],
    weekly: Option<&DivergentUniverseWeeklyModifierId>,
    room_mark: Option<&DivergentUniverseRoomMarkId>,
    contributions: &[DivergentUniverseProgressionContribution],
) -> [u8; 32] {
    let mut hash = CanonicalDigestBuilder::new();
    field(
        &mut hash,
        b"starclock.divergent-universe.permanent-progression-snapshot.v1",
    );
    field(&mut hash, &component_digest);
    field(&mut hash, &state_hash.bytes());
    for value in talents {
        field(&mut hash, value.as_str().as_bytes());
    }
    for value in unlocks {
        field(&mut hash, value.as_str().as_bytes());
    }
    for value in areas {
        field(&mut hash, value.as_str().as_bytes());
    }
    optional_field(&mut hash, weekly.map(|value| value.as_str()));
    optional_field(&mut hash, room_mark.map(|value| value.as_str()));
    for value in contributions {
        field(&mut hash, value.id.as_str().as_bytes());
        field(&mut hash, &[value.scope as u8]);
        field(&mut hash, value.operation.as_bytes());
        field(&mut hash, value.metric.as_bytes());
        field(&mut hash, value.condition.as_bytes());
        for parameter in &value.parameters {
            field(&mut hash, parameter.as_bytes());
        }
    }
    hash.finalize()
}

fn optional_field(hash: &mut CanonicalDigestBuilder, value: Option<&str>) {
    match value {
        Some(value) => {
            hash.update([1]);
            field(hash, value.as_bytes());
        }
        None => hash.update([0]),
    }
}

fn field(hash: &mut CanonicalDigestBuilder, value: &[u8]) {
    hash.update(
        u64::try_from(value.len())
            .expect("slice length fits u64")
            .to_le_bytes(),
    );
    hash.update(value);
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniversePermanentProgressionRuntimeError {
    InvalidCatalog,
    UnknownIdentity,
    InvalidState,
    InvalidProgram,
    InvalidAmount,
    CurrencyOutOfRange,
    InsufficientTalentCurrency,
    TalentAlreadyUnlocked,
    NotAdjacentTalent,
    UnresolvedPrerequisiteDirection,
    UnlockAlreadyRecorded,
    WeeklyModifierOutsideCyclical,
    WeeklyModifierAlreadyActive,
    RoomTypeMismatch,
    PreserveCurrentRoomMark,
    MissingServiceGraph,
    Activity(GraphActivityCommandError),
}

impl std::fmt::Display for DivergentUniversePermanentProgressionRuntimeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invalid Divergent Universe permanent progression runtime: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniversePermanentProgressionRuntimeError {}
