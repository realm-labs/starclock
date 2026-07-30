# Goal 14 Status — Gold and Gears Runtime

## Goal state

| Field | Value |
|---|---|
| Goal ID | `gold-and-gears-runtime-v1` |
| State | `InProgress` |
| Active phase | Phase 3 — Custom Dice and Knowledge |
| Active batch | None |
| Next unblocked batch | `G14-P3-B1` |
| Snapshot | Version 4.4 / Goal 08 reference release dated 2026-07-29 |
| Profile | `gold-gears.profile.v1` |
| Candidate bundle | `97eefe25954b16df3b96c713101ed28bf28806d0bdff0d8925b0734a756bfe7b` |
| Normalized pack | `ea2f3a35807b9a7dae39be2d67fb5de955bfad7852718eb1d3393affed5a5623` |
| Runtime denominator | 7,913 source obligations / 1,224 rules / 18 fixture families |
| Inherited policy boundaries | 16; runtime dispositions pending |
| Content lane | Candidate reference input; target `Released` runtime component |
| Blocking condition | None |

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Contract, audit and execution plan | `Complete` | `G14-P0-B1`–`B4`: prerequisites, 7,913/1,224/18 assignments, APIs/state/identity, 25-run matrix, 16 policy owners, workloads, CI and release scaffold verified. |
| Phase 1 — Bundle and catalogs | `Complete` | `G14-P1-B1`–`B5`: exact private bundle loading, component-aware identity, 29,140-row immutable lowering, shared Standard identity binding, full cross-catalog closure and 7,913/7,913 catalog coverage verified. |
| Phase 2 — Entry, topology and Cognition | `Complete` | `G14-P2-B1`–`B5`: caller-explicit entry, 17 typed slots, canonical three-plane graph, typed map overlays, 13 Cognition ranges, 20 Secret frontiers, six explicit boss choices and atomic plane/final transitions verified. |
| Phase 3 — Custom Dice and Knowledge | `Pending` | None |
| Phase 4 — Progression, content and battle contributions | `Pending` | None |
| Phase 5 — Mechanic partitions | `Pending` | None |
| Phase 6 — Encounters and full-run integration | `Pending` | None |
| Phase 7 — Replay, controllers and external surfaces | `Pending` | None |
| Phase 8 — Hardening and release | `Pending` | None |

## Batch ledger

Only the earliest unblocked row may be `InProgress`. Replace `None` with exact
commands, counts, digests and executable evidence in the completing commit.

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G14-P0-B1` | `Complete` | This batch commit | `node tools/repository-check/verify-release-snapshots.mjs`; `node tools/reference-integration/verify.mjs`; `node tools/goal14/verify-foundation.mjs`; `node tools/repository-check/run.mjs`; 8 snapshots, 46,110 merged records, 15/15 mode pairs, 0 conflicts, 5 protected roots and 9 baseline crate trees verified; quick gate passed with Rust scope skipped. |
| `G14-P0-B2` | `Complete` | This batch commit | `node tools/goal14/generate-dispositions.mjs --check`; 7,913 source obligations, 1,224 rules and 18 fixture families assigned exact-once; nine P5 partitions frozen at 5/6/6/40/160/384/38/495/90 rules; 0 gaps, 0 duplicates, 0 native handlers admitted. |
| `G14-P0-B3` | `Complete` | This batch commit | `node tools/goal14/verify-runtime-contract.mjs`; 6 public mode types, 4 physical + 3 logical scopes, 17 typed slot families, 5 generic commands, 9 Activity event families, 10 components, 8 RNG labels and 7 failure policies frozen; 0 native handlers admitted. |
| `G14-P0-B4` | `Complete` | This batch commit | `node tools/goal14/generate-coverage-matrix.mjs --check`; `node tools/goal14/verify-phase0.mjs`; quick gate passed; `node tools/repository-check/run.mjs --full` passed in 282.7s with 136 workspace test harnesses and 4 source-cache-only checks skipped; 25 valid runs cover 5 difficulties, 9 Paths, 12 dice, both Conundrum tracks 0–6 and 6+6; 16 policy probes, 7 workloads, 3 native + 3 compile-only profiles and 13 release gates frozen. |
| `G14-P1-B1` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --test gold_gears_bundle --all-features`; `cargo test -p starclock-mode-universe --lib gold_gears_catalog --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase1-b1.mjs`; quick gate passed; exact Candidate digest, schema fingerprint, 52/52 tables and 29,140 rows validated through a private generated reader; 2 integration + 3 unit tests cover six stable rejection families; 0 generated public types. |
| `G14-P1-B2` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --test gold_gears_identity --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase1-b2.mjs`; quick gate passed; 10 canonically ordered components compose Gold content, shared content, core catalogs, registries, overlay and caller controller; 2 immutable Activity bundles and 0 admitted handlers; 4 digest goldens; Standard handler/component composer blobs remain unchanged from Goal start. |
| `G14-P1-B3` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --lib gold_gears_structural --all-features`; `cargo test -p starclock-mode-universe --test gold_gears_identity --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase1-b3.mjs`; quick gate passed; 12 private tables / 8,621 rows lowered to typed immutable definitions; 115 static graph supersets validate 1,313 columns, 2,502 exact-once nodes, 3,407 next-column edges, start-to-terminal reachability and 12-domain closure; `G14-R02` remains accurately `InheritedPolicy`. |
| `G14-P1-B4` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --lib gold_gears_unique --all-features`; `cargo test -p starclock-mode-universe --test gold_gears_identity --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase1-b4.mjs`; quick gate passed; 18 private tables / 462 rows lowered with closed identities for 13 Cognition ranges, 20 Secrets, 12 dice, 80 faces, 22 Knowledge rules, 40 Neural nodes, 12 Conundrum levels, 9 Paths, 36 Resonances, 36 Extrapolations and 18 Interplays; canonical decimal strings remain float-free; inherited execution policies remain non-terminal. |
| `G14-P1-B5` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --lib gold_gears_content --all-features`; `cargo test -p starclock-mode-universe --test gold_gears_identity --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase1-b5.mjs`; quick gate passed in 158.8s; full gate passed in 352.9s with 138 workspace test harnesses; 21 private tables / 20,056 rows lower shared and Gold content, 12,806 JSON payloads, 1,224 owner/fixture rule links and 90 enemy identities; 67 enemy definitions resolve through released core/Standard catalogs and 23 are explicitly owned by P6 materialization; 42 categories publish 7,913/7,913 catalog coverage without claiming runtime execution. |
| `G14-P2-B1` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --lib gold_gears_entry --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase2-b1.mjs`; quick gate passed in 96.4s with 53 selected harnesses and 3 downstream packages checked; full gate passed in 362.3s with 138 workspace harnesses after deferred policy/runner inputs; 5 formal difficulties × 9 Paths × 12 Custom Dice compile 540 explicit entries, six ordered faces validate, 40 Neural nodes canonicalize with prerequisite closure, Difficulty 5 prior-clear evidence gates both 0–6 Conundrum tracks and 6+6, and 17 final typed slot families compile; `G14-R01` is terminal as `VersionedExecutablePolicy`. |
| `G14-P2-B2` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --lib gold_gears_entry --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase2-b2.mjs`; quick gate passed in 172.1s with 53 selected harnesses and 3 downstream packages; full gate passed in 357.4s with 138 workspace harnesses after 2 deferred generated/release inputs; all five formal entries compile the authored `2021`/`2022`/`2023` plane order into root boards `2112021`/`2112022`/`2112023`, 81 once-visit nodes, 120 nearest-column routes, 2 plane transitions, 1 reachable terminal and 81 three-level logical-scope bindings; graph digest `a62dce4db977515ad3f156c654a263e8bea16e9b0b3e6608309813b283187c3b`; `G14-R02` remains accurately non-terminal for P2-B3 overlay mutation. |
| `G14-P2-B3` | `Complete` | This batch commit | `cargo test -p starclock-mode-universe --lib gold_gears_content --all-features`; `cargo test -p starclock-mode-universe --lib gold_gears_entry --all-features`; `cargo clippy -p starclock-mode-universe --all-targets --all-features -- -D warnings`; `node tools/goal14/verify-phase2-b3.mjs`; the first cold-cache quick attempt exhausted the 180s budget during selected-test dispatch, then the completed build passed quick in 107.6s with 53 selected harnesses and 3 downstream packages; full gate passed in 458.9s with 138 workspace harnesses; 332 map events close 221 cell/111 row triggers and six effect families, 1,091 block rules close typed count/domain/beacon candidates across 115 boards, seeded root creation writes all 27 node overlays through ordinary Activity operations, replace/copy/blank commit atomically without changing the graph digest, blanked targets are removed from canonical routes, event operations precede block creation, empty candidates consume no draw and only the Graph stream advances; `G14-R02` is terminal as `VersionedExecutablePolicy`. |
| `G14-P2-B4` | `Complete` | This batch commit | `cargo test -p starclock-activity --all-features`; `cargo test -p starclock-mode-universe --lib gold_gears_entry --all-features`; focused Clippy for both affected crates; `node tools/goal14/verify-phase2-b4.mjs`; the first cold-cache quick attempt exhausted the 180s budget after building 67 selected harnesses, then the completed build passed quick in 98.8s with 2 direct and 7 downstream packages; the deferred-input full gate passed in 427.6s with 138 workspace harnesses; all 13 inclusive Cognition ranges, 20 Secrets, 22 constants, zero-reset, exact carry, global/area clamp, three plane-boss frontiers, predecessor gating and `(minimum, maximum, source Secret ID)` tie order execute through ordinary Activity operations with zero RNG draws; `G14-R03` is terminal as `VersionedExecutablePolicy`. |
| `G14-P2-B5` | `Complete` | This batch commit | `cargo test -p starclock-activity --all-features`; `cargo test -p starclock-mode-universe --lib gold_gears_entry --all-features`; focused Clippy for both affected crates; `node tools/goal14/verify-phase2-b5.mjs`; the first cold-cache quick attempt exhausted the 180s budget after all 67 selected harnesses passed, then the completed build passed quick in 97.9s with 2 direct and 7 downstream packages; the Phase 2 checkpoint full gate passed in 218.8s with 138 workspace harnesses; 81 authored nodes plus one post-boss terminal form an 82-node/123-edge graph with digest `4f07183a4a53189208a402a6ae69a3dbe491f678252d8a1ba04c9ba5000bca48`; six caller-explicit boss choices, typed Section/Node resets, Cognition/Secret carry, atomic completion, rejected-state byte identity and RNG transaction isolation are verified. |
| `G14-P3-B1` | `Pending` | None | Implement dice slots, loadouts and upgrades. |
| `G14-P3-B2` | `Pending` | None | Implement Custom Dice roll/reroll/cheat/passives. |
| `G14-P3-B3` | `Pending` | None | Implement all dice faces and target policies. |
| `G14-P3-B4` | `Pending` | None | Implement Knowledge lifecycle. |
| `G14-P3-B5` | `Pending` | None | Prove simultaneous ordering and fixture parity. |
| `G14-P4-B1` | `Pending` | None | Implement 40 Neural Network nodes. |
| `G14-P4-B2` | `Pending` | None | Implement both Conundrum tracks and Berserk. |
| `G14-P4-B3` | `Pending` | None | Implement bonuses, Path boosts and Resonance additions. |
| `G14-P4-B4` | `Pending` | None | Link shared content and implement Curio copies/lifecycle. |
| `G14-P4-B5` | `Pending` | None | Implement Occurrences, services and Adventure outcomes. |
| `G14-P5-M01` | `Pending` | None | Execute 5 profile-entry rules. |
| `G14-P5-M02` | `Pending` | None | Execute 6 Stats Conundrum rules. |
| `G14-P5-M03` | `Pending` | None | Execute 6 Auxiliary Conundrum rules. |
| `G14-P5-M04` | `Pending` | None | Execute 40 Neural Network rules. |
| `G14-P5-M05` | `Pending` | None | Execute 160 Curio lifecycle rules. |
| `G14-P5-M06` | `Pending` | None | Execute 384 Occurrence-choice rules. |
| `G14-P5-M07` | `Pending` | None | Execute 38 service/Adventure rules. |
| `G14-P5-M08` | `Pending` | None | Execute 495 Path-boost rules. |
| `G14-P5-M09` | `Pending` | None | Execute 90 Resonance Extrapolation rules. |
| `G14-P5-B1` | `Pending` | None | Execute all 18 production semantic fixture families. |
| `G14-P5-B2` | `Pending` | None | Prove exact-once runtime completeness. |
| `G14-P6-B1` | `Pending` | None | Implement encounter and difficulty selection. |
| `G14-P6-B2` | `Pending` | None | Materialize current-state real BattleSpecs. |
| `G14-P6-B3` | `Pending` | None | Execute and settle real nested battles. |
| `G14-P6-B4` | `Pending` | None | Complete the frozen seeded matrix. |
| `G14-P7-B1` | `Pending` | None | Complete component-addressed replay. |
| `G14-P7-B2` | `Pending` | None | Complete deterministic baseline controller behavior. |
| `G14-P7-B3` | `Pending` | None | Add CLI run/coverage/replay surfaces. |
| `G14-P7-B4` | `Pending` | None | Add agent Activity support. |
| `G14-P7-B5` | `Pending` | None | Add MCP Activity support. |
| `G14-P8-B1` | `Pending` | None | Complete determinism and malformed-input hardening. |
| `G14-P8-B2` | `Pending` | None | Enforce performance/allocation budgets. |
| `G14-P8-B3` | `Pending` | None | Complete release audits and clean-checkout verification. |
| `G14-P8-B4` | `Pending` | None | Freeze release evidence and completion snapshot. |

## Frozen starting denominators

P0 may add derived runtime counters but may not change these Goal 08 counts
without a documented data revision and compatibility decision.

| Dimension | Frozen input | Required terminal state |
|---|---:|---|
| Source obligations | 7,913 | 7,913 exact-once runtime dispositions |
| Ownership | 7,199 Gold / 714 Shared | Every shared row resolves a released stable identity |
| Normalized files | 51 | Reference-only; zero runtime reads |
| Sora tables / workbook rows | 52 / 29,140 | 52 privately loaded; workbook rows remain authoring-only |
| Mechanic rules | 1,224 | 1,224 terminal executable/non-executable dispositions |
| Semantic fixture families | 18 | 18 production-executed |
| Policy boundaries | 16 | 16 terminal, none silently exact or legally unresolved |
| Chessboards / columns / nodes | 115 / 1,313 / 2,502 | Validated and runtime-reachable as assigned |
| Derived map edges / events / block rules | 3,407 / 332 / 1,091 | Versioned policy or exact runtime execution |
| Rooms | 1,224 | Runtime-disposed with bounded graph ownership |
| Cognition ranges / Secrets / constants | 13 / 20 / 22 | Executable or validated metadata as assigned |
| Custom Dice / categories / Path bindings | 12 / 4 / 108 | Every valid selection constructs and executes |
| Dice slots / faces / tags / Knowledge bindings | 6 / 80 / 10 / 22 | Complete loadout and mechanic execution |
| Neural Network nodes | 40 | Prerequisites, costs and effects executable |
| Conundrum definitions | 12 | Six Stats and six Auxiliary levels executable |
| Paths / Resonances / boosts / Extrapolations / Interplays | 9 / 36 / 9 / 36 / 18 | Shared identity plus Gold contributions executable |
| Blessings / levels | 162 / 324 | Shared-integrated without duplicated semantics |
| Curios / mode states | 80 / 80 | Reachability, copy state and lifecycle executable |
| Occurrences / variants / choices | 62 / 65 / 257 | Atomic executable choice graphs |
| Services / Adventure outcomes | 15 / 8 | Atomic service programs and offered outcomes |
| Encounter groups / waves / enemy slots | 181 / 478 / 1,513 | Every reachable binding materializes or fails validation |
| Referenced enemy variants | 90 | Resolved to exact combat definitions or terminally blocked |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-30 | Create Goal 14 as a runtime goal separate from immutable Goal 08 reference evidence. | Reference completeness is not executable behavior, and historical evidence must remain unchanged. |
| 2026-07-30 | Extend `starclock-mode-universe` instead of adding a Gold and Gears state-machine crate. | The mode is a profile over the released generic Activity/Battle boundaries. |
| 2026-07-30 | Keep the Goal 08 bundle as an independently identified consumed component. | Component-aware identity preserves Standard and unrelated mode replay compatibility. |
| 2026-07-30 | Generate runtime dispositions before implementing mechanic partitions. | The 7,913/1,224 denominator must not be hidden, hand-counted or reduced during execution. |
| 2026-07-30 | Treat every inherited policy as unresolved for runtime until P0 assigns a terminal owner. | Candidate reference policies do not automatically authorize a Released execution claim. |
| 2026-07-30 | Reuse released shared content by stable identity and digest. | Shared source rows do not justify duplicated semantics or mode-owned copies. |
| 2026-07-30 | Freeze a valid coverage matrix in P0 rather than claiming an arbitrary Cartesian product. | Difficulty, Conundrum and unlock constraints must be respected while every required axis and interaction is covered. |
| 2026-07-30 | Give shared rows `SharedIntegrated` precedence except the eight Adventure abstractions, which are `ExternalOutcome`; fixture-manifest rows remain `Metadata` until production execution. | This preserves released shared semantics, keeps external physics outside the runtime and prevents fixture metadata from being counted as implementation. |
| 2026-07-30 | Represent the mutable chessboard as a validated immutable graph superset plus bounded typed state overlays. | Creation, replacement, copying, blanking, domain, beacon and Knowledge changes can commit through ordinary Activity operations without introducing a second graph aggregate. |
| 2026-07-30 | Keep the existing eight Activity RNG labels; use `Spawn` only for a Custom Dice resolution and Knowledge causally owned by that resolution. | This isolates dice work from graph, encounter, reward, shop, occurrence and battle streams without changing released Standard Activity RNG state. |
| 2026-07-30 | Give Gold and Gears a distinct 10-component set and replay entry while leaving the released Standard component set and replay bytes unchanged. | Compatibility is based on consumed components, not on a whole multi-mode bundle or central mode registry. |
| 2026-07-30 | Freeze 25 valid seeded complete runs instead of a Cartesian matrix. | Twelve baseline runs cover all difficulty/Path/dice axes; twelve single-track and one combined-cap Difficulty 5 runs cover legal Conundrum boundaries with explicit prior-clear evidence. |
| 2026-07-30 | Bind every inherited policy to one matrix probe and one or more exact owner batches. | A policy cannot disappear between reference evidence and runtime release, and pending ownership cannot be mistaken for a terminal disposition. |
| 2026-07-30 | Validate the 3,407 derived topology edges as a static graph superset while retaining `G14-R02` as `InheritedPolicy`. | Catalog closure proves safe bounded inputs; runtime generation and mutation parity are owned by P2-B2/P2-B3. |
| 2026-07-30 | Lower canonical numeric authoring values as validated decimal strings and keep embedded programs private until their typed executor batches. | Catalog construction must reject malformed numerics without introducing floating arithmetic or confusing JSON transport validation with executable semantics. |
| 2026-07-30 | Bind 67 Gold encounter enemy identities to released core/Standard definitions and retain the remaining 23 exact v4.4 identities as an explicit P6 materialization obligation. | P1 must close the reference catalog without falsely claiming that a stable released identity already has an executable combat definition. |
| 2026-07-30 | Resolve `G14-R01` with caller-explicit, fail-closed entry policy revision `gold-and-gears-entry-policy-v1`. | No Path, Custom Dice, face loadout or Trailblaze Bonus is silently selected; formal difficulty is derived from the selected formal area, Neural input is prerequisite-closed, and Conundrum requires explicit prior-clear evidence. |
| 2026-07-30 | Compile formal topology under `gold-and-gears-topology-policy-v1`: select root board `211{plane-source}`, preserve authored area plane order and derived forward-nearest-column edges, then connect consecutive root end/start nodes. | This produces one bounded immutable Activity graph without random entry draws; `G14-R02` remains non-terminal until P2-B3 implements typed overlay mutation and route filtering. |
| 2026-07-30 | Resolve `G14-R02` as `gold-and-gears-topology-policy-v1` with canonical integer-weighted Graph-stream event/count/beacon candidates, event-before-creation order and overlay-only create/replace/copy/blank mutations. | Released rows provide exact candidates and weights but not verified engine enumeration; stable authored order, no-draw empty candidates and immutable graph overlays make the retained ProjectPolicy executable and replaceable. |
| 2026-07-30 | Resolve `G14-R03` as `gold-and-gears-cognition-policy-v1`: checked delta, global then selected-area clamp, Activity-scope exact carry, new-run zero reset, post-plane-boss Secret evaluation and canonical `(minimum, maximum, source Secret ID)` tie order. | Released numeric ranges, Secret graph and boss boundary remain exact; the unreleased lifecycle ordering stays visibly `ProjectPolicy`, executable and replaceable rather than being relabeled as observed parity. |
| 2026-07-30 | Add one synthetic post-boss terminal and resolve plane completion as `gold-and-gears-plane-completion-policy-v1`: caller-explicit released boss candidates, same-layer validation, Cognition/Secret evaluation, completion marker, traversal and final settlement commit atomically. | The 81 authored nodes are all rooms rather than terminals; the synthetic node gives the final boss an explicit completion edge while preserving prior batch evidence as immutable history. Real encounter eligibility and `BattleSpec` materialization remain owned by P6. |

## Research and policy register

`InheritedPolicy` means the Goal 08 policy is retained accurately but has not
yet earned a Goal 14 runtime disposition.

| ID | State | Runtime question | Owner |
|---|---|---|---|
| `G14-R01` | `VersionedExecutablePolicy` | `gold-and-gears-entry-policy-v1`: explicit selections, fixed initial resource policy, zero entry draws and fail-closed validation. | P2-B1 |
| `G14-R02` | `VersionedExecutablePolicy` | `gold-and-gears-topology-policy-v1`: forward-nearest-column edges, root-board mapping, canonical Graph-stream weighted creation and overlay-only mutation. | P2-B2/P2-B3 |
| `G14-R03` | `VersionedExecutablePolicy` | `gold-and-gears-cognition-policy-v1`: checked adjustment, global/area clamp, exact carry, zero reset and deterministic post-boss Secret frontier/tie order. | P2-B4 |
| `G14-R04` | `InheritedPolicy` | Dice numeric filter-tag to mechanical-code mapping. | P3-B1/P3-B3 |
| `G14-R05` | `InheritedPolicy` | Dice-face candidate, priority, duration and empty-target resolution. | P3-B3 |
| `G14-R06` | `InheritedPolicy` | Knowledge target selection. | P3-B4 |
| `G14-R07` | `InheritedPolicy` | Simultaneous movement, Knowledge, collapse and reward order. | P3-B5 |
| `G14-R08` | `InheritedPolicy` | Neural reroll with no alternate candidate. | P4-B1 |
| `G14-R09` | `InheritedPolicy` | Neural slot-upgrade target among equal-rarity slots. | P4-B1 |
| `G14-R10` | `InheritedPolicy` | Unreleased Conundrum combat numerics and Berserk values. | P4-B2 |
| `G14-R11` | `InheritedPolicy` | Resonance Extrapolation selection, scheduling and polarity. | P4-B3 |
| `G14-R12` | `InheritedPolicy` | Curio offer-specific eligibility and ordering. | P4-B4/P5-M05 |
| `G14-R13` | `InheritedPolicy` | Hidden random Occurrence outcome weights/order. | P4-B5/P5-M06 |
| `G14-R14` | `InheritedPolicy` | Adventure Fragment/reward selection. | P4-B5/P5-M07 |
| `G14-R15` | `InheritedPolicy` | Static room/domain/encounter-group selection. | P6-B1 |
| `G14-R16` | `InheritedPolicy` | Effective encounter difficulty by area and plane. | P6-B1 |

## Terminal checklist

- [ ] Goals 01–08 immutable prerequisites and merged Candidate audit pass.
- [ ] All 52 Goal 08 Sora tables load privately and validate.
- [ ] 7,913/7,913 source obligations have exact-once runtime dispositions.
- [ ] 1,224/1,224 mechanic rules have terminal execution evidence/dispositions.
- [ ] All 18 semantic fixture families execute against production values.
- [ ] All 16 inherited policy boundaries are terminal and accurately labeled.
- [ ] Entry, topology, Cognition, dice, Knowledge, Neural and Conundrum execute.
- [ ] Content pools, services, Adventure outcomes and encounters execute.
- [ ] Real nested battles use current Activity state and verified projections.
- [ ] The frozen seeded matrix completes and freshly verifies its replays.
- [ ] CLI, baseline AI, agent API and MCP offered-command parity passes.
- [ ] Cross-platform determinism and RNG-isolation goldens pass.
- [ ] Performance, dependency, architecture, security and generated-drift
      audits pass.
- [ ] Goals 01–08 current compatibility and immutable snapshots pass.
- [ ] The full clean-checkout release gate passes.
- [ ] `G14-P8-B4` is committed and the completion snapshot is registered.
