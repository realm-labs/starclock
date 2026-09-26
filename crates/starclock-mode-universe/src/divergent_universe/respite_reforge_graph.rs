//! Finite shared-graph menus for the optional Respite overwrite service.

use std::iter::once;

use super::{COMPLETION_LIMIT, OPEN_REFORGE, ReforgeRoom};
use crate::divergent_universe::{
    respite_room::{
        CompiledRespiteRoom, LEAVE, PAGE_WIDTH, RespiteRoomError, integer, literal, program,
    },
    state::{
        BLESSING_OFFER_SOURCE_SLOT, BLESSING_OFFERS_SLOT, BLESSINGS_SLOT,
        EQUATION_PROGRESS_DIRTY_SLOT, WORKBENCH_SLOT,
    },
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityExpression, ActivityNodeDefinition, ActivityNodeKind,
    ActivityOperation, ActivityOptionDefinition, ActivityOptionId, ActivityValue,
};

impl ReforgeRoom {
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
        for menu in &room.menus {
            let enter = connect(*menu, self.inputs[0], 414)?;
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
                option(OPEN_REFORGE)?,
                0,
                common.clone(),
                vec![
                    ActivityOperation::Require(common.clone()),
                    ActivityOperation::Traverse(enter),
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
                    .checked_add(64)
                    .ok_or(RespiteRoomError::InvalidPolicy)?,
            )
            .map_err(|_| RespiteRoomError::InvalidPolicy)?;
        }
        let back = connect(self.output, self.inputs[0], 64)?;
        for (page_index, blessings) in self.runtime.blessings().chunks(PAGE_WIDTH).enumerate() {
            let menu = self.inputs[page_index];
            let begin = connect(menu, self.output, 64)?;
            let leave = connect(menu, room.fragment.exit_node, 1)?;
            let enhance = connect(menu, room.menus[0], 414)?;
            let mut controls = vec![
                ActivityOptionDefinition::new(
                    option(LEAVE)?,
                    0,
                    always(),
                    vec![ActivityOperation::Traverse(leave)],
                ),
                ActivityOptionDefinition::new(
                    option(LEAVE - 1)?,
                    0,
                    always(),
                    vec![ActivityOperation::Traverse(enhance)],
                ),
            ];
            let mut fallback = controls.clone();
            fallback.sort_by_key(|option| (option.priority(), option.id()));
            for (target, target_page) in self.runtime.blessings().chunks(PAGE_WIDTH).enumerate() {
                if target == page_index {
                    continue;
                }
                let route = connect(menu, self.inputs[target], 414)?;
                let ordinal = u64::try_from(target)
                    .ok()
                    .and_then(|value| value.checked_add(10))
                    .ok_or(RespiteRoomError::InvalidPolicy)?;
                let key = LEAVE
                    .checked_sub(ordinal)
                    .ok_or(RespiteRoomError::InvalidPolicy)?;
                let owned = ActivityCondition::Any(
                    target_page
                        .iter()
                        .map(|blessing| owned(blessing.state_key()))
                        .collect(),
                );
                controls.push(ActivityOptionDefinition::new(
                    option(key)?,
                    0,
                    owned.clone(),
                    vec![
                        ActivityOperation::Require(owned),
                        ActivityOperation::Traverse(route),
                    ],
                ));
            }
            let mut choices = controls.clone();
            for blessing in blessings {
                choices.push(ActivityOptionDefinition::new(
                    option(blessing.state_key())?,
                    0,
                    owned(blessing.state_key()),
                    vec![
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Slot(self.addresses.accepted),
                        )),
                        self.accepted(false),
                        ActivityOperation::Traverse(begin),
                    ],
                ));
            }
            choices.sort_by_key(|option| (option.priority(), option.id()));
            controls.sort_by_key(|option| (option.priority(), option.id()));
            room.fragment.programs.push(program(
                menu,
                vec![
                    ActivityOperation::SetSlot {
                        slot: WORKBENCH_SLOT,
                        value: literal(ActivityValue::OptionalId(Some(self.workbench))),
                    },
                    self.accepted(false),
                    ActivityOperation::Conditional {
                        condition: common.clone(),
                        if_true: vec![ActivityOperation::Offer {
                            kind: ActivityDecisionKind::Service,
                            options: choices.into(),
                        }]
                        .into(),
                        if_false: vec![ActivityOperation::Offer {
                            kind: ActivityDecisionKind::Service,
                            options: fallback.into(),
                        }]
                        .into(),
                    },
                ],
            )?);
        }
        let options = self
            .pool
            .iter()
            .map(|(key, _)| {
                Ok(ActivityOptionDefinition::new(
                    option(*key)?,
                    0,
                    ActivityCondition::Equal(
                        ActivityExpression::CounterValue {
                            slot: self.addresses.offers,
                            key: *key,
                        },
                        integer(1),
                    ),
                    vec![
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Slot(self.addresses.accepted),
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
                ActivityOperation::SetSlot {
                    slot: WORKBENCH_SLOT,
                    value: literal(ActivityValue::OptionalId(Some(self.workbench))),
                },
                self.accepted(false),
                ActivityOperation::Require(ActivityCondition::Compare {
                    left: ActivityExpression::CounterEntryCount(self.addresses.offers),
                    operator: ActivityComparison::Greater,
                    right: integer(0),
                }),
                ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Service,
                    options: options.into(),
                },
            ],
        )?);
        for node in self.inputs.iter().copied().chain(once(self.output)) {
            let limit = if node == self.output { 64 } else { 415 };
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

    fn available(&self) -> ActivityCondition {
        ActivityCondition::All(
            vec![
                ActivityCondition::Equal(
                    ActivityExpression::Slot(WORKBENCH_SLOT),
                    literal(ActivityValue::OptionalId(Some(self.workbench))),
                ),
                self.price
                    .affordable_condition(self.fragments, self.receipt),
                ActivityCondition::LessThan(
                    ActivityExpression::Slot(self.addresses.completed),
                    integer(COMPLETION_LIMIT),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(self.addresses.selected),
                    literal(ActivityValue::OptionalId(None)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::CounterEntryCount(self.addresses.offers),
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
                ActivityCondition::Compare {
                    left: ActivityExpression::CounterEntryCount(BLESSINGS_SLOT),
                    operator: ActivityComparison::Greater,
                    right: integer(0),
                },
                ActivityCondition::Any(
                    self.pool
                        .iter()
                        .map(|(key, _)| {
                            ActivityCondition::Equal(
                                ActivityExpression::CounterValue {
                                    slot: BLESSINGS_SLOT,
                                    key: *key,
                                },
                                integer(0),
                            )
                        })
                        .collect(),
                ),
            ]
            .into(),
        )
    }
}

fn owned(key: u64) -> ActivityCondition {
    ActivityCondition::Compare {
        left: ActivityExpression::CounterValue {
            slot: BLESSINGS_SLOT,
            key,
        },
        operator: ActivityComparison::Greater,
        right: integer(0),
    }
}
fn always() -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(true)))
}
fn option(key: u64) -> Result<ActivityOptionId, RespiteRoomError> {
    ActivityOptionId::new(key).ok_or(RespiteRoomError::InvalidPolicy)
}
