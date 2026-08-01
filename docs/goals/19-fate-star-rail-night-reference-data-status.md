# Goal 19 Status — Fate/Star Rail Night Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `fate-star-rail-night-reference-v1` |
| State | `Complete` — Candidate reference data; runtime unreleased |
| Active phase | Complete |
| Active batch | — |
| Next unblocked batch | — |
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
| Phase 2 — Encounters and complete pack | `Complete` | The 17-file pack accounts 1,904/1,904 obligations, 1,491 eligible DataReady rows, 413 evidence-only rows, 13 replaceable policies, 58 fixtures and zero unresolved records. |
| Phase 3 — Excel and Sora | `Complete` | 48 non-empty Sora tables / 5,936 rows, four byte-stable openpyxl workbooks, a byte-stable binary/debug export loaded through all generated readers, and every-sheet visual QA are complete. |
| Phase 4 — Audit and freeze | `Complete` | Exact-once ownership, semantic fixtures, replacement boundaries and immutable peer manifests pass. The terminal verifier regenerates four workbooks and the 48-table Sora release byte-identically; Candidate evidence freezes all counters, digests and 25 prerequisite batch commits. |

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
| `G19-P2-B5` | `Complete` | This row's containing commit | Initially assembled 17 files / 2,018 normalized records with 56 enabled-family fixtures; coverage is 1,904/1,904 with 1,491 eligible DataReady, 413 evidence-only, 13 explicit replaceable policies and zero unresolved. P4-B2 later exposed and closed two missing policy-bound fixture links, producing the current 58-fixture pack at digest `3a931ae7…4018`. |
| `G19-P3-B1` | `Complete` | This row's containing commit | Added fourteen isolated activity Sora tables containing 154 exact-once rows: one derived Candidate profile plus areas, difficulties, phases, zones, progress, boards, nodes, participants, teams, owners, traits, levels and unlocks. The generated schema binds only the dedicated workbook, preserves the normalized bilingual/provenance envelope as strings and rejects empty or duplicate stable IDs. Runtime loading remains disabled. |
| `G19-P3-B2` | `Complete` | This row's containing commit | Added fourteen binding Sora tables with 597 exact-once rows for Masters, Servants, Noble Phantasms and levels, rarity/tag/keyword catalogs, decks/recommendations, Command Spells/affixes, resources and typed rule/lifecycle binding receipts. Together with B1 the isolated project exposes 28 non-empty tables / 751 rows with no duplicate stable key; opaque programs remain digests and are not lowered into runtime. |
| `G19-P3-B3` | `Complete` | This row's containing commit | Added thirteen combat Sora tables with 849 exact-once rows spanning stage/area/encounter graphs, waves/slots, enemy identities/program receipts, status/buff/MazeBuff definitions and bounded BattleEvent/BattleTarget bindings. The cumulative 41 gameplay tables contain 1,600 unique stable keys; event/target policies remain reference identities without invented operations. |
| `G19-P3-B4` | `Complete` | This row's containing commit | Added seven review tables and the 48-table Sora foundation. P4-B2's policy-fixture correction updates the current review denominator to 58 fixtures and the complete workbook denominator to 5,936 rows without changing the 48-table schema, four templates or 50 isolated readers. No generated reader is imported by runtime. |
| `G19-P3-B5` | `Complete` | This row's containing commit | Authored four clean-target workbooks with pinned `openpyxl==3.1.5`, exact Sora metadata, filters/frozen panes/validations and no formulas. P4-B2 regenerated the current 48 sheets / 5,936 rows twice byte-identically at digests `bb56d38b…74ea`, `c191e548…31f5`, `e0536821…d30f` and `99d68277…67e8`. |
| `G19-P3-B6` | `Complete` | This row's containing commit | Exported all 48 tables and completed generated-reader plus every-sheet visual QA. P4-B2 regenerated the current 5,936-row bundle at `f2897da1…336` / tree `47179254…5df`; the standalone loader parses every table and the corrected 144-band visual evidence remains `PassedHumanInspection` with zero severe defect. |
| `G19-P4-B1` | `Complete` | This row's containing commit | Independently audited 1,904 unique manifest obligations against 1,904 unique coverage receipts, 1,491 DataReady + 413 EvidenceOnly dispositions, thirteen explicit policies and zero unresolved rows. All 2,018 normalized rows have unique stable IDs, bilingual names/summaries and exact provenance; all 1,914 source receipts are unique. Six selector pools remain exact zero, RtBattle/GridFight exclusions do not leak and runtime crates contain no Fate reference path. |
| `G19-P4-B2` | `Complete` | This row's containing commit | Executed 58 fixtures / 118 source-backed assertions and verified all thirteen `IdentityOnlyNoOperationLowering` policies, rejected alternatives and released-evidence replacement conditions. The audit exposed two missing policy fixture links for BattleEvent/BattleTarget; added disabled ResearchRequired boundary fixtures, regenerated the 17-file pack, workbooks, Sora bundle/debug export and 144-band visual evidence, then reran byte-stability and generated-reader checks. |
| `G19-P4-B3` | `Complete` | This row's containing commit | Locked and reverified the three concurrent frozen manifests at commits `6b30afec`, `50fa7e37` and `f9f70e20`. Exact path+locator+digest comparison yields zero exact shared receipts and zero same-locator digest conflicts, so no definition is copied or silently unified. Regenerated the pack (`59bcb142…171d`), 5,936-row workbooks, Sora export and 144-band visual evidence. After recording missing Sora archive and default-Python/openpyxl prerequisites, the corrected full gate passed 28 generated checks, Clippy and 33 workspace harnesses in 451.0 seconds. |
| `G19-P4-B4` | `Complete` | This row's containing commit | Froze machine-verifiable Candidate evidence at `evidence/fate-star-rail-night-reference-v1/release/release-evidence.json` (`a5097123…6fbf`), pinning 1,904 obligations, 2,018 normalized records, 1,914 sources, thirteen policies, 58 fixtures, four workbooks, 144 visual bands, 48 Sora tables / 5,936 rows and zero runtime profiles. The ordered terminal verifier rechecked pack/semantic/peer evidence, regenerated all workbooks and Sora artifacts byte-identically and loaded every generated reader. The containing commit must be pushed and verified by exact local/tracking/remote equality. |

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
| `G19-R03` | `ResolvedReferenceBound` | The exact graph, Case Boards, nodes, phases, zones, difficulty/day progress and authored unlock locators are normalized; unavailable runtime lifecycle meaning remains outside this reference-only Goal. | P1-B1/P4-B1 |
| `G19-R04` | `ResolvedReferenceBound` | Master, Servant, avatar, team, owner and initial-loadout identities are normalized without inventing name-based uniqueness or invalidation behavior. | P1-B2/P4-B1 |
| `G19-R05` | `ResolvedFixtureBound` | Noble Phantasm/Conceptual Mystic Code identities, configs, catalogs, decks, effects and selection facts are exact; hidden target/order/stack/teardown operations remain explicit fixture-bound boundaries. | P1-B3–B4/P4-B2 |
| `G19-R06` | `ResolvedFixtureBound` | Command Spell/Reiju definitions, affixes, constants and program receipts are normalized; hidden settlement/order/reroll timing is explicitly not lowered. | P1-B5/P4-B2 |
| `G19-R07` | `ResolvedFixtureBound` | Affixes, experience, traits, levels, initial owner/loadout bindings and program receipts are normalized; carry/reset timing remains a declared runtime boundary. | P1-B6/P4-B2 |
| `G19-R08` | `ResolvedReferenceBound` | Enabled challenge/story/map/Infinite Trial fight locators and monster pools are separated from 411 evidence-only presentation/reward rows; no reward or story payload enters mechanics. | P1-B7/P2-B2/P4-B1 |
| `G19-R09` | `ResolvedReferenceBound` | Eight stages, 24 waves, 80 slots, variants, templates, skills and ten enemy-program receipts close every selected encounter without inventing a separate ability-program path. | P2-B3–B4/P4-B1 |
| `G19-R10` | `ResolvedFixtureBound` | Which hidden ordering, timing, weights, caps, rounding and fallbacks remain unavailable? Thirteen BattleEvent/BattleTarget operation meanings use `IdentityOnlyNoOperationLowering`, each with alternatives, fixture and released-evidence replacement condition. | P2-B5/P4-B2 |

## Completion record

| Field | Value |
|---|---|
| Final state | `Complete` — Candidate reference data; runtime unreleased |
| Completion commit | This row's containing `G19-P4-B4` commit |
| Remote verification | Required exact local/tracking/`git ls-remote` equality for the containing commit |
| Reference bundle | `content-reference/fate-star-rail-night-v1/`; full tree SHA-256 `edfd1fd99eac92b89e78fffbafe2fd9e4f1fcefc7481bafbea583b80c797e68f` |
| Coverage | 1,904/1,904 accounted; 1,491 eligible DataReady, 413 EvidenceOnly, thirteen policy-bound, zero unresolved |
| Workbooks | Four byte-identical openpyxl workbooks; 48 sheets / 5,936 rows; 144 visual-review bands passed |
| Sora release | 48 tables / 5,936 independently loaded rows; bundle SHA-256 `f2897da1190ebfe5d6634982382b1bcd5eadcda50b2a050ef1be247b78343336` |
| Release evidence | `evidence/fate-star-rail-night-reference-v1/release/release-evidence.json`; SHA-256 `a5097123d4f50062b5e2e02da024216cd89850b9f65c97c3f74d8eda4b266fbf` |
| Full gate | At pushed `6931110d`: 28 generated checks, Clippy and 33 workspace harnesses passed in 451.0 seconds |
| Clean checkout | Detached pushed `6931110d` passed focused Candidate checks and the quick repository gate; tracked status remained clean |
| Runtime profile | `Unreleased`; zero runtime-enabled profiles and no runtime import |
