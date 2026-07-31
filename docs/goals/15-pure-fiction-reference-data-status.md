# Goal 15 Status — Pure Fiction Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `pure-fiction-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Unique mode systems |
| Active batch | — |
| Next unblocked batch | `G15-P1-B3` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 / planning audit 2026-07-30 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | Both caches clean/detached at pinned commits; origins, required commit readability and connectivity verified; execution must reproduce in `G15-P0-B1` |
| Starting source oracle | Dedicated `ChallengeStory*`/schedule/theme/target tables; shared entry, MazeBuff, BattleEvent, StageConfig, monster and `FantasticStory*`/level-program closure; CHS/EN TextMaps |
| Active-season hypothesis | Schedule `202024` → group `2024` (`借虚成真` / `Falsehood to Fact`) → rows `20241`–`20244`, Tierce `20245`, nine StageConfig candidates, Grit/Fever buffs and three selectable Cacophonies; not a denominator until `G15-P0-B3` |
| Focused inventory | Pending `G15-P0-B2` |
| Content manifest | Generated exact-once manifest freezes schedule `202024`, group `2024`, stages `20241`–`20244`, Tierce `20245`, nine StageConfig rows, active buffs, all encounter slots and transitive summon/template/skill closure; group `2025` is fail-closed evidence-only |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | Target `codex/goal15-pure-fiction-reference`; create it from current `origin/master` before `G15-P0-B1` |
| Branch base | `92febad080dd4cf9997718d64b3648fc198ab1f8`; equal to `origin/master` and the remote execution branch before `G15-P0-B1` |
| Parallel inspection | The 2026-07-30 audit covered the legacy setup base; `G15-P0-B1` must repeat it against current Goal 14, 16 and 17 work and the execution branch |
| Parallel condition | Separate branch/worktree and six isolated Goal 15 roots while other Goals or integration work are active |
| Publication policy | Push and remotely verify each completed batch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources, manifest and contracts | `Complete` | Pinned caches, 1,816-file inventory, 796-obligation active manifest, exact exclusions/zero proofs and normalized/evidence/workbook/fixture contracts are machine-verified. |
| Phase 1 — Unique mode systems | `InProgress` | Profile/season flow is next; denominators and all authoring contracts are frozen. |
| Phase 2 — Content pools, services, events and enemies | `Pending` | Awaiting nonzero/zero pool proofs, themes/MazeBuffs/BattleEvents/configs, exact StageConfig waves, enemies, AI and abilities. |
| Phase 3 — Independent Sora and Excel | `Pending` | Awaiting isolated schemas/readers, three complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Ownership audit, fixtures, reconciliation and freeze | `Pending` | Awaiting exact-once audit, semantic fixtures, cross-goal receipts, regeneration and clean-checkout evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G15-P0-B1` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; push with `git push origin HEAD:codex/goal15-pure-fiction-reference`, verify using `git ls-remote --heads origin codex/goal15-pure-fiction-reference`. Reproduced clean pinned revisions `fd978d6e…` and `7b349e39…`, verified Goal 03 Complete, froze base/remote equality `92febad0…`, Version 4.4 scope/exclusions and all six isolated roots without changing another cache, workbook or production bundle. |
| `G15-P0-B2` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; pushed with `git push origin HEAD:codex/goal15-pure-fiction-reference` and verified by exact `git ls-remote`. Added an independently reproducible sparse-cache fetcher and deterministic `inventory.mjs --check`; the current generated inventory hashes 1,816 dedicated/adjacent challenge, entry/TextMap/StageConfig, `FantasticStory*`, enemy character, AI and ability-program files while retaining ownership only as a manifest decision. P0-B3 expanded the inventory only to close selectors discovered from the active rows. |
| `G15-P0-B3` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; pushed and resolved exactly by `git ls-remote`. `manifest.mjs --check` freezes 796 obligations through schedule `202024` → group `2024` → four stages/eight ordinary nodes plus Tierce `20245`, nine StageConfig rows, all waves/slots, active MazeBuff/Grit/Fever/Cacophony records, recursive summon/template/skill/character-config/AI/ability/status closure, seven selector-backed zero-family proofs, 25 rule and 18 fixture families, and the scheduled-unreleased group `2025` exclusion. |
| `G15-P0-B4` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; pushed and remotely resolved exactly. `contracts.mjs --check` freezes 36 normalized pack files, the common bilingual/provenance envelope, five evidence and mechanism qualities, canonical decimal/JSON/order rules, source-path/locator/digest reconciliation, 37 isolated Sora tables across three workbooks and the closed reference-only semantic fixture fact language. The phase full gate passed generated-artifact checks through the production catalog boundary, then was interrupted during a 17-minute concurrent-worktree Clippy compile; the change-aware gate passed in 129.7s and the mandatory clean full gate remains owned by P4-B3. |
| `G15-P1-B1` | `Complete` | This row's containing commit | Pushed to `origin/codex/goal15-pure-fiction-reference` and remotely resolved exactly. Imported one profile, one active season, four ordinary stages, eight ordinary nodes, one Tierce node and one Tierce/Starward record; preserves predecessor order, exact selectors, objectives, turn limits and clear scores without inventing a second Tierce team. |
| `G15-P1-B2` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on the execution branch. Imported three participant/loadout lock policies and five accepted/rejected/timeout/abandon/retry records with stage-attempt uniqueness, whole-loadout snapshotting and byte-identical rejection requirements; all unpublished scope details remain visible ProjectPolicy boundaries. |
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

## Goal package integration

| Field | Value |
|---|---|
| Legacy setup commit | `0e10539ffcffc4778cfb56e686663fbd2f4d72a4` (`G15-SETUP`) |
| Mainline integration | Merge commit `1e10d3c702461a869454dbb20ff6bf64969ed412` |
| Remote | `origin` |
| Integrated branch | `master` |
| Execution branch | Create `codex/goal15-pure-fiction-reference` from then-current `origin/master` before `G15-P0-B1` |
| Result | The planning package is published on mainline; no execution batch has started |

The legacy setup branch is historical input only. Goal 15 execution must freeze
the mainline integration and its new branch base rather than treating the
deleted setup branch as execution-owned foundation evidence.

## Frozen counters

Populate required counts only from the generated manifest in `G15-P0-B3`.
Do not estimate denominators from raw table sizes, prefixes, ID ranges,
schedule adjacency or display names. A zero denominator also requires a
generated selector-closure proof.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profile/season/entry/terminal outcomes | 1 / 1 / 1 / 2 | 0 | 0 | Stable family plus released active season; group `2025` is one separate evidence-only manifest exclusion. |
| Stages/nodes/Tierce/Starward/transitions | 4 stages / 8 ordinary nodes / 1 Tierce / 1 Tierce node | 0 | 0 | Transition behavior is additionally frozen in the rule/fixture denominator. |
| Participant/team/loadout/attempt records | 1 participant / 1 loadout / 1 attempt / 1 retry policy | 0 | 0 | Includes later semantic detail for uniqueness, snapshot, abandonment and reset. |
| Turn/AV clocks and wave lifecycle | 4 clocks / 27 waves | 0 | 0 | Tierce clock meaning remains an explicit policy-bound research field rather than an invented clock row. |
| Spawn queues/replacement rules | 1 policy / 63 authored slots | 0 | 0 | Ordered refill and simultaneous defeat behavior are frozen mechanic/fixture obligations. |
| Initial resources/battle entry | 1 | 0 | 0 | Exact values or a visible released-text policy are required before DataReady. |
| Score sources/caps/objectives/stars | 1 aggregation / 6 objective rows | 0 | 0 | Defeat/damage/cap/settlement behavior also has frozen rule/fixture obligations. |
| Whimsicality/Grit/Fever | 2 base MazeBuffs / 3 Fever sub-buffs | 0 | 0 | Exact active group/stage bindings and three transition contributions. |
| Cacophony choices/effects | 3 | 0 | 0 | MazeBuffs `3031359`, `3031362`, `3031361`. |
| Blessings/Curios/Occurrences/choices | 4 exact-zero proofs | 0 | 0 | One generated selector-closure proof per family. |
| Services/currencies/shops | 3 exact-zero proofs | 0 | 0 | One generated selector-closure proof per family. |
| Themes/MazeBuffs/BattleEvents/configs | 1 theme / 1 BattleEvent / 4 active FantasticStory programs | 0 | 0 | Seasonal MazeBuff rows are counted in their mechanic families above. |
| Encounter groups/waves/enemy slots | 9 StageConfig / 27 waves / 63 slots | 0 | 0 | Includes eight ordinary sides and one Tierce/Starward encounter. |
| Enemy skills/statuses/AI/abilities | 42 variants / 41 templates / 164 skills / 41 character configs / 28 AI / 112 abilities / 168 statuses | 0 | 0 | Recursive summons and selected config/ability matches close from the nine StageConfig rows. |
| Mechanic rules | 25 | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | 18 | 0 | 0 | One frozen fixture family per distinct lifecycle/spawn/score/seasonal/choice/encounter policy group. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-30 | Create Goal 15 as a complete Pure Fiction reference-data package, not a runtime goal. | Version 4.4 research can proceed independently without changing challenge, Activity or combat runtime. |
| 2026-07-30 | Use `pure-fiction` as the stable mode slug and reserve Goal 15. | Current `master` already uses Goal 14 for Gold and Gears runtime, while a clean published parallel branch also uses Goal 14 for Memory of Chaos reference setup; no local or remote Goal 15 package, branch or Pure Fiction root existed during the audit. |
| 2026-07-30 | Base the setup branch on committed `master@0191cc71b1735d6e101e6e04817181423c599232`. | The main workspace was clean and this captures the current Goal 14 runtime progress without modifying its files. |
| 2026-07-31 | Integrate the Ready planning package into mainline and retire the legacy setup branch. | The plan has no execution-owned data or frozen denominator yet; recreating the execution branch from current `origin/master` avoids carrying an obsolete base while preserving the setup commit in history. |
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
| `G15-R01` | `Resolved` | The generated source inventory closes the dedicated/shared table, entry mapping, TextMap, StageConfig, `FantasticStory*`, shared config and available enemy/stage program boundary. Enabled row-level closure remains manifest-owned. | P0-B2 | Reopen only when a selected manifest reference is absent from the hashed inventory or a stronger released source adds a required file. |
| `G15-R02` | `Resolved` | Structured schedule `202024` and official Version 4.3/4.4 boundaries select group `2024`; schedule `202025`/group `2025` begins after the fixed access boundary and is evidence-only. | P0-B3 | Reopen only for a revisioned snapshot with a later access boundary. |
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
