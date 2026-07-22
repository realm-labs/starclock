# Session events and terminal observations

Each successful `Battle::apply` is processed once at commit time. Its complete
exact command/resulting-hash boundary is appended to the replay trace, while
each emitted `BattleEvent` is converted to the fixed payload-free public
summary and added to a bounded observation window.

The session retains the newest 8,192 summaries. `observe` accepts only opaque
canonical `event_<id>` cursors and returns at most 256 summaries whose event ID
is strictly greater than the cursor. The returned cursor names the last emitted
summary (or preserves the request when the page is empty), and
`events_truncated` is true when another retained page exists.

Cursor outcomes are exact:

- a cursor immediately before the oldest retained summary remains valid;
- an older evicted cursor returns `event_cursor_expired`;
- a future event identity or wrong opaque family returns `invalid_request`;
- summary eviction never removes accepted command/hash replay facts.

Action responses capture the latest cursor before committing, so their cached
next observation includes precisely that action's external/internal settlement
events up to the page bound. Independent observations may resume from any
still-retained cursor.

Terminal `won`, `lost` and `faulted` states produce stable observations with no
decision and no offered actions. Tests exercise concession to `lost`, ordered
terminal events and complete trace/controller retention. Core battle suites
already exercise deterministic fault commits; the same `Faulted` phase maps to
the frozen `faulted` agent status without a separate mutation path.
