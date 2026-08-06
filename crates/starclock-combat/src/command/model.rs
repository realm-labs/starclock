use core::{cmp::Ordering, fmt};

use crate::{
    battle::spec::TeamSide,
    id::{AbilityId, ActionBoundaryId, DecisionId, UnitId},
};

/// External intent accepted only when it exactly appears in the current decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    /// Enter the first battle decision boundary.
    StartBattle { decision: DecisionId },
    /// Use an offered normal ability and target commitment.
    UseAbility {
        decision: DecisionId,
        actor: UnitId,
        ability: AbilityId,
        primary_target: Option<UnitId>,
    },
    /// Request one ready Ultimate at the current stable action boundary.
    RequestUltimate {
        boundary: ActionBoundaryId,
        actor: UnitId,
        ability: AbilityId,
    },
    /// Commit one exact offered input for the currently prepared action.
    CommitPreparedAction {
        decision: DecisionId,
        primary_target: Option<UnitId>,
    },
    /// Cancel the currently prepared action before declaration or payment.
    CancelPreparedAction { decision: DecisionId },
    /// Advance deterministic battle work from the current stable action boundary.
    Advance { boundary: ActionBoundaryId },
    /// End the battle as a player loss when the profile offers concession.
    Concede { decision: DecisionId },
}

impl Command {
    /// Returns the exact decision identity answered by this command.
    #[must_use]
    pub const fn decision(&self) -> Option<DecisionId> {
        match self {
            Self::StartBattle { decision }
            | Self::UseAbility { decision, .. }
            | Self::CommitPreparedAction { decision, .. }
            | Self::CancelPreparedAction { decision }
            | Self::Concede { decision } => Some(*decision),
            Self::RequestUltimate { .. } | Self::Advance { .. } => None,
        }
    }

    /// Returns the stable action boundary addressed by this command, when applicable.
    #[must_use]
    pub const fn boundary(&self) -> Option<ActionBoundaryId> {
        match self {
            Self::RequestUltimate { boundary, .. } | Self::Advance { boundary } => Some(*boundary),
            Self::StartBattle { .. }
            | Self::UseAbility { .. }
            | Self::CommitPreparedAction { .. }
            | Self::CancelPreparedAction { .. }
            | Self::Concede { .. } => None,
        }
    }

    /// Compares two command values by the replay-canonical stable identity key.
    #[must_use]
    pub fn canonical_cmp(&self, other: &Self) -> Ordering {
        self.canonical_key().cmp(&other.canonical_key())
    }

    fn canonical_key(&self) -> (u8, u64, u64, u32, u64) {
        match self {
            Self::StartBattle { decision } => (0, decision.get(), 0, 0, 0),
            Self::UseAbility {
                decision,
                actor,
                ability,
                primary_target,
            } => (
                1,
                decision.get(),
                actor.get(),
                ability.get(),
                primary_target.map_or(0, UnitId::get),
            ),
            Self::RequestUltimate {
                boundary,
                actor,
                ability,
            } => (2, boundary.get(), actor.get(), ability.get(), 0),
            Self::CommitPreparedAction {
                decision,
                primary_target,
            } => (
                3,
                decision.get(),
                0,
                0,
                primary_target.map_or(0, UnitId::get),
            ),
            Self::CancelPreparedAction { decision } => (4, decision.get(), 0, 0, 0),
            Self::Advance { boundary } => (5, boundary.get(), 0, 0, 0),
            Self::Concede { decision } => (6, decision.get(), 0, 0, 0),
        }
    }
}

/// Stable category of controller input currently requested by a battle.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DecisionKind {
    /// Initial explicit battle-start command.
    BattleStart,
    /// One normal controllable unit action.
    NormalAction,
    /// Target or variant input for an action selected at an earlier boundary.
    PreparedAction,
    /// Typed action target, variant, segment, or battle-local authored choice.
    BattleChoice,
}

/// One exact ready-Ultimate request at an action boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UltimateOption {
    actor: UnitId,
    ability: AbilityId,
}

impl UltimateOption {
    pub(crate) const fn new(actor: UnitId, ability: AbilityId) -> Self {
        Self { actor, ability }
    }
    #[must_use]
    pub const fn actor(self) -> UnitId {
        self.actor
    }
    #[must_use]
    pub const fn ability(self) -> AbilityId {
        self.ability
    }
}

/// Controller that owns one decision point.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DecisionOwner {
    /// Lifecycle/system boundary rather than a team controller.
    System,
    /// Controller for the named formation side.
    Team(TeamSide),
}

/// Immutable offered command set in replay-canonical order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionPoint {
    id: DecisionId,
    kind: DecisionKind,
    owner: DecisionOwner,
    legal_commands: Box<[Command]>,
}

impl DecisionPoint {
    pub(crate) fn new(
        id: DecisionId,
        kind: DecisionKind,
        owner: DecisionOwner,
        mut legal_commands: Vec<Command>,
    ) -> Self {
        debug_assert!(
            legal_commands
                .iter()
                .all(|command| command.decision() == Some(id)),
            "decision points contain only commands addressed to that decision"
        );
        legal_commands.sort_by(Command::canonical_cmp);
        legal_commands.dedup();
        Self {
            id,
            kind,
            owner,
            legal_commands: legal_commands.into_boxed_slice(),
        }
    }

    /// Returns the battle-local monotonic decision identity.
    #[must_use]
    pub const fn id(&self) -> DecisionId {
        self.id
    }
    /// Returns the requested decision family.
    #[must_use]
    pub const fn kind(&self) -> DecisionKind {
        self.kind
    }
    /// Returns the controller owner.
    #[must_use]
    pub const fn owner(&self) -> DecisionOwner {
        self.owner
    }
    /// Returns legal command values in canonical replay order.
    #[must_use]
    pub fn legal_commands(&self) -> &[Command] {
        &self.legal_commands
    }

    pub(crate) fn contains(&self, command: &Command) -> bool {
        self.legal_commands
            .binary_search_by(|candidate| candidate.canonical_cmp(command))
            .is_ok()
    }
}

/// Stable rejected-command category. Every variant guarantees no mutation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CommandErrorKind {
    /// Terminal battles accept no command.
    TerminalBattle,
    /// `Resolving` never accepts reentrant external input.
    ResolutionInProgress,
    /// Command answers a prior or forged decision identity.
    StaleDecision,
    /// Command addresses a prior or forged action-boundary identity.
    StaleActionBoundary,
    /// Command value is not one of the exact offered values.
    NotOffered,
    /// Command family does not match the current lifecycle phase.
    WrongPhase,
}

/// Typed legality rejection with no platform-dependent diagnostic text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandError {
    kind: CommandErrorKind,
}

impl CommandError {
    pub(crate) const fn new(kind: CommandErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable rejection category.
    #[must_use]
    pub const fn kind(self) -> CommandErrorKind {
        self.kind
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "command rejected: {:?}", self.kind)
    }
}

impl std::error::Error for CommandError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn runtime<I: TryFrom<u64>>(raw: u64) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).expect("test ID is non-zero")
    }

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: core::fmt::Debug,
    {
        I::try_from(raw).expect("test ID is non-zero")
    }

    #[test]
    fn decision_commands_have_an_explicit_total_order() {
        let decision = runtime(3);
        let actor = runtime(2);
        let target = runtime(7);
        let mut expected = vec![
            Command::StartBattle { decision },
            Command::UseAbility {
                decision,
                actor,
                ability: definition(4),
                primary_target: None,
            },
            Command::UseAbility {
                decision,
                actor,
                ability: definition(4),
                primary_target: Some(target),
            },
            Command::Concede { decision },
        ];
        let mut reversed = expected.clone();
        reversed.reverse();
        reversed.push(expected[1].clone());
        let point = DecisionPoint::new(
            decision,
            DecisionKind::NormalAction,
            DecisionOwner::Team(TeamSide::Player),
            reversed,
        );
        assert_eq!(point.legal_commands(), expected);
        expected.reverse();
        assert_ne!(point.legal_commands(), expected);
    }
}
