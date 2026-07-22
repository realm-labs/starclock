//! MCP handler shell; tools are added only through reviewed adapter batches.

use rmcp::{ServerHandler, model::ServerCapabilities};

use crate::metadata;

#[derive(Clone, Copy, Debug, Default)]
pub struct StarclockMcp;

impl StarclockMcp {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ServerHandler for StarclockMcp {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        metadata::server_info(ServerCapabilities::builder().build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ProtocolVersion;

    #[test]
    fn handler_reports_only_the_frozen_metadata() {
        let info = StarclockMcp::new().get_info();
        assert_eq!(info.protocol_version, ProtocolVersion::V_2025_11_25);
        assert_eq!(info.server_info.name, metadata::SERVER_NAME);
        assert!(info.capabilities.tools.is_none());
        assert!(info.capabilities.resources.is_none());
    }
}
