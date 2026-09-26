//! Fixed-stock offer conditions and finite shared graph operations.

use super::{LEAVE_SHOP, ShopRoomCompiler, ShopRoomError, program, set};
use crate::divergent_universe::{
    domain_route::{DomainRoomContext, DomainRoomProgram, DomainRouteError},
    shop_purchase::{ShopPurchaseError, ShopReward, ShopStockItem},
    state::{CURIO_STATES_SLOT, CURRENCIES_SLOT},
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityExpression, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityValue, NodeId,
};

impl ShopRoomCompiler {
    pub(super) fn compile_graph(
        &self,
        context: &DomainRoomContext,
        menu: NodeId,
    ) -> Result<DomainRoomProgram, ShopRoomError> {
        let entry = context.entry_node();
        let enter = context.edge(0).map_err(ShopRoomError::Route)?;
        let again = context.edge(1).map_err(ShopRoomError::Route)?;
        let limit = u32::try_from(self.runtime.items().len())
            .map_err(|_| ShopRoomError::DefinitionMismatch)?;
        let mut options = Vec::new();
        for item in self.runtime.items() {
            options.push(ActivityOptionDefinition::new(
                ActivityOptionId::new(u64::from(item.id.get()))
                    .ok_or(ShopRoomError::DefinitionMismatch)?,
                0,
                self.availability(item)?,
                vec![
                    ActivityOperation::Require(accepted(self)),
                    set(self.slots.accepted, ActivityValue::Boolean(false)),
                    ActivityOperation::Traverse(again),
                ],
            ));
        }
        options.push(ActivityOptionDefinition::new(
            ActivityOptionId::new(LEAVE_SHOP).expect("nonzero Leave key"),
            0,
            ActivityCondition::Boolean(ActivityExpression::Literal(ActivityValue::Boolean(true))),
            vec![
                ActivityOperation::Require(accepted(self)),
                set(self.slots.accepted, ActivityValue::Boolean(false)),
                ActivityOperation::Traverse(context.exit_edge()),
            ],
        ));
        Ok(DomainRoomProgram {
            exit_node: menu,
            nodes: vec![
                ActivityNodeDefinition::new(entry, context.section, ActivityNodeKind::Choice, 1)
                    .map_err(DomainRouteError::Graph)
                    .map_err(ShopRoomError::Route)?,
                ActivityNodeDefinition::new(
                    menu,
                    context.section,
                    ActivityNodeKind::Choice,
                    limit
                        .checked_add(1)
                        .ok_or(ShopRoomError::DefinitionMismatch)?,
                )
                .map_err(DomainRouteError::Graph)
                .map_err(ShopRoomError::Route)?,
            ],
            edges: [(enter, entry, menu, 1), (again, menu, menu, limit)]
                .into_iter()
                .map(|(id, from, to, visits)| {
                    ActivityEdgeDefinition::new(
                        id,
                        from,
                        to,
                        ActivityEdgeCondition::Always,
                        0,
                        visits,
                    )
                    .map_err(DomainRouteError::Graph)
                    .map_err(ShopRoomError::Route)
                })
                .collect::<Result<Vec<_>, _>>()?,
            programs: vec![
                program(
                    entry,
                    vec![
                        ActivityOperation::SetCounterMap {
                            slot: self.slots.purchased,
                            values: Box::new([]),
                        },
                        ActivityOperation::Traverse(enter),
                    ],
                )?,
                program(
                    menu,
                    vec![
                        set(self.slots.accepted, ActivityValue::Boolean(false)),
                        ActivityOperation::Offer {
                            kind: ActivityDecisionKind::Shop,
                            options: options.into(),
                        },
                    ],
                )?,
            ],
            random_offers: Vec::new(),
        })
    }
    fn availability(&self, item: &ShopStockItem) -> Result<ActivityCondition, ShopRoomError> {
        let mut conditions = vec![
            equal_zero(ActivityExpression::CounterValue {
                slot: self.slots.purchased,
                key: u64::from(item.id.get()),
            }),
            ActivityCondition::Compare {
                left: ActivityExpression::CounterValue {
                    slot: CURRENCIES_SLOT,
                    key: self.runtime.currency.key(),
                },
                operator: ActivityComparison::GreaterOrEqual,
                right: integer(
                    i64::try_from(item.price).map_err(|_| ShopRoomError::DefinitionMismatch)?,
                ),
            },
        ];
        match &item.reward {
            ShopReward::Blessing(id) => conditions.push(
                self.rewards
                    .blessing_identity_availability_condition(id)
                    .map_err(ShopRoomError::Reward)?,
            ),
            ShopReward::Curio(id) => {
                let state = self
                    .runtime
                    .curios
                    .state(id)
                    .map_err(ShopPurchaseError::Curio)
                    .map_err(ShopRoomError::Purchase)?;
                let owner = state.curio().ok_or(ShopRoomError::DefinitionMismatch)?;
                for copy in self.runtime.curios.states().iter().filter(|copy| {
                    copy.curio() == Some(owner) || copy.evolution_owner() == Some(owner)
                }) {
                    conditions.push(equal_zero(ActivityExpression::CounterValue {
                        slot: CURIO_STATES_SLOT,
                        key: copy.state_key(),
                    }));
                }
                if self
                    .factory
                    .decision_catalog()
                    .curio_acquisitions()
                    .iter()
                    .any(|effect| effect.state == *id)
                    && let Some(condition) = self
                        .rewards
                        .acquisition_availability_condition(id)
                        .map_err(ShopRoomError::Reward)?
                {
                    conditions.push(condition);
                }
            }
        }
        Ok(ActivityCondition::All(conditions.into()))
    }
}
fn accepted(compiler: &ShopRoomCompiler) -> ActivityCondition {
    ActivityCondition::Boolean(ActivityExpression::Slot(compiler.slots.accepted))
}
fn integer(value: i64) -> ActivityExpression {
    ActivityExpression::Literal(ActivityValue::BoundedInteger(value))
}
fn equal_zero(value: ActivityExpression) -> ActivityCondition {
    ActivityCondition::Equal(value, integer(0))
}
