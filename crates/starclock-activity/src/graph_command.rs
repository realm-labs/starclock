use crate::{
    ActivityBattleHandoffId, ActivityDecisionId, ActivityExternalOutcomeId, ActivityOptionId,
    ActivityStateHash, BattleResult,
};

pub const GRAPH_ACTIVITY_API_REVISION: &str = "starclock-activity-api-v2";

/// Closed command vocabulary for graph-capable Activities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphActivityCommandKind {
    ChooseOption { option: ActivityOptionId },
    StartBattle { handoff: ActivityBattleHandoffId },
    SubmitBattleResult { result: Box<BattleResult> },
    SubmitExternalOutcome { outcome: ActivityExternalOutcomeId },
    Abandon,
}

/// Version-2 optimistic command envelope. The aggregate validates all three
/// identity axes before beginning a mutation transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GraphActivityCommand {
    expected_state_hash: ActivityStateHash,
    decision: ActivityDecisionId,
    kind: GraphActivityCommandKind,
}

impl GraphActivityCommand {
    #[must_use]
    pub const fn new(
        expected_state_hash: ActivityStateHash,
        decision: ActivityDecisionId,
        kind: GraphActivityCommandKind,
    ) -> Self {
        Self {
            expected_state_hash,
            decision,
            kind,
        }
    }

    #[must_use]
    pub const fn expected_state_hash(&self) -> ActivityStateHash {
        self.expected_state_hash
    }

    #[must_use]
    pub const fn decision(&self) -> ActivityDecisionId {
        self.decision
    }

    #[must_use]
    pub const fn kind(&self) -> &GraphActivityCommandKind {
        &self.kind
    }
}
