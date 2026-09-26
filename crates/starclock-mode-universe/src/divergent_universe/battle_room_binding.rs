//! Exact fragment/profile binding for the existing battle and reward executors.

use std::{collections::BTreeSet, sync::Arc};

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseFlowInstance, DivergentUniverseLogicalScopeKind, DivergentUniverseRoomPolicy,
    DivergentUniverseRuntimeFactory,
    battle_room::{BattleRoomError, CompiledBattleRoom},
    domain_choices::set_domain,
    state::BATTLE_DOMAIN_SLOT,
    tawot_room::{BoundTawotRoom, CompiledTawotRoom},
};
use starclock_activity::{
    ActivityConfigDigest, ActivityDecisionKind, ActivityDefinitionDigest,
    ActivityDefinitionIdentity, ActivityEdgeCondition, ActivityExpression, ActivityGraphDefinition,
    ActivityNodeKind, ActivityOperation, ActivityPlayerView, GraphActivity,
    GraphActivityCommandError, GraphActivityDefinition,
};
use starclock_data::{
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId,
};

#[derive(Clone, Debug)]
pub(in crate::divergent_universe) struct BoundBattleRooms {
    rooms: Vec<CompiledBattleRoom>,
    services: Vec<BoundTawotRoom>,
    definition: Arc<GraphActivityDefinition>,
}

impl DivergentUniverseRuntimeFactory {
    /// Binds the base entry, current source/decision digests, graph and every
    /// sorted explicit battle fragment. `payload` must bind the owning profile's
    /// deck/width, non-battle programs, policies and remaining immutable inputs;
    /// it is not an inference of source admission or encoded replay support.
    pub fn battle_room_identity(
        &self,
        base: &DivergentUniverseFlowInstance,
        graph: &ActivityGraphDefinition,
        rooms: &[CompiledBattleRoom],
        payload: ActivityConfigDigest,
    ) -> Result<ActivityDefinitionIdentity, BattleRoomError> {
        self.position_room_identity(base, graph, rooms, &[], payload)
    }

    /// Exact combined battle/Tawot identity. Every independently selected
    /// service level and placement is bound; list ordering is not significant.
    /// `payload` still owns all other non-battle programs and deck policies.
    pub fn position_room_identity(
        &self,
        base: &DivergentUniverseFlowInstance,
        graph: &ActivityGraphDefinition,
        rooms: &[CompiledBattleRoom],
        services: &[CompiledTawotRoom],
        payload: ActivityConfigDigest,
    ) -> Result<ActivityDefinitionIdentity, BattleRoomError> {
        let mut rooms = rooms.iter().collect::<Vec<_>>();
        rooms.sort_by_key(|room| room.context.entry_node());
        let mut services = services.iter().collect::<Vec<_>>();
        services.sort_by_key(|room| room.context().entry_node());
        if rooms.is_empty()
            || rooms
                .windows(2)
                .any(|pair| pair[0].context.entry_node() == pair[1].context.entry_node())
            || base.component_digest != self.bundle_identity().component_digest().bytes()
            || rooms.iter().any(|room| {
                room.component != base.component_digest
                    || room.decisions != self.decision_catalog().digest()
            })
            || services
                .windows(2)
                .any(|pair| pair[0].context().entry_node() == pair[1].context().entry_node())
            || services.iter().any(|room| !room.matches_factory(self))
        {
            return Err(BattleRoomError::InvalidDefinition);
        }
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.divergent-universe.position-battles.definition.v1");
        hash.update(base.definition().identity().definition_digest().bytes());
        hash.update(graph.digest().bytes());
        for room in &rooms {
            hash.update(room.configuration_digest());
        }
        if !services.is_empty() {
            hash.update(b"tawot");
            hash.update(
                u32::try_from(services.len())
                    .map_err(|_| BattleRoomError::InvalidDefinition)?
                    .to_le_bytes(),
            );
            for service in &services {
                hash.update(service.configuration_digest());
            }
        }
        let definition = ActivityDefinitionDigest::new(hash.finalize())
            .ok_or(BattleRoomError::InvalidDefinition)?;
        let mut hash = CanonicalDigestBuilder::new();
        hash.update(b"starclock.divergent-universe.position-battles.config.v1");
        hash.update(base.definition().identity().config_digest().bytes());
        hash.update(self.bundle_identity().component_digest().bytes());
        hash.update(self.decision_catalog().digest());
        hash.update(definition.bytes());
        hash.update(payload.bytes());
        let config =
            ActivityConfigDigest::new(hash.finalize()).ok_or(BattleRoomError::InvalidDefinition)?;
        Ok(ActivityDefinitionIdentity::new(
            base.definition().identity().id(),
            definition,
            config,
        ))
    }

    /// Compiles an immutable source-position profile over a plain mapped
    /// `with_runtime_battle_route` entry. No live Activity is rebound. All Battle
    /// nodes must be accounted for exactly once; changed fragment programs,
    /// scopes, continuations or runtime state declarations reject. Legacy initial
    /// occurrence/equation/service/deck and per-layer handlers cannot be grafted
    /// onto foreign addresses. The caller still supplies every non-battle room.
    pub fn bind_battle_rooms(
        &self,
        base: DivergentUniverseFlowInstance,
        definition: Arc<GraphActivityDefinition>,
        rooms: &[CompiledBattleRoom],
        payload: ActivityConfigDigest,
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        self.bind_position_rooms(base, definition, rooms, &[], payload)
    }

    /// Validates one immutable profile with exactly bound battle and Tawot
    /// fragments. Only the four supplied service declarations may replace base
    /// declarations, and only with their exact logical-room policies. All other
    /// state, participants and battle checks are unchanged. No live rebinding or
    /// automatic source admission is performed. Service commands are exposed by
    /// the existing flow observation/selection APIs and shared transaction engine.
    pub fn bind_position_rooms(
        &self,
        mut base: DivergentUniverseFlowInstance,
        definition: Arc<GraphActivityDefinition>,
        rooms: &[CompiledBattleRoom],
        services: &[CompiledTawotRoom],
        payload: ActivityConfigDigest,
    ) -> Result<DivergentUniverseFlowInstance, BattleRoomError> {
        if !base.has_runtime_battle_route()
            || base.is_first_ordinary_vertical_slice()
            || base.position_battles.is_some()
            || base.encounter_pool.is_some()
            || base.source_deck_selection.is_some()
            || base.tawot_service.is_some()
            || base.occurrence_binding.is_some()
            || base.initial_equations.is_some()
            || base.evolution_events.is_some()
        {
            return Err(BattleRoomError::UnsupportedEntry);
        }
        if definition.identity()
            != self.position_room_identity(&base, definition.graph(), rooms, services, payload)?
            || definition.participants().as_ref() != base.definition().participants().as_ref()
            || definition.interactions().is_some()
            || base
                .definition()
                .state_definition()
                .slots()
                .iter()
                .any(|slot| {
                    let expected = services
                        .first()
                        .and_then(|room| {
                            room.slot_definitions()
                                .iter()
                                .find(|candidate| candidate.id() == slot.id())
                        })
                        .unwrap_or(slot);
                    !definition.state_definition().slots().contains(expected)
                })
            || definition.state_definition().inventories()
                != base.definition().state_definition().inventories()
            || definition.state_definition().modifiers()
                != base.definition().state_definition().modifiers()
        {
            return Err(BattleRoomError::InvalidDefinition);
        }
        let mut services = services.iter().collect::<Vec<_>>();
        services.sort_by_key(|room| room.context().entry_node());
        let services = services
            .into_iter()
            .map(|room| {
                let context = room.context();
                let plane = context
                    .plane_ordinal
                    .checked_sub(1)
                    .and_then(|value| usize::try_from(value).ok());
                if context.area != *base.area()
                    || plane.and_then(|index| base.layers().get(index)) != Some(&context.layer)
                {
                    return Err(BattleRoomError::InvalidContext);
                }
                room.bind(Arc::clone(&definition))
                    .map_err(BattleRoomError::Service)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut bound = rooms.to_vec();
        bound.sort_by_key(|room| room.context.entry_node());
        for room in &bound {
            validate_room(&base, &definition, room)?;
        }
        let battles = bound
            .iter()
            .map(|room| room.battle)
            .collect::<BTreeSet<_>>();
        if battles.len() != bound.len()
            || definition
                .graph()
                .nodes()
                .iter()
                .filter(|node| node.kind() == ActivityNodeKind::Battle)
                .map(|node| node.id())
                .collect::<BTreeSet<_>>()
                != battles
        {
            return Err(BattleRoomError::InvalidDefinition);
        }
        base.position_battles = Some(Arc::new(BoundBattleRooms {
            rooms: bound,
            services,
            definition: Arc::clone(&definition),
        }));
        base.definition = definition;
        base.room_policy =
            DivergentUniverseRoomPolicy::ExplicitSourcePositionProgramsNoAutomaticAdmission;
        Ok(base)
    }
}

fn validate_room(
    base: &DivergentUniverseFlowInstance,
    definition: &GraphActivityDefinition,
    room: &CompiledBattleRoom,
) -> Result<(), BattleRoomError> {
    let context = &room.context;
    let plane = context
        .plane_ordinal
        .checked_sub(1)
        .and_then(|value| usize::try_from(value).ok());
    if context.area != *base.area()
        || plane.and_then(|index| base.layers().get(index)) != Some(&context.layer)
    {
        return Err(BattleRoomError::InvalidContext);
    }
    let owns = |node| {
        room.fragment
            .nodes
            .iter()
            .any(|candidate| candidate.id() == node)
    };
    let graph = definition.graph();
    let exit = graph
        .edges()
        .iter()
        .find(|edge| edge.id() == context.exit_edge());
    if exit.is_none_or(|edge| {
        edge.from() != room.reward
            || edge.to() != context.successor()
            || edge.condition() != ActivityEdgeCondition::Always
            || edge.maximum_traversals() != 1
    }) || room
        .fragment
        .nodes
        .iter()
        .any(|node| graph.node(node.id()) != Some(node))
        || room
            .fragment
            .edges
            .iter()
            .any(|edge| !graph.edges().contains(edge))
        || room.fragment.programs.iter().any(|program| {
            let expected = if program.node() == context.entry_node() {
                &room.entry_program
            } else {
                program
            };
            !definition.programs().contains(expected)
        })
        || graph.edges().iter().any(|edge| {
            (owns(edge.from())
                && edge.id() != context.exit_edge()
                && !room.fragment.edges.contains(edge))
                || (!owns(edge.from()) && owns(edge.to()) && edge.to() != context.entry_node())
        })
        || definition
            .random_offers()
            .iter()
            .any(|offer| owns(offer.node()))
        || definition
            .random_checkpoints()
            .iter()
            .any(|checkpoint| owns(checkpoint.node()))
    {
        return Err(BattleRoomError::InvalidDefinition);
    }
    let scopes = definition.state_definition().logical_scopes();
    for node in &room.fragment.nodes {
        let path = scopes
            .bindings()
            .iter()
            .find(|binding| binding.node() == node.id())
            .ok_or(BattleRoomError::InvalidDefinition)?
            .path();
        let is_battle = node.id() == room.battle;
        if path.len() != if is_battle { 4 } else { 3 }
            || path[0].class() != DivergentUniverseLogicalScopeKind::Run.class_id()
            || path[0].key() != 1
            || path[1].class() != DivergentUniverseLogicalScopeKind::Plane.class_id()
            || path[1].key() != u64::from(context.plane_ordinal)
            || path[2].class() != DivergentUniverseLogicalScopeKind::Node.class_id()
            || path[2].key() != u64::from(context.position_ordinal)
            || (is_battle
                && (path[3].class() != DivergentUniverseLogicalScopeKind::Battle.class_id()
                    || path[3].key() != u64::from(node.id().get())))
        {
            return Err(BattleRoomError::InvalidDefinition);
        }
    }
    Ok(())
}

impl BoundBattleRooms {
    pub(in crate::divergent_universe) fn tawot(
        &self,
        activity: &GraphActivity,
    ) -> Option<&BoundTawotRoom> {
        self.services
            .iter()
            .find(|room| room.offered(activity).is_some())
    }

    pub(in crate::divergent_universe) fn matches(&self, activity: &GraphActivity) -> bool {
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
                && actual.interactions().is_none())
    }

    pub(in crate::divergent_universe) fn offered(
        &self,
        flow: &DivergentUniverseFlowInstance,
        activity: &GraphActivity,
    ) -> Result<(&DivergentUniverseEncounterGroupId, &str), GraphActivityCommandError> {
        let invalid = || GraphActivityCommandError::DecisionNotOffered;
        let view = activity.player_view();
        let room = self
            .rooms
            .iter()
            .find(|room| room.encounter == view.current_node())
            .ok_or_else(invalid)?;
        let decision = view.decision().ok_or_else(invalid)?;
        if !self.matches(activity)
            || decision.kind() != ActivityDecisionKind::Encounter
            || decision.options().len() != 1
            || decision.options()[0].id().get() != 1
            || flow.encounter_destination(view.current_node())
                != Some((room.battle, room.context.section))
            || self.domain(&view)? != Some(room.selection.domain)
        {
            return Err(invalid());
        }
        Ok((&room.selection.group, &room.selection.stage))
    }

    pub(in crate::divergent_universe) fn domain(
        &self,
        view: &ActivityPlayerView,
    ) -> Result<Option<BattleRewardDomain>, GraphActivityCommandError> {
        let Some(room) = self.rooms.iter().find(|room| {
            room.fragment
                .nodes
                .iter()
                .any(|node| node.id() == view.current_node())
        }) else {
            return Ok(None);
        };
        let expected = match set_domain(Some(room.selection.domain)) {
            ActivityOperation::SetSlot {
                value: ActivityExpression::Literal(value),
                ..
            } => value,
            _ => return Err(GraphActivityCommandError::DecisionNotOffered),
        };
        if view
            .slots()
            .iter()
            .any(|slot| slot.id() == BATTLE_DOMAIN_SLOT && slot.value() == &expected)
        {
            Ok(Some(room.selection.domain))
        } else {
            Err(GraphActivityCommandError::DecisionNotOffered)
        }
    }
}
