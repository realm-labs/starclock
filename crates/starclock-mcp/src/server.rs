//! MCP handler over one injected registry, factory and authority binding.

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{
        GetPromptRequestParams, GetPromptResult, ListPromptsResult, ListResourceTemplatesResult,
        ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
        ServerCapabilities,
    },
    service::RequestContext,
    tool_handler,
};
use starclock_agent_api::session::{AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry};

use crate::{metadata, resources};

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
        metadata::server_info(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        reject_cursor(request)?;
        Ok(resources::list_resources())
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        reject_cursor(request)?;
        Ok(resources::list_resource_templates())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(&self.factory, &request.uri)
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        reject_cursor(request)?;
        Ok(resources::list_prompts())
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        if request
            .arguments
            .as_ref()
            .is_some_and(|arguments| !arguments.is_empty())
        {
            return Err(McpError::invalid_params(
                "The Starclock usage prompt accepts no arguments.",
                None,
            ));
        }
        resources::get_prompt(&request.name)
    }
}

fn reject_cursor(request: Option<PaginatedRequestParams>) -> Result<(), McpError> {
    if request.and_then(|request| request.cursor).is_some() {
        return Err(McpError::invalid_params(
            "The bounded Starclock collection has no continuation cursor.",
            None,
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_collections_reject_unissued_continuation_cursors() {
        assert!(reject_cursor(None).is_ok());
        assert!(reject_cursor(Some(PaginatedRequestParams::default())).is_ok());
        assert!(
            reject_cursor(Some(
                PaginatedRequestParams::default().with_cursor(Some("not-issued".into()))
            ))
            .is_err()
        );
    }
}
