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
        self.audience_rules.compile_initialization(self, state)
    }

    /// Whether the one-time Audience initialization has committed exactly once.
    pub fn audience_initialization_applied(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.audience_rules.initialization_applied(self, state)
    }

    /// Whether an initial authored roll is currently available.
    pub fn dice_roll_available(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.audience_rules.roll_available(self, state)
    }

    /// Whether a charged authored reroll is currently available.
    pub fn dice_reroll_available(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.audience_rules.reroll_available(self, state)
    }

    /// Whether an exact-face charged cheat is currently available.
    pub fn dice_cheat_available(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.audience_rules.cheat_available(self, state)
    }

    /// Whether the unlocked authored abandon control is currently available.
    pub fn dice_abandon_available(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.audience_rules.abandon_available(self, state)
    }

    /// Rolls the selected Die in authored Sort then stable-face-ID order.
    ///
    /// Exactly one Spawn-stream draw is consumed. An empty candidate set or
    /// unavailable lifecycle state rejects without consuming RNG.
    pub fn compile_dice_roll(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.audience_rules.compile_roll(self, state, rng)
    }

    /// Consumes one typed reroll charge and draws from the same authored faces.
    pub fn compile_dice_reroll(
        &self,
        state: &ActivityTransactionState,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.audience_rules.compile_reroll(self, state, rng)
    }

    /// Consumes one typed cheat charge and selects an exact authored face.
    /// Cheats consume no RNG.
    pub fn compile_dice_cheat(
        &self,
        state: &ActivityTransactionState,
        selected_face: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.audience_rules
            .compile_cheat(self, state, selected_face)
    }

    /// Abandons the current face, grants the authored reward and closes this
    /// Attempt's dice phase. The optional authored control must be unlocked.
    pub fn compile_dice_abandon(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.audience_rules.compile_abandon(self, state)
    }

    /// Returns the currently selected authored face, when one exists.
    #[must_use]
    pub fn dice_resolution_face<'a>(&'a self, state: &ActivityTransactionState) -> Option<&'a str> {
        self.audience_rules.resolution_face(self, state)
    }

    /// Returns the stable resolution code: roll 1, reroll 2, cheat 3, abandon 4.
    pub fn dice_resolution_kind(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<Option<u8>, UniverseCatalogLoadError> {
        self.audience_rules.resolution_kind(self, state)
    }

    /// Activates the currently selected face through its typed target policy.
    ///
    /// Explicit selectors accept one eligible node. Random selectors consume
    /// only the labeled Spawn target stream. An empty legal set commits the
    /// deterministic no-op result without consuming RNG.
    pub fn compile_dice_face_activation(
        &self,
        state: &ActivityTransactionState,
        explicit_target: Option<NodeId>,
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.audience_rules
            .compile_face_activation(self, state, explicit_target, rng)
    }

    /// Returns the authored activation stage: immediate 1, post-movement 2,
    /// or next-battle contribution 3.
    #[must_use]
    pub fn dice_face_activation_stage(&self, face: &str) -> Option<u8> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::activation_stage)
    }

    /// Returns the target contract for one selected-Die face.
    #[must_use]
    pub fn dice_face_target_contract(&self, face: &str) -> Option<&'static str> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::target_contract)
    }

    /// Returns the typed selector name for one selected-Die face.
    #[must_use]
    pub fn dice_face_selector(&self, face: &str) -> Option<&'static str> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::selector_name)
    }

    /// Returns the typed duration name for one selected-Die face.
    #[must_use]
    pub fn dice_face_duration(&self, face: &str) -> Option<&'static str> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::duration_name)
    }

    /// Returns the authored operation name for one selected-Die face.
    #[must_use]
    pub fn dice_face_operation(&self, face: &str) -> Option<&'static str> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::operation_name)
    }

    /// Returns exact canonical parameters scaled by one million.
    #[must_use]
    pub fn dice_face_parameters_scaled(&self, face: &str) -> Option<&[i64]> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::parameters_scaled)
    }

    /// Returns exact description parameters scaled by one million.
    #[must_use]
    pub fn dice_face_description_scaled(&self, face: &str) -> Option<&[i64]> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::description_scaled)
    }

    /// Returns a finite next-battle turn duration when one is authored.
    #[must_use]
    pub fn dice_face_turn_duration(&self, face: &str) -> Option<u16> {
        self.dice_face_runtime(face)
            .and_then(super::face_effect::RuntimeDiceFace::turn_duration)
    }

    /// Returns released source-effect references in authored order.
    #[must_use]
    pub fn dice_face_effect_references(&self, face: &str) -> Option<&[u32]> {
        self.dice_face_runtime(face)
            .map(super::face_effect::RuntimeDiceFace::effect_references)
    }

    fn dice_face_runtime(&self, face: &str) -> Option<&super::face_effect::RuntimeDiceFace> {
        self.audience
            .faces()
            .any(|candidate| candidate == face)
            .then(|| self.face_effects.face(face))
            .flatten()
    }

    /// Returns the seven released choices for a Communing story stage.
    pub fn communing_choices(&self, story_stage: u16) -> impl Iterator<Item = &str> {
        self.communing_rules.choices(self, story_stage)
    }

    /// Whether an authored Communing choice is currently eligible.
    pub fn communing_choice_available(
        &self,
        state: &ActivityTransactionState,
        story_stage: u16,
        choice: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.communing_rules
            .choice_available(self, state, story_stage, choice)
    }

    /// Compiles one branch-scoped Communing choice counter increment.
    ///
    /// Released story choices do not directly grant permanent Communing
    /// points. The accepted program consumes no RNG and closes that stage for
    /// the current Attempt.
    pub fn compile_communing_choice(
        &self,
        state: &ActivityTransactionState,
        story_stage: u16,
        choice: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.communing_rules
            .compile_choice(self, state, story_stage, choice)
    }

    /// Returns the accepted-choice count for one shared Path.
    pub fn communing_choice_count(
        &self,
        state: &ActivityTransactionState,
        shared_path: &str,
    ) -> Result<i64, UniverseCatalogLoadError> {
        self.communing_rules.choice_count(self, state, shared_path)
    }

    /// Returns persistent Communing points for a dimension or shared Path.
    pub fn communing_points(
        &self,
        state: &ActivityTransactionState,
        dimension: &str,
    ) -> Result<i64, UniverseCatalogLoadError> {
        self.communing_rules
            .dimension_points(self, state, dimension)
    }

    /// Returns the released per-dimension maximum.
    #[must_use]
    pub fn communing_maximum(&self, dimension: &str) -> Option<i64> {
        self.communing_rules.dimension_maximum(self, dimension)
    }

    /// Returns currently eligible Pathstrider cabinets in authored order.
    pub fn available_pathstrider_cabinets<'a>(
        &'a self,
        state: &ActivityTransactionState,
    ) -> Result<Box<[&'a str]>, UniverseCatalogLoadError> {
        self.communing_rules.available_cabinets(self, state)
    }

    /// Whether one authored Pathstrider cabinet is currently eligible.
    pub fn pathstrider_cabinet_available(
        &self,
        state: &ActivityTransactionState,
        cabinet: &str,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.communing_rules.cabinet_available(self, state, cabinet)
    }

    /// Returns the exact released objective that authorizes a cabinet.
    #[must_use]
    pub fn pathstrider_cabinet_objective(&self, cabinet: &str) -> Option<&str> {
        self.communing_rules.cabinet_objective(self, cabinet)
    }

    /// Returns prerequisite cabinets in authored order.
    #[must_use]
    pub fn pathstrider_cabinet_prerequisites(
        &self,
        cabinet: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.communing_rules.cabinet_prerequisites(self, cabinet)
    }

    /// Compiles an objective-authorized cabinet completion and ordered point
    /// grants. Each increment clamps independently to its dimension maximum.
    pub fn compile_pathstrider_cabinet_completion(
        &self,
        state: &ActivityTransactionState,
        cabinet: &str,
        completed_objective: &str,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.communing_rules
            .compile_cabinet_completion(self, state, cabinet, completed_objective)
    }

    /// Compiles one five-tier Phase 3 boundary into a single Activity program.
    ///
    /// The ProjectPolicy order is movement/Countdown, selected face,
    /// optional map replacement, optional Communing choice, then optional
    /// cabinet completion. `rng` is transactional: a later validation failure
    /// restores every stream. No new mode-owned state machine is introduced.
    // Goal 20 freezes exactly four public mode types; grouping the two optional
    // Communing operations avoids introducing a fifth request wrapper type.
    #[allow(clippy::type_complexity)]
    pub fn compile_simultaneous_resolution(
        &self,
        state: &ActivityTransactionState,
        movement: Option<(NodeId, &[(u32, i64)])>,
        explicit_face_target: Option<NodeId>,
        map_replacement: Option<(NodeId, &str, Option<&str>)>,
        communing: (Option<(u16, &str)>, Option<(&str, &str)>),
        rng: &mut ActivityRngStreams,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        let mut request = super::simultaneous::SwarmSimultaneousResolution::new()
            .with_face_activation(explicit_face_target);
        if let Some((target, adjustments)) = movement {
            request = request.with_movement(target, adjustments);
        }
        if let Some((target, domain, beacon)) = map_replacement {
            request = request.with_map_replacement(target, domain, beacon);
        }
        if let Some((story_stage, choice)) = communing.0 {
            request = request.with_communing_choice(story_stage, choice);
        }
        if let Some((cabinet, objective)) = communing.1 {
            request = request.with_cabinet_completion(cabinet, objective);
        }
        super::simultaneous::compile(self, state, request, rng)
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
        self.topology_rules.compile_creation(
            &self.map,
            super::topology_rule_runtime::PlaneMapContext {
                board: &plane.board_key,
                nodes: &nodes,
                terminal: plane.end,
                terminal_domain: super::map_overlay::terminal_domain(plane_ordinal)?,
            },
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
        self.topology_rules.compile_event_then_creation(
            &self.map,
            super::topology_rule_runtime::PlaneMapContext {
                board: &plane.board_key,
                nodes: &nodes,
                terminal: plane.end,
                terminal_domain: super::map_overlay::terminal_domain(plane_ordinal)?,
            },
            trigger,
            parameter,
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
        self.topology_rules
            .compile_replacement(&self.map, target, domain, beacon)
    }

    /// Compiles a source-domain copy while preserving the target beacon.
    pub fn compile_node_copy(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.require_graph_node(source)?;
        self.require_graph_node(target)?;
        self.topology_rules.compile_copy(&self.map, source, target)
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
        self.topology_rules.compile_blank(&self.map, target)
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
        self.disarray_rules
            .compile_move(&self.countdown, state, adjustments)
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
        self.disarray_rules
            .compile_adjustments(&self.countdown, state, adjustments)
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
        self.disarray_rules
            .compile_boss_decay_selection(&self.countdown, state, keys)
    }

    /// Returns the two released displayed boss choices in stable source order.
    #[must_use]
    pub fn boss_choices(&self) -> impl ExactSizeIterator<Item = &str> {
        self.boss_rules.transitions(&self.transitions).choices()
    }

    /// Returns the explicitly selected boss for `plane_layer`, when present.
    #[must_use]
    pub fn selected_boss(&self, state: &ActivityTransactionState, plane_layer: u8) -> Option<&str> {
        self.boss_rules
            .transitions(&self.transitions)
            .selected_boss(state, plane_layer)
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
        self.boss_rules
            .transitions(&self.transitions)
            .compile_selection(plane_layer, boss)
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
        let boss_decay =
            self.disarray_rules
                .completion_requirements(&self.countdown, state, plane_layer)?;
        self.boss_rules
            .transitions(&self.transitions)
            .compile_completion(
                boss_decay,
                state,
                &self.graph,
                &self.planes,
                plane_layer,
                self.progression_rules.trail_next_plane_rerolls(self),
            )
    }

    /// Applies selected Trail run-start resources exactly once through an
    /// ordinary accepted Activity transaction.
    pub fn compile_trail_run_start(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<ActivityProgramDefinition, UniverseCatalogLoadError> {
        self.progression_rules.compile_trail_run_start(self, state)
    }

    /// Selected Trail nodes in canonical dimension/threshold/source order.
    pub fn communing_trail_nodes(&self) -> impl ExactSizeIterator<Item = (&str, u16)> {
        self.progression_rules.trail_nodes(self)
    }

    /// Returns the selected predecessor chain for one selected Trail node.
    pub fn communing_trail_prerequisites(
        &self,
        node: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.progression_rules.trail_prerequisites(self, node)
    }

    /// Immutable BattleSpec-bound Trail effect references and provenance.
    pub fn communing_trail_battle_effects(
        &self,
    ) -> impl ExactSizeIterator<Item = (&str, &str, &str)> {
        self.progression_rules.trail_battle_effects(self)
    }

    /// Canonical scalar parameters for one selected battle effect reference.
    pub fn communing_trail_battle_effect_parameters(
        &self,
        effect_ref: &str,
    ) -> Option<impl ExactSizeIterator<Item = &str>> {
        self.progression_rules
            .trail_battle_effect_parameters(self, effect_ref)
    }

    /// Canonical digest of selected Trail nodes and exact effect parameters.
    #[must_use]
    pub const fn communing_trail_digest(&self) -> [u8; 32] {
        self.trail.digest()
    }

    /// Accounts for the bounded First Plane non-boss entry-damage effect.
    /// The immutable damage ratio remains a BattleSpec contribution.
    pub fn compile_trail_battle_entry_accounting(
        &self,
        state: &ActivityTransactionState,
        plane_layer: u8,
        boss: bool,
        previous_first_plane_completed: bool,
    ) -> Result<Option<ActivityProgramDefinition>, UniverseCatalogLoadError> {
        self.progression_rules
            .compile_trail_battle_entry_accounting(
                self,
                state,
                plane_layer,
                boss,
                previous_first_plane_completed,
            )
    }

    /// Returns the current authoritative Countdown.
    pub fn countdown(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        self.disarray_rules.countdown(&self.countdown, state)
    }

    /// Returns the current uncapped Planar Disarray level.
    pub fn disarray_level(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<i64, UniverseCatalogLoadError> {
        self.disarray_rules.disarray_level(&self.countdown, state)
    }

    /// Returns cumulative enemy damage, mitigation and Speed percentages.
    ///
    /// The tuple is `(damage dealt, damage received reduction, speed)` and is
    /// capped at the level-20 contribution schedule.
    pub fn disarray_modifiers(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<(i64, i64, i64), UniverseCatalogLoadError> {
        self.disarray_rules
            .disarray_modifiers(&self.countdown, state)
    }

    /// Whether the current Countdown is at or below the catalog warning value.
    pub fn countdown_warning_active(
        &self,
        state: &ActivityTransactionState,
    ) -> Result<bool, UniverseCatalogLoadError> {
        self.disarray_rules.warning_active(&self.countdown, state)
    }

    fn require_graph_node(&self, node: NodeId) -> Result<(), UniverseCatalogLoadError> {
        self.graph
            .node(node)
            .map(|_| ())
            .ok_or_else(super::map_overlay::invalid_node)
    }
}
