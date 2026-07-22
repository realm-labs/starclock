# Session preconditions and idempotency

`PlayActionRequest` matches the frozen action request schema: schema revision,
session ID, expected decision ID, expected state hash, opaque action token and
idempotency key are all exact checked values.

The serialized mutation lane applies checks in this order:

1. session ownership;
2. existing idempotency-key lookup and full-request equality;
3. cache capacity;
4. current decision and state-hash preconditions;
5. decision-scoped token lookup;
6. authoritative commit and internal settlement;
7. stable observation construction, canonical JSON serialization and cache
   insertion before the response is returned.

A repeated key with the identical request returns the original cached response
object and byte-identical canonical JSON even though the battle has advanced.
The frozen `idempotent_replay` field remains the value recorded by the original
response (`false`); changing it on retry would violate the stronger frozen
response-loss requirement that the returned bytes are identical. A reused key
with any different payload returns `idempotency_conflict` before current-state
checks.

Each session retains at most 1,024 entries and each committed response at most
512 KiB, matching the threat-model limits. Tests discard an initial response,
retry it, and prove state hash, replay length, controller records and combat RNG
draw count do not change. Separate tests cover stale hashes, forged tokens,
conflicting reuse and two sequential racing-equivalent requests; every rejected
case is inert and only the first request commits.

The cached response is the complete frozen action envelope, including the next
player-visible observation. Event pages are initially empty at cursor
`event_0`; retained cursor-based facts are added in G02-P2-B4 without changing
the precondition or cache semantics.
