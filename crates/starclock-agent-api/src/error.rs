//! Stable errors for validation, ownership, concurrency and domain boundaries.

use core::fmt;
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::schema::{AgentHash, AgentUInt, AgentValueError, SessionId};

/// Human-readable responsibility marker used by architecture tests.
pub const RESPONSIBILITY: &str = "stable protocol-neutral failures";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentErrorCode {
    InvalidSchema,
    UnknownRevision,
    InvalidRequest,
    UnknownSession,
    ExpiredSession,
    SessionNotOwned,
    SessionClosed,
    StaleDecision,
    StaleStateHash,
    InvalidActionToken,
    IdempotencyConflict,
    UnauthorizedPolicy,
    ConfigurationRejected,
    CombatRejected,
    BattleFaulted,
    EventCursorExpired,
    RequestTooLarge,
    ObservationTooLarge,
    SettlementBudgetExceeded,
    SessionQuotaExceeded,
    RateLimited,
    ReplayDiverged,
    AdapterFailure,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AgentError {
    pub code: AgentErrorCode,
    pub message: Box<str>,
    pub retryable: bool,
    pub committed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_id: Option<AgentUInt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_hash: Option<AgentHash>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub details: BTreeMap<Box<str>, Box<str>>,
}

impl AgentError {
    pub fn new(
        code: AgentErrorCode,
        message: impl Into<Box<str>>,
        retryable: bool,
        committed: bool,
    ) -> Result<Self, AgentValueError> {
        let message = message.into();
        if message.is_empty() {
            return Err(AgentValueError::Empty);
        }
        if message.len() > 1024 {
            return Err(AgentValueError::TooLong);
        }
        Ok(Self {
            code,
            message,
            retryable,
            committed,
            session_id: None,
            decision_id: None,
            state_hash: None,
            details: BTreeMap::new(),
        })
    }
}

impl fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "agent request failed: {:?}", self.code)
    }
}

impl std::error::Error for AgentError {}
