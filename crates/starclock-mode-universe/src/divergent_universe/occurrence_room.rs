//! Authored event rewards at explicitly selected source-position rooms.
//! No original event-card membership or missing NPC graph is inferred.

#[path = "occurrence_room_sequence.rs"]
mod sequence;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioRuntime, DivergentUniverseLogicalScopeKind,
    DivergentUniverseRuntimeFactory,
    decision_rewards::DecisionRewardGrant,
    domain_route::{DomainRoomContext, DomainRoomProgram, DomainRouteError},
    occurrence_binding::{OccurrenceBinding, OccurrenceBindingError, OccurrenceExecutionError},
    state::{ROOM_CONTENT_ENABLED_SLOT, ROOM_DOORS_OPEN_SLOT},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityExpression, ActivityGeneratedBoundaryResolution,
    ActivityInteractionBindings, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityStateHash, ActivityValue, GraphActivity, GraphActivityDefinition,
    GraphActivityNodeProgram, NodeId,
};
use starclock_data::divergent_universe_decisions::reward_occurrences::RewardOccurrenceId;
use starclock_data::divergent_universe_service_catalog::DivergentUniverseOccurrenceVariantId;
use std::iter::once;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OccurrenceRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    binding: Arc<OccurrenceBinding>,
    curios: Arc<DivergentUniverseCurioRuntime>,
}

/// Immutable raw contribution plus the required lifecycle-prefixed entry.
/// Supply the raw fragment through `compile_curio_domain_route` and bind the
/// validated whole profile before accepting an event selection.
#[derive(Clone, Debug)]
pub struct CompiledOccurrenceRoom {
    authored_reward: Option<RewardOccurrenceId>,
    context: DomainRoomContext,
    binding: Arc<OccurrenceBinding>,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    choice_node: NodeId,
    following: Box<[OccurrenceRoomStep]>,
    component: [u8; 32],
    decisions: [u8; 32],
}

#[derive(Clone, Debug)]
struct OccurrenceRoomStep {
    node: NodeId,
    binding: Arc<OccurrenceBinding>,
}

/// Capability for exactly one room in an immutable whole Activity definition.
#[derive(Clone, Debug)]
pub struct BoundOccurrenceRoom {
    room: CompiledOccurrenceRoom,
    definition: Arc<GraphActivityDefinition>,
}

#[derive(Debug)]
pub enum OccurrenceRoomError {
    Binding(OccurrenceBindingError),
    Route(DomainRouteError),
    InvalidContext,
    DefinitionMismatch,
    InvalidSequenceLength,
}
impl std::fmt::Display for OccurrenceRoomError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe occurrence room: {self:?}")
    }
}
impl std::error::Error for OccurrenceRoomError {}

impl DivergentUniverseRuntimeFactory {
    /// Compiles an explicitly selected authored event, not a source room pool.
    /// Unknown/unimplemented variants reject; no generic event is substituted.
    pub fn occurrence_room_compiler(
        &self,
        variant: &DivergentUniverseOccurrenceVariantId,
    ) -> Result<OccurrenceRoomCompiler, OccurrenceRoomError> {
        Ok(OccurrenceRoomCompiler {
            factory: self.clone(),
            binding: Arc::new(
                OccurrenceBinding::compile(self, variant).map_err(OccurrenceRoomError::Binding)?,
            ),
            curios: Arc::new(
                self.curio_runtime()
                    .map_err(DomainRouteError::Curio)
                    .map_err(OccurrenceRoomError::Route)?,
            ),
        })
    }
}

impl OccurrenceRoomCompiler {
    /// Instantiates the existing choices/costs/rewards and finish-before-door
    /// protocol in this exact namespace. Entry counts once; selection and Leave
    /// are separate offered commands at the choice node. A small entry transition
    /// keeps lifecycle branches from duplicating the full authored choice program.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledOccurrenceRoom, OccurrenceRoomError> {
        if !self.factory.room_context_matches(context) {
            return Err(OccurrenceRoomError::InvalidContext);
        }
        let node = context.entry_node();
        let choice_node = context.node(1).map_err(OccurrenceRoomError::Route)?;
        let enter = context.edge(0).map_err(OccurrenceRoomError::Route)?;
        let route = ActivityOperation::Offer {
            kind: ActivityDecisionKind::Route,
            options: vec![ActivityOptionDefinition::new(
                ActivityOptionId::new(1).expect("fixed nonzero continuation"),
                0,
                ActivityCondition::Boolean(ActivityExpression::Slot(ROOM_DOORS_OPEN_SLOT)),
                vec![
                    ActivityOperation::Require(ActivityCondition::Boolean(
                        ActivityExpression::Slot(ROOM_DOORS_OPEN_SLOT),
                    )),
                    ActivityOperation::Traverse(context.exit_edge()),
                ],
            )]
            .into(),
        };
        let choices = self
            .binding
            .wrap_checkpoint(vec![
                ActivityOperation::SetSlot {
                    slot: ROOM_CONTENT_ENABLED_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
                },
                route,
            ])
            .map_err(OccurrenceRoomError::Binding)?;
        let operations = vec![ActivityOperation::Traverse(enter)];
        let id = ActivityProgramId::new(node.get()).ok_or(OccurrenceRoomError::InvalidContext)?;
        let raw = ActivityProgramDefinition::new(id, operations.clone())
            .map_err(DomainRouteError::Program)
            .map_err(OccurrenceRoomError::Route)?;
        let entry_program = GraphActivityNodeProgram::new(
            node,
            ActivityProgramDefinition::new(
                id,
                self.curios
                    .compiled_domain_entry_operations(operations)
                    .map_err(DomainRouteError::Curio)
                    .map_err(OccurrenceRoomError::Route)?,
            )
            .map_err(DomainRouteError::Program)
            .map_err(OccurrenceRoomError::Route)?,
        );
        Ok(CompiledOccurrenceRoom {
            authored_reward: None,
            context: context.clone(),
            binding: Arc::clone(&self.binding),
            entry_program,
            choice_node,
            following: Box::new([]),
            component: self.factory.bundle_identity().component_digest().bytes(),
            decisions: self.factory.decision_catalog().digest(),
            fragment: DomainRoomProgram {
                exit_node: choice_node,
                nodes: [node, choice_node]
                    .into_iter()
                    .map(|physical| {
                        ActivityNodeDefinition::new(
                            physical,
                            context.section,
                            ActivityNodeKind::Choice,
                            1,
                        )
                        .map_err(DomainRouteError::Graph)
                        .map_err(OccurrenceRoomError::Route)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
                programs: vec![
                    GraphActivityNodeProgram::new(node, raw),
                    GraphActivityNodeProgram::new(
                        choice_node,
                        ActivityProgramDefinition::new(
                            ActivityProgramId::new(choice_node.get())
                                .ok_or(OccurrenceRoomError::InvalidContext)?,
                            choices,
                        )
                        .map_err(DomainRouteError::Program)
                        .map_err(OccurrenceRoomError::Route)?,
                    ),
                ],
                edges: vec![
                    ActivityEdgeDefinition::new(
                        enter,
                        node,
                        choice_node,
                        ActivityEdgeCondition::Always,
                        0,
                        1,
                    )
                    .map_err(DomainRouteError::Graph)
                    .map_err(OccurrenceRoomError::Route)?,
                ],
                random_offers: Vec::new(),
            },
        })
    }
}

impl CompiledOccurrenceRoom {
    pub(super) fn with_authored_reward_selection(mut self, id: RewardOccurrenceId) -> Self {
        self.authored_reward = Some(id);
        self
    }

    /// Absent for explicitly caller-placed events; present for Sora Reward admission.
    #[must_use]
    pub fn authored_reward_selection(&self) -> Option<&RewardOccurrenceId> {
        self.authored_reward.as_ref()
    }

    #[must_use]
    pub fn context(&self) -> &DomainRoomContext {
        &self.context
    }
    #[must_use]
    pub fn variant(&self) -> &DivergentUniverseOccurrenceVariantId {
        self.binding.variant()
    }
    /// All explicitly selected events in execution order, including repeats.
    /// `variant()` remains the first selection; observation reports the active
    /// event rather than assuming that every step uses the first variant.
    pub fn variants(&self) -> impl Iterator<Item = &DivergentUniverseOccurrenceVariantId> {
        once(self.variant()).chain(self.following.iter().map(|step| step.binding.variant()))
    }
    /// Raw contribution for `compile_curio_domain_route`, not a ready binding.
    #[must_use]
    pub fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }
    /// Binds current catalogs, event and exact placement; no compatibility claim.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.divergent-universe.explicit-position-occurrence.v1");
        hash.update(self.component);
        hash.update(self.decisions);
        if let Some(selection) = &self.authored_reward {
            hash.update(b"authored-reward-selection");
            hash.update(
                u64::try_from(selection.as_str().len())
                    .expect("bounded key")
                    .to_le_bytes(),
            );
            hash.update(selection.as_str().as_bytes());
        }
        for value in [
            self.context.area.as_str(),
            self.context.layer.as_str(),
            self.context.position_key.as_ref(),
            self.context.preset_source.as_ref(),
            self.variant().as_str(),
        ] {
            hash.update(
                u64::try_from(value.len())
                    .expect("bounded authored input")
                    .to_le_bytes(),
            );
            hash.update(value.as_bytes());
        }
        hash.update(self.context.entry_node().get().to_le_bytes());
        hash.update(self.context.exit_edge().get().to_le_bytes());
        hash.update(self.context.successor().get().to_le_bytes());
        hash.update(self.context.level.to_le_bytes());
        if !self.following.is_empty() {
            hash.update(
                b"explicit-ordered-events.partial-work.final-room-finish.independent-continuation",
            );
            hash.update(
                u16::try_from(self.following.len())
                    .expect("compiler bounds following events")
                    .to_le_bytes(),
            );
            for step in &self.following {
                hash.update(step.node.get().to_le_bytes());
                let variant = step.binding.variant().as_str();
                hash.update(
                    u64::try_from(variant.len())
                        .expect("bounded variant key")
                        .to_le_bytes(),
                );
                hash.update(variant.as_bytes());
            }
        }
        hash.finalize()
    }
    pub(in crate::divergent_universe) fn matches_factory(
        &self,
        factory: &DivergentUniverseRuntimeFactory,
    ) -> bool {
        self.component == factory.bundle_identity().component_digest().bytes()
            && self.decisions == factory.decision_catalog().digest()
            && factory.room_context_matches(&self.context)
    }
    /// Rejects missing/changed programs, bypass exits, extra random policies and
    /// wrong logical-room bindings before issuing a command capability.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundOccurrenceRoom, OccurrenceRoomError> {
        let node = self.context.entry_node();
        let graph = definition.graph();
        let owns = |id| self.fragment.nodes.iter().any(|owned| owned.id() == id);
        let scopes = definition.state_definition().logical_scopes();
        // Other rooms may carry authored external outcomes, but none may target
        // this event fragment or introduce executable handler registrations.
        if definition.interactions().is_some_and(|interactions| {
            interactions
                .bindings()
                .iter()
                .any(|binding| owns(binding.node()) || binding.handler().is_some())
                || interactions
                    .registry()
                    .bundles()
                    .iter()
                    .any(|bundle| !bundle.registrations().is_empty())
        }) || (owns(graph.entry()) && graph.entry() != node)
            || !graph.edges().iter().any(|edge| {
                edge.id() == self.context.exit_edge()
                    && edge.from() == self.fragment.exit_node
                    && edge.to() == self.context.successor()
                    && edge.condition() == ActivityEdgeCondition::Always
                    && edge.maximum_traversals() == 1
            })
            || self
                .fragment
                .nodes
                .iter()
                .any(|owned| graph.node(owned.id()) != Some(owned))
            || self
                .fragment
                .edges
                .iter()
                .any(|edge| !graph.edges().contains(edge))
            || graph.edges().iter().any(|edge| {
                (owns(edge.from())
                    && edge.id() != self.context.exit_edge()
                    && !self.fragment.edges.contains(edge))
                    || (!owns(edge.from()) && owns(edge.to()) && edge.to() != node)
            })
            || self.fragment.programs.iter().any(|program| {
                let expected = if program.node() == node {
                    &self.entry_program
                } else {
                    program
                };
                !definition.programs().contains(expected)
            })
            || definition
                .random_offers()
                .iter()
                .any(|offer| owns(offer.node()))
            || definition
                .random_checkpoints()
                .iter()
                .any(|checkpoint| owns(checkpoint.node()))
            || self.fragment.nodes.iter().any(|owned| {
                !scopes.bindings().iter().any(|binding| {
                    let path = binding.path();
                    binding.node() == owned.id()
                        && path.len() == 3
                        && path[0].class() == DivergentUniverseLogicalScopeKind::Run.class_id()
                        && path[0].key() == 1
                        && path[1].class() == DivergentUniverseLogicalScopeKind::Plane.class_id()
                        && path[1].key() == u64::from(self.context.plane_ordinal)
                        && path[2].class() == DivergentUniverseLogicalScopeKind::Node.class_id()
                        && path[2].key() == u64::from(self.context.position_ordinal)
                })
            })
        {
            return Err(OccurrenceRoomError::DefinitionMismatch);
        }
        Ok(BoundOccurrenceRoom {
            room: self.clone(),
            definition,
        })
    }
}

impl BoundOccurrenceRoom {
    /// Observes only this exact graph's active authored event-choice offer.
    #[must_use]
    pub fn offered(
        &self,
        activity: &GraphActivity,
    ) -> Option<&DivergentUniverseOccurrenceVariantId> {
        let view = activity.player_view();
        if !self.matches(activity)
            || !view
                .decision()
                .is_some_and(|offer| offer.kind() == ActivityDecisionKind::Choice)
        {
            return None;
        }
        self.room
            .binding_at(view.current_node())
            .map(OccurrenceBinding::variant)
    }
    /// Authentication precedes RNG. Reward, acquisition effects, finish and door
    /// publication commit in the existing shared generated-option transaction.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<DecisionRewardGrant>, OccurrenceExecutionError>
    {
        if !self.matches(activity) {
            return Err(OccurrenceExecutionError::DefinitionMismatch);
        }
        let node = activity.current_node();
        self.room
            .binding_at(node)
            .ok_or(OccurrenceExecutionError::NotOffered)?
            .execute_at_node(activity, expected, decision, option, node)
    }
    pub(in crate::divergent_universe) fn matches_factory(
        &self,
        factory: &DivergentUniverseRuntimeFactory,
    ) -> bool {
        self.room.matches_factory(factory)
    }
    fn matches(&self, activity: &GraphActivity) -> bool {
        let actual = activity.definition();
        Arc::ptr_eq(actual, &self.definition)
            || (actual.identity() == self.definition.identity()
                && actual.graph().digest() == self.definition.graph().digest()
                && actual.state_definition() == self.definition.state_definition()
                && actual.participants().digest() == self.definition.participants().digest()
                && actual.programs() == self.definition.programs()
                && actual.bootstrap() == self.definition.bootstrap()
                && actual.random_offers() == self.definition.random_offers()
                && actual.random_checkpoints() == self.definition.random_checkpoints()
                && actual
                    .interactions()
                    .map(ActivityInteractionBindings::bindings)
                    == self
                        .definition
                        .interactions()
                        .map(ActivityInteractionBindings::bindings)
                && actual
                    .interactions()
                    .map(|bindings| bindings.registry().digest())
                    == self
                        .definition
                        .interactions()
                        .map(|bindings| bindings.registry().digest()))
    }
}

impl CompiledOccurrenceRoom {
    fn binding_at(&self, node: NodeId) -> Option<&OccurrenceBinding> {
        if node == self.choice_node {
            Some(&self.binding)
        } else {
            self.following
                .iter()
                .find(|step| step.node == node)
                .map(|step| step.binding.as_ref())
        }
    }
}
