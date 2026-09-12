//! Frozen `G22-P6-A02` Activity decision-lifecycle mechanic partition.

use std::collections::BTreeSet;

use starclock_activity::{
    ActivityExpression, ActivityOperation, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityTransactionEvent, ActivityValue, GraphActivity,
    GraphActivityCommandError, NodeId,
};
use starclock_data::divergent_universe_mechanic_catalog::{
    DivergentUniverseMechanicRuleId, DivergentUniverseMechanicSourceId,
};

use crate::digest::Encoder;

use super::room_lifecycle::{self, RoomDialogueSignal, RoomLifecycleError};
use super::{DivergentUniverseRuntimeFactory, state::ACTIVITY_MECHANIC_LIFECYCLE_SLOT};

const DECISION_PROGRAM_COUNT: usize = 638;
const PROGRAM_BASE: u32 = 22_700;
const STATE_KEY_BASE: u64 = 1_u64 << 62;
const FIXTURE: &str = "divergent-universe.review-fixture.occurrence-choice-cost-and-outcome";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseActivityDecisionPartition {
    A02,
    A03,
    A04,
    A05,
    A06,
    A07,
    A08,
    A09,
    A10,
    A11,
}

impl DivergentUniverseActivityDecisionPartition {
    const fn spec(self) -> PartitionSpec {
        match self {
            Self::A02 => PartitionSpec {
                batch: "G22-P6-A02",
                offset: 0,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-maze-mazerogue-roguetourn-roguetourn-goup-waitdialogue-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0412801-act0412801-json",
            },
            Self::A03 => PartitionSpec {
                batch: "G22-P6-A03",
                offset: 64,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0412801-opt0412801-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0419504-act0419504-json",
            },
            Self::A04 => PartitionSpec {
                batch: "G22-P6-A04",
                offset: 128,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn1-event0419504-opt0419504-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0610901-act0610901-json",
            },
            Self::A05 => PartitionSpec {
                batch: "G22-P6-A05",
                offset: 192,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0610901-opt0610901-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0613801-act0613801-json",
            },
            Self::A06 => PartitionSpec {
                batch: "G22-P6-A06",
                offset: 256,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0613801-opt0613801-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0620701-act0620701-json",
            },
            Self::A07 => PartitionSpec {
                batch: "G22-P6-A07",
                offset: 320,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0620701-opt0620701-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0623001-act0623001-json",
            },
            Self::A08 => PartitionSpec {
                batch: "G22-P6-A08",
                offset: 384,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0623001-opt0623001-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0625902-act0625902-json",
            },
            Self::A09 => PartitionSpec {
                batch: "G22-P6-A09",
                offset: 448,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguedialogue-rogueeventtourn2-event0625902-opt0625902-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-230-roguenpc413301-json",
            },
            Self::A10 => PartitionSpec {
                batch: "G22-P6-A10",
                offset: 512,
                count: 64,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-230-roguenpc413401-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-310-roguenpc614301-json",
            },
            Self::A11 => PartitionSpec {
                batch: "G22-P6-A11",
                offset: 576,
                count: 62,
                first: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-310-roguenpc614401-json",
                last: "divergent-universe.mechanic-rule.config-level-rogue-roguenpc-roguenpc-330-roguenpc627001-json",
            },
        }
    }
}

#[derive(Clone, Copy)]
struct PartitionSpec {
    batch: &'static str,
    offset: usize,
    count: usize,
    first: &'static str,
    last: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DivergentUniverseActivityDecisionBoundary {
    OptionSelected,
    DialogueCompleted,
    RoomDialogueCompleted,
}

impl DivergentUniverseActivityDecisionBoundary {
    const fn code(self) -> i64 {
        match self {
            Self::OptionSelected => 1,
            Self::DialogueCompleted => 2,
            Self::RoomDialogueCompleted => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DivergentUniverseActivityDecisionOperation {
    operation_type: Box<str>,
    ordinal: u16,
    source_occurrences: u32,
}

impl DivergentUniverseActivityDecisionOperation {
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
pub struct DivergentUniverseActivityDecisionDefinition {
    id: DivergentUniverseMechanicRuleId,
    source: DivergentUniverseMechanicSourceId,
    source_path: Box<str>,
    source_sha256: Box<str>,
    operations: Box<[DivergentUniverseActivityDecisionOperation]>,
    boundary: DivergentUniverseActivityDecisionBoundary,
    state_key: u64,
    digest: [u8; 32],
}

impl DivergentUniverseActivityDecisionDefinition {
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
    pub fn operations(&self) -> &[DivergentUniverseActivityDecisionOperation] {
        &self.operations
    }

    #[must_use]
    pub const fn boundary(&self) -> DivergentUniverseActivityDecisionBoundary {
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
pub struct DivergentUniverseActivityDecisionMechanicRuntime {
    partition: DivergentUniverseActivityDecisionPartition,
    definitions: Box<[DivergentUniverseActivityDecisionDefinition]>,
    digest: [u8; 32],
}

impl DivergentUniverseRuntimeFactory {
    pub fn activity_decision_mechanic_runtime(
        &self,
        partition: DivergentUniverseActivityDecisionPartition,
    ) -> Result<
        DivergentUniverseActivityDecisionMechanicRuntime,
        DivergentUniverseActivityDecisionError,
    > {
        DivergentUniverseActivityDecisionMechanicRuntime::compile(self, partition)
    }
}

impl DivergentUniverseActivityDecisionMechanicRuntime {
    fn compile(
        factory: &DivergentUniverseRuntimeFactory,
        partition: DivergentUniverseActivityDecisionPartition,
    ) -> Result<Self, DivergentUniverseActivityDecisionError> {
        let spec = partition.spec();
        let catalog = factory.bundle.mechanic_catalog();
        let decision_rules = catalog
            .rules()
            .iter()
            .filter(|rule| rule.state_lifecycle.as_ref() == "CrossBattleDecisionLifecycle")
            .collect::<Vec<_>>();
        if decision_rules.len() != DECISION_PROGRAM_COUNT {
            return Err(DivergentUniverseActivityDecisionError::InvalidCatalog);
        }
        let mut definitions = Vec::with_capacity(spec.count);
        let mut keys = BTreeSet::new();
        for (partition_index, rule) in decision_rules
            .into_iter()
            .skip(spec.offset)
            .take(spec.count)
            .enumerate()
        {
            if rule.scope.as_ref() != "Activity"
                || rule.trigger.as_ref() != "AcceptedModeDecision"
                || rule.fixture_ids.len() != 1
                || rule.fixture_ids[0].as_ref() != FIXTURE
            {
                return Err(DivergentUniverseActivityDecisionError::InvalidCatalog);
            }
            let source = catalog
                .sources()
                .binary_search_by(|source| source.id.cmp(&rule.source))
                .ok()
                .and_then(|source_index| catalog.sources().get(source_index))
                .ok_or(DivergentUniverseActivityDecisionError::InvalidCatalog)?;
            if source.source_path.is_empty()
                || source.source_sha256.len() != 64
                || source.scope.as_ref() != "Activity"
                || source.operation_types.len() != rule.ordered_operations.len()
            {
                return Err(DivergentUniverseActivityDecisionError::InvalidCatalog);
            }
            let operations = rule
                .ordered_operations
                .iter()
                .map(|operation| DivergentUniverseActivityDecisionOperation {
                    operation_type: operation.operation_type.clone(),
                    ordinal: operation.ordinal,
                    source_occurrences: operation.source_occurrences,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice();
            let boundary = classify(&operations)?;
            let index = spec
                .offset
                .checked_add(partition_index)
                .ok_or(DivergentUniverseActivityDecisionError::InvalidCatalog)?;
            let state_key = STATE_KEY_BASE
                .checked_add(
                    u64::try_from(index)
                        .map_err(|_| DivergentUniverseActivityDecisionError::InvalidCatalog)?,
                )
                .ok_or(DivergentUniverseActivityDecisionError::InvalidCatalog)?;
            if !keys.insert(state_key) {
                return Err(DivergentUniverseActivityDecisionError::InvalidCatalog);
            }
            let digest = definition_digest(
                rule.id.as_str(),
                source.id.as_str(),
                &source.source_sha256,
                &operations,
                boundary,
                state_key,
            );
            definitions.push(DivergentUniverseActivityDecisionDefinition {
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
        if definitions.len() != spec.count
            || definitions.first().map(|value| value.id.as_str()) != Some(spec.first)
            || definitions.last().map(|value| value.id.as_str()) != Some(spec.last)
        {
            return Err(DivergentUniverseActivityDecisionError::PartitionDrift);
        }
        let mut encoder =
            Encoder::new(b"starclock.divergent-universe.activity-decision-mechanics.v1");
        encoder.text(spec.batch);
        encoder.u32(
            u32::try_from(definitions.len())
                .map_err(|_| DivergentUniverseActivityDecisionError::InvalidCatalog)?,
        );
        for definition in &definitions {
            encoder.digest(definition.digest);
        }
        Ok(Self {
            partition,
            definitions: definitions.into_boxed_slice(),
            digest: encoder.finish(),
        })
    }

    #[must_use]
    pub const fn partition(&self) -> DivergentUniverseActivityDecisionPartition {
        self.partition
    }

    #[must_use]
    pub fn definitions(&self) -> &[DivergentUniverseActivityDecisionDefinition] {
        &self.definitions
    }

    #[must_use]
    pub const fn digest(&self) -> [u8; 32] {
        self.digest
    }

    /// Binds the current logical room to this released completion program.
    /// The owning content executor must call this before offering/playing its
    /// dialogue. It does not select a map candidate or simulate the dialogue.
    /// Rejected, stale and wrong-node calls do not mutate Activity state.
    pub fn begin_room_dialogue(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        node: NodeId,
        id: &DivergentUniverseMechanicRuleId,
    ) -> Result<Box<[ActivityTransactionEvent]>, DivergentUniverseActivityDecisionError> {
        let definition = self.room_definition(id)?;
        room_lifecycle::begin(activity, expected, node, definition.state_key).map_err(room_error)
    }

    /// Records one node-bound notification after the content executor commits
    /// the corresponding work. Duplicate notifications and late notifications
    /// from another room are rejected without consuming RNG or changing state.
    pub fn record_room_dialogue_signal(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        node: NodeId,
        id: &DivergentUniverseMechanicRuleId,
        signal: RoomDialogueSignal,
    ) -> Result<Box<[ActivityTransactionEvent]>, DivergentUniverseActivityDecisionError> {
        let definition = self.room_definition(id)?;
        room_lifecycle::signal(activity, expected, node, definition.state_key, signal)
            .map_err(room_error)
    }

    fn room_definition(
        &self,
        id: &DivergentUniverseMechanicRuleId,
    ) -> Result<&DivergentUniverseActivityDecisionDefinition, DivergentUniverseActivityDecisionError>
    {
        let definition = self
            .definitions
            .iter()
            .find(|definition| &definition.id == id)
            .ok_or(DivergentUniverseActivityDecisionError::UnknownMechanic)?;
        let expected = [
            "RPG.GameCore.WaitRogueFinishDialogue",
            "RPG.GameCore.WaitPredicateSucc",
            "RPG.GameCore.SetRogueRoomFinish",
            "RPG.GameCore.SetAllRogueDoorState",
            "RPG.GameCore.WaitRogueRoomContentUpdateFinish",
        ];
        if definition.boundary != DivergentUniverseActivityDecisionBoundary::RoomDialogueCompleted
            || definition.operations.len() != expected.len()
            || expected.iter().any(|name| {
                definition
                    .operations
                    .iter()
                    .filter(|operation| {
                        operation.operation_type() == *name && operation.source_occurrences == 1
                    })
                    .count()
                    != 1
            })
        {
            return Err(DivergentUniverseActivityDecisionError::UnsupportedOperationShape);
        }
        // Shapes omit operands and sequence topology. This lowering is reviewed
        // only for the exact released source; a matching type set cannot admit
        // a changed predicate, door default or task order silently.
        if definition.source_path()
            != "Config/Level/Maze/MazeRogue/RogueTourn/RogueTourn_Goup_WaitDialogue.json"
            || definition.source_sha256()
                != "2bddbca2731e612f24db7247ab91ccb07c2f682fa9307cae1ad7104261a373d6"
        {
            return Err(DivergentUniverseActivityDecisionError::UnreviewedRoomSource);
        }
        Ok(definition)
    }

    pub fn execute(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        id: &DivergentUniverseMechanicRuleId,
        boundary: DivergentUniverseActivityDecisionBoundary,
    ) -> Result<DivergentUniverseActivityDecisionResolution, DivergentUniverseActivityDecisionError>
    {
        let (index, definition) = self
            .definitions
            .binary_search_by(|definition| definition.id.cmp(id))
            .ok()
            .and_then(|index| self.definitions.get(index).map(|value| (index, value)))
            .ok_or(DivergentUniverseActivityDecisionError::UnknownMechanic)?;
        if definition.boundary != boundary {
            return Err(DivergentUniverseActivityDecisionError::BoundaryMismatch);
        }
        if boundary == DivergentUniverseActivityDecisionBoundary::RoomDialogueCompleted {
            self.room_definition(id)?;
            let events = room_lifecycle::complete(activity, expected, definition.state_key)
                .map_err(room_error)?;
            return Ok(DivergentUniverseActivityDecisionResolution {
                mechanic_digest: definition.digest,
                boundary,
                events,
                state_hash: activity.state_hash(),
            });
        }
        if lifecycle_value(activity, definition.state_key)?.is_some() {
            return Err(DivergentUniverseActivityDecisionError::AlreadyExecuted);
        }
        let raw_program = PROGRAM_BASE
            .checked_add(
                u32::try_from(
                    self.partition
                        .spec()
                        .offset
                        .checked_add(index)
                        .ok_or(DivergentUniverseActivityDecisionError::InvalidProgram)?,
                )
                .map_err(|_| DivergentUniverseActivityDecisionError::InvalidProgram)?,
            )
            .ok_or(DivergentUniverseActivityDecisionError::InvalidProgram)?;
        let program = ActivityProgramDefinition::new(
            ActivityProgramId::new(raw_program)
                .ok_or(DivergentUniverseActivityDecisionError::InvalidProgram)?,
            vec![ActivityOperation::SetCounter {
                slot: ACTIVITY_MECHANIC_LIFECYCLE_SLOT,
                key: definition.state_key,
                value: ActivityExpression::Literal(ActivityValue::BoundedInteger(boundary.code())),
            }],
        )
        .map_err(|_| DivergentUniverseActivityDecisionError::InvalidProgram)?;
        let events = activity
            .apply_boundary_program(expected, &program)
            .map_err(DivergentUniverseActivityDecisionError::Activity)?;
        Ok(DivergentUniverseActivityDecisionResolution {
            mechanic_digest: definition.digest,
            boundary,
            events,
            state_hash: activity.state_hash(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct DivergentUniverseActivityDecisionResolution {
    mechanic_digest: [u8; 32],
    boundary: DivergentUniverseActivityDecisionBoundary,
    events: Box<[ActivityTransactionEvent]>,
    state_hash: ActivityStateHash,
}

impl DivergentUniverseActivityDecisionResolution {
    #[must_use]
    pub const fn mechanic_digest(&self) -> [u8; 32] {
        self.mechanic_digest
    }

    #[must_use]
    pub const fn boundary(&self) -> DivergentUniverseActivityDecisionBoundary {
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
    operations: &[DivergentUniverseActivityDecisionOperation],
) -> Result<DivergentUniverseActivityDecisionBoundary, DivergentUniverseActivityDecisionError> {
    let has = |expected: &str| {
        operations
            .iter()
            .any(|operation| operation.operation_type() == expected)
    };
    if has("RPG.GameCore.SetRogueRoomFinish") {
        Ok(DivergentUniverseActivityDecisionBoundary::RoomDialogueCompleted)
    } else if operations.len() == 1 && has("Structure:OptionList") {
        Ok(DivergentUniverseActivityDecisionBoundary::OptionSelected)
    } else if has("RPG.GameCore.FinishLevelGraph")
        || (operations.len() == 2 && has("Structure:DialogueList") && has("Structure:DialogueType"))
    {
        Ok(DivergentUniverseActivityDecisionBoundary::DialogueCompleted)
    } else {
        Err(DivergentUniverseActivityDecisionError::UnsupportedOperationShape)
    }
}

fn definition_digest(
    id: &str,
    source: &str,
    source_sha256: &str,
    operations: &[DivergentUniverseActivityDecisionOperation],
    boundary: DivergentUniverseActivityDecisionBoundary,
    state_key: u64,
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.divergent-universe.activity-decision-mechanic.v1");
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
    encoder.u64(state_key);
    encoder.finish()
}

fn lifecycle_value(
    activity: &GraphActivity,
    key: u64,
) -> Result<Option<i64>, DivergentUniverseActivityDecisionError> {
    let view = activity.player_view();
    let value = view
        .slots()
        .iter()
        .find(|slot| slot.id() == ACTIVITY_MECHANIC_LIFECYCLE_SLOT)
        .map(|slot| slot.value())
        .ok_or(DivergentUniverseActivityDecisionError::InvalidState)?;
    let ActivityValue::BoundedCounterMap(values) = value else {
        return Err(DivergentUniverseActivityDecisionError::InvalidState);
    };
    Ok(values
        .binary_search_by_key(&key, |(entry, _)| *entry)
        .ok()
        .map(|index| values[index].1))
}

#[derive(Debug)]
pub enum DivergentUniverseActivityDecisionError {
    InvalidCatalog,
    PartitionDrift,
    UnknownMechanic,
    UnsupportedOperationShape,
    UnreviewedRoomSource,
    BoundaryMismatch,
    AlreadyExecuted,
    InvalidState,
    InvalidProgram,
    Activity(GraphActivityCommandError),
    RoomLifecycle(RoomLifecycleError),
}

fn room_error(error: RoomLifecycleError) -> DivergentUniverseActivityDecisionError {
    match error {
        RoomLifecycleError::AlreadyCompleted => {
            DivergentUniverseActivityDecisionError::AlreadyExecuted
        }
        RoomLifecycleError::Activity(error) => {
            DivergentUniverseActivityDecisionError::Activity(error)
        }
        error => DivergentUniverseActivityDecisionError::RoomLifecycle(error),
    }
}

impl core::fmt::Display for DivergentUniverseActivityDecisionError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "Divergent Universe Activity decision mechanic error: {self:?}"
        )
    }
}

impl std::error::Error for DivergentUniverseActivityDecisionError {}

#[cfg(test)]
mod source_binding_tests {
    use crate::divergent_universe::{
        DivergentUniverseActivityDecisionError, DivergentUniverseActivityDecisionPartition,
        DivergentUniverseRuntimeFactory,
    };

    #[test]
    fn matching_shapes_cannot_admit_an_unreviewed_room_source() {
        let factory = DivergentUniverseRuntimeFactory::production().expect("production catalog");
        let mut runtime = factory
            .activity_decision_mechanic_runtime(DivergentUniverseActivityDecisionPartition::A02)
            .expect("decision catalog");
        let id = runtime.definitions[0].id.clone();
        runtime.room_definition(&id).expect("reviewed room source");
        runtime.definitions[0].source_sha256 = "0".repeat(64).into_boxed_str();
        assert!(matches!(
            runtime.room_definition(&id),
            Err(DivergentUniverseActivityDecisionError::UnreviewedRoomSource)
        ));
    }
}
