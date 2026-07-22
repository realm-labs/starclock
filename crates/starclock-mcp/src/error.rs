//! Structured tool failures preserve the frozen agent error schema.

use rmcp::{ErrorData as McpError, model::CallToolResult};
use serde::Serialize;
use starclock_agent_api::error::AgentError;

pub fn structured_agent_error(error: AgentError) -> CallToolResult {
    let value = serde_json::to_value(error)
        .expect("validated AgentError values always serialize to bounded JSON");
    CallToolResult::structured_error(value)
}

pub fn internal_protocol_error() -> McpError {
    McpError::internal_error("The Starclock MCP adapter failed.", None)
}

pub(crate) fn structured_result<T: Serialize>(
    result: Result<T, AgentError>,
) -> Result<CallToolResult, McpError> {
    match result {
        Ok(value) => serde_json::to_value(value)
            .map(CallToolResult::structured)
            .map_err(|_| internal_protocol_error()),
        Err(error) => Ok(structured_agent_error(error)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ErrorCode;
    use starclock_agent_api::error::AgentErrorCode;

    const CODES: [AgentErrorCode; 23] = [
        AgentErrorCode::InvalidSchema,
        AgentErrorCode::UnknownRevision,
        AgentErrorCode::InvalidRequest,
        AgentErrorCode::UnknownSession,
        AgentErrorCode::ExpiredSession,
        AgentErrorCode::SessionNotOwned,
        AgentErrorCode::SessionClosed,
        AgentErrorCode::StaleDecision,
        AgentErrorCode::StaleStateHash,
        AgentErrorCode::InvalidActionToken,
        AgentErrorCode::IdempotencyConflict,
        AgentErrorCode::UnauthorizedPolicy,
        AgentErrorCode::ConfigurationRejected,
        AgentErrorCode::CombatRejected,
        AgentErrorCode::BattleFaulted,
        AgentErrorCode::EventCursorExpired,
        AgentErrorCode::RequestTooLarge,
        AgentErrorCode::ObservationTooLarge,
        AgentErrorCode::SettlementBudgetExceeded,
        AgentErrorCode::SessionQuotaExceeded,
        AgentErrorCode::RateLimited,
        AgentErrorCode::ReplayDiverged,
        AgentErrorCode::AdapterFailure,
    ];

    #[test]
    fn every_agent_failure_is_a_structured_tool_error() {
        for code in CODES {
            let error = AgentError::new(code, "Stable failure.", false, false).unwrap();
            let expected = serde_json::to_value(&error).unwrap();
            let result = structured_agent_error(error);
            assert_eq!(result.is_error, Some(true));
            assert_eq!(result.structured_content, Some(expected.clone()));
            assert_eq!(result.content.len(), 1);
            let encoded = serde_json::to_value(&result.content[0]).unwrap();
            assert_eq!(encoded["text"], expected.to_string());
        }
    }

    #[test]
    fn internal_protocol_failure_is_generic_and_data_free() {
        let error = internal_protocol_error();
        assert_eq!(error.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(error.message, "The Starclock MCP adapter failed.");
        assert_eq!(error.data, None);
    }
}
