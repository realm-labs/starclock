# Goal 15 Status — Pure Fiction Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `pure-fiction-reference-v1` |
| State | `Complete` — Candidate reference data; runtime unreleased |
| Active phase | None |
| Active batch | — |
| Next unblocked batch | — |
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
| Phase 1 — Unique mode systems | `Complete` | Profile/season/Tierce, participants/loadouts/attempts, clocks/refill, score/objectives, Whimsicality/Grit/Fever, three Cacophonies and initial-resource policies regenerate byte-identically. |
| Phase 2 — Content pools, services, events and enemies | `Complete` | Seven exact-zero pool proofs, active event/program bindings, all nine encounters/27 waves/63 slots, complete 596-row enemy mechanic closure, 25 rule families, 18 fixtures and exact-once 796-row coverage are machine-verified. |
| Phase 3 — Independent Sora and Excel | `Complete` | Three workbooks regenerate byte-identically; 37 Sora exports load all 6,810 rows through isolated generated readers and every sheet/schema column passed rendered review. |
| Phase 4 — Ownership audit, fixtures, reconciliation and freeze | `Complete` | Ownership, semantic fixtures, peer reconciliation, full source-cache/clean-checkout acceptance and Candidate identity are frozen. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G15-P0-B1` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; push with `git push origin HEAD:codex/goal15-pure-fiction-reference`, verify using `git ls-remote --heads origin codex/goal15-pure-fiction-reference`. Reproduced clean pinned revisions `fd978d6e…` and `7b349e39…`, verified Goal 03 Complete, froze base/remote equality `92febad0…`, Version 4.4 scope/exclusions and all six isolated roots without changing another cache, workbook or production bundle. |
| `G15-P0-B2` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; pushed with `git push origin HEAD:codex/goal15-pure-fiction-reference` and verified by exact `git ls-remote`. Added an independently reproducible sparse-cache fetcher and deterministic `inventory.mjs --check`; the current generated inventory hashes 1,816 dedicated/adjacent challenge, entry/TextMap/StageConfig, `FantasticStory*`, enemy character, AI and ability-program files while retaining ownership only as a manifest decision. P0-B3 expanded the inventory only to close selectors discovered from the active rows. |
| `G15-P0-B3` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; pushed and resolved exactly by `git ls-remote`. `manifest.mjs --check` freezes 796 obligations through schedule `202024` → group `2024` → four stages/eight ordinary nodes plus Tierce `20245`, nine StageConfig rows, all waves/slots, active MazeBuff/Grit/Fever/Cacophony records, recursive summon/template/skill/character-config/AI/ability/status closure, seven selector-backed zero-family proofs, 25 rule and 18 fixture families, and the scheduled-unreleased group `2025` exclusion. |
| `G15-P0-B4` | `Complete` | This row's containing commit | `origin` / `codex/goal15-pure-fiction-reference`; pushed and remotely resolved exactly. `contracts.mjs --check` freezes 36 normalized pack files, the common bilingual/provenance envelope, five evidence and mechanism qualities, canonical decimal/JSON/order rules, source-path/locator/digest reconciliation, 37 isolated Sora tables across three workbooks and the closed reference-only semantic fixture fact language. The phase full gate passed generated-artifact checks through the production catalog boundary, then was interrupted during a 17-minute concurrent-worktree Clippy compile; the change-aware gate passed in 129.7s and the mandatory clean full gate remains owned by P4-B3. |
| `G15-P1-B1` | `Complete` | This row's containing commit | Pushed to `origin/codex/goal15-pure-fiction-reference` and remotely resolved exactly. Imported one profile, one active season, four ordinary stages, eight ordinary nodes, one Tierce node and one Tierce/Starward record; preserves predecessor order, exact selectors, objectives, turn limits and clear scores without inventing a second Tierce team. |
| `G15-P1-B2` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on the execution branch. Imported three participant/loadout lock policies and five accepted/rejected/timeout/abandon/retry records with stage-attempt uniqueness, whole-loadout snapshotting and byte-identical rejection requirements; all unpublished scope details remain visible ProjectPolicy boundaries. |
| `G15-P1-B3` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported four exact stage turn budgets plus one bounded continuous-refill policy, preserving 150/100 AV windows as an explicit challenge preset, independent node clocks, authored wave/slot order, stable simultaneous-defeat settlement, timeout score finalization and a stronger-evidence replacement condition. |
| `G15-P1-B4` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported defeat, partial-damage and stage aggregation programs plus all six ordinary/Tierce objective identities. Authoritative event attribution, integer score transport, simultaneous event order, 30,000 ordinary and 45,000 Tierce clear-score boundaries are explicit; still-obfuscated Tierce thresholds remain policy-bound rather than guessed. |
| `G15-P1-B5` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported the two selected Whimsicality/base bindings and all three Grit/Fever contributions (`3031227`–`3031229`) with exact stage-ability keys and canonical parameter vectors; hidden target/tie behavior remains fixture-bound ProjectPolicy rather than observed parity. |
| `G15-P1-B6` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported all three selectable Cacophonies (`3031359`, `3031362`, `3031361`) with exact post-character-birth binding keys and canonical parameter order; attempt-node choice locking and base-rule interaction remain explicit future-runtime contribution boundaries. |
| `G15-P1-B7` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported the battle-entry resource, retry restoration and cross-battle projection boundary; absence of a selected season override is recorded as ProjectPolicy with a field-level replacement condition, while exact Tierce identity/score/target contributions remain in the P1-B1 record. |
| `G15-P2-B1` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Generated exact-zero selector-closure proofs for Blessing, Curio, Occurrence and event-choice families; each distinguishes the complete 796-obligation active selector from reward/account tables and records zero selectors and zero reachable members. |
| `G15-P2-B2` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Extended the generated exact-zero closure to service, currency and shop, yielding seven fail-closed pool proofs with no inferred membership from rewards, names, shared tables or adjacent challenge rows. |
| `G15-P2-B3` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported theme 4, all eight active MazeBuff bindings, neutral continuous-spawn BattleEvent `31001` and four `FantasticStory*` programs reached by exact binding-key matches; program bytes remain immutable reference evidence and are not treated as executable handlers. |
| `G15-P2-B4` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported all nine released StageConfig encounters, 27 authored waves and 63 stable ordered enemy slots, preserving level, scoring group, infinite-group and BattleEvent joins for eight ordinary nodes plus Tierce `30322043`. |
| `G15-P2-B5` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Imported the recursive encounter closure: 42 concrete variants, 41 templates/character configs, 164 skills, 28 AI programs, 112 matched ability programs and 168 referenced statuses. Exact stat ratios, weaknesses, summons, skills, base stats, source bytes and modifier/status relationships remain Shared reference rows with no runtime lowering claim. |
| `G15-P2-B6` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly. Generated 25 mechanic rules, 796 provenance rows, 796 exact-once coverage rows, 606 shared reconciliation receipts, three nonblocking field-level gaps, 18 semantic fixtures and a 3,007-row canonical pack index. `verify-pack.mjs` proves all 796 obligations DataReady, bilingual/provenance closure, zero runtime leakage and pack digest `8197bc56…a4766`. |
| `G15-P3-B1` | `Complete` | This row's containing commit | Pushed to `origin/codex/goal15-pure-fiction-reference` with `git push origin HEAD:codex/goal15-pure-fiction-reference` and remotely resolved exactly using `git ls-remote --heads origin codex/goal15-pure-fiction-reference`. `generate-sora-schema.mjs --batch=G15-P3-B1 --check` freezes seven isolated, indexed Sora tables for profile, season, stage, node, Tierce/Starward, participant and attempt authoring with bilingual/provenance/quality envelopes and no production-bundle dependency. |
| `G15-P3-B2` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. `generate-sora-schema.mjs --batch=G15-P3-B2 --check` freezes seven isolated clock, spawn, score, objective, seasonal-mechanic, Cacophony and initial-resource tables; exact payload order is preserved as canonical JSON and unavailable semantics remain visibly quality-bound. |
| `G15-P3-B3` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. `generate-sora-schema.mjs --batch=G15-P3-B3 --check` freezes 16 isolated pool-proof, theme, MazeBuff, event/program, encounter/wave/slot, enemy-closure and mechanic-rule tables. Shared enemy facts stay quality-labelled reference payloads instead of duplicated runtime semantics. |
| `G15-P3-B4` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. Sora 0.3.0 `check`, `schema-lock`, `excel-template` and Rust `gen` succeed for seven review tables and the complete 37-table project, producing lock `5e285b8d…4c99`, three isolated workbook templates and 43 reader modules without linking the reference package into production runtime. |
| `G15-P3-B5` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. Deterministic `openpyxl` authoring creates three clean workbooks with 37 contract-named sheets and 6,810 rows, including distinct manifest audit and coverage views. Structural/semantic verification proves template directives, stable-key uniqueness/order, complete headers, filters, frozen panes, zero formulas and zero runtime-executable rows; Sora 0.3.0 accepts all authored data. |
| `G15-P3-B6` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. `verify-authoring.mjs` proves byte-identical double workbook generation, normalized Sora templates, lock/readers, 37 debug tables and zstd bundle `d6602e68…02c3`; the isolated generated-reader crate loads all 37 tables/6,810 rows. LibreOffice/pdftoppm rendered 37 pages and five contact sheets; direct inspection of every page found consistent header bands, visible rows across A:P and no blank/clipped/broken required sheet, recorded in `workbook-visual-review.json`. |
| `G15-P4-B1` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. `audit-release.mjs` verifies all 796 manifest obligations appear exactly once in coverage and sources, all 5,218 authored rows are bilingual/DataReady/non-runtime with valid manifest/provenance references, seven selector-backed pools are ExactZero, group `2025` remains excluded, all three gaps are nonblocking and 606 reconciliation rows report no conflict or peer mutation. The 6,014-row normalized pack audit is frozen in `ownership-audit.json`. |
| `G15-P4-B2` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. The generated fixture pack now binds each of 18 lifecycle/spawn/score/Grit/Fever/Cacophony/encounter/Tierce families to its relevant normalized input and a closed `equals`/`contains`/`ordered_equals`/`absent` fact. `execute-semantic-fixtures.mjs` resolves and passes all 18 assertions, audits all three nonblocking ProjectPolicy replacement conditions and freezes result digest `17477b19…cad6`; regenerated workbooks/Sora bundle `c7960036…f9d0` remain deterministic and 37-page visual QA still passes. |
| `G15-P4-B3` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. Reconciliation against Memory of Chaos `30ed7491…`, completed Apocalyptic Shadow `f9f70e20…` and Fate `982b6bc5…` inspects all 606 shared receipts, finds one exact triple overlap and zero digest conflicts or peer mutation. A discovered file-level-vs-row-level digest mismatch for shared enemy `2032010` was corrected at the manifest generator boundary; canonical row digest now exactly matches Goal 17. `verify-release.mjs` passes pinned source/pack/workbook/Sora/reader/fixture regeneration, and `node tools/repository-check/run.mjs --full --with-source-cache` passes in 371.8s including 32 generated checks, Clippy and 33 workspace harnesses. |
| `G15-P4-B4` | `Complete` | This row's containing commit | Pushed and remotely resolved exactly on `origin/codex/goal15-pure-fiction-reference`. `freeze-candidate-release.mjs --check` pins all 26 prerequisite batch commits, allowed-path isolation, 796/796 DataReady obligations, 6,014 normalized rows, 796 sources, 606 shared receipts, three reviewed replacement boundaries, 18 fixtures, three workbooks/37 sheets/6,810 rows, 37 Sora tables/43 reader files and zero runtime profiles. Release evidence `d9ae1add…6fac` records manifest `87134542…50e`, pack tree `49f50997…ece7`, workbook semantic digest `63cd6b0a…a2ee`, Sora bundle `06775011…c4df`, the 371.8s full source-cache pass, clean pushed-checkout pass at `4c8aadfb…f8af`, and final peer manifests through Fate Candidate `5517b217…b1c`. |

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
| Profile/season/entry/terminal outcomes | 1 / 1 / 1 / 2 | 1 / 1 / 1 / 2 | 1 / 1 / 1 / 2 | Stable family, released active season and terminal reference outcomes; group `2025` remains a separate evidence-only exclusion. |
| Stages/nodes/Tierce/Starward/transitions | 4 stages / 8 ordinary nodes / 1 Tierce / 1 Tierce node | 4 / 8 / 1 / 1 | 4 / 8 / 1 / 1 | Transition behavior is additionally frozen in rule and fixture families. |
| Participant/team/loadout/attempt records | 1 participant / 1 loadout / 1 attempt / 1 retry policy | 1 / 1 / 1 / 1 | 1 / 1 / 1 / 1 | Uniqueness, snapshot, rejection and retry boundaries are explicit reference policies. |
| Turn/AV clocks and wave lifecycle | 4 clocks / 27 waves | 4 / 27 | 4 / 27 | Tierce clock interpretation remains a visible nonblocking replacement boundary. |
| Spawn queues/replacement rules | 1 policy / 63 authored slots | 1 / 63 | 1 / 63 | Ordered refill and simultaneous defeat policy are fixture-reviewed. |
| Initial resources/battle entry | 1 | 1 | 1 | Ordinary defaults remain an explicit field-level ProjectPolicy pending a stronger selector. |
| Score sources/caps/objectives/stars | 1 aggregation / 6 objective rows | 1 / 6 | 1 / 6 | Defeat, damage, cap, aggregation and Tierce target identity boundaries are retained. |
| Whimsicality/Grit/Fever | 2 base MazeBuffs / 3 Fever sub-buffs | 2 / 3 | 2 / 3 | Exact active bindings and canonical parameter vectors are retained. |
| Cacophony choices/effects | 3 | 3 | 3 | MazeBuffs `3031359`, `3031362`, `3031361`. |
| Blessings/Curios/Occurrences/choices | 4 exact-zero proofs | 4 | 4 | Generated selector closure proves zero reachable members. |
| Services/currencies/shops | 3 exact-zero proofs | 3 | 3 | Generated selector closure proves zero reachable members. |
| Themes/MazeBuffs/BattleEvents/configs | 1 theme / 1 BattleEvent / 4 active FantasticStory programs | 1 / 1 / 4 | 1 / 1 / 4 | Seasonal MazeBuff rows are counted in their mechanic families above. |
| Encounter groups/waves/enemy slots | 9 StageConfig / 27 waves / 63 slots | 9 / 27 / 63 | 9 / 27 / 63 | Includes eight ordinary sides and one Tierce/Starward encounter. |
| Enemy skills/statuses/AI/abilities | 42 variants / 41 templates / 164 skills / 41 character configs / 28 AI / 112 abilities / 168 statuses | 42 / 41 / 164 / 41 / 28 / 112 / 168 | 42 / 41 / 164 / 41 / 28 / 112 / 168 | Recursive summons and selected config/ability matches close from all nine StageConfig rows. |
| Mechanic rules | 25 | 25 | 25 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | 18 | 18 | 18 | All distinct lifecycle/spawn/score/seasonal/choice/encounter families pass reference execution. |

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
| 2026-08-01 | Canonicalize unique structured-table rows before hashing manifest evidence; retain whole-file digests only for file/program or non-unique policy locators. | Shared row `MonsterID=2032010` exposed that a whole-table digest is not comparable to peer row evidence even at the same path and locator; row-level canonical hashing removes the false conflict without changing source facts. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G15-R01` | `Resolved` | The generated source inventory closes the dedicated/shared table, entry mapping, TextMap, StageConfig, `FantasticStory*`, shared config and available enemy/stage program boundary. Enabled row-level closure remains manifest-owned. | P0-B2 | Reopen only when a selected manifest reference is absent from the hashed inventory or a stronger released source adds a required file. |
| `G15-R02` | `Resolved` | Structured schedule `202024` and official Version 4.3/4.4 boundaries select group `2024`; schedule `202025`/group `2025` begins after the fixed access boundary and is evidence-only. | P0-B3 | Reopen only for a revisioned snapshot with a later access boundary. |
| `G15-R03` | `PolicyBound` | What exact topology, participant slots, clock, objective and settlement semantics do Tierce `20245`, Starward and Fever add? | P0-B3 / P1-B1–B7 | Exact identities and contributions are retained; unavailable interpretation stays nonblocking until decoded released schema/reference joins replace it. |
| `G15-R04` | `PolicyBound` | What are the exact character/combat-form, Light Cone and Relic-instance uniqueness and loadout invalidation scopes across every ordinary and Tierce/Starward node? | P1-B2 | Participant/lock rows and rejection fixtures freeze the deterministic reference boundary pending stronger released evidence. |
| `G15-R05` | `PolicyBound` | How do turn/AV budgets, node scope, wave transitions, continuous enemy replacement, timeout and early final-group completion compose? | P1-B3 | Exact clocks/waves and explicit refill/tie policies carry reviewed replacement conditions. |
| `G15-R06` | `PolicyBound` | How are defeat and damage points attributed, capped and aggregated under simultaneous defeats, overkill, summons, environmental damage, timeout and early completion? | P1-B4 | Reference score sources and deterministic settlement policies are fixture-reviewed; runtime lowering remains future work. |
| `G15-R07` | `PolicyBound` | What are the exact Whimsicality and Grit/Fever gain filters, per-enemy trigger cap, thresholds, effect parameters, transition order, target selection and teardown? | P1-B5 / P2-B3 | Exact bindings/parameters and visible policy boundaries are retained with stronger-evidence triggers. |
| `G15-R08` | `PolicyBound` | When and per what scope are the three Cacophonies offered, locked, applied and removed, and how do they interact with the base seasonal mechanic? | P1-B6 / P2-B3 | All three exact choices are retained; choice/lifecycle semantics remain reference-only and fixture-reviewed. |
| `G15-R09` | `PolicyBound` | Which config lists and challenge constants define initial resources, battle entry, retry restoration and cross-node carry/reset? | P1-B7 / P2-B3 | The missing active override is recorded as a nonblocking explicit default policy with a released-selector replacement condition. |
| `G15-R10` | `Resolved` | Are any Blessings, Curios, Occurrences, event choices, services, currencies, shops or analogous pools mechanically reachable? | P2-B1–B2 | Seven generated selector-closure proofs establish exact zero reachable members. |
| `G15-R11` | `Resolved` | Which StageConfig waves, concrete variants, levels, skills, statuses, AI, summons, phases and abilities define every ordinary and Tierce/Starward encounter? | P2-B4–B5 | Nine StageConfig rows, 27 waves, 63 slots and the complete recursive enemy/config/AI/ability/status closure are retained. |
| `G15-R12` | `PolicyBound` | Which hidden ordering, timing, random selection, caps, rounding and fallback fields remain unavailable after bounded research? | P2-B6 / P4-B2 | Three nonblocking ProjectPolicy rows name selected policies, rejected alternatives where relevant, affected fixtures and concrete replacement conditions. |

## Terminal checklist

- [x] Exact active-season category manifests and denominators are frozen.
- [x] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [x] Complete normalized pack and canonical pack index regenerate without
      drift.
- [x] All required rows have bilingual names, independent summaries and
      row-level provenance.
- [x] Ownership, active-release enablement and shared reachability are explicit
      and fail closed.
- [x] Tierce, Starward and Fever topology, participant, clock, objective and
      settlement semantics are proved or explicitly policy-bounded.
- [x] Empty content-pool families have generated selector-closure proofs.
- [x] Shared classifications reconcile with committed overlapping Goal facts.
- [x] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [x] Participant/loadout locks, attempts, clocks, spawn/refill, scores,
      objectives/stars, Whimsicality, Grit/Fever and Cacophony have complete
      semantic fixtures.
- [x] Encounter identities, StageConfig rows, waves, variants, AI, abilities
      and boss bindings resolve.
- [x] Isolated Sora schemas, templates and generated readers validate.
- [x] All three complete `openpyxl` workbooks pass structural and visual QA.
- [x] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [x] Goal 03 evidence and all other mode/production bundle identities remain
      unchanged.
- [x] Coverage reports 100% `DataReady` and no blocking research row.
- [x] Every completed batch commit is reachable from its recorded remote branch
      at the recorded full commit ID.
- [x] Clean-checkout acceptance passes and `G15-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | `Complete` — Candidate reference data; runtime unreleased |
| Completion commit | This row's containing `G15-P4-B4` commit |
| Remote/branch verification | `origin/codex/goal15-pure-fiction-reference`; local, tracking and `git ls-remote` resolve to the containing commit after publication |
| Pure Fiction reference bundle | Manifest `87134542…50e`; normalized pack tree `49f50997…ece7`; Sora bundle `06775011…c4df` |
| Workbook semantic digest | `63cd6b0a774d85c57b20daac1cf94786a785dd4303c57f57a124c36d8743a2ee`; 3 workbooks / 37 sheets / 6,810 rows |
| Coverage | 796/796 `DataReady`; 25 rules, 18 fixtures, 3 nonblocking replacement boundaries and zero runtime rows |
| Release evidence | `evidence/pure-fiction-v1/release/release-evidence.json`; SHA-256 `d9ae1addd2c7898c9637da7367138f6b06e31f73360941123b776579d5316fac` |
| Remaining required work | Pure Fiction runtime lowering, integration, handlers, controller/API exposure and seeded full challenge runs belong to a later goal. |
