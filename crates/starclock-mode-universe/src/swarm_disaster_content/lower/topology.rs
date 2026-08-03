use crate::swarm_disaster_content::SwarmDisasterContentErrorKind;
use crate::swarm_disaster_generated::{
    SoraConfig, swarm_disaster_block_create_rule::SwarmDisasterBlockCreateRule,
    swarm_disaster_map_event::SwarmDisasterMapEvent,
    swarm_disaster_topology_consequence::SwarmDisasterTopologyConsequence,
};

use super::{json, metadata, nonempty, nonnegative_u16, positive, scalar, stable};
use crate::swarm_disaster_content::{SwarmDisasterContentError, types::*};

pub(super) type TopologyTables = (
    Box<[MapEventDefinition]>,
    Box<[BlockRuleDefinition]>,
    Box<[TopologyConsequenceDefinition]>,
);

pub(super) fn lower(source: &SoraConfig) -> Result<TopologyTables, SwarmDisasterContentError> {
    Ok((
        source
            .swarm_disaster_map_event()
            .ordered_rows()
            .map(map_event)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        source
            .swarm_disaster_block_create_rule()
            .ordered_rows()
            .map(block_rule)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
        source
            .swarm_disaster_topology_consequence()
            .ordered_rows()
            .map(topology_consequence)
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice(),
    ))
}

fn map_event(row: &SwarmDisasterMapEvent) -> Result<MapEventDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(MapEventDefinition {
        id: MapEventId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        chessboard_id: positive(row.chessboard_id, &row.stable_key)?,
        trigger: json(&row.trigger_json, &row.stable_key)?,
        weight: scalar(&row.weight, &row.stable_key)?,
        operations: json(&row.ordered_effects_json, &row.stable_key)?,
    })
}

fn block_rule(
    row: &SwarmDisasterBlockCreateRule,
) -> Result<BlockRuleDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(BlockRuleDefinition {
        id: BlockRuleId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        chessboard_id: positive(row.chessboard_id, &row.stable_key)?,
        group: nonempty(&row.group_id, &row.stable_key)?,
        domain_id: positive(row.domain_id, &row.stable_key)?,
        order: nonnegative_u16(row.order, &row.stable_key)?,
        count: json(&row.count_json, &row.stable_key)?,
        candidates: json(&row.mark_candidates_json, &row.stable_key)?,
    })
}

fn topology_consequence(
    row: &SwarmDisasterTopologyConsequence,
) -> Result<TopologyConsequenceDefinition, SwarmDisasterContentError> {
    metadata(&row.stable_key, &row.schema_revision, &row.kind)?;
    Ok(TopologyConsequenceDefinition {
        id: TopologyConsequenceId(positive(row.id, &row.stable_key)?),
        key: stable(&row.stable_key, &row.stable_key)?,
        trigger_kind: nonempty(&row.trigger_kind, &row.stable_key)?,
        scope: nonempty(&row.scope, &row.stable_key)?,
        operations: json(&row.ordered_operations_json, &row.stable_key)?,
        audience_die_id: row.aeon_dice_id.parse::<u32>().map_err(|_| {
            super::error(SwarmDisasterContentErrorKind::Identifier, &row.stable_key)
        })?,
        active_stage: nonnegative_u16(row.active_stage, &row.stable_key)?,
    })
}
