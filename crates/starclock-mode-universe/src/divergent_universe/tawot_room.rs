//! Explicitly placed paid Tawot service in a current source-position room.
//!
//! The caller selects an existing authored service level independently of the
//! source preset's composition level. This is the existing explicit-service
//! placement policy, not an automatic Forge selector. In particular, the nine
//! current decks only contain level-one Reforge presets, while the authored
//! Tawot service covers levels two through five. Nothing here promotes those
//! cards to a different level or claims a level-one Forge payload is complete.

use std::sync::Arc;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioRuntime, DivergentUniverseEntryFlowError,
    DivergentUniverseLogicalScopeKind, DivergentUniverseRuntimeFactory,
    domain_route::{DomainRoomContext, DomainRoomProgram, DomainRouteError},
    economy::{self, DivergentUniverseEconomyError},
    state::{TAWOT_ACCEPTED_SLOT, TAWOT_OFFER_SLOT, TAWOT_OPENS_SLOT, TAWOT_PURCHASES_SLOT},
    tawot_service::{TawotService, TawotServicePhase},
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityGeneratedBoundaryResolution, ActivityInteractionBindings,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivityScope, ActivitySlotDefinition,
    ActivityStateHash, ActivityStateSource, ActivityStateVisibility, ActivityValue, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition, GraphActivityNodeProgram, NodeId,
    SlotCarryPolicy, SlotResetPoint,
};
use starclock_data::divergent_universe_decisions::TawotServiceDefinition;

/// Immutable executable fragment plus the service that generated its offers.
/// Include its exact slots and fragment through `compile_curio_domain_route`,
/// then bind the validated whole graph before accepting any service command.
#[derive(Clone, Debug)]
pub struct CompiledTawotRoom {
    context: DomainRoomContext,
    service: Arc<TawotService>,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    slots: Vec<ActivitySlotDefinition>,
    menu: NodeId,
    cards: NodeId,
    component: [u8; 32],
    decisions: [u8; 32],
}

/// One immutable authored service compiler reused across position alternatives.
/// Sharing this value does not share purchases, cached offers or RNG state.
#[derive(Clone, Debug)]
pub struct TawotRoomCompiler {
    service: Arc<TawotService>,
    factory: DivergentUniverseRuntimeFactory,
    curios: Arc<DivergentUniverseCurioRuntime>,
}

/// A trusted mode-executor capability for one exact immutable graph definition.
/// All mutable state and all menu/card loops remain in the shared Activity.
#[derive(Clone, Debug)]
pub struct BoundTawotRoom {
    room: CompiledTawotRoom,
    definition: Arc<GraphActivityDefinition>,
}

#[derive(Debug)]
pub enum TawotRoomError {
    Route(DomainRouteError),
    Economy(DivergentUniverseEconomyError),
    Service(DivergentUniverseEntryFlowError),
    InvalidSlots,
    InvalidContext,
    DefinitionMismatch,
}

impl std::fmt::Display for TawotRoomError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe Tawot room: {self:?}")
    }
}
impl std::error::Error for TawotRoomError {}

impl DivergentUniverseRuntimeFactory {
    /// Explicitly places the current paid service at a source-position context.
    /// `service_level` is a caller-selected policy input, NOT `context.level`.
    /// Unknown service levels reject; no original NPC/Forge membership is inferred.
    /// The owning profile must bind this placement, its service level and the
    /// factory's exact source/decision digests into configuration identity.
    pub fn compile_tawot_room(
        &self,
        context: &DomainRoomContext,
        service_level: u16,
    ) -> Result<CompiledTawotRoom, TawotRoomError> {
        self.tawot_room_compiler(service_level)?.compile(context)
    }

    /// Loads one selected service policy once for all room alternatives. The
    /// returned compiler retains only immutable validated definitions.
    pub fn tawot_room_compiler(
        &self,
        service_level: u16,
    ) -> Result<TawotRoomCompiler, TawotRoomError> {
        let economy = economy::compile(
            self.bundle.service_catalog(),
            self.bundle.progression_catalog(),
            self.bundle.curio_catalog(),
            self.decision_catalog(),
        )
        .map_err(TawotRoomError::Economy)?;
        let service = TawotService::compile(self, service_level, &economy)
            .map_err(TawotRoomError::Service)?;
        Ok(TawotRoomCompiler {
            service: Arc::new(service),
            factory: self.clone(),
            curios: Arc::new(
                self.curio_runtime()
                    .map_err(DomainRouteError::Curio)
                    .map_err(TawotRoomError::Route)?,
            ),
        })
    }
}

impl TawotRoomCompiler {
    /// Instantiates the exact program in this context's bounded namespace.
    /// Admission remains caller-selected and independent of the preset level.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledTawotRoom, TawotRoomError> {
        if !self.factory.room_context_matches(context) {
            return Err(TawotRoomError::InvalidContext);
        }
        let service = &self.service;
        let entry = context.entry_node();
        let menu = context.node(1).map_err(TawotRoomError::Route)?;
        let cards = context.node(2).map_err(TawotRoomError::Route)?;
        let enter = context.edge(0).map_err(TawotRoomError::Route)?;
        let open = context.edge(1).map_err(TawotRoomError::Route)?;
        let back = context.edge(2).map_err(TawotRoomError::Route)?;
        let limit = TawotService::open_limit();
        let nodes = [
            (entry, ActivityNodeKind::Choice, 1),
            (menu, ActivityNodeKind::Choice, limit + 1),
            (cards, ActivityNodeKind::Reward, limit),
        ]
        .into_iter()
        .map(|(node, kind, visits)| {
            ActivityNodeDefinition::new(node, context.section, kind, visits)
                .map_err(DomainRouteError::Graph)
                .map_err(TawotRoomError::Route)
        })
        .collect::<Result<Vec<_>, _>>()?;
        let edges = [
            (enter, entry, menu, 1),
            (open, menu, cards, limit),
            (back, cards, menu, limit),
        ]
        .into_iter()
        .map(|(id, from, to, traversals)| {
            ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::Always, 0, traversals)
                .map_err(DomainRouteError::Graph)
                .map_err(TawotRoomError::Route)
        })
        .collect::<Result<Vec<_>, _>>()?;
        let initialize = vec![
            ActivityOperation::SetSlot {
                slot: TAWOT_PURCHASES_SLOT,
                value: literal(ActivityValue::BoundedInteger(0)),
            },
            ActivityOperation::SetSlot {
                slot: TAWOT_OPENS_SLOT,
                value: literal(ActivityValue::BoundedInteger(0)),
            },
            ActivityOperation::SetOrderedIdSet {
                slot: TAWOT_OFFER_SLOT,
                values: Box::new([]),
            },
            ActivityOperation::Traverse(enter),
        ];
        let programs = [
            (entry, initialize),
            (menu, service.menu_program(open, context.exit_edge())),
            (cards, service.cards_program(back)),
        ]
        .into_iter()
        .map(|(node, operations)| {
            let id =
                ActivityProgramId::new(node.get()).ok_or(TawotRoomError::DefinitionMismatch)?;
            let program = ActivityProgramDefinition::new(id, operations)
                .map_err(DomainRouteError::Program)
                .map_err(TawotRoomError::Route)?;
            Ok(GraphActivityNodeProgram::new(node, program))
        })
        .collect::<Result<Vec<_>, TawotRoomError>>()?;
        let fragment = DomainRoomProgram {
            exit_node: menu,
            nodes,
            edges,
            programs,
            random_offers: Vec::new(),
        };
        let original = &fragment.programs[0];
        let entry_program = GraphActivityNodeProgram::new(
            original.node(),
            ActivityProgramDefinition::new(
                original.program().id(),
                self.curios
                    .compiled_domain_entry_operations(original.program().operations().to_vec())
                    .map_err(DomainRouteError::Curio)
                    .map_err(TawotRoomError::Route)?,
            )
            .map_err(DomainRouteError::Program)
            .map_err(TawotRoomError::Route)?,
        );
        Ok(CompiledTawotRoom {
            context: context.clone(),
            service: Arc::clone(service),
            fragment,
            entry_program,
            slots: room_slots()?,
            menu,
            cards,
            component: self.factory.bundle_identity().component_digest().bytes(),
            decisions: self.factory.decision_catalog().digest(),
        })
    }
}

impl CompiledTawotRoom {
    /// Binds exact source/decision inputs, placement and independently selected
    /// service level. Identical menu shapes at different levels are not the same
    /// configuration. This does not establish original Forge admission.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        for value in [
            b"starclock.divergent-universe.explicit-position-tawot-curio-entry.v1".as_slice(),
            &self.component,
            &self.decisions,
            self.context.area.as_str().as_bytes(),
            self.context.layer.as_str().as_bytes(),
            self.context.position_key.as_bytes(),
            self.context.preset_source.as_bytes(),
        ] {
            hash.update(
                u64::try_from(value.len())
                    .expect("bounded validated input length")
                    .to_le_bytes(),
            );
            hash.update(value);
        }
        hash.update(self.context.entry_node().get().to_le_bytes());
        hash.update(self.context.exit_edge().get().to_le_bytes());
        hash.update(self.context.successor().get().to_le_bytes());
        hash.update(self.context.level.to_le_bytes());
        hash.update(self.definition().forge_level.to_le_bytes());
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

    #[must_use]
    pub fn context(&self) -> &DomainRoomContext {
        &self.context
    }

    #[must_use]
    pub fn definition(&self) -> &TawotServiceDefinition {
        self.service.definition()
    }

    /// These replace the four baseline service declarations; counters/cache reset
    /// when the logical room changes, not at internal physical node transitions.
    #[must_use]
    pub fn slot_definitions(&self) -> &[ActivitySlotDefinition] {
        &self.slots
    }

    #[must_use]
    /// Raw contribution; runtime binding requires the entry lifecycle supplied
    /// by `compile_curio_domain_route`, without counting internal menu/card loops.
    pub fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }

    /// Rejects changed/missing service programs, nodes, internal edges, exit,
    /// room-lifetime bindings or slot declarations before issuing a capability.
    /// The graph must already have passed the shared full-definition validator.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundTawotRoom, TawotRoomError> {
        let graph = definition.graph();
        let scopes = definition.state_definition().logical_scopes();
        let exit_matches = graph.edges().iter().any(|edge| {
            edge.id() == self.context.exit_edge()
                && edge.from() == self.menu
                && edge.to() == self.context.successor()
                && edge.maximum_traversals() == 1
                && edge.condition() == ActivityEdgeCondition::Always
        });
        if !exit_matches
            || graph.edges().iter().any(|edge| {
                let owns = |id| self.fragment.nodes.iter().any(|node| node.id() == id);
                (owns(edge.from())
                    && edge.id() != self.context.exit_edge()
                    && !self.fragment.edges.contains(edge))
                    || (!owns(edge.from())
                        && owns(edge.to())
                        && edge.to() != self.context.entry_node())
            })
            || self
                .fragment
                .nodes
                .iter()
                .any(|node| graph.node(node.id()) != Some(node))
            || self
                .fragment
                .edges
                .iter()
                .any(|edge| !graph.edges().contains(edge))
            || self.fragment.programs.iter().any(|program| {
                let expected = if program.node() == self.context.entry_node() {
                    &self.entry_program
                } else {
                    program
                };
                !definition.programs().contains(expected)
            })
            || self
                .slots
                .iter()
                .any(|slot| !definition.state_definition().slots().contains(slot))
            || self.fragment.nodes.iter().any(|node| {
                !scopes.bindings().iter().any(|binding| {
                    let path = binding.path();
                    binding.node() == node.id()
                        && path.len() == 3
                        && path[0].class() == DivergentUniverseLogicalScopeKind::Run.class_id()
                        && path[0].key() == 1
                        && path[1].class() == DivergentUniverseLogicalScopeKind::Plane.class_id()
                        && path[1].key() == u64::from(self.context.plane_ordinal)
                        && path[2].class() == DivergentUniverseLogicalScopeKind::Node.class_id()
                        && path[2].key() == u64::from(self.context.position_ordinal)
                })
            })
            || definition.random_offers().iter().any(|offer| {
                self.fragment
                    .nodes
                    .iter()
                    .any(|node| node.id() == offer.node())
            })
            || definition.random_checkpoints().iter().any(|checkpoint| {
                self.fragment
                    .nodes
                    .iter()
                    .any(|node| node.id() == checkpoint.node())
            })
        {
            return Err(TawotRoomError::DefinitionMismatch);
        }
        Ok(BoundTawotRoom {
            room: self.clone(),
            definition,
        })
    }
}

impl BoundTawotRoom {
    /// Non-mutating observation; returns None for a foreign graph or any node
    /// other than this room's authenticated menu and cached-card offer.
    #[must_use]
    pub fn offered(&self, activity: &GraphActivity) -> Option<&TawotServiceDefinition> {
        self.phase(activity).map(|_| self.room.service.definition())
    }

    /// Offered-ID authentication precedes RNG generation. Payment, acquisition,
    /// immediate grants, allowance, card cache and movement commit atomically.
    /// Rejection, including downstream next-room failure, restores everything.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<()>, GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let phase = self
            .phase(activity)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        activity.choose_option_with_generated_prefix(expected, decision, selected, |view, rng| {
            let mut operations =
                self.room
                    .service
                    .generate_for_phase(view, selected.get(), rng, phase)?;
            operations.push(ActivityOperation::SetSlot {
                slot: TAWOT_ACCEPTED_SLOT,
                value: literal(ActivityValue::Boolean(true)),
            });
            Ok((operations, ()))
        })
    }

    fn phase(&self, activity: &GraphActivity) -> Option<TawotServicePhase> {
        let actual = activity.definition();
        // Pointer identity is only an equality fast path for immutable inputs.
        // Freshly reconstructed definitions still pass structural comparison.
        if !Arc::ptr_eq(actual, &self.definition)
            && (actual.identity() != self.definition.identity()
                || actual.graph().digest() != self.definition.graph().digest()
                || actual.state_definition() != self.definition.state_definition()
                || actual.participants().digest() != self.definition.participants().digest()
                || actual.programs() != self.definition.programs()
                || actual.bootstrap() != self.definition.bootstrap()
                || actual.random_checkpoints() != self.definition.random_checkpoints()
                || actual
                    .interactions()
                    .map(ActivityInteractionBindings::bindings)
                    != self
                        .definition
                        .interactions()
                        .map(ActivityInteractionBindings::bindings)
                || actual
                    .interactions()
                    .map(|bindings| bindings.registry().digest())
                    != self
                        .definition
                        .interactions()
                        .map(|bindings| bindings.registry().digest())
                || actual.random_offers() != self.definition.random_offers())
        {
            return None;
        }
        let view = activity.player_view();
        let kind = view.decision()?.kind();
        match (view.current_node(), kind) {
            (node, ActivityDecisionKind::Service) if node == self.room.menu => {
                Some(TawotServicePhase::Menu)
            }
            (node, ActivityDecisionKind::Reward) if node == self.room.cards => {
                Some(TawotServicePhase::Cards)
            }
            _ => None,
        }
    }
}

fn room_slots() -> Result<Vec<ActivitySlotDefinition>, TawotRoomError> {
    let values = [
        (
            TAWOT_PURCHASES_SLOT,
            ActivityValue::BoundedInteger(0),
            Some((0, 8)),
            None,
            true,
        ),
        (
            TAWOT_OFFER_SLOT,
            ActivityValue::OrderedIdSet(Box::new([])),
            None,
            Some(8),
            true,
        ),
        (
            TAWOT_ACCEPTED_SLOT,
            ActivityValue::Boolean(false),
            None,
            None,
            false,
        ),
        (
            TAWOT_OPENS_SLOT,
            ActivityValue::BoundedInteger(0),
            Some((0, 64)),
            None,
            true,
        ),
    ];
    values
        .into_iter()
        .map(|(id, initial, integer_bounds, entries, logical)| {
            let definition = ActivitySlotDefinition::new_with_policy(
                id,
                ActivityScope::Node,
                initial,
                integer_bounds,
                entries,
                vec![SlotResetPoint::NodeStart],
                SlotCarryPolicy::Reset,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(u64::from(id.get()))
                    .ok_or(TawotRoomError::InvalidSlots)?,
            )
            .map_err(|_| TawotRoomError::InvalidSlots)?;
            Ok(if logical {
                definition.with_logical_scope(DivergentUniverseLogicalScopeKind::Node.class_id())
            } else {
                definition
            })
        })
        .collect()
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
