//! OAuth resource-server validation and exact MCP operation scopes.

use std::{
    collections::BTreeSet,
    fmt,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::http::{HeaderMap, Uri, header::AUTHORIZATION};
use starclock_agent_api::session::AgentSessionOwner;

pub const SCOPE_SCENARIO_READ: &str = "starclock:scenario:read";
pub const SCOPE_BATTLE_CREATE: &str = "starclock:battle:create";
pub const SCOPE_BATTLE_READ: &str = "starclock:battle:read";
pub const SCOPE_BATTLE_ACT: &str = "starclock:battle:act";
pub const SCOPE_BATTLE_REPLAY: &str = "starclock:battle:replay";
pub const SCOPE_BATTLE_CLOSE: &str = "starclock:battle:close";
pub const SCOPE_REPLAY_VERIFY: &str = "starclock:replay:verify";
pub const SCOPE_ACTIVITY_CREATE: &str = "starclock:activity:create";
pub const SCOPE_ACTIVITY_READ: &str = "starclock:activity:read";
pub const SCOPE_ACTIVITY_ACT: &str = "starclock:activity:act";
pub const SCOPE_ACTIVITY_REPLAY: &str = "starclock:activity:replay";
pub const SCOPE_ACTIVITY_CLOSE: &str = "starclock:activity:close";
pub const SCOPE_DEBUG_OMNISCIENT: &str = "starclock:debug:omniscient";
pub const SUPPORTED_SCOPES: [&str; 13] = [
    SCOPE_SCENARIO_READ,
    SCOPE_BATTLE_CREATE,
    SCOPE_BATTLE_READ,
    SCOPE_BATTLE_ACT,
    SCOPE_BATTLE_REPLAY,
    SCOPE_BATTLE_CLOSE,
    SCOPE_REPLAY_VERIFY,
    SCOPE_ACTIVITY_CREATE,
    SCOPE_ACTIVITY_READ,
    SCOPE_ACTIVITY_ACT,
    SCOPE_ACTIVITY_REPLAY,
    SCOPE_ACTIVITY_CLOSE,
    SCOPE_DEBUG_OMNISCIENT,
];

const MAX_BEARER_TOKEN_BYTES: usize = 8 * 1024;
const MAX_CLAIM_TEXT_BYTES: usize = 512;
const MAX_IDENTITY_BYTES: usize = 128;
const MAX_AUDIENCES: usize = 8;
const MAX_SCOPES: usize = 32;
const MAX_SCOPE_BYTES: usize = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SignatureVerificationError {
    Invalid,
    Unavailable,
}

pub trait AccessTokenSignatureVerifier: Send + Sync {
    /// Verify the token's signature using trusted key material, then decode its
    /// claims. The raw token must not be forwarded to another resource server.
    fn verify_signature_and_decode(
        &self,
        bearer_token: &str,
    ) -> Result<SignedTokenClaims, SignatureVerificationError>;
}

pub trait AuthorizationClock: Send + Sync {
    fn now_seconds(&self) -> u64;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemAuthorizationClock;

impl AuthorizationClock for SystemAuthorizationClock {
    fn now_seconds(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedTokenClaims {
    issuer: String,
    audiences: Vec<String>,
    expires_at: u64,
    not_before: Option<u64>,
    tenant_id: String,
    principal_id: String,
    scopes: BTreeSet<String>,
}

impl SignedTokenClaims {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        issuer: String,
        audiences: Vec<String>,
        expires_at: u64,
        not_before: Option<u64>,
        tenant_id: String,
        principal_id: String,
        scopes: Vec<String>,
    ) -> Result<Self, TokenClaimsError> {
        if !valid_absolute_uri(&issuer, true)
            || audiences.is_empty()
            || audiences.len() > MAX_AUDIENCES
            || audiences
                .iter()
                .any(|audience| !valid_absolute_uri(audience, false))
            || expires_at == 0
            || tenant_id.is_empty()
            || tenant_id.len() > MAX_IDENTITY_BYTES
            || principal_id.is_empty()
            || principal_id.len() > MAX_IDENTITY_BYTES
            || scopes.len() > MAX_SCOPES
            || scopes.iter().any(|scope| !valid_scope(scope))
        {
            return Err(TokenClaimsError);
        }
        AgentSessionOwner::new(&tenant_id, &principal_id).map_err(|_| TokenClaimsError)?;
        let scopes = scopes.into_iter().collect::<BTreeSet<_>>();
        if scopes.len() > MAX_SCOPES {
            return Err(TokenClaimsError);
        }
        Ok(Self {
            issuer,
            audiences,
            expires_at,
            not_before,
            tenant_id,
            principal_id,
            scopes,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TokenClaimsError;

impl fmt::Display for TokenClaimsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the verified access-token claims are invalid")
    }
}

impl std::error::Error for TokenClaimsError {}

#[derive(Clone)]
pub struct AuthorizationPolicy {
    expected_issuer: Arc<str>,
    expected_audience: Arc<str>,
    resource_metadata_url: Arc<str>,
    verifier: Arc<dyn AccessTokenSignatureVerifier>,
    clock: Arc<dyn AuthorizationClock>,
}

impl AuthorizationPolicy {
    pub fn new(
        expected_issuer: String,
        expected_audience: String,
        resource_metadata_url: String,
        verifier: Arc<dyn AccessTokenSignatureVerifier>,
        clock: Arc<dyn AuthorizationClock>,
    ) -> Result<Self, AuthorizationConfigurationError> {
        if !valid_absolute_uri(&expected_issuer, true)
            || !valid_absolute_uri(&expected_audience, false)
            || !valid_absolute_uri(&resource_metadata_url, false)
        {
            return Err(AuthorizationConfigurationError);
        }
        Ok(Self {
            expected_issuer: expected_issuer.into(),
            expected_audience: expected_audience.into(),
            resource_metadata_url: resource_metadata_url.into(),
            verifier,
            clock,
        })
    }

    pub fn authenticate(
        &self,
        headers: &HeaderMap,
    ) -> Result<AuthorizationGrant, AuthorizationFailure> {
        let token = bearer_token(headers)?;
        let claims = self
            .verifier
            .verify_signature_and_decode(token)
            .map_err(|_| AuthorizationFailure::InvalidToken)?;
        let now = self.clock.now_seconds();
        if claims.issuer != self.expected_issuer.as_ref()
            || !claims
                .audiences
                .iter()
                .any(|audience| audience == self.expected_audience.as_ref())
            || claims.expires_at <= now
            || claims.not_before.is_some_and(|not_before| not_before > now)
        {
            return Err(AuthorizationFailure::InvalidToken);
        }
        Ok(AuthorizationGrant {
            tenant_id: claims.tenant_id,
            principal_id: claims.principal_id,
            expires_at: claims.expires_at,
            scopes: claims.scopes,
        })
    }

    pub fn authorize_scope(
        &self,
        grant: &AuthorizationGrant,
        required_scope: Option<&'static str>,
    ) -> Result<(), AuthorizationFailure> {
        if let Some(scope) = required_scope
            && !grant.scopes.contains(scope)
        {
            return Err(AuthorizationFailure::InsufficientScope(scope));
        }
        Ok(())
    }

    #[must_use]
    pub fn expected_issuer(&self) -> &str {
        &self.expected_issuer
    }

    #[must_use]
    pub fn expected_audience(&self) -> &str {
        &self.expected_audience
    }

    #[must_use]
    pub fn resource_metadata_url(&self) -> &str {
        &self.resource_metadata_url
    }
}

impl fmt::Debug for AuthorizationPolicy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthorizationPolicy")
            .field("expected_issuer", &self.expected_issuer)
            .field("expected_audience", &self.expected_audience)
            .field("resource_metadata_url", &self.resource_metadata_url)
            .field("verifier", &"<redacted>")
            .finish_non_exhaustive()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationConfigurationError;

impl fmt::Display for AuthorizationConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the OAuth resource-server policy is invalid")
    }
}

impl std::error::Error for AuthorizationConfigurationError {}

#[derive(Clone, Eq, PartialEq)]
pub struct AuthorizationGrant {
    tenant_id: String,
    principal_id: String,
    expires_at: u64,
    scopes: BTreeSet<String>,
}

impl AuthorizationGrant {
    #[must_use]
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }

    #[must_use]
    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    #[must_use]
    pub const fn expires_at(&self) -> u64 {
        self.expires_at
    }

    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(scope)
    }
}

impl fmt::Debug for AuthorizationGrant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("AuthorizationGrant(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationFailure {
    InvalidToken,
    InsufficientScope(&'static str),
}

#[must_use]
pub fn required_scope_for_json_rpc(body: &[u8]) -> Option<&'static str> {
    let value = serde_json::from_slice::<serde_json::Value>(body).ok()?;
    match value.get("method")?.as_str()? {
        "resources/list" | "resources/templates/list" | "prompts/list" | "prompts/get" => {
            Some(SCOPE_SCENARIO_READ)
        }
        "resources/read" => {
            value
                .get("params")?
                .get("uri")?
                .as_str()
                .map_or(Some(SCOPE_SCENARIO_READ), |uri| {
                    if uri.starts_with("starclock://universe/")
                        || uri == "starclock://rules/standard-universe"
                    {
                        Some(SCOPE_ACTIVITY_READ)
                    } else {
                        Some(SCOPE_SCENARIO_READ)
                    }
                })
        }
        "tools/call" => match value.get("params")?.get("name")?.as_str()? {
            "starclock_list_scenarios" => Some(SCOPE_SCENARIO_READ),
            "starclock_create_battle" => Some(SCOPE_BATTLE_CREATE),
            "starclock_observe_battle" => Some(SCOPE_BATTLE_READ),
            "starclock_play_action" => Some(SCOPE_BATTLE_ACT),
            "starclock_export_replay" => Some(SCOPE_BATTLE_REPLAY),
            "starclock_close_battle" => Some(SCOPE_BATTLE_CLOSE),
            "starclock_verify_replay" => Some(SCOPE_REPLAY_VERIFY),
            "starclock_create_universe" => Some(SCOPE_ACTIVITY_CREATE),
            "starclock_observe_activity" => Some(SCOPE_ACTIVITY_READ),
            "starclock_play_activity_action" => Some(SCOPE_ACTIVITY_ACT),
            "starclock_export_activity_replay" => Some(SCOPE_ACTIVITY_REPLAY),
            "starclock_close_activity" => Some(SCOPE_ACTIVITY_CLOSE),
            "starclock_verify_activity_replay" => Some(SCOPE_REPLAY_VERIFY),
            _ => None,
        },
        _ => None,
    }
}

fn bearer_token(headers: &HeaderMap) -> Result<&str, AuthorizationFailure> {
    let mut values = headers.get_all(AUTHORIZATION).iter();
    let value = values.next().ok_or(AuthorizationFailure::InvalidToken)?;
    if values.next().is_some() {
        return Err(AuthorizationFailure::InvalidToken);
    }
    let value = value
        .to_str()
        .map_err(|_| AuthorizationFailure::InvalidToken)?;
    let (scheme, token) = value
        .split_once(' ')
        .ok_or(AuthorizationFailure::InvalidToken)?;
    if !scheme.eq_ignore_ascii_case("Bearer")
        || token.is_empty()
        || token.len() > MAX_BEARER_TOKEN_BYTES
        || token
            .bytes()
            .any(|byte| byte.is_ascii_whitespace() || byte.is_ascii_control())
    {
        return Err(AuthorizationFailure::InvalidToken);
    }
    Ok(token)
}

fn valid_scope(scope: &str) -> bool {
    !scope.is_empty()
        && scope.len() <= MAX_SCOPE_BYTES
        && scope
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b':' | b'-' | b'_' | b'.'))
}

fn valid_absolute_uri(value: &str, require_https: bool) -> bool {
    if value.is_empty() || value.len() > MAX_CLAIM_TEXT_BYTES || value.contains('#') {
        return false;
    }
    let Ok(uri) = value.parse::<Uri>() else {
        return false;
    };
    let Some(scheme) = uri.scheme_str() else {
        return false;
    };
    uri.authority().is_some() && (scheme == "https" || (!require_https && scheme == "http"))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::http::HeaderValue;

    use super::*;

    #[derive(Clone)]
    struct StaticVerifier(SignedTokenClaims);

    impl AccessTokenSignatureVerifier for StaticVerifier {
        fn verify_signature_and_decode(
            &self,
            bearer_token: &str,
        ) -> Result<SignedTokenClaims, SignatureVerificationError> {
            (bearer_token == "valid")
                .then(|| self.0.clone())
                .ok_or(SignatureVerificationError::Invalid)
        }
    }

    struct FixedClock;

    impl AuthorizationClock for FixedClock {
        fn now_seconds(&self) -> u64 {
            100
        }
    }

    fn claims(issuer: &str, audience: &str, expires_at: u64) -> SignedTokenClaims {
        SignedTokenClaims::new(
            issuer.into(),
            vec![audience.into()],
            expires_at,
            Some(90),
            "tenant-test".into(),
            "principal-test".into(),
            vec![SCOPE_SCENARIO_READ.into()],
        )
        .unwrap()
    }

    fn policy(claims: SignedTokenClaims) -> AuthorizationPolicy {
        AuthorizationPolicy::new(
            "https://issuer.example".into(),
            "https://mcp.example/mcp".into(),
            "https://mcp.example/.well-known/oauth-protected-resource/mcp".into(),
            Arc::new(StaticVerifier(claims)),
            Arc::new(FixedClock),
        )
        .unwrap()
    }

    fn headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer valid"));
        headers
    }

    #[test]
    fn exact_scope_matrix_covers_every_frozen_operation() {
        let cases = [
            ("starclock_list_scenarios", SCOPE_SCENARIO_READ),
            ("starclock_create_battle", SCOPE_BATTLE_CREATE),
            ("starclock_observe_battle", SCOPE_BATTLE_READ),
            ("starclock_play_action", SCOPE_BATTLE_ACT),
            ("starclock_export_replay", SCOPE_BATTLE_REPLAY),
            ("starclock_close_battle", SCOPE_BATTLE_CLOSE),
            ("starclock_verify_replay", SCOPE_REPLAY_VERIFY),
            ("starclock_create_universe", SCOPE_ACTIVITY_CREATE),
            ("starclock_observe_activity", SCOPE_ACTIVITY_READ),
            ("starclock_play_activity_action", SCOPE_ACTIVITY_ACT),
            ("starclock_export_activity_replay", SCOPE_ACTIVITY_REPLAY),
            ("starclock_close_activity", SCOPE_ACTIVITY_CLOSE),
            ("starclock_verify_activity_replay", SCOPE_REPLAY_VERIFY),
        ];
        for (tool, scope) in cases {
            let body = serde_json::json!({
                "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": {"name": tool, "arguments": {}}
            });
            assert_eq!(
                required_scope_for_json_rpc(body.to_string().as_bytes()),
                Some(scope)
            );
        }
        assert_eq!(SUPPORTED_SCOPES.len(), 13);
        assert!(SUPPORTED_SCOPES.contains(&SCOPE_DEBUG_OMNISCIENT));
        for method in [
            "resources/list",
            "resources/templates/list",
            "resources/read",
            "prompts/list",
            "prompts/get",
        ] {
            let body = serde_json::json!({"jsonrpc":"2.0","id":1,"method":method,"params":{"uri":"starclock://catalog/manifest"}});
            assert_eq!(
                required_scope_for_json_rpc(body.to_string().as_bytes()),
                Some(SCOPE_SCENARIO_READ)
            );
        }
        let universe = serde_json::json!({"jsonrpc":"2.0","id":1,"method":"resources/read","params":{"uri":"starclock://universe/manifest"}});
        assert_eq!(
            required_scope_for_json_rpc(universe.to_string().as_bytes()),
            Some(SCOPE_ACTIVITY_READ)
        );
    }

    #[test]
    fn bearer_parser_rejects_duplicates_whitespace_and_oversize_without_echo() {
        let mut headers = HeaderMap::new();
        assert_eq!(
            bearer_token(&headers),
            Err(AuthorizationFailure::InvalidToken)
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_static("Bearer stable-token"),
        );
        assert_eq!(bearer_token(&headers).unwrap(), "stable-token");
        headers.append(AUTHORIZATION, HeaderValue::from_static("Bearer second"));
        assert_eq!(
            bearer_token(&headers),
            Err(AuthorizationFailure::InvalidToken)
        );
        assert_eq!(
            format!("{:?}", AuthorizationFailure::InvalidToken),
            "InvalidToken"
        );
    }

    #[test]
    fn signature_issuer_audience_time_and_scope_are_checked_without_token_retention() {
        let valid = policy(claims(
            "https://issuer.example",
            "https://mcp.example/mcp",
            101,
        ));
        let grant = valid.authenticate(&headers()).unwrap();
        assert_eq!(grant.tenant_id(), "tenant-test");
        assert_eq!(grant.principal_id(), "principal-test");
        assert_eq!(grant.expires_at(), 101);
        assert!(grant.has_scope(SCOPE_SCENARIO_READ));
        assert_eq!(format!("{grant:?}"), "AuthorizationGrant(<redacted>)");
        assert_eq!(
            valid.authorize_scope(&grant, Some(SCOPE_BATTLE_ACT)),
            Err(AuthorizationFailure::InsufficientScope(SCOPE_BATTLE_ACT))
        );

        for invalid in [
            claims("https://other.example", "https://mcp.example/mcp", 101),
            claims("https://issuer.example", "https://other.example/mcp", 101),
            claims("https://issuer.example", "https://mcp.example/mcp", 100),
        ] {
            assert_eq!(
                policy(invalid).authenticate(&headers()),
                Err(AuthorizationFailure::InvalidToken)
            );
        }
        let mut bad_signature = headers();
        bad_signature.insert(AUTHORIZATION, HeaderValue::from_static("Bearer forged"));
        assert_eq!(
            valid.authenticate(&bad_signature),
            Err(AuthorizationFailure::InvalidToken)
        );
    }
}
