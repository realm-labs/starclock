//! Fixed Conversion admission and explicit replaceable battle-substitute policy.

use crate::divergent_universe::{
    battle_room::{
        BattleRoomCompiler, BattleRoomError, BattleRoomSequenceLength, BattleRoomSequencePolicy,
        CompiledBattleRoom,
    },
    domain_route::{DomainRoomComposition, DomainRoomContext, DomainRouteError},
};
use starclock_activity::{ActivityEdgeCondition, ActivityEdgeDefinition, TerminalOutcome};
use starclock_data::divergent_universe_domain_layout::FixedDomainKind;

impl BattleRoomCompiler {
    /// Compiles only a currently joined fixed preset 1004 / Conversion / level 1.
    /// Missing original waves and reward tiers use an explicit one-through-four
    /// same-candidate battle substitute with the caller's reviewed reward domain.
    /// Every victory settles normally. A verified loss skips remaining battles
    /// and exits this room without victory rewards, retry, revival or healing;
    /// faults remain terminal. Actual enemy waves inside each battle still belong
    /// to combat. This is not original wave-count, partial-wave reward or admission
    /// parity. Invalid contexts construct nothing and consume no RNG.
    pub fn compile_conversion_sequence(
        &self,
        context: &DomainRoomContext,
        length: BattleRoomSequenceLength,
    ) -> Result<CompiledBattleRoom, BattleRoomError> {
        if !self.factory.room_context_matches(context)
            || context.composition != DomainRoomComposition::Fixed(FixedDomainKind::Conversion)
            || context.preset_source.as_ref() != "1004"
            || context.level != 1
        {
            return Err(BattleRoomError::InvalidContext);
        }
        let mut room = self.compile_sequence(context, length)?;
        for edge in &mut room.fragment.edges {
            if edge.condition() == ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed) {
                *edge = ActivityEdgeDefinition::new(
                    edge.id(),
                    edge.from(),
                    room.reward,
                    edge.condition(),
                    edge.priority(),
                    edge.maximum_traversals(),
                )
                .map_err(DomainRouteError::Graph)
                .map_err(BattleRoomError::Route)?;
            }
        }
        // The ordinary defeat terminal is no longer reachable. Keep only the
        // independently reachable fault terminal, not a misleading failure node.
        let failed = context.node(4).map_err(BattleRoomError::Route)?;
        room.fragment.nodes.retain(|node| node.id() != failed);
        room.sequence_policy = BattleRoomSequencePolicy::VersionedProjectPolicyConversionSameCandidateSequenceLossEndsRoomWithoutHealing;
        Ok(room)
    }
}
