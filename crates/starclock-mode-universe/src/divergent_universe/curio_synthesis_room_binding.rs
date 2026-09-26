//! Whole-definition authentication and generated choice prefixes, no new engine.

use super::{
    BoundCurioSynthesisRoom, CANCEL_SYNTHESIS, CompiledCurioSynthesisRoom, CurioSynthesisRoomError,
    CurioSynthesisRoomPhase, LEAVE_SYNTHESIS, OPEN_LIMIT, OPEN_SYNTHESIS, SynthesisService,
    choices, count, integer, invalid, optional, set,
};
use crate::divergent_universe::{
    DivergentUniverseLogicalScopeKind,
    curio_synthesis::{AcceptedCurioSynthesis, settlement_operations},
    state::WORKBENCH_SLOT,
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeCondition, ActivityEdgeDefinition,
    ActivityGeneratedBoundaryResolution, ActivityInteractionBindings, ActivityOperation,
    ActivityOptionId, ActivityPlayerView, ActivityRngStreams, ActivityStateHash, ActivityValue,
    GraphActivity, GraphActivityCommandError, GraphActivityDefinition,
};
use starclock_data::divergent_universe_curio_catalog::DivergentUniverseCurioStateId;
use std::sync::Arc;

impl CompiledCurioSynthesisRoom {
    /// Bind only an exact fragment, exit, slots and three-level logical scopes.
    /// No injected edges, alternative entry, program, random offers/checkpoints
    /// or altered entry-lifetime prefix may impersonate the service.
    pub fn bind(
        &self,
        definition: Arc<GraphActivityDefinition>,
    ) -> Result<BoundCurioSynthesisRoom, CurioSynthesisRoomError> {
        let graph = definition.graph();
        let owns = |id| self.fragment.nodes.iter().any(|node| node.id() == id);
        let exit = ActivityEdgeDefinition::new(
            self.context.exit_edge(),
            self.menu,
            self.context.successor(),
            ActivityEdgeCondition::Always,
            0,
            1,
        )
        .map_err(|_| CurioSynthesisRoomError::DefinitionMismatch)?;
        if !graph.edges().contains(&exit)
            || graph.edges().iter().any(|edge| {
                (owns(edge.from()) && edge != &exit && !self.fragment.edges.contains(edge))
                    || (!owns(edge.from())
                        && owns(edge.to())
                        && edge.to() != self.context.entry_node())
            })
            || self
                .fragment
                .nodes
                .iter()
                .any(|node| graph.node(node.id()) != Some(node))
            || self
                .fragment
                .edges
                .iter()
                .any(|edge| !graph.edges().contains(edge))
            || self.fragment.programs.iter().any(|program| {
                let expected = if program.node() == self.context.entry_node() {
                    &self.entry_program
                } else {
                    program
                };
                !definition.programs().contains(expected)
            })
            || self
                .slots
                .iter()
                .any(|slot| !definition.state_definition().slots().contains(slot))
            || self.fragment.nodes.iter().any(|node| {
                !definition
                    .state_definition()
                    .logical_scopes()
                    .bindings()
                    .iter()
                    .any(|binding| {
                        let path = binding.path();
                        binding.node() == node.id()
                            && path.len() == 3
                            && path[0].class() == DivergentUniverseLogicalScopeKind::Run.class_id()
                            && path[0].key() == 1
                            && path[1].class()
                                == DivergentUniverseLogicalScopeKind::Plane.class_id()
                            && path[1].key() == u64::from(self.context.plane_ordinal)
                            && path[2].class() == DivergentUniverseLogicalScopeKind::Node.class_id()
                            && path[2].key() == u64::from(self.context.position_ordinal)
                    })
            })
            || definition
                .random_offers()
                .iter()
                .any(|offer| owns(offer.node()))
            || definition
                .random_checkpoints()
                .iter()
                .any(|checkpoint| owns(checkpoint.node()))
        {
            return Err(CurioSynthesisRoomError::DefinitionMismatch);
        }
        Ok(BoundCurioSynthesisRoom {
            room: self.clone(),
            definition,
        })
    }
}

impl BoundCurioSynthesisRoom {
    /// Non-mutating observation. Foreign whole definitions, decisions or nodes
    /// yield None, including structurally changed programs/bootstrap/registry.
    #[must_use]
    pub fn offered(&self, activity: &GraphActivity) -> Option<CurioSynthesisRoomPhase> {
        let actual = activity.definition();
        if !Arc::ptr_eq(actual, &self.definition)
            && (actual.identity() != self.definition.identity()
                || actual.graph().digest() != self.definition.graph().digest()
                || actual.state_definition() != self.definition.state_definition()
                || actual.participants().digest() != self.definition.participants().digest()
                || actual.programs() != self.definition.programs()
                || actual.bootstrap() != self.definition.bootstrap()
                || actual.random_offers() != self.definition.random_offers()
                || actual.random_checkpoints() != self.definition.random_checkpoints()
                || actual
                    .interactions()
                    .map(ActivityInteractionBindings::bindings)
                    != self
                        .definition
                        .interactions()
                        .map(ActivityInteractionBindings::bindings)
                || actual
                    .interactions()
                    .map(|bindings| bindings.registry().digest())
                    != self
                        .definition
                        .interactions()
                        .map(|bindings| bindings.registry().digest()))
        {
            return None;
        }
        let view = activity.player_view();
        if view.decision()?.kind() != ActivityDecisionKind::Service {
            return None;
        }
        let node = view.current_node();
        if node == self.room.menu {
            Some(CurioSynthesisRoomPhase::Menu)
        } else if node == self.room.first {
            Some(CurioSynthesisRoomPhase::FirstInput)
        } else if node == self.room.second {
            Some(CurioSynthesisRoomPhase::SecondInput)
        } else if node == self.room.output {
            Some(CurioSynthesisRoomPhase::Confirmation)
        } else {
            None
        }
    }

    /// Authenticate the offered ID before RNG. Input/cache changes, sampling,
    /// confirmation inventory/rewards, receipts and following graph execution
    /// each use ONE shared generated-choice transaction. Failure restores exact
    /// bytes/events/RNG. Cancellation is available only before sampling; once a
    /// sample exists, its offered output must be confirmed, without free rerolls.
    pub fn choose(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<ActivityGeneratedBoundaryResolution<()>, GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        let phase = self
            .offered(activity)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        activity.choose_option_with_generated_prefix(expected, decision, selected, |view, rng| {
            let mut operations = self
                .room
                .service
                .generate(view, phase, selected.get(), rng)?;
            operations.push(set(
                self.room.service.slots.accepted,
                ActivityValue::Boolean(true),
            ));
            Ok((operations, ()))
        })
    }
}

impl SynthesisService {
    fn generate(
        &self,
        view: &ActivityPlayerView,
        phase: CurioSynthesisRoomPhase,
        selected: u64,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let slots = self.slots;
        if optional(view, WORKBENCH_SLOT)? != Some(self.workbench) {
            return Err(invalid());
        }
        let first = optional(view, slots.first)?;
        let second = optional(view, slots.second)?;
        let cached = choices(view, slots.choices)?;
        if phase == CurioSynthesisRoomPhase::Menu {
            if first.is_some() || second.is_some() || !cached.is_empty() {
                return Err(invalid());
            }
            if selected == LEAVE_SYNTHESIS {
                return Ok(Vec::new());
            }
            if selected != OPEN_SYNTHESIS
                || count(view, slots.completed)? >= i64::from(self.policy.limit)
                || count(view, slots.opens)? >= i64::from(OPEN_LIMIT)
            {
                return Err(invalid());
            }
            let inputs = self.offers.first_inputs(view).map_err(|_| invalid())?;
            if inputs.is_empty() {
                return Err(invalid());
            }
            return Ok(vec![
                self.cache(&inputs)?,
                ActivityOperation::AddToSlot {
                    slot: slots.opens,
                    delta: integer(1),
                },
            ]);
        }
        if phase != CurioSynthesisRoomPhase::Confirmation && selected == CANCEL_SYNTHESIS {
            if cached.is_empty()
                || second.is_some()
                || (phase == CurioSynthesisRoomPhase::FirstInput && first.is_some())
                || (phase == CurioSynthesisRoomPhase::SecondInput && first.is_none())
            {
                return Err(invalid());
            }
            return Ok(self.clear_selection());
        }
        if !cached.contains(&(selected, 1)) {
            return Err(GraphActivityCommandError::DecisionNotOffered);
        }
        let selected_id = self.id(selected)?;
        match phase {
            CurioSynthesisRoomPhase::Menu => Err(invalid()),
            CurioSynthesisRoomPhase::FirstInput => {
                if first.is_some() || second.is_some() {
                    return Err(invalid());
                }
                let inputs = self
                    .offers
                    .second_inputs(view, &selected_id)
                    .map_err(|_| invalid())?;
                if inputs.is_empty() {
                    return Err(invalid());
                }
                Ok(vec![
                    set(slots.first, ActivityValue::OptionalId(Some(selected))),
                    self.cache(&inputs)?,
                ])
            }
            CurioSynthesisRoomPhase::SecondInput => {
                if second.is_some() {
                    return Err(invalid());
                }
                let first_id = self.id(first.ok_or_else(invalid)?)?;
                let plan = self
                    .offers
                    .plan(view, [first_id, selected_id])
                    .map_err(|_| invalid())?;
                let sampled = plan.sample(rng).map_err(|_| invalid())?;
                Ok(vec![
                    set(slots.second, ActivityValue::OptionalId(Some(selected))),
                    self.cache(&sampled)?,
                ])
            }
            CurioSynthesisRoomPhase::Confirmation => {
                if cached.is_empty()
                    || cached.len() > 3
                    || count(view, slots.completed)? >= i64::from(self.policy.limit)
                {
                    return Err(invalid());
                }
                let pair = [
                    self.id(first.ok_or_else(invalid)?)?,
                    self.id(second.ok_or_else(invalid)?)?,
                ];
                let plan = self
                    .offers
                    .plan(view, pair.clone())
                    .map_err(|_| invalid())?;
                for (key, value) in cached {
                    if *value != 1 || !plan.candidates().contains(&self.id(*key)?) {
                        return Err(invalid());
                    }
                }
                let request =
                    AcceptedCurioSynthesis::new(pair, selected_id).map_err(|_| invalid())?;
                let mut operations =
                    settlement_operations(&self.curios, view, &request, self.receipt, rng)
                        .map_err(|_| invalid())?;
                operations.push(ActivityOperation::AddToSlot {
                    slot: slots.completed,
                    delta: integer(1),
                });
                operations.extend(self.clear_selection());
                Ok(operations)
            }
        }
    }
    fn id(&self, key: u64) -> Result<DivergentUniverseCurioStateId, GraphActivityCommandError> {
        self.curios
            .states()
            .iter()
            .find(|state| state.state_key() == key)
            .map(|state| state.id().clone())
            .ok_or_else(invalid)
    }
    fn cache(
        &self,
        ids: &[DivergentUniverseCurioStateId],
    ) -> Result<ActivityOperation, GraphActivityCommandError> {
        let mut entries = ids
            .iter()
            .map(|id| {
                self.curios
                    .state(id)
                    .map(|state| (state.state_key(), 1))
                    .map_err(|_| invalid())
            })
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_unstable_by_key(|entry| entry.0);
        Ok(ActivityOperation::SetCounterMap {
            slot: self.slots.choices,
            values: entries.into(),
        })
    }
}
