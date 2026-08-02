//! Gold and Gears offered-command scoring over the generic Activity baseline.

use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityEdgeId, ActivityOptionId, NodeId,
};

use crate::baseline_controller::{
    ActivityBaselineController, ActivityBaselineDecision, ActivityBaselineHints,
    ActivityDecisionError, ActivityHintError, ActivityOptionHint, ActivityScoreComponents,
};

use super::{
    GoldAndGearsRuntimeInstance, GoldAndGearsSeededRunError, GoldAndGearsSeededRunReport,
    GoldAndGearsSeededRunRequest,
};
use crate::{battle_materialization::UniverseBattleRoster, digest::Encoder};

const GOLD_AND_GEARS_BASELINE_CONTROLLER_DOMAIN: &str = "gold-and-gears-baseline-controller";

/// Caller-owned controller identity included in the composed configuration root.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoldAndGearsControllerIdentity<'a> {
    pub id: &'a str,
    pub digest: [u8; 32],
}

const ROUTE_OPTION_BASE: u64 = 0x4747_0100_0000_0000;
const BOSS_OPTION_BASE: u64 = 0x4747_0200_0000_0000;
const COMMAND_FAMILIES: [GoldAndGearsCommandFamily; 10] = [
    GoldAndGearsCommandFamily::Route,
    GoldAndGearsCommandFamily::BossSelection,
    GoldAndGearsCommandFamily::DiceLoadout,
    GoldAndGearsCommandFamily::DiceAction,
    GoldAndGearsCommandFamily::Cognition,
    GoldAndGearsCommandFamily::Knowledge,
    GoldAndGearsCommandFamily::Conundrum,
    GoldAndGearsCommandFamily::Reward,
    GoldAndGearsCommandFamily::Service,
    GoldAndGearsCommandFamily::AdventureOutcome,
];

/// Player-selectable command families exposed by the Gold facade.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum GoldAndGearsCommandFamily {
    Route = 0,
    BossSelection = 1,
    DiceLoadout = 2,
    DiceAction = 3,
    Cognition = 4,
    Knowledge = 5,
    Conundrum = 6,
    Reward = 7,
    Service = 8,
    AdventureOutcome = 9,
}

impl GoldAndGearsCommandFamily {
    const fn decision_kind(self) -> ActivityDecisionKind {
        match self {
            Self::Route => ActivityDecisionKind::Route,
            Self::BossSelection => ActivityDecisionKind::Encounter,
            Self::DiceLoadout
            | Self::DiceAction
            | Self::Cognition
            | Self::Knowledge
            | Self::Conundrum => ActivityDecisionKind::Choice,
            Self::Reward => ActivityDecisionKind::Reward,
            Self::Service => ActivityDecisionKind::Service,
            Self::AdventureOutcome => ActivityDecisionKind::ExternalOutcome,
        }
    }
}

/// Exact typed command carried by one authorized offer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoldAndGearsOfferedAction {
    Traverse {
        edge: ActivityEdgeId,
    },
    SelectBoss {
        plane: u8,
        boss: Box<str>,
    },
    SelectDiceLoadout {
        custom_dice: Box<str>,
        faces: Box<[Box<str>]>,
    },
    ActivateDice {
        face: Box<str>,
        targets: Box<[NodeId]>,
    },
    AdjustCognition {
        delta: i64,
    },
    ResolveKnowledge {
        rule: Box<str>,
        targets: Box<[NodeId]>,
    },
    SelectConundrum {
        stats: u8,
        auxiliary: u8,
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

impl GoldAndGearsOfferedAction {
    #[must_use]
    pub const fn family(&self) -> GoldAndGearsCommandFamily {
        match self {
            Self::Traverse { .. } => GoldAndGearsCommandFamily::Route,
            Self::SelectBoss { .. } => GoldAndGearsCommandFamily::BossSelection,
            Self::SelectDiceLoadout { .. } => GoldAndGearsCommandFamily::DiceLoadout,
            Self::ActivateDice { .. } => GoldAndGearsCommandFamily::DiceAction,
            Self::AdjustCognition { .. } => GoldAndGearsCommandFamily::Cognition,
            Self::ResolveKnowledge { .. } => GoldAndGearsCommandFamily::Knowledge,
            Self::SelectConundrum { .. } => GoldAndGearsCommandFamily::Conundrum,
            Self::SelectReward { .. } => GoldAndGearsCommandFamily::Reward,
            Self::PurchaseService { .. } => GoldAndGearsCommandFamily::Service,
            Self::SubmitAdventureOutcome { .. } => GoldAndGearsCommandFamily::AdventureOutcome,
        }
    }
}

/// One canonically identified exact command offered to a controller.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsOfferedCommand {
    id: ActivityOptionId,
    authored_priority: i32,
    score: ActivityScoreComponents,
    action: GoldAndGearsOfferedAction,
}

impl GoldAndGearsOfferedCommand {
    #[must_use]
    pub const fn new(
        id: ActivityOptionId,
        authored_priority: i32,
        score: ActivityScoreComponents,
        action: GoldAndGearsOfferedAction,
    ) -> Self {
        Self {
            id,
            authored_priority,
            score,
            action,
        }
    }

    #[must_use]
    pub const fn id(&self) -> ActivityOptionId {
        self.id
    }

    #[must_use]
    pub const fn authored_priority(&self) -> i32 {
        self.authored_priority
    }

    #[must_use]
    pub const fn action(&self) -> &GoldAndGearsOfferedAction {
        &self.action
    }

    #[must_use]
    pub const fn family(&self) -> GoldAndGearsCommandFamily {
        self.action.family()
    }

    pub(super) fn route(edge: ActivityEdgeId, preferred: bool) -> Self {
        Self::new(
            ActivityOptionId::new(ROUTE_OPTION_BASE | u64::from(edge.get()))
                .expect("the route option prefix is non-zero"),
            i32::from(preferred),
            score(i32::from(preferred) * 100, 0, 0, 0, 0),
            GoldAndGearsOfferedAction::Traverse { edge },
        )
    }

    pub(super) fn boss(plane: u8, ordinal: usize, boss: &str, preferred: bool) -> Self {
        let ordinal = u64::try_from(ordinal).expect("the bounded boss catalog fits u64");
        Self::new(
            ActivityOptionId::new(BOSS_OPTION_BASE | (u64::from(plane) << 16) | (ordinal + 1))
                .expect("the boss option prefix is non-zero"),
            i32::from(preferred),
            score(0, 0, 0, i32::from(preferred) * 100, 0),
            GoldAndGearsOfferedAction::SelectBoss {
                plane,
                boss: boss.into(),
            },
        )
    }
}

fn score(
    progress: i32,
    survival: i32,
    resources: i32,
    synergy: i32,
    risk: i32,
) -> ActivityScoreComponents {
    ActivityScoreComponents::new(progress, survival, resources, synergy, risk)
        .expect("internal Gold baseline scores stay within the generic bound")
}

/// Exact selected offered command plus the generic auditable score breakdown.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBaselineDecision {
    selected: GoldAndGearsOfferedCommand,
    diagnostic: ActivityBaselineDecision,
}

impl GoldAndGearsBaselineDecision {
    #[must_use]
    pub const fn selected(&self) -> &GoldAndGearsOfferedCommand {
        &self.selected
    }

    #[must_use]
    pub const fn diagnostic(&self) -> &ActivityBaselineDecision {
        &self.diagnostic
    }
}

/// Stateless deterministic controller over exact Gold command offers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoldAndGearsBaselineController {
    generic: ActivityBaselineController,
}

impl GoldAndGearsBaselineController {
    pub const ID: &'static str = GOLD_AND_GEARS_BASELINE_CONTROLLER_DOMAIN;

    /// Canonical controller-policy digest suitable for the caller-selected
    /// Controller configuration component.
    #[must_use]
    pub fn identity_digest() -> [u8; 32] {
        let mut encoder = Encoder::new(b"starclock.gold-and-gears.baseline-controller.identity");
        encoder.text(Self::ID);
        encoder.text(ActivityBaselineController::ID);
        encoder.text("authored-priority-plus-bounded-activity-score");
        encoder.text("highest-total-then-lowest-option-id");
        encoder.u8(COMMAND_FAMILIES.len() as u8);
        for family in COMMAND_FAMILIES {
            encoder.u8(family as u8);
            encoder.u8(family.decision_kind() as u8);
        }
        encoder.finish()
    }

    pub fn decide(
        self,
        decision: ActivityDecisionId,
        offers: &[GoldAndGearsOfferedCommand],
    ) -> Result<GoldAndGearsBaselineDecision, GoldAndGearsBaselineError> {
        let family = offers
            .first()
            .map(GoldAndGearsOfferedCommand::family)
            .ok_or(GoldAndGearsBaselineError::EmptyOffer)?;
        if offers.iter().any(|offer| offer.family() != family) {
            return Err(GoldAndGearsBaselineError::MixedFamilies);
        }
        let hints = ActivityBaselineHints::new(
            offers
                .iter()
                .map(|offer| ActivityOptionHint::new(offer.id, offer.score))
                .collect(),
        )
        .map_err(GoldAndGearsBaselineError::Hints)?;
        let generic_offers = offers
            .iter()
            .map(|offer| (offer.id, offer.authored_priority))
            .collect::<Vec<_>>();
        let diagnostic = self
            .generic
            .decide_offers(decision, family.decision_kind(), &generic_offers, &hints)
            .map_err(GoldAndGearsBaselineError::Decision)?;
        let selected = offers
            .iter()
            .find(|offer| offer.id == diagnostic.option())
            .expect("the generic controller returns one exact offered identity")
            .clone();
        Ok(GoldAndGearsBaselineDecision {
            selected,
            diagnostic,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GoldAndGearsBaselineError {
    EmptyOffer,
    MixedFamilies,
    Hints(ActivityHintError),
    Decision(ActivityDecisionError),
}

/// Complete real-battle run plus every player-controller decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoldAndGearsBaselineRunReport {
    run: GoldAndGearsSeededRunReport,
    decisions: Box<[GoldAndGearsBaselineDecision]>,
    decision_digest: [u8; 32],
}

impl GoldAndGearsBaselineRunReport {
    #[must_use]
    pub const fn run(&self) -> &GoldAndGearsSeededRunReport {
        &self.run
    }

    #[must_use]
    pub fn decisions(&self) -> &[GoldAndGearsBaselineDecision] {
        &self.decisions
    }

    #[must_use]
    pub const fn decision_digest(&self) -> [u8; 32] {
        self.decision_digest
    }
}

impl GoldAndGearsRuntimeInstance {
    /// Completes a real seeded run while retaining every exact player offer
    /// selected by the deterministic baseline controller.
    pub fn execute_baseline_run(
        &self,
        request: GoldAndGearsSeededRunRequest,
        roster: &UniverseBattleRoster,
    ) -> Result<GoldAndGearsBaselineRunReport, GoldAndGearsSeededRunError> {
        let mut decisions = Vec::new();
        let execution =
            self.execute_seeded_run_recorded_with_decisions(request, roster, Some(&mut decisions))?;
        let decision_digest = decision_digest(execution.report.transcript_digest(), &decisions);
        Ok(GoldAndGearsBaselineRunReport {
            run: execution.report,
            decisions: decisions.into_boxed_slice(),
            decision_digest,
        })
    }
}

fn decision_digest(run_digest: [u8; 32], decisions: &[GoldAndGearsBaselineDecision]) -> [u8; 32] {
    let mut encoder = Encoder::new(b"starclock.gold-and-gears.baseline-controller.v1");
    encoder.text(GOLD_AND_GEARS_BASELINE_CONTROLLER_DOMAIN);
    encoder.digest(run_digest);
    encoder.u32(u32::try_from(decisions.len()).expect("seeded decision count is bounded"));
    for decision in decisions {
        encoder.u64(decision.diagnostic.decision().get());
        encoder.u8(decision.selected.family() as u8);
        encoder.u64(decision.selected.id().get());
        encoder.u32(
            u32::try_from(decision.diagnostic.scores().len()).expect("each offered set is bounded"),
        );
        for score in decision.diagnostic.scores() {
            encoder.u64(score.option().get());
            encoder.i64(score.total());
        }
    }
    encoder.finish()
}
