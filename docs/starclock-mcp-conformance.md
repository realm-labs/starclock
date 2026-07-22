# Starclock MCP stdio conformance

The Phase 3 acceptance client is a protocol-independent JSON-RPC client in
`crates/starclock-cli/tests/mcp_stdio.rs`. It launches the production CLI as a
child process and communicates only through newline-delimited stdin/stdout; it
does not link to `StarclockMcp`, the registry, or battle internals.

Run the retained conformance proof from the repository root:

```text
cargo test -p starclock-cli --test mcp_stdio --all-features
node tools/agent-control/verify-mcp-stdio-conformance.mjs
```

The client negotiates MCP `2025-11-25`, discovers all seven typed tools, both
static resources, both RFC 6570 templates and the usage prompt, and reads
bounded inert resources. It then completes the basic frozen Standard scenario
using only each observation's legal action values. The retained trace is eight
external actions, nine replay commands and terminal hash
`5021cdd6…1b507ec`. It exports the replay, closes the session, verifies the
replay against a fresh battle and requires the same final hash.

Negative probes submit an obsolete decision/hash/token tuple and a tool call
with missing required arguments. Both leave the accepted state hash unchanged
and the server remains usable. A `notifications/cancelled` continuity probe
records the frozen advisory-cancellation rule: notification delivery must not
roll back authoritative work, and a subsequent observe succeeds. The test
finally closes stdin and requires success, no trailing stdout frame and empty
stderr. The separate frame-limit test retains the generic stderr-only failure
contract for oversized input.

## MCP Inspector launch fixture

`tools/agent-control/mcp-inspector-config.json` is an Inspector-compatible
server configuration. Start Inspector from the repository root and select the
`starclock` server, for example:

```text
npx @modelcontextprotocol/inspector --config tools/agent-control/mcp-inspector-config.json --server starclock
```

The fixture uses `cargo run --quiet` so Cargo does not write ordinary progress
on stdout. Starclock itself reserves stdout for the MCP SDK. Inspector is a
manual interoperability surface; the retained Rust scripted client and JSON
evidence are the deterministic automated acceptance authority.
