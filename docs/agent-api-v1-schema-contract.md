# Agent API v1 schema contract

`G02-P0-B3` freezes the protocol-neutral wire vocabulary before Rust DTOs or MCP
adapters are implemented. The limits and policy live in
[`policy/agent-api-v1.json`](../policy/agent-api-v1.json); the normative JSON
Schemas and goldens live under [`schemas/agent-api-v1`](../schemas/agent-api-v1).

## Exact numeric and identity encoding

All authoritative integers—including runtime IDs, revisions, cursors, counts,
HP and fixed-point backing values—are canonical base-10 strings. Signed
fixed-point fields carry a `_scaled` suffix and represent millionths. Hashes are
lowercase 64-character hexadecimal strings. JSON numbers and floats are not an
authoritative path. This keeps JavaScript and cross-language clients exact over
the full Rust integer domains.

Operational session/action/cursor/idempotency values are opaque URL-safe strings
with bounded length. Clients compare or return them; they do not parse meaning
from them. Offered action tokens are distinct from private exact combat commands.

## Visibility and ordering

`player_visible` is the default and is exercised by both observation goldens.
It excludes enemy controller graph/cursor/candidates, future RNG, unrevealed
intent, exact commands, internal stores and resolver queues. Public intent is
optional and appears only when authored policy exposes it. `omniscient_debug`
requires separate authorization plus `debug_authorized: true`; Phase 1 adds and
tests its explicit extension without changing the default envelope.

Collections retain domain canonical order: units, effects and timeline entries
by stable runtime identity; actions by exact offered-command order; events by
ascending battle event identity. An event cursor is opaque and means
“strictly after”; retention loss returns `event_cursor_expired` rather than a
silently incomplete page.

## Frozen bounds

Requests are at most 16 KiB, observations 256 KiB and errors 32 KiB. One
observation contains at most 256 actions, 128 units, 2,048 effects, 256 timeline
entries and 256 event summaries; each summary is at most 4 KiB and a session
retains at most 8,192 summaries. Complete accepted facts remain in replay even
when summaries expire.

One external action settlement is capped at 4,096 accepted commands, 65,536
emitted events and 262,144 resolver operations. Reaching a budget produces
`settlement_budget_exceeded` under the later session transaction policy; an
adapter may not silently truncate authoritative settlement.

The ordinary golden exercises a compact player boundary. The trigger-heavy
golden exercises automatic enemy settlement, effects, target invalidation,
return/replacement, paging pressure and explicit settlement counters. The error
golden freezes a non-mutating, retryable stale-decision response.
