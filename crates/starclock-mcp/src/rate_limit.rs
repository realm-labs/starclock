//! Bounded operational rate limits keyed only by validated request authority.

use std::{
    collections::BTreeMap,
    fmt,
    sync::{Arc, Mutex},
};

use crate::authorization::AuthorizationGrant;

pub const CREATE_REQUESTS_PER_PRINCIPAL_PER_MINUTE: u32 = 30;
pub const MUTATION_REQUESTS_PER_TENANT_PER_MINUTE: u32 = 600;
pub const READ_REQUESTS_PER_TENANT_PER_MINUTE: u32 = 1_200;
pub const MAX_RATE_TENANTS: usize = 4_096;
pub const MAX_RATE_PRINCIPALS: usize = 4_096;
const WINDOW_SECONDS: u64 = 60;

/// Injected monotonic operational time; it never enters combat state or output.
pub trait RateLimitClock: Send + Sync {
    fn now_seconds(&self) -> u64;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RateClass {
    Read,
    Create,
    Mutation,
}

#[derive(Clone)]
pub struct McpRateLimiter {
    inner: Arc<RateLimiterInner>,
}

struct RateLimiterInner {
    clock: Arc<dyn RateLimitClock>,
    state: Mutex<RateState>,
}

#[derive(Default)]
struct RateState {
    last_now: u64,
    tenants: BTreeMap<String, TenantWindows>,
    principals: BTreeMap<(String, String), Window>,
}

#[derive(Clone, Copy, Default)]
struct TenantWindows {
    read: Window,
    mutation: Window,
}

#[derive(Clone, Copy, Default)]
struct Window {
    index: u64,
    count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimitExceeded {
    retry_after_seconds: u64,
}

impl RateLimitExceeded {
    #[must_use]
    pub const fn retry_after_seconds(self) -> u64 {
        self.retry_after_seconds
    }
}

impl fmt::Display for RateLimitExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the request rate limit was reached")
    }
}

impl std::error::Error for RateLimitExceeded {}

impl McpRateLimiter {
    #[must_use]
    pub fn new(clock: Arc<dyn RateLimitClock>) -> Self {
        Self {
            inner: Arc::new(RateLimiterInner {
                clock,
                state: Mutex::new(RateState::default()),
            }),
        }
    }

    pub(crate) fn admit(
        &self,
        grant: &AuthorizationGrant,
        class: RateClass,
    ) -> Result<(), RateLimitExceeded> {
        let now = self.inner.clock.now_seconds();
        let mut state = self.inner.state.lock().map_err(|_| exceeded(60))?;
        if now < state.last_now {
            return Err(exceeded(60));
        }
        state.last_now = now;
        let index = now / WINDOW_SECONDS;
        let retry_after = WINDOW_SECONDS - now % WINDOW_SECONDS;
        match class {
            RateClass::Create => {
                let key = (
                    grant.tenant_id().to_owned(),
                    grant.principal_id().to_owned(),
                );
                ensure_principal_capacity(&mut state, &key, index, retry_after)?;
                admit_window(
                    state.principals.entry(key).or_default(),
                    index,
                    CREATE_REQUESTS_PER_PRINCIPAL_PER_MINUTE,
                    retry_after,
                )
            }
            RateClass::Read | RateClass::Mutation => {
                ensure_tenant_capacity(&mut state, grant.tenant_id(), index, retry_after)?;
                let windows = state
                    .tenants
                    .entry(grant.tenant_id().to_owned())
                    .or_default();
                let (window, limit) = match class {
                    RateClass::Read => (&mut windows.read, READ_REQUESTS_PER_TENANT_PER_MINUTE),
                    RateClass::Mutation => (
                        &mut windows.mutation,
                        MUTATION_REQUESTS_PER_TENANT_PER_MINUTE,
                    ),
                    RateClass::Create => unreachable!(),
                };
                admit_window(window, index, limit, retry_after)
            }
        }
    }
}

fn ensure_principal_capacity(
    state: &mut RateState,
    key: &(String, String),
    index: u64,
    retry_after: u64,
) -> Result<(), RateLimitExceeded> {
    if state.principals.contains_key(key) || state.principals.len() < MAX_RATE_PRINCIPALS {
        return Ok(());
    }
    state.principals.retain(|_, window| window.index == index);
    if state.principals.len() == MAX_RATE_PRINCIPALS {
        return Err(exceeded(retry_after));
    }
    Ok(())
}

fn ensure_tenant_capacity(
    state: &mut RateState,
    tenant: &str,
    index: u64,
    retry_after: u64,
) -> Result<(), RateLimitExceeded> {
    if state.tenants.contains_key(tenant) || state.tenants.len() < MAX_RATE_TENANTS {
        return Ok(());
    }
    state
        .tenants
        .retain(|_, windows| windows.read.index == index || windows.mutation.index == index);
    if state.tenants.len() == MAX_RATE_TENANTS {
        return Err(exceeded(retry_after));
    }
    Ok(())
}

fn admit_window(
    window: &mut Window,
    index: u64,
    limit: u32,
    retry_after: u64,
) -> Result<(), RateLimitExceeded> {
    if window.index != index {
        *window = Window { index, count: 0 };
    }
    if window.count >= limit {
        return Err(exceeded(retry_after));
    }
    window.count += 1;
    Ok(())
}

const fn exceeded(retry_after_seconds: u64) -> RateLimitExceeded {
    RateLimitExceeded {
        retry_after_seconds,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::authorization::{
        AccessTokenSignatureVerifier, AuthorizationClock, AuthorizationPolicy, SignedTokenClaims,
    };

    struct TestClock(AtomicU64);

    impl TestClock {
        fn set(&self, value: u64) {
            self.0.store(value, Ordering::Relaxed);
        }
    }

    impl RateLimitClock for TestClock {
        fn now_seconds(&self) -> u64 {
            self.0.load(Ordering::Relaxed)
        }
    }

    impl AuthorizationClock for TestClock {
        fn now_seconds(&self) -> u64 {
            self.0.load(Ordering::Relaxed)
        }
    }

    struct Verifier;

    impl AccessTokenSignatureVerifier for Verifier {
        fn verify_signature_and_decode(
            &self,
            bearer: &str,
        ) -> Result<SignedTokenClaims, crate::authorization::SignatureVerificationError> {
            let (tenant, principal) = bearer
                .split_once(':')
                .ok_or(crate::authorization::SignatureVerificationError::Invalid)?;
            SignedTokenClaims::new(
                "https://issuer.example".into(),
                vec!["http://127.0.0.1:39127/mcp".into()],
                10_000,
                None,
                tenant.into(),
                principal.into(),
                crate::authorization::SUPPORTED_SCOPES
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            )
            .map_err(|_| crate::authorization::SignatureVerificationError::Invalid)
        }
    }

    fn make_grant(clock: Arc<TestClock>, token: &'static str) -> AuthorizationGrant {
        let policy = AuthorizationPolicy::new(
            "https://issuer.example".into(),
            "http://127.0.0.1:39127/mcp".into(),
            "http://127.0.0.1:39127/.well-known/oauth-protected-resource/mcp".into(),
            Arc::new(Verifier),
            clock,
        )
        .unwrap();
        let mut headers = axum::http::HeaderMap::new();
        headers.insert("authorization", format!("Bearer {token}").parse().unwrap());
        policy.authenticate(&headers).unwrap()
    }

    #[test]
    fn exact_independent_limits_reset_only_on_monotonic_minute_boundary() {
        let clock = Arc::new(TestClock(AtomicU64::new(120)));
        let limiter = McpRateLimiter::new(clock.clone());
        let grant = make_grant(clock.clone(), "tenant-a:principal-a");

        for _ in 0..CREATE_REQUESTS_PER_PRINCIPAL_PER_MINUTE {
            limiter.admit(&grant, RateClass::Create).unwrap();
        }
        assert_eq!(
            limiter
                .admit(&grant, RateClass::Create)
                .unwrap_err()
                .retry_after_seconds(),
            60
        );
        let other_principal = make_grant(clock.clone(), "tenant-a:principal-b");
        assert!(limiter.admit(&other_principal, RateClass::Create).is_ok());
        for _ in 0..MUTATION_REQUESTS_PER_TENANT_PER_MINUTE {
            limiter.admit(&grant, RateClass::Mutation).unwrap();
        }
        assert!(limiter.admit(&grant, RateClass::Mutation).is_err());
        for _ in 0..READ_REQUESTS_PER_TENANT_PER_MINUTE {
            limiter.admit(&grant, RateClass::Read).unwrap();
        }
        assert!(limiter.admit(&grant, RateClass::Read).is_err());
        let other_tenant = make_grant(clock.clone(), "tenant-b:principal-a");
        assert!(limiter.admit(&other_tenant, RateClass::Read).is_ok());

        clock.set(180);
        assert!(limiter.admit(&grant, RateClass::Create).is_ok());
        assert!(limiter.admit(&grant, RateClass::Mutation).is_ok());
        assert!(limiter.admit(&grant, RateClass::Read).is_ok());
        clock.set(179);
        assert_eq!(
            limiter
                .admit(&grant, RateClass::Read)
                .unwrap_err()
                .retry_after_seconds(),
            60
        );
    }
}
