//! Standard Universe runtime facade over the generic graph Activity.

use std::sync::Arc;

use starclock_activity::{
    ActivityBattleHandoff, ActivityDecisionId, ActivityOptionId, ActivityPlayerView,
    ActivityPreparationBoundary, ActivityPreparationView, ActivityRosterLock, ActivityScopePath,
    ActivityStateHash, AttemptId, BattleResult, BattleSequence, GraphActivity,
    GraphActivityBattleError, GraphActivityBattleResolution, GraphActivityCommandError,
    GraphActivityEncounterError, GraphActivityPreparationResolution, GraphActivityResolution,
    GraphActivityStartError, ParticipantLock,
};

use crate::{battle_overlay::UniverseEncounterOverlay, topology::EncounterOptionBinding};

pub struct StandardUniverseActivity {
    graph: GraphActivity,
    participants: Arc<ParticipantLock>,
    encounter_options: Arc<[EncounterOptionBinding]>,
    overlay: Arc<UniverseEncounterOverlay>,
}

impl StandardUniverseActivity {
    pub(crate) fn new(
        graph: GraphActivity,
        participants: Arc<ParticipantLock>,
        encounter_options: Arc<[EncounterOptionBinding]>,
        overlay: Arc<UniverseEncounterOverlay>,
    ) -> Self {
        Self {
            graph,
            participants,
            encounter_options,
            overlay,
        }
    }

    #[must_use]
    pub const fn graph(&self) -> &GraphActivity {
        &self.graph
    }
    #[must_use]
    pub fn view(&self) -> ActivityPlayerView {
        self.graph.player_view()
    }
    #[must_use]
    pub fn preparation_view(&self) -> Option<ActivityPreparationView> {
        self.graph.preparation_view()
    }

    pub fn choose_option(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
    ) -> Result<Box<[starclock_activity::ActivityTransactionEvent]>, GraphActivityCommandError>
    {
        self.graph
            .choose_option(expected_state_hash, decision, option)
    }

    pub fn engage_encounter(
        &mut self,
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        option: ActivityOptionId,
        technique_points: u16,
    ) -> Result<GraphActivityPreparationResolution, StandardUniverseEncounterError> {
        let member = self
            .encounter_options
            .binary_search_by_key(&option, |binding| binding.option())
            .ok()
            .map(|index| self.encounter_options[index].member())
            .ok_or(StandardUniverseEncounterError::UnknownEncounterOption)?;
        let binding = self
            .overlay
            .binding(member)
            .ok_or(StandardUniverseEncounterError::MissingBattleOverlay(member))?;
        let current = self.graph.current_node();
        let section = self
            .graph
            .definition()
            .graph()
            .node(current)
            .ok_or(StandardUniverseEncounterError::InvalidScope)?
            .section();
        let instance = self.graph.instance();
        let path = ActivityScopePath::new(instance)
            .enter_section(section)
            .and_then(|path| path.enter_node(current))
            .and_then(|path| {
                path.enter_attempt(AttemptId::new(1).expect("static attempt ID is non-zero"))
            })
            .map_err(|_| StandardUniverseEncounterError::InvalidScope)?;
        let roster = ActivityRosterLock::new(
            ActivityScopePath::new(instance),
            self.participants.as_ref().clone(),
        )
        .map_err(|_| StandardUniverseEncounterError::InvalidScope)?;
        let sequence = BattleSequence::new(current.get())
            .ok_or(StandardUniverseEncounterError::InvalidScope)?;
        self.graph
            .engage_encounter(
                expected_state_hash,
                decision,
                option,
                starclock_activity::ActivityBattlePreparationRequest::new(
                    path,
                    roster,
                    sequence,
                    technique_points,
                    Arc::clone(binding.preparation()),
                ),
            )
            .map_err(StandardUniverseEncounterError::Activity)
    }

    pub fn choose_preparation_option(
        &mut self,
        expected_state_hash: ActivityStateHash,
        option: ActivityOptionId,
    ) -> Result<ActivityPreparationBoundary, GraphActivityEncounterError> {
        self.graph
            .choose_preparation_option(expected_state_hash, option)
    }

    pub fn start_pending_battle(
        &mut self,
        expected_state_hash: ActivityStateHash,
    ) -> Result<ActivityBattleHandoff, StandardUniverseBattleStartError> {
        let digest = self
            .graph
            .pending_battle()
            .ok_or(StandardUniverseBattleStartError::MissingPendingBattle)?
            .battle_spec_digest();
        let binding = self
            .overlay
            .binding_for_spec(digest.bytes())
            .ok_or(StandardUniverseBattleStartError::MissingBattleOverlay)?;
        self.graph
            .start_pending_battle(expected_state_hash, Arc::clone(binding.contract()))
            .map_err(StandardUniverseBattleStartError::Activity)
    }

    pub fn submit_pending_battle_result(
        &mut self,
        expected_state_hash: ActivityStateHash,
        result: BattleResult,
    ) -> Result<GraphActivityBattleResolution, GraphActivityBattleError> {
        self.graph
            .submit_pending_battle_result(expected_state_hash, result)
    }
}

pub struct StandardUniverseStartResolution {
    activity: StandardUniverseActivity,
    events: Box<[starclock_activity::ActivityTransactionEvent]>,
}

impl StandardUniverseStartResolution {
    pub(crate) fn new(
        resolution: GraphActivityResolution,
        participants: Arc<ParticipantLock>,
        encounter_options: Arc<[EncounterOptionBinding]>,
        overlay: Arc<UniverseEncounterOverlay>,
    ) -> Self {
        let events = resolution.events().to_vec().into_boxed_slice();
        let activity = StandardUniverseActivity::new(
            resolution.into_activity(),
            participants,
            encounter_options,
            overlay,
        );
        Self { activity, events }
    }
    #[must_use]
    pub fn into_activity(self) -> StandardUniverseActivity {
        self.activity
    }
    #[must_use]
    pub fn events(&self) -> &[starclock_activity::ActivityTransactionEvent] {
        &self.events
    }
    #[must_use]
    pub fn view(&self) -> ActivityPlayerView {
        self.activity.view()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseStartError {
    MissingEncounterOverlay,
    Activity(GraphActivityStartError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseEncounterError {
    UnknownEncounterOption,
    MissingBattleOverlay(crate::id::EncounterMemberId),
    InvalidScope,
    Activity(GraphActivityEncounterError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardUniverseBattleStartError {
    MissingPendingBattle,
    MissingBattleOverlay,
    Activity(starclock_activity::ActivityBattleSettlementError),
}

pub(crate) fn start(
    resolution: Result<GraphActivityResolution, GraphActivityStartError>,
    participants: Arc<ParticipantLock>,
    encounter_options: Arc<[EncounterOptionBinding]>,
    overlay: Option<Arc<UniverseEncounterOverlay>>,
) -> Result<StandardUniverseStartResolution, StandardUniverseStartError> {
    let overlay = overlay.ok_or(StandardUniverseStartError::MissingEncounterOverlay)?;
    let resolution = resolution.map_err(StandardUniverseStartError::Activity)?;
    Ok(StandardUniverseStartResolution::new(
        resolution,
        participants,
        encounter_options,
        overlay,
    ))
}
