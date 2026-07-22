//! Offered action summaries and decision-scoped opaque token binding.
//!
//! Exact combat commands remain private implementation values and may only be
//! submitted through the authoritative session mutation boundary.

use core::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starclock_combat::{Command, DecisionId, DecisionPoint};

use crate::schema::{ActionToken, AgentUInt, SessionId};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "opaque offered actions and exact command bindings";
pub const MAX_OFFERED_ACTIONS: usize = 256;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActionKind {
    UseAbility,
    UseInterrupt,
    PassInterrupt,
    Concede,
    BattleChoice,
}

/// Player-safe description of one retained exact command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OfferedAction {
    pub token: ActionToken,
    pub kind: AgentActionKind,
    pub label: Box<str>,
    pub actor_unit_id: Option<AgentUInt>,
    pub primary_target_unit_id: Option<AgentUInt>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionBindingError {
    TooManyActions,
    EmptyActions,
    MixedDecision,
    NonCanonicalCommands,
    UnsupportedCommand,
    StaleDecision,
    InvalidActionToken,
    InvalidTokenEncoding,
}

impl fmt::Display for ActionBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "offered action binding failed: {self:?}")
    }
}

impl std::error::Error for ActionBindingError {}

/// Opaque result accepted only by the session module in this crate.
pub struct SelectedAction {
    command: Command,
}

impl SelectedAction {
    #[must_use]
    pub fn decision_id(&self) -> AgentUInt {
        AgentUInt::from_u64(self.command.decision().get())
    }

    pub(crate) fn into_command(self) -> Command {
        self.command
    }
}

impl fmt::Debug for SelectedAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SelectedAction([private command])")
    }
}

/// Decision-scoped public actions plus their private exact command table.
pub struct OfferedActionSet {
    decision: DecisionId,
    public: Box<[OfferedAction]>,
    private: Box<[(ActionToken, Command)]>,
}

impl OfferedActionSet {
    /// Binds the canonical commands already offered by an immutable decision.
    pub fn bind(
        session_id: &SessionId,
        decision: &DecisionPoint,
    ) -> Result<Self, ActionBindingError> {
        Self::bind_commands(session_id, decision.id(), decision.legal_commands())
    }

    fn bind_commands(
        session_id: &SessionId,
        decision: DecisionId,
        commands: &[Command],
    ) -> Result<Self, ActionBindingError> {
        if commands.is_empty() {
            return Err(ActionBindingError::EmptyActions);
        }
        if commands.len() > MAX_OFFERED_ACTIONS {
            return Err(ActionBindingError::TooManyActions);
        }
        if commands
            .iter()
            .any(|command| command.decision() != decision)
        {
            return Err(ActionBindingError::MixedDecision);
        }
        if commands
            .windows(2)
            .any(|pair| pair[0].canonical_cmp(&pair[1]).is_ge())
        {
            return Err(ActionBindingError::NonCanonicalCommands);
        }

        let mut public = Vec::with_capacity(commands.len());
        let mut private = Vec::with_capacity(commands.len());
        for (ordinal, command) in commands.iter().enumerate() {
            let token = action_token(session_id, decision, ordinal)?;
            public.push(summarize(token.clone(), command)?);
            private.push((token, command.clone()));
        }
        Ok(Self {
            decision,
            public: public.into_boxed_slice(),
            private: private.into_boxed_slice(),
        })
    }

    #[must_use]
    pub fn decision_id(&self) -> AgentUInt {
        AgentUInt::from_u64(self.decision.get())
    }

    #[must_use]
    pub fn actions(&self) -> &[OfferedAction] {
        &self.public
    }

    /// Selects only a token from this exact decision; no command is constructed.
    pub fn select(
        &self,
        expected_decision: &AgentUInt,
        token: &ActionToken,
    ) -> Result<SelectedAction, ActionBindingError> {
        if expected_decision.as_str() != self.decision.get().to_string() {
            return Err(ActionBindingError::StaleDecision);
        }
        self.private
            .iter()
            .find(|(candidate, _)| candidate == token)
            .map(|(_, command)| SelectedAction {
                command: command.clone(),
            })
            .ok_or(ActionBindingError::InvalidActionToken)
    }
}

fn action_token(
    session_id: &SessionId,
    decision: DecisionId,
    ordinal: usize,
) -> Result<ActionToken, ActionBindingError> {
    let ordinal = u32::try_from(ordinal).map_err(|_| ActionBindingError::TooManyActions)?;
    let mut hash = Sha256::new();
    hash.update(b"starclock-agent-action-v1\0");
    hash.update((session_id.as_str().len() as u64).to_be_bytes());
    hash.update(session_id.as_str().as_bytes());
    hash.update(decision.get().to_be_bytes());
    hash.update(ordinal.to_be_bytes());
    let digest = hash.finalize();
    let mut encoded = String::with_capacity(66);
    encoded.push_str("a_");
    for byte in digest {
        use core::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }
    ActionToken::parse(&encoded).map_err(|_| ActionBindingError::InvalidTokenEncoding)
}

fn summarize(token: ActionToken, command: &Command) -> Result<OfferedAction, ActionBindingError> {
    let (kind, label, actor, target) = match command {
        Command::UseAbility {
            actor,
            ability,
            primary_target,
            ..
        } => (
            AgentActionKind::UseAbility,
            format!("Use ability {} with unit {}.", ability.get(), actor.get()),
            Some(*actor),
            *primary_target,
        ),
        Command::UseInterrupt {
            actor,
            ability,
            primary_target,
            ..
        } => (
            AgentActionKind::UseInterrupt,
            format!(
                "Use interrupt ability {} with unit {}.",
                ability.get(),
                actor.get()
            ),
            Some(*actor),
            *primary_target,
        ),
        Command::PassInterruptWindow { .. } => (
            AgentActionKind::PassInterrupt,
            "Pass the current interrupt window.".to_owned(),
            None,
            None,
        ),
        Command::Concede { .. } => (
            AgentActionKind::Concede,
            "Concede the battle.".to_owned(),
            None,
            None,
        ),
        Command::StartBattle { .. } => return Err(ActionBindingError::UnsupportedCommand),
    };
    Ok(OfferedAction {
        token,
        kind,
        label: label.into_boxed_str(),
        actor_unit_id: actor.map(|value| AgentUInt::from_u64(value.get())),
        primary_target_unit_id: target.map(|value| AgentUInt::from_u64(value.get())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use starclock_combat::{AbilityId, UnitId};

    fn runtime<I: TryFrom<u64>>(raw: u64) -> I
    where
        I::Error: fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn definition<I: TryFrom<u32>>(raw: u32) -> I
    where
        I::Error: fmt::Debug,
    {
        I::try_from(raw).unwrap()
    }

    fn commands(decision: DecisionId) -> Vec<Command> {
        vec![
            Command::UseAbility {
                decision,
                actor: runtime::<UnitId>(1),
                ability: definition::<AbilityId>(2),
                primary_target: None,
            },
            Command::UseAbility {
                decision,
                actor: runtime::<UnitId>(1),
                ability: definition::<AbilityId>(2),
                primary_target: Some(runtime::<UnitId>(3)),
            },
            Command::Concede { decision },
        ]
    }

    #[test]
    fn canonical_commands_get_stable_summaries_and_distinct_tokens() {
        let session = SessionId::parse("session_a").unwrap();
        let decision = runtime::<DecisionId>(7);
        let set = OfferedActionSet::bind_commands(&session, decision, &commands(decision)).unwrap();
        assert_eq!(set.decision_id().as_str(), "7");
        assert_eq!(set.actions().len(), 3);
        assert_eq!(set.actions()[0].kind, AgentActionKind::UseAbility);
        assert_eq!(
            set.actions()[0].actor_unit_id.as_ref().unwrap().as_str(),
            "1"
        );
        assert_eq!(
            set.actions()[1]
                .primary_target_unit_id
                .as_ref()
                .unwrap()
                .as_str(),
            "3"
        );
        assert_ne!(set.actions()[0].token, set.actions()[1].token);
        assert_eq!(
            serde_json::to_value(set.actions()).unwrap()[2]["kind"],
            "concede"
        );
    }

    #[test]
    fn forged_stale_and_cross_session_tokens_never_select_a_command() {
        let decision = runtime::<DecisionId>(7);
        let first = OfferedActionSet::bind_commands(
            &SessionId::parse("session_a").unwrap(),
            decision,
            &commands(decision),
        )
        .unwrap();
        let second = OfferedActionSet::bind_commands(
            &SessionId::parse("session_b").unwrap(),
            decision,
            &commands(decision),
        )
        .unwrap();
        let token = first.actions()[0].token.clone();
        assert_eq!(
            first.select(&AgentUInt::from_u64(8), &token).unwrap_err(),
            ActionBindingError::StaleDecision
        );
        assert_eq!(
            second.select(&AgentUInt::from_u64(7), &token).unwrap_err(),
            ActionBindingError::InvalidActionToken
        );
        assert_eq!(
            first
                .select(
                    &AgentUInt::from_u64(7),
                    &ActionToken::parse("a_forged").unwrap(),
                )
                .unwrap_err(),
            ActionBindingError::InvalidActionToken
        );
        let selected = first.select(&AgentUInt::from_u64(7), &token).unwrap();
        assert_eq!(selected.decision_id().as_str(), "7");
        assert_eq!(format!("{selected:?}"), "SelectedAction([private command])");
        assert_eq!(selected.into_command(), commands(decision)[0]);
    }

    #[test]
    fn mixed_unsorted_and_internal_start_commands_fail_closed() {
        let session = SessionId::parse("session_a").unwrap();
        let decision = runtime::<DecisionId>(7);
        let mut unsorted = commands(decision);
        unsorted.reverse();
        assert!(matches!(
            OfferedActionSet::bind_commands(&session, decision, &unsorted),
            Err(ActionBindingError::NonCanonicalCommands)
        ));
        assert!(matches!(
            OfferedActionSet::bind_commands(
                &session,
                decision,
                &[Command::UseAbility {
                    decision: runtime(8),
                    actor: runtime(1),
                    ability: definition(1),
                    primary_target: None,
                }],
            ),
            Err(ActionBindingError::MixedDecision)
        ));
        assert!(matches!(
            OfferedActionSet::bind_commands(
                &session,
                decision,
                &[Command::StartBattle { decision }],
            ),
            Err(ActionBindingError::UnsupportedCommand)
        ));
    }
}
