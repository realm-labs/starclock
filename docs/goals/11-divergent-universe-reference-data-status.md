# Goal 11 Status — Divergent Universe Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `divergent-universe-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Unique mode systems |
| Active batch | None |
| Next unblocked batch | `G11-P1-B5` after `G11-P1-B4` is pushed and remotely verified |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Source-cache reproduction | `G11-P0-B1`: both fixed caches materialized twice in an isolated target from the clean connected object cache; exact detached HEAD, origin, worktree cleanliness, connectivity and the 64-table/direct-entry seed verified |
| Starting source oracle | 64 `RogueTourn*` tables, 3 direct ability programs plus layout companions, CHS/EN TextMaps, StageConfig and transitive config/shared-source closure |
| Focused inventory | 2,684 pinned files: 2,675 `turnbasedgamedata` and 9 `StarRailRes` |
| Content manifest | 6,215 obligations in 50 categories: 4,507 `DivergentUniverse`, 1 proven `Shared` and 1,707 fail-closed `SharedCandidate` records |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal11-divergent-universe-reference` |
| Branch base | `db5268bbe46e36739f51824967458e2987d61fc5` (`G10-SETUP`; excludes later Goal 10 data batches) |
| Parallel condition | Separate branch/worktree and isolated Goal 11 artifacts while Goals 07 through 10 are active |
| Publication policy | Push each completed batch commit and verify the remote branch commit before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Complete` | Goal 03, both pinned caches, the 2,684-file focused inventory, 6,215 exact row/source obligations, 80 normalized families, three isolated workbooks, 25 semantic fixture families, Goal 08/09/10 checkpoints, Candidate-only scope and Sora authority are machine-frozen. |
| Phase 1 — Unique mode systems | `InProgress` | Version 4.4 entry/module, stage closure, policy-bound carry/reset, Arithmetic Mapping, all 80 current Equations and all 414 current Blessings/828 levels are normalized. Curios/Grand Miracles/Hex, Golden Blood/Titan, protocols, services and progression remain. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool ownership, Blessings, Curios, Occurrences, services, Adventure outcomes and encounters. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership reconciliation, semantic fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G11-P0-B1` | `Complete` | `ab6d3ed8131463a4c99ce337845502895ece6d53` | Froze foundation `9f8a3e33…144b7`, Goal 03 commit/tree and preserved bundle digests, the two Version 4.4 source revisions, 64 inherited `RogueTourn` seed rows, 29 batches, Candidate-only scope, Excel/openpyxl/pinned Sora 0.3.0 authority and six isolated roots. Ownership checkpoints: Goal 08 local committed `c283c7f1…55a` (7,913 rows: 7,199 mode-owned/714 shared), Goal 09 remote-backed `d5d261a3…c46` (6,963: 6,305/658), Goal 10 remote-backed `a2e64e1d…94d` (5,377: 5,243/134). Fresh GitLab reproduction failed once with HTTP/2 `PROTOCOL_ERROR`; a bounded HTTP/1.1 attempt resolved the commit but stalled during promisor checkout. The accepted isolated object-cache materialization ran twice; the focused verifier, `git diff --check`, batch-set check and quick repository gate pass; no fresh-network success is claimed. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `b0cd3cb9..ab6d3ed8 HEAD -> codex/goal11-divergent-universe-reference`; `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `ab6d3ed8131463a4c99ce337845502895ece6d53` before P0-B2 began. |
| `G11-P0-B2` | `Complete` | `f202c1bd0769922be394d4983dc4d0f0f3121779` | Generated `source-inventory.json` (`60d9a9cd…6dc4`, 1,028,567 bytes, 2,684 files): all 2,646 Goal 03 source files plus 29 focused StageConfig/TextMap/Tourn configuration entries and nine bilingual StarRailRes indexes. The closure contains 64 `RogueTourn` tables, six direct ability/layout files, 478 occurrence graphs, 159 candidate NPC graphs, three service graphs, 13 Adventure graphs and nine Maze graphs; 125 named other-mode files and three mode test/GodMode files remain fail-closed exclusion evidence. Raw Git blob hashing avoids checkout-EOL variance; prefixes and file families grant no row ownership. Byte-identical regeneration and the inventory verifier pass. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `ab6d3ed8..f202c1bd HEAD -> codex/goal11-divergent-universe-reference`; local and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `f202c1bd0769922be394d4983dc4d0f0f3121779` before P0-B3 began. |
| `G11-P0-B3` | `Complete` | `982af8887fdd9ba29f1a323efc0ff5f6595ba411` | Generated and rechecked `content-manifest.json` (`5cbfa748…c13e`, 2,669,145 bytes): 6,215 obligations in 50 categories, split into 4,507 `DivergentUniverse`, one proven `Shared` and 1,707 fail-closed `SharedCandidate` records. The exact Version 4.4 entry selects `TournRogue`, `Tourn3`, module `6002201`, main tournament 3 and sub-tournament 1. Twenty-eight current areas close to 22 difficulties and 11 layers; 13 finish rows explicitly test mode 3. No `Tourn3` room row or matching layer-room row exists, so 848 `Tourn2` rooms remain candidate obligations pending P1-B1 stage/config proof rather than being promoted by name or ID shape. The manifest also freezes 414 Blessings/828 levels under eight active types, 80 Equations, 235 Curio states/179 non-null handbook identities, 17 Grand Miracles, all Golden Blood/Titan direct rows, 118 Occurrences, 97 referenced current variants, 23 current service NPCs, 669 mechanic source files and 25 semantic fixture families. There are 666 explicitly historical module/Tourn1/Tourn2 rows, 128 named other-mode/test source files and 35 presentation/account source files retained as exclusion evidence. Manifest/foundation/inventory verifiers, deterministic regeneration, `git diff --check`, batch-set/link checks and the quick repository gate pass. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `f202c1bd..982af888 HEAD -> codex/goal11-divergent-universe-reference`; local and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `982af8887fdd9ba29f1a323efc0ff5f6595ba411` before P0-B4 began. |
| `G11-P0-B4` | `Complete` | `06470c6d8e5f2eabca9ae19e6c444b6b4dc8ef57` | Generated and rechecked 80 normalized file families (`5d5da896…2a90`, 35,988 bytes), the three-workbook Excel/openpyxl/Sora contract (`76a6ecef…3019`, 6,486 bytes) and 25 semantic fixture families (`e61ea6cd…c865`, 7,676 bytes). Every common row requires bilingual independent mechanical summaries, explicit ownership/coverage/evidence, ordered row sources and sorted tags; source refs include revision, game version, locator, digest, evidence and mechanism quality. Canonical decimals are strings and bytes are UTF-8/NFC/LF/two-space JSON. The 80 families partition exactly once across `DivergentUniverse.xlsx` (58), `DivergentUniverseBindings.xlsx` (12) and `DivergentUniverseReview.xlsx` (10); Sora project/generated-reader paths are isolated. Reconciliation joins source path, row locator and evidence digest against Goal 08/09/10 checkpoints; conflicts block instead of mutating another Goal. Shared candidates cannot become normalized `Shared` rows without an exact promotion receipt. Contract/manifest verifiers, byte-identical regeneration, `git diff --check`, batch-set/link checks and the quick repository gate pass. The Phase 0 boundary `node tools/repository-check/run.mjs --full --with-source-cache` reaches Goal 06 and repeats its immutable `Cargo.lock baseline differs` failure; no Goal 11 input causes that historical boundary. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `982af888..06470c6d HEAD -> codex/goal11-divergent-universe-reference`; local and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `06470c6d8e5f2eabca9ae19e6c444b6b4dc8ef57` before P1-B1 began. |
| `G11-P1-B1` | `Complete` | `9adccc92581bf81734b5ec7d9350e02fe87daa7e` | Generated and rechecked 1,050 normalized rows in eleven files (digest `ca3becfb…9bb8`): one “Divergent Universe: Arcadian Chronicles” profile, one exact module, two entries, 13 Tourn3 finish conditions, 28 areas (2 Guide/13 Formal/13 WeekChallenge), 13 Cyclical area bindings, 22 referenced difficulties, 11 referenced layers, zero matching layer-room rows, 848 room candidates and 111 stage/carry/reset rules. Entry `105` selects `TournRogue` and module `6002201`; the module resolves main tournament 3/sub-tournament 1. Every source-backed row is bilingual and carries a manifest-matching source path, locator and digest. The snapshot has no `Tourn3` room rows and no layer-room rows for the selected layers, so all `Tourn2` rooms remain `Shared`/`Cataloged` `UnprovenSharedCandidate` records with empty offered pools; none is promoted by ID shape or table prefix. Ordered area-layer flow and field-level carry/reset are explicit replaceable `ProjectPolicy`, not observed parity, and no runtime code is added. Flow/contract/manifest verifiers, byte-identical regeneration, `git diff --check` and the quick repository gate pass. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `06470c6d..9adccc92 HEAD -> codex/goal11-divergent-universe-reference`; local and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `9adccc92581bf81734b5ec7d9350e02fe87daa7e` before P1-B2 began. |
| `G11-P1-B2` | `Complete` | `071ae54b43bab9507748bf91849b87e46c7f803d` | Generated and rechecked 186 rows in three files (digest `08c5cc8c…89d4`), closing all 258 manifest receipts: 84 exact build-reference eligibility rows, 95 role/build rows containing 79 opaque special-avatar bindings and all 95 resolved `RogueMazeBuff` role contributions, plus seven mapping lifecycle rules. Exact released CHS/EN text proves mode-only scope, below-cap character leveling, inactive/below-required Trace activation/raising and below-required total Relic enhancement replacement; an already-sufficient field is preserved. The source names Light Cone mapping but publishes neither its predicate nor temporary identity, and the three catalogs intentionally have different stable-ID sets. Four role locators (`1014`, `1015`, `1508`, `1509`) have no released `AvatarConfig` identity and remain `Cataloged` without a character claim; 91 resolved role/build rows are `DataReady`. Opaque per-avatar temporary Trace/Light Cone/Relic loadouts stay `Unspecified`. Refresh at run entry/accepted party change and teardown without account mutation are explicit replaceable `ProjectPolicy`; no runtime lowering is added. Mapping/flow/contract/manifest verifiers, byte-identical regeneration, `git diff --check` and the quick repository gate pass. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `9adccc92..071ae54b HEAD -> codex/goal11-divergent-universe-reference`; local and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `071ae54b43bab9507748bf91849b87e46c7f803d` before P1-B3 began. |
| `G11-P1-B3` | `Complete` | `54346bf26f293301debbb0f904cee20f9cc557ee` | Generated and rechecked 569 rows in eight files (digest `4f898403…2614`), closing all 330 manifest receipts: 80 Tourn3 Equations, 80 exact Path-count recipes, four categories, 136 fail-closed offer locators, 80 progress definitions, 160 expansion states, 25 keyword effects with nine parameter bindings and four transition rules. Every Equation resolves its display and `RogueMazeBuff`, retains bilingual name, category, main/sub Path IDs/counts, binding/effect locators and handbook visibility, and excludes story payload. The exact distribution is 8 PathEcho/32 Rare/24 Epic/16 Legendary across eight current Path types. Twenty-three keywords and eight parameter rows bind a current Path; other direct rows remain catalog evidence. `RogueTournFormulaRandom` publishes only `RandomID`, so candidate lists, weights, consumers, draw counts and fallback remain `Unspecified`. Enhanced/rewrite contribution is deferred to P1-B4. Acquire/progress/replace/discard ordering and reject-without-mutation are explicit replaceable `ProjectPolicy`; no runtime lowering is added. Equation/mapping/flow/contract/manifest verifiers, byte-identical regeneration, `git diff --check` and the quick repository gate pass. Publication: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; result `071ae54b..54346bf2 HEAD -> codex/goal11-divergent-universe-reference`; local and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference` both resolved `54346bf26f293301debbb0f904cee20f9cc557ee` before P1-B4 began. |
| `G11-P1-B4` | `Complete` | This row's containing commit | Generated and rechecked 2,198 rows in six files (digest `985ba244…da79`), closing all 1,368 Blessing manifest obligations: eight exact active Paths, 414 identities, 828 exact base/enhanced levels, 118 Tourn3 groups, 416 rewrite rules and 414 Equation-contribution rows. Every level resolves its same-ID/same-level `RogueMazeBuff`, retaining bilingual name, category, authored tag, effects, binding and canonical parameters without runtime lowering. The identity distribution is 184 Common/161 Rare/69 Legendary; Path types 121–128 except inactive 123 have 54 each, and 129 has 36. Base/enhanced forms share one identity and each contributes one count to matching Equation recipes; enhancement preserves the contribution, while accepted identity replacement removes/adds the corresponding Path contribution. All 414 base-to-enhanced transitions are exact; two generic replacement/rewrite rules are replaceable `ProjectPolicy` and reject without mutation. Fifty-seven groups resolve entirely in the mode level catalog; 61 contain shared source IDs and stay `DeferredToP2B1` with 176 unresolved occurrences rather than inventing membership or weights. Blessing/Equation/mapping/flow/contract/manifest verifiers, byte-identical regeneration, `git diff --check` and the quick repository gate pass. Publication contract: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; push command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference`, requiring identical full commit IDs before P1-B5 starts. |
| `G11-P1-B5` | `Pending` | — | Import Curio/Weighted Curio eligibility and lifecycle plus Grand Miracle/Hex behavior. |
| `G11-P1-B6` | `Pending` | — | Import Golden Blood's Boons, Titan types/talents, choices, levels/states and contributions. |
| `G11-P1-B7` | `Pending` | — | Import Threshold Protocol, Astronomical Division, Star-Pioneer/Practice Mode, Cognoculi, unlocks, enemies and modifiers. |
| `G11-P1-B8` | `Pending` | — | Import workbench/gamble transforms, currencies, prices, offers and fallback policies. |
| `G11-P1-B9` | `Pending` | — | Import permanent talents/Inspiration Circuit, weekly modifiers, rooms/services and cross-battle contributions. |
| `G11-P2-B1` | `Pending` | — | Freeze reachable shared and mode-owned Blessing/Path/Equation-related pools. |
| `G11-P2-B2` | `Pending` | — | Import Curio/Weighted Curio identities, copies, states and offer-pool bindings. |
| `G11-P2-B3` | `Pending` | — | Import Occurrences, variants, choices, chests, conditions, costs and outcomes. |
| `G11-P2-B4` | `Pending` | — | Import currencies, services, workbench/gamble bindings and abstract Adventure outcomes. |
| `G11-P2-B5` | `Pending` | — | Import encounter groups, StageConfig waves, enemy variants, elite/boss pools and module/difficulty bindings. |
| `G11-P2-B6` | `Pending` | — | Generate rules, sources, coverage, research gaps, fixtures and pack index. |
| `G11-P3-B1` | `Pending` | — | Add isolated profile/module/stage/difficulty/protocol/Arithmetic Mapping Sora tables. |
| `G11-P3-B2` | `Pending` | — | Add Equation, Blessing, Curio/Grand Miracle/Hex, Golden Blood/Titan and lifecycle tables. |
| `G11-P3-B3` | `Pending` | — | Add progression, service, occurrence, Adventure, encounter and rule-binding tables. |
| `G11-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated schema locks/templates/readers. |
| `G11-P3-B5` | `Pending` | — | Generate all three complete isolated `openpyxl` workbooks and structural/semantic QA. |
| `G11-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G11-P4-B1` | `Pending` | — | Audit exact-once coverage, enabled modules, ownership, references, provenance and bilingual fields. |
| `G11-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G11-P4-B3` | `Pending` | — | Reconcile Goal 08/09/10 overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G11-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but unverified batch
remains `InProgress`.

The Goal package setup commit is identified as “this document's containing
commit” to avoid a recursive self-hash. Its push command and remote resolution
are reported in the setup handoff; `G11-P0-B1` records the full setup commit
and remote verification as immutable foundation evidence before any data
mutation.

## Frozen counters

Required counts are generated from `content-manifest.json`. Accounted and
DataReady remain zero until normalized rows close the frozen parent
obligations. Do not estimate denominators from Wiki totals, raw table sizes,
prefixes, modules or ID ranges.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/modules/entries/finish conditions | 17 | 17 | 17 | Exact `TournRogue`/`Tourn3`/`6002201` boundary; includes terminal conditions. |
| Areas/difficulties/layers/rooms | 921 | 921 | 62 | All parent obligations are represented; 848 room candidates and 11 derived room types remain non-DataReady pending exact stage/config promotion or exclusion. |
| Threshold Protocol/Astronomical Division | 17 | 0 | 0 | Star-Pioneer/Practice Mode and Cognoculi may expand normalized children only where simulation-visible. |
| Arithmetic Mappings | 258 | 258 | 252 | All receipts close; four missing public identities leave two mapping and four role obligations non-DataReady. |
| Equations/recipes/expansion states | 330 | 330 | 191 | Definitions/displays and current-Path keyword/parameter receipts are DataReady; 136 offer locators and three unselected-Path keyword/parameter receipts remain non-DataReady. |
| Divergent Blessings/levels/transforms | 1,360 | 1,360 | 1,299 | All identities/levels and 57 closed groups are DataReady; 61 groups await shared pool closure. |
| Curios/Weighted Curios/states | 700 | 0 | 0 | Includes eligibility, weighting, charges, destruction, repair and replacement. |
| Grand Miracles/Hex states | 74 | 0 | 0 | Includes character/Path/element eligibility, effects and lifecycle. |
| Golden Blood/Titan definitions | 132 | 0 | 0 | Includes Boons, Titan types/talents, choices, levels and contributions. |
| Workbench/gamble/services | 261 | 0 | 0 | Includes operations, currencies, prices, offered sets and deterministic fallback. |
| Permanent talents/unlocks/modifiers | 296 | 0 | 0 | Only simulation-visible progression, weekly and module effects may become DataReady. |
| Blessing/Path/shared content pools | 8 | 8 | 8 | All eight current active Path types are exact; shared group members still require P2-B1 receipts. |
| Occurrences/variants/choices | 215 | 0 | 0 | Presentation prose is excluded; mechanical graphs and outcomes are included. |
| Services/Adventure outcomes | 316 | 0 | 0 | Adventure input is an abstract offered result, not simulated action gameplay. |
| Encounter groups/waves/enemy slots | 877 | 0 | 0 | Parent obligations expand into exact StageConfig rows, identities and waves in P2-B5. |
| Mechanic rules | 669 | 0 | 0 | Reference source-file obligations only; no runtime executability claim. |
| Semantic fixtures | 25 | 0 | 0 | Non-shrinking minimum covering every distinct lifecycle and selection policy. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 11 as a complete reference-data package, not a runtime goal. | Divergent Universe research can proceed independently while Standard mechanics and other mode research continue. |
| 2026-07-29 | Use `divergent-universe` as the stable mode slug and reserve Goal 11. | Goal 10 already has a committed package and active isolated branch; no local or remote Goal 11 package/branch existed during the planning audit. |
| 2026-07-29 | Base the branch on `db5268bb…`, the committed Goal 10 setup package. | The base includes Goal 01–10 indexing but excludes Goal 10 data batches and every concurrent worktree's uncommitted changes. |
| 2026-07-29 | Inherit the pinned Version 4.4 structured snapshot and identity cross-check used by Goals 03 and 08–10. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require `G11-P0-B1` to reproduce both caches even though the planning audit found them clean at the pinned commits. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-29 | Treat all 64 `RogueTourn*` tables and known config/ability paths as an inventory seed, not the content denominator. | Test, display, reward and multi-module rows coexist with enabled mechanics; shared Rogue rows, TextMaps, StageConfig, enemies and transitive programs determine exact reachability. |
| 2026-07-29 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent mode goals must preserve isolated ownership ledgers and surface conflicts for merge coordination. |
| 2026-07-29 | Reuse shared stable IDs only after Divergent Universe reachability is proven. | Prefixes, module labels, matching names and adjacent IDs do not prove identical ownership, state or eligibility. |
| 2026-07-29 | Exclude story/presentation and account/collection/fitting rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-29 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |
| 2026-07-29 | Freeze Version 4.4 to `TournRogue`/`Tourn3`, activity module `6002201`, main tournament 3 and sub-tournament 1. | The resident activity row selects `6002201`, and the module table resolves that stable reference exactly; older module rows remain historical evidence. |
| 2026-07-29 | Keep all 848 `Tourn2` room rows as `SharedCandidate` obligations instead of reachable shared content. | The fixed snapshot has no `Tourn3` room rows or layer-room rows matching the 11 selected layers; only P1 stage/config closure may promote or exclude a room. |
| 2026-07-29 | Freeze 80 normalized families across three isolated workbooks and bind them to a private Sora 0.3.0 project/reader. | Keeps definitions, levels, states, variants, services, encounters, sources, coverage and fixtures typed and independently regenerable without touching shared or other-mode generated paths. |
| 2026-07-29 | Require a promotion receipt before any `SharedCandidate` manifest row becomes normalized `Shared` content. | Prevents conservative room and encounter obligations from being mistaken for proven Version 4.4 reachability. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G11-R01` | `ResolvedExact` | Which direct and transitive configuration, TextMap, StageConfig, enemy/wave, shared Rogue and ability files complete the 64-table seed inventory? | P0-B2 | Replaced by the 2,684-file raw-Git-blob inventory (`60d9a9cd…6dc4`); reopen only if P0-B3 discovers a referenced path absent from this fail-closed closure. |
| `G11-R02` | `ResolvedExact` | Which Version 4.4 module selectors separate enabled Divergent-owned, shared, evidence-only, historical/test and other-mode rows? | P0-B3 | Replaced by `content-manifest.json` (`5cbfa748…c13e`): exact `TournRogue`/`Tourn3`/`6002201` selection, transitive row closures, 666 historical rows and named fail-closed exclusions. Room reuse remains deliberately unpromoted under `G11-R03`. |
| `G11-R03` | `ResolvedPolicyBound` | What are the exact Ordinary/Cyclical area, difficulty, layer, room, transition, finish and carry/reset boundaries for each enabled module? | P1-B1 | Entry/module, 28 areas, 22 difficulties, 11 layers and 13 finish rows are exact. The fixed source has no current layer-room rows; ordered area-layer flow and carry/reset are replaceable `ProjectPolicy`. All 848 rooms remain unpromoted until P2-B5 supplies exact stage/config receipts. |
| `G11-R04` | `ResolvedPolicyBound` | How are temporary character, Trace, Light Cone and Relic mappings selected, refreshed and removed, including already-stronger account builds? | P1-B2 | Exact public text freezes mode-only below-threshold behavior and preservation of sufficient fields; all three structured catalogs and role buffs resolve. Opaque per-avatar loadouts, the Light Cone predicate and refresh checkpoints remain `Unspecified` or replaceable `ProjectPolicy`; four source locators have no released AvatarConfig identity. |
| `G11-R05` | `ResolvedPolicyBound` | How are Equations offered, rerolled, progressed, expanded, replaced and evaluated when Blessing ownership changes? | P1-B3–B4 | Definitions, recipes, base/enhanced identity contribution and ownership-change refresh are exact. RandomID membership/weights and service costs/timing remain `Unspecified`; acquire/replace/discard ordering and no-legal fallback use replaceable state-preserving `ProjectPolicy`. |
| `G11-R06` | `Open` | What are the exact Weighted Curio and Grand Miracle/Hex eligibility, weighting, activation, simultaneous-trigger and no-legal-target semantics? | P1-B5 / P2-B2 | Replace with source-backed pools/lifecycles or explicit deterministic policies for each unresolved field. |
| `G11-R07` | `Open` | How do Golden Blood's Boons, Titan types/talents and choices level, activate, stack, carry and contribute to battle? | P1-B6 | Replace with complete definitions, transition programs and one semantic fixture per distinct lifecycle/contribution. |
| `G11-R08` | `Open` | Which Threshold Protocol, Astronomical Division, Star-Pioneer/Practice Mode and Cognoculus rows change entry, enemy composition, combat numerics, unlocks or lifecycle rather than account rank/rewards? | P1-B7 | Replace with explicit simulation-visible classification and boundary fixtures; reward-only rows remain evidence-only. |
| `G11-R09` | `Open` | How do workbench/gamble operations price and choose Equation/Blessing/Curio inputs/outputs, caps, failures and replacements? | P1-B8 | Replace with source-backed operation programs and candidate-set fixtures or per-field replaceable policies. |
| `G11-R10` | `Open` | Which permanent talents, weekly/cyclical modifiers, unlocks, room marks and services alter a run or battle? | P1-B9 | Replace with manifest classification backed by explicit downstream references and simulation-visible effect evidence. |
| `G11-R11` | `Open` | Which Blessings, Curios, Occurrences, services and mode-specific copies are reachable in each enabled Version 4.4 pool? | P2-B1–B4 | Replace with exact selector/transitive-reference/stable-ID closures and 100% pool accounting. |
| `G11-R12` | `Open` | Which encounters, enemy variants, waves and bosses bind to each module/stage/difficulty, and which ability programs make them distinct? | P2-B5 | Replace with resolved StageConfig/wave/enemy/ability dossiers or a documented nonblocking boundary for unavailable released evidence. |
| `G11-R13` | `Open` | Which hidden weights, target orders, timing, caps, rounding and fallbacks remain unavailable after bounded research? | P2-B6 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete evidence-triggered replacement condition. |

## Terminal checklist

- [x] Exact enabled-module category manifests and denominators are frozen.
- [x] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership, module enablement and shared reachability are explicit and
      fail closed.
- [ ] Shared classifications reconcile with committed Goal 08/09/10 facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Stage flow, Arithmetic Mapping, Equations, Blessings, Curios/Grand
      Miracles/Hex, Golden Blood/Titan, protocols and services have complete
      semantic fixtures.
- [ ] Encounter identities, StageConfig rows, waves and boss bindings resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] All three complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and current Standard/Gold/Swarm/Unknowable/production
      bundle identities remain unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from its recorded remote
      branch at the recorded commit ID.
- [ ] Clean-checkout acceptance passes and `G11-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Divergent Universe reference bundle | — |
| Workbook semantic digest | — |
| Coverage | 6,215 frozen obligations; normalized accounting pending Phase 1–4 |
| Release evidence | — |
| Remaining required work | Divergent Universe runtime lowering, integration, handlers, controller/API exposure and seeded full runs belong to a later goal. |
