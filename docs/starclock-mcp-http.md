# Starclock Streamable HTTP boundary

Batch `G02-P4-B1` exposes the frozen MCP `2025-11-25` Streamable HTTP
transport only as an explicit loopback development profile:

```text
starclock mcp serve --transport streamable-http --development-loopback --bind 127.0.0.1:8765 --allow-origin http://127.0.0.1:3000
```

`--development-loopback`, an IP socket address, and at least one exact origin
are required. The bind IP must satisfy the operating system's loopback
classification, the port must be nonzero, origins must be exact `http` or
`https` origins, and wildcard/`null`/path-bearing origins are rejected before
the listener opens. No non-loopback or anonymous remote production profile
exists yet; that remains gated on the complete authorization work in
`G02-P4-B2`.

The single endpoint is `/mcp`. It uses the official SDK's stateful POST and
DELETE implementation. Initialization returns an SDK-generated UUID in
`MCP-Session-Id`; subsequent transport-session requests must carry that header.
Every POST/DELETE also requires exact `MCP-Protocol-Version: 2025-11-25`.
`Host` must equal the configured bind authority. A present `Origin` must match
the configured allowlist byte-for-byte. `Forwarded` and `X-Forwarded-*` are
rejected because this local profile trusts no proxy. The optional GET listening
channel returns 405, as permitted by the frozen transport specification, so the
profile has no long-lived server-initiated SSE stream.

The outer boundary admits at most 32 active requests. It rejects additional
work immediately with 503 and `Retry-After: 1`. A complete request is collected
under a 2 MiB cap before SDK JSON decoding. POST responses are buffered under a
2 MiB cap before delivery; an oversized committed response becomes a generic
500 and application idempotency resolves delivery ambiguity. These transport
caps are intentionally tighter than the application's 64 MiB decoded replay
limit, just as the stdio frame cap is: a replay transported over HTTP must
satisfy both layers.

The exact contract is retained in `policy/mcp-http-boundary.json` and checked by:

```text
node tools/agent-control/verify-mcp-http-boundary.mjs
cargo test -p starclock-mcp --all-targets --all-features
cargo test -p starclock-cli --test cli_contract --all-features
```
