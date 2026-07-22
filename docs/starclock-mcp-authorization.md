# Starclock MCP authorization boundary

Starclock follows the MCP `2025-11-25` authorization profile as an OAuth 2.1
resource server. It does not operate an authorization server, register clients,
issue tokens, or forward an inbound token to another service.

An embedding supplies an `AccessTokenSignatureVerifier` backed by locally
trusted key material or a deployment's established validation component. The
verifier proves the signature and decodes bounded claims. Starclock then
independently requires the configured HTTPS issuer, exact canonical MCP
audience, unexpired `exp`, satisfied `nbf`, canonical tenant/principal IDs, and
bounded scopes using an injected operational clock. The raw bearer string is
passed only to that verifier and is never stored in `AuthorizationGrant`, logs,
errors, session state, replay, hashes, or RNG inputs.

The public discovery document is served at
`/.well-known/oauth-protected-resource/mcp` with `resource`,
`authorization_servers`, all eight `scopes_supported`, and header-only bearer
usage. Protected requests require `Authorization: Bearer ...` every time,
including requests carrying an existing `MCP-Session-Id`. A missing, invalid,
expired, wrong-issuer, or wrong-audience token receives 401 and a
`WWW-Authenticate` challenge naming the metadata URL. A valid token missing an
operation scope receives 403 with `error="insufficient_scope"` and the one
required scope.

The exact scope matrix is:

| Operation | Required scope |
|---|---|
| Resources, templates and prompts; `starclock_list_scenarios` | `starclock:scenario:read` |
| `starclock_create_battle` | `starclock:battle:create` |
| `starclock_observe_battle` | `starclock:battle:read` |
| `starclock_play_action` | `starclock:battle:act` |
| `starclock_export_replay` | `starclock:battle:replay` |
| `starclock_close_battle` | `starclock:battle:close` |
| `starclock_verify_replay` | `starclock:replay:verify` |
| Omniscient debug | `starclock:debug:omniscient` (reserved; no v1 MCP debug operation exists) |

Scope and credential checks occur in the bounded HTTP middleware before the
SDK performs MCP transport-session or Starclock battle-session lookup. The
validated grant is inserted as request-local context for tenant/owner binding
in `G02-P4-B3`; it contains no bearer bytes.

`authorized_loopback_router` makes this boundary executable for embedding and
tests. The CLI still exposes only explicit loopback development. Non-loopback
startup remains impossible until tenant quotas/rates, trusted-proxy policy,
security audit and bounded draining are implemented in the subsequent batches;
authentication alone is not treated as a complete remote security profile.
