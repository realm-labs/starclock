use crate::id::{EncounterGroupId, RoomId, TopologyNodeId};
use starclock_activity::GraphActivityDefinitionError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UniverseTopologyCompileError {
    InvalidGraph,
    InvalidProgram,
    InvalidEncounterWeight,
    InvalidBlessingRuntime,
    InvalidOccurrence,
    NoEligibleRoom(TopologyNodeId),
    MissingPrimaryRoomContent(RoomId),
    AmbiguousPrimaryRoomContent(RoomId),
    MissingEncounterGroup(EncounterGroupId),
    RuntimeDefinition(GraphActivityDefinitionError),
    InvalidOccurrenceInteraction,
    InvalidServiceInteraction,
}
