# Goal 17 Status — Memory of Chaos Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `memory-of-chaos-reference-v1` |
| State | `Ready` |
| Active phase | Not started |
| Active batch | — |
| Next unblocked batch | `G17-P0-B1` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-30: both caches clean/detached at pinned commits; origins, required commit/blob readability and connectivity verified; execution must reproduce in `G17-P0-B1` |
| Starting source oracle | Dedicated `ChallengeMaze*`/schedule/target/const tables; shared entry, MazeBuff, BattleEvent, StageConfig, monster and config/ability closure; CHS/EN TextMaps |
| Active-season hypothesis | Schedule `201033` → group `1033` (`学院怪谈` / `Academy Ghost Story`) → rows `5201`–`5212`, Tierce `5213`, MazeBuff `3030146`, targets `251`–`253`, 25 StageConfig candidates and Battle Event `30146`; not a denominator until `G17-P0-B3` |
| Focused inventory | Pending `G17-P0-B2` |
| Content manifest | Denominators pending `G17-P0-B3` |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | Target `codex/goal17-memory-of-chaos-reference`; create it from current `origin/master` before `G17-P0-B1` |
| Branch base | Pending execution-time freeze in `G17-P0-B1`; this plan is integrated by the containing mainline merge commit |
| Parallel inspection | The 2026-07-30 audit covered the original setup base; `G17-P0-B1` must repeat it against current Goal 14–16 work and the execution branch |
| Parallel condition | Separate branch/worktree and six isolated Goal 17 roots while other Goals or integration work are active |
| Publication policy | Push and remotely verify each completed batch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Pending` | Awaiting execution-owned cache reproduction, focused inventory, active-season/Tierce manifest and authoring contracts. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting profile/season flow, participants/loadouts, attempts, clocks, objectives/stars, resources, Turbulence and Tierce semantics. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool zero/nonzero proofs, shared events/configs, exact StageConfig waves, enemies, AI and abilities. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, three complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership audit, semantic fixtures, reconciliation, regeneration and clean-checkout evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G17-P0-B1` | `Pending` | — | Reproduce caches, verify Goal 03 and concurrent boundaries, freeze scope and prove isolation. |
| `G17-P0-B2` | `Pending` | — | Inventory dedicated/adjacent tables, entry mappings, config/ability programs, TextMaps, StageConfig, enemies and exclusions. |
| `G17-P0-B3` | `Pending` | — | Freeze active-season selectors, exact obligations/counts, Tierce semantics, ownership, reachability, proven-empty pools and exclusions. |
| `G17-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G17-P1-B1` | `Pending` | — | Import profile, active season, entry/unlocks, floors/stages, ordinary nodes, Tierce identity, legal order and outcomes. |
| `G17-P1-B2` | `Pending` | — | Import participants, team/loadout uniqueness, snapshots/locks, attempts, retries, reset and transitions. |
| `G17-P1-B3` | `Pending` | — | Import cycle/AV clocks, tick boundaries, node/wave carry or reset, expiry and failure timing. |
| `G17-P1-B4` | `Pending` | — | Import completion/survival/cycle objectives, evaluation, stars and aggregation. |
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
| Profile/season/entry/terminal outcomes | TBD | 0 | 0 | Stable family plus active released Version 4.4 season; historical/future seasons remain evidence-only. |
| Floors/stages/nodes/Tierce/transitions | TBD | 0 | 0 | Freeze ordinary topology and the exact selected Tierce role without assuming a third node from its name. |
| Participant/team/loadout/attempt records | TBD | 0 | 0 | Includes uniqueness, snapshots, locks, substitution, retry and reset scope. |
| Cycle/AV clocks and wave carry | TBD | 0 | 0 | Includes initial budget, first-cycle window, tick, node/wave boundary, expiry and failure. |
| Initial resources/battle entry | TBD | 0 | 0 | Includes exact HP/Energy/Skill Point and selected entry operations. |
| Targets/objectives/stars/aggregation | TBD | 0 | 0 | Includes completion, downed-character and remaining-cycle evaluation. |
| Memory Turbulence/MazeBuff/BattleEvent | TBD | 0 | 0 | Includes parameters, triggers, hit state, target selection, True DMG and teardown. |
| Blessings/Curios/Occurrences | TBD | 0 | 0 | Freeze exact reachable rows or an exact-zero selector proof for each family. |
| Services/currencies/shops/choices | TBD | 0 | 0 | Reward/account tables do not prove a mechanically reachable service or currency. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Resolve every ordinary and Tierce StageConfig row, variant, difficulty and boss phase. |
| Enemy skills/statuses/AI/abilities | TBD | 0 | 0 | Include complete transitive mechanic closure for each enabled enemy variant. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every unique lifecycle, objective, Turbulence and encounter policy. |

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
| `G17-R01` | `Open` | Which dedicated/shared table, entry mapping, TextMap, StageConfig, config/ability, enemy and AI files complete the focused inventory? | P0-B2 | Replace when generated inventory closes every enabled selector/reference and byte-identical double generation passes. |
| `G17-R02` | `Open` | Which released selector proves group `1033` is the active Version 4.4 season and excludes historical groups plus scheduled-but-unreleased group `1034`? | P0-B3 | Replace with an exact-once manifest whose rows carry active-release selector/reference evidence and fail-closed exclusions. |
| `G17-R03` | `Open` | What exact topology, participant slot, clock, objective and settlement semantics does selected Tierce record `5213` add? | P0-B3 / P1-B1–B6 | Replace with decoded schema/reference joins and fixtures; record any missing runtime capability for a later goal without changing runtime here. |
| `G17-R04` | `Open` | What are the exact character/combat-form, Light Cone and Relic-instance uniqueness scopes and loadout invalidation rules across ordinary and Tierce nodes? | P1-B2 | Replace with source-backed participant/lock rows and accepted/rejected/retry fixtures. |
| `G17-R05` | `Open` | How do the initial cycle budget, first-cycle AV, tick, wave boundary, node carry, Tierce carry, expiry and failure order compose? | P1-B3 | Replace with config/level evidence or field-level policies carrying alternatives and lifecycle fixtures. |
| `G17-R06` | `Open` | When are completion, remaining-cycle and downed-character objectives evaluated, and how do their stars aggregate across all required nodes? | P1-B4 | Replace with typed objective/aggregation rows and fixtures for victory, failure, retry and partial completion. |
| `G17-R07` | `Open` | What are the exact Memory Turbulence trigger filters, per-action hit gain, cap, cycle-start order, random target set, True-DMG calculation, attribution and empty-target behavior? | P1-B5 / P2-B2 | Replace with MazeBuff/BattleEvent/ability evidence and one fixture per distinct trigger/target/lifecycle boundary. |
| `G17-R08` | `Open` | Which `ConfigList`, stage-template and challenge constants define initial HP, Energy, Skill Points and battle-entry state? | P1-B6 / P2-B2 | Replace with explicit selector-to-operation joins and initial-state fixtures; unsupported mappings fail closed. |
| `G17-R09` | `Open` | Are any Blessings, Curios, Occurrences, services, currencies, shops, choices or analogous pools mechanically reachable? | P2-B1 | Replace each family with a selector-backed nonzero closure or generated exact-zero proof. |
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
