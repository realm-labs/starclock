# Agent session Phase 2 acceptance and baseline

`G02-P2-B7` closes the protocol-neutral session phase with workload revision
`g02-agent-session-baseline-v1`.

## Six-scenario acceptance

An integration controller sees only `AgentObservation`, selects the first
offered deterministic progress action (`use_ability`,
`commit_prepared_action`, or `advance`), and submits its opaque token with the
current boundary/hash preconditions. It never reads or constructs a combat
`Command`.

| Frozen Standard scenario | External steps | Replay commands | Terminal hash |
|---|---:|---:|---|
| `basic-single-wave` | 8 | 20 | `2ab5d393…21e0` |
| `cocolia-phase-change` | 2 | 4 | `d8ad2d64…1baa` |
| `elite-control-counter` | 6 | 14 | `22d2606b…1118` |
| `layered-toughness` | 3 | 6 | `aedde8be…706c` |
| `multi-wave-dot-revival` | 17 | 40 | `b433b020…1a77` |
| `target-invalidation-and-return` | 22 | 54 | `a1a7a6d3…e6ba` |

Every result is `won`, matches the existing production-domain golden hash and
round-trips its entire exported command/hash stream against a fresh battle.
The replay also records system-owned advancement between externally visible
stable boundaries, so replay command counts are intentionally larger than the
number of controller submissions.

## Performance workloads

The release-only harness uses the already pinned `allocation-counter 0.8.1`
as a dev dependency. Instrumentation is thread-local and never enters session
state, replay, observations or production features. Projection and registry
rows intentionally omit JSON serialization from timing; `payload_bytes` is a
stable size fact measured afterward. MCP/JSON costs get their own later row.

| Workload | Fixed work | Five-sample median | Operations/s | Allocated | Peak live | Retained |
|---|---:|---:|---:|---:|---:|---:|
| `projection-1000-v1` | 1,000 direct projections | 14.61 ms | 68,457 | 4,800,000 B | 2,851 B | 0 B |
| `agent-step-100-v1` | 100 isolated ability settlements | 6.82 ms | 14,664 | 4,660,324 B | 942,179 B | 930,840 B |
| `registry-observe-1000-v1` | 1,000 owned locked projections | 14.89 ms | 67,143 | 4,809,000 B | 2,860 B | 0 B |
| `resident-sessions-16-v1` | create and retain 16 sessions | 0.35 ms | 45,950 | 259,408 B | 125,015 B | 124,496 B |

The step row retains one committed response/idempotency entry in each of 100
sessions, which explains its measured retained bytes. The resident row retains
about 7.6 KiB per newly started session while sharing the production catalogs.
Read-only direct and registry projections retain no measured allocation.

The committed report is
It was captured as a five-sample median on the designated Windows x64 runner
and passes the reviewed strict budgets in
[`agent-benchmark-workloads.json`](../policy/agent-benchmark-workloads.json).
Shared CI uses broad hang/order-of-magnitude ceilings rather than treating
uncontrolled host timing as a product promise.

Run the smoke or strict gate with:

```text
node tools/agent-control/verify-agent-benchmark.mjs
STARCLOCK_BENCH_RUNNER_ID=starclock-bench-win10-i7-10700f-v1 \
  node tools/agent-control/verify-agent-benchmark.mjs --strict --samples 5
```
