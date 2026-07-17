# Goal 01 Status — Complete Core Combat and Released Character Content

This file is the persistent execution ledger for
[Goal 01](01-core-combat-and-content.md). The executor must update it in the same
commit as every implementation or content batch.

## Goal state

| Field | Value |
|---|---|
| Goal ID | `core-combat-v1` |
| State | `ReadyToStart` |
| Active phase | Phase 0 — Freeze scope and evidence |
| Next unblocked batch | `G01-P0-B1` |
| Last completed batch | None |
| Last completed commit | None |
| Goal plan baseline | Prepared; execution has not started |
| Content prerequisite | Prepared and awaiting `G01-P0-B1` digest verification |
| Content reference digest | `0dca8ae581b4fa1e9fe8ce0c9e67ac6eb72c251deacbd4831751ce685e45ef5a` |
| Blocking condition | None |

Allowed goal states are `ReadyToStart`, `InProgress`, `Blocked` and `Complete`.
Use `Blocked` only when no independent batch can progress and record the exact
external decision or evidence required. Phase completion alone never changes the
goal to `Complete`.

## Frozen manifest counters

These totals are populated by `G01-P0-B1` from machine-readable manifests. Do
not manually estimate missing totals.

| Required manifest | Digest | Required | DataReady | Disabled announced | Coverage |
|---|---:|---:|---:|---:|---:|
| Released character combat forms | Reference pack bound in `G01-P0-B1` | 88 prepared reference records | 0 | 2 announced outside enabled pack | 0% runtime data |
| Released Light Cones | Reference pack bound in `G01-P0-B1` | 165 prepared reference records | 0 | 0 | 0% runtime data |
| `standard-v1` enemies/variants | Selected in `G01-P0-B1` from 613 templates / 2,591 variants | Pending subset | 0 | 0 | 0% runtime data |
| `standard-v1` encounters | Pending | Pending | 0 | 0 | 0% |
| `standard-v1` scenarios | Pending | Pending | 0 | 0 | 0% |

Disabled announced entries are recorded for audit but are never included in a
required-coverage denominator until public release and a new manifest revision.

## Phase ledger

| Phase | State | Exit evidence |
|---|---|---|
| Phase 0 — Freeze scope and evidence | `Pending` | None |
| Phase 1 — Workspace and reproducible data foundation | `Pending` | None |
| Phase 2 — Deterministic primitives | `Pending` | None |
| Phase 3 — Executable combat vertical slice | `Pending` | None |
| Phase 4 — Complete shared combat kernel | `Pending` | None |
| Phase 5 — Build compiler, Traces, Eidolons and Light Cones | `Pending` | None |
| Phase 6 — Standard orchestration, AI, CLI and replay | `Pending` | None |
| Phase 7 — Complete released content import | `Pending` | None |
| Phase 8 — Hardening and documentation freeze | `Pending` | None |

Allowed batch/phase states are `Pending`, `InProgress`, `Researching`, `Blocked`,
`Complete` and `NotApplicable`. `NotApplicable` requires a decision-log entry and
may not be used for a required acceptance gate.

## Batch ledger

Add one row per concrete batch. Expand the Phase 7 partition families after the
Phase 0 manifests are frozen.

| Batch | State | Commit | Validation evidence | Notes |
|---|---|---|---|---|
| `G01-P0-B1` | `Pending` | — | — | Freeze goal manifests. |
| `G01-P0-B2` | `Pending` | — | — | Provenance staging and evidence hashes. |
| `G01-P0-B3` | `Pending` | — | — | Blocking research cases, including Elation and V1a probes. |
| `G01-P0-B4` | `Pending` | — | — | Initial generated coverage. |
| `G01-P1-B1` through `G01-P1-B11` | `Pending` | — | — | Workspace, CI, Sora capability proof, schema families and reproducible pipeline. Expand before Phase 1 starts. |
| `G01-P2-B1` through `G01-P2-B6` | `Pending` | — | — | Deterministic primitives, replay contract and initial property harness. Expand before Phase 2 starts. |
| `G01-P3-B1` through `G01-P3-B8` | `Pending` | — | — | Synthetic vertical slice, performance baseline and command properties. Expand before Phase 3 starts. |
| `G01-P4-B1` through `G01-P4-B11` | `Pending` | — | — | Shared kernel interleaved with Excel/Sora V1a mechanism probes. Expand before Phase 4 starts. |
| `G01-P5-B1` through `G01-P5-B6` | `Pending` | — | — | Build, Trace, Eidolon and Light Cone compiler. Expand before Phase 5 starts. |
| `G01-P6-B1` through `G01-P6-B6` | `Pending` | — | — | Standard Activity, controllers, replay payloads, CLI and scenarios. Expand before Phase 6 starts. |
| `G01-P7-V1B` | `Pending` | — | — | Promote representative probes into complete production content. |
| `G01-P7-Cnn` | `Pending` | — | — | Expand from stable character manifest partitions, at most 8 forms each. |
| `G01-P7-Lnn` | `Pending` | — | — | Expand from stable Light Cone partitions, at most 16 cones each. |
| `G01-P7-Mnn` | `Pending` | — | — | Register known mechanic batches during Phase 0; add newly discovered prerequisites before dependent content. |
| `G01-P7-R1` | `Pending` | — | — | Clean catalog and coverage regeneration. |
| `G01-P7-R2` | `Pending` | — | — | Manifest-wide build compilation. |
| `G01-P8-B1` through `G01-P8-B7` | `Pending` | — | — | Audits, established CI matrix, fuzz expansion, performance gate and freeze. Expand before Phase 8 starts. |

For a completed row, validation evidence must include commands and a link to a
committed report or fixture when applicable. A commit hash alone is insufficient.

## Required content partitions

### Character partitions

Pending `G01-P0-B1`. Record stable manifest IDs, not display names alone, and
replace the `Cnn` family placeholder with every concrete ledger row in the same
batch. Each partition must contain no more than 8 combat forms and must be
complete through E6 before its commit is accepted.

### Light Cone partitions

Pending `G01-P0-B1`. Record stable manifest IDs and replace the `Lnn` family
placeholder with every concrete ledger row in the same batch. Each partition
must contain no more than 16 Light Cones and must be complete through S5 before
acceptance.

### Standard battle partitions

Pending `G01-P0-B1`. The manifest must include all archetypes required by Goal
01 section 4.4 and map each entry to at least one seeded golden scenario.

## Research and blockers

| ID | State | Question or blocker | Evidence required | Owner/batch |
|---|---|---|---|---|
| `G01-R-SABER-ARCHER-SOURCE` | `Pending` | Verify the pinned 4.3 fallback and 4.4 manifest identity mapping for Saber and Archer without weakening released-content provenance. | Source revisions/hashes, identity mapping and discrepancy report. | `G01-P0-B2` |
| `G01-R-ELATION-SEMANTICS` | `Pending` | Define shared Elation damage, Elation Skill, Punchline, Certified Banger, forced-action and shared-actor/resource semantics from more than one released form. | Cross-kit evidence, decision record and probe fixture specification. | `G01-P0-B3` |
| `G01-R-V1A-PROBES` | `Pending` | Identify every unresolved timing/ownership question needed by the Asta, Kafka, Clara, Firefly and Aglaea mechanism probes. | Named per-mechanic cases with reproducible observation or golden fixture specifications. | `G01-P0-B3` |

An unresolved research case may not be converted into a default implementation
without a documented project-policy decision and regression fixture.

## Decision log

| Date | Decision | Reason |
|---|---|---|
| 2026-07-17 | Goal 01 targets complete core battle plus released character forms, Traces, Techniques, Eidolons and Light Cones. | Establish the first independently playable milestone. |
| 2026-07-17 | Universe families and all three recurring challenge families are excluded. | Prevent activity-specific systems from delaying the core battle milestone. |
| 2026-07-17 | Full relic/planar and public enemy catalogs are excluded; future boundaries remain protected. | They are not part of the requested first content batch. Standard battle uses a frozen representative public-data manifest. |
| 2026-07-17 | Content completeness is manifest-based and requires `DataReady`; behavioral profiles and placeholders do not count. | Make completion auditable and prevent scope inflation. |
| 2026-07-17 | Every batch is committed separately and updates this ledger. | Preserve reviewability, resumption and deterministic progress tracking. |
| 2026-07-17 | Goal 01 binds the prepared Version 4.4 content-reference pack before implementation. | Prevent compact profiles, memory, or ad-hoc websites from becoming the Excel source of truth. |
| 2026-07-17 | Excel workbooks plus pinned Sora output remain the only authoritative production authoring/runtime-data chain; prepared JSON is bootstrap evidence, not a runtime shortcut. | Preserve the formal editable and validated configuration workflow selected for Starclock. |
| 2026-07-17 | Phase 4 interleaves shared-kernel batches with non-production V1a probes compiled from a dedicated Excel/Sora scope. | Make complex released mechanics constrain the Rule IR and lifecycle before bulk import without misreporting partial content as DataReady. |
| 2026-07-17 | Cross-platform CI, property-test scaffolding and performance measurement begin before hardening. | Make Phase 8 consume accumulated evidence instead of creating its prerequisites at the end. |

## Terminal acceptance checklist

Change an item to `[x]` only with evidence in this file.

- [ ] Required workspace crates compile with enforced dependency direction.
- [ ] Pinned dependencies/tools have purpose, license, deterministic-impact and
      compile-cost records; the Sora 0.3.0 golden project proves every relied-on
      command, schema and export capability before production schemas.
- [ ] Fixed-point, RNG, canonical codec and state hashing pass cross-platform
      golden vectors.
- [ ] Core formula, timeline, effect, Toughness, lifecycle and rule suites pass.
- [ ] Asta, Kafka, Clara, Firefly, Aglaea and cross-kit Elation V1a probes pass
      through the production Excel/Sora-to-domain boundary without entering
      production coverage.
- [ ] Standard single-wave, multi-wave, elite and multi-phase boss scenarios run
      from build selection to terminal battle result.
- [ ] CLI configuration validation, coverage, battle run and replay verification
      pass from a clean checkout.
- [ ] Released character combat-form manifest is 100% `DataReady`, including
      abilities, Techniques, Traces and E1-E6.
- [ ] Released Light Cone manifest is 100% `DataReady`, including levels,
      promotions and S1-S5.
- [ ] `standard-v1` enemy, encounter and scenario manifests are 100% `DataReady`.
- [ ] All required bilingual fields, provenance and evidence hashes validate.
- [ ] Sora/Excel export and generated outputs reproduce without drift.
- [ ] Manifest-wide E0/S1 and E6/S5 build compilation passes.
- [ ] Baseline controller decisions and replay hashes are deterministic.
- [ ] Committed Windows/Linux/macOS CI workflows distinguish native execution
      from compile-only CPU coverage and retain golden evidence.
- [ ] Versioned performance workloads satisfy the reviewed stable-runner budgets
      for latency, throughput, allocations, state-copy cost and journal growth.
- [ ] Formatting, clippy, workspace tests, source-size and public-API audits pass.
- [ ] No excluded universe, challenge, UI, account or full relic/enemy dataset is
      claimed as part of Goal 01.
- [ ] Clean-checkout acceptance report is committed and linked here.

## Completion record

| Field | Value |
|---|---|
| Final state | Not complete |
| Completion commit | — |
| Catalog digest | — |
| Clean-checkout report | — |
| Cross-platform report | — |
| Remaining required work | All execution phases |
