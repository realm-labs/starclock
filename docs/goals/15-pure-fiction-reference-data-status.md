# Goal 15 Status — Pure Fiction Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `pure-fiction-reference-v1` |
| State | `Ready` |
| Active phase | Not started |
| Active batch | — |
| Next unblocked batch | `G15-P0-B1` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 / planning audit 2026-07-30 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | Both caches clean/detached at pinned commits; origins, required commit readability and connectivity verified; execution must reproduce in `G15-P0-B1` |
| Starting source oracle | Dedicated `ChallengeStory*`/schedule/theme/target tables; shared entry, MazeBuff, BattleEvent, StageConfig, monster and `FantasticStory*`/level-program closure; CHS/EN TextMaps |
| Active-season hypothesis | Schedule `202024` → group `2024` (`借虚成真` / `Falsehood to Fact`) → rows `20241`–`20244`, Tierce `20245`, nine StageConfig candidates, Grit/Fever buffs and three selectable Cacophonies; not a denominator until `G15-P0-B3` |
| Focused inventory | Pending `G15-P0-B2` |
| Content manifest | Denominators pending `G15-P0-B3` |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal15-pure-fiction-reference` |
| Branch base | `0191cc71b1735d6e101e6e04817181423c599232` (`master`) |
| Parallel inspection | Main workspace and Goal 09–14 worktrees were clean; Goal 09–13 and the separate Goal 14 Memory of Chaos setup branches matched their remote refs; current `master` independently advances Goal 14 Gold and Gears runtime |
| Parallel condition | Separate branch/worktree and six isolated Goal 15 roots while other Goals or integration work are active |
| Publication policy | Push and remotely verify each completed batch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources, manifest and contracts | `Pending` | Awaiting execution-owned cache reproduction, focused inventory, active-release/Tierce/Starward manifest and authoring contracts. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting profile/season flow, participants/loadouts, attempts, clocks, spawn/refill, scoring, objectives, Whimsicality, Grit/Fever and Cacophony semantics. |
| Phase 2 — Content pools, services, events and enemies | `Pending` | Awaiting nonzero/zero pool proofs, themes/MazeBuffs/BattleEvents/configs, exact StageConfig waves, enemies, AI and abilities. |
| Phase 3 — Independent Sora and Excel | `Pending` | Awaiting isolated schemas/readers, three complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Ownership audit, fixtures, reconciliation and freeze | `Pending` | Awaiting exact-once audit, semantic fixtures, cross-goal receipts, regeneration and clean-checkout evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G15-P0-B1` | `Pending` | — | Reproduce caches, verify Goal 03 and concurrent boundaries, freeze released Version 4.4 scope and prove isolation. |
| `G15-P0-B2` | `Pending` | — | Inventory dedicated/adjacent tables, entry mappings, `FantasticStory*` and shared config/ability programs, TextMaps, StageConfig, enemies and exclusions. |
| `G15-P0-B3` | `Pending` | — | Freeze active selectors, exact obligations/counts, Tierce/Starward semantics, ownership, reachability, exact-zero pools and scheduled/unreleased exclusions. |
| `G15-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G15-P1-B1` | `Pending` | — | Import profile, active season, entry/unlocks, stages/nodes, Tierce/Starward identity, legal order and outcomes. |
| `G15-P1-B2` | `Pending` | — | Import participants, team/loadout uniqueness, snapshots/locks, attempts, retries, abandonment, reset and transitions. |
| `G15-P1-B3` | `Pending` | — | Import clocks, wave boundaries, continuous spawn/refill programs, timeout and early completion. |
| `G15-P1-B4` | `Pending` | — | Import defeat/damage scoring, attribution, caps, simultaneous outcomes, objectives, stars and aggregation. |
| `G15-P1-B5` | `Pending` | — | Import Whimsicality and Grit/Fever gain, states, thresholds, effects, target policies, transitions and teardown. |
| `G15-P1-B6` | `Pending` | — | Import selectable Cacophony choices, eligibility, timing, parameters, base-rule interactions and battle contributions. |
| `G15-P1-B7` | `Pending` | — | Import initial resources, battle entry, cross-battle projections and remaining Tierce/Starward contributions. |
| `G15-P2-B1` | `Pending` | — | Freeze selector-backed nonzero or exact-zero Blessing, Curio, Occurrence and event-choice pools. |
| `G15-P2-B2` | `Pending` | — | Freeze selector-backed nonzero or exact-zero service, currency, shop and other content pools. |
| `G15-P2-B3` | `Pending` | — | Import challenge definitions, themes, MazeBuffs, BattleEvents, stage templates and config/ability relationships. |
| `G15-P2-B4` | `Pending` | — | Import exact StageConfig encounters, waves, spawn slots, variants, levels and difficulty bindings. |
| `G15-P2-B5` | `Pending` | — | Import enemy skills/statuses/AI/abilities, summons, linked actors, boss phases and rule contributions. |
| `G15-P2-B6` | `Pending` | — | Generate mechanics, sources, coverage, research gaps, fixtures and pack index. |
| `G15-P3-B1` | `Pending` | — | Add profile/season/stage/node/Tierce/participant/attempt Sora tables. |
| `G15-P3-B2` | `Pending` | — | Add clock/spawn/score/objective/star/Whimsicality/Grit/Cacophony/resource Sora tables. |
| `G15-P3-B3` | `Pending` | — | Add pool, event, MazeBuff, encounter, wave, enemy and mechanic-binding Sora tables. |
| `G15-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated locks/templates/readers. |
| `G15-P3-B5` | `Pending` | — | Generate and structurally/semantically verify all three complete `openpyxl` workbooks. |
| `G15-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet and schema column. |
| `G15-P4-B1` | `Pending` | — | Audit exact-once coverage, active-release selection, ownership, references, provenance, bilingual fields and exclusions. |
| `G15-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G15-P4-B3` | `Pending` | — | Reconcile shared overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G15-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but remotely unverified
batch remains `InProgress`.

## Goal package publication

| Field | Value |
|---|---|
| Setup batch | `G15-SETUP` |
| Setup commit | This document's containing commit (`G15-SETUP`) |
| Remote | `origin` |
| Branch | `codex/goal15-pure-fiction-reference` |
| Push command | `git push -u origin codex/goal15-pure-fiction-reference` |
| Verification command | `test "$(git rev-parse HEAD)" = "$(git ls-remote --heads origin refs/heads/codex/goal15-pure-fiction-reference \| awk '{print $1}')"` |
| Result | Successful: the remote full commit ID equals this document's containing commit; the setup handoff reports the resolved ID and `G15-P0-B1` freezes it as foundation evidence |

The setup commit uses “this document's containing commit” to avoid a recursive
self-hash. If the push or remote-resolution check fails, this package remains
unpublished and `G15-P0-B1` must not begin.

## Frozen counters

Populate required counts only from the generated manifest in `G15-P0-B3`.
Do not estimate denominators from raw table sizes, prefixes, ID ranges,
schedule adjacency or display names. A zero denominator also requires a
generated selector-closure proof.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profile/season/entry/terminal outcomes | TBD | 0 | 0 | Stable family plus the released season active in Version 4.4; historical and scheduled/unreleased seasons remain evidence-only. |
| Stages/nodes/Tierce/Starward/transitions | TBD | 0 | 0 | Freeze ordinary topology and selected extensions without inferring team or node count from names. |
| Participant/team/loadout/attempt records | TBD | 0 | 0 | Includes uniqueness, snapshots, locks, substitution, retry, abandonment and reset scope. |
| Turn/AV clocks and wave lifecycle | TBD | 0 | 0 | Includes node scope, tick, wave boundary, refill order, timeout and early completion. |
| Spawn queues/replacement rules | TBD | 0 | 0 | Includes ordered candidates, simultaneous defeats, slot reuse, final group and empty-candidate behavior. |
| Initial resources/battle entry | TBD | 0 | 0 | Includes HP, Energy, Skill Points and selected entry operations. |
| Score sources/caps/objectives/stars | TBD | 0 | 0 | Includes defeat/damage attribution, per-node cap, aggregation and settlement timing. |
| Whimsicality/Grit/Fever | TBD | 0 | 0 | Includes triggers, per-target caps, thresholds, effects, target policies and teardown. |
| Cacophony choices/effects | TBD | 0 | 0 | Includes every offered choice, eligibility, selected scope and base-rule interaction. |
| Blessings/Curios/Occurrences/choices | TBD | 0 | 0 | Freeze exact reachable rows or a generated exact-zero proof per family. |
| Services/currencies/shops | TBD | 0 | 0 | Reward/account tables do not prove a mechanically reachable service or currency. |
| Themes/MazeBuffs/BattleEvents/configs | TBD | 0 | 0 | Includes every selected shared or mode-owned program and contribution. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Resolve every ordinary and Tierce/Starward StageConfig row and spawn formation. |
| Enemy skills/statuses/AI/abilities | TBD | 0 | 0 | Include complete transitive mechanic closure for every enabled variant. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every unique lifecycle, spawn, score, seasonal, choice and encounter policy. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-30 | Create Goal 15 as a complete Pure Fiction reference-data package, not a runtime goal. | Version 4.4 research can proceed independently without changing challenge, Activity or combat runtime. |
| 2026-07-30 | Use `pure-fiction` as the stable mode slug and reserve Goal 15. | Current `master` already uses Goal 14 for Gold and Gears runtime, while a clean published parallel branch also uses Goal 14 for Memory of Chaos reference setup; no local or remote Goal 15 package, branch or Pure Fiction root existed during the audit. |
| 2026-07-30 | Base the setup branch on committed `master@0191cc71b1735d6e101e6e04817181423c599232`. | The main workspace was clean and this captures the current Goal 14 runtime progress without modifying its files. |
| 2026-07-30 | Inherit the pinned Version 4.4 source and identity revisions used by Goals 03 and 08–14. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-30 | Require `G15-P0-B1` to reproduce both caches even though the planning audit found them clean, detached and connected. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-30 | Treat schedule `202024`, group `2024`, rows `20241`–`20244`, Tierce `20245`, nine StageConfig candidates and the listed buff/program chain only as planning seeds. | Explicit references make them strong candidates, but the generated manifest must prove exact active-release membership, ownership and exclusions. |
| 2026-07-30 | Treat `Falsehood to Fact` as the released season active in the Version 4.4 snapshot while excluding group `2025` at the fixed access boundary. | Official release notes place `Falsehood to Fact` through 2026-08-03 and Version 4.4 from 2026-07-15; group `2025` starts after the 2026-07-30 access date and schedule presence alone is not release evidence. |
| 2026-07-30 | Keep Tierce, Starward and Fever topology/lifecycle semantics open until released table/config evidence proves them. | Obfuscated Tierce fields and labels cannot establish team count, node count, clocks or settlement behavior. |
| 2026-07-30 | Treat the dedicated `ChallengeStory*` tables and `FantasticStory*` programs as inventory seeds, not ownership or completeness oracles. | Historical Pure Fiction, rewards, presentation and adjacent challenge families coexist with shared challenge data and programs. |
| 2026-07-30 | Audit Blessing, Curio, Occurrence, service, currency, shop and choice families even when the expected result is empty. | Completeness requires generated zero proofs; absence cannot be inferred from the challenge-mode label. |
| 2026-07-30 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent and completed goals must preserve isolated ledgers and surface conflicts for merge coordination. |
| 2026-07-30 | Exclude presentation, calendar behavior, quick-clear/account state and rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-30 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full challenge runs require a later goal. |
| 2026-07-30 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G15-R01` | `Open` | Which dedicated/shared table, entry mapping, TextMap, StageConfig, `FantasticStory*`/shared config, enemy, ability and AI files complete the focused inventory? | P0-B2 | Replace when the generated inventory closes every enabled selector/reference and byte-identical double generation passes. |
| `G15-R02` | `Open` | Which released selectors prove group `2024` is the season active in Version 4.4 and exclude historical plus scheduled-but-unreleased group `2025`? | P0-B3 | Replace with an exact-once manifest whose rows carry structured and public release evidence and fail-closed exclusions. |
| `G15-R03` | `Open` | What exact topology, participant slots, clock, objective and settlement semantics do Tierce `20245`, Starward and Fever add? | P0-B3 / P1-B1–B7 | Replace with decoded schema/reference joins and fixtures; record missing runtime capabilities for a later goal without changing runtime here. |
| `G15-R04` | `Open` | What are the exact character/combat-form, Light Cone and Relic-instance uniqueness and loadout invalidation scopes across every ordinary and Tierce/Starward node? | P1-B2 | Replace with source-backed participant/lock rows and accepted/rejected/retry fixtures. |
| `G15-R05` | `Open` | How do turn/AV budgets, node scope, wave transitions, continuous enemy replacement, timeout and early final-group completion compose? | P1-B3 | Replace with config/ability evidence or field-level policies carrying rejected alternatives and lifecycle fixtures. |
| `G15-R06` | `Open` | How are defeat and damage points attributed, capped and aggregated under simultaneous defeats, overkill, summons, environmental damage, timeout and early completion? | P1-B4 | Replace with typed score operations and fixtures for every source, cap and settlement boundary. |
| `G15-R07` | `Open` | What are the exact Whimsicality and Grit/Fever gain filters, per-enemy trigger cap, thresholds, effect parameters, transition order, target selection and teardown? | P1-B5 / P2-B3 | Replace with MazeBuff/config/ability evidence and one fixture per distinct state transition and target boundary. |
| `G15-R08` | `Open` | When and per what scope are the three Cacophonies offered, locked, applied and removed, and how do they interact with the base seasonal mechanic? | P1-B6 / P2-B3 | Replace with exact group-extra/MazeBuff/program joins and choice/lifecycle fixtures. |
| `G15-R09` | `Open` | Which config lists and challenge constants define initial resources, battle entry, retry restoration and cross-node carry/reset? | P1-B7 / P2-B3 | Replace with explicit selector-to-operation joins and initial/retry fixtures; unsupported mappings fail closed. |
| `G15-R10` | `Open` | Are any Blessings, Curios, Occurrences, event choices, services, currencies, shops or analogous pools mechanically reachable? | P2-B1–B2 | Replace each family with a selector-backed nonzero closure or generated exact-zero proof. |
| `G15-R11` | `Open` | Which StageConfig waves, concrete variants, levels, skills, statuses, AI, summons, phases and abilities define every ordinary and Tierce/Starward encounter? | P2-B4–B5 | Replace with complete encounter/enemy dossiers or an explicit nonblocking boundary for unavailable released evidence. |
| `G15-R12` | `Open` | Which hidden ordering, timing, random selection, caps, rounding and fallback fields remain unavailable after bounded research? | P2-B6 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete stronger-evidence trigger. |

## Terminal checklist

- [ ] Exact active-season category manifests and denominators are frozen.
- [ ] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual names, independent summaries and
      row-level provenance.
- [ ] Ownership, active-release enablement and shared reachability are explicit
      and fail closed.
- [ ] Tierce, Starward and Fever topology, participant, clock, objective and
      settlement semantics are proved or explicitly policy-bounded.
- [ ] Empty content-pool families have generated selector-closure proofs.
- [ ] Shared classifications reconcile with committed overlapping Goal facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Participant/loadout locks, attempts, clocks, spawn/refill, scores,
      objectives/stars, Whimsicality, Grit/Fever and Cacophony have complete
      semantic fixtures.
- [ ] Encounter identities, StageConfig rows, waves, variants, AI, abilities
      and boss bindings resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] All three complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and all other mode/production bundle identities remain
      unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from its recorded remote branch
      at the recorded full commit ID.
- [ ] Clean-checkout acceptance passes and `G15-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Pure Fiction reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G15-P0-B3` |
| Release evidence | — |
| Remaining required work | Pure Fiction runtime lowering, integration, handlers, controller/API exposure and seeded full challenge runs belong to a later goal. |
