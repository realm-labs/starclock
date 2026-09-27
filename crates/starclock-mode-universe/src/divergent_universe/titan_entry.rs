//! Immutable permanent-talent entry projection and eventful starting fragments.

use starclock_activity::{
    ActivityEdgeCondition, ActivityEdgeDefinition, ActivityEdgeId, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityProgramDefinition,
    ActivityProgramId, GraphActivityNodeProgram, NodeId, SectionId,
};
use starclock_data::divergent_universe_titan_catalog::DivergentUniverseTitanTalentId;

use super::economy::{DivergentUniverseCurrencyKind, DivergentUniverseEconomyProjection};
use super::entry_flow::{DivergentUniverseEntryFlowError, DivergentUniverseRuntimeFactory};
use super::titan_runtime::DivergentUniverseTitanRuntimeError;

pub(super) const NODE: NodeId = NodeId::new(22_535).expect("nonzero Titan entry node");
pub(super) const ENTRY_POLICY: &[u8] =
    b"starclock.divergent-universe.titan-entry.before-offers-once-per-run";

/// These are initial owned talents, not a live account query or a free unlock.
/// Only the released RunEntry fragment program executes here; other effects
/// remain descriptors and are still rejected by current battle assembly.
pub(super) struct TitanEntry {
    pub(super) keys: Box<[u64]>,
    fragments: u64,
}

impl TitanEntry {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
        ids: &[DivergentUniverseTitanTalentId],
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        if ids.is_empty() {
            return Ok(Self {
                keys: Box::new([]),
                fragments: 0,
            });
        }
        let runtime = factory.titan_runtime().map_err(titan_error)?;
        let keys = runtime.entry_talent_keys(ids).map_err(titan_error)?;
        let mut fragments = 0_u64;
        for talent in runtime
            .talents()
            .iter()
            .filter(|talent| ids.contains(talent.id()))
        {
            if talent.condition() != "RunEntry" {
                continue;
            }
            if talent.operation() != "Increase" || talent.metric() != "starting_cosmic_fragments" {
                return Err(titan_error(
                    DivergentUniverseTitanRuntimeError::InvalidCatalog,
                ));
            }
            let amount = talent
                .value()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
                .ok_or_else(|| titan_error(DivergentUniverseTitanRuntimeError::InvalidCatalog))?;
            fragments = fragments
                .checked_add(amount)
                .ok_or_else(|| titan_error(DivergentUniverseTitanRuntimeError::InvalidCatalog))?;
        }
        Ok(Self { keys, fragments })
    }

    pub(super) const fn has_fragment_grant(&self) -> bool {
        self.fragments != 0
    }

    /// The one-visit automatic entry node runs before optional source-deck,
    /// Equation and occurrence offers. Currency gains use the shared economy
    /// operations, never an out-of-band initial counter or post-unlock grant.
    pub(super) fn attach(
        &self,
        graph: ActivityGraphDefinition,
        programs: &mut Vec<GraphActivityNodeProgram>,
        economy: &DivergentUniverseEconomyProjection,
    ) -> Result<ActivityGraphDefinition, DivergentUniverseEntryFlowError> {
        if !self.has_fragment_grant() {
            return Ok(graph);
        }
        let edge = ActivityEdgeId::new(22_535).expect("nonzero Titan entry edge");
        let mut nodes = graph.nodes().to_vec();
        nodes.push(
            ActivityNodeDefinition::new(
                NODE,
                SectionId::new(1).expect("nonzero section"),
                ActivityNodeKind::Choice,
                1,
            )
            .map_err(|_| invalid_definition())?,
        );
        let mut edges = graph.edges().to_vec();
        edges.push(
            ActivityEdgeDefinition::new(
                edge,
                NODE,
                graph.entry(),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .map_err(|_| invalid_definition())?,
        );
        let mut operations = economy
            .currency(DivergentUniverseCurrencyKind::CosmicFragment)
            .credit_operations(self.fragments)?;
        operations.push(ActivityOperation::Traverse(edge));
        programs.push(GraphActivityNodeProgram::new(
            NODE,
            ActivityProgramDefinition::new(
                ActivityProgramId::new(22_535).expect("nonzero Titan entry program"),
                operations,
            )
            .map_err(|_| invalid_definition())?,
        ));
        ActivityGraphDefinition::new(
            NODE,
            nodes,
            edges,
            graph
                .maximum_total_visits()
                .checked_add(1)
                .ok_or_else(invalid_definition)?,
        )
        .map_err(|_| invalid_definition())
    }
}

fn invalid_definition() -> DivergentUniverseEntryFlowError {
    DivergentUniverseEntryFlowError::InvalidActivityDefinition
}

fn titan_error(error: DivergentUniverseTitanRuntimeError) -> DivergentUniverseEntryFlowError {
    DivergentUniverseEntryFlowError::Titan(error)
}
