//! Authorized Streamable HTTP embedding boundary.
//!
//! The executable constructs a deny-all loopback router to prove configuration.
//! A deployment supplies its established signature verifier to `build_router`;
//! this example deliberately does not invent a bearer-token format.

use std::{net::SocketAddr, sync::Arc};

use axum::Router;
use starclock_mcp::{
    authorization::{AccessTokenSignatureVerifier, AuthorizationClock, AuthorizationPolicy},
    http::{LoopbackHttpConfig, MCP_HTTP_PATH, PROTECTED_RESOURCE_METADATA_PATH},
};

fn build_router(
    bind: SocketAddr,
    allowed_origins: Vec<String>,
    issuer: String,
    verifier: Arc<dyn AccessTokenSignatureVerifier>,
    clock: Arc<dyn AuthorizationClock>,
) -> Result<Router, Box<dyn std::error::Error>> {
    let config = LoopbackHttpConfig::new(bind, allowed_origins)?;
    let audience = format!("http://{bind}{MCP_HTTP_PATH}");
    let metadata = format!("http://{bind}{PROTECTED_RESOURCE_METADATA_PATH}");
    let policy = AuthorizationPolicy::new(issuer, audience, metadata, verifier, clock)?;
    Ok(starclock_mcp::http::authorized_loopback_router(
        &config, policy,
    )?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bind = "127.0.0.1:3001".parse()?;
    let _router = build_router(
        bind,
        vec!["https://agent.example".to_owned()],
        "https://authorization.example".to_owned(),
        Arc::new(DenyAllVerifier),
        Arc::new(starclock_mcp::authorization::SystemAuthorizationClock),
    )?;
    println!(
        "constructed deny-all authorized MCP router for {bind}; inject a deployment verifier before serving"
    );
    Ok(())
}

struct DenyAllVerifier;

impl AccessTokenSignatureVerifier for DenyAllVerifier {
    fn verify_signature_and_decode(
        &self,
        _bearer_token: &str,
    ) -> Result<
        starclock_mcp::authorization::SignedTokenClaims,
        starclock_mcp::authorization::SignatureVerificationError,
    > {
        Err(starclock_mcp::authorization::SignatureVerificationError::Invalid)
    }
}
