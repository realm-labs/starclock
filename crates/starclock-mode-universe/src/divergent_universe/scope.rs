//! Logical Divergent Universe lifetimes over generic Activity scopes.

use starclock_activity::{
    ActivityScope, LogicalScopeAddress, LogicalScopeClassDefinition, LogicalScopeClassId,
    LogicalScopeDefinitions, LogicalScopeNodeBinding, NodeId,
};

use super::battle_route::{LayerBattleAddress, domain_choice_node, layer_entry_node};
use super::entry_flow::DivergentUniverseEntryFlowError;
use super::source_deck_selection::NODE as SOURCE_DECK_NODE;
use super::tawot_service::nodes as tawot_service_nodes;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DivergentUniverseLogicalScopeKind {
    Run,
    Plane,
    Node,
    Battle,
}

impl DivergentUniverseLogicalScopeKind {
    pub const ALL: [Self; 4] = [Self::Run, Self::Plane, Self::Node, Self::Battle];

    #[must_use]
    pub const fn activity_scope(self) -> ActivityScope {
        match self {
            Self::Run => ActivityScope::Activity,
            Self::Plane => ActivityScope::Section,
            Self::Node => ActivityScope::Node,
            Self::Battle => ActivityScope::Attempt,
        }
    }

    #[must_use]
    pub const fn class_id(self) -> LogicalScopeClassId {
        match self {
            Self::Run => class(1),
            Self::Plane => class(2),
            Self::Node => class(3),
            Self::Battle => class(4),
        }
    }
}

pub(super) fn compile(
    layer_count: usize,
    first_ordinary_vertical_slice: bool,
    initial_equation: bool,
    all_layers: bool,
    tawot_service: bool,
    source_deck_selection: bool,
) -> Result<LogicalScopeDefinitions, DivergentUniverseEntryFlowError> {
    let maximum_planes = u32::try_from(layer_count)
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)?;
    if maximum_planes == 0 {
        return Err(DivergentUniverseEntryFlowError::InvalidActivityDefinition);
    }
    let classes = vec![
        scope_class(DivergentUniverseLogicalScopeKind::Run, None, 1)?,
        scope_class(
            DivergentUniverseLogicalScopeKind::Plane,
            Some(DivergentUniverseLogicalScopeKind::Run),
            maximum_planes,
        )?,
        scope_class(
            DivergentUniverseLogicalScopeKind::Node,
            Some(DivergentUniverseLogicalScopeKind::Plane),
            maximum_planes,
        )?,
        scope_class(
            DivergentUniverseLogicalScopeKind::Battle,
            Some(DivergentUniverseLogicalScopeKind::Node),
            256,
        )?,
    ];
    let mut bindings = Vec::with_capacity(layer_count + 1);
    if source_deck_selection {
        bindings.push(binding(
            SOURCE_DECK_NODE,
            vec![address(DivergentUniverseLogicalScopeKind::Run, 1)?],
        )?);
    }
    if tawot_service {
        for physical in tawot_service_nodes() {
            bindings.push(binding(
                physical,
                vec![
                    address(DivergentUniverseLogicalScopeKind::Run, 1)?,
                    address(DivergentUniverseLogicalScopeKind::Plane, 1)?,
                    address(DivergentUniverseLogicalScopeKind::Node, 1)?,
                ],
            )?);
        }
    }
    if initial_equation {
        bindings.push(binding(
            node(maximum_planes + 7)?,
            vec![
                address(DivergentUniverseLogicalScopeKind::Run, 1)?,
                address(DivergentUniverseLogicalScopeKind::Plane, 1)?,
                address(DivergentUniverseLogicalScopeKind::Node, 1)?,
            ],
        )?);
    }
    for ordinal in 1..=maximum_planes {
        bindings.push(binding(
            node(ordinal)?,
            vec![
                address(DivergentUniverseLogicalScopeKind::Run, 1)?,
                address(DivergentUniverseLogicalScopeKind::Plane, u64::from(ordinal))?,
                address(DivergentUniverseLogicalScopeKind::Node, u64::from(ordinal))?,
            ],
        )?);
    }
    bindings.push(binding(
        node(maximum_planes + 1)?,
        vec![address(DivergentUniverseLogicalScopeKind::Run, 1)?],
    )?);
    if first_ordinary_vertical_slice {
        if all_layers {
            for ordinal in 2..=maximum_planes {
                let address_pair = LayerBattleAddress::new(maximum_planes, ordinal)?;
                for physical in [
                    address_pair.battle,
                    address_pair.reward,
                    layer_entry_node(maximum_planes, ordinal)?,
                    domain_choice_node(maximum_planes, ordinal)?,
                ] {
                    bindings.push(binding(
                        physical,
                        vec![
                            address(DivergentUniverseLogicalScopeKind::Run, 1)?,
                            address(DivergentUniverseLogicalScopeKind::Plane, u64::from(ordinal))?,
                            address(DivergentUniverseLogicalScopeKind::Node, u64::from(ordinal))?,
                        ],
                    )?);
                }
            }
        }
        bindings.push(binding(
            node(maximum_planes + 6)?,
            vec![
                address(DivergentUniverseLogicalScopeKind::Run, 1)?,
                address(DivergentUniverseLogicalScopeKind::Plane, 1)?,
                address(DivergentUniverseLogicalScopeKind::Node, 1)?,
            ],
        )?);
        bindings.push(binding(
            node(maximum_planes + 3)?,
            vec![
                address(DivergentUniverseLogicalScopeKind::Run, 1)?,
                address(DivergentUniverseLogicalScopeKind::Plane, 1)?,
                address(DivergentUniverseLogicalScopeKind::Node, 1)?,
            ],
        )?);
    }
    LogicalScopeDefinitions::new(classes, bindings)
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

fn scope_class(
    kind: DivergentUniverseLogicalScopeKind,
    parent: Option<DivergentUniverseLogicalScopeKind>,
    maximum_instances: u32,
) -> Result<LogicalScopeClassDefinition, DivergentUniverseEntryFlowError> {
    LogicalScopeClassDefinition::new(
        kind.class_id(),
        parent.map(DivergentUniverseLogicalScopeKind::class_id),
        maximum_instances,
    )
    .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

fn binding(
    node: NodeId,
    path: Vec<LogicalScopeAddress>,
) -> Result<LogicalScopeNodeBinding, DivergentUniverseEntryFlowError> {
    LogicalScopeNodeBinding::new(node, path)
        .map_err(|_| DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

fn address(
    kind: DivergentUniverseLogicalScopeKind,
    key: u64,
) -> Result<LogicalScopeAddress, DivergentUniverseEntryFlowError> {
    LogicalScopeAddress::new(kind.class_id(), key)
        .ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

fn node(raw: u32) -> Result<NodeId, DivergentUniverseEntryFlowError> {
    NodeId::new(raw).ok_or(DivergentUniverseEntryFlowError::InvalidActivityDefinition)
}

const fn class(raw: u32) -> LogicalScopeClassId {
    match LogicalScopeClassId::new(raw) {
        Some(value) => value,
        None => panic!("non-zero logical scope class"),
    }
}
