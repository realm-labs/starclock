//! Current Persona positions compiled into the shared Activity graph.
//!
//! This is a strict composition boundary, not a room-payload implementation.
//! An explicitly selected authored deck supplies unspecified positions. Every
//! reachable fixed/card preset requires a caller-owned executable room fragment;
//! missing fragments fail closed. The provisional baseline does not use this
//! compiler yet. Neither compiling a route nor supplying test probes earns
//! source-mechanic or full-run coverage.
//! Runtime profiles use `compile_curio_domain_route` to add the authored Curio
//! lifecycle at fixed/selected entries; the raw composition API does not add it.

use std::error::Error;
use std::fmt::{Display, Formatter};

#[path = "domain_route_compilation.rs"]
mod compilation;
#[path = "domain_route_validation.rs"]
mod validation;

use crate::divergent_universe::{
    DivergentUniverseCurioRuntimeError,
    domain_deck::{DomainDeck, DomainDeckError},
};
use starclock_activity::{
    ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition, ActivityGraphDefinitionError,
    ActivityNodeDefinition, ActivityProgramDefinitionError, ActivityRandomOffer,
    GraphActivityNodeProgram, LogicalScopeDefinitions, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_catalog::{DivergentUniverseAreaId, DivergentUniverseLayerId},
    divergent_universe_domain_decks::DomainCardKind,
    divergent_universe_domain_layout::FixedDomainKind,
};

/// A fixed source preset and a deck composition are different admission claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomainRoomComposition {
    Fixed(FixedDomainKind),
    Card(DomainCardKind),
}

/// One room alternative at one exact source position. The compiler allocates
/// disjoint bounded namespaces; a fragment cannot jump to another alternative
/// or skip positions. All its physical nodes share the same logical room.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainRoomContext {
    pub area: DivergentUniverseAreaId,
    pub layer: DivergentUniverseLayerId,
    pub position_key: Box<str>,
    pub position_ordinal: u16,
    pub plane_ordinal: u32,
    pub preset_source: Box<str>,
    pub composition: DomainRoomComposition,
    pub level: u16,
    pub section: SectionId,
    node_base: u32,
    edge_base: u32,
    next: NodeId,
}

impl DomainRoomContext {
    /// Local node indices are 0..64; index zero is the required room entry.
    pub fn node(&self, index: u16) -> Result<NodeId, DomainRouteError> {
        if index >= 64 {
            return Err(DomainRouteError::InvalidFragment(
                self.preset_source.clone(),
            ));
        }
        NodeId::new(self.node_base + u32::from(index)).ok_or(DomainRouteError::AddressOverflow)
    }

    /// Local internal edges are 0..127. Index 127 is reserved for the single
    /// compiler-owned exit; programs may traverse it only from `exit_node`.
    pub fn edge(&self, index: u16) -> Result<ActivityEdgeId, DomainRouteError> {
        if index >= 127 {
            return Err(DomainRouteError::InvalidFragment(
                self.preset_source.clone(),
            ));
        }
        ActivityEdgeId::new(self.edge_base + u32::from(index))
            .ok_or(DomainRouteError::AddressOverflow)
    }

    #[must_use]
    pub fn entry_node(&self) -> NodeId {
        NodeId::new(self.node_base).expect("compiler checked room namespace")
    }

    #[must_use]
    pub fn exit_edge(&self) -> ActivityEdgeId {
        ActivityEdgeId::new(self.edge_base + 127).expect("compiler checked room namespace")
    }

    pub(super) const fn successor(&self) -> NodeId {
        self.next
    }
}

/// Executable room fragment supplied by the owning content compiler. This can
/// contain encounter, battle, reward, service and failure nodes, not just one
/// room marker. Programs own their completion guards; the route never completes
/// room content or grants rewards on their behalf. `exit_node` is the sole
/// successful continuation. Failed/abandoned/faulted terminals are permitted.
/// Program IDs use the same 64-value namespace as `context.node(index)`; internal
/// edges use `context.edge(index)`. Offer prefixes/markers/reroll counters cannot
/// write deck-owned slots or impersonate the deck's Graph-purpose offer.
#[derive(Clone, Debug)]
pub struct DomainRoomProgram {
    pub exit_node: NodeId,
    pub nodes: Vec<ActivityNodeDefinition>,
    pub edges: Vec<ActivityEdgeDefinition>,
    pub programs: Vec<GraphActivityNodeProgram>,
    pub random_offers: Vec<ActivityRandomOffer>,
}

/// Immutable graph contribution for an explicitly selected deck and exact area.
/// The owning profile must bind the factory's source/decision digests, selection
/// and caller width policy into its configuration identity, use `deck`'s exact
/// slot declarations and these logical scopes, then validate all contributions
/// together with `GraphActivityDefinition::new`. Mutable execution, RNG,
/// commands, replay and hashes remain entirely in the shared Activity engine.
#[derive(Clone, Debug)]
pub struct CompiledDomainRoute {
    pub graph: ActivityGraphDefinition,
    pub programs: Vec<GraphActivityNodeProgram>,
    pub random_offers: Vec<ActivityRandomOffer>,
    pub logical_scopes: LogicalScopeDefinitions,
    pub deck: DomainDeck,
    pub rooms: Box<[DomainRoomContext]>,
}

#[derive(Debug)]
pub enum DomainRouteError {
    UnknownArea,
    MissingLayer(DivergentUniverseLayerId),
    MissingRoomProgram(Box<str>),
    InvalidFragment(Box<str>),
    AddressOverflow,
    InvalidLogicalScopes,
    Deck(DomainDeckError),
    Graph(ActivityGraphDefinitionError),
    Program(ActivityProgramDefinitionError),
    Curio(DivergentUniverseCurioRuntimeError),
}

impl Display for DomainRouteError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe domain route: {self:?}")
    }
}
impl Error for DomainRouteError {}
