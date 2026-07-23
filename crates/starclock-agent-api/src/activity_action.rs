//! Opaque offered actions for graph-Activity boundaries.

use core::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use starclock_activity::{
    ActivityDecisionId, ActivityDecisionKind, ActivityOptionId, ActivityPlayerView,
    ActivityPreparationOptionKind, ActivityStateHash,
};

use crate::schema::{ActionToken, AgentSInt, AgentUInt, SessionId};

pub const RESPONSIBILITY: &str = "opaque Activity actions and exact option bindings";
pub const MAX_OFFERED_ACTIVITY_ACTIONS: usize = 256;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentActivityActionKind {
    SelectOption,
    EngageEncounter,
    SubmitExternalOutcome,
    UseTechnique,
    EngageBattle,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OfferedActivityAction {
    pub token: ActionToken,
    pub kind: AgentActivityActionKind,
    pub label: Box<str>,
    pub option_id: AgentUInt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<AgentSInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<AgentUInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technique_point_cost: Option<AgentUInt>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActivityActionBindingError {
    NoOffer,
    TooManyActions,
    DuplicateOption,
    InvalidTokenEncoding,
    StaleBoundary,
    InvalidActionToken,
}

impl fmt::Display for ActivityActionBindingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Activity action binding failed: {self:?}")
    }
}

impl std::error::Error for ActivityActionBindingError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoundActivityAction {
    Decision {
        decision: ActivityDecisionId,
        kind: ActivityDecisionKind,
        option: ActivityOptionId,
    },
    Preparation {
        option: ActivityOptionId,
    },
}

pub(crate) struct SelectedActivityAction {
    action: BoundActivityAction,
}

impl SelectedActivityAction {
    pub(crate) const fn into_action(self) -> BoundActivityAction {
        self.action
    }
}

impl fmt::Debug for SelectedActivityAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SelectedActivityAction([private option binding])")
    }
}

pub(crate) struct OfferedActivityActionSet {
    boundary: u64,
    state_hash: ActivityStateHash,
    public: Box<[OfferedActivityAction]>,
    private: Box<[(ActionToken, BoundActivityAction)]>,
}

impl OfferedActivityActionSet {
    pub(crate) fn bind(
        session: &SessionId,
        view: &ActivityPlayerView,
    ) -> Result<Self, ActivityActionBindingError> {
        let boundary = view.command_sequence();
        let mut definitions = Vec::new();
        if let Some(decision) = view.decision() {
            definitions.extend(
                decision
                    .options()
                    .iter()
                    .copied()
                    .map(|option| {
                        let kind = match decision.kind() {
                            ActivityDecisionKind::Encounter => {
                                AgentActivityActionKind::EngageEncounter
                            }
                            ActivityDecisionKind::ExternalOutcome => {
                                AgentActivityActionKind::SubmitExternalOutcome
                            }
                            _ => AgentActivityActionKind::SelectOption,
                        };
                        Ok((
                            BoundActivityAction::Decision {
                                decision: decision.id(),
                                kind: decision.kind(),
                                option: option.id(),
                            },
                            OfferedActivityAction {
                                token: placeholder_token()?,
                                kind,
                                label: decision_label(decision.kind(), option.id()).into(),
                                option_id: AgentUInt::from_u64(option.id().get()),
                                priority: Some(AgentSInt::from_i64(i64::from(option.priority()))),
                                participant_id: None,
                                technique_point_cost: None,
                            },
                        ))
                    })
                    .collect::<Result<Vec<_>, ActivityActionBindingError>>()?,
            );
        } else if let Some(preparation) = view.preparation() {
            definitions.extend(
                preparation
                    .options()
                    .iter()
                    .copied()
                    .map(|option| {
                        let (kind, label) = match option.kind() {
                            ActivityPreparationOptionKind::NormalEngagement => (
                                AgentActivityActionKind::EngageBattle,
                                "Engage the prepared encounter.".to_owned(),
                            ),
                            ActivityPreparationOptionKind::Technique(engagement) => (
                                AgentActivityActionKind::UseTechnique,
                                format!(
                                    "Use {engagement:?} technique option {}.",
                                    option.id().get()
                                ),
                            ),
                        };
                        Ok((
                            BoundActivityAction::Preparation {
                                option: option.id(),
                            },
                            OfferedActivityAction {
                                token: placeholder_token()?,
                                kind,
                                label: label.into_boxed_str(),
                                option_id: AgentUInt::from_u64(option.id().get()),
                                priority: None,
                                participant_id: option
                                    .participant()
                                    .map(|value| AgentUInt::from_u64(u64::from(value.get()))),
                                technique_point_cost: Some(AgentUInt::from_u64(u64::from(
                                    option.point_cost(),
                                ))),
                            },
                        ))
                    })
                    .collect::<Result<Vec<_>, ActivityActionBindingError>>()?,
            );
        }
        if definitions.is_empty() {
            return Err(ActivityActionBindingError::NoOffer);
        }
        if definitions.len() > MAX_OFFERED_ACTIVITY_ACTIONS {
            return Err(ActivityActionBindingError::TooManyActions);
        }
        definitions.sort_by_key(|(binding, _)| binding.option().get());
        if definitions
            .windows(2)
            .any(|pair| pair[0].0.option() == pair[1].0.option())
        {
            return Err(ActivityActionBindingError::DuplicateOption);
        }
        let mut public = Vec::with_capacity(definitions.len());
        let mut private = Vec::with_capacity(definitions.len());
        for (ordinal, (binding, mut summary)) in definitions.into_iter().enumerate() {
            let token = activity_action_token(
                session,
                view.state_hash(),
                boundary,
                binding.option(),
                ordinal,
            )?;
            summary.token = token.clone();
            public.push(summary);
            private.push((token, binding));
        }
        Ok(Self {
            boundary,
            state_hash: view.state_hash(),
            public: public.into_boxed_slice(),
            private: private.into_boxed_slice(),
        })
    }

    pub(crate) const fn boundary(&self) -> u64 {
        self.boundary
    }

    pub(crate) const fn state_hash(&self) -> ActivityStateHash {
        self.state_hash
    }

    pub(crate) fn actions(&self) -> &[OfferedActivityAction] {
        &self.public
    }

    pub(crate) fn select(
        &self,
        expected_boundary: &AgentUInt,
        token: &ActionToken,
    ) -> Result<SelectedActivityAction, ActivityActionBindingError> {
        if expected_boundary.to_u64() != self.boundary {
            return Err(ActivityActionBindingError::StaleBoundary);
        }
        self.private
            .iter()
            .find(|(candidate, _)| candidate == token)
            .map(|(_, action)| SelectedActivityAction { action: *action })
            .ok_or(ActivityActionBindingError::InvalidActionToken)
    }
}

impl BoundActivityAction {
    pub(crate) const fn option(self) -> ActivityOptionId {
        match self {
            Self::Decision { option, .. } | Self::Preparation { option } => option,
        }
    }
}

fn activity_action_token(
    session: &SessionId,
    state_hash: ActivityStateHash,
    boundary: u64,
    option: ActivityOptionId,
    ordinal: usize,
) -> Result<ActionToken, ActivityActionBindingError> {
    let ordinal = u32::try_from(ordinal).map_err(|_| ActivityActionBindingError::TooManyActions)?;
    let mut hash = Sha256::new();
    hash.update(b"starclock-agent-activity-action-v1\0");
    hash.update((session.as_str().len() as u64).to_be_bytes());
    hash.update(session.as_str().as_bytes());
    hash.update(state_hash.bytes());
    hash.update(boundary.to_be_bytes());
    hash.update(option.get().to_be_bytes());
    hash.update(ordinal.to_be_bytes());
    let digest = hash.finalize();
    let mut encoded = String::with_capacity(66);
    encoded.push_str("u_");
    for byte in digest {
        use core::fmt::Write as _;
        write!(&mut encoded, "{byte:02x}").expect("writing to a string cannot fail");
    }
    ActionToken::parse(&encoded).map_err(|_| ActivityActionBindingError::InvalidTokenEncoding)
}

fn placeholder_token() -> Result<ActionToken, ActivityActionBindingError> {
    ActionToken::parse("u_pending").map_err(|_| ActivityActionBindingError::InvalidTokenEncoding)
}

fn decision_label(kind: ActivityDecisionKind, option: ActivityOptionId) -> String {
    format!("Select {kind:?} option {}.", option.get())
}
