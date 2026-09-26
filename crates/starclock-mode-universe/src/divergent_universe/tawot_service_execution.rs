//! Visible menu/selection commands preserve service state through shared graph loops.
use super::{
    BUY, CANCEL, CARDS, LEAVE, MENU, OPEN_LIMIT, TawotService, TawotServicePhase, clear_offer,
    invalid, set,
};
use crate::divergent_universe::{
    DivergentUniverseFlowInstance,
    state::{
        CURRENCIES_SLOT, TAWOT_ACCEPTED_SLOT, TAWOT_OFFER_SLOT, TAWOT_OPENS_SLOT,
        TAWOT_PURCHASES_SLOT,
    },
};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityExpression, ActivityOperation,
    ActivityOptionId, ActivityPlayerView, ActivityRngLabel, ActivityRngStreams, ActivitySlotId,
    ActivityStateHash, ActivityValue, GraphActivity, GraphActivityCommandError,
};
use starclock_data::divergent_universe_decisions::TawotServiceDefinition;

impl DivergentUniverseFlowInstance {
    /// Explicit caller-selected service configuration, not observed Forge placement.
    #[must_use]
    pub fn initial_tawot_service_level(&self) -> Option<u16> {
        self.tawot_service
            .as_ref()
            .map(|runtime| runtime.definition.forge_level)
    }

    pub(in crate::divergent_universe) fn additional_service_action_budget(&self) -> u32 {
        if self.tawot_service.is_some() {
            2 * OPEN_LIMIT + 2
        } else {
            0
        }
    }

    /// Reports only this flow's currently visible, explicitly admitted service.
    /// It does not certify original Forge placement or every candidate's effects.
    #[must_use]
    pub fn offered_tawot_service(
        &self,
        activity: &GraphActivity,
    ) -> Option<&TawotServiceDefinition> {
        if let Some(rooms) = &self.position_battles {
            return rooms
                .tawot(activity)
                .and_then(|room| room.offered(activity));
        }
        if activity.definition().identity() != self.definition().identity()
            || activity.definition().graph().digest() != self.definition().graph().digest()
        {
            return None;
        }
        let view = activity.player_view();
        let kind = view.decision()?.kind();
        if !matches!(
            (view.current_node().get(), kind),
            (MENU, ActivityDecisionKind::Service) | (CARDS, ActivityDecisionKind::Reward)
        ) {
            return None;
        }
        self.tawot_service
            .as_ref()
            .map(|runtime| &runtime.definition)
    }

    /// Authenticate the visible menu/card option before generating any RNG or
    /// mutation. Payment, same-owner replacement, acquisition effects, allowance,
    /// graph movement and later offers share one rollback boundary. Cancellation
    /// retains the sampled offer; raw graph choices cannot bypass effect gates.
    pub fn choose_tawot_service_option(
        &self,
        activity: &mut GraphActivity,
        expected: ActivityStateHash,
        decision: ActivityDecisionId,
        selected: ActivityOptionId,
    ) -> Result<(), GraphActivityCommandError> {
        if expected != activity.state_hash() {
            return Err(GraphActivityCommandError::StaleStateHash);
        }
        if let Some(rooms) = &self.position_battles {
            rooms
                .tawot(activity)
                .ok_or(GraphActivityCommandError::DecisionNotOffered)?
                .choose(activity, expected, decision, selected)?;
            return Ok(());
        }
        self.offered_tawot_service(activity)
            .ok_or(GraphActivityCommandError::DecisionNotOffered)?;
        let runtime = self.tawot_service.as_ref().ok_or_else(invalid)?;
        activity.choose_option_with_generated_prefix(
            expected,
            decision,
            selected,
            |view, rng| {
                let mut operations = runtime.generate(view, selected.get(), rng)?;
                operations.push(ActivityOperation::SetSlot {
                    slot: TAWOT_ACCEPTED_SLOT,
                    value: ActivityExpression::Literal(ActivityValue::Boolean(true)),
                });
                Ok((operations, ()))
            },
        )?;
        Ok(())
    }
}

impl TawotService {
    fn generate(
        &self,
        view: &ActivityPlayerView,
        selected: u64,
        rng: &mut ActivityRngStreams,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        let phase = match view.current_node().get() {
            MENU => TawotServicePhase::Menu,
            CARDS => TawotServicePhase::Cards,
            _ => return Err(invalid()),
        };
        self.generate_for_phase(view, selected, rng, phase)
    }

    pub(in crate::divergent_universe) fn generate_for_phase(
        &self,
        view: &ActivityPlayerView,
        selected: u64,
        rng: &mut ActivityRngStreams,
        phase: TawotServicePhase,
    ) -> Result<Vec<ActivityOperation>, GraphActivityCommandError> {
        match (phase, selected) {
            (TawotServicePhase::Menu, LEAVE) | (TawotServicePhase::Cards, CANCEL) => Ok(Vec::new()),
            (TawotServicePhase::Menu, BUY) => {
                self.require_funds(view)?;
                let count = integer(view, TAWOT_PURCHASES_SLOT)?;
                let opens = integer(view, TAWOT_OPENS_SLOT)?;
                if count >= i64::from(self.definition.purchase_limit)
                    || opens >= i64::from(OPEN_LIMIT)
                {
                    return Err(invalid());
                }
                // No draw before mandatory reward preflight. Incomplete full
                // Curio programs remain gaps, not grounds for removing candidates.
                for (_, state) in self.states.iter() {
                    if !self
                        .curios
                        .acquisition_rewards_available(view, state)
                        .map_err(|_| invalid())?
                    {
                        return Err(invalid());
                    }
                }
                let cached = offer(view)?;
                let mut operations = vec![set(
                    TAWOT_OPENS_SLOT,
                    opens.checked_add(1).ok_or_else(invalid)?,
                )];
                if cached.is_empty() {
                    let mut candidates =
                        self.states.iter().map(|(key, _)| *key).collect::<Vec<_>>();
                    let mut selected = Vec::new();
                    for _ in 0..self.definition.offer_width {
                        let length = u32::try_from(candidates.len()).map_err(|_| invalid())?;
                        let draw = rng
                            .choose_index(ActivityRngLabel::Reward, 24124, length)
                            .map_err(|_| invalid())?
                            .ok_or_else(invalid)?;
                        let index = usize::try_from(draw.value()).map_err(|_| invalid())?;
                        selected.push(candidates.remove(index));
                    }
                    selected.sort_unstable();
                    operations.push(ActivityOperation::SetOrderedIdSet {
                        slot: TAWOT_OFFER_SLOT,
                        values: selected.into_boxed_slice(),
                    });
                } else if cached.len() != usize::from(self.definition.offer_width)
                    || cached
                        .iter()
                        .any(|key| !self.states.iter().any(|(candidate, _)| candidate == key))
                {
                    return Err(invalid());
                }
                Ok(operations)
            }
            (TawotServicePhase::Cards, selected) => {
                self.require_funds(view)?;
                if !offer(view)?.contains(&selected) {
                    return Err(invalid());
                }
                let count = integer(view, TAWOT_PURCHASES_SLOT)?;
                if count >= i64::from(self.definition.purchase_limit) {
                    return Err(invalid());
                }
                let (_, state) = self
                    .states
                    .iter()
                    .find(|(key, _)| *key == selected)
                    .ok_or_else(invalid)?;
                let mut operations = vec![
                    self.fragments
                        .spend_operation(u64::from(self.definition.fragment_cost))
                        .map_err(|_| invalid())?,
                ];
                operations.extend(
                    self.curios
                        .reacquisition_operations(view, state, rng)
                        .map_err(|_| invalid())?,
                );
                operations.push(set(
                    TAWOT_PURCHASES_SLOT,
                    count.checked_add(1).ok_or_else(invalid)?,
                ));
                operations.push(clear_offer());
                Ok(operations)
            }
            _ => Err(invalid()),
        }
    }

    fn require_funds(&self, view: &ActivityPlayerView) -> Result<(), GraphActivityCommandError> {
        let Some(ActivityValue::BoundedCounterMap(values)) = view
            .slots()
            .iter()
            .find(|slot| slot.id() == CURRENCIES_SLOT)
            .map(|slot| slot.value())
        else {
            return Err(invalid());
        };
        let balance = values
            .iter()
            .find(|(key, _)| *key == self.fragments.key())
            .map_or(0, |(_, value)| *value);
        if balance < i64::from(self.definition.fragment_cost) {
            return Err(invalid());
        }
        Ok(())
    }
}
fn integer(
    view: &ActivityPlayerView,
    id: ActivitySlotId,
) -> Result<i64, GraphActivityCommandError> {
    match view
        .slots()
        .iter()
        .find(|slot| slot.id() == id)
        .map(|slot| slot.value())
    {
        Some(ActivityValue::BoundedInteger(value)) => Ok(*value),
        _ => Err(invalid()),
    }
}
fn offer(view: &ActivityPlayerView) -> Result<&[u64], GraphActivityCommandError> {
    match view
        .slots()
        .iter()
        .find(|slot| slot.id() == TAWOT_OFFER_SLOT)
        .map(|slot| slot.value())
    {
        Some(ActivityValue::OrderedIdSet(values)) => Ok(values),
        _ => Err(invalid()),
    }
}
