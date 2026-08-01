# Goal 20 Status — Swarm Disaster Runtime

## Goal state

| Field | Value |
|---|---|
| Goal ID | `swarm-disaster-runtime-v1` |
| State | `InProgress` |
| Active phase | Phase 2 — Entry, topology, Countdown and Disarray |
| Active batch | None |
| Next unblocked batch | `G20-P2-B1` |
| Snapshot | Version 4.4 / Goal 09 reference release dated 2026-07-29 |
| Profile | `swarm-disaster.profile.v1` |
| Candidate bundle | `385727a8a5875795b29c996102040f7f4419c6adac7b5e10ee6b09c084409362` |
| Normalized pack | `82f3ffc444a1cdcd8bcba5a946bee3a3c8d58527b93a1c9d77f285697401b2d8` |
| Runtime denominator | 6,963 source obligations / 23 rules / 23 fixture families |
| Inherited policy boundaries | 31 inherited / 0 terminal / 31 pending |
| Content lane | Candidate reference input; target `Released` runtime component |
| Branch | `codex/goal20-swarm-disaster-runtime` in the current worktree by user direction |
| Blocking condition | None |

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Contract, audit and execution plan | `Complete` | P0-B1–B4 freeze prerequisites/runtime baseline, exact 6,963/23/23/31 assignments, minimal APIs/state/identity, a 16-run matrix covering 5 difficulties/8 Paths and dice/42 faces/8 Countdown-Disarray cases, 31 policy owners, seven workloads, CI and 13 release gates. |
| Phase 1 — Bundle and catalogs | `Complete` | All 65 tables/33,380 rows lower privately with exact 12/6,716 structural, 24/772 unique-system and 29/25,892 content/rule/coverage closure; component identities remain isolated. |
| Phase 2 — Entry, topology, Countdown and Disarray | `Pending` | — |
| Phase 3 — Audience Dice and Communing Device | `Pending` | — |
| Phase 4 — Progression, content and battle contributions | `Pending` | — |
| Phase 5 — Mechanic partitions | `Pending` | — |
| Phase 6 — Encounters and full-run integration | `Pending` | — |
| Phase 7 — Replay, controllers and external surfaces | `Pending` | — |
| Phase 8 — Hardening and release | `Pending` | — |

## Batch ledger

Only the earliest unblocked row may be `InProgress`. Replace `—` with exact
commands, counts, digests and executable evidence in the completing commit.

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G20-P0-B1` | `Complete` | This batch commit | `node tools/repository-check/verify-release-snapshots.mjs`, `node tools/reference-integration/verify.mjs`, `node tools/goal20/verify-foundation.mjs` and the quick gate pass. `policy/goal20-foundation.json` freezes 10 prerequisites, all 19 registered snapshot checks, the exact Goal 09 artifacts and 6,963/23/23/31 denominator, five immutable roots, nine runtime crate trees, 51 planned batches and current numeric/RNG/replay/adapter revisions. The first full attempt stopped because this shell exposed no `python`; an isolated temporary Python 3.14.6 environment with pinned `openpyxl==3.1.5` made no repository/global dependency change, then full passed in 154.0s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. No runtime source changed. |
| `G20-P0-B2` | `Complete` | This batch commit | `node tools/goal20/generate-dispositions.mjs --check` proves exact-once 6,963/23/23/31 assignment. Source targets are 6,282 `Integrated`, 652 `SharedIntegrated`, six `ExternalOutcome` and 23 `Metadata`; 23 rules split into three `ExactStructured` and 20 `VersionedProjectPolicy` assignments across 12 frozen P5 partitions. All 31 policy rows retain `InheritedPolicy`, exact affected-record digests and implementation owners; zero gaps, duplicates or native handlers. Runtime-disposition SHA-256 `c3e7e82a…dc00`; partition SHA-256 `20e5e240…4b10`. The final isolated-Python full gate passed in 116.2s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P0-B3` | `Complete` | This batch commit | `node tools/goal20/verify-runtime-contract.mjs` freezes four intentional public mode types, four physical/three logical scopes, 16 typed slot families, five existing Activity command kinds, nine transaction event families, 10 ordered components, eight existing RNG labels and seven failure boundaries. Candidate/generated inputs remain private; P0 admits zero native handlers; Standard and Gold replay revisions/bytes remain preserved. The isolated-Python full gate passed in 167.8s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P0-B4` | `Complete` | This batch commit | `node tools/goal20/generate-coverage-matrix.mjs --check` and `node tools/goal20/verify-phase0.mjs` freeze 16 valid three-plane runs covering five difficulties, eight paired Paths/Audience Dice, all 42 faces, eight Countdown/Disarray boundary cases and 31 policy probes. The first real-combat vertical slice, seven performance workloads, zero-clone/allocation structural budgets, three native plus three compile-only CI profiles and 13 release gates close. The isolated-Python Phase 0 full gate passed in 232.6s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P1-B1` | `Complete` | This batch commit | `cargo test -p starclock-test-kit --test universe_suite swarm_disaster_bundle --all-features` and `cargo test -p starclock-mode-universe --lib swarm_disaster_catalog --all-features` prove the exact Goal 09 bundle loads 65 private Sora tables/33,380 rows and preserves the 6,963/23/23/31 manifest denominators. Digest, format, schema fingerprint, table closure, manifest revision and row-denominator rejection families are covered before lowering. Generated rows and summary/error types remain private, no `pub use` is added, and the only new public surface is a generated-type-free validation function returning the existing catalog error. Clippy and `node tools/goal20/verify-phase1-b1.mjs` pass. The quick gate exhausted its 180-second budget while running selected workspace tests; the first full attempt exposed a legacy hard-coded `python` call, then an isolated Python 3.14.6 environment with pinned `openpyxl==3.1.5`, both on `PATH` and in `STARCLOCK_PYTHON`, passed `node tools/repository-check/run.mjs --full` in 244.5s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P1-B2` | `Complete` | This batch commit | The private catalog identity separates Candidate bundle, shared content, profile, Activity registry and composition digests. `swarm_disaster_component_set` returns the existing generic `ConfigurationComponentSet` with exactly 10 canonical components and no new public domain type or `pub use`. The static registry composes core plus one empty Swarm bundle with zero admitted handlers. Two new integration tests and two new unit tests freeze five digest vectors, controller sensitivity and the existing Gold fixture root; source-blob checks prove Standard and Gold handler/component composers remain byte-identical to the `G20-P1-B1` baseline. Clippy and `node tools/goal20/verify-phase1-b2.mjs` pass. The quick gate passed in 147.8s and deferred two generated/release/CI inputs; an isolated Python 3.14.6 environment with pinned `openpyxl==3.1.5` then passed the full gate in 198.6s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P1-B3` | `Complete` | This batch commit | Twelve private structural tables lower into typed immutable definitions: 4 profiles, 8 areas, 20 difficulty segments, 11 planes, 101 chessboards, 1,109 columns, 1,991 nodes, 2,593 edges, 861 rooms, 12 domains, four beacons and two boss choices (6,716 rows total). Validation proves exact denominators and stable IDs, closed profile/difficulty/element/policy values, cross-table references, node exact-once membership, endpoint and adjacent-column closure, one start/end per board and start-to-terminal reachability for all 101 graphs. The derived edge set stays an explicitly labeled static `ProjectPolicy` superset; the `topology_policy` boundary remains `InheritedPolicy` until `G20-P2-B2`. Two structural unit tests, seven aggregate Swarm unit tests and four integration tests pass; identity construction now fails before composition if structural lowering fails. Clippy and `node tools/goal20/verify-phase1-b3.mjs` pass. The quick gate passed in 144.0s and deferred three generated/release/CI inputs; an isolated Python 3.14.6 environment with pinned `openpyxl==3.1.5` then passed the full gate in 258.4s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P1-B4` | `Complete` | This batch commit | Twenty-four private unique-system tables lower 772 rows into typed immutable identities, references, canonical scalar strings and validated embedded programs. Closure covers one Countdown/Disarray policy, 42 boss-decay rows, eight Audience Path/Die pairs, 42 faces/targets, four controls, 21 choices, seven dimensions, 55 adjustments, 63 Trail nodes/effects and 56 prerequisites, 31 cabinets/objectives, 102 finish conditions, 110 unlocks, 13 chapters, six bonuses, eight Paths/boosts, 32 Resonances and 16 Interplays. All cross-table references and exact-once memberships validate; generated types remain private, authoritative floats are absent and all 31 inherited policies remain non-terminal because catalog validation is not execution. Two unique tests, nine aggregate Swarm unit tests and four integration tests pass; identity construction now rejects unique-catalog drift before composition. Clippy, dependency policy and `node tools/goal20/verify-phase1-b4.mjs` pass. The quick gate completed its three selected harnesses in 169.5s but exhausted the 180-second budget during the following `cargo check`; an isolated Python 3.14.6 environment with pinned `openpyxl==3.1.5` then passed the full gate in 270.4s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P1-B5` | `Complete` | This batch commit | Twenty-nine remaining tables lower 25,892 rows into private typed topology-event, content, Curio, Occurrence, service, Adventure, encounter and mechanic-rule definitions plus a validated audit summary. Cross-catalog closure covers 349 events, 1,212 block rules, 13 consequences, 144/288 Blessings and levels, 184 pool members, three 66-row Curio tables, 75/57/308 Occurrence rows, 15 services, six Adventure outcomes, one currency, 19 service rules, 179/347/1,070/15 encounter rows and the exact 23/8,139/6,963/31/5,560/23/609/1/63 rule/evidence closure. Combined with P1-B3/B4 this proves all 65 tables/33,380 rows are accounted exactly once. Generated types remain private, authoritative floats and new `pub use` declarations are absent, and ReferenceOnly programs plus all 31 inherited policies remain non-terminal. Two content tests, 11 aggregate Swarm unit tests and four integration tests pass; identity construction now rejects content or cross-catalog drift before composition. Clippy, dependency policy and `node tools/goal20/verify-phase1-b5.mjs` pass. The quick gate passed in 117.9s and deferred three generated/release/CI inputs; an isolated Python 3.14.6 environment with pinned `openpyxl==3.1.5` then passed the full gate in 216.7s with 33 generated checks, four source-cache-only skips and 34 workspace harnesses. |
| `G20-P2-B1` | `Pending` | — | Entry/profile compilation. |
| `G20-P2-B2` | `Pending` | — | Three-plane graph compilation. |
| `G20-P2-B3` | `Pending` | — | Domain/beacon/topology mutation. |
| `G20-P2-B4` | `Pending` | — | Countdown/Disarray/boss-decay lifecycle. |
| `G20-P2-B5` | `Pending` | — | Plane/boss/final transitions and rollback. |
| `G20-P3-B1` | `Pending` | — | Audience Die definitions/passives. |
| `G20-P3-B2` | `Pending` | — | Roll/reroll/cheat/abandon lifecycle. |
| `G20-P3-B3` | `Pending` | — | All 42 face effects and targets. |
| `G20-P3-B4` | `Pending` | — | Communing choices/dimensions/cabinets. |
| `G20-P3-B5` | `Pending` | — | Simultaneous ordering and fixture parity. |
| `G20-P4-B1` | `Pending` | — | Communing Trail effects. |
| `G20-P4-B2` | `Pending` | — | Pathstrider progress and unlocks. |
| `G20-P4-B3` | `Pending` | — | Bonuses, Propagation and Interplays. |
| `G20-P4-B4` | `Pending` | — | Blessings and Curio lifecycle/pools. |
| `G20-P4-B5` | `Pending` | — | Occurrences, services and Adventure. |
| `G20-P5-M01` | `Pending` | — | Profile-entry rule. |
| `G20-P5-M02` | `Pending` | — | Four topology/event/domain/beacon rules. |
| `G20-P5-M03` | `Pending` | — | Three Countdown/Disarray/boss-decay rules. |
| `G20-P5-M04` | `Pending` | — | Three Audience Die rules. |
| `G20-P5-M05` | `Pending` | — | Two Communing choice/dimension rules. |
| `G20-P5-M06` | `Pending` | — | Two Communing Trail/Pathstrider rules. |
| `G20-P5-M07` | `Pending` | — | Two Path/Propagation/Interplay rules. |
| `G20-P5-M08` | `Pending` | — | Curio lifecycle rule. |
| `G20-P5-M09` | `Pending` | — | Occurrence-choice rule. |
| `G20-P5-M10` | `Pending` | — | Service/Adventure rule. |
| `G20-P5-M11` | `Pending` | — | Two boss/final-boss consequence rules. |
| `G20-P5-M12` | `Pending` | — | Encounter-selection rule. |
| `G20-P5-B1` | `Pending` | — | Execute all 23 semantic fixtures. |
| `G20-P5-B2` | `Pending` | — | Prove exact-once 6,963/23/23 runtime coverage. |
| `G20-P6-B1` | `Pending` | — | Encounter selection and difficulty policies. |
| `G20-P6-B2` | `Pending` | — | Real BattleSpec materialization. |
| `G20-P6-B3` | `Pending` | — | Nested battle execution and settlement. |
| `G20-P6-B4` | `Pending` | — | Complete seeded matrix and fresh replay verification. |
| `G20-P7-B1` | `Pending` | — | Component-addressed replay. |
| `G20-P7-B2` | `Pending` | — | Baseline controller. |
| `G20-P7-B3` | `Pending` | — | CLI mode/config/coverage/replay surfaces. |
| `G20-P7-B4` | `Pending` | — | Agent Activity sessions. |
| `G20-P7-B5` | `Pending` | — | Authorized MCP surfaces. |
| `G20-P8-B1` | `Pending` | — | Determinism, corruption and property hardening. |
| `G20-P8-B2` | `Pending` | — | Performance and allocation gates. |
| `G20-P8-B3` | `Pending` | — | Dependency, architecture, drift and clean-checkout audits. |
| `G20-P8-B4` | `Pending` | — | Release evidence, docs and immutable snapshot registration. |

## Decision ledger

| Date | Decision | Reason |
|---|---|---|
| 2026-08-01 | Create Goal 20 as a runtime goal separate from immutable Goal 09 evidence. | Candidate reference completeness is not executable behavior and historical hashes must remain unchanged. |
| 2026-08-01 | Reuse the Goal 14 phase structure while freezing Swarm-specific mechanics and denominators. | Gold and Gears proved the intended generic Activity, battle, replay and adapter extension path. |
| 2026-08-01 | Execute on `codex/goal20-swarm-disaster-runtime` in the current worktree. | The user explicitly requested no new worktree; the prior public-API change was committed and the checkout was clean before Goal 20 authoring. |
| 2026-08-01 | Treat all 31 Candidate policy boundaries as unresolved until P0 assigns terminal runtime owners. | Candidate policy labels do not authorize a Released execution claim. |

## Policy register

`G20-P0-B2` must generate the exact 31-row policy register from Goal 09. Until
then all 31 rows remain `InheritedPolicy`; no provisional runtime behavior is
terminal.

## Terminal checklist

- [ ] Immutable prerequisites and protected roots pass.
- [ ] All 65 Candidate tables load privately and validate.
- [ ] 6,963/6,963 source obligations have exact-once runtime dispositions.
- [ ] 23/23 mechanic rules have terminal execution evidence/dispositions.
- [ ] All 23 semantic fixture families execute against production values.
- [ ] All 31 inherited policy boundaries are terminal and accurately labeled.
- [ ] Entry, topology, Countdown/Disarray, dice and Communing systems execute.
- [ ] Progression, content, services, Adventure and encounters execute.
- [ ] Real nested battles use current Activity state and verified projections.
- [ ] The seeded matrix completes and freshly verifies every replay.
- [ ] CLI, baseline AI, agent API and MCP offered-command parity passes.
- [ ] Cross-platform determinism and RNG-isolation goldens pass.
- [ ] Performance, dependency, architecture and generated-drift audits pass.
- [ ] Prior-release compatibility and the full clean-checkout gate pass.
- [ ] `G20-P8-B4` is committed and the completion snapshot is registered.

## Completion record

Not yet released.
