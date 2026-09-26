//! Compact admission predicate and finite cached-choice graph for synthesis.

use super::{
    CANCEL_SYNTHESIS, CurioSynthesisRoomError, LEAVE_SYNTHESIS, OPEN_LIMIT, OPEN_SYNTHESIS,
    SynthesisService, integer, literal, set,
};
use crate::divergent_universe::{
    curio_synthesis::quality,
    domain_route::{DomainRoomContext, DomainRoomProgram},
    state::{CURIO_STATES_SLOT, WORKBENCH_SLOT},
};
use starclock_activity::{
    ActivityComparison, ActivityCondition, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityNodeDefinition,
    ActivityNodeKind, ActivityOperation, ActivityOptionDefinition, ActivityOptionId,
    ActivityProgramDefinition, ActivityProgramId, ActivitySlotId, ActivityValue,
    GraphActivityNodeProgram, NodeId,
};

impl SynthesisService {
    pub(super) fn compile_graph(
        &self,
        context: &DomainRoomContext,
        nodes: [NodeId; 4],
    ) -> Result<DomainRoomProgram, CurioSynthesisRoomError> {
        let [menu, first, second, output] = nodes;
        let mut edges = Vec::new();
        let mut connect = |index, from, to, limit| {
            let id = context
                .edge(index)
                .map_err(CurioSynthesisRoomError::Route)?;
            edges.push(
                ActivityEdgeDefinition::new(id, from, to, ActivityEdgeCondition::Always, 0, limit)
                    .map_err(|_| CurioSynthesisRoomError::InvalidPolicy)?,
            );
            Ok::<_, CurioSynthesisRoomError>(id)
        };
        let enter = connect(0, context.entry_node(), menu, 1)?;
        let open = connect(1, menu, first, 64)?;
        let select_first = connect(2, first, second, 64)?;
        let select_second = connect(3, second, output, 64)?;
        let confirm = connect(4, output, menu, 64)?;
        let cancel_first = connect(5, first, menu, 64)?;
        let cancel_second = connect(6, second, menu, 64)?;
        let slots = self.slots;
        let clear = self.clear_selection();
        let mut entry = clear.clone();
        entry.extend([
            set(slots.completed, ActivityValue::BoundedInteger(0)),
            set(slots.opens, ActivityValue::BoundedInteger(0)),
            ActivityOperation::Traverse(enter),
        ]);
        let empty = ActivityCondition::All(
            vec![
                ActivityCondition::Equal(
                    ActivityExpression::Slot(slots.first),
                    literal(ActivityValue::OptionalId(None)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::Slot(slots.second),
                    literal(ActivityValue::OptionalId(None)),
                ),
                ActivityCondition::Equal(
                    ActivityExpression::CounterEntryCount(slots.choices),
                    integer(0),
                ),
            ]
            .into(),
        );
        let available = ActivityCondition::All(
            vec![
                empty.clone(),
                self.admission(),
                ActivityCondition::LessThan(
                    ActivityExpression::Slot(slots.completed),
                    integer(i64::from(self.policy.limit)),
                ),
                ActivityCondition::LessThan(
                    ActivityExpression::Slot(slots.opens),
                    integer(i64::from(OPEN_LIMIT)),
                ),
            ]
            .into(),
        );
        let menu_program = vec![
            set(
                WORKBENCH_SLOT,
                ActivityValue::OptionalId(Some(self.workbench)),
            ),
            set(slots.accepted, ActivityValue::Boolean(false)),
            ActivityOperation::Require(empty),
            ActivityOperation::Offer {
                kind: ActivityDecisionKind::Service,
                options: vec![
                    self.option(OPEN_SYNTHESIS, available, open)?,
                    self.option(LEAVE_SYNTHESIS, yes(), context.exit_edge())?,
                ]
                .into(),
            },
        ];
        let input_program = |which, route, cancel| -> Result<_, CurioSynthesisRoomError> {
            let mut options = self
                .curios
                .states()
                .iter()
                .filter(|state| {
                    quality(state.category()).is_ok()
                        && (state.curio().is_some() || state.evolution_owner().is_some())
                })
                .map(|state| self.option(state.state_key(), self.cached(state.state_key()), route))
                .collect::<Result<Vec<_>, _>>()?;
            options.push(self.option(CANCEL_SYNTHESIS, yes(), cancel)?);
            options.sort_by_key(|option| (option.priority(), option.id()));
            Ok(vec![
                set(
                    WORKBENCH_SLOT,
                    ActivityValue::OptionalId(Some(self.workbench)),
                ),
                set(slots.accepted, ActivityValue::Boolean(false)),
                ActivityOperation::Require(ActivityCondition::Equal(
                    ActivityExpression::Slot(slots.second),
                    literal(ActivityValue::OptionalId(None)),
                )),
                ActivityOperation::Require(if which == first {
                    ActivityCondition::Equal(
                        ActivityExpression::Slot(slots.first),
                        literal(ActivityValue::OptionalId(None)),
                    )
                } else {
                    ActivityCondition::Not(Box::new(ActivityCondition::Equal(
                        ActivityExpression::Slot(slots.first),
                        literal(ActivityValue::OptionalId(None)),
                    )))
                }),
                ActivityOperation::Require(positive_choices(slots.choices)),
                ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Service,
                    options: options.into(),
                },
            ])
        };
        let mut outputs = self
            .curios
            .states()
            .iter()
            .filter(|state| {
                quality(state.category()).is_ok()
                    && state.curio().is_some()
                    && state.evolution_owner().is_none()
            })
            .map(|state| self.option(state.state_key(), self.cached(state.state_key()), confirm))
            .collect::<Result<Vec<_>, _>>()?;
        outputs.sort_by_key(|option| (option.priority(), option.id()));
        let output_program = vec![
            set(
                WORKBENCH_SLOT,
                ActivityValue::OptionalId(Some(self.workbench)),
            ),
            set(slots.accepted, ActivityValue::Boolean(false)),
            ActivityOperation::Require(positive_choices(slots.choices)),
            ActivityOperation::Require(ActivityCondition::Compare {
                left: ActivityExpression::CounterEntryCount(slots.choices),
                operator: ActivityComparison::LessOrEqual,
                right: integer(3),
            }),
            ActivityOperation::Offer {
                kind: ActivityDecisionKind::Service,
                options: outputs.into(),
            },
        ];
        let programs = vec![
            program(context.entry_node(), entry)?,
            program(menu, menu_program)?,
            program(first, input_program(first, select_first, cancel_first)?)?,
            program(second, input_program(second, select_second, cancel_second)?)?,
            program(output, output_program)?,
        ];
        let nodes = [
            (context.entry_node(), 1),
            (menu, 65),
            (first, 64),
            (second, 64),
            (output, 64),
        ]
        .into_iter()
        .map(|(node, limit)| {
            ActivityNodeDefinition::new(node, context.section, ActivityNodeKind::Choice, limit)
                .map_err(|_| CurioSynthesisRoomError::InvalidPolicy)
        })
        .collect::<Result<Vec<_>, _>>()?;
        Ok(DomainRoomProgram {
            exit_node: menu,
            nodes,
            edges,
            programs,
            random_offers: Vec::new(),
        })
    }

    /// Each owner is one active predicate, including evolved aliases. Four-way
    /// pair decomposition is O(n log n), with bounded shallow conditions rather
    /// than all state pairs repeated in every input option. It is not a selector.
    fn admission(&self) -> ActivityCondition {
        let mut qualities = Vec::new();
        for rank in 1..=3 {
            let mut active = Vec::new();
            let mut unowned = Vec::new();
            for owner in self.curios.curios() {
                let states = self
                    .curios
                    .states()
                    .iter()
                    .filter(|state| state.curio().or(state.evolution_owner()) == Some(owner.id()))
                    .collect::<Vec<_>>();
                let input = states
                    .iter()
                    .filter(|state| quality(state.category()).is_ok_and(|value| value == rank))
                    .map(|state| ActivityCondition::Equal(counter(state.state_key()), integer(1)))
                    .collect::<Vec<_>>();
                if !input.is_empty() {
                    active.push(ActivityCondition::Any(input.into()));
                }
                let output = states.iter().any(|state| {
                    state.curio().is_some()
                        && state.evolution_owner().is_none()
                        && quality(state.category())
                            .is_ok_and(|output| output > rank || (rank == 3 && output == 3))
                });
                if output {
                    unowned.push(ActivityCondition::All(
                        states
                            .iter()
                            .map(|state| {
                                ActivityCondition::Equal(counter(state.state_key()), integer(0))
                            })
                            .collect(),
                    ));
                }
            }
            qualities.push(ActivityCondition::All(
                vec![two_owners(&active), ActivityCondition::Any(unowned.into())].into(),
            ));
        }
        ActivityCondition::Any(qualities.into())
    }
    fn cached(&self, key: u64) -> ActivityCondition {
        ActivityCondition::Equal(
            ActivityExpression::CounterValue {
                slot: self.slots.choices,
                key,
            },
            integer(1),
        )
    }
    fn option(
        &self,
        key: u64,
        enabled: ActivityCondition,
        edge: ActivityEdgeId,
    ) -> Result<ActivityOptionDefinition, CurioSynthesisRoomError> {
        Ok(ActivityOptionDefinition::new(
            ActivityOptionId::new(key).ok_or(CurioSynthesisRoomError::InvalidPolicy)?,
            0,
            enabled,
            vec![
                ActivityOperation::Require(ActivityCondition::Boolean(ActivityExpression::Slot(
                    self.slots.accepted,
                ))),
                set(self.slots.accepted, ActivityValue::Boolean(false)),
                ActivityOperation::Traverse(edge),
            ],
        ))
    }
    pub(super) fn clear_selection(&self) -> Vec<ActivityOperation> {
        vec![
            set(self.slots.first, ActivityValue::OptionalId(None)),
            set(self.slots.second, ActivityValue::OptionalId(None)),
            ActivityOperation::SetCounterMap {
                slot: self.slots.choices,
                values: Box::new([]),
            },
        ]
    }
}
fn two_owners(owners: &[ActivityCondition]) -> ActivityCondition {
    if owners.len() < 2 {
        return ActivityCondition::Boolean(literal(ActivityValue::Boolean(false)));
    }
    if owners.len() == 2 {
        return ActivityCondition::All(owners.to_vec().into());
    }
    let groups = owners.chunks(owners.len().div_ceil(4)).collect::<Vec<_>>();
    let mut combinations = groups
        .iter()
        .map(|group| two_owners(group))
        .collect::<Vec<_>>();
    for (i, group) in groups.iter().enumerate() {
        for other in &groups[i + 1..] {
            combinations.push(ActivityCondition::All(
                vec![
                    ActivityCondition::Any(group.to_vec().into()),
                    ActivityCondition::Any(other.to_vec().into()),
                ]
                .into(),
            ));
        }
    }
    ActivityCondition::Any(combinations.into())
}
fn counter(key: u64) -> ActivityExpression {
    ActivityExpression::CounterValue {
        slot: CURIO_STATES_SLOT,
        key,
    }
}
fn yes() -> ActivityCondition {
    ActivityCondition::Boolean(literal(ActivityValue::Boolean(true)))
}
fn positive_choices(slot: ActivitySlotId) -> ActivityCondition {
    ActivityCondition::Compare {
        left: ActivityExpression::CounterEntryCount(slot),
        operator: ActivityComparison::Greater,
        right: integer(0),
    }
}
pub(super) fn program(
    node: NodeId,
    operations: Vec<ActivityOperation>,
) -> Result<GraphActivityNodeProgram, CurioSynthesisRoomError> {
    Ok(GraphActivityNodeProgram::new(
        node,
        ActivityProgramDefinition::new(
            ActivityProgramId::new(node.get()).ok_or(CurioSynthesisRoomError::InvalidPolicy)?,
            operations,
        )
        .map_err(|_| CurioSynthesisRoomError::InvalidPolicy)?,
    ))
}
