# MCP 2025-11-25 capability lock

`G02-P0-B2` freezes MCP revision `2025-11-25` and the official Rust SDK
`rmcp 2.2.0`. The exact tag, registry checksums, features, licenses, fixture and
unsupported assumptions are machine-readable in
[`policy/mcp-sdk-lock.json`](../policy/mcp-sdk-lock.json).

## Primary sources

The reviewed protocol pages are the official MCP
[`2025-11-25` specification](https://modelcontextprotocol.io/specification/2025-11-25),
its [transport contract](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports),
[cancellation utility](https://modelcontextprotocol.io/specification/2025-11-25/basic/utilities/cancellation),
[tool contract](https://modelcontextprotocol.io/specification/2025-11-25/server/tools),
and [resource contract](https://modelcontextprotocol.io/specification/2025-11-25/server/resources).
SDK behavior is bound to the official
[`rmcp-v2.2.0` tag](https://github.com/modelcontextprotocol/rust-sdk/tree/rmcp-v2.2.0)
at peeled commit `519577601db3823616dbd7c4eb84ed569d8e17d4`.

The protocol uses newline-delimited JSON-RPC over stdio and a single POST/GET
endpoint for Streamable HTTP. HTTP clients send the negotiated
`MCP-Protocol-Version`; servers reject unsupported versions. Cancellation is a
notification that marks a result unwanted and asks associated processing to
cease—it is not transactional rollback.

## SDK selection

The selected crates are `rmcp 2.2.0` and its macro crate at their crates.io
SHA-256 checksums. Both declare Apache-2.0. Default features are disabled; the
reviewed set enables client/server protocol handling, macros, stdio/async I/O,
and Streamable HTTP server support. OAuth/auth features are intentionally not
enabled because Goal 02 supplies a narrower verified authorization boundary in
Phase 4 rather than treating transport construction as authorization.

The fixture is a standalone locked Cargo project under
`tools/mcp-sdk-capability`. Its real stdio child emits only MCP messages on
stdout, negotiates `2025-11-25`, and exposes tool discovery. A duplex client
proves input/output schema generation, `structuredContent`, resources,
resource templates, reads, typed missing-tool errors and cancellation delivery.
A loopback Streamable HTTP test proves successful frozen-version initialization
and rejection of an inconsistent protocol header/body pair. The exact passing
test names and toolchain are retained in
[`mcp-sdk-capabilities.json`](../evidence/agent-control-mcp-v1/protocol/mcp-sdk-capabilities.json).

## Explicit limitations

The SDK also implements capabilities and revisions Starclock does not expose.
Newer protocol negotiation, tasks, prompts, completions, sampling, elicitation,
logging and subscriptions remain out of contract. MCP transport sessions never
stand in for Starclock session ownership or idempotency. Cancellation may stop
pre-commit work but cannot tear an atomic domain commit. Streamable HTTP still
requires Starclock-owned origin, authorization, scope, tenant, TLS, rate-limit,
quota and audit enforcement. Text content is convenience output; the
protocol-neutral schema and structured output are authoritative.
