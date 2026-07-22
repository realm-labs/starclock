use std::sync::{Arc, atomic::AtomicUsize};

use rmcp::ServiceExt;
use starclock_mcp_sdk_capability::CapabilityServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    CapabilityServer::new(Arc::new(AtomicUsize::new(0)))
        .serve(rmcp::transport::stdio())
        .await?
        .waiting()
        .await?;
    Ok(())
}
