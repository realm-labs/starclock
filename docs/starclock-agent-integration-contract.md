# Starclock Agent Integration Contract

The stable Goal 02 entry point is the protocol-neutral `starclock-agent-api`.
MCP is an adapter for interoperable agent hosts; it is not the authority for
combat rules and it is not required by in-process consumers.

## In-process Rust

Run the complete minimal loop with:

```text
cargo run -p starclock-agent-api --example g02_in_process
```

The source is
[`g02_in_process.rs`](../crates/starclock-agent-api/examples/g02_in_process.rs).
It loads the frozen production catalog, selects only an offered opaque action,
uses decision/hash/idempotency preconditions, reaches a terminal outcome,
exports the canonical replay and verifies it from a fresh battle. Applications
should retain the `AgentSession` incrementally and serialize mutation per
session; they must not replay a growing command prefix to implement live play.

The stable library responsibilities are:

- `schema`: `agent-api-v1`, exact string integers, IDs and hashes;
- `observation`: bounded player-visible DTOs and separately acknowledged debug;
- `action`: public offered summaries over a private exact-command table;
- `session`: production factory, incremental session, owner registry and replay;
- `error`: bounded stable machine error codes and ordered context.

No MCP, HTTP or async runtime is needed for this path.

## Local stdio MCP

An MCP host launches:

```text
starclock mcp serve --transport stdio
```

Stdout is newline-delimited JSON-RPC only. The host must negotiate MCP
`2025-11-25`, send `notifications/initialized`, discover schemas rather than
guess them, and close stdin to stop the server. A dependency-free discovery
client is retained at
[`stdio-discovery.mjs`](../examples/agent-control/stdio-discovery.mjs):

```text
cargo build -p starclock-cli
node examples/agent-control/stdio-discovery.mjs target/debug/starclock
```

On Windows, pass `target/debug/starclock.exe`. Production integrations should
normally let their MCP host own this framing; the script exists to make the
wire contract inspectable.

## Authorized Streamable HTTP embedding

The supported network surface is stateful Streamable HTTP at `/mcp`. Goal 02
ships an explicit loopback-only listener. `authorized_loopback_router` requires
an `AuthorizationPolicy` whose expected audience and protected-resource
metadata URL exactly match the configured listener. Every request is
revalidated and mapped to tenant/principal authority before MCP session work.

[`g02_authorized_http.rs`](../crates/starclock-mcp/examples/g02_authorized_http.rs)
shows the embedding seam:

```text
cargo run -p starclock-mcp --example g02_authorized_http
```

The runnable example constructs a deny-all authorized router and exits. This is
intentional: Starclock does not invent a token signature format or operate an
authorization server. A deployment passes an established local signature/JWT
or introspection verifier through `AccessTokenSignatureVerifier`, then serves
the returned Axum router within its reviewed TLS/proxy boundary. The example
cannot bind a non-loopback address; Goal 02 exposes no non-loopback startup
profile until TLS/proxy attestation and a deployment security-audit sink exist.

All protected calls use the exact thirteen-scope matrix in
[`mcp-authorization.json`](../policy/mcp-authorization.json). Origin, Host,
protocol, session, payload, worker, quota and rate controls compose with auth;
an MCP transport session is never battle authority.

## MCP and CLI surface

MCP revision `2025-11-25` retains the seven frozen Goal 02 Battle tools:

```text
starclock_list_scenarios
starclock_create_battle
starclock_observe_battle
starclock_play_action
starclock_export_replay
starclock_close_battle
starclock_verify_replay
```

Goal 04 adds six Activity tools without changing those Battle contracts:

```text
starclock_create_universe
starclock_observe_activity
starclock_play_activity_action
starclock_export_activity_replay
starclock_close_activity
starclock_verify_activity_replay
```

Activity actions use the same owner, opaque-token, state precondition,
idempotency, lease and quota model. Their settlement boundary is the next
external Activity decision or terminal state; nested battles settle inside the
protocol-neutral session facade. The MCP adapter never mutates Activity or
Battle state directly.

The adapter exposes four static resources, two RFC 6570 templates and the fixed
`starclock_battle_loop` prompt. `structuredContent` and `agent-api-v1` are
authoritative; convenience text is not. The Goal 02 CLI adds only:

```text
starclock mcp serve --transport stdio
starclock mcp serve --transport streamable-http --development-loopback \
  --bind 127.0.0.1:3001 --allow-origin https://agent.example
```

The second command is an unauthenticated explicit development listener. Use the
library embedding above when authorization is required. Neither command enables
non-loopback serving.

## When not to use MCP

Use the in-process library or a separately reviewed batch/RPC adapter when the
consumer controls the Rust process, needs high-throughput verification or
training, runs many short battles, or needs to amortize serialization. MCP is
designed for interoperable tool hosts and human-scale agent decisions, not one
call per hit, event, trigger or resolver operation.

Do not use MCP to upload arbitrary catalogs or battle specifications, bypass
offered actions, run model inference, store chain-of-thought, provide accounts
or durable sessions, or expose an Internet listener without the complete
deployment security profile. Bulk replay verification should call the
protocol-neutral/batch boundary directly.

The reproducible byte and semantic lock is retained in
[`contract-freeze.json`](../evidence/agent-control-mcp-v1/contracts/contract-freeze.json)
and checked by `node tools/agent-control/verify-agent-contract-freeze.mjs`.
