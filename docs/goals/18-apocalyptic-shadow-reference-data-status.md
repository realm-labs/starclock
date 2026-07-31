# Goal 18 Status — Apocalyptic Shadow Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `apocalyptic-shadow-reference-v1` |
| State | `Ready` |
| Active phase | Not started |
| Active batch | — |
| Next unblocked batch | `G18-P0-B1` |
| Snapshot | Version 4.4 / access 2026-08-01 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Active selector seed | `ScheduleDataChallengeBoss:203019` → group `3019`; ordinary rows `30191`–`30194`, selected Tierce `30195`; denominator pending Phase 0 |
| Content lane | `Experimental`; target `CandidateReferenceData` |
| Runtime state | `Unreleased`; zero runtime changes permitted |
| Branch | `codex/goal18-apocalyptic-shadow-reference` |
| Branch base | Pending `G18-P0-B1` |
| Remote | `origin` |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Foundation and frozen denominator | `Pending` | Awaiting cache reproduction, inventory, manifest and contracts. |
| Phase 1 — Mode systems | `Pending` | Awaiting lifecycle, clocks/progress/score and unique mechanics. |
| Phase 2 — Encounters and closure | `Pending` | Awaiting pools, exact encounters and enemy/program closure. |
| Phase 3 — Excel/Sora authoring | `Pending` | Awaiting isolated schemas, workbooks, readers and bundle. |
| Phase 4 — Candidate freeze | `Pending` | Awaiting audits, fixtures, regeneration and release evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G18-P0-B1` | `Pending` | — | Reproduce snapshots/prerequisites, branch base and isolated-scope proof. |
| `G18-P0-B2` | `Pending` | — | Focused source inventory and exclusions. |
| `G18-P0-B3` | `Pending` | — | Frozen active selector, manifest, ownership and exact-zero proofs. |
| `G18-P0-B4` | `Pending` | — | Frozen normalization/evidence/workbook/reconciliation/fixture contracts. |
| `G18-P1-B1` | `Pending` | — | Profile, period, stages/nodes/Tierce, entry and outcomes. |
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
| Total | Pending | Pending | Pending | Exact-once denominator not frozen. |

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
