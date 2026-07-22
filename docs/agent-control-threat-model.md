# Agent control and MCP threat model

`G02-P0-B4` freezes the Goal 02 security boundary. The complete machine-readable
model is [`policy/agent-control-threat-model.json`](../policy/agent-control-threat-model.json).
Every threat names later batches that must prove its controls; documentation
alone does not close a verification owner.

## Trust boundary and invariants

Agents, MCP clients, HTTP peers, proxies, replay bytes and catalog inputs are
outside the authoritative battle boundary. The protocol-neutral API validates
them before a session mutation lane; only exact retained commands cross into
`Battle::apply` and only generic activity commands cross into `Activity::apply`.
Rejected or unauthorized input consumes no operational/domain identity, RNG or
state mutation.

Each session is bound to one validated tenant/principal and has one serialized
mutation lane. Expected decision/hash checks happen inside that lane. A
successful result is cached against a request-bound idempotency key before
delivery, so response loss and retry cannot double-commit. Operational clocks,
opaque IDs, transport sessions, auth claims and limits never enter deterministic
state, hashes, replay facts or RNG.

## Remote fail-closed startup

Stdio has no listener and reserves stdout exclusively for MCP. Loopback HTTP
binds only to loopback, validates Host and any present Origin against explicit
local allowlists, and distrusts forwarded headers by default.

Non-loopback HTTP cannot start unless all remote prerequisites are present:
TLS (or an explicitly attested TLS-terminating proxy), token signature/issuer/
audience/expiry validation, a nonempty exact Origin allowlist, tenant/principal
claim extraction, tool scopes, trusted-proxy configuration when applicable,
quotas, rate limits, payload limits and a security audit sink. Wildcard Origin,
anonymous remote mode, token passthrough and ambient forwarded-header trust are
forbidden. Missing one prerequisite is a startup error, not a warning.

## Authorization and abuse controls

Scopes separate scenario read, battle create/read/act/replay/close, replay
verification and omniscient debug. Denials occur before session existence or
ownership can leak. The default projection is player-visible; debug needs its
distinct scope and output marker.

The model freezes global, tenant and principal session quotas; idle and maximum
lifetime; idempotency cache bounds; create/mutate/read rates; and replay import,
record and record-payload bounds. These values are operational policy inputs,
not domain data. Expiry is checked before entering a mutation lane and never
mid-commit. Cancellation can stop pre-commit work but cannot tear an atomic
commit; observe/idempotent retry resolves delivery ambiguity.

Catalog/event strings are inert bounded data, never prompts. Goal 02 contains no
model provider, private reasoning or chain-of-thought fields. Stable errors and
logs redact bearer/session/action/idempotency values and hidden state.

## Verification ownership

The machine policy enumerates 19 threats covering ownership, forgery/staleness,
response loss, races, payload/replay abuse, prompt/data separation, origins,
authentication/scopes/tenancy, exhaustion, expiry, cancellation, logging,
stdio integrity, revision drift, hidden information and adapter isolation.
Phase 1 proves projection/token controls; Phase 2 proves authoritative sessions;
Phase 3 proves local protocol handling; Phase 4 proves remote controls; Phase 5
re-runs abuse, dependency and clean-checkout gates.
