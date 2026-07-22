# Starclock MCP tenant authority and rates

An authenticated HTTP request derives its operational `AgentSessionOwner`
only from the validated request-local `AuthorizationGrant`. The MCP transport
session ID selects SDK transport state; it is never authority. A client may
therefore reuse one MCP transport session while changing credentials, but each
battle create, observe, action, replay export and close is checked against the
tenant and principal on that individual request.

Ownership is checked before a registry lookup discloses any result. A different
tenant or principal receives the same bounded `session_not_owned` tool error
whether the requested battle is active, closed or expired. The error contains
no session ID, state hash or idempotency result. Since idempotency entries live
inside one owner-bound session, a cross-authority request is rejected before
the cache is examined. A same-authority retry of the same canonical action
returns the exact cached committed response.

The shared registry retains the frozen active-session limits: 1,024 global,
64 per tenant and 16 per principal. HTTP conformance tests drive the authorized
request path to the exact principal and tenant boundaries, including multiple
principals using one tenant.

Authenticated HTTP admission also uses an injected monotonic operational clock
and independent fixed one-minute windows:

| Class | Authority key | Requests per minute |
|---|---|---:|
| Create | tenant + principal | 30 |
| Mutation (`play`, `close`, MCP `DELETE`) | tenant | 600 |
| Read (all other authenticated MCP requests) | tenant | 1,200 |

The limiter retains at most 4,096 tenant entries and 4,096 principal entries,
prunes expired windows before refusing new identities, and fails closed if its
clock regresses or lock becomes unavailable. Exhaustion returns HTTP 429 with a
bounded `Retry-After` of 1 through 60 seconds and a generic body. Identity,
bearer, session and battle data never enter the response, combat state, replay,
RNG or canonical hashes.

Non-loopback startup remains unavailable. Tenant binding and rates complete
this batch's controls, but they do not substitute for the trusted-proxy, audit
and bounded-drain requirements of the later remote profile.
