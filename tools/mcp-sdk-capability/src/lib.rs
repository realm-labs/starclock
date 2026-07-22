#![forbid(unsafe_code)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use rmcp::{
    ErrorData as McpError, Json, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolRequestParams, CancelledNotificationParam, Implementation,
        ListResourceTemplatesResult, ListResourcesResult, PaginatedRequestParams,
        ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents,
        ResourceTemplate, ServerCapabilities, ServerInfo,
    },
    service::{NotificationContext, RequestContext},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, JsonSchema)]
pub struct EchoInput {
    pub value: String,
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct EchoOutput {
    pub echoed: String,
}

#[derive(Clone, Debug)]
pub struct CapabilityServer {
    tool_router: ToolRouter<Self>,
    cancelled: Arc<AtomicUsize>,
}

#[tool_router(router = tool_router)]
impl CapabilityServer {
    pub fn new(cancelled: Arc<AtomicUsize>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            cancelled,
        }
    }

    #[tool(name = "echo", description = "Return one schema-bound value")]
    async fn echo(
        &self,
        Parameters(input): Parameters<EchoInput>,
    ) -> Result<Json<EchoOutput>, McpError> {
        Ok(Json(EchoOutput {
            echoed: input.value,
        }))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for CapabilityServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("starclock-capability", "0.0.0"))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                Resource::new("starclock://fixture/static", "static")
                    .with_mime_type("application/json"),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![
                ResourceTemplate::new("starclock://fixture/{name}", "fixture-by-name")
                    .with_mime_type("application/json"),
            ],
            next_cursor: None,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        if request.uri != "starclock://fixture/static" {
            return Err(McpError::resource_not_found(
                "fixture resource not found",
                None,
            ));
        }
        Ok(ReadResourceResult::new(vec![
            ResourceContents::text(r#"{"fixture":true}"#, request.uri)
                .with_mime_type("application/json"),
        ]))
    }

    async fn on_cancelled(
        &self,
        _params: CancelledNotificationParam,
        _context: NotificationContext<RoleServer>,
    ) {
        self.cancelled.fetch_add(1, Ordering::SeqCst);
    }
}

pub fn stdio_transport_typechecks() {
    let _transport = rmcp::transport::stdio();
}

pub fn unknown_tool_request() -> CallToolRequestParams {
    CallToolRequestParams::new("missing-tool")
}
