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

    /// Returns the selected Path's authored Audience ordering position.
    #[must_use]
    pub fn audience_path_sort(&self) -> u16 {
        self.audience.path_sort()
    }

    /// Returns the authored unlock ID, or `None` for the always-available Path.
    #[must_use]
    pub fn audience_path_unlock_id(&self) -> Option<&str> {
        self.audience.unlock_id()
    }

    /// Whether selecting this Audience Path requires its authored unlock ID.
    #[must_use]
    pub fn audience_path_requires_unlock(&self) -> bool {
        self.audience.requires_unlock()
    }

    /// Returns selected Die faces in authored Sort then stable-ID order.
    #[must_use]
    pub fn audience_die_faces(&self) -> impl ExactSizeIterator<Item = &str> {
        self.audience.faces()
    }

    /// Returns the selected Path's run-start effect operation.
    #[must_use]
    pub fn audience_initial_rule(&self) -> &str {
        self.audience.initial_rule()
    }

    /// Returns primary run-start parameters in authored slot order.
    #[must_use]
    pub fn audience_initial_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.audience.initial_parameters()
    }

    /// Returns secondary run-start parameters in authored slot order.
    #[must_use]
    pub fn audience_initial_secondary_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.audience.initial_secondary_parameters()
    }

    /// Returns the persistent selected-Path graph-rule operation.
    #[must_use]
    pub fn audience_passive_rule(&self) -> &str {
        self.audience.passive_rule()
    }

    /// Returns primary persistent graph-rule parameters in authored order.
    #[must_use]
    pub fn audience_passive_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.audience.passive_parameters()
    }

    /// Returns secondary persistent graph-rule parameters in authored order.
    #[must_use]
    pub fn audience_passive_secondary_parameters(&self) -> impl ExactSizeIterator<Item = &str> {
        self.audience.passive_secondary_parameters()
    }

    /// Compiles one-time run-start Maze Buff and persistent passive activation.
    ///
    /// This program consumes no RNG and records the selected typed graph rules
    /// in existing Activity-owned slots. Later rule execution remains owned by
    /// the frozen mechanic partition rather than by a second state machine.
    pub fn compile_audience_initialization(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.audience.compile_initialization(state)
    }

    /// Whether the one-time Audience initialization has committed exactly once.
    pub fn audience_initialization_applied(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.audience.initialization_applied(state)
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

    /// Compiles one accepted move followed by stable-ID-ordered adjustments.
    ///
    /// Movement always consumes the catalog delta first. A pre-move Countdown
    /// of zero enters Planar Disarray; an active level advances and modifiers
    /// retain the level-20 cap. The program rejects stale state atomically.
    pub fn compile_countdown_move(
        &self,
        state: &ActivityTransactionState,
        adjustments: &[(u32, i64)],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.countdown.compile_move(state, adjustments)
    }

    /// Compiles non-movement Countdown changes in stable operation-ID order.
    ///
    /// Duplicate IDs, zero deltas and values outside the declared slot bounds
    /// are rejected before an Activity program is returned.
    pub fn compile_countdown_adjustments(
        &self,
        state: &ActivityTransactionState,
        adjustments: &[(u32, i64)],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.countdown.compile_adjustments(state, adjustments)
    }

    /// Compiles one not-yet-selected, released Swarm boss-decay contribution.
    ///
    /// Contributions are addressed by stable Starclock key. Unproven shared
    /// DLC rows and a second contribution for the same plane fail closed.
    pub fn compile_boss_decay_selection(
        &self,
        state: &ActivityTransactionState,
        keys: &[&str],
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.countdown.compile_boss_decay_selection(state, keys)
    }

    /// Returns the two released displayed boss choices in stable source order.
    #[must_use]
    pub fn boss_choices(&self) -> impl ExactSizeIterator<Item = &str> {
        self.transitions.choices()
    }

    /// Returns the explicitly selected boss for `plane_layer`, when present.
    #[must_use]
    pub fn selected_boss(&self, state: &ActivityTransactionState, plane_layer: u8) -> Option<&str> {
        self.transitions.selected_boss(state, plane_layer)
    }

    /// Compiles caller-explicit displayed-boss selection for one plane.
    ///
    /// Selection consumes no RNG. Unknown bosses and layers outside `1..=3`
    /// return typed catalog errors before an Activity program is produced.
    pub fn compile_boss_selection(
        &self,
        plane_layer: u8,
        boss: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.transitions.compile_selection(plane_layer, boss)
    }

    /// Compiles atomic post-boss traversal or final Activity completion.
    ///
    /// The caller's state must be at the selected plane's end and contain an
    /// explicit boss choice plus the required released Boss Decay threshold.
    /// Countdown and Disarray carry exactly across section transitions while
    /// section-owned overlays reset through the generic Activity lifecycle.
    pub fn compile_plane_completion(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.transitions.compile_completion(
            &self.countdown,
            state,
            &self.graph,
            &self.planes,
            plane_layer,
        )
    }

    /// Returns the current authoritative Countdown.
    pub fn countdown(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        self.countdown.countdown(state)
    }

    /// Returns the current uncapped Planar Disarray level.
    pub fn disarray_level(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        self.countdown.disarray_level(state)
    }

    /// Returns cumulative enemy damage, mitigation and Speed percentages.
    ///
    /// The tuple is `(damage dealt, damage received reduction, speed)` and is
    /// capped at the level-20 contribution schedule.
    pub fn disarray_modifiers(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<(i64, i64, i64), UniverseCatalogLoadError> {
        self.countdown.disarray_modifiers(state)
    }

    /// Whether the current Countdown is at or below the catalog warning value.
    pub fn countdown_warning_active(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.countdown.warning_active(state)
    }

    fn require_graph_node(&self, node: NodeId) -> Result<(), UniverseCatalogLoadError> {
        self.graph
            .node(node)
            .map(|_| ())
            .ok_or_else(super::map_overlay::invalid_node)
    }
}
