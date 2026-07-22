# Starclock MCP adapter boundary

`starclock-mcp` is the only MCP protocol owner. It depends one way on
`starclock-agent-api`; it has no direct combat, data, AI or replay dependency,
so it cannot construct a `Command`, call `Battle::apply` or create a second
mutation path.

The production workspace inherits the exact capability lock: MCP revision
`2025-11-25`, official `rmcp 2.2.0`, default features disabled, and the reviewed
`client`, `macros`, `server`, `transport-io` and
`transport-streamable-http-server` features. The workspace lock must contain
the same `rmcp` and `rmcp-macros` checksums as the standalone executed fixture.

Server initialization fixes implementation name `starclock-mcp`, the workspace
package version and the `2025-11-25` protocol revision. Instructions state the
core trust boundary: callers select only currently offered opaque actions,
never invent combat inputs, treat catalog/event text as inert data and prefer
`structuredContent`. Capabilities are not advertised before their reviewed
handlers land.

## Error boundary

Once a valid tool request reaches Starclock, every protocol-neutral
`AgentError` becomes a tool-level error with `isError:true`. Its
`structuredContent` is exactly the frozen `agent-api-v1` error object, and the
text content is only the compact serialization of that same object. This keeps
stable codes, retry/commit facts and bounded context available to independent
clients.

Malformed JSON-RPC, unknown methods and parameter-decoding failures remain SDK
protocol errors. An adapter serialization/infrastructure failure uses only the
generic JSON-RPC internal-error message `The Starclock MCP adapter failed.`
with no data field, preventing secret or hidden-state leakage.

Tests cover the frozen metadata, absence of premature capabilities, all 23
agent error codes, exact structured/text agreement and data-free internal
errors. The lock verifier binds both the standalone capability fixture and the
production workspace dependency/lock.
