use starclock_combat::{
    AiStateId, Command, DecisionPoint, UnitId,
    catalog::encounter::{AiGraphDefinition, AiNoTargetFallback, AiTransitionTiming},
    rng::types::RngSeed,
    rule::model::ConditionExpr,
};

use crate::{EnemyController, EnemyCursor, EnemyDecision, EnemyDecisionError, select};

impl EnemyController {
    #[must_use]
    pub fn new(seed: RngSeed) -> Self {
        Self {
            rng: starclock_combat::rng::engine::DeterministicRng::from_seed(seed),
            cursors: std::collections::BTreeMap::new(),
        }
    }

    #[must_use]
    pub const fn draw_count(&self) -> u64 {
        self.rng.draw_count()
    }

    /// Returns one controller-owned graph/state/turn cursor.
    #[must_use]
    pub fn cursor(&self, actor: UnitId) -> Option<(starclock_combat::AiGraphId, AiStateId, u16)> {
        self.cursors
            .get(&actor)
            .map(|cursor| (cursor.graph, cursor.state, cursor.turns))
    }

    /// Stabilizes automatic transitions and selects one exact offered command.
    pub fn decide(
        &mut self,
        graph: &AiGraphDefinition,
        initial_state: AiStateId,
        actor: UnitId,
        decision: &DecisionPoint,
        mut condition: impl FnMut(&ConditionExpr) -> bool,
    ) -> Result<EnemyDecision, EnemyDecisionError> {
        let state = self
            .cursors
            .get(&actor)
            .filter(|cursor| cursor.graph == graph.id())
            .map_or(initial_state, |cursor| cursor.state);
        let result = self.decide_offered(
            graph,
            state,
            actor,
            decision.legal_commands(),
            &mut condition,
        )?;
        let turns = self.cursors.get(&actor).map_or(0, |cursor| cursor.turns);
        self.cursors.insert(
            actor,
            EnemyCursor {
                graph: graph.id(),
                state: result.state(),
                turns,
            },
        );
        Ok(result)
    }

    /// Applies the first passing canonical transition at a completed boundary.
    pub fn settle(
        &mut self,
        graph: &AiGraphDefinition,
        actor: UnitId,
        timing: AiTransitionTiming,
        mut condition: impl FnMut(&ConditionExpr) -> bool,
    ) -> Result<(AiStateId, u16), EnemyDecisionError> {
        let cursor = self
            .cursors
            .get_mut(&actor)
            .filter(|cursor| cursor.graph == graph.id())
            .ok_or(EnemyDecisionError::MissingState)?;
        let state = graph
            .state(cursor.state)
            .ok_or(EnemyDecisionError::MissingState)?;
        if let Some(transition) = state
            .transitions()
            .iter()
            .find(|item| item.timing() == timing && condition(item.condition()))
        {
            cursor.state = transition.target();
            let target = graph
                .state(cursor.state)
                .ok_or(EnemyDecisionError::MissingState)?;
            if target.resets_turn_counter() {
                cursor.turns = 0;
            }
        }
        if timing == AiTransitionTiming::AfterAction {
            cursor.turns = cursor
                .turns
                .checked_add(1)
                .ok_or(EnemyDecisionError::AutomaticTransitionBudget)?;
        }
        Ok((cursor.state, cursor.turns))
    }

    pub(crate) fn decide_offered(
        &mut self,
        graph: &AiGraphDefinition,
        initial_state: AiStateId,
        actor: UnitId,
        commands: &[Command],
        condition: &mut impl FnMut(&ConditionExpr) -> bool,
    ) -> Result<EnemyDecision, EnemyDecisionError> {
        let mut state_id = initial_state;
        let mut transition_count: u16 = 0;
        loop {
            let state = graph
                .state(state_id)
                .ok_or(EnemyDecisionError::MissingState)?;
            let transition = state.transitions().iter().find(|transition| {
                transition.timing() == AiTransitionTiming::AutomaticBeforeDecision
                    && condition(transition.condition())
            });
            if let Some(transition) = transition {
                if transition_count == graph.automatic_transition_budget() {
                    return Err(EnemyDecisionError::AutomaticTransitionBudget);
                }
                state_id = transition.target();
                transition_count += 1;
                continue;
            }
            let candidates = state
                .candidates()
                .iter()
                .filter(|candidate| condition(candidate.condition()))
                .collect::<Vec<_>>();
            if candidates.is_empty() {
                return select::fallback(
                    commands,
                    actor,
                    state_id,
                    state.mandatory_fallback(),
                    None,
                );
            }
            let priority = candidates[0].priority();
            let tied = candidates
                .into_iter()
                .take_while(|candidate| candidate.priority() == priority)
                .collect::<Vec<_>>();
            let (candidate, draw) = select::candidate(&mut self.rng, &tied)?;
            if let Some(command) = select::offered(commands, actor, candidate.ability()) {
                return Ok(EnemyDecision {
                    state: state_id,
                    candidate: Some(candidate.id()),
                    command,
                    draw,
                });
            }
            match candidate.no_target() {
                AiNoTargetFallback::UseFallbackAbility(ability) => {
                    return select::fallback(
                        commands,
                        actor,
                        state_id,
                        ability,
                        Some(candidate.id()),
                    );
                }
                AiNoTargetFallback::StayInState => {
                    return select::fallback(
                        commands,
                        actor,
                        state_id,
                        state.mandatory_fallback(),
                        Some(candidate.id()),
                    );
                }
                AiNoTargetFallback::Transition(target) => state_id = target,
                AiNoTargetFallback::SkipAction => {
                    return Err(EnemyDecisionError::SkipActionUnsupported);
                }
                AiNoTargetFallback::Fault => return Err(EnemyDecisionError::NoTargetFault),
            }
            transition_count = transition_count
                .checked_add(1)
                .ok_or(EnemyDecisionError::AutomaticTransitionBudget)?;
            if transition_count > graph.automatic_transition_budget() {
                return Err(EnemyDecisionError::AutomaticTransitionBudget);
            }
        }
    }
}
