//! Explicit source-deck entry choice, not a released mask pool or mask effects.

use starclock_activity::{
    ActivityCondition, ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition,
    ActivityEdgeDefinition, ActivityEdgeId, ActivityExpression, ActivityGraphDefinition,
    ActivityNodeDefinition, ActivityNodeKind, ActivityOperation, ActivityOptionDefinition,
    ActivityOptionId, ActivityProgramDefinition, ActivityProgramId, ActivityScope,
    ActivitySlotDefinition, ActivitySlotId, ActivityStateHash, ActivityStateSource,
    ActivityStateVisibility, ActivityValue, GraphActivity, GraphActivityCommandError,
    GraphActivityNodeProgram, GraphActivityRuntimeError, NodeId, SectionId, SlotCarryPolicy,
    SlotResetPoint,
};
use starclock_data::divergent_universe_domain_decks::DomainDeckDefinition;

use super::domain_deck::{DomainDeck, DomainDeckObservation, DomainDeckSlots};
use super::{
    DivergentUniverseEntryFlowError, DivergentUniverseFlowInstance, DivergentUniverseRuntimeFactory,
};

pub(super) const NODE: NodeId = NodeId::new(24_201).expect("nonzero source-deck node");
const SELECTED_DECK: ActivitySlotId = slot(70);
const SLOTS: DomainDeckSlots = DomainDeckSlots {
    draw: slot(66),
    discard: slot(67),
    selected: slot(68),
    accepted: slot(69),
};

#[derive(Clone, Debug)]
pub(super) struct SourceDeckSelection {
    entries: Box<[(DomainDeckDefinition, DomainDeck)]>,
}

impl SourceDeckSelection {
    pub(super) fn compile(
        factory: &DivergentUniverseRuntimeFactory,
    ) -> Result<Self, DivergentUniverseEntryFlowError> {
        let entries = factory
            .decision_catalog()
            .domain_decks()
            .iter()
            .map(|definition| {
                let deck = factory
                    .compile_domain_deck(&definition.key, 3, SLOTS)
                    .map_err(|_| invalid_definition())?;
                Ok((definition.clone(), deck.deferred_initialization()))
            })
            .collect::<Result<Box<[_]>, DivergentUniverseEntryFlowError>>()?;
        if entries.is_empty() {
            return Err(invalid_definition());
        }
        Ok(Self { entries })
    }

    pub(super) fn slots(
        &self,
    ) -> Result<Vec<ActivitySlotDefinition>, DivergentUniverseEntryFlowError> {
        let mut slots = self.entries[0]
            .1
            .slot_definitions()
            .map_err(|_| invalid_definition())?;
        slots.push(
            ActivitySlotDefinition::new_with_policy(
                SELECTED_DECK,
                ActivityScope::Activity,
                ActivityValue::OptionalId(None),
                None,
                None,
                vec![SlotResetPoint::ActivityStart],
                SlotCarryPolicy::CarryExact,
                ActivityStateVisibility::Player,
                ActivityStateSource::new(70).expect("nonzero source"),
            )
            .map_err(|_| invalid_definition())?,
        );
        Ok(slots)
    }

    pub(super) fn attach(
        &self,
        graph: ActivityGraphDefinition,
        programs: &mut Vec<GraphActivityNodeProgram>,
    ) -> Result<ActivityGraphDefinition, DivergentUniverseEntryFlowError> {
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
        let edge = ActivityEdgeId::new(24_201).expect("nonzero edge");
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
        let options = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, (definition, _))| {
                ActivityOptionDefinition::new(
                    option(index),
                    0,
                    unselected(),
                    vec![
                        ActivityOperation::Require(unselected()),
                        ActivityOperation::Require(ActivityCondition::Boolean(
                            ActivityExpression::Slot(SLOTS.accepted),
                        )),
                        ActivityOperation::SetSlot {
                            slot: SELECTED_DECK,
                            value: ActivityExpression::Literal(ActivityValue::OptionalId(Some(
                                option(index).get(),
                            ))),
                        },
                        ActivityOperation::SetOrderedIdSet {
                            slot: SLOTS.draw,
                            values: definition
                                .cards
                                .iter()
                                .map(|card| card.instance.get())
                                .collect(),
                        },
                        ActivityOperation::SetOrderedIdSet {
                            slot: SLOTS.discard,
                            values: Box::new([]),
                        },
                        ActivityOperation::SetSlot {
                            slot: SLOTS.selected,
                            value: ActivityExpression::Literal(ActivityValue::OptionalId(None)),
                        },
                        ActivityOperation::Traverse(edge),
                    ],
                )
            })
            .collect();
        programs.push(GraphActivityNodeProgram::new(
            NODE,
            ActivityProgramDefinition::new(
                ActivityProgramId::new(24_201).expect("nonzero program"),
                vec![ActivityOperation::Offer {
                    kind: ActivityDecisionKind::Preparation,
                    options,
                }],
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

impl DivergentUniverseFlowInstance {
    #[must_use]
    pub fn has_source_deck_selection(&self) -> bool {
        self.source_deck_selection.is_some()
    }

    /// Exposes explicit source definitions only at the accepted entry offer.
    /// Option order is the catalog's stable key order, with no mask RNG or unlock claim.
    pub fn source_deck_options(
        &self,
        activity: &GraphActivity,
    ) -> Result<Box<[(ActivityOptionId, &DomainDeckDefinition)]>, GraphActivityCommandError> {
        let selection = self.source_deck_binding(activity)?;
        let view = activity.player_view();
        let offer = view
            .decision()
            .filter(|offer| offer.kind() == ActivityDecisionKind::Preparation)
            .ok_or_else(invalid)?;
        if activity.current_node() != NODE || offer.options().len() != selection.entries.len() {
            return Err(invalid());
        }
        Ok(selection
            .entries
            .iter()
            .enumerate()
            .map(|(index, (definition, _))| (option(index), definition))
            .collect())
    }

    /// Initializes exactly one authored deck and advances atomically. Stale,
    /// unoffered, duplicate and foreign-flow choices leave state/RNG unchanged.
    pub fn choose_source_deck(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        self.source_deck_options(activity)?;
        activity.choose_option_with_generated_prefix(expected, decision, selected, |view, _| {
            if view
                .slots()
                .iter()
                .find(|slot| slot.id() == SELECTED_DECK)
                .is_none_or(|slot| slot.value() != &ActivityValue::OptionalId(None))
            {
                return Err(invalid());
            }
            for id in [SLOTS.draw, SLOTS.discard] {
                if view
                    .slots()
                    .iter()
                    .find(|slot| slot.id() == id)
                    .is_none_or(|slot| slot.value() != &ActivityValue::OrderedIdSet(Box::new([])))
                {
                    return Err(invalid());
                }
            }
            Ok((
                vec![ActivityOperation::SetSlot {
                    slot: SLOTS.accepted,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
                }],
                (),
            ))
        })?;
        Ok(())
    }

    /// Returns the selected source definition and its authoritative pile view.
    /// It does not draw cards or execute source mask effects. A selected deck
    /// persists across current baseline battles; domain routing is still unbound.
    pub fn selected_source_deck(
        &self,
        activity: &GraphActivity,
    ) -> Result<Option<(&DomainDeckDefinition, DomainDeckObservation)>, GraphActivityCommandError>
    {
        let selection = self.source_deck_binding(activity)?;
        let view = activity.player_view();
        let selected = match view
            .slots()
            .iter()
            .find(|slot| slot.id() == SELECTED_DECK)
            .map(|slot| slot.value())
        {
            Some(ActivityValue::OptionalId(None)) => return Ok(None),
            Some(ActivityValue::OptionalId(Some(id))) => *id,
            _ => return Err(invalid()),
        };
        let (definition, deck) = selection
            .entries
            .iter()
            .enumerate()
            .find(|(index, _)| option(*index).get() == selected)
            .map(|(_, entry)| entry)
            .ok_or_else(invalid)?;
        Ok(Some((definition, deck.observe(activity)?)))
    }

    fn source_deck_binding(
        &self,
        activity: &GraphActivity,
    ) -> Result<&SourceDeckSelection, GraphActivityCommandError> {
        if activity.definition().identity() != self.definition.identity()
            || activity.definition().graph().digest() != self.definition.graph().digest()
            || activity.definition().state_definition() != self.definition.state_definition()
        {
            return Err(invalid());
        }
        self.source_deck_selection.as_deref().ok_or_else(invalid)
    }
}

fn unselected() -> ActivityCondition {
    ActivityCondition::Equal(
        ActivityExpression::Slot(SELECTED_DECK),
        ActivityExpression::Literal(ActivityValue::OptionalId(None)),
    )
}
fn option(index: usize) -> ActivityOptionId {
    ActivityOptionId::new(u64::try_from(index + 1).expect("bounded source deck count"))
        .expect("positive option")
}
const fn slot(raw: u32) -> ActivitySlotId {
    ActivitySlotId::new(raw).expect("nonzero source-deck slot")
}
fn invalid_definition() -> DivergentUniverseEntryFlowError {
    DivergentUniverseEntryFlowError::InvalidActivityDefinition
}
fn invalid() -> GraphActivityCommandError {
    GraphActivityCommandError::Runtime(GraphActivityRuntimeError::InvalidBoundaryProgram)
}
