//! Source-position ordering, deck dispatch and logical lifetime composition.

use std::collections::BTreeMap;

use crate::divergent_universe::{
    DivergentUniverseLogicalScopeKind, DivergentUniverseRuntimeFactory,
    domain_deck::{DomainCardId, DomainDeckSlots},
    domain_route::{
        CompiledDomainRoute, DomainRoomComposition, DomainRoomContext, DomainRoomProgram,
        DomainRouteError, validation::validate_fragment,
    },
};
use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, ActivityRandomOffer, ActivityTerminalOutcome, GraphActivityNodeProgram,
    LogicalScopeAddress, LogicalScopeClassDefinition, LogicalScopeDefinitions,
    LogicalScopeNodeBinding, NodeId, SectionId,
};
use starclock_data::{
    divergent_universe_catalog::{DivergentUniverseAreaId, DivergentUniverseLayerId},
    divergent_universe_domain_decks::DomainCardDefinition,
    divergent_universe_domain_layout::{DomainPosition, DomainPositionKind},
};

const TERMINAL: u32 = 1_000_000;

struct Position {
    source: DomainPosition,
    layer: DivergentUniverseLayerId,
    plane_ordinal: u32,
    section: SectionId,
    base: u32,
    entry: NodeId,
}

#[derive(Default)]
struct RouteContribution {
    nodes: Vec<ActivityNodeDefinition>,
    edges: Vec<ActivityEdgeDefinition>,
    programs: Vec<GraphActivityNodeProgram>,
    offers: Vec<ActivityRandomOffer>,
    bindings: Vec<LogicalScopeNodeBinding>,
}

impl Position {
    fn context(
        &self,
        area: &DivergentUniverseAreaId,
        next: NodeId,
        alternative: u32,
        preset_source: Box<str>,
        composition: DomainRoomComposition,
        level: u16,
    ) -> DomainRoomContext {
        DomainRoomContext {
            area: area.clone(),
            layer: self.layer.clone(),
            position_key: self.source.key.clone(),
            position_ordinal: self.source.ordinal,
            plane_ordinal: self.plane_ordinal,
            preset_source,
            composition,
            level,
            section: self.section,
            node_base: self.base + 100 + alternative * 64,
            edge_base: self.base + 100 + alternative * 128,
            next,
        }
    }
}

impl DivergentUniverseRuntimeFactory {
    /// Compiles every position in the selected area's explicit ordered layer
    /// list, using real Sora records. Fixed positions never draw or refill.
    /// Unspecified positions draw only from the explicitly selected source deck;
    /// equal presets share an executor but copies retain distinct card IDs.
    ///
    /// `compile_room` is trusted, deterministic mode-owned compilation, not an
    /// adapter operation injection API. Every reachable preset must return a
    /// fragment or an explicit error. No default room, candidate-pool admission,
    /// room-completion signal or successful empty fragment is synthesized.
    pub fn compile_domain_route(
        &self,
        area: &DivergentUniverseAreaId,
        deck_key: &str,
        width: u16,
        slots: DomainDeckSlots,
        mut compile_room: impl FnMut(&DomainRoomContext) -> Result<DomainRoomProgram, DomainRouteError>,
    ) -> Result<CompiledDomainRoute, DomainRouteError> {
        let deck = self
            .compile_domain_deck(deck_key, width, slots)
            .map_err(DomainRouteError::Deck)?;
        let authored = self
            .decision_catalog()
            .domain_decks()
            .iter()
            .find(|candidate| candidate.key.as_ref() == deck_key)
            .expect("deck compiler validated authored key");
        let alternatives: BTreeMap<&str, &DomainCardDefinition> = authored
            .cards
            .iter()
            .map(|card| (card.preset_source.as_ref(), card))
            .collect();
        if alternatives.len() > 64 {
            return Err(DomainRouteError::AddressOverflow);
        }
        let positions = self.route_positions(area)?;
        let mut contribution = RouteContribution::default();
        let mut rooms = Vec::new();
        for (index, position) in positions.iter().enumerate() {
            let next = positions
                .get(index + 1)
                .map_or(node(TERMINAL)?, |next| next.entry);
            match &position.source.kind {
                DomainPositionKind::Fixed {
                    kind,
                    level,
                    preset_source,
                } => {
                    let context = position.context(
                        area,
                        next,
                        0,
                        preset_source.clone(),
                        DomainRoomComposition::Fixed(*kind),
                        *level,
                    );
                    let fragment = compile_room(&context)?;
                    contribution.append_room(&context, fragment, slots)?;
                    rooms.push(context);
                }
                DomainPositionKind::Unspecified => {
                    let prepare = position.entry;
                    let draw = node(position.base + 2)?;
                    let enter_draw = edge(position.base + 1)?;
                    for current in [prepare, draw] {
                        contribution.nodes.push(activity_node(
                            current,
                            position.section,
                            ActivityNodeKind::Choice,
                        )?);
                        contribution.bindings.push(binding(
                            current,
                            ActivityNodeKind::Choice,
                            position.plane_ordinal,
                            position.source.ordinal,
                        )?);
                    }
                    contribution
                        .edges
                        .push(activity_edge(enter_draw, prepare, draw)?);
                    contribution
                        .programs
                        .push(program(prepare, deck.prepare_program(enter_draw))?);
                    let mut destinations = BTreeMap::new();
                    for (alternative_index, (preset, card)) in alternatives.iter().enumerate() {
                        let ordinal = u32::try_from(alternative_index)
                            .map_err(|_| DomainRouteError::AddressOverflow)?;
                        let context = position.context(
                            area,
                            next,
                            ordinal,
                            (*preset).into(),
                            DomainRoomComposition::Card(card.kind),
                            card.level,
                        );
                        let enter_room = edge(position.base + 10 + ordinal)?;
                        contribution.edges.push(activity_edge(
                            enter_room,
                            draw,
                            context.entry_node(),
                        )?);
                        destinations.insert(*preset, enter_room);
                        let fragment = compile_room(&context)?;
                        contribution.append_room(&context, fragment, slots)?;
                        rooms.push(context);
                    }
                    let instances = authored
                        .cards
                        .iter()
                        .map(|card| {
                            (
                                DomainCardId::new(card.instance.get())
                                    .expect("validated authored card instance"),
                                destinations[card.preset_source.as_ref()],
                            )
                        })
                        .collect::<Vec<_>>();
                    contribution.programs.push(program(
                        draw,
                        deck.offer_program_by_card(&instances)
                            .map_err(DomainRouteError::Deck)?,
                    )?);
                    contribution
                        .offers
                        .push(deck.random_offer(draw).map_err(DomainRouteError::Deck)?);
                }
            }
        }
        let last = positions.last().ok_or(DomainRouteError::UnknownArea)?;
        contribution.nodes.push(activity_node(
            node(TERMINAL)?,
            last.section,
            ActivityNodeKind::Terminal(ActivityTerminalOutcome::Completed),
        )?);
        contribution.bindings.push(
            LogicalScopeNodeBinding::new(
                node(TERMINAL)?,
                vec![address(DivergentUniverseLogicalScopeKind::Run, 1)?],
            )
            .map_err(|_| DomainRouteError::InvalidLogicalScopes)?,
        );
        let visits = contribution.nodes.iter().try_fold(0_u32, |sum, node| {
            sum.checked_add(node.maximum_visits())
                .ok_or(DomainRouteError::AddressOverflow)
        })?;
        let graph = ActivityGraphDefinition::new(
            positions[0].entry,
            contribution.nodes,
            contribution.edges,
            visits,
        )
        .map_err(DomainRouteError::Graph)?;
        let planes = last.plane_ordinal;
        let room_count =
            u32::try_from(positions.len()).map_err(|_| DomainRouteError::AddressOverflow)?;
        let kinds = [
            (DivergentUniverseLogicalScopeKind::Run, None, 1),
            (
                DivergentUniverseLogicalScopeKind::Plane,
                Some(DivergentUniverseLogicalScopeKind::Run),
                planes,
            ),
            (
                DivergentUniverseLogicalScopeKind::Node,
                Some(DivergentUniverseLogicalScopeKind::Plane),
                room_count,
            ),
            (
                DivergentUniverseLogicalScopeKind::Battle,
                Some(DivergentUniverseLogicalScopeKind::Node),
                visits,
            ),
        ];
        let classes = kinds
            .into_iter()
            .map(|(kind, parent, maximum)| {
                LogicalScopeClassDefinition::new(
                    kind.class_id(),
                    parent.map(DivergentUniverseLogicalScopeKind::class_id),
                    maximum,
                )
                .ok_or(DomainRouteError::InvalidLogicalScopes)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let logical_scopes = LogicalScopeDefinitions::new(classes, contribution.bindings)
            .map_err(|_| DomainRouteError::InvalidLogicalScopes)?;
        Ok(CompiledDomainRoute {
            graph,
            programs: contribution.programs,
            random_offers: contribution.offers,
            logical_scopes,
            deck,
            rooms: rooms.into(),
        })
    }

    fn route_positions(
        &self,
        area: &DivergentUniverseAreaId,
    ) -> Result<Vec<Position>, DomainRouteError> {
        let selected = self
            .bundle
            .catalog()
            .area(area)
            .ok_or(DomainRouteError::UnknownArea)?;
        let mut result = Vec::new();
        for (plane_index, layer) in selected.layers.iter().enumerate() {
            let layout = self
                .decision_catalog()
                .domain_layout()
                .iter()
                .find(|layout| &layout.layer == layer)
                .ok_or_else(|| DomainRouteError::MissingLayer(layer.clone()))?;
            let plane_ordinal =
                u32::try_from(plane_index + 1).map_err(|_| DomainRouteError::AddressOverflow)?;
            let section = SectionId::new(plane_ordinal).ok_or(DomainRouteError::AddressOverflow)?;
            for source in &layout.positions {
                let ordinal = u32::try_from(result.len() + 1)
                    .map_err(|_| DomainRouteError::AddressOverflow)?;
                let base = ordinal
                    .checked_mul(10_000)
                    .filter(|base| *base < TERMINAL - 10_000)
                    .ok_or(DomainRouteError::AddressOverflow)?;
                let entry = match &source.kind {
                    DomainPositionKind::Fixed { .. } => node(base + 100)?,
                    DomainPositionKind::Unspecified => node(base + 1)?,
                };
                result.push(Position {
                    source: source.clone(),
                    base,
                    entry,
                    layer: layer.clone(),
                    plane_ordinal,
                    section,
                });
            }
        }
        Ok(result)
    }
}

// Collecting one executable contribution is deliberately separate from source
// selection. This function does not grant any room-completion signals.
impl RouteContribution {
    fn append_room(
        &mut self,
        context: &DomainRoomContext,
        fragment: DomainRoomProgram,
        slots: DomainDeckSlots,
    ) -> Result<(), DomainRouteError> {
        validate_fragment(context, &fragment, slots)?;
        for current in &fragment.nodes {
            self.bindings.push(binding(
                current.id(),
                current.kind(),
                context.plane_ordinal,
                context.position_ordinal,
            )?);
        }
        self.edges.push(activity_edge(
            context.exit_edge(),
            fragment.exit_node,
            context.next,
        )?);
        self.nodes.extend(fragment.nodes);
        self.edges.extend(fragment.edges);
        self.programs.extend(fragment.programs);
        self.offers.extend(fragment.random_offers);
        Ok(())
    }
}

fn binding(
    node: NodeId,
    kind: ActivityNodeKind,
    plane_ordinal: u32,
    position_ordinal: u16,
) -> Result<LogicalScopeNodeBinding, DomainRouteError> {
    let mut path = vec![
        address(DivergentUniverseLogicalScopeKind::Run, 1)?,
        address(
            DivergentUniverseLogicalScopeKind::Plane,
            u64::from(plane_ordinal),
        )?,
        address(
            DivergentUniverseLogicalScopeKind::Node,
            u64::from(position_ordinal),
        )?,
    ];
    if kind == ActivityNodeKind::Battle {
        path.push(address(
            DivergentUniverseLogicalScopeKind::Battle,
            u64::from(node.get()),
        )?);
    }
    LogicalScopeNodeBinding::new(node, path).map_err(|_| DomainRouteError::InvalidLogicalScopes)
}
fn address(
    kind: DivergentUniverseLogicalScopeKind,
    key: u64,
) -> Result<LogicalScopeAddress, DomainRouteError> {
    LogicalScopeAddress::new(kind.class_id(), key).ok_or(DomainRouteError::InvalidLogicalScopes)
}
fn node(raw: u32) -> Result<NodeId, DomainRouteError> {
    NodeId::new(raw).ok_or(DomainRouteError::AddressOverflow)
}
fn edge(raw: u32) -> Result<ActivityEdgeId, DomainRouteError> {
    ActivityEdgeId::new(raw).ok_or(DomainRouteError::AddressOverflow)
}
fn activity_node(
    id: NodeId,
    section: SectionId,
    kind: ActivityNodeKind,
) -> Result<ActivityNodeDefinition, DomainRouteError> {
    ActivityNodeDefinition::new(id, section, kind, 1).map_err(DomainRouteError::Graph)
}
fn activity_edge(
    id: ActivityEdgeId,
    from: NodeId,
    to: NodeId,
) -> Result<ActivityEdgeDefinition, DomainRouteError> {
    ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::Always, 0, 1)
        .map_err(DomainRouteError::Graph)
}
fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, DomainRouteError> {
    let id = ActivityProgramId::new(node.get()).ok_or(DomainRouteError::AddressOverflow)?;
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(id, operations).map_err(DomainRouteError::Program)?,
    ))
}
