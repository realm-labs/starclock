# Goal 17 Status — Memory of Chaos Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `memory-of-chaos-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Unique mode systems |
| Active batch | — |
| Next unblocked batch | `G17-P1-B5` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-30: both caches clean/detached at pinned commits; origins, required commit/blob readability and connectivity verified; execution must reproduce in `G17-P0-B1` |
| Starting source oracle | Dedicated `ChallengeMaze*`/schedule/target/const tables; shared entry, MazeBuff, BattleEvent, StageConfig, monster and config/ability closure; CHS/EN TextMaps |
| Active-season hypothesis | Schedule `201033` → group `1033` (`学院怪谈` / `Academy Ghost Story`) → rows `5201`–`5212`, Tierce `5213`, MazeBuff `3030146`, targets `251`–`253`, 25 StageConfig candidates and Battle Event `30146`; not a denominator until `G17-P0-B3` |
| Focused inventory | 2,703 pinned receipts: 2,646 inherited Goal 03 enemy/mechanic files, 10 dedicated Memory/Forgotten Hall tables, 17 shared entry/stage/event/config/TextMap seeds, 26 named adjacent Challenge exclusions, one additional Challenge evidence table and three StarRailRes identity receipts |
| Content manifest | 477 exact obligations: 172 MemoryOfChaos-owned and 305 Shared; active schedule `201033`, group `1033`, 12 ordinary stages, one Tierce, 25 StageConfig rows, 99 enemy slots, 41 variants/templates, 221 abilities and 10 exact-zero pool proofs |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal17-memory-of-chaos-reference`, tracking `origin/codex/goal17-memory-of-chaos-reference` |
| Branch base | `92febad080dd4cf9997718d64b3648fc198ab1f8`, equal to `origin/master` and the execution remote branch at launch |
| Parallel inspection | Reproduced 2026-08-01 against the main workspace and all registered worktrees; Goal 17 owns one dedicated worktree and six isolated roots. |
| Parallel condition | Separate branch/worktree and six isolated Goal 17 roots while other Goals or integration work are active |
| Publication policy | Push and remotely verify each completed batch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Complete` | Isolated cache/Goal 03/worktree foundation, 2,703-file inventory, 477-obligation active manifest, exact Tierce outer role, 10 empty-pool proofs and the 27-file/27-table/18-fixture authoring contract are frozen and deterministic. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting profile/season flow, participants/loadouts, attempts, clocks, objectives/stars, resources, Turbulence and Tierce semantics. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool zero/nonzero proofs, shared events/configs, exact StageConfig waves, enemies, AI and abilities. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, three complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership audit, semantic fixtures, reconciliation, regeneration and clean-checkout evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G17-P0-B1` | `Complete` | This row's containing commit | Reproduced 28 required-file receipts in isolated clean/detached caches at `turnbasedgamedata@fd978d6e…c3568` and `StarRailRes@7b349e39…7a93`; verified origins, commit/blob readability and `git fsck --connectivity-only --no-dangling`. Goal 03 is Complete at immutable commit `60ca52ed…89de`. The execution branch/upstream/base, all registered worktrees, six writable roots and eight committed Goal 07–13/16 ownership inputs are frozen in `content-manifests/memory-of-chaos-v1/foundation.json`. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos node tools/memory-of-chaos-reference/foundation.mjs --check`, `git diff --check`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push command `git push origin HEAD:codex/goal17-memory-of-chaos-reference`; verification command `git ls-remote --exit-code origin refs/heads/codex/goal17-memory-of-chaos-reference` must resolve to this row's containing commit. |
| `G17-P0-B2` | `Complete` | This row's containing commit | Generated and rechecked `source-inventory.json` with 2,703 pinned receipts: 2,646 inherited Goal 03 enemy/mechanic files, 10 dedicated Memory/Forgotten Hall tables, 17 shared entry/stage/event/config/TextMap seeds, 26 named adjacent Challenge exclusions, one remaining Challenge evidence table and three StarRailRes identity receipts. The 25 active StageConfig rows remain planning candidates; membership and transitive closure are reserved for P0-B3. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos node tools/memory-of-chaos-reference/inventory.mjs --check`, `git diff --check`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push command `git push origin HEAD:codex/goal17-memory-of-chaos-reference`; verification command `git ls-remote --exit-code origin refs/heads/codex/goal17-memory-of-chaos-reference` must resolve to this row's containing commit. |
| `G17-P0-B3` | `Complete` | This row's containing commit | Froze 477 exact obligations (172 MemoryOfChaos-owned, 305 Shared): active schedule `201033` and group `1033`; 12 ordinary stages; selected Tierce `5213`; six objectives; 25 released StageConfig rows; 99 exact wave/slot bindings; 41 enemy variants/templates; 221 inherited enemy abilities; 14 participant/attempt/clock/resource semantic obligations; and 10 generated empty-pool proofs. Tierce is proved as a separately selected extension after `5212` with one StageConfig `30123123`, targets `601`–`603` and countdown `45`; no third ordinary node/team/clock/settlement rule is inferred. Future schedule/group `201034`/`1034` are fail-closed exclusions. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos node tools/memory-of-chaos-reference/manifest.mjs --check`, `git diff --check`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push command `git push origin HEAD:codex/goal17-memory-of-chaos-reference`; verification command `git ls-remote --exit-code origin refs/heads/codex/goal17-memory-of-chaos-reference` must resolve to this row's containing commit. |
| `G17-P0-B4` | `Complete` | This row's containing commit | Froze 27 normalized files, canonical UTF-8/LF/two-space JSON and decimal-string encoding, common bilingual/DataReady/evidence envelopes, field-level approximation requirements, source-path/row-locator/digest reconciliation identity, three isolated workbooks with 27 primary Sora tables and 18 non-shrinking semantic fixture families. Runtime rows and profile publication remain forbidden. Commands: `node tools/memory-of-chaos-reference/contracts.mjs --check`, `git diff --check`, `PATH="$PWD/.cache/python/bin:$PATH" STARCLOCK_SORA_BIN=/Users/mikai/CLionProjects/starclock/.cache/tools/sora-cli-0.3.0/bin/sora fnm exec --using 24.15.0 node tools/repository-check/run.mjs --full`; the full gate passed in 5,526.4 seconds, including 33 workspace test harnesses. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push command `git push origin HEAD:codex/goal17-memory-of-chaos-reference`; verification command `git ls-remote --exit-code origin refs/heads/codex/goal17-memory-of-chaos-reference` must resolve to this row's containing commit. |
| `G17-P1-B1` | `Complete` | This row's containing commit | Imported the stable profile, active schedule/group, two entry locators, 12 ordered ordinary stages, 24 derived node selectors, selected Tierce extension and three fail-closed terminal outcomes. All 18 frozen family/season/entry/stage/Tierce obligations are claimed exactly once; Tierce participant, clock-carry and settlement semantics remain explicit runtime prerequisites. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos fnm exec --using 24.15.0 node tools/memory-of-chaos-reference/import-flow.mjs --check`, `git diff --check`, `node tools/repository-check/run.mjs`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push and remote-resolution verification use the Goal publication commands. |
| `G17-P1-B2` | `Complete` | This row's containing commit | Resolved all six participant/attempt obligations: two ordinary node-bound team slots, Section-scope combat-form uniqueness, immutable Light Cone/Relic-instance loadout snapshots, accepted/rejected start boundaries, fail-closed whole-stage retry policy and locked Node 1→2 transition. Released public text is retained only as a Version 1.1 stable-family two-team cross-check; Version 4.4 instance/retry details remain explicit ProjectPolicy with replacement conditions. Tierce does not inherit ordinary participant semantics. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos fnm exec --using 24.15.0 node tools/memory-of-chaos-reference/import-participants.mjs --check`, `git diff --check`, `node tools/repository-check/run.mjs`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push and remote-resolution verification use the Goal publication commands. |
| `G17-P1-B3` | `Complete` | This row's containing commit | Imported all six clock obligations. Twelve exact active stage rows declare countdown 30; Section ownership, the 150/100 AV preset, integer-only Node 1→2 carry, no-reset wave carry and expiry-before-cycle-start ordering are explicit Candidate ProjectPolicy rather than observed parity. Tierce clock composition is not inherited. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos fnm exec --using 24.15.0 node tools/memory-of-chaos-reference/import-clocks.mjs --check`, `git diff --check`, `node tools/repository-check/run.mjs`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push and remote-resolution verification use the Goal publication commands. |
| `G17-P1-B4` | `Complete` | This row's containing commit | Imported all six exact target rows: ordinary remaining-cycle thresholds 10/20 plus zero downed, and Tierce thresholds 15/30 plus zero downed. Target identity/type/value is ExactStructured; completion-gated timing, all-required-battle survival scope and independent cumulative best-objective aggregation use stable-family public cross-checks plus explicit Version 4.4 ProjectPolicy. Reward payloads remain excluded. Commands: `STARCLOCK_SOURCE_CACHE=/Users/mikai/.codex/source-caches/goal17-memory-of-chaos fnm exec --using 24.15.0 node tools/memory-of-chaos-reference/import-objectives.mjs --check`, `git diff --check`, `node tools/repository-check/run.mjs`. Remote `origin`, branch `codex/goal17-memory-of-chaos-reference`; push and remote-resolution verification use the Goal publication commands. |
| `G17-P1-B5` | `Pending` | — | Import Memory Turbulence, hit accumulation, cap, cycle trigger, target policy, True DMG and teardown. |
| `G17-P1-B6` | `Pending` | — | Import initial resources, battle entry, Tierce-specific contributions and cross-battle projections. |
| `G17-P2-B1` | `Pending` | — | Audit/freeze reachable or exact-zero Blessing, Curio, Occurrence, service, currency, shop and choice pools. |
| `G17-P2-B2` | `Pending` | — | Import enabled challenge definitions, stage templates, MazeBuffs, BattleEvents and config/ability relationships. |
| `G17-P2-B3` | `Pending` | — | Import exact StageConfig encounters, waves, slots, variants, levels and difficulty bindings. |
| `G17-P2-B4` | `Pending` | — | Import enemy skills/statuses/AI/abilities, summons, linked actors, boss phases and rule contributions. |
| `G17-P2-B5` | `Pending` | — | Generate mechanics, sources, coverage, research gaps, fixtures and pack index. |
| `G17-P3-B1` | `Pending` | — | Add profile/season/stage/node/Tierce/participant/attempt Sora tables. |
| `G17-P3-B2` | `Pending` | — | Add clock/resource/objective/star/Turbulence/event/contribution Sora tables. |
| `G17-P3-B3` | `Pending` | — | Add pool, encounter, wave, enemy and mechanic-binding Sora tables. |
| `G17-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated locks/templates/readers. |
| `G17-P3-B5` | `Pending` | — | Generate and structurally/semantically verify all three complete `openpyxl` workbooks. |
| `G17-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G17-P4-B1` | `Pending` | — | Audit exact-once coverage, season/Tierce selection, ownership, references, provenance and bilingual fields. |
| `G17-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G17-P4-B3` | `Pending` | — | Reconcile overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G17-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but remotely unverified
batch remains `InProgress`.

## Goal package integration

| Field | Value |
|---|---|
| Legacy setup commit | `c4a01cbc4b6f8ee5f86987dd0d5f7f2574358a05` (`G14-SETUP` before collision resolution) |
| Mainline integration | This document's containing merge commit, renumbered atomically to Goal 17 |
| Remote | `origin` |
| Integrated branch | `master` |
| Execution branch | Create `codex/goal17-memory-of-chaos-reference` from then-current `origin/master` before `G17-P0-B1` |
| Result | The planning package is published on mainline; no execution batch has started |

The original Goal 14 setup branch is historical input only. Goal 17 execution
must freeze the containing mainline commit and its new branch base rather than
claiming the legacy setup commit as execution-owned foundation evidence.

## Frozen counters

Populate required counts only from the generated manifest in `G17-P0-B3`.
Do not estimate denominators from raw table sizes, prefixes, ID ranges,
schedule adjacency or display names. A zero denominator also requires a
generated selector-closure proof.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profile/season/entry/terminal outcomes | 5 | 5 | 5 | Stable Memory family, active schedule/group and two entry locators are exact; three lifecycle outcomes are explicit fail-closed policy rows outside the frozen source denominator. |
| Floors/stages/nodes/Tierce/transitions | 13 | 13 | 13 | Twelve ordinary stages plus one separately selected Tierce extension; 24 node selectors are derived without inflating the frozen denominator. |
| Participant/team/loadout/attempt records | 6 | 6 | 6 | Two-team stable-family evidence plus explicit Version 4.4 ProjectPolicy rows cover uniqueness, immutable snapshots, retry/reset and node transition; Tierce remains separate. |
| Cycle/AV clocks and wave carry | 6 | 6 | 6 | Countdown 30 is exact on all 12 active rows; ownership, AV window, carry/reset and expiry ordering are explicit replaceable ProjectPolicy. |
| Initial resources/battle entry | 2 | 0 | 0 | HP/Energy/Skill Point state and selected battle-entry operations. |
| Targets/objectives/stars/aggregation | 6 | 6 | 6 | Exact ordinary 10/20/zero-downed and Tierce 15/30/zero-downed targets; timing and cumulative best-objective aggregation are explicit replaceable policies. |
| Memory Turbulence/MazeBuff/BattleEvent | 2 | 0 | 0 | MazeBuff `3030146` and BattleEvent `30146`; derived operations are imported later. |
| Blessings/Curios/Occurrences | 3 exact-zero proofs | 0 | 0 | Active selector closure contains no mechanically reachable row in any family. |
| Services/currencies/shops/choices | 7 exact-zero proofs | 0 | 0 | Includes service, currency, shop, choice and analogous Rogue families. |
| Encounter groups/waves/enemy slots | 124 | 0 | 0 | Twenty-five StageConfig rows and 99 ordered enemy slots. |
| Enemy skills/statuses/AI/abilities | 303 | 0 | 0 | Forty-one exact variants, 41 inherited templates and 221 inherited abilities; transitive mechanics import remains Phase 2. |
| Mechanic rules | 18 families | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | 18 families | 0 | 0 | Non-shrinking families cover selectors, lifecycles, objectives, Turbulence, encounter and exclusion/reconciliation policies. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-30 | Draft a complete Memory of Chaos reference-data package, not a runtime goal. | Version 4.4 research can proceed independently without changing challenge, Activity or combat runtime. |
| 2026-07-30 | Use `memory-of-chaos` as the stable mode slug. | The slug and isolated artifact ownership remain valid independently of the planning-time Goal number. |
| 2026-07-30 | Base the legacy `G14-SETUP` commit on `7dd2a4cb…`. | This records the original planning audit without pretending that its now-conflicting Goal number is current. |
| 2026-07-31 | Renumber the planning package from Goal 14 to Goal 17 during mainline integration. | Current mainline already owns Goal 14 for Gold and Gears Runtime, Goal 15 for Pure Fiction and Goal 16 for Galactic Baseballer; Goal 17 is the first collision-free number. No execution batch or artifact had started, so the rename changes no frozen evidence. |
| 2026-07-30 | Inherit the pinned Version 4.4 source and identity revisions used by Goals 03 and 08–13. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-30 | Require `G17-P0-B1` to reproduce both caches even though the planning audit found them clean, detached and connected. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-30 | Treat schedule `201033`, group `1033`, rows `5201`–`5212`, Tierce `5213`, MazeBuff `3030146`, targets `251`–`253`, StageConfig candidates and Battle Event `30146` only as a planning selector chain. | Explicit references make them strong seeds, but the machine manifest must still prove exact active-season membership and exclusions. |
| 2026-07-30 | Exclude scheduled group `1034` unless released public evidence at the frozen access boundary proves availability. | A future schedule row in structured data does not override the prohibition on announced-but-unavailable content. |
| 2026-07-30 | Keep Tierce semantics open until source/config evidence proves topology, team, clock and settlement behavior. | `ChallengeMazeTierce` is selected by group `1033` but has obfuscated fields; naming and ID adjacency are insufficient semantic evidence. |
| 2026-07-30 | Treat the dedicated Challenge tables as inventory seeds, not ownership or completeness oracles. | Static Memory, rotating Memory of Chaos, account rewards, quick-clear state and adjacent challenge families share related tables. |
| 2026-07-30 | Audit Blessing, Curio, Occurrence, service, currency, shop and choice families even when the expected result is empty. | Completeness requires generated zero proofs; absence cannot be inferred from the challenge-mode label. |
| 2026-07-30 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent and completed goals must preserve isolated ledgers and surface conflicts for merge coordination. |
| 2026-07-30 | Exclude presentation, calendar behavior, quick-clear/account state and rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-30 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-30 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G17-R01` | `Closed` | The focused inventory contains 2,703 pinned receipts: 2,646 inherited Goal 03 transitive enemy/mechanic files, 54 Goal 17 dedicated/shared/adjacent Challenge receipts and three StarRailRes identity rows. | P0-B2 | `inventory.mjs --check` reproduces the path-sorted file inventory byte-identically; row reachability is separately frozen by P0-B3. |
| `G17-R02` | `Closed` | Schedule `201033` is active at the frozen 2026-08-01 boundary and explicitly selects group `1033`; schedule `201034` begins 2026-08-17 and group `1034` is excluded. | P0-B3 | Exact-once selector and exclusion receipts are frozen in `content-manifest.json`. |
| `G17-R03` | `PolicyBound` | Group `1033` selects Tierce `5213` once after `5212`; it binds one StageConfig `30123123`, objectives `601`–`603` and countdown `45`. Participant/team, clock carry and settlement details remain unclaimed and are a later runtime prerequisite. | P0-B3 / P1-B1–B6 | Replace remaining fields with decoded schema/reference joins and fixtures; do not infer them from the Tierce name or obfuscated field adjacency. |
| `G17-R04` | `PolicyBound` | Ordinary stages use two Section-unique combat-form team slots and immutable resolved Light Cone/Relic-instance snapshots. Tierce participant scope and released client retry/instance invalidation details are not inherited. | P1-B2 | Replace ordinary policy fields and Tierce boundary with released Version 4.4 lineup, equipment mutation and retry traces. |
| `G17-R05` | `PolicyBound` | Ordinary stages declare 30 exactly. Candidate policy uses one Section clock, 150/100 AV windows, integer-only node carry, no-reset wave carry and expiry before cycle-start rules; Tierce remains separate. | P1-B3 | Replace policy fields with a released Version 4.4 clock selector or traces covering partial node/wave windows and zero-boundary ordering. |
| `G17-R06` | `PolicyBound` | Six target kinds/thresholds are exact. Candidate policy evaluates after all required victories, uses all required battles for survival and latches each one-star objective independently across completed attempts. | P1-B4 | Replace evaluation, survival and cumulative aggregation fields with a released Version 4.4 settlement trace. |
| `G17-R07` | `Open` | What are the exact Memory Turbulence trigger filters, per-action hit gain, cap, cycle-start order, random target set, True-DMG calculation, attribution and empty-target behavior? | P1-B5 / P2-B2 | Replace with MazeBuff/BattleEvent/ability evidence and one fixture per distinct trigger/target/lifecycle boundary. |
| `G17-R08` | `Open` | Which `ConfigList`, stage-template and challenge constants define initial HP, Energy, Skill Points and battle-entry state? | P1-B6 / P2-B2 | Replace with explicit selector-to-operation joins and initial-state fixtures; unsupported mappings fail closed. |
| `G17-R09` | `Closed` | The active schedule/group/stage/config closure exposes no mechanically reachable Blessing, Curio, Occurrence, service, currency, shop, choice or analogous Rogue selector. | P0-B3 / P2-B1 | Ten exact-zero proof obligations are frozen; P2-B1 promotes their generated audit rows to DataReady. |
| `G17-R10` | `Open` | Which StageConfig waves, concrete enemy variants, levels, skills, statuses, AI, summons, phases and abilities define every ordinary and Tierce encounter? | P2-B3–B4 | Replace with complete encounter/enemy dossiers or an explicit nonblocking boundary for unavailable released evidence. |
| `G17-R11` | `Open` | Which hidden ordering, timing, random selection, caps, rounding and fallback fields remain unavailable after bounded research? | P2-B5 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete stronger-evidence trigger. |

## Terminal checklist

- [ ] Exact active-season category manifests and denominators are frozen.
- [ ] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership, active-release enablement and shared reachability are explicit
      and fail closed.
- [ ] Tierce identity and all of its topology, participant, clock, objective and
      encounter semantics are proved or explicitly policy-bounded.
- [ ] Empty content-pool families have generated selector-closure proofs.
- [ ] Shared classifications reconcile with committed Goal 07–13 and Goal 16 facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Participant/loadout locks, attempts, clocks, resources, objectives/stars,
      Turbulence and Tierce transitions have complete semantic fixtures.
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
- [ ] Clean-checkout acceptance passes and `G17-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Memory of Chaos reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G17-P0-B3` |
| Release evidence | — |
| Remaining required work | Memory of Chaos runtime lowering, integration, handlers, controller/API exposure and seeded full challenge runs belong to a later goal. |
