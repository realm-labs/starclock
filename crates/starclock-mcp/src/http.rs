//! Bounded Streamable HTTP boundary for explicit loopback development.

use std::{
    collections::HashSet,
    fmt,
    future::{Future, IntoFuture},
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::http_observability::{
    DRAIN_TIMEOUT_SECONDS, HEALTH_PATH, HttpOperations, METRICS_PATH, READINESS_PATH,
};
use crate::{
    authorization::{
        AuthorizationFailure, AuthorizationPolicy, SUPPORTED_SCOPES, required_scope_for_json_rpc,
    },
    metadata::MCP_PROTOCOL_REVISION,
    rate_limit::{McpRateLimiter, RateClass, RateLimitClock, RateLimitExceeded},
    server::StarclockMcp,
};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{
        HeaderMap, HeaderValue, Method, Request, Response, StatusCode, Uri,
        header::{
            ALLOW, CONTENT_LENGTH, CONTENT_TYPE, HOST, RETRY_AFTER, TRANSFER_ENCODING,
            WWW_AUTHENTICATE,
        },
    },
    middleware::{self, Next},
    routing::get,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use starclock_agent_api::{
    activity_session::{ActivityAgentSessionFactory, registry::ActivityAgentSessionRegistry},
    error::{AgentError, AgentErrorCode},
    schema::SessionId,
    session::{
        AgentSessionFactory, AgentSessionOwner, AgentSessionRegistry, OperationalClock,
        SessionIdSource,
    },
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const MCP_HTTP_PATH: &str = "/mcp";
pub const PROTECTED_RESOURCE_METADATA_PATH: &str = "/.well-known/oauth-protected-resource/mcp";
pub const MAX_HTTP_REQUEST_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_HTTP_RESPONSE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_HTTP_WORKERS: usize = 32;
pub const MAX_ALLOWED_ORIGINS: usize = 16;
const MAX_ORIGIN_BYTES: usize = 256;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoopbackHttpConfig {
    bind: SocketAddr,
    allowed_origins: Vec<String>,
}

impl LoopbackHttpConfig {
    pub fn new(bind: SocketAddr, allowed_origins: Vec<String>) -> Result<Self, HttpServeError> {
        if !bind.ip().is_loopback() || bind.port() == 0 {
            return Err(HttpServeError::Configuration);
        }
        if allowed_origins.is_empty() || allowed_origins.len() > MAX_ALLOWED_ORIGINS {
            return Err(HttpServeError::Configuration);
        }
        let mut distinct = HashSet::with_capacity(allowed_origins.len());
        for origin in &allowed_origins {
            if !valid_exact_origin(origin) || !distinct.insert(origin.clone()) {
                return Err(HttpServeError::Configuration);
            }
        }
        Ok(Self {
            bind,
            allowed_origins,
        })
    }

    #[must_use]
    pub const fn bind(&self) -> SocketAddr {
        self.bind
    }

    #[must_use]
    pub fn allowed_origins(&self) -> &[String] {
        &self.allowed_origins
    }

    fn authority(&self) -> String {
        self.bind.to_string()
    }
}

#[derive(Debug)]
pub enum HttpServeError {
    Configuration,
    Runtime,
    Startup,
    Bind,
    Transport,
}

impl fmt::Display for HttpServeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration => formatter.write_str("the HTTP profile is not safe to start"),
            Self::Runtime => formatter.write_str("the MCP async runtime could not start"),
            Self::Startup => formatter.write_str("the MCP application could not initialize"),
            Self::Bind => formatter.write_str("the loopback MCP listener could not bind"),
            Self::Transport => formatter.write_str("the MCP HTTP transport stopped"),
        }
    }
}

impl std::error::Error for HttpServeError {}

pub fn serve_loopback(config: LoopbackHttpConfig) -> Result<(), HttpServeError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|_| HttpServeError::Runtime)?
        .block_on(serve_loopback_async(config))
}

async fn serve_loopback_async(config: LoopbackHttpConfig) -> Result<(), HttpServeError> {
    let app = build_loopback_app(&config, None)?;
    let listener = tokio::net::TcpListener::bind(config.bind())
        .await
        .map_err(|_| HttpServeError::Bind)?;
    run_loopback_server(
        listener,
        app,
        async {
            let _ = tokio::signal::ctrl_c().await;
        },
        Duration::from_secs(DRAIN_TIMEOUT_SECONDS),
    )
    .await
}

pub fn loopback_router(config: &LoopbackHttpConfig) -> Result<Router, HttpServeError> {
    build_loopback_router(config, None)
}

pub fn authorized_loopback_router(
    config: &LoopbackHttpConfig,
    authorization: AuthorizationPolicy,
) -> Result<Router, HttpServeError> {
    let resource = format!("http://{}{MCP_HTTP_PATH}", config.authority());
    let metadata = format!(
        "http://{}{}",
        config.authority(),
        PROTECTED_RESOURCE_METADATA_PATH
    );
    if authorization.expected_audience() != resource
        || authorization.resource_metadata_url() != metadata
    {
        return Err(HttpServeError::Configuration);
    }
    build_loopback_router(config, Some(authorization))
}

fn build_loopback_router(
    config: &LoopbackHttpConfig,
    authorization: Option<AuthorizationPolicy>,
) -> Result<Router, HttpServeError> {
    Ok(build_loopback_app(config, authorization)?.router)
}

struct LoopbackApp {
    router: Router,
    operations: HttpOperations,
}

fn build_loopback_app(
    config: &LoopbackHttpConfig,
    authorization: Option<AuthorizationPolicy>,
) -> Result<LoopbackApp, HttpServeError> {
    let factory = AgentSessionFactory::load_production().map_err(|_| HttpServeError::Startup)?;
    let activity_factory =
        ActivityAgentSessionFactory::load_production().map_err(|_| HttpServeError::Startup)?;
    let operational_clock = Arc::new(HttpClock::new());
    let session_ids = Arc::new(HttpBattleSessionIds::new());
    let registry = AgentSessionRegistry::new(
        factory.clone(),
        operational_clock.clone(),
        session_ids.clone(),
    );
    let activity_registry = ActivityAgentSessionRegistry::new(
        activity_factory.clone(),
        operational_clock.clone(),
        session_ids,
    );
    let rate_limiter = authorization
        .as_ref()
        .map(|_| McpRateLimiter::new(operational_clock));
    let request_authority = authorization.is_some();
    let owner_ids = Arc::new(AtomicU64::new(1));
    let service: StreamableHttpService<StarclockMcp, LocalSessionManager> =
        StreamableHttpService::new(
            move || {
                if request_authority {
                    return Ok(StarclockMcp::new_authorized(
                        registry.clone(),
                        factory.clone(),
                        activity_registry.clone(),
                        activity_factory.clone(),
                    ));
                }
                let ordinal = owner_ids.fetch_add(1, Ordering::Relaxed);
                let owner = AgentSessionOwner::new(
                    "loopback-development",
                    &format!("http-transport-{ordinal}"),
                )
                .map_err(|_| std::io::Error::other("MCP HTTP owner allocation failed"))?;
                Ok(StarclockMcp::new(
                    registry.clone(),
                    factory.clone(),
                    activity_registry.clone(),
                    activity_factory.clone(),
                    owner,
                ))
            },
            Arc::new(LocalSessionManager::default()),
            StreamableHttpServerConfig::default()
                .with_stateful_mode(true)
                .with_sse_keep_alive(None)
                .with_sse_retry(None)
                .with_allowed_hosts([config.authority()])
                .with_allowed_origins(config.allowed_origins().iter().cloned()),
        );
    let operations = HttpOperations::new();
    let guard = HttpGuard::new(
        config,
        authorization.clone(),
        rate_limiter,
        operations.clone(),
    );
    let mut router = Router::new().nest_service(MCP_HTTP_PATH, service);
    if let Some(policy) = authorization {
        router = router.route(
            PROTECTED_RESOURCE_METADATA_PATH,
            get(move || {
                let policy = policy.clone();
                async move { protected_resource_metadata(&policy) }
            }),
        );
    }
    let health = operations.clone();
    router = router.route(
        HEALTH_PATH,
        get(move || {
            let health = health.clone();
            async move { health.health_response() }
        }),
    );
    let readiness = operations.clone();
    router = router.route(
        READINESS_PATH,
        get(move || {
            let readiness = readiness.clone();
            async move { readiness.readiness_response() }
        }),
    );
    let metrics = operations.clone();
    router = router.route(
        METRICS_PATH,
        get(move || {
            let metrics = metrics.clone();
            async move { metrics.metrics_response() }
        }),
    );
    Ok(LoopbackApp {
        router: router.layer(middleware::from_fn_with_state(guard, guard_request)),
        operations,
    })
}

async fn run_loopback_server(
    listener: tokio::net::TcpListener,
    app: LoopbackApp,
    shutdown: impl Future<Output = ()> + Send + 'static,
    drain_timeout: Duration,
) -> Result<(), HttpServeError> {
    let operations = app.operations.clone();
    let shutdown_operations = operations.clone();
    let (draining_sender, draining_receiver) = tokio::sync::oneshot::channel();
    let server = axum::serve(listener, app.router)
        .with_graceful_shutdown(async move {
            shutdown.await;
            shutdown_operations.begin_draining();
            let _ = draining_sender.send(());
        })
        .into_future();
    tokio::pin!(server);
    let result = tokio::select! {
        result = &mut server => result.map_err(|_| HttpServeError::Transport),
        _ = draining_receiver => match tokio::time::timeout(drain_timeout, &mut server).await {
            Ok(result) => result.map_err(|_| HttpServeError::Transport),
            Err(_) => Ok(()),
        },
    };
    operations.stop();
    result
}

#[derive(Clone)]
struct HttpGuard {
    authority: Arc<str>,
    allowed_origins: Arc<[String]>,
    workers: Arc<Semaphore>,
    authorization: Option<AuthorizationPolicy>,
    rate_limiter: Option<McpRateLimiter>,
    operations: HttpOperations,
}

impl HttpGuard {
    fn new(
        config: &LoopbackHttpConfig,
        authorization: Option<AuthorizationPolicy>,
        rate_limiter: Option<McpRateLimiter>,
        operations: HttpOperations,
    ) -> Self {
        Self {
            authority: config.authority().into(),
            allowed_origins: config.allowed_origins.clone().into(),
            workers: Arc::new(Semaphore::new(MAX_HTTP_WORKERS)),
            authorization,
            rate_limiter,
            operations,
        }
    }

    fn acquire_worker(&self) -> Option<OwnedSemaphorePermit> {
        self.workers.clone().try_acquire_owned().ok()
    }
}

async fn guard_request(
    axum::extract::State(guard): axum::extract::State<HttpGuard>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    if let Some(error) = network_header_error(&guard, request.headers()) {
        return error;
    }
    if is_management_path(request.uri().path()) {
        return cap_response(next.run(request).await).await;
    }
    let Some(_request_guard) = guard.operations.start_request() else {
        return draining_response();
    };
    if request.method() == Method::GET {
        if let Some(policy) = &guard.authorization {
            let grant = match policy.authenticate(request.headers()) {
                Ok(grant) => grant,
                Err(failure) => return authorization_failure_response(policy, failure),
            };
            if let Some(failure) = rate_failure(&guard, &grant, RateClass::Read) {
                return failure;
            }
        }
        return method_not_allowed();
    }
    if request.method() != Method::POST && request.method() != Method::DELETE {
        return method_not_allowed();
    }
    if request
        .headers()
        .get("MCP-Protocol-Version")
        .and_then(|value| value.to_str().ok())
        != Some(MCP_PROTOCOL_REVISION)
    {
        return response(
            StatusCode::BAD_REQUEST,
            "MCP protocol version is not supported",
        );
    }
    let Some(_permit) = guard.acquire_worker() else {
        guard.operations.record_worker_rejection();
        return response_with_retry();
    };
    let (parts, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_HTTP_REQUEST_BYTES).await else {
        return response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "MCP request exceeds the fixed limit",
        );
    };
    let mut request = Request::from_parts(parts, Body::from(bytes.clone()));
    if let Some(policy) = &guard.authorization {
        let grant = match policy.authenticate(request.headers()) {
            Ok(grant) => grant,
            Err(failure) => return authorization_failure_response(policy, failure),
        };
        if let Err(failure) = policy.authorize_scope(&grant, required_scope_for_json_rpc(&bytes)) {
            return authorization_failure_response(policy, failure);
        }
        let class = rate_class_for_request(request.method(), &bytes);
        if let Some(failure) = rate_failure(&guard, &grant, class) {
            return failure;
        }
        request.extensions_mut().insert(grant);
    }
    cap_response(next.run(request).await).await
}

fn rate_class_for_request(method: &Method, body: &[u8]) -> RateClass {
    if method == Method::DELETE {
        return RateClass::Mutation;
    }
    let value = serde_json::from_slice::<serde_json::Value>(body).ok();
    match value
        .as_ref()
        .and_then(|value| value.get("method"))
        .and_then(serde_json::Value::as_str)
    {
        Some("tools/call") => match value
            .as_ref()
            .and_then(|value| value.get("params"))
            .and_then(|params| params.get("name"))
            .and_then(serde_json::Value::as_str)
        {
            Some("starclock_create_battle" | "starclock_create_universe") => RateClass::Create,
            Some(
                "starclock_play_action"
                | "starclock_close_battle"
                | "starclock_play_activity_action"
                | "starclock_close_activity",
            ) => RateClass::Mutation,
            _ => RateClass::Read,
        },
        _ => RateClass::Read,
    }
}

fn rate_failure(
    guard: &HttpGuard,
    grant: &crate::authorization::AuthorizationGrant,
    class: RateClass,
) -> Option<Response<Body>> {
    guard
        .rate_limiter
        .as_ref()
        .and_then(|limiter| limiter.admit(grant, class).err())
        .map(|failure| {
            guard.operations.record_rate_rejection();
            rate_limit_response(failure)
        })
}

fn is_management_path(path: &str) -> bool {
    matches!(
        path,
        HEALTH_PATH | READINESS_PATH | METRICS_PATH | PROTECTED_RESOURCE_METADATA_PATH
    )
}

fn network_header_error(guard: &HttpGuard, headers: &HeaderMap) -> Option<Response<Body>> {
    if has_forwarding_header(headers) {
        return Some(response(
            StatusCode::BAD_REQUEST,
            "Forwarded headers are not accepted",
        ));
    }
    if headers.get(HOST).and_then(|value| value.to_str().ok()) != Some(guard.authority.as_ref()) {
        return Some(response(StatusCode::FORBIDDEN, "Host is not allowed"));
    }
    if let Some(origin) = headers.get("origin") {
        let Ok(origin) = origin.to_str() else {
            return Some(response(StatusCode::FORBIDDEN, "Origin is not allowed"));
        };
        if !guard
            .allowed_origins
            .iter()
            .any(|allowed| allowed == origin)
        {
            return Some(response(StatusCode::FORBIDDEN, "Origin is not allowed"));
        }
    }
    None
}

fn authorization_failure_response(
    policy: &AuthorizationPolicy,
    failure: AuthorizationFailure,
) -> Response<Body> {
    let (status, challenge, message) = match failure {
        AuthorizationFailure::InvalidToken => (
            StatusCode::UNAUTHORIZED,
            format!(
                "Bearer resource_metadata=\"{}\"",
                policy.resource_metadata_url()
            ),
            "A valid access token is required",
        ),
        AuthorizationFailure::InsufficientScope(scope) => (
            StatusCode::FORBIDDEN,
            format!(
                "Bearer error=\"insufficient_scope\", scope=\"{scope}\", resource_metadata=\"{}\"",
                policy.resource_metadata_url()
            ),
            "The access token lacks the required scope",
        ),
    };
    let mut response = response(status, message);
    if let Ok(challenge) = HeaderValue::from_str(&challenge) {
        response.headers_mut().insert(WWW_AUTHENTICATE, challenge);
    }
    response
}

fn protected_resource_metadata(policy: &AuthorizationPolicy) -> Response<Body> {
    let body = serde_json::json!({
        "resource": policy.expected_audience(),
        "authorization_servers": [policy.expected_issuer()],
        "scopes_supported": SUPPORTED_SCOPES,
        "bearer_methods_supported": ["header"]
    })
    .to_string();
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .expect("bounded protected-resource metadata response is valid")
}

fn has_forwarding_header(headers: &HeaderMap) -> bool {
    [
        "forwarded",
        "x-forwarded-for",
        "x-forwarded-host",
        "x-forwarded-proto",
    ]
    .iter()
    .any(|name| headers.contains_key(*name))
}

async fn cap_response(source: Response<Body>) -> Response<Body> {
    if source.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return source;
    }
    let (mut parts, body) = source.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_HTTP_RESPONSE_BYTES).await else {
        return response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "MCP response exceeds the fixed limit",
        );
    };
    parts.headers.remove(CONTENT_LENGTH);
    parts.headers.remove(TRANSFER_ENCODING);
    Response::from_parts(parts, Body::from(bytes))
}

fn response(status: StatusCode, message: &'static str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(message))
        .expect("static bounded HTTP response is valid")
}

fn response_with_retry() -> Response<Body> {
    let mut response = response(StatusCode::SERVICE_UNAVAILABLE, "HTTP worker limit reached");
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_static("1"));
    response
}

fn draining_response() -> Response<Body> {
    let mut response = response(StatusCode::SERVICE_UNAVAILABLE, "HTTP service is draining");
    response
        .headers_mut()
        .insert(RETRY_AFTER, HeaderValue::from_static("1"));
    response
}

fn rate_limit_response(failure: RateLimitExceeded) -> Response<Body> {
    let mut response = response(StatusCode::TOO_MANY_REQUESTS, "Request rate limit reached");
    let retry_after = failure.retry_after_seconds().clamp(1, 60).to_string();
    if let Ok(value) = HeaderValue::from_str(&retry_after) {
        response.headers_mut().insert(RETRY_AFTER, value);
    }
    response
}

fn method_not_allowed() -> Response<Body> {
    let mut response = response(
        StatusCode::METHOD_NOT_ALLOWED,
        "This profile does not expose an SSE listening stream",
    );
    response
        .headers_mut()
        .insert(ALLOW, HeaderValue::from_static("POST, DELETE"));
    response
}

fn valid_exact_origin(origin: &str) -> bool {
    if origin.is_empty()
        || origin.len() > MAX_ORIGIN_BYTES
        || origin.contains('*')
        || origin == "null"
    {
        return false;
    }
    let Some((raw_scheme, raw_authority)) = origin.split_once("://") else {
        return false;
    };
    if raw_authority.is_empty()
        || raw_authority
            .chars()
            .any(|character| matches!(character, '/' | '?' | '#' | '@'))
    {
        return false;
    }
    let Ok(uri) = origin.parse::<Uri>() else {
        return false;
    };
    let Some(scheme) = uri.scheme_str() else {
        return false;
    };
    if scheme != "http" && scheme != "https" {
        return false;
    }
    let Some(authority) = uri.authority() else {
        return false;
    };
    raw_scheme == scheme
        && raw_authority == authority.as_str()
        && origin == format!("{scheme}://{authority}")
}

struct HttpClock {
    started: Instant,
}

impl HttpClock {
    fn new() -> Self {
        Self {
            started: Instant::now(),
        }
    }
}

impl OperationalClock for HttpClock {
    fn now_seconds(&self) -> u64 {
        self.started.elapsed().as_secs()
    }
}

impl RateLimitClock for HttpClock {
    fn now_seconds(&self) -> u64 {
        self.started.elapsed().as_secs()
    }
}

struct HttpBattleSessionIds {
    process: u32,
    started_nanos: u128,
    next: AtomicU64,
}

impl HttpBattleSessionIds {
    fn new() -> Self {
        Self {
            process: std::process::id(),
            started_nanos: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            next: AtomicU64::new(1),
        }
    }
}

impl SessionIdSource for HttpBattleSessionIds {
    fn next_session_id(&self) -> Result<SessionId, AgentError> {
        let ordinal = self.next.fetch_add(1, Ordering::Relaxed);
        SessionId::parse(&format!(
            "session_http_{:x}_{:x}_{ordinal:x}",
            self.process, self.started_nanos
        ))
        .map_err(|_| {
            AgentError::new(
                AgentErrorCode::AdapterFailure,
                "The HTTP MCP session identity could not be allocated.",
                false,
                false,
            )
            .expect("static HTTP identity error is bounded")
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::header::ACCEPT;
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;
    use crate::authorization::{
        AccessTokenSignatureVerifier, AuthorizationClock, SCOPE_SCENARIO_READ, SUPPORTED_SCOPES,
        SignatureVerificationError, SignedTokenClaims,
    };

    const AUTHORITY: &str = "127.0.0.1:43123";
    const ORIGIN: &str = "http://127.0.0.1:43123";

    #[derive(Clone)]
    struct FixedVerifier {
        claims: SignedTokenClaims,
    }

    impl AccessTokenSignatureVerifier for FixedVerifier {
        fn verify_signature_and_decode(
            &self,
            bearer_token: &str,
        ) -> Result<SignedTokenClaims, SignatureVerificationError> {
            (bearer_token == "valid-token")
                .then(|| self.claims.clone())
                .ok_or(SignatureVerificationError::Invalid)
        }
    }

    struct FixedClock;

    impl AuthorizationClock for FixedClock {
        fn now_seconds(&self) -> u64 {
            1_000
        }
    }

    impl RateLimitClock for FixedClock {
        fn now_seconds(&self) -> u64 {
            1_000
        }
    }

    #[derive(Clone, Copy)]
    struct AuthorityVerifier;

    impl AccessTokenSignatureVerifier for AuthorityVerifier {
        fn verify_signature_and_decode(
            &self,
            bearer_token: &str,
        ) -> Result<SignedTokenClaims, SignatureVerificationError> {
            let (tenant, principal) = bearer_token
                .split_once(':')
                .ok_or(SignatureVerificationError::Invalid)?;
            SignedTokenClaims::new(
                "https://auth.example".into(),
                vec![format!("http://{AUTHORITY}{MCP_HTTP_PATH}")],
                2_000,
                Some(900),
                tenant.into(),
                principal.into(),
                SUPPORTED_SCOPES.iter().map(ToString::to_string).collect(),
            )
            .map_err(|_| SignatureVerificationError::Invalid)
        }
    }

    fn config() -> LoopbackHttpConfig {
        LoopbackHttpConfig::new(AUTHORITY.parse().unwrap(), vec![ORIGIN.into()]).unwrap()
    }

    fn request(method: Method, body: Body) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(MCP_HTTP_PATH)
            .header(HOST, AUTHORITY)
            .header("origin", ORIGIN)
            .header(ACCEPT, "application/json, text/event-stream")
            .header(CONTENT_TYPE, "application/json")
            .header("MCP-Protocol-Version", "2025-11-25")
            .body(body)
            .unwrap()
    }

    fn management_request(path: &'static str) -> Request<Body> {
        Request::builder()
            .method(Method::GET)
            .uri(path)
            .header(HOST, AUTHORITY)
            .header("origin", ORIGIN)
            .body(Body::empty())
            .unwrap()
    }

    fn initialize_body() -> Body {
        Body::from(
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {"name": "http-boundary", "version": "1"}
                }
            })
            .to_string(),
        )
    }

    fn policy(scopes: Vec<String>) -> AuthorizationPolicy {
        let claims = SignedTokenClaims::new(
            "https://auth.example".into(),
            vec![format!("http://{AUTHORITY}{MCP_HTTP_PATH}")],
            2_000,
            Some(900),
            "tenant-test".into(),
            "principal-test".into(),
            scopes,
        )
        .unwrap();
        AuthorizationPolicy::new(
            "https://auth.example".into(),
            format!("http://{AUTHORITY}{MCP_HTTP_PATH}"),
            format!("http://{AUTHORITY}{PROTECTED_RESOURCE_METADATA_PATH}"),
            Arc::new(FixedVerifier { claims }),
            Arc::new(FixedClock),
        )
        .unwrap()
    }

    fn authority_policy() -> AuthorizationPolicy {
        AuthorizationPolicy::new(
            "https://auth.example".into(),
            format!("http://{AUTHORITY}{MCP_HTTP_PATH}"),
            format!("http://{AUTHORITY}{PROTECTED_RESOURCE_METADATA_PATH}"),
            Arc::new(AuthorityVerifier),
            Arc::new(FixedClock),
        )
        .unwrap()
    }

    fn with_bearer(mut request: Request<Body>, token: &'static str) -> Request<Body> {
        request.headers_mut().insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
        );
        request
    }

    fn session_request(
        body: serde_json::Value,
        session: &HeaderValue,
        token: &'static str,
    ) -> Request<Body> {
        let mut request = with_bearer(request(Method::POST, Body::from(body.to_string())), token);
        request
            .headers_mut()
            .insert("mcp-session-id", session.clone());
        request
    }

    async fn response_json(response: Response<Body>) -> serde_json::Value {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), MAX_HTTP_RESPONSE_BYTES)
            .await
            .unwrap();
        if let Ok(value) = serde_json::from_slice(&bytes) {
            return value;
        }
        let text = String::from_utf8_lossy(&bytes);
        let data = text
            .lines()
            .filter_map(|line| line.strip_prefix("data: "))
            .find(|line| line.starts_with('{'))
            .unwrap_or_else(|| panic!("expected JSON response, got {status} and {text:?}"));
        serde_json::from_str(data)
            .unwrap_or_else(|error| panic!("invalid SSE JSON from {status}: {error}"))
    }

    #[test]
    fn startup_accepts_only_exact_explicit_loopback_profiles() {
        assert!(
            LoopbackHttpConfig::new("0.0.0.0:43123".parse().unwrap(), vec![ORIGIN.into()]).is_err()
        );
        assert!(
            LoopbackHttpConfig::new("127.0.0.1:0".parse().unwrap(), vec![ORIGIN.into()]).is_err()
        );
        assert!(LoopbackHttpConfig::new(AUTHORITY.parse().unwrap(), vec![]).is_err());
        for origin in ["*", "null", "file://local", "http://127.0.0.1:43123/path"] {
            assert!(
                LoopbackHttpConfig::new(AUTHORITY.parse().unwrap(), vec![origin.into()]).is_err()
            );
        }
        assert_eq!(config().bind().to_string(), AUTHORITY);
    }

    include!("http_observability_test.rs");

    #[test]
    fn rate_classes_and_bounded_http_retry_are_frozen() {
        let create = json!({
            "method":"tools/call",
            "params":{"name":"starclock_create_battle"}
        });
        let act = json!({
            "method":"tools/call",
            "params":{"name":"starclock_play_action"}
        });
        let observe = json!({
            "method":"tools/call",
            "params":{"name":"starclock_observe_battle"}
        });
        assert_eq!(
            rate_class_for_request(&Method::POST, create.to_string().as_bytes()),
            RateClass::Create
        );
        assert_eq!(
            rate_class_for_request(&Method::POST, act.to_string().as_bytes()),
            RateClass::Mutation
        );
        assert_eq!(
            rate_class_for_request(&Method::DELETE, &[]),
            RateClass::Mutation
        );
        assert_eq!(
            rate_class_for_request(&Method::POST, observe.to_string().as_bytes()),
            RateClass::Read
        );

        let policy = authority_policy();
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer tenant-r:principal-r"),
        );
        let grant = policy.authenticate(&headers).unwrap();
        let limiter = McpRateLimiter::new(Arc::new(FixedClock));
        let guard = HttpGuard::new(
            &config(),
            Some(policy),
            Some(limiter.clone()),
            HttpOperations::new(),
        );
        for _ in 0..crate::rate_limit::READ_REQUESTS_PER_TENANT_PER_MINUTE {
            limiter.admit(&grant, RateClass::Read).unwrap();
        }
        let denied = rate_failure(&guard, &grant, RateClass::Read).unwrap();
        assert_eq!(denied.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(denied.headers()[RETRY_AFTER], "20");
    }

    #[tokio::test]
    async fn stateful_transport_enforces_host_origin_protocol_and_session_headers() {
        let app = loopback_router(&config()).unwrap();
        let mut wrong_host = request(Method::POST, initialize_body());
        wrong_host
            .headers_mut()
            .insert(HOST, HeaderValue::from_static("evil.test"));
        assert_eq!(app.clone().oneshot(wrong_host).await.unwrap().status(), 403);

        let mut wrong_origin = request(Method::POST, initialize_body());
        wrong_origin
            .headers_mut()
            .insert("origin", HeaderValue::from_static("http://evil.test"));
        assert_eq!(
            app.clone().oneshot(wrong_origin).await.unwrap().status(),
            403
        );

        let mut forwarded = request(Method::POST, initialize_body());
        forwarded
            .headers_mut()
            .insert("forwarded", HeaderValue::from_static("host=evil.test"));
        assert_eq!(app.clone().oneshot(forwarded).await.unwrap().status(), 400);

        let initialize = app
            .clone()
            .oneshot(request(Method::POST, initialize_body()))
            .await
            .unwrap();
        assert_eq!(initialize.status(), StatusCode::OK);
        let session = initialize.headers()["mcp-session-id"].clone();
        assert!(
            session
                .as_bytes()
                .iter()
                .all(|byte| (0x21..=0x7e).contains(byte))
        );
        let bytes = to_bytes(initialize.into_body(), MAX_HTTP_RESPONSE_BYTES)
            .await
            .unwrap();
        assert!(
            String::from_utf8(bytes.to_vec())
                .unwrap()
                .contains("2025-11-25")
        );

        let list = json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}).to_string();
        let missing_session = request(Method::POST, Body::from(list.clone()));
        assert_eq!(
            app.clone().oneshot(missing_session).await.unwrap().status(),
            StatusCode::UNPROCESSABLE_ENTITY
        );

        let mut wrong_protocol = request(Method::POST, Body::from(list.clone()));
        wrong_protocol
            .headers_mut()
            .insert("mcp-session-id", session.clone());
        wrong_protocol.headers_mut().insert(
            "MCP-Protocol-Version",
            HeaderValue::from_static("2025-03-26"),
        );
        assert_eq!(
            app.clone().oneshot(wrong_protocol).await.unwrap().status(),
            400
        );

        let mut valid = request(Method::POST, Body::from(list));
        valid.headers_mut().insert("mcp-session-id", session);
        let listed = app.oneshot(valid).await.unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let bytes = to_bytes(listed.into_body(), MAX_HTTP_RESPONSE_BYTES)
            .await
            .unwrap();
        assert!(
            String::from_utf8(bytes.to_vec())
                .unwrap()
                .contains("starclock_play_action")
        );
    }

    #[tokio::test]
    async fn get_body_response_and_workers_are_strictly_bounded() {
        let app = loopback_router(&config()).unwrap();
        let get = app
            .clone()
            .oneshot(request(Method::GET, Body::empty()))
            .await
            .unwrap();
        assert_eq!(get.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(get.headers()[ALLOW], "POST, DELETE");

        let oversized = request(
            Method::POST,
            Body::from(vec![b'x'; MAX_HTTP_REQUEST_BYTES + 1]),
        );
        assert_eq!(
            app.oneshot(oversized).await.unwrap().status(),
            StatusCode::PAYLOAD_TOO_LARGE
        );

        let oversized_response = Response::new(Body::from(vec![b'x'; MAX_HTTP_RESPONSE_BYTES + 1]));
        assert_eq!(
            cap_response(oversized_response).await.status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );

        let guard = HttpGuard::new(&config(), None, None, HttpOperations::new());
        let permits = (0..MAX_HTTP_WORKERS)
            .map(|_| guard.acquire_worker().unwrap())
            .collect::<Vec<_>>();
        assert!(guard.acquire_worker().is_none());
        assert_eq!(
            response_with_retry().status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(permits.len(), MAX_HTTP_WORKERS);
    }

    #[tokio::test]
    async fn authorized_profile_serves_metadata_and_denies_before_mcp_session_work() {
        let app =
            authorized_loopback_router(&config(), policy(vec![SCOPE_SCENARIO_READ.to_owned()]))
                .unwrap();
        let metadata_request = Request::builder()
            .method(Method::GET)
            .uri(PROTECTED_RESOURCE_METADATA_PATH)
            .header(HOST, AUTHORITY)
            .header("origin", ORIGIN)
            .body(Body::empty())
            .unwrap();
        let metadata = app.clone().oneshot(metadata_request).await.unwrap();
        assert_eq!(metadata.status(), StatusCode::OK);
        let metadata = to_bytes(metadata.into_body(), 16 * 1024).await.unwrap();
        let metadata: serde_json::Value = serde_json::from_slice(&metadata).unwrap();
        assert_eq!(metadata["resource"], format!("http://{AUTHORITY}/mcp"));
        assert_eq!(metadata["authorization_servers"][0], "https://auth.example");
        assert_eq!(metadata["scopes_supported"].as_array().unwrap().len(), 13);

        let missing = app
            .clone()
            .oneshot(request(Method::POST, initialize_body()))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        assert!(
            missing.headers()[WWW_AUTHENTICATE]
                .to_str()
                .unwrap()
                .contains("resource_metadata=")
        );

        let invalid = app
            .clone()
            .oneshot(with_bearer(
                request(Method::POST, initialize_body()),
                "raw-secret-token",
            ))
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
        let invalid_body = to_bytes(invalid.into_body(), 1024).await.unwrap();
        assert!(
            !String::from_utf8(invalid_body.to_vec())
                .unwrap()
                .contains("raw-secret-token")
        );

        let initialized = app
            .clone()
            .oneshot(with_bearer(
                request(Method::POST, initialize_body()),
                "valid-token",
            ))
            .await
            .unwrap();
        assert_eq!(initialized.status(), StatusCode::OK);
        let session = initialized.headers()["mcp-session-id"].clone();

        let list = json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}});
        let mut session_without_bearer = request(Method::POST, Body::from(list.to_string()));
        session_without_bearer
            .headers_mut()
            .insert("mcp-session-id", session.clone());
        assert_eq!(
            app.clone()
                .oneshot(session_without_bearer)
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );

        let create = json!({
            "jsonrpc":"2.0", "id":2, "method":"tools/call",
            "params":{"name":"starclock_create_battle","arguments":{
                "schema_revision":"agent-api-v1",
                "scenario_id":"scenario.standard-v1.basic-single-wave"
            }}
        });
        let mut create_request = with_bearer(
            request(Method::POST, Body::from(create.to_string())),
            "valid-token",
        );
        create_request
            .headers_mut()
            .insert("mcp-session-id", session);
        let denied = app.oneshot(create_request).await.unwrap();
        assert_eq!(denied.status(), StatusCode::FORBIDDEN);
        let challenge = denied.headers()[WWW_AUTHENTICATE].to_str().unwrap();
        assert!(challenge.contains("insufficient_scope"));
        assert!(challenge.contains("starclock:battle:create"));
    }

    include!("http_authority_test.rs");
    include!("http_quota_test.rs");
}
