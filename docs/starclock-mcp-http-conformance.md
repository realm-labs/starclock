# Starclock MCP HTTP conformance and load

The retained HTTP client in `crates/starclock-mcp/tests/http_conformance.rs`
uses raw HTTP/1.1 over `TcpStream` against an ephemeral real loopback listener.
It does not call MCP handlers directly. The client supplies the exact Host,
Origin, protocol and bearer headers, negotiates MCP `2025-11-25`, sends the
initialized notification and discovers all seven tools. It also probes health,
readiness and metrics through the production network-header boundary.

The conformance script completes the basic frozen Standard scenario using only
the first public `use_ability` action, or `pass_interrupt` when no ability is
offered. In-process, stdio and HTTP tests all consume the same retained
[`basic-transport-trace.json`](../evidence/agent-control-mcp-v1/protocol/basic-transport-trace.json).
That artifact freezes nine decision-boundary state hashes and the exact
987-byte, nine-command replay envelope. Each transport must reproduce every
hash and every replay byte, not only the terminal result.

After the primary conformance pass, eight clients initialize independent MCP
transport sessions concurrently. Every client creates, completes, exports,
verifies and closes its own battle, and all eight traces and replays equal the
same frozen artifact. The executable and retained evidence are verified with:

```text
cargo test -p starclock-mcp --test http_conformance --all-features
node tools/agent-control/verify-mcp-http-conformance.mjs
```

## HTTP adapter baseline

The release-only `g02_http_benchmark` harness isolates the in-process Axum
middleware, authorization/rate classification, MCP SDK dispatch and JSON/SSE
serialization layers. It deliberately excludes TCP and does not attribute
combat cost to the adapter. The three fixed workloads are 256 authorized
observations, one committed action in each of 16 sessions, and creation plus
retention of 16 sessions.

On the designated Windows runner, the five-sample medians are:

| Workload | Median latency/op | Throughput | Allocated bytes | Peak live | Retained/session |
|---|---:|---:|---:|---:|---:|
| 256 observations | 221.083 µs | 4,523/s | 45,292,420 | 1,108,389 | 1,065,856 |
| 16 first actions | 321.775 µs | 3,107/s | 4,044,278 | 347,817 | 18,161 |
| 16 resident creates | 264.456 µs | 3,781/s | 3,162,914 | 294,515 | 15,749 |

The observation row's retained value includes warmed current-thread
executor/transport capacity and is not a domain-session size claim. The
resident-create row is the reviewed simultaneous-session measurement; its
aggregate peak is divided by 16 for the recorded 18,408 peak bytes/session.
Payload bytes, allocations and hashes are exact across samples; timing alone
is aggregated by median.

Run the broad smoke or designated-runner strict gate with:

```text
node tools/agent-control/verify-mcp-http-benchmark.mjs
$env:STARCLOCK_BENCH_RUNNER_ID='starclock-bench-win10-i7-10700f-v1'
node tools/agent-control/verify-mcp-http-benchmark.mjs --strict --samples 5
```
