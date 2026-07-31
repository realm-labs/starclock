# Goal 18 Status — Apocalyptic Shadow Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `apocalyptic-shadow-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Mode systems |
| Active batch | — |
| Next unblocked batch | `G18-P1-B2` |
| Snapshot | Version 4.4 / access 2026-08-01 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Active selector seed | `ScheduleDataChallengeBoss:203019` → group `3019`; ordinary rows `30191`–`30194`, selected Tierce `30195`; denominator pending Phase 0 |
| Content lane | `Experimental`; target `CandidateReferenceData` |
| Runtime state | `Unreleased`; zero runtime changes permitted |
| Branch | `codex/goal18-apocalyptic-shadow-reference` |
| Branch base | `92febad080dd4cf9997718d64b3648fc198ab1f8` |
| Remote | `origin` |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Foundation and frozen denominator | `Complete` | Snapshots/isolation, 59-file inventory, 129-obligation manifest, six exact-zero proofs and normalized/evidence/authoring/fixture contracts frozen. |
| Phase 1 — Mode systems | `InProgress` | Profile/period and all five stages/nine nodes imported; lifecycle, scoring and unique mechanics continue. |
| Phase 2 — Encounters and closure | `Pending` | Awaiting pools, exact encounters and enemy/program closure. |
| Phase 3 — Excel/Sora authoring | `Pending` | Awaiting isolated schemas, workbooks, readers and bundle. |
| Phase 4 — Candidate freeze | `Pending` | Awaiting audits, fixtures, regeneration and release evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G18-P0-B1` | `Complete` | This row's containing commit | `foundation.json`: Goal 03 complete; both pinned revisions readable and clean/detached; branch base `92febad0`; six isolated roots and zero runtime/peer mutation policy frozen. |
| `G18-P0-B2` | `Complete` | This row's containing commit | Deterministic inventory freezes 59 source files: 10 dedicated, 18 StrongChallenge mechanic/program, 10 shared-closure and 21 adjacent-exclusion files, each with byte length and SHA-256. |
| `G18-P0-B3` | `Complete` | This row's containing commit | Frozen 129 exact-once obligations (48 mode-owned, 81 shared): 1 family, 1 period, 2 group rows, 5 stages, 9 nodes, 6 targets, 11 buffs, 10 enemy variants, 4 templates, 67 skills, 7 programs and 6 generated exact-zero pool proofs. |
| `G18-P0-B4` | `Complete` | This row's containing commit | Frozen 35 normalized files, canonical encoding, provenance/approximation fields, three-workbook clean-generation contract and 16 semantic fixture families (39 minimum cases). |
| `G18-P1-B1` | `Complete` | This row's containing commit | Imported profile, active period `203019`/Vanguard Knight, four ordinary two-node stages, selected Tierce `30195`, nine exact node selectors and entry/unlock locators; all runtime-disabled. |
| `G18-P1-B2` | `Pending` | — | Participants, loadouts, attempts and transitions. |
| `G18-P1-B3` | `Pending` | — | AV clocks, boss progress, scores, objectives and stars. |
| `G18-P1-B4` | `Pending` | — | Safeguard, Axiom, Embers, buffs and contributions. |
| `G18-P2-B1` | `Pending` | — | Pool audits and selected challenge/config relationships. |
| `G18-P2-B2` | `Pending` | — | Encounters, waves, slots, variants and level bindings. |
| `G18-P2-B3` | `Pending` | — | Enemy/program closure and generated review metadata. |
| `G18-P3-B1` | `Pending` | — | Sora schemas/readers/exports and three generated workbooks. |
| `G18-P3-B2` | `Pending` | — | Sora/workbook drift and rendered visual verification. |
| `G18-P4-B1` | `Pending` | — | Exact-once ownership/reconciliation and semantic fixtures. |
| `G18-P4-B2` | `Pending` | — | Deterministic regeneration, repository and clean-checkout gates. |
| `G18-P4-B3` | `Pending` | — | Immutable Candidate release snapshot. |

## Frozen counters

Counters are frozen by `G18-P0-B3`; no planning-time estimate is a denominator.

| Family | Manifest | Normalized | DataReady | Note |
|---|---:|---:|---:|---|
| Total | 129 | Pending | Pending | Active schedule `203019`, group `3019`, stages `30191`–`30195`; 48 mode-owned and 81 shared. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-08-01 | Keep Goal 18 reference-only. | User requested another high-priority collection wave before runtime work. |
| 2026-08-01 | Treat group `3019`, not scheduled group `3020`, as the active released 4.4 selector. | Pinned schedule interval contains the audit/access date; later rows remain exclusion evidence. |
| 2026-08-01 | Reconcile shared rows by receipt only. | Goals 13, 15 and 17 run independently and own their isolated artifacts. |

## Verification log

| Date | Batch | Commands | Result |
|---|---|---|---|
| 2026-08-01 | Planning | `git status --short --branch`; pinned source revision checks; direct inspection of `ScheduleDataChallengeBoss`, `ChallengeBossGroupConfig`, `ChallengeBossMazeConfig`, `ChallengeBossGroupExtra`, `ChallengeBossMazeExtra`, `ChallengeBossMazeTierce` and targets | Branch clean at `92febad0`; source revisions matched; active selector seed recorded. |
| 2026-08-01 | `G18-P0-B1` | `git -C <cache> rev-parse HEAD`; `git -C <cache> status --short --branch`; Goal 03 ledger inspection; `git diff --name-only 92febad0..HEAD` | Passed; prerequisites and isolation recorded in `evidence/apocalyptic-shadow-reference-v1/foundation.json`. |
| 2026-08-01 | `G18-P0-B2` | `node tools/apocalyptic-shadow-reference/inventory.mjs`; same with `--check` | 59 files frozen and deterministic: dedicated 10, mechanic-program 18, shared-closure 10, adjacent-exclusion 21. |
| 2026-08-01 | `G18-P0-B3` | `node tools/apocalyptic-shadow-reference/manifest.mjs`; `node tools/apocalyptic-shadow-reference/audit-pools.mjs` | 129 obligations frozen; six selected-row pool scans each concluded `ExactZero`. |
| 2026-08-01 | `G18-P0-B4` | JSON contract parse/schema review; `git diff --check` | 35-file normalized schema, evidence labels, Sora/openpyxl ownership and 16 fixture families frozen. |
| 2026-08-01 | `G18-P1-B1` | `node --check tools/apocalyptic-shadow-reference/build-pack.mjs`; `node tools/apocalyptic-shadow-reference/build-pack.mjs --batch=G18-P1-B1` | Generated 16 DataReady rows across profile, period, stages and nodes; Tierce selector remains explicit. |
