# Goal 19 Status — Fate/Star Rail Night Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `fate-star-rail-night-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 3 — Excel and Sora |
| Active batch | — |
| Next unblocked batch | `G19-P3-B6` |
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
| Phase 0 — Scope, sources and contracts | `Complete` | Reproduced sources, corrected the Fate/FateRin oracle, froze 1,904 obligations and six exact-zero pools, then bound 48 sheets across four workbook/evidence/Sora contracts. |
| Phase 1 — Unique activity systems | `Complete` | Seven normalized partitions contain 1,805 direct records: 1,392 enabled mode facts and all 413 evidence-only obligations; with six zero records, the complete 1,398 Fate-owned denominator is accounted. |
| Phase 2 — Encounters and complete pack | `Complete` | The 17-file pack accounts 1,904/1,904 obligations, 1,491 eligible DataReady rows, 413 evidence-only rows, 13 replaceable policies, 56 fixtures and zero unresolved records. |
| Phase 3 — Excel and Sora | `Pending` | Awaiting isolated schemas/readers, four workbooks, deterministic exports and visual QA. |
| Phase 4 — Audit and freeze | `Pending` | Awaiting ownership audit, semantic execution, reconciliation and Candidate release. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G19-P0-B1` | `Complete` | This row's containing commit | Reproduced clean detached caches twice; froze 25 dedicated FateRin tables plus 64 Fate configuration seeds at receipt digest `e75abcf…0cad`; verified base, upstream, source trees, ownership roots, named Currency Wars/RtBattle exclusions and no runtime lowering. `freeze-foundation.mjs --check` and `git diff --check` pass. Push and full remote equality are recorded by the containing commit publication before B2 starts. |
| `G19-P0-B2` | `Complete` | This row's containing commit | Generated and rechecked 177 files / 959,455 top-level rows at digest `48ebe846…f025`; corrected the planning oracle with 26 distinct `Fate` tables beside 25 `FateRin` tables; retained 31 Fate gameplay configs, 33 focused layouts, eight shared closure tables, two TextMaps and 48 identity indexes. Named exclusions cover 166 RtBattle paths, 23 Currency Wars Trait paths and two reward/talk tables. Push and remote equality are verified before B3 starts. |
| `G19-P0-B3` | `Complete` | This row's containing commit | Froze 1,904 exact-once obligations at `d3d6000e…684e`: 1,398 Fate-owned, 93 shared and 413 evidence-only; 1,478 DataReady and 13 conservative BattleEvent/BattleTarget joins retained as research. Added BattleArea join tables to the current 179-file inventory and proved exact-zero Blessing/Curio/Occurrence/shop/service/generic-currency pools. Manifest/inventory regeneration checks pass; publication is remote-verified before B4. |
| `G19-P0-B4` | `Complete` | This row's containing commit | Bound the 1,904-obligation manifest to canonical JSON/decimal/ID rules, common bilingual/provenance envelopes, field-level approximation, exact reconciliation identity and a closed semantic fact language. Froze 48 unique sheets across four clean-target openpyxl workbooks with Sora 0.3.0 authority at contract digest `13db3c9f…25b7`; focused and quick gates pass and publication is remote-verified before P1. |
| `G19-P1-B1` | `Complete` | This row's containing commit | Normalized 99 enabled rows at `0054237e…0b9b`: three areas, seven difficulties, ten phases, twelve battle zones, eight difficulty-progress rows, seven day-progress rows, six boards/18 nodes, four challenge fights, six story-fight locators and 18 map group/fight rows. Bilingual/provenance and lossless 64-bit/canonical-number checks pass; publication is remote-verified before B2. |
| `G19-P1-B2` | `Complete` | This row's containing commit | Normalized 85 participant records at `16b2b37b…5e8b`: eight classes, 21 handbook Masters, 21 Master rows, six avatars, ten Case Board Servants, nine teams, six owners and two Master config digests. 83 are enabled and two description rows evidence-only; no name-based uniqueness/loadout inference. Focused/quick gates and remote publication pass. |
| `G19-P1-B3` | `Complete` | This row's containing commit | Normalized 172 enabled Noble Phantasm/catalog rows at `7f7c6ef2…2223`: 34 core identities, 107 FateRin configs, three rarities, five tags, twelve keywords, four decks and seven recommendations. Core/config identities remain distinct and source-linked; focused/quick gates and remote publication pass. |
| `G19-P1-B4` | `Complete` | This row's containing commit | Normalized 671 enabled effect rows at `38530013…b378`: 51 buffs, twelve slots, 383 Fate MazeBuff rows, 141 statuses, 64 trait buffs and twenty challenge selections. Parameter vectors are canonical and definition/state families stay separate; hidden target/order/stack/teardown semantics remain fixture-bound. Focused/quick gates and remote publication pass. |
| `G19-P1-B5` | `Complete` | This row's containing commit | Normalized 223 enabled resource rows at `1cf7ed0c…fcc7`: 70 Command Spells/Reiju, 60 affixes, 71 common/client constants and 22 Reiju program digests. Definitions/affixes/constants/program identities remain separate; hidden settlement/order/reroll timing stays fixture-bound. Focused/quick gates and remote publication pass. |
| `G19-P1-B6` | `Complete` | This row's containing commit | Normalized 137 enabled progression rows at `9c34bfff…bedf`: 71 affixes, thirty experience steps, nineteen traits, four levels, six initial owner/Noble-Phantasm bindings and seven trait program digests. Definition/progression/loadout identities stay separate and carry/reset timing remains fixture-bound. Focused/quick gates and remote publication pass. |
| `G19-P1-B7` | `Complete` | This row's containing commit | Normalized 418 rows at `4f62f92e…768b`: seven enabled monster pools and 411 explicit evidence-only broadcast/talk/display/mission/reward/layout locators. Phase 1 closes all 1,805 direct rows (1,392 enabled + 413 evidence-only); no prose, rewards, assets or programs enter mechanics. Focused/quick gates and remote publication pass. |
| `G19-P2-B1` | `Complete` | This row's containing commit | Materialized six `0/0/0` selector-closure audits at `e2107a08…7de0` for Blessing, Curio, Occurrence, Shop, Service and generic run currency. Fate-owned positive families remain distinct; focused/quick gates and remote publication pass. |
| `G19-P2-B2` | `Complete` | This row's containing commit | Normalized 67 shared bindings at `0cd67f04…674a`: 18 BattleAreas, thirteen unified configs, 23 MazeBuffs, two BattleEvents and eleven BattleTargets. 54 typed/direct rows are DataReady; thirteen scalar event/target matches remain bounded research for P2-B5. Focused/quick gates and remote publication pass. |
| `G19-P2-B3` | `Complete` | This row's containing commit | Expanded eight FateActivity obligations into 112 DataReady rows at `b8db101d…24e5`: eight stages, 24 ordered waves and eighty ordered slots. Derived children retain parent provenance without enlarging the exact-once denominator; focused/quick gates and remote publication pass. |
| `G19-P2-B4` | `Complete` | This row's containing commit | Closed eighteen manifest rows at `07e489d2…2d5b` (five variants, five templates, eight skills) plus ten template-derived character/AI program receipts at `fdbbfb0c…6372`; 15 AnimEvent paths are presentation exclusions and no typed separate ability-program path exists. Primary inventory remains stable; focused/quick gates and remote publication pass. |
| `G19-P2-B5` | `Complete` | This row's containing commit | Assembled 17 files / 2,018 normalized records at pack digest `ae040b74…ecc3`; coverage is 1,904/1,904 with 1,491 eligible DataReady, 413 evidence-only, 13 explicit replaceable policies and zero unresolved. Generated 1,914 source receipts, 56 family fixtures and 11 peer reconciliation receipts (three concurrent peers deferred to merge). Focused/quick gates and remote publication pass. |
| `G19-P3-B1` | `Complete` | This row's containing commit | Added fourteen isolated activity Sora tables containing 154 exact-once rows: one derived Candidate profile plus areas, difficulties, phases, zones, progress, boards, nodes, participants, teams, owners, traits, levels and unlocks. The generated schema binds only the dedicated workbook, preserves the normalized bilingual/provenance envelope as strings and rejects empty or duplicate stable IDs. Runtime loading remains disabled. |
| `G19-P3-B2` | `Complete` | This row's containing commit | Added fourteen binding Sora tables with 597 exact-once rows for Masters, Servants, Noble Phantasms and levels, rarity/tag/keyword catalogs, decks/recommendations, Command Spells/affixes, resources and typed rule/lifecycle binding receipts. Together with B1 the isolated project exposes 28 non-empty tables / 751 rows with no duplicate stable key; opaque programs remain digests and are not lowered into runtime. |
| `G19-P3-B3` | `Complete` | This row's containing commit | Added thirteen combat Sora tables with 849 exact-once rows spanning stage/area/encounter graphs, waves/slots, enemy identities/program receipts, status/buff/MazeBuff definitions and bounded BattleEvent/BattleTarget bindings. The cumulative 41 gameplay tables contain 1,600 unique stable keys; event/target policies remain reference identities without invented operations. |
| `G19-P3-B4` | `Complete` | This row's containing commit | Added seven review tables: 1,914 source receipts, 419 content/zero-pool audit rows, 1,904 coverage receipts, 13 policies, 11 reconciliation receipts, 56 fixtures and 17 pack-file rows. The complete project has 48 non-empty tables / 5,934 unique workbook rows. Pinned Sora 0.3.0 generated four canonical templates, one schema lock and 50 isolated Rust reader files; independent clean regeneration is byte-identical at tree `96478ee7…77c2`. No generated reader is imported by runtime. |
| `G19-P3-B5` | `Complete` | This row's containing commit | Authored all four clean-target workbooks with pinned `openpyxl==3.1.5`: 48 sheets / 5,934 rows. Every sheet preserves the seven Sora metadata rows and exact schema order, uses frozen panes, filters, bounded widths, semantic validations and policy/evidence conditional formatting. All cells round-trip exactly, formulas are forbidden and two independent generations are byte-identical to the committed targets at digests `9dde5a7e…e1fe`, `c191e548…31f5`, `e0536821…d30f` and `dfa59026…5ec`. |
| `G19-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review. |
| `G19-P4-B1` | `Pending` | — | Audit exact-once coverage, selectors, ownership, provenance and exclusions. |
| `G19-P4-B2` | `Pending` | — | Execute semantic fixtures and replacement checks. |
| `G19-P4-B3` | `Pending` | — | Reconcile overlaps and run full regeneration/clean-checkout acceptance. |
| `G19-P4-B4` | `Pending` | — | Freeze Candidate release evidence and completion snapshot. |

## Frozen counters

The frozen denominator is 1,904 exact-once obligations: 1,398 Fate-owned, 93
shared and 413 evidence-only. Current disposition is 1,478 `DataReady`, 413
`EvidenceOnly` and 13 `ResearchRequired`. Blessing, Curio, Occurrence, Shop,
Service and generic run-currency families have generated selector-closure zero
proofs. Never reduce this denominator to make later coverage pass.

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G19-R01` | `ResolvedNarrowed` | Which Fate/FateRin/shared tables, config programs, stages and enemy files close the focused inventory? The current 179-file inventory includes 51 dedicated tables, 64 configs, ten shared join tables, TextMaps and identity indexes; P2 retains transitive BattleEvent/enemy-program work. | P0-B2/P0-B3 |
| `G19-R02` | `Resolved` | Which selectors prove released/permanent Fate/Star Rail Night membership and exclude Currency Wars/RtBattle/unrelated rows? Direct Fate tables/configs plus typed closure own the pack; 166 RtBattle and 23 Currency Wars Trait paths remain named exclusions. | P0-B3 |
| `G19-R03` | `Open` | What exact graph, Case Board, day/progress and unlock lifecycle is mechanical? | P1-B1 |
| `G19-R04` | `Open` | What are exact Master/Servant/team/trial/loadout uniqueness and invalidation scopes? | P1-B2 |
| `G19-R05` | `Open` | What are every Mystic Code candidate, acquisition, cost, target, effect, repeat, upgrade and teardown rule? | P1-B3–B4 |
| `G19-R06` | `Open` | How do magical energy and Command Spell/Reiju choices, rerolls and resources settle? | P1-B5 |
| `G19-R07` | `Open` | How do progression, traits/affixes, owner/init loadouts and state carry compose? | P1-B6 |
| `G19-R08` | `Open` | Which story/map/Infinite Trial fights, affixes, objectives and retry/settlement rules are enabled? | P1-B7/P2-B2 |
| `G19-R09` | `Open` | Which StageConfig waves, enemies, AI, skills, statuses and abilities define every fight? | P2-B3–B4 |
| `G19-R10` | `ResolvedFixtureBound` | Which hidden ordering, timing, weights, caps, rounding and fallbacks remain unavailable? Thirteen BattleEvent/BattleTarget operation meanings use `IdentityOnlyNoOperationLowering`, each with alternatives, fixture and released-evidence replacement condition. | P2-B5/P4-B2 |

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote verification | — |
| Reference bundle | — |
| Coverage | Pending G19-P0-B3 |
| Runtime profile | Unreleased |
