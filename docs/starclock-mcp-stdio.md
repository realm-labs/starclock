# Starclock local MCP stdio service

Batch `G02-P3-B4` adds the exact command:

```text
starclock mcp serve --transport stdio
```

This profile opens no listener and derives no ambient remote identity. It loads
the validated production factory once, shares its immutable catalogs across a
bounded in-memory registry, and binds every session to the single local stdio
invoker. Operational expiry uses a process-local monotonic clock. Opaque local
session IDs include process/start uniqueness plus a monotonic ordinal; they are
not authorization credentials and never enter battle state, replay or RNG.

Each newline-delimited input frame is capped at 16 KiB before JSON decoding.
An oversized frame terminates that local transport without echoing its content.
This transport cap is intentionally tighter than the application verifier's
64 MiB decoded replay ceiling; replay artifacts sent through stdio must fit the
complete 16 KiB JSON-RPC frame. Session, settlement, observation, event,
idempotency and replay decoder limits remain independently enforced.

The MCP SDK owns stdout and emits only newline-delimited JSON-RPC frames.
Starclock writes no startup banner or diagnostic there. Startup/transport
failure is reduced to a generic CLI diagnostic on stderr after serving stops;
input bytes, session IDs, action tokens and hidden state are never logged.
Normal EOF performs a clean successful shutdown.

Child-process tests initialize the frozen `2025-11-25` protocol, discover the
seven tools, assert every stdout line is a JSON-RPC object and require empty
stderr. A separate oversized-frame child proves zero stdout bytes, a generic
stderr-only failure and rejection before JSON decoding.
