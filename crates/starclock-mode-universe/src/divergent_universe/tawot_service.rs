//! Explicitly admitted purchase service using shared graph offers and transactions.
#[path = "tawot_service_execution.rs"]
mod execution;

use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseRuntimeFactory,
    curio_runtime::DivergentUniverseCurioRuntime,
    economy::{
        DivergentUniverseCurrencyKind, DivergentUniverseCurrencyRuntime,
        DivergentUniverseEconomyProjection,
    },
    state::{
        CURRENCIES_SLOT, TAWOT_ACCEPTED_SLOT, TAWOT_OFFER_SLOT, TAWOT_OPENS_SLOT,
        TAWOT_PURCHASES_SLOT,
    },
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivitySlotId, ActivityValue,
    GraphActivityCommandError, GraphActivityNodeProgram, GraphActivityRuntimeError, NodeId,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use starclock_data::divergent_universe_decisions::{TawotServiceDefinition, TawotServicePolicy};

const MENU: u32 = 0x7e40_0001;
const CARDS: u32 = 0x7e40_0002;
pub(super) const ENCOUNTER: u32 = 0x7e40_0003;
const BUY: u64 = 1;
const LEAVE: u64 = 2;
const CANCEL: u64 = 0x7e42_0001;
const OPEN_LIMIT: u32 = 64;

pub(super) fn nodes() -> [NodeId; 3] {
    [node(MENU), node(CARDS), node(ENCOUNTER)]
}

#[derive(Clone, Debug)]
pub(super) struct TawotService {
    definition: TawotServiceDefinition,
    states: Box<[(u64, DivergentUniverseCurioStateId)]>,
    curios: DivergentUniverseCurioRuntime,
    fragments: DivergentUniverseCurrencyRuntime,
}

impl TawotService {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
        level: u16,
        economy: &DivergentUniverseEconomyProjection,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let definition = factory
            .decision_catalog()
            .tawot_services()
            .iter()
            .find(|definition| definition.forge_level == level)
            .ok_or_else(invalid_definition)?
            .clone();
        match definition.policy {
            TawotServicePolicy::VersionedProjectPolicyUniformDistinctCurrentStatesPaidSelection => {
            }
        }
        let curios = factory.curio_runtime().map_err(|_| invalid_definition())?;
        let states = definition
            .states
            .iter()
            .map(|id| {
                curios
                    .states()
                    .iter()
                    .find(|state| state.id() == id)
                    .map(|state| (state.state_key(), id.clone()))
                    .ok_or_else(invalid_definition)
            })
            .collect::<Result<Box<[_]>, _>>()?;
        Ok(Self {
            definition,
            states,
            curios,
            fragments: economy
                .currency(DivergentUniverseCurrencyKind::CosmicFragment)
                .clone(),
        })
    }

    pub(super) fn attach(
        &self,
        graph: ActivityGraphDefinition,
        programs: &mut Vec<GraphActivityNodeProgram>,
    ) -> Result<ActivityGraphDefinition, DivergentUniverseEntryFlowError> {
        let original = graph
            .edges()
            .iter()
            .find(|edge| edge.id().get() == 1 && edge.from().get() == 1)
            .copied()
            .ok_or_else(invalid_definition)?;
        if !graph
            .nodes()
            .iter()
            .any(|node| node.id() == original.to() && node.kind() == ActivityNodeKind::Battle)
        {
            return Err(invalid_definition());
        }
        let section = graph
            .nodes()
            .iter()
            .find(|node| node.id().get() == 1)
            .ok_or_else(invalid_definition)?
            .section();
        let mut nodes = graph.nodes().to_vec();
        for (raw, limit) in [(MENU, OPEN_LIMIT + 1), (CARDS, OPEN_LIMIT), (ENCOUNTER, 1)] {
            nodes.push(
                ActivityNodeDefinition::new(node(raw), section, ActivityNodeKind::Choice, limit)
                    .map_err(|_| invalid_definition())?,
            );
        }
        let mut edges = graph
            .edges()
            .iter()
            .copied()
            .filter(|edge| edge.id() != original.id())
            .collect::<Vec<_>>();
        edges.push(
            ActivityEdgeDefinition::new(
                original.id(),
                original.from(),
                node(MENU),
                original.condition(),
                original.priority(),
                original.maximum_traversals(),
            )
            .map_err(|_| invalid_definition())?,
        );
        for (ordinal, from, to, limit) in [
            (1, MENU, CARDS, OPEN_LIMIT),
            (2, CARDS, MENU, OPEN_LIMIT),
            (3, MENU, ENCOUNTER, 1),
        ] {
            edges.push(
                ActivityEdgeDefinition::new(
                    edge(ordinal),
                    node(from),
                    node(to),
                    ActivityEdgeCondition::Always,
                    0,
                    limit,
                )
                .map_err(|_| invalid_definition())?,
            );
        }
        edges.push(
            ActivityEdgeDefinition::new(
                edge(4),
                node(ENCOUNTER),
                original.to(),
                ActivityEdgeCondition::Always,
                0,
                1,
            )
            .map_err(|_| invalid_definition())?,
        );
        let first = programs
            .iter_mut()
            .find(|program| program.node().get() == 1)
            .ok_or_else(invalid_definition)?;
        *first = GraphActivityNodeProgram::new(
            first.node(),
            ActivityProgramDefinition::new(
                first.program().id(),
                route_offers(first.program().operations()),
            )
            .map_err(|_| invalid_definition())?,
        );
        programs.push(program(MENU, 24121, self.menu())?);
        programs.push(program(CARDS, 24122, self.cards())?);
        programs.push(program(
            ENCOUNTER,
            24123,
            vec![ActivityOperation::Offer {
                kind: ActivityDecisionKind::Encounter,
                options: vec![option(1, yes(), vec![ActivityOperation::Traverse(edge(4))])]
                    .into_boxed_slice(),
            }],
        )?);
        ActivityGraphDefinition::new(
            graph.entry(),
            nodes,
            edges,
            graph
                .maximum_total_visits()
                .checked_add(2 * OPEN_LIMIT + 2)
                .ok_or_else(invalid_definition)?,
        )
        .map_err(|_| invalid_definition())
    }

    fn menu(&self) -> Vec<ActivityOperation> {
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Service,
            options: vec![
                option(
                    BUY,
                    ActivityCondition::All(
                        vec![
                            self.can_buy(),
                            compare(
                                ActivityExpression::Slot(TAWOT_OPENS_SLOT),
                                ActivityComparison::Less,
                                i64::from(OPEN_LIMIT),
                            ),
                        ]
                        .into_boxed_slice(),
                    ),
                    vec![require_accepted(), ActivityOperation::Traverse(edge(1))],
                ),
                option(
                    LEAVE,
                    yes(),
                    vec![
                        require_accepted(),
                        set(TAWOT_PURCHASES_SLOT, 0),
                        set(TAWOT_OPENS_SLOT, 0),
                        clear_offer(),
                        ActivityOperation::Traverse(edge(3)),
                    ],
                ),
            ]
            .into_boxed_slice(),
        }]
    }
    fn cards(&self) -> Vec<ActivityOperation> {
        let mut options = self
            .states
            .iter()
            .map(|(key, _)| {
                option(
                    *key,
                    ActivityCondition::All(
                        vec![
                            self.can_buy(),
                            ActivityCondition::OrderedIdSetContains {
                                slot: TAWOT_OFFER_SLOT,
                                id: *key,
                            },
                        ]
                        .into_boxed_slice(),
                    ),
                    vec![require_accepted(), ActivityOperation::Traverse(edge(2))],
                )
            })
            .collect::<Vec<_>>();
        options.push(option(
            CANCEL,
            yes(),
            vec![require_accepted(), ActivityOperation::Traverse(edge(2))],
        ));
        vec![ActivityOperation::Offer {
            kind: ActivityDecisionKind::Reward,
            options: options.into_boxed_slice(),
        }]
    }
    fn can_buy(&self) -> ActivityCondition {
        ActivityCondition::All(
            vec![
                compare(
                    ActivityExpression::CounterValue {
                        slot: CURRENCIES_SLOT,
                        key: self.fragments.key(),
                    },
                    ActivityComparison::GreaterOrEqual,
                    i64::from(self.definition.fragment_cost),
                ),
                compare(
                    ActivityExpression::Slot(TAWOT_PURCHASES_SLOT),
                    ActivityComparison::Less,
                    i64::from(self.definition.purchase_limit),
                ),
            ]
            .into_boxed_slice(),
        )
    }
}

// Only the initial checkpoint's encounter offer is redirected to the explicit
// service. Preserve nested occurrence costs, conditions and completion ordering.
fn route_offers(operations: &[ActivityOperation]) -> Vec<ActivityOperation> {
    operations
        .iter()
        .map(|operation| match operation {
            ActivityOperation::Offer { kind, options } => ActivityOperation::Offer {
                kind: if *kind == ActivityDecisionKind::Encounter {
                    ActivityDecisionKind::Route
                } else {
                    *kind
                },
                options: options
                    .iter()
                    .map(|item| {
                        ActivityOptionDefinition::new(
                            item.id(),
                            item.priority(),
                            item.enabled().clone(),
                            route_offers(item.operations()),
                        )
                    })
                    .collect(),
            },
            _ => operation.clone(),
        })
        .collect()
}
fn program(
    raw: u32,
    id: u32,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, DivergentUniverseEntryFlowError> {
    Ok(GraphActivityNodeProgram::new(
        node(raw),
        ActivityProgramDefinition::new(
            ActivityProgramId::new(id).ok_or_else(invalid_definition)?,
            operations,
        )
        .map_err(|_| invalid_definition())?,
    ))
}
fn option(
    raw: u64,
    condition: ActivityCondition,
    operations: Vec<ActivityOperation>,
) -> ActivityOptionDefinition {
    ActivityOptionDefinition::new(
        ActivityOptionId::new(raw).expect("compiled service option is nonzero"),
        0,
        condition,
        operations,
    )
}
fn node(raw: u32) -> NodeId {
    NodeId::new(raw).expect("reserved nonzero service node")
}
fn edge(ordinal: u32) -> ActivityEdgeId {
    ActivityEdgeId::new(0x7e41_0000 + ordinal).expect("reserved nonzero service edge")
}
fn yes() -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true)))
}
fn compare(lhs: ActivityExpression, comparison: ActivityComparison, rhs: i64) -> ActivityCondition {
    ActivityCondition::Compare {
        left: lhs,
        operator: comparison,
        right: ActivityExpression::Literal(ActivityValue::BoundedInteger(rhs)),
    }
}
fn set(slot: ActivitySlotId, value: i64) -> ActivityOperation {
    ActivityOperation::SetSlot {
        slot,
        value: ActivityExpression::Literal(ActivityValue::BoundedInteger(value)),
    }
}
fn clear_offer() -> ActivityOperation {
    ActivityOperation::SetOrderedIdSet {
        slot: TAWOT_OFFER_SLOT,
        values: Box::new([]),
    }
}
fn require_accepted() -> ActivityOperation {
    ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Slot(
        TAWOT_ACCEPTED_SLOT,
    )))
}
fn invalid_definition() -> DivergentUniverseEntryFlowError {
    DivergentUniverseEntryFlowError::InvalidActivityDefinition
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
