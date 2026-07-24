//! Focused lookup, weighting and logical-scope helpers for topology lowering.

use starclock_activity::{
    ActivityGraphDefinition, LogicalScopeAddress, LogicalScopeClassDefinition, LogicalScopeClassId,
    LogicalScopeDefinitions, LogicalScopeNodeBinding,
};

use crate::{
    catalog::UniverseCatalog,
    id::TopologyNodeId,
    occurrence::OccurrenceDefinition,
    path::ExactParameter,
    topology::{
        DomainHubDefinition, ResolvedRoomContent, STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS,
        UniverseTopologyCompileError,
    },
};

pub(super) fn domain_logical_scopes(
    graph: &ActivityGraphDefinition,
    hubs: &[DomainHubDefinition],
) -> Result<LogicalScopeDefinitions, UniverseTopologyCompileError> {
    let class = LogicalScopeClassId::new(STANDARD_UNIVERSE_DOMAIN_VISIT_CLASS)
        .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    let class_definition =
        LogicalScopeClassDefinition::new(class, None, graph.maximum_total_visits())
            .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
    let mut bindings = Vec::with_capacity(hubs.len().saturating_mul(7));
    for hub in hubs {
        let address = LogicalScopeAddress::new(class, u64::from(hub.source_node().get()))
            .ok_or(UniverseTopologyCompileError::InvalidGraph)?;
        for node in [
            hub.node(),
            hub.content_node(),
            hub.member_node(),
            hub.battle_node(),
            hub.reward_node(),
            hub.formation_node(),
            hub.route_node(),
        ] {
            bindings.push(
                LogicalScopeNodeBinding::new(node, vec![address])
                    .map_err(|_| UniverseTopologyCompileError::InvalidGraph)?,
            );
        }
    }
    LogicalScopeDefinitions::new(vec![class_definition], bindings)
        .map_err(|_| UniverseTopologyCompileError::InvalidGraph)
}

pub(super) fn resolve_rooms(
    catalog: &UniverseCatalog,
    source_node: u32,
) -> Result<Box<[ResolvedRoomContent]>, UniverseTopologyCompileError> {
    let mut resolved = Vec::new();
    for room in catalog
        .rooms()
        .iter()
        .filter(|room| room_is_eligible(room.section_ids(), source_node))
    {
        let mut bindings = catalog.room_content().iter().filter(|binding| {
            binding.room() == room.id() && binding.condition_key() == room.source_group_id()
        });
        let binding =
            bindings
                .next()
                .ok_or(UniverseTopologyCompileError::MissingPrimaryRoomContent(
                    room.id(),
                ))?;
        if bindings.next().is_some() {
            return Err(UniverseTopologyCompileError::AmbiguousPrimaryRoomContent(
                room.id(),
            ));
        }
        resolved.push(ResolvedRoomContent::new(
            room.id(),
            binding.kind(),
            binding.encounter_group(),
            binding.source_content_id(),
        ));
    }
    if resolved.is_empty() {
        return Err(UniverseTopologyCompileError::NoEligibleRoom(
            TopologyNodeId::new(source_node).ok_or(UniverseTopologyCompileError::InvalidGraph)?,
        ));
    }
    Ok(resolved.into_boxed_slice())
}

pub(super) fn exact_weight(value: ExactParameter) -> Result<u64, UniverseTopologyCompileError> {
    if value.coefficient() <= 0 || value.scale() > 6 {
        return Err(UniverseTopologyCompileError::InvalidEncounterWeight);
    }
    let multiplier = 10_u64
        .checked_pow(u32::from(6 - value.scale()))
        .ok_or(UniverseTopologyCompileError::InvalidEncounterWeight)?;
    u64::try_from(value.coefficient())
        .ok()
        .and_then(|coefficient| coefficient.checked_mul(multiplier))
        .ok_or(UniverseTopologyCompileError::InvalidEncounterWeight)
}

pub(super) fn occurrence_for_source<'a>(
    catalog: &'a UniverseCatalog,
    source: &str,
) -> Option<&'a OccurrenceDefinition> {
    let variant = catalog.occurrence_variants().iter().find(|variant| {
        variant.stable_key() == source
            || variant
                .stable_key()
                .rsplit_once('.')
                .is_some_and(|(_, suffix)| suffix == source)
    })?;
    catalog.occurrence(variant.occurrence())
}

fn room_is_eligible(section_ids: &[u32], source_node: u32) -> bool {
    section_ids.is_empty() || section_ids.contains(&0) || section_ids.contains(&source_node)
}
