//! One explicitly placed encounter/Battle/reward fragment at a source position.
//! Selection is an existing explicit stage-candidate policy, NOT a source room
//! selector or a claim of exact original boss membership or enemy execution.

use std::sync::Arc;

#[path = "battle_room_binding.rs"]
mod binding;
pub(super) use binding::BoundBattleRooms;

use crate::digest::CanonicalDigestBuilder;
use crate::divergent_universe::{
    DivergentUniverseCurioRuntime, DivergentUniverseEntryFlowError,
    DivergentUniverseRuntimeFactory,
    battle_blessings::BattleBlessings,
    domain_choices::set_domain,
    domain_route::{DomainRoomContext, DomainRoomProgram, DomainRouteError},
    occurrence_room::OccurrenceRoomError,
    state::{LAYER_SEQUENCE_SLOT, LAYER_SLOT},
    tawot_room::TawotRoomError,
};
use starclock_activity::{
    ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityExpression, ActivityNodeDefinition, ActivityNodeKind, ActivityOperation,
    ActivityOptionDefinition, ActivityOptionId, ActivityProgramDefinition, ActivityProgramId,
    ActivityTerminalOutcome, ActivityValue, GraphActivityNodeProgram, NodeId, TerminalOutcome,
};
use starclock_data::{
    divergent_universe_decisions::BattleRewardDomain,
    divergent_universe_encounter_catalog::DivergentUniverseEncounterGroupId,
};

/// An explicit caller-selected reviewed candidate and independent reward domain.
/// This does not infer stage eligibility from a preset's kind or level.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BattleRoomSelection {
    pub group: DivergentUniverseEncounterGroupId,
    pub stage: Box<str>,
    pub domain: BattleRewardDomain,
}

/// Immutable current catalogs reused across position alternatives.
#[derive(Clone, Debug)]
pub struct BattleRoomCompiler {
    factory: DivergentUniverseRuntimeFactory,
    selection: BattleRoomSelection,
    rewards: Arc<BattleBlessings>,
    curios: Arc<DivergentUniverseCurioRuntime>,
}

/// One bounded fragment plus its exact configuration and encounter binding.
#[derive(Clone, Debug)]
pub struct CompiledBattleRoom {
    context: DomainRoomContext,
    selection: BattleRoomSelection,
    fragment: DomainRoomProgram,
    entry_program: GraphActivityNodeProgram,
    component: [u8; 32],
    decisions: [u8; 32],
    encounter: NodeId,
    battle: NodeId,
    reward: NodeId,
}

#[derive(Debug)]
pub enum BattleRoomError {
    Entry(DivergentUniverseEntryFlowError),
    Route(DomainRouteError),
    Service(TawotRoomError),
    Occurrence(OccurrenceRoomError),
    InvalidEncounter,
    UnsupportedRewardDomain,
    InvalidContext,
    InvalidDefinition,
    UnsupportedEntry,
}
impl std::fmt::Display for BattleRoomError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Divergent Universe battle room: {self:?}")
    }
}
impl std::error::Error for BattleRoomError {}

impl DivergentUniverseRuntimeFactory {
    /// Validates candidate-group membership and an executable authored reward.
    /// Missing rewards reject before construction, never fall
    /// back. Placement, battle count and domain are explicit caller inputs,
    /// never inferred original room rules. Failure has no state/RNG/cache effects.
    pub fn battle_room_compiler(
        &self,
        selection: BattleRoomSelection,
    ) -> Result<BattleRoomCompiler, BattleRoomError> {
        if !self
            .decision_catalog()
            .battle_fragments()
            .iter()
            .any(|definition| definition.domain == selection.domain)
        {
            return Err(BattleRoomError::UnsupportedRewardDomain);
        }
        let reachability = self
            .encounter_reachability_runtime()
            .map_err(|_| BattleRoomError::InvalidEncounter)?;
        if !reachability.groups().iter().any(|group| {
            group.id() == &selection.group
                && group
                    .candidate_stage_ids()
                    .iter()
                    .any(|stage| stage == &selection.stage)
        }) {
            return Err(BattleRoomError::InvalidEncounter);
        }
        Ok(BattleRoomCompiler {
            factory: self.clone(),
            selection,
            rewards: Arc::new(BattleBlessings::compile(self).map_err(BattleRoomError::Entry)?),
            curios: Arc::new(
                self.curio_runtime()
                    .map_err(DomainRouteError::Curio)
                    .map_err(BattleRoomError::Route)?,
            ),
        })
    }
}

impl BattleRoomCompiler {
    /// Instantiates a primitive single-battle handoff, with typed failure and
    /// fault terminals and the existing verified victory reward program. All
    /// battle mutation, carry, grants and reward RNG remain in shared runtimes.
    pub fn compile(
        &self,
        context: &DomainRoomContext,
    ) -> Result<CompiledBattleRoom, BattleRoomError> {
        if !self.factory.room_context_matches(context) {
            return Err(BattleRoomError::InvalidContext);
        }
        let entry = context.entry_node();
        let encounter = context.node(1).map_err(BattleRoomError::Route)?;
        let battle = context.node(2).map_err(BattleRoomError::Route)?;
        let reward = context.node(3).map_err(BattleRoomError::Route)?;
        let failed = context.node(4).map_err(BattleRoomError::Route)?;
        let faulted = context.node(5).map_err(BattleRoomError::Route)?;
        let kinds = [
            (entry, ActivityNodeKind::Choice),
            (encounter, ActivityNodeKind::Choice),
            (battle, ActivityNodeKind::Battle),
            (reward, ActivityNodeKind::Reward),
            (
                failed,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Failed),
            ),
            (
                faulted,
                ActivityNodeKind::Terminal(ActivityTerminalOutcome::Faulted),
            ),
        ];
        let nodes = kinds
            .into_iter()
            .map(|(node, kind)| {
                ActivityNodeDefinition::new(node, context.section, kind, 1)
                    .map_err(DomainRouteError::Graph)
                    .map_err(BattleRoomError::Route)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let edges = [
            (entry, encounter, ActivityEdgeCondition::Always),
            (encounter, battle, ActivityEdgeCondition::Always),
            (
                battle,
                reward,
                ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Complete),
            ),
            (
                battle,
                failed,
                ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Failed),
            ),
            (
                battle,
                faulted,
                ActivityEdgeCondition::BattleOutcome(TerminalOutcome::Faulted),
            ),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, (from, to, condition))| {
            let index = u16::try_from(index).map_err(|_| BattleRoomError::InvalidDefinition)?;
            ActivityEdgeDefinition::new(
                context.edge(index).map_err(BattleRoomError::Route)?,
                from,
                to,
                condition,
                0,
                1,
            )
            .map_err(DomainRouteError::Graph)
            .map_err(BattleRoomError::Route)
        })
        .collect::<Result<Vec<_>, _>>()?;
        let layer = self
            .factory
            .bundle
            .catalog()
            .layers()
            .iter()
            .position(|layer| layer.id == context.layer)
            .and_then(|index| u64::try_from(index).ok())
            .and_then(|index| index.checked_add(1))
            .ok_or(BattleRoomError::InvalidContext)?;
        let marker = set_domain(Some(self.selection.domain));
        let ActivityOperation::SetSlot { slot, value } = &marker else {
            return Err(BattleRoomError::InvalidDefinition);
        };
        let operations = [
            (
                entry,
                vec![
                    ActivityOperation::SetSlot {
                        slot: LAYER_SLOT,
                        value: literal(ActivityValue::StableId(layer)),
                    },
                    ActivityOperation::SetSlot {
                        slot: LAYER_SEQUENCE_SLOT,
                        value: literal(ActivityValue::BoundedInteger(i64::from(
                            context.plane_ordinal,
                        ))),
                    },
                    marker.clone(),
                    ActivityOperation::Traverse(edges[0].id()),
                ],
            ),
            (
                encounter,
                vec![ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Encounter,
                    options: vec![ActivityOptionDefinition::new(
                        ActivityOptionId::new(1).expect("fixed nonzero engagement option"),
                        0,
                        ActivityCondition::Boolean(literal(ActivityValue::Boolean(true))),
                        vec![
                            ActivityOperation::Require(ActivityCondition::Equal(
                                ActivityExpression::Slot(*slot),
                                value.clone(),
                            )),
                            ActivityOperation::Traverse(edges[1].id()),
                        ],
                    )]
                    .into_boxed_slice(),
                }],
            ),
            (battle, Vec::new()),
            (reward, self.rewards.node_program(context.exit_edge())),
        ];
        let programs = operations
            .into_iter()
            .map(|(node, operations)| {
                let id =
                    ActivityProgramId::new(node.get()).ok_or(BattleRoomError::InvalidDefinition)?;
                ActivityProgramDefinition::new(id, operations)
                    .map(|program| GraphActivityNodeProgram::new(node, program))
                    .map_err(DomainRouteError::Program)
                    .map_err(BattleRoomError::Route)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fragment = DomainRoomProgram {
            exit_node: reward,
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
                    .map_err(BattleRoomError::Route)?,
            )
            .map_err(DomainRouteError::Program)
            .map_err(BattleRoomError::Route)?,
        );
        Ok(CompiledBattleRoom {
            context: context.clone(),
            selection: self.selection.clone(),
            component: self.factory.bundle_identity().component_digest().bytes(),
            decisions: self.factory.decision_catalog().digest(),
            encounter,
            battle,
            reward,
            fragment,
            entry_program,
        })
    }
}

impl CompiledBattleRoom {
    #[must_use]
    pub fn context(&self) -> &DomainRoomContext {
        &self.context
    }
    #[must_use]
    pub fn selection(&self) -> &BattleRoomSelection {
        &self.selection
    }
    #[must_use]
    /// Raw room contribution for `compile_curio_domain_route`. Binding requires
    /// that compiler's exact lifecycle-prefixed entry, not this raw initializer.
    pub fn fragment(&self) -> &DomainRoomProgram {
        &self.fragment
    }

    /// Exact current inputs for this primitive; not a compatibility identity.
    #[must_use]
    pub fn configuration_digest(&self) -> [u8; 32] {
        let mut hash = CanonicalDigestBuilder::new();
        for value in [
            b"starclock.divergent-universe.explicit-position-battle-curio-entry.v1".as_slice(),
            &self.component,
            &self.decisions,
            self.context.area.as_str().as_bytes(),
            self.context.layer.as_str().as_bytes(),
            self.context.position_key.as_bytes(),
            self.context.preset_source.as_bytes(),
            self.selection.group.as_str().as_bytes(),
            self.selection.stage.as_bytes(),
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
        hash.update([match self.selection.domain {
            BattleRewardDomain::Combat => 1,
            BattleRewardDomain::Elite => 2,
            BattleRewardDomain::Aberration => 3,
            BattleRewardDomain::Boss => 4,
        }]);
        hash.finalize()
    }
}
fn literal(value: ActivityValue) -> ActivityExpression {
    ActivityExpression::Literal(value)
}
