//! Frozen `G22-P6-A01` Activity state-lifecycle mechanic partition.

use std::collections::BTreeSet;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError,
};
use starclock_data::divergent_universe_mechanic_catalog::{
    DivergentUniverseMechanicRuleId, DivergentUniverseMechanicSourceId,
};

use crate::digest::Encoder;

use super::{DivergentUniverseRuntimeFactory, state::ACTIVITY_MECHANIC_LIFECYCLE_SLOT};

const PARTITION: &str = "G22-P6-A01";
const PROGRAM_COUNT: usize = 24;
const PROGRAM_BASE: u32 = 22_600;
const FIXTURE: &str = "divergent-universe.review-fixture.weekly-modifier-and-room-service";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseActivityMechanicBoundary {
    RoomFinishedDoorsUnlocked,
    DoorsUnlocked,
    AdventurePrepared,
    AdventureSettled,
    RoomTransitionObserved,
    NextRoomEntered,
    RunFinished,
    BattleWon,
}

impl DivergentUniverseActivityMechanicBoundary {
    const fn code(self) -> i64 {
        match self {
            Self::RoomFinishedDoorsUnlocked => 1,
            Self::DoorsUnlocked => 2,
            Self::AdventurePrepared => 3,
            Self::AdventureSettled => 4,
            Self::RoomTransitionObserved => 5,
            Self::NextRoomEntered => 6,
            Self::RunFinished => 7,
            Self::BattleWon => 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseActivityMechanicOperation {
    operation_type: Box<str>,
    ordinal: u16,
    source_occurrences: u32,
}

impl DivergentUniverseActivityMechanicOperation {
    #[must_use]
    pub fn operation_type(&self) -> &str {
        &self.operation_type
    }
    #[must_use]
    pub const fn ordinal(&self) -> u16 {
        self.ordinal
    }
    #[must_use]
    pub const fn source_occurrences(&self) -> u32 {
        self.source_occurrences
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseActivityMechanicDefinition {
    id: DivergentUniverseMechanicRuleId,
    source: DivergentUniverseMechanicSourceId,
    source_path: Box<str>,
    source_sha256: Box<str>,
    operations: Box<[DivergentUniverseActivityMechanicOperation]>,
    boundary: DivergentUniverseActivityMechanicBoundary,
    state_key: u64,
    digest: [u8; 32],
}

impl DivergentUniverseActivityMechanicDefinition {
    #[must_use]
    pub const fn id(&self) -> &DivergentUniverseMechanicRuleId {
        &self.id
    }
    #[must_use]
    pub const fn source(&self) -> &DivergentUniverseMechanicSourceId {
        &self.source
    }
    #[must_use]
    pub fn source_path(&self) -> &str {
        &self.source_path
    }
    #[must_use]
    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
    #[must_use]
    pub fn operations(&self) -> &[DivergentUniverseActivityMechanicOperation] {
        &self.operations
    }
    #[must_use]
    pub const fn boundary(&self) -> DivergentUniverseActivityMechanicBoundary {
        self.boundary
    }
    #[must_use]
    pub const fn state_key(&self) -> u64 {
        self.state_key
    }
    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseActivityStateMechanicRuntime {
    definitions: Box<[DivergentUniverseActivityMechanicDefinition]>,
    digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn activity_state_mechanic_runtime(
        &self,
    ) -> Result<DivergentUniverseActivityStateMechanicRuntime, DivergentUniverseActivityMechanicError>
    {
        DivergentUniverseActivityStateMechanicRuntime::compile(self)
    }
}

impl DivergentUniverseActivityStateMechanicRuntime {
    fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseActivityMechanicError> {
        let catalog = factory.bundle.mechanic_catalog();
        let mut definitions = Vec::new();
        let mut keys = BTreeSet::new();
        for rule in catalog
            .rules()
            .iter()
            .filter(|rule| rule.state_lifecycle.as_ref() == "CrossBattleStateLifecycle")
        {
            if rule.scope.as_ref() != "Activity"
                || rule.trigger.as_ref() != "ModeOrRoomLifecycle"
                || rule.fixture_ids.len() != 1
                || rule.fixture_ids[0].as_ref() != FIXTURE
            {
                return Err(DivergentUniverseActivityMechanicError::InvalidCatalog);
            }
            let source = catalog
                .sources()
                .binary_search_by(|source| source.id.cmp(&rule.source))
                .ok()
                .and_then(|index| catalog.sources().get(index))
                .ok_or(DivergentUniverseActivityMechanicError::InvalidCatalog)?;
            if source.source_path.is_empty()
                || source.source_sha256.len() != 64
                || source.scope.as_ref() != "Activity"
                || source.operation_types.len() != rule.ordered_operations.len()
            {
                return Err(DivergentUniverseActivityMechanicError::InvalidCatalog);
            }
            let operations = rule
                .ordered_operations
                .iter()
                .map(|operation| DivergentUniverseActivityMechanicOperation {
                    operation_type: operation.operation_type.clone(),
                    ordinal: operation.ordinal,
                    source_occurrences: operation.source_occurrences,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let boundary = classify(&source.source_path, &operations)?;
            let digest = definition_digest(
                rule.id.as_str(),
                source.id.as_str(),
                &source.source_sha256,
                &operations,
                boundary,
            );
            let state_key = state_key(digest);
            if !keys.insert(state_key) {
                return Err(DivergentUniverseActivityMechanicError::InvalidCatalog);
            }
            definitions.push(DivergentUniverseActivityMechanicDefinition {
                id: rule.id.clone(),
                source: source.id.clone(),
                source_path: source.source_path.clone(),
                source_sha256: source.source_sha256.clone(),
                operations,
                boundary,
                state_key,
                digest,
            });
        }
        if definitions.len() != PROGRAM_COUNT {
            return Err(DivergentUniverseActivityMechanicError::InvalidCatalog);
        }
        definitions.sort_unstable_by(|left, right| left.id.cmp(&right.id));
        let mut encoder = Encoder::new(b"starclock.divergent-universe.activity-state-mechanics.v1");
        encoder.text(PARTITION);
        encoder.u32(
            u32::try_from(definitions.len())
                .map_err(|_| DivergentUniverseActivityMechanicError::InvalidCatalog)?,
        );
        for definition in &definitions {
            encoder.digest(definition.digest);
        }
        Ok(Self {
            definitions: definitions.into_boxed_slice(),
            digest: encoder.finish(),
        })
    }

    #[must_use]
    pub fn definitions(&self) -> &[DivergentUniverseActivityMechanicDefinition] {
        &self.definitions
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    pub fn execute(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        id: &DivergentUniverseMechanicRuleId,
        boundary: DivergentUniverseActivityMechanicBoundary,
    ) -> Result<DivergentUniverseActivityMechanicResolution, DivergentUniverseActivityMechanicError>
    {
        let (index, definition) = self
            .definitions
            .binary_search_by(|definition| definition.id.cmp(id))
            .ok()
            .and_then(|index| self.definitions.get(index).map(|value| (index, value)))
            .ok_or(DivergentUniverseActivityMechanicError::UnknownMechanic)?;
        if definition.boundary != boundary {
            return Err(DivergentUniverseActivityMechanicError::BoundaryMismatch);
        }
        if lifecycle_value(activity, definition.state_key)?.is_some() {
            return Err(DivergentUniverseActivityMechanicError::AlreadyExecuted);
        }
        let raw_program = PROGRAM_BASE
            .checked_add(
                u32::try_from(index)
                    .map_err(|_| DivergentUniverseActivityMechanicError::InvalidProgram)?,
            )
            .ok_or(DivergentUniverseActivityMechanicError::InvalidProgram)?;
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(raw_program)
                .ok_or(DivergentUniverseActivityMechanicError::InvalidProgram)?,
            vec![ActivityOperation::SetCounter {
                slot: ACTIVITY_MECHANIC_LIFECYCLE_SLOT,
                key: definition.state_key,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(boundary.code())),
            }],
        )
        .map_err(|_| DivergentUniverseActivityMechanicError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected, &program)
            .map_err(DivergentUniverseActivityMechanicError::Activity)?;
        Ok(DivergentUniverseActivityMechanicResolution {
            mechanic_digest: definition.digest,
            boundary,
            events,
            state_hash: activity.state_hash(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseActivityMechanicResolution {
    mechanic_digest: [u8; 32],
    boundary: DivergentUniverseActivityMechanicBoundary,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseActivityMechanicResolution {
    #[must_use]
    pub const fn mechanic_digest(&self) -> [u8; 32] {
        self.mechanic_digest
    }
    #[must_use]
    pub const fn boundary(&self) -> DivergentUniverseActivityMechanicBoundary {
        self.boundary
    }
    #[must_use]
    pub fn events(&self) -> &[ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }
}

fn classify(
    source_path: &str,
    operations: &[DivergentUniverseActivityMechanicOperation],
) -> Result<DivergentUniverseActivityMechanicBoundary, DivergentUniverseActivityMechanicError> {
    let has = |expected: &str| {
        operations
            .iter()
            .any(|operation| operation.operation_type() == expected)
    };
    if has("RPG.GameCore.SetRogueRoomFinish") {
        Ok(DivergentUniverseActivityMechanicBoundary::RoomFinishedDoorsUnlocked)
    } else if has("RPG.GameCore.RogueTournFinish") {
        Ok(DivergentUniverseActivityMechanicBoundary::RunFinished)
    } else if has("RPG.GameCore.RogueTournEnterNextRoom") {
        Ok(DivergentUniverseActivityMechanicBoundary::NextRoomEntered)
    } else if has("RPG.GameCore.WaitBattleWinWithScore")
        || has("RPG.GameCore.WaitBattleWinWithTargetCount")
    {
        Ok(DivergentUniverseActivityMechanicBoundary::BattleWon)
    } else if has("RPG.GameCore.RogueDLC1Dot3AdventureRoomPrepare") {
        Ok(DivergentUniverseActivityMechanicBoundary::AdventurePrepared)
    } else if has("RPG.GameCore.RogueDLC1Dot3AdventureRoomProcess")
        || has("RPG.GameCore.WaitRogueAdventureRoomStart")
        || source_path.contains("AdvWolfGun")
    {
        Ok(DivergentUniverseActivityMechanicBoundary::AdventureSettled)
    } else if has("RPG.GameCore.ByIsRogueTournCurRoomFinish") {
        Ok(DivergentUniverseActivityMechanicBoundary::RoomTransitionObserved)
    } else if has("RPG.GameCore.SetAllRogueDoorState") {
        Ok(DivergentUniverseActivityMechanicBoundary::DoorsUnlocked)
    } else {
        Err(DivergentUniverseActivityMechanicError::UnsupportedOperationShape)
    }
}

fn definition_digest(
    id: &str,
    source: &str,
    source_sha256: &str,
    operations: &[DivergentUniverseActivityMechanicOperation],
    boundary: DivergentUniverseActivityMechanicBoundary,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.divergent-universe.activity-state-mechanic.v1");
    encoder.text(id);
    encoder.text(source);
    encoder.text(source_sha256);
    encoder.u32(u32::try_from(operations.len()).expect("bounded operation shapes fit u32"));
    for operation in operations {
        encoder.text(operation.operation_type());
        encoder.u32(u32::from(operation.ordinal));
        encoder.u32(operation.source_occurrences);
    }
    encoder.u8(u8::try_from(boundary.code()).expect("closed boundary code fits u8"));
    encoder.finish()
}

fn state_key(digest: [u8; 32]) -> u64 {
    let mut bytes = [0_u8; 8];
    bytes.copy_from_slice(&digest[..8]);
    u64::from_le_bytes(bytes) | (1_u64 << 63)
}

fn lifecycle_value(
    activity: &GraphActivity,
    key: u64,
) -> Result<Option<i64>, DivergentUniverseActivityMechanicError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == ACTIVITY_MECHANIC_LIFECYCLE_SLOT)
        .map(|slot| slot.value())
        .ok_or(DivergentUniverseActivityMechanicError::InvalidState)?;
    let ActivityValue::BoundedCounterMap(values) = value else {
        return Err(DivergentUniverseActivityMechanicError::InvalidState);
    };
    Ok(values
        .binary_search_by_key(&key, |(entry, _)| *entry)
        .ok()
        .map(|index| values[index].1))
}

#[derive(Debug)]
pub enum DivergentUniverseActivityMechanicError {
    InvalidCatalog,
    UnknownMechanic,
    UnsupportedOperationShape,
    BoundaryMismatch,
    AlreadyExecuted,
    InvalidState,
    InvalidProgram,
    Activity(GraphActivityCommandError),
}

impl core::fmt::Display for DivergentUniverseActivityMechanicError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Activity mechanic error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseActivityMechanicError {}
