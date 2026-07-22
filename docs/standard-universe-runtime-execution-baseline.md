# Standard Universe Runtime Execution Baseline

## First vertical slice

The first executable slice is
`goal04.standard-universe.world01.vertical-slice-v1`. It is deliberately a
representative World 1 path, not a claim that World 1 is complete.

It fixes World 1 standard difficulty 1, Preservation, the Asta/Clara/Kafka/
Firefly participant fixture, room 100, encounter-pool condition 3 and encounter
group 1001. The exact battle contains Antibaryon and Baryon variants. After a
verified nested battle it awards level-1 Divine Construct: Resonance Transfer,
uses the Enhance Blessing service to reach level 2, and settles terminally.

The path must cross offered boundaries for Path selection, a spatial-free
domain hub, encounter preparation, battle handoff/result, reward, service and
terminal settlement. Graph, encounter, reward and battle RNG streams are
independent. No coordinate or `position_hint` is an input.

This slice is implemented incrementally across P1–P3. Every responsible batch
must keep it compiling or make its still-missing boundary explicit. A complete
World/run claim begins only after the Standard profile and assigned mechanics
can construct and finish the authored graph.

## Server-verification workloads

Workload revision `g04-standard-universe-service-v1` freezes six shapes:

1. load and validate both bundles ten times;
2. apply 1,024 incremental accepted Activity commands;
3. reject 4,096 stale/not-offered commands with byte/RNG identity;
4. complete 32 seeded World 1 activities with one shared catalog;
5. linearly verify 32 complete Activity replays;
6. run 64 isolated sessions sharing immutable catalogs.

Every row reports elapsed time, throughput, allocation count/bytes, peak and
retained bytes, catalog clone count, replayed-prefix count and final hash.
`allocation-counter = 0.8.1` remains benchmark-only and non-authoritative.

P0 freezes broad shared-CI safety ceilings and structural invariants, not a
performance claim. `G04-P2-B7` measures the first post-slice baseline and sets
provisional budgets. `G04-P6-B3` freezes strict stable-runner budgets. Per-session
catalog clones and incremental replay-prefix reconstruction are already fixed at
zero; changing those is an architecture change, not a tuning adjustment.

### Phase 2 provisional measurement

`G04-P2-B7` records only the generic Activity-core slice that exists before the
Standard profile compiler. Its release-mode rows cover 4,096 canonical state
hashes, 4,096 alternating stale/not-offered battle commands and 4,096 stable
integer RNG mappings. All retain zero catalog clones, zero reconstructed replay
prefixes and deterministic final hashes. The first Windows x64 measurement is
stored in `evidence/standard-universe-runtime-v1/activity/activity-hardening.json`.

This baseline intentionally does not claim catalog-load, complete World 1,
full-run replay or concurrent shared-catalog rows. Those frozen workloads become
executable as P3 and P5 land. Broad CI ceilings are enforced now; strict
stable-runner regression ratios remain owned by `G04-P6-B3`. The baseline also
shows that current state hashing builds canonical scratch bytes (nine allocations
per sample in the recorded fixture), making streaming state hashing an explicit
optimization candidate rather than a hidden performance assumption.

## CI matrix

Goal 04 inherits three native jobs—Windows x64, Linux x64 and macOS ARM64—and
three compile-only alternate CPU targets. Native jobs run
`node tools/goal04/run-native-ci.mjs --foundation` in addition to the existing
repository and Goal 02 gates. The foundation gate grows with Goal 04; the hosted
matrix itself is expanded into full Activity golden suites in `G04-P6-B2`.

Compile-only jobs never claim deterministic execution. Only native profiles may
produce runtime/hash evidence. Action commits, Rust 1.97.0, Node 24.15.0 and the
checksum-bound Sora installer remain inherited exact inputs.

## Dependency and license baseline

Goal 04 begins with the existing 147-package locked graph and introduces no new
runtime dependency in Phase 0. The frozen Cargo.lock and reviewed dependency
policy digests are recorded in `policy/goal04-foundation.json`.

The Universe mode crate may initially use reviewed workspace dependencies such
as Serde/Zstd for private generated Sora transport, SHA-256 for canonical
identity, and the existing numeric/RNG stack through domain APIs. Generated
rows and backend types remain private. Any package/version/feature addition must
be exact-pinned and receive license, deterministic-impact, compile-cost and
alternative review in the introducing batch.

Excel authoring remains `openpyxl==3.1.5`; Sora export remains
`sora-cli==0.3.0`. Neither is linked into the runtime.

## Release scaffold

`policy/goal04-release-contract.json` freezes seven phases, fifty atomic batch
commits and the terminal denominators. During implementation,
`node tools/goal04/verify-release-contract.mjs . --scaffold` verifies that the
plan, ledgers, prerequisite releases and foundation artifacts remain coherent.

The `--release` path is intentionally unavailable until every batch is complete
and the scaffold is promoted in `G04-P6-B4`. It will require complete coverage,
all evidence families, prior release contracts and a clean worktree. A scaffold
pass is not a release claim and never substitutes for terminal evidence.
