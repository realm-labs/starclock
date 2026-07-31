# Goal 19 Status — Fate/Star Rail Night Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `fate-star-rail-night-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 0 — Scope, sources and contracts |
| Active batch | — |
| Next unblocked batch | `G19-P0-B2` |
| Snapshot | Version 4.4 / released 2026-07-24 / access 2026-08-01 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Source oracle | `FateRin*`, `Config/Gameplays/Fate`, FateRin enemy/config closure, StageConfig and CHS/EN TextMaps |
| Content lane | `Experimental`; target Candidate reference bundle |
| Branch | `codex/goal19-fate-star-rail-night-reference` |
| Base | `origin/master@92febad080dd4cf9997718d64b3648fc198ab1f8` |
| Remote | `origin` |
| Blocking condition | None |

## Goal package setup

The dedicated worktree was created from
`origin/master@92febad080dd4cf9997718d64b3648fc198ab1f8`, branch
`codex/goal19-fate-star-rail-night-reference` was pushed with upstream
tracking, and the local and remote refs were equal before this package was
authored. `git diff --check` passes. The Node 24.15.0 quick gate completed the
runner, extension, dependency, workspace-boundary and source-policy checks,
then exhausted its 180-second budget while waiting for `cargo fmt` in the new
worktree; this is recorded as a setup-time cache/tool contention result rather
than a passed gate. `G19-P0-B1` owns the clean rerun and exact command record.

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `InProgress` | B1 reproduced both pinned caches twice, froze the 89-file discovery seed and proved branch/path/runtime isolation. |
| Phase 1 — Unique activity systems | `Pending` | Awaiting graph, participants, Mystic Codes, resources, progression and fights. |
| Phase 2 — Encounters and complete pack | `Pending` | Awaiting pools, configs, encounters, enemy closure, rules and fixtures. |
| Phase 3 — Excel and Sora | `Pending` | Awaiting isolated schemas/readers, four workbooks, deterministic exports and visual QA. |
| Phase 4 — Audit and freeze | `Pending` | Awaiting ownership audit, semantic execution, reconciliation and Candidate release. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G19-P0-B1` | `Complete` | This row's containing commit | Reproduced clean detached caches twice; froze 25 dedicated FateRin tables plus 64 Fate configuration seeds at receipt digest `e75abcf…0cad`; verified base, upstream, source trees, ownership roots, named Currency Wars/RtBattle exclusions and no runtime lowering. `freeze-foundation.mjs --check` and `git diff --check` pass. Push and full remote equality are recorded by the containing commit publication before B2 starts. |
| `G19-P0-B2` | `Pending` | — | Generate focused table/config/TextMap/Stage/enemy inventory and exclusions. |
| `G19-P0-B3` | `Pending` | — | Freeze selector-backed manifests, denominators, ownership and exact-zero pools. |
| `G19-P0-B4` | `Pending` | — | Freeze normalized, evidence, workbook, reconciliation and fixture contracts. |
| `G19-P1-B1` | `Pending` | — | Import profile, graph, Case Boards, unlocks and outcomes. |
| `G19-P1-B2` | `Pending` | — | Import participants, Masters/Servants, teams and loadout policies. |
| `G19-P1-B3` | `Pending` | — | Import Treasures, Mystic Code catalog, tags, decks and acquisition. |
| `G19-P1-B4` | `Pending` | — | Import Mystic Code costs, targets, effects, upgrades and lifecycle. |
| `G19-P1-B5` | `Pending` | — | Import energy, Command Spells/Reiju, choices and resource transitions. |
| `G19-P1-B6` | `Pending` | — | Import owners, traits/affixes, progression and carry/reset. |
| `G19-P1-B7` | `Pending` | — | Import story/map/Infinite Trial flow, objectives, retry and settlement. |
| `G19-P2-B1` | `Pending` | — | Freeze reachable or exact-zero generic content/service/shop/currency pools. |
| `G19-P2-B2` | `Pending` | — | Import fight/buff/BattleEvent/MazeBuff/config relationships. |
| `G19-P2-B3` | `Pending` | — | Import exact StageConfig encounters, waves, slots, variants and difficulty. |
| `G19-P2-B4` | `Pending` | — | Import enemy/AI/ability/phase and event-specific participant closure. |
| `G19-P2-B5` | `Pending` | — | Generate rules, sources, coverage, gaps, reconciliation, fixtures and index. |
| `G19-P3-B1` | `Pending` | — | Add profile/graph/participant/progression Sora tables. |
| `G19-P3-B2` | `Pending` | — | Add Mystic Code/deck/resource/Command Spell/trait Sora tables. |
| `G19-P3-B3` | `Pending` | — | Add fight/buff/encounter/wave/enemy/mechanic-binding Sora tables. |
| `G19-P3-B4` | `Pending` | — | Add review tables, locks, templates and isolated readers. |
| `G19-P3-B5` | `Pending` | — | Generate and verify four complete openpyxl workbooks. |
| `G19-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review. |
| `G19-P4-B1` | `Pending` | — | Audit exact-once coverage, selectors, ownership, provenance and exclusions. |
| `G19-P4-B2` | `Pending` | — | Execute semantic fixtures and replacement checks. |
| `G19-P4-B3` | `Pending` | — | Reconcile overlaps and run full regeneration/clean-checkout acceptance. |
| `G19-P4-B4` | `Pending` | — | Freeze Candidate release evidence and completion snapshot. |

## Frozen counters

All denominators remain `TBD` until generated by `G19-P0-B3`. Never infer
counts from prefixes, raw table size, names or ID adjacency. Zero requires a
selector-closure proof.

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G19-R01` | `Open` | Which FateRin/shared tables, config programs, stages and enemy files close the focused inventory? | P0-B2 |
| `G19-R02` | `Open` | Which selectors prove released/permanent Fate/Star Rail Night membership and exclude Currency Wars/RtBattle/unrelated rows? | P0-B3 |
| `G19-R03` | `Open` | What exact graph, Case Board, day/progress and unlock lifecycle is mechanical? | P1-B1 |
| `G19-R04` | `Open` | What are exact Master/Servant/team/trial/loadout uniqueness and invalidation scopes? | P1-B2 |
| `G19-R05` | `Open` | What are every Mystic Code candidate, acquisition, cost, target, effect, repeat, upgrade and teardown rule? | P1-B3–B4 |
| `G19-R06` | `Open` | How do magical energy and Command Spell/Reiju choices, rerolls and resources settle? | P1-B5 |
| `G19-R07` | `Open` | How do progression, traits/affixes, owner/init loadouts and state carry compose? | P1-B6 |
| `G19-R08` | `Open` | Which story/map/Infinite Trial fights, affixes, objectives and retry/settlement rules are enabled? | P1-B7/P2-B2 |
| `G19-R09` | `Open` | Which StageConfig waves, enemies, AI, skills, statuses and abilities define every fight? | P2-B3–B4 |
| `G19-R10` | `Open` | Which hidden ordering, timing, weights, caps, rounding and fallbacks remain unavailable? | P2-B5/P4-B2 |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote verification | — |
| Reference bundle | — |
| Coverage | Pending G19-P0-B3 |
| Runtime profile | Unreleased |
