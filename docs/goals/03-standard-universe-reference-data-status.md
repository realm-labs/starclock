# Goal 03 Status — Standard Simulated Universe Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `standard-universe-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 0 — Freeze scope and evidence |
| Active batch | None |
| Next unblocked batch | `G03-P0-B3` |
| Snapshot | Version 4.4 / accessed 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope/evidence | `InProgress` | Snapshot/scope frozen; ignored source cache reproducibly includes and hashes all 372 Rogue table/ability files. Concrete row manifests remain B3. |
| Phase 1 — Reference pack | `Pending` | — |
| Phase 2 — Sora schema | `Pending` | — |
| Phase 3 — Excel authoring | `Pending` | — |
| Phase 4 — Review/freeze | `Pending` | — |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G03-P0-B1` | `Complete` | This row's containing commit | Frozen Version 4.4 / 2026-07-22 main-world scope, nine-World/nine-Path public boundary, pinned source revisions, required/excluded categories, evidence/quality policy, normalized pack and Excel/Sora promotion contracts. Added the 28-batch plan, persistent ledger and launch prompt; both prior release contracts and source-policy checks pass. |
| `G03-P0-B2` | `Complete` | This row's containing commit | Expanded the pinned sparse cache with every Rogue Excel table and Rogue battle/level ability file. Deterministic inventory `4c45418e…8972` hashes 372 files: 35 Standard candidates, 17 shared/reachability-review tables, 165 other-mode tables, 17 presentation/account tables and 138 mechanic-evidence files. The generator's `--check` mode rejects drift; classification is table-level only and does not pre-approve shared rows. |
| `G03-P0-B3` | `Pending` | — | — |
| `G03-P0-B4` | `Pending` | — | — |
| `G03-P1-B1` | `Pending` | — | — |
| `G03-P1-B2` | `Pending` | — | — |
| `G03-P1-B3` | `Pending` | — | — |
| `G03-P1-B4` | `Pending` | — | — |
| `G03-P1-B5` | `Pending` | — | — |
| `G03-P1-B6` | `Pending` | — | — |
| `G03-P1-B7` | `Pending` | — | — |
| `G03-P1-B8` | `Pending` | — | — |
| `G03-P1-B9` | `Pending` | — | — |
| `G03-P2-B1` | `Pending` | — | — |
| `G03-P2-B2` | `Pending` | — | — |
| `G03-P2-B3` | `Pending` | — | — |
| `G03-P2-B4` | `Pending` | — | — |
| `G03-P2-B5` | `Pending` | — | — |
| `G03-P3-B1` | `Pending` | — | — |
| `G03-P3-B2` | `Pending` | — | — |
| `G03-P3-B3` | `Pending` | — | — |
| `G03-P3-B4` | `Pending` | — | — |
| `G03-P3-B5` | `Pending` | — | — |
| `G03-P3-B6` | `Pending` | — | — |
| `G03-P4-B1` | `Pending` | — | — |
| `G03-P4-B2` | `Pending` | — | — |
| `G03-P4-B3` | `Pending` | — | — |
| `G03-P4-B4` | `Pending` | — | — |

## Frozen counters

Populate only from generated manifests in `G03-P0-B3`.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Worlds | 9 | 0 | 0 | Public topology fixed; exact IDs pending source inventory. |
| Difficulties/topology | Pending | 0 | 0 | — |
| Paths | 9 | 0 | 0 | Main-world selectable Paths. |
| Resonances/Formations | Pending | 0 | 0 | — |
| Blessings/upgrades | Pending | 0 | 0 | — |
| Curios/states | Pending | 0 | 0 | — |
| Occurrences/choices | Pending | 0 | 0 | — |
| Services/currency rules | Pending | 0 | 0 | — |
| Ability Tree | Pending | 0 | 0 | Battle-affecting only. |
| Encounter pools | Pending | 0 | 0 | — |
| Mechanic fixtures | Pending | 0 | 0 | Distinct shared families. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-22 | Freeze Version 4.4 main-world Standard SU separately from every DLC profile. | Shared Rogue tables cannot be treated as proof of main-world pool membership. |
| 2026-07-22 | Use normalized JSON only for research/bootstrap; production authoring is Excel/Sora. | Preserves the established configuration architecture. |
| 2026-07-22 | Use Python `openpyxl` for workbook authoring/inspection. | Explicit user direction; Sora retains schema/export authority. |
| 2026-07-22 | Include exact or labeled approximate mechanics, not account rewards/story/assets. | Keeps the package implementation-ready and legally bounded. |

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G03-R01` | `Open` | Which base/shared Rogue rows are reachable in main Worlds at the frozen snapshot? | P0-B2/P0-B3 |
| `G03-R02` | `Open` | Which hidden occurrence conditions/outcomes are not proven by structured rows? | P1-B5 |
| `G03-R03` | `Open` | Which Curio effects require battle ability-program inspection or explicit policy? | P1-B4 |
| `G03-R04` | `Open` | Which Ability Tree nodes affect battle/run state versus account rewards only? | P1-B7 |

## Terminal checklist

- [ ] Concrete category manifests and counts are frozen.
- [ ] Complete normalized pack and evidence index regenerate deterministically.
- [ ] All required rows have bilingual summaries and provenance.
- [ ] All required mechanics are exact or explicitly approximate/policy-bound.
- [ ] Universe/Activity/Rule Sora schemas and generated readers validate.
- [ ] `openpyxl` workbooks are complete, styled, checked and visually reviewed.
- [ ] Sora production/debug exports regenerate without drift.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Semantic review fixtures cover every distinct mechanic family.
- [ ] Clean checkout and prior Goal 01/02 release contracts pass.
- [ ] `G03-P4-B4` completion commit is recorded and worktree is clean.
