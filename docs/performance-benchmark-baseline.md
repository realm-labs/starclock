# Phase 3 Performance Benchmark Baseline

`G01-P3-B7` establishes workload revision
`g01-phase3-benchmark-v1`. The harness is a release-mode Cargo example behind
the `benchmark-harness` feature; it is not a production CLI command and its
instrumentation never enters canonical state, events, RNG or replay bytes.

## Workloads

The fixed seed is `7`. Every job owns its `Battle`, RNG, transaction scratch and
verifier state. `BenchmarkFactory` retains one immutable `Arc<CombatCatalog>`;
the four-worker workload shares only that catalog and replay bytes.

| Workload | Fixed work |
|---|---:|
| `ordinary-apply-v1` | 1,000 accepted commands in a two-combatant state |
| `trigger-heavy-proxy-v1` | 500 accepted commands; the player action has eight damage operations |
| `invalid-rejection-v1` | 10,000 stale-command rejections |
| `hash-small/medium/large-v1` | 5,000 streaming hashes over 2/4/8 combatants |
| `one-shot-replay-100-v1` | 20 isolated 100-command audits |
| `one-shot-replay-500-v1` | 5 isolated 500-command audits |
| `concurrent-replay-shared-catalog-v1` | 16 isolated 100-command audits on four workers |

The trigger rule evaluator does not land until Phase 4. The named heavy
workload is therefore an explicit Phase 3 proxy: eight ordered
operation/event-producing hits stress the transaction path that triggers will
extend. `G01-P4-B11` must retain the ID or version it when replacing this proxy
with real trigger evaluation.

The allocator counter is pinned at `0.8.1`, is a dev-dependency of
`starclock-cli`, and counts only the current worker thread. Concurrent workers
measure independently and report the maximum live bytes of one worker as peak
bytes/job. Combat's feature-gated snapshot counts canonical semantic-copy
bytes and exact latest-command journal, event and operation entries. It is
non-authoritative and excluded from the public compatibility contract.

## Runner and budgets

Shared native CI runs one sample and enforces only the deliberately broad
order-of-magnitude ceilings in
[`benchmark-workloads.json`](../policy/benchmark-workloads.json): 30 seconds
total, 10 seconds per row, at least 100 commands/second/core, at most 2 GB
allocated per row and 128 MiB peak live bytes/job. This is a hang/quadratic
regression smoke gate, not a latency promise.

Strict budgets belong only to runner
`starclock-bench-win10-i7-10700f-v1`: Windows 10 build 19045, x64,
Intel i7-10700F, 16 logical processors, at least 64 GB memory, Rust 1.97.0
MSVC. Strict execution requires the matching explicit runner ID and verifies
the observed host fingerprint before applying per-row budgets. A different
machine may produce informative results but cannot satisfy the strict gate.

## Recorded baseline

The committed five-sample median report is
[`phase3-baseline-windows-x64.json`](../evidence/core-combat-v1/performance/phase3-baseline-windows-x64.json).
Representative medians are:

| Workload | Time | Commands/s/core | Allocated bytes | Peak bytes/job |
|---|---:|---:|---:|---:|
| ordinary apply | 5.53 ms | 180,802 | 3,657,640 | 6,324 |
| heavy proxy apply | 3.46 ms | 144,596 | 4,162,736 | 21,540 |
| 100-command one-shot replay | 11.01 ms / 20 jobs | 181,714 | 7,469,460 | 13,744 |
| 500-command one-shot replay | 13.02 ms / 5 jobs | 192,034 | 9,311,365 | 39,344 |
| four-worker replay | 2.79 ms / 16 jobs | 143,174 | 5,975,568 | 13,744 |

The concurrent total throughput is 3.151 times the sequential 100-command
audit workload. The ordinary stream records 864,250 semantic-copy bytes,
23,479 journal entries and 6,492 event entries. The heavy proxy records
432,125 semantic-copy bytes, 21,104 journal entries, 5,992 events and 1,000
operation allocations. All three streaming-hash workloads allocate zero bytes;
their canonical state sizes are 807, 1,139 and 1,803 bytes. Invalid rejection
also allocates zero, produces no journal entry and preserves state/RNG.

Run the broad or strict gate with:

```text
node tools/benchmark/verify.mjs
STARCLOCK_BENCH_RUNNER_ID=starclock-bench-win10-i7-10700f-v1 \
  node tools/benchmark/verify.mjs --strict --samples 5 --output PATH
```

PowerShell uses `$env:STARCLOCK_BENCH_RUNNER_ID=...` before the strict command.
Phase 8 reuses these inputs and runner identity for the representative final
report rather than inventing a new harness.
