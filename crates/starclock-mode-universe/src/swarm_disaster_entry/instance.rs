use starclock_activity::{
    ActivityEdgeId, ActivityGraphDefinition, ActivityProgramDefinition, ActivityRngStreams,
    ActivityStateDefinition, ActivityTransactionState, NodeId, ParticipantLock,
};

use crate::error::UniverseCatalogLoadError;

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

    /// Compiles one plane's canonical node-domain and beacon initialization.
    ///
    /// The caller owns `rng`; successful compilation consumes only labeled
    /// Graph-stream draws. An invalid plane is rejected before any draw.
    pub fn compile_plane_creation(
        &self,
        plane_ordinal: usize,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let plane = self
            .planes
            .get(plane_ordinal)
            .ok_or_else(super::map_overlay::invalid_plane)?;
        let section =
            u32::try_from(plane_ordinal + 1).map_err(|_| super::map_overlay::invalid_plane())?;
        let nodes = self
            .graph
            .nodes()
            .iter()
            .filter(|node| node.section().get() == section && node.kind().terminal().is_none())
            .map(|node| node.id())
            .collect::<Vec<_>>();
        self.map.compile_creation(
            &plane.board_key,
            &nodes,
            plane.end,
            super::map_overlay::terminal_domain(plane_ordinal)?,
            rng,
        )
    }

    /// Selects a matching map event and orders its descriptor before creation.
    ///
    /// Empty or invalid event selections return a typed error. An empty
    /// candidate set consumes no random draw.
    pub fn compile_map_event_then_creation(
        &self,
        plane_ordinal: usize,
        trigger: &str,
        parameter: u32,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let plane = self
            .planes
            .get(plane_ordinal)
            .ok_or_else(super::map_overlay::invalid_plane)?;
        let section =
            u32::try_from(plane_ordinal + 1).map_err(|_| super::map_overlay::invalid_plane())?;
        let nodes = self
            .graph
            .nodes()
            .iter()
            .filter(|node| node.section().get() == section && node.kind().terminal().is_none())
            .map(|node| node.id())
            .collect::<Vec<_>>();
        self.map.compile_event_then_creation(
            &plane.board_key,
            trigger,
            parameter,
            &nodes,
            plane.end,
            super::map_overlay::terminal_domain(plane_ordinal)?,
            rng,
        )
    }

    /// Compiles replacement of one immutable-graph node's domain overlay.
    ///
    /// Omitting `beacon` preserves the target node's current beacon value.
    pub fn compile_node_replacement(
        &self,
        target: NodeId,
        domain: &str,
        beacon: Option<&str>,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.require_graph_node(target)?;
        self.map.compile_replacement(target, domain, beacon)
    }

    /// Compiles a source-domain copy while preserving the target beacon.
    pub fn compile_node_copy(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.require_graph_node(source)?;
        self.require_graph_node(target)?;
        self.map.compile_copy(source, target)
    }

    /// Compiles target blanking without changing the immutable graph.
    ///
    /// Blanking clears the domain, preserves the target beacon and causes
    /// [`Self::legal_routes`] to filter routes entering that node.
    pub fn compile_node_blanking(
        &self,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.require_graph_node(target)?;
        self.map.compile_blank(target)
    }

    /// Returns outgoing edges whose target node is not blanked in `state`.
    #[must_use]
    pub fn legal_routes(
        &self,
        state: &ActivityTransactionState,
        source: NodeId,
    ) -> Box<[ActivityEdgeId]> {
        self.graph
            .outgoing(source)
            .filter(|edge| !super::map_overlay::node_is_blanked(state, edge.to()))
            .map(|edge| edge.id())
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn require_graph_node(&self, node: NodeId) -> Result<(), UniverseCatalogLoadError> {
        self.graph
            .node(node)
            .map(|_| ())
            .ok_or_else(super::map_overlay::invalid_node)
    }
}
