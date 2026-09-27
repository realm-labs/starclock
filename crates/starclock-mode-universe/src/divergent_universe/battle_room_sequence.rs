//! Required repeated handoffs within the same logical room, using shared settlement.

use crate::divergent_universe::{
    battle_room::{
        BattleRoomCompiler, BattleRoomError, BattleRoomHandoff, BattleRoomSequenceLength,
        CompiledBattleRoom, literal,
    },
    domain_choices::set_domain,
    domain_route::{DomainRoomContext, DomainRouteError},
    state::{BATTLE_BLESSING_ACCEPTED_SLOT, BATTLE_BLESSING_CANDIDATES_SLOT},
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityValue, GraphActivityNodeProgram, NodeId, TerminalOutcome,
};

impl BattleRoomCompiler {
    /// Compiles one through four required sequential battles in the same room.
    /// All use this compiler's explicit stage/group/domain. Only the first entry
    /// runs Curio domain operations; rewards and verified carry execute per battle.
    /// Intermediate reward acceptance/suppression advances to the next encounter.
    /// Loss/fault terminates the sequence without future rewards. No source enemy
    /// object-to-battle mapping, room count, original admission or early Leave is inferred.
    pub fn compile_sequence(
        &self,
        context: &DomainRoomContext,
        length: BattleRoomSequenceLength,
    ) -> Result<CompiledBattleRoom, BattleRoomError> {
        let mut room = self.compile(context)?;
        let mut handoffs = room.handoffs.into_vec();
        let marker = set_domain(Some(self.selection.domain));
        let ActivityOperation::SetSlot { slot, value } = marker else {
            return Err(BattleRoomError::InvalidDefinition);
        };
        for index in 1..length.get() {
            let offset = index
                .checked_mul(3)
                .and_then(|offset| offset.checked_add(3))
                .ok_or(BattleRoomError::InvalidSequenceLength)?;
            let encounter = context.node(offset).map_err(BattleRoomError::Route)?;
            let battle = context
                .node(address_offset(offset, 1)?)
                .map_err(BattleRoomError::Route)?;
            let reward = context
                .node(address_offset(offset, 2)?)
                .map_err(BattleRoomError::Route)?;
            for (node, kind) in [
                (encounter, ActivityNodeKind::Choice),
                (battle, ActivityNodeKind::Battle),
                (reward, ActivityNodeKind::Reward),
            ] {
                room.fragment.nodes.push(
                    ActivityNodeDefinition::new(node, context.section, kind, 1)
                        .map_err(DomainRouteError::Graph)
                        .map_err(BattleRoomError::Route)?,
                );
            }
            let edge_base = index
                .checked_mul(5)
                .ok_or(BattleRoomError::InvalidSequenceLength)?;
            let edges = [
                (room.reward, encounter, ActivityEdgeCondition::Always),
                (encounter, battle, ActivityEdgeCondition::Always),
                (
                    battle,
                    reward,
                    ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete),
                ),
                (
                    battle,
                    context.node(4).map_err(BattleRoomError::Route)?,
                    ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed),
                ),
                (
                    battle,
                    context.node(5).map_err(BattleRoomError::Route)?,
                    ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Faulted),
                ),
            ];
            for (delta, (from, to, condition)) in edges.into_iter().enumerate() {
                let delta = u16::try_from(delta).map_err(|_| BattleRoomError::InvalidDefinition)?;
                room.fragment.edges.push(
                    ActivityEdgeDefinition::new(
                        context
                            .edge(address_offset(edge_base, delta)?)
                            .map_err(BattleRoomError::Route)?,
                        from,
                        to,
                        condition,
                        0,
                        1,
                    )
                    .map_err(DomainRouteError::Graph)
                    .map_err(BattleRoomError::Route)?,
                );
            }
            let continuation = context.edge(edge_base).map_err(BattleRoomError::Route)?;
            let previous = room
                .fragment
                .programs
                .iter_mut()
                .find(|program| program.node() == room.reward)
                .ok_or(BattleRoomError::InvalidDefinition)?;
            *previous = program(room.reward, self.rewards.node_program(continuation))?;
            room.fragment.programs.extend([
                program(
                    encounter,
                    vec![
                        ActivityOperation::SetSlot {
                            slot: BATTLE_BLESSING_ACCEPTED_SLOT,
                            value: literal(ActivityValue::Boolean(false)),
                        },
                        ActivityOperation::SetOrderedIdSet {
                            slot: BATTLE_BLESSING_CANDIDATES_SLOT,
                            values: Vec::new().into_boxed_slice(),
                        },
                        ActivityOperation::Offer {
                            kind: ActivityDecisionKind::Encounter,
                            options: vec![ActivityOptionDefinition::new(
                                ActivityOptionId::new(1).expect("fixed nonzero engagement"),
                                0,
                                ActivityCondition::Boolean(literal(ActivityValue::Boolean(true))),
                                vec![
                                    ActivityOperation::Require(ActivityCondition::Equal(
                                        ActivityExpression::Slot(slot),
                                        value.clone(),
                                    )),
                                    ActivityOperation::Traverse(
                                        context
                                            .edge(address_offset(edge_base, 1)?)
                                            .map_err(BattleRoomError::Route)?,
                                    ),
                                ],
                            )]
                            .into_boxed_slice(),
                        },
                    ],
                )?,
                program(battle, Vec::new())?,
                program(reward, self.rewards.node_program(context.exit_edge()))?,
            ]);
            handoffs.push(BattleRoomHandoff { encounter, battle });
            room.reward = reward;
            room.fragment.exit_node = reward;
        }
        room.handoffs = handoffs.into_boxed_slice();
        room.sequence_length = length;
        Ok(room)
    }
}

fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, BattleRoomError> {
    let id = ActivityProgramId::new(node.get()).ok_or(BattleRoomError::InvalidDefinition)?;
    ActivityProgramDefinition::new(id, operations)
        .map(|program| GraphActivityNodeProgram::new(node, program))
        .map_err(DomainRouteError::Program)
        .map_err(BattleRoomError::Route)
}

fn address_offset(base: u16, delta: u16) -> Result<u16, BattleRoomError> {
    base.checked_add(delta)
        .ok_or(BattleRoomError::InvalidSequenceLength)
}
