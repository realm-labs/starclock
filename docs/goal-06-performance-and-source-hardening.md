# Goal 06 Performance and Source Hardening

This document is normative for `G06-P4-B1`.

## Measured service shape

The release-only `g06_dynamic_assembly_benchmark` measures five Goal 06
workloads without adding instrumentation to authoritative state:

| Workload | Iterations | Stable-runner observation |
|---|---:|---:|
| Combat-input validation and digest | 10,000 | 64.0 ms |
| Cold assembly across all entry pairs | 33 | 23.2 s |
| Exact-key warm assembly | 10,000 | 15.0 ms, zero allocations |
| Capacity-one eviction replay | 256 | 4.02 s |
| Shared-catalog concurrent complete runs | 16 | 2.85 s, 784 transaction/hash boundaries |

Elapsed values are the recorded Windows x64 sample, not normative results for
other hosts. Policy ceilings are deliberately broader and deterministic final
digests, cache counters and transaction counts are mandatory on every host.
Allocation-counter measurements are diagnostic because concurrent worker
allocations are outside its coordinator-thread scope.

## Cache memory decision

The first measurement used the Phase 2 default of 64 entries. Thirty-three
distinct assemblies retained about 317 MiB because each completed
materialization currently owns an independently validated composite
`CombatCatalog`. A 64-entry service cache could therefore retain substantially
more memory than intended.

P4-B1 reduces the production default to 8 entries. The same 33-entry workload
now retains about 76.9 MiB and must remain below the 128 MiB policy ceiling.
Cache capacity and eviction never enter canonical state, RNG, replay or battle
identity, so this change does not alter authoritative results.

Warm lookup is already cheap and allocation-free. Cold construction still
causes high cumulative allocation traffic because `CombatCatalogBuilder`
copies the base catalog before appending selected rules. A future layered or
persistent catalog representation could reduce miss cost, but it is not
required for Goal 06 correctness and must not leak a generic numeric or
mode-specific type into combat APIs.

## Verification budgets

`node tools/goal06/verify-performance.mjs` verifies the checked-in stable
sample, source/policy hashes, deterministic workload digests, cache shape and
all elapsed/allocation ceilings. Adding `--run` executes one local release
sample.

The daily repository gate remains the cached incremental
`node tools/repository-check/run.mjs` profile with a hard 180-second budget.
Release-mode performance and the 33-entry matrix are explicit broad checks and
are not inserted into every edit cycle.

## Source structure

All touched handwritten Rust files remain below 1,200 physical lines. The MCP
surface-parity workflow lives in its own integration test rather than growing
the main tool adapter. The dynamic reconstruction fixture remains isolated
from replay transport implementation. No file-size exception or new
dependency was added.
