//! Swarm Disaster offered-command scoring over the generic Activity baseline.

use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId, ActivityOptionId,
    ActivityTransactionState, NodeId,
};

use crate::{
    baseline_controller::{
        ActivityBaselineController, ActivityBaselineDecision, ActivityBaselineHints,
        ActivityDecisionError, ActivityHintError, ActivityOptionHint, ActivityScoreComponents,
    },
    digest::Encoder,
};

#[cfg(test)]
use crate::battle_materialization::UniverseBattleRoster;

#[cfg(test)]
use super::seeded_run::{SwarmSeededRunReport, SwarmSeededRunRequest};
use super::{SwarmDisasterRuntimeInstance, seeded_run::SwarmSeededRunError};

/// Stable caller-selected baseline controller revision.
pub const SWARM_DISASTER_BASELINE_CONTROLLER_REVISION: &str =
    "swarm-disaster-baseline-controller-v1";

const ROUTE_OPTION_BASE: u64 = 0x5344_0100_0000_0000;
const BOSS_OPTION_BASE: u64 = 0x5344_0200_0000_0000;
const COMMAND_FAMILIES: [SwarmCommandFamily; 10] = [
    SwarmCommandFamily::Route,
    SwarmCommandFamily::BossSelection,
    SwarmCommandFamily::DiceControl,
    SwarmCommandFamily::DiceTarget,
    SwarmCommandFamily::Countdown,
    SwarmCommandFamily::Communing,
    SwarmCommandFamily::Progression,
    SwarmCommandFamily::Reward,
    SwarmCommandFamily::Service,
    SwarmCommandFamily::AdventureOutcome,
];

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub(super) enum SwarmCommandFamily {
    Route = 0,
    BossSelection = 1,
    DiceControl = 2,
    DiceTarget = 3,
    Countdown = 4,
    Communing = 5,
    Progression = 6,
    Reward = 7,
    Service = 8,
    AdventureOutcome = 9,
}

impl SwarmCommandFamily {
    pub(super) const fn decision_kind(self) -> ActivityDecisionKind {
        match self {
            Self::Route => ActivityDecisionKind::Route,
            Self::BossSelection => ActivityDecisionKind::Encounter,
            Self::DiceControl
            | Self::DiceTarget
            | Self::Countdown
            | Self::Communing
            | Self::Progression => ActivityDecisionKind::Choice,
            Self::Reward => ActivityDecisionKind::Reward,
            Self::Service => ActivityDecisionKind::Service,
            Self::AdventureOutcome => ActivityDecisionKind::ExternalOutcome,
        }
    }
}

// The complete baseline currently reaches Route and BossSelection. The other
// frozen offered families are consumed by the following CLI/agent/MCP batches.
#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum SwarmOfferedAction {
    Traverse {
        edge: ActivityEdgeId,
        target: NodeId,
    },
    SelectBoss {
        plane: u8,
        boss: Box<str>,
    },
    SelectDiceControl {
        control: Box<str>,
    },
    SelectDiceTarget {
        target: Option<NodeId>,
    },
    AdjustCountdown {
        delta: i64,
    },
    SelectCommuning {
        choice: Box<str>,
    },
    SelectProgression {
        objective: Box<str>,
    },
    SelectReward {
        source: Box<str>,
        selection: Box<str>,
    },
    PurchaseService {
        service: Box<str>,
    },
    SubmitAdventureOutcome {
        adventure: Box<str>,
        achieved: u32,
    },
}

impl SwarmOfferedAction {
    pub(super) const fn family(&self) -> SwarmCommandFamily {
        match self {
            Self::Traverse { .. } => SwarmCommandFamily::Route,
            Self::SelectBoss { .. } => SwarmCommandFamily::BossSelection,
            Self::SelectDiceControl { .. } => SwarmCommandFamily::DiceControl,
            Self::SelectDiceTarget { .. } => SwarmCommandFamily::DiceTarget,
            Self::AdjustCountdown { .. } => SwarmCommandFamily::Countdown,
            Self::SelectCommuning { .. } => SwarmCommandFamily::Communing,
            Self::SelectProgression { .. } => SwarmCommandFamily::Progression,
            Self::SelectReward { .. } => SwarmCommandFamily::Reward,
            Self::PurchaseService { .. } => SwarmCommandFamily::Service,
            Self::SubmitAdventureOutcome { .. } => SwarmCommandFamily::AdventureOutcome,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SwarmOfferedCommand {
    id: ActivityOptionId,
    authored_priority: i32,
    score: ActivityScoreComponents,
    action: SwarmOfferedAction,
}

impl SwarmOfferedCommand {
    pub(super) const fn new(
        id: ActivityOptionId,
        authored_priority: i32,
        score: ActivityScoreComponents,
        action: SwarmOfferedAction,
    ) -> Self {
        Self {
            id,
            authored_priority,
            score,
            action,
        }
    }

    pub(super) const fn id(&self) -> ActivityOptionId {
        self.id
    }

    pub(super) const fn authored_priority(&self) -> i32 {
        self.authored_priority
    }

    pub(super) const fn action(&self) -> &SwarmOfferedAction {
        &self.action
    }

    pub(super) const fn family(&self) -> SwarmCommandFamily {
        self.action.family()
    }

    pub(super) fn route(
        edge: ActivityEdgeId,
        target: NodeId,
        preferred: bool,
        countdown: i64,
        disarray: i64,
    ) -> Self {
        let survival = bounded_i64(countdown);
        let risk = bounded_i64(disarray);
        Self::new(
            ActivityOptionId::new(ROUTE_OPTION_BASE | u64::from(edge.get()))
                .expect("the Swarm route option prefix is non-zero"),
            i32::from(preferred),
            score(i32::from(preferred) * 100, survival, 0, 0, risk),
            SwarmOfferedAction::Traverse { edge, target },
        )
    }

    pub(super) fn boss(plane: u8, ordinal: usize, boss: &str, preferred: bool) -> Self {
        let ordinal = u64::try_from(ordinal).expect("the bounded boss catalog fits u64");
        Self::new(
            ActivityOptionId::new(BOSS_OPTION_BASE | (u64::from(plane) << 16) | (ordinal + 1))
                .expect("the Swarm boss option prefix is non-zero"),
            i32::from(preferred),
            score(0, 0, 0, i32::from(preferred) * 100, 0),
            SwarmOfferedAction::SelectBoss {
                plane,
                boss: boss.into(),
            },
        )
    }
}

fn bounded_i64(value: i64) -> i32 {
    i32::try_from(value.clamp(-1_000_000, 1_000_000))
        .expect("the explicit clamp keeps the controller score within i32")
}

fn score(
    progress: i32,
    survival: i32,
    resources: i32,
    synergy: i32,
    risk: i32,
) -> ActivityScoreComponents {
    ActivityScoreComponents::new(progress, survival, resources, synergy, risk)
        .expect("internal Swarm baseline scores stay within the generic bound")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SwarmBaselineDecision {
    selected: SwarmOfferedCommand,
    diagnostic: ActivityBaselineDecision,
}

impl SwarmBaselineDecision {
    pub(super) const fn selected(&self) -> &SwarmOfferedCommand {
        &self.selected
    }

    #[cfg(test)]
    pub(super) const fn diagnostic(&self) -> &ActivityBaselineDecision {
        &self.diagnostic
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct SwarmBaselineController {
    generic: ActivityBaselineController,
}

impl SwarmBaselineController {
    pub(super) const REVISION: &'static str = SWARM_DISASTER_BASELINE_CONTROLLER_REVISION;

    pub(super) fn identity_digest() -> [u8; 32] {
        let mut encoder = Encoder::new(b"starclock.swarm-disaster.baseline-controller.identity.v1");
        encoder.text(Self::REVISION);
        encoder.text(ActivityBaselineController::REVISION);
        encoder.text("authored-priority-plus-bounded-activity-score-v1");
        encoder.text("highest-total-then-lowest-option-id-v1");
        encoder.u8(COMMAND_FAMILIES.len() as u8);
        for family in COMMAND_FAMILIES {
            encoder.u8(family as u8);
            encoder.u8(family.decision_kind() as u8);
        }
        encoder.finish()
    }

    pub(super) fn decide(
        self,
        decision: ActivityDecisionId,
        offers: &[SwarmOfferedCommand],
    ) -> Result<SwarmBaselineDecision, SwarmBaselineError> {
        let family = offers
            .first()
            .map(SwarmOfferedCommand::family)
            .ok_or(SwarmBaselineError::EmptyOffer)?;
        if offers.iter().any(|offer| offer.family() != family) {
            return Err(SwarmBaselineError::MixedFamilies);
        }
        let hints = ActivityBaselineHints::new(
            offers
                .iter()
                .map(|offer| ActivityOptionHint::new(offer.id, offer.score))
                .collect(),
        )
        .map_err(SwarmBaselineError::Hints)?;
        let generic_offers = offers
            .iter()
            .map(|offer| (offer.id, offer.authored_priority))
            .collect::<Vec<_>>();
        let diagnostic = self
            .generic
            .decide_offers(decision, family.decision_kind(), &generic_offers, &hints)
            .map_err(SwarmBaselineError::Decision)?;
        let selected = offers
            .iter()
            .find(|offer| offer.id == diagnostic.option())
            .expect("the generic controller returns one exact offered identity")
            .clone();
        Ok(SwarmBaselineDecision {
            selected,
            diagnostic,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SwarmBaselineError {
    EmptyOffer,
    MixedFamilies,
    Hints(ActivityHintError),
    Decision(ActivityDecisionError),
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct SwarmBaselineRunReport {
    run: SwarmSeededRunReport,
    decisions: Box<[SwarmBaselineDecision]>,
    decision_digest: [u8; 32],
}

#[cfg(test)]
impl SwarmBaselineRunReport {
    pub(super) const fn run(&self) -> &SwarmSeededRunReport {
        &self.run
    }

    pub(super) fn decisions(&self) -> &[SwarmBaselineDecision] {
        &self.decisions
    }

    pub(super) const fn decision_digest(&self) -> [u8; 32] {
        self.decision_digest
    }
}

#[cfg(test)]
impl SwarmDisasterRuntimeInstance {
    pub(super) fn execute_baseline_run(
        &self,
        request: SwarmSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<SwarmBaselineRunReport, SwarmSeededRunError> {
        let mut decisions = Vec::new();
        let execution =
            self.execute_seeded_run_recorded_with_decisions(request, roster, Some(&mut decisions))?;
        let decision_digest = decision_digest(execution.report.transcript_digest, &decisions);
        Ok(SwarmBaselineRunReport {
            run: execution.report,
            decisions: decisions.into_boxed_slice(),
            decision_digest,
        })
    }
}

#[cfg(test)]
pub(super) fn decision_digest(
    run_digest: [u8; 32],
    decisions: &[SwarmBaselineDecision],
) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.swarm-disaster.baseline-controller.v1");
    encoder.text(SWARM_DISASTER_BASELINE_CONTROLLER_REVISION);
    encoder.digest(run_digest);
    encoder.u32(u32::try_from(decisions.len()).expect("seeded decision count is bounded"));
    for decision in decisions {
        encoder.u64(decision.diagnostic().decision().get());
        encoder.u8(decision.selected.family() as u8);
        encoder.u64(decision.selected().id().get());
        encoder.u32(
            u32::try_from(decision.diagnostic().scores().len())
                .expect("each offered set is bounded"),
        );
        for score in decision.diagnostic().scores() {
            encoder.u64(score.option().get());
            encoder.i64(score.total());
        }
    }
    encoder.finish()
}

pub(super) fn route_offers(
    instance: &SwarmDisasterRuntimeInstance,
    state: &ActivityTransactionState,
    source: NodeId,
    preferred: NodeId,
) -> Result<Vec<SwarmOfferedCommand>, SwarmSeededRunError> {
    let countdown = instance.countdown(state)?;
    let disarray = instance.disarray_level(state)?;
    instance
        .legal_routes(state, source)
        .iter()
        .map(|edge| {
            let target = instance
                .graph_definition()
                .edges()
                .iter()
                .find(|candidate| candidate.id() == *edge)
                .map(|candidate| candidate.to())
                .ok_or(SwarmSeededRunError::MissingRoute(source))?;
            Ok(SwarmOfferedCommand::route(
                *edge,
                target,
                target == preferred,
                countdown,
                disarray,
            ))
        })
        .collect()
}

pub(super) fn select_offered(
    controller: SwarmBaselineController,
    decision: ActivityDecisionId,
    offers: &[SwarmOfferedCommand],
    decisions: &mut Option<&mut Vec<SwarmBaselineDecision>>,
) -> Result<SwarmOfferedCommand, SwarmSeededRunError> {
    let selected = controller
        .decide(decision, offers)
        .map_err(SwarmSeededRunError::Controller)?;
    let command = selected.selected().clone();
    if let Some(decisions) = decisions.as_deref_mut() {
        decisions.push(selected);
    }
    Ok(command)
}
