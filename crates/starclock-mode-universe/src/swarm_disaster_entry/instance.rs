use starclock_activity::{
    ActivityGraphDefinition, ActivityStateDefinition, NodeId, ParticipantLock,
};

use super::SwarmDisasterRuntimeInstance;

impl SwarmDisasterRuntimeInstance {
    #[must_use]
    pub fn area(&self) -> &str {
        &self.area
    }
    #[must_use]
    pub const fn difficulty(&self) -> u8 {
        self.difficulty
    }
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub fn audience_die(&self) -> &str {
        &self.audience_die
    }
    #[must_use]
    pub fn participants(&self) -> &ParticipantLock {
        &self.participants
    }
    #[must_use]
    pub fn trailblaze_bonus(&self) -> Option<&str> {
        self.trailblaze_bonus.as_deref()
    }
    #[must_use]
    pub const fn state_definition(&self) -> &ActivityStateDefinition {
        &self.state
    }
    #[must_use]
    pub const fn graph_definition(&self) -> &ActivityGraphDefinition {
        &self.graph
    }
    #[must_use]
    pub fn planes(&self) -> impl ExactSizeIterator<Item = &str> {
        self.planes.iter().map(|plane| plane.plane_key.as_ref())
    }
    #[must_use]
    pub fn chessboards(&self) -> impl ExactSizeIterator<Item = &str> {
        self.planes.iter().map(|plane| plane.board_key.as_ref())
    }
    #[must_use]
    pub fn plane_starts(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.planes.iter().map(|plane| plane.start)
    }
    #[must_use]
    pub fn plane_ends(&self) -> impl ExactSizeIterator<Item = NodeId> + '_ {
        self.planes.iter().map(|plane| plane.end)
    }
}
