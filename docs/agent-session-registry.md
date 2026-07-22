# In-memory agent session registry

The protocol-neutral registry binds each active session to one validated
tenant/principal owner and one mutex-protected mutation lane. Registry lookup
checks that binding before it exposes an observation, mutation result or replay.
Concurrent requests for one session therefore recheck decision/hash
preconditions in a single lane, while unrelated sessions do not share a domain
lock.

Creation accepts an injected monotonic clock and opaque session-ID source.
Scenario and visibility validation plus quota checks occur before the ID source
is called. A hosting adapter is responsible for supplying unpredictable,
high-entropy IDs; duplicate IDs fail closed. Owner values, IDs, clock readings,
lease state and quota counters never enter battle state, RNG, replay records or
canonical hashes.

The frozen active-session policy is:

- 1,024 sessions globally;
- 64 sessions per tenant;
- 16 sessions per tenant/principal binding;
- 1,800 seconds of idle lifetime;
- 14,400 seconds of absolute lifetime.

Expiry time is sampled once before attempting to enter a session lane. A
request admitted at that sample completes atomically without a second clock
read, and a regressing injected clock is an adapter failure. Successful owned
operations renew only the idle timestamp; no operation can renew absolute
lifetime.

Close and expiry remove the authoritative session immediately and release all
three quota counts. A bounded 1,024-entry terminal tombstone window preserves
stable `session_closed` or `expired_session` outcomes without retaining battle
state indefinitely. Repeated close, expired access and unauthorized access are
inert.

Concurrency tests start simultaneous same-session mutations and prove exactly
one stale-precondition winner, then reorder mutations across independently
owned sessions and prove identical state hashes and replay bytes. Quota tests
cover principal, tenant and global limits independently and prove rejected
creation does not consume an operational ID.
