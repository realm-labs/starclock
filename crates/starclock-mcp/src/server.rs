//! MCP handler over one injected registry, factory and authority binding.

use rmcp::{
    ServerHandler, handler::server::router::tool::ToolRouter, model::ServerCapabilities,
    tool_handler,
};
use starclock_agent_api::session::{AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry};

use crate::metadata;

#[derive(Clone)]
pub struct StarclockMcp {
    pub(crate) registry: AgentSessionRegistry,
    pub(crate) factory: AgentSessionFactory,
    pub(crate) owner: AgentSessionOwner,
    pub(crate) tool_router: ToolRouter<Self>,
}

impl StarclockMcp {
    #[must_use]
    pub fn new(
        registry: AgentSessionRegistry,
        factory: AgentSessionFactory,
        owner: AgentSessionOwner,
    ) -> Self {
        Self {
            registry,
            factory,
            owner,
            tool_router: Self::registered_tool_router(),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for StarclockMcp {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        metadata::server_info(ServerCapabilities::builder().enable_tools().build())
    }
}
