# Starclock MCP HTTP operations

The explicit loopback HTTP service exposes three process-local management
endpoints alongside `/mcp`: `/healthz`, `/readyz` and `/metrics`. They use the
same exact `Host`, optional exact `Origin` and no-forwarded-header boundary as
MCP traffic. They need no bearer credential because the listener cannot bind
outside loopback, and they grant no MCP, session or battle authority.

`/healthz` reports process liveness with HTTP 200 while the listener is able to
answer. `/readyz` returns 200 only while new MCP work is admitted, then 503 as
soon as draining begins. `/metrics` returns one fixed JSON aggregate with the
schema `starclock.mcp-http-metrics.v1`. Its fields are readiness/drain state,
in-flight and started/completed request totals, plus drain, worker and rate
rejection totals. The payload explicitly says `authoritative:false`; it has no
tenant, principal, transport-session, battle, tool, token or user-text labels.

Ctrl-C starts graceful shutdown before the listener stops. New MCP requests
are rejected with HTTP 503 and `Retry-After: 1`. Requests admitted before the
transition retain their guard and may finish normally. The Axum server receives
at most ten seconds to drain; after that bound its future is dropped and the
operational lifecycle is stopped. Saturating counters and RAII guards keep the
bookkeeping finite and correct across every early response path.

Cancellation remains an advisory delivery event. It cannot interrupt or roll
back an accepted authoritative mutation. If response delivery is ambiguous, a
same-authority retry with the same idempotency key returns the exact committed
result; a different tenant still fails before cache lookup.

Lifecycle state, counters, signals and management requests are adapter-only.
They never enter an `AgentSession`, domain command, event, replay record, RNG
input or canonical encoder. The HTTP conformance test probes all three
management endpoints between battle creation and observation and proves the
state hash is unchanged.

Non-loopback startup remains unavailable. This operational surface does not
implement the trusted-proxy and security-audit requirements of a remote
profile.
