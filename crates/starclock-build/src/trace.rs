//! Trace graph, typed B2 patch language and canonical topological ordering.

use std::collections::{BTreeMap, BTreeSet};

use starclock_combat::{UnitDefinitionId, rule::model::RuleSource};

use crate::{id::TraceNodeId, patch::BuildPatch, spec::PromotionStage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceNodeDefinition {
    id: TraceNodeId,
    source: RuleSource,
    prerequisites: Box<[TraceNodeId]>,
    promotion_requirement: PromotionStage,
    patches: Box<[BuildPatch]>,
}

impl TraceNodeDefinition {
    #[must_use]
    pub fn new(
        id: TraceNodeId,
        source: RuleSource,
        prerequisites: Vec<TraceNodeId>,
        promotion_requirement: PromotionStage,
        patches: Vec<BuildPatch>,
    ) -> Self {
        Self {
            id,
            source,
            prerequisites: prerequisites.into_boxed_slice(),
            promotion_requirement,
            patches: patches.into_boxed_slice(),
        }
    }
    #[must_use]
    pub const fn id(&self) -> TraceNodeId {
        self.id
    }
    #[must_use]
    pub const fn source(&self) -> &RuleSource {
        &self.source
    }
    #[must_use]
    pub fn prerequisites(&self) -> &[TraceNodeId] {
        &self.prerequisites
    }
    #[must_use]
    pub const fn promotion_requirement(&self) -> PromotionStage {
        self.promotion_requirement
    }
    #[must_use]
    pub fn patches(&self) -> &[BuildPatch] {
        &self.patches
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceGraphDefinition {
    form: UnitDefinitionId,
    nodes: Box<[TraceNodeDefinition]>,
    canonical_order: Box<[TraceNodeId]>,
}

impl TraceGraphDefinition {
    #[must_use]
    pub fn new(form: UnitDefinitionId, nodes: Vec<TraceNodeDefinition>) -> Self {
        Self {
            form,
            nodes: nodes.into_boxed_slice(),
            canonical_order: Box::new([]),
        }
    }
    pub(crate) fn canonicalize(&mut self) -> Result<(), TraceGraphError> {
        self.nodes.sort_unstable_by_key(TraceNodeDefinition::id);
        if self.nodes.windows(2).any(|pair| pair[0].id == pair[1].id) {
            return Err(TraceGraphError::DuplicateNode);
        }
        let known = self
            .nodes
            .iter()
            .map(|node| node.id)
            .collect::<BTreeSet<_>>();
        if self.nodes.iter().any(|node| {
            node.prerequisites.windows(2).any(|pair| pair[0] >= pair[1])
                || node
                    .prerequisites
                    .iter()
                    .any(|dependency| !known.contains(dependency))
        }) {
            return Err(TraceGraphError::InvalidPrerequisite);
        }
        let mut indegree = BTreeMap::new();
        let mut dependents = BTreeMap::<TraceNodeId, Vec<TraceNodeId>>::new();
        for node in &self.nodes {
            indegree.insert(node.id, node.prerequisites.len());
            for dependency in &node.prerequisites {
                dependents.entry(*dependency).or_default().push(node.id);
            }
        }
        let mut ready = indegree
            .iter()
            .filter_map(|(id, count)| (*count == 0).then_some(*id))
            .collect::<BTreeSet<_>>();
        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(id) = ready.pop_first() {
            order.push(id);
            for dependent in dependents.get(&id).into_iter().flatten() {
                let count = indegree.get_mut(dependent).expect("known dependent");
                *count -= 1;
                if *count == 0 {
                    ready.insert(*dependent);
                }
            }
        }
        if order.len() != self.nodes.len() {
            return Err(TraceGraphError::Cycle);
        }
        self.canonical_order = order.into_boxed_slice();
        Ok(())
    }
    #[must_use]
    pub const fn form(&self) -> UnitDefinitionId {
        self.form
    }
    #[must_use]
    pub fn nodes(&self) -> &[TraceNodeDefinition] {
        &self.nodes
    }
    #[must_use]
    pub fn node(&self, id: TraceNodeId) -> Option<&TraceNodeDefinition> {
        self.nodes
            .binary_search_by_key(&id, TraceNodeDefinition::id)
            .ok()
            .map(|index| &self.nodes[index])
    }
    #[must_use]
    pub fn canonical_order(&self) -> &[TraceNodeId] {
        &self.canonical_order
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TraceGraphError {
    DuplicateNode,
    InvalidPrerequisite,
    Cycle,
}
