//! Two bounded menus in the existing logical Respite room, not a new executor.

use super::{EquationReforgeRoom, OPEN_EQUATION_REFORGE};
use crate::divergent_universe::{
    respite_room::{CompiledRespiteRoom, LEAVE, RespiteRoomError, integer, literal, program},
    state::{
        BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, EQUATION_OFFER_SOURCE_SLOT,
        EQUATION_OFFERS_SLOT, EQUATION_PROGRESS_DIRTY_SLOT, EQUATION_REROLL_COUNT_SLOT,
        EQUATIONS_SLOT, WORKBENCH_SLOT,
    },
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityExpression, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityValue,
};
use starclock_data::divergent_universe_equation_catalog::DivergentUniverseEquationCategory;

impl EquationReforgeRoom {
    pub(super) fn attach(&self, room: &mut CompiledRespiteRoom) -> Result<(), RespiteRoomError> {
        let mut edges = Vec::new();
        let mut index = u16::try_from(room.fragment.edges.len())
            .map_err(|_| RespiteRoomError::InvalidPolicy)?;
        let mut connect = |from, to, limit| {
            let id = room.context.edge(index).map_err(RespiteRoomError::Route)?;
            index = index
                .checked_add(1)
                .ok_or(RespiteRoomError::InvalidPolicy)?;
            edges.push(
                ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::Always, 0, limit)
                    .map_err(|_| RespiteRoomError::InvalidPolicy)?,
            );
            Ok::<_, RespiteRoomError>(id)
        };
        let common = self.available();
        for menu in &room.enhancement_menus {
            let edge = connect(*menu, self.input, 414)?;
            let record = room
                .fragment
                .programs
                .iter_mut()
                .find(|program| program.node() == *menu)
                .ok_or(RespiteRoomError::DefinitionMismatch)?;
            let mut operations = record.program().operations().to_vec();
            let Some(ActivityOperation::Offer { options, .. }) = operations.last_mut() else {
                return Err(RespiteRoomError::DefinitionMismatch);
            };
            let mut updated = options.to_vec();
            updated.push(ActivityOptionDefinition::new(
                option(OPEN_EQUATION_REFORGE)?,
                0,
                common.clone(),
                vec![
                    ActivityOperation::Require(common.clone()),
                    ActivityOperation::Traverse(edge),
                ],
            ));
            updated.sort_by_key(|option| (option.priority(), option.id()));
            *options = updated.into();
            *record = program(*menu, operations)?;
            let node = room
                .fragment
                .nodes
                .iter_mut()
                .find(|node| node.id() == *menu)
                .ok_or(RespiteRoomError::DefinitionMismatch)?;
            *node = ActivityNodeDefinition::new(
                node.id(),
                node.section(),
                node.kind(),
                node.maximum_visits()
                    .checked_add(u32::from(self.policy.limit))
                    .ok_or(RespiteRoomError::InvalidPolicy)?,
            )
            .map_err(|_| RespiteRoomError::InvalidPolicy)?;
        }
        let begin = connect(self.input, self.output, u32::from(self.policy.limit))?;
        let back = connect(self.output, self.input, u32::from(self.policy.limit))?;
        let enhance = connect(self.input, room.enhancement_menus[0], 414)?;
        let leave = connect(self.input, room.fragment.exit_node, 1)?;
        let controls = vec![
            ActivityOptionDefinition::new(
                option(LEAVE - 1)?,
                0,
                always(),
                vec![ActivityOperation::Traverse(enhance)],
            ),
            ActivityOptionDefinition::new(
                option(LEAVE)?,
                0,
                always(),
                vec![ActivityOperation::Traverse(leave)],
            ),
        ];
        let mut choices = Vec::new();
        for entry in &self.entries {
            choices.push(ActivityOptionDefinition::new(
                option(entry.key)?,
                0,
                ActivityCondition::All(
                    vec![owned(entry.key), self.unowned_category(entry.category)].into(),
                ),
                vec![
                    ActivityOperation::Require(ActivityCondition::Boolean(
                        ActivityExpression::Slot(self.slots.accepted),
                    )),
                    self.accepted(false),
                    ActivityOperation::Traverse(begin),
                ],
            ));
        }
        choices.extend(controls.clone());
        room.fragment.programs.push(program(
            self.input,
            vec![
                self.workbench_operation(),
                self.accepted(false),
                ActivityOperation::Conditional {
                    condition: common,
                    if_true: vec![ActivityOperation::Offer {
                        kind: ActivityDecisionKind::Service,
                        options: choices.into(),
                    }]
                    .into(),
                    if_false: vec![ActivityOperation::Offer {
                        kind: ActivityDecisionKind::Service,
                        options: controls.into(),
                    }]
                    .into(),
                },
            ],
        )?);
        let choices = self
            .entries
            .iter()
            .map(|entry| {
                Ok(ActivityOptionDefinition::new(
                    option(entry.key)?,
                    0,
                    ActivityCondition::Equal(
                        ActivityExpression::CounterValue {
                            slot: self.slots.offers,
                            key: entry.key,
                        },
                        integer(1),
                    ),
                    vec![
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Slot(self.slots.accepted),
                        )),
                        self.accepted(false),
                        ActivityOperation::Traverse(back),
                    ],
                ))
            })
            .collect::<Result<Vec<_>, RespiteRoomError>>()?;
        room.fragment.programs.push(program(
            self.output,
            vec![
                self.workbench_operation(),
                self.accepted(false),
                ActivityOperation::Require(ActivityCondition::Compare {
                    left: ActivityExpression::CounterEntryCount(self.slots.offers),
                    operator: ActivityComparison::Greater,
                    right: integer(0),
                }),
                ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Service,
                    options: choices.into(),
                },
            ],
        )?);
        for (node, limit) in [
            (self.input, 415),
            (self.output, u32::from(self.policy.limit)),
        ] {
            room.fragment.nodes.push(
                ActivityNodeDefinition::new(
                    node,
                    room.context.section,
                    ActivityNodeKind::Choice,
                    limit,
                )
                .map_err(|_| RespiteRoomError::InvalidPolicy)?,
            );
        }
        room.fragment.edges.extend(edges);
        Ok(())
    }
    fn workbench_operation(&self) -> ActivityOperation {
        ActivityOperation::SetSlot {
            slot: WORKBENCH_SLOT,
            value: literal(ActivityValue::OptionalId(Some(self.workbench))),
        }
    }
    fn unowned_category(&self, category: DivergentUniverseEquationCategory) -> ActivityCondition {
        ActivityCondition::Any(
            self.entries
                .iter()
                .filter(|entry| entry.category == category)
                .map(|entry| ActivityCondition::Not(Box::new(owned(entry.key))))
                .collect(),
        )
    }
    fn available(&self) -> ActivityCondition {
        let eligible = [
            DivergentUniverseEquationCategory::Rare,
            DivergentUniverseEquationCategory::Epic,
            DivergentUniverseEquationCategory::Legendary,
            DivergentUniverseEquationCategory::PathEcho,
        ]
        .into_iter()
        .map(|category| {
            ActivityCondition::All(
                vec![
                    ActivityCondition::Any(
                        self.entries
                            .iter()
                            .filter(|entry| entry.category == category)
                            .map(|entry| owned(entry.key))
                            .collect(),
                    ),
                    self.unowned_category(category),
                ]
                .into(),
            )
        })
        .collect();
        ActivityCondition::All(
            vec![
                ActivityCondition::Equal(
                    ActivityExpression::Slot(WORKBENCH_SLOT),
                    literal(ActivityValue::OptionalId(Some(self.workbench))),
                ),
                self.policy
                    .price
                    .affordable_condition(self.fragments, self.receipt),
                ActivityCondition::LessThan(
                    ActivityExpression::Slot(self.slots.completed),
                    integer(i64::from(self.policy.limit)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(self.slots.selected),
                    literal(ActivityValue::OptionalId(None)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::CounterEntryCount(self.slots.offers),
                    integer(0),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(EQUATION_OFFER_SOURCE_SLOT),
                    literal(ActivityValue::OptionalId(None)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(EQUATION_REROLL_COUNT_SLOT),
                    integer(0),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::OrderedIdSetCount(EQUATION_OFFERS_SLOT),
                    integer(0),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(BLESSING_OFFER_SOURCE_SLOT),
                    literal(ActivityValue::OptionalId(None)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::CounterEntryCount(BLESSING_OFFERS_SLOT),
                    integer(0),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(EQUATION_PROGRESS_DIRTY_SLOT),
                    literal(ActivityValue::Boolean(false)),
                ),
                ActivityCondition::Any(eligible),
            ]
            .into(),
        )
    }
}
fn owned(id: u64) -> ActivityCondition {
    ActivityCondition::OrderedIdSetContains {
        slot: EQUATIONS_SLOT,
        id,
    }
}
fn always() -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(true)))
}
fn option(id: u64) -> Result<ActivityOptionId, RespiteRoomError> {
    ActivityOptionId::new(id).ok_or(RespiteRoomError::InvalidPolicy)
}
