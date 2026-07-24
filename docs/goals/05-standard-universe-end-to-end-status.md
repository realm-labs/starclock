# Goal 05 Status — Standard Universe End-to-End Integration

## Current state

`InProgress`

Goal 05 starts from the immutable Goal 04 completion snapshot. It closes the
gap between independently executable mechanic plans/reference battle
settlement and production end-to-end Activity plus combat execution.

## Batch ledger

| Batch | State | Evidence | Result |
|---|---|---|---|
| `G05-P0-B1` | `Complete` | This row's containing commit | Froze the execution package, six observed Goal 04 integration gaps, an additive migration contract and 19 commit-sized batches. Goal 04 remains an immutable historical snapshot; Goal 05 explicitly requires real nested combat and atomic effect application without expanding into general ForkJoin. |
| `G05-P0-B2` | `Complete` | `node tools/goal05/verify-integration-probes.mjs`; `node tools/repository-check/run.mjs` | Froze the observable starting debt: two production reference-settlement callers, nine standalone Path entry points, two effect-plan-only noncombat entry points, seven physical nodes per domain, 37 acyclic templates/579 source nodes and the 4,058-node/5,993-edge compiled graph. The focused loop is capped at 180 seconds; every later batch must replace debt assertions with positive integration tests. |
| `G05-P1-B1` | `Complete` | `cargo test -p starclock-activity --test handler_registry`; `cargo test -p starclock-mode-universe --test handler_bundle`; focused Clippy; quick repository gate | Added bounded immutable Activity handler bundles, validated stable metadata/schema/dependencies, deterministic bundle/handler ordering, canonical SHA-256 registry identity and function-pointer execution returning ordinary Activity operations. The empty core and Standard Universe bundles compose without editing a central registry and reserve later P2 handler registrations. |
| `G05-P1-B2` | `Complete` | `cargo test -p starclock-activity --lib --tests --all-features`; `cargo test -p starclock-mode-universe --lib --tests --all-features`; focused Clippy; quick repository gate | Added bounded logical-scope definitions and transactional runtime state. The compiled Standard Universe topology binds each domain's seven physical room nodes to one logical `DomainVisit`: movement inside the domain preserves the visit identity, leaving closes it and later re-entry allocates a fresh deterministic visit sequence. Canonical state, hashes and filtered/debug views now cover active logical scopes; activities without logical scopes retain their prior encoding. |
| `G05-P1-B3` | `Pending` | — | Component identity/replay scaffold. |
| `G05-P2-B1` | `Pending` | — | Bound atomic external interactions. |
| `G05-P2-B2` | `Pending` | — | Occurrence effects. |
| `G05-P2-B3` | `Pending` | — | Services and currency. |
| `G05-P2-B4` | `Pending` | — | Curio and Ability Tree effects. |
| `G05-P3-B1` | `Pending` | — | Universe combat contributions. |
| `G05-P3-B2` | `Pending` | — | Encounter/wave BattleSpec materialization. |
| `G05-P3-B3` | `Pending` | — | Production nested Battle executor. |
| `G05-P3-B4` | `Pending` | — | End-to-end mechanic battle fixtures. |
| `G05-P4-B1` | `Pending` | — | Component-aware real-battle replay. |
| `G05-P4-B2` | `Pending` | — | CLI migration. |
| `G05-P4-B3` | `Pending` | — | Agent/MCP migration. |
| `G05-P5-B1` | `Pending` | — | Real-battle seeded matrix/coverage. |
| `G05-P5-B2` | `Pending` | — | Determinism/performance hardening. |
| `G05-P5-B3` | `Pending` | — | Release freeze. |

## Frozen findings

| ID | Finding | Required closure |
|---|---|---|
| `G05-F01` | Production CLI/agent full runs use `verified-reference-projection-v1`. | Execute actual `starclock-combat` battles and seal their projections. |
| `G05-F02` | Path/Blessing/Curio evaluators return proposals outside the combat resolver. | Compile proposals into validated combat contributions/operations. |
| `G05-F03` | Occurrence/service effect plans are not atomically applied by `submit_external_outcome`. | Bind handlers and apply costs/effects/transition in one Activity transaction. |
| `G05-F04` | Seven physical nodes model one domain while local state is Activity-scoped. | Add `DomainVisit` logical ownership and fresh revisit semantics. |
| `G05-F05` | Mode dispatch is represented by central runtime fields and family-specific methods. | Compose immutable handler/rule bundles. |
| `G05-F06` | Replay identity is coupled to manually assembled whole-catalog revisions. | Bind ordered consumed components and registry digests. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-24 | Goal 04 remains a historical release and its evidence is not regenerated. | Goal 05 is an additive semantic/runtime revision, not retroactive relabeling. |
| 2026-07-24 | Do not implement general multi-pending tasks in Goal 05. | Standard Universe is sequential; this goal closes actual observed integration gaps. |
| 2026-07-24 | Reference battle projection is test-only after migration. | Production completion must prove real combat behavior. |
| 2026-07-24 | Retain documented approximations where exact public data is absent. | Integration must not fabricate authoritative values. |

## Terminal record

| Field | Value |
|---|---|
| Final state | In progress |
| Completion commit | — |
| Activity/replay revision | To be frozen by P1/P4 |
| Integration coverage | To be generated |
| Performance evidence | To be generated |
| Release evidence | To be generated |
