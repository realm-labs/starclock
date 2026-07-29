# Goal 10 Status — Unknowable Domain Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `unknowable-domain-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Unique mode systems |
| Active batch | None |
| Next unblocked batch | `G10-P1-B4` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-29: both caches clean at pinned commits; commit readability and connectivity checked; execution must reproduce in `G10-P0-B1` |
| Focused inventory | 2,684 pinned files: 2,675 `turnbasedgamedata` and 9 `StarRailRes` |
| Content manifest | 5,377 obligations in 43 categories; 5,243 Unknowable-owned and 134 shared |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal10-unknowable-domain-reference` |
| Parallel condition | Separate branch/worktree and isolated Goal 10 artifacts while Goals 07, 08 or 09 are active |
| Publication policy | Push each completed batch commit and verify the remote branch commit before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Complete` | Goal 03, both pinned caches, the 2,684-file inventory, 5,377 obligations, 65 normalized families, three isolated workbooks, evidence/canonical/reconciliation contracts and 24 fixture families are machine-frozen. |
| Phase 1 — Unique mode systems | `InProgress` | Profile/entry, 135 finish conditions, 13 areas, 97 difficulty rows, 32 layers, 176 positions, 1,518 rooms, policy-bounded flow/carry/reset rules, all four Extrapolation Alignments, 24 Scepters, 72 levels/activation rules and 216 lifecycle boundaries are normalized; later unique systems remain. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool ownership, Blessings, Curios, Occurrences, services, Adventure outcomes and encounters. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership reconciliation, semantic fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G10-P0-B1` | `Complete` | This row's containing commit | Froze foundation `270d016b…9407`, Goal 03 commit/tree and bundle digests, the Version 4.4 revisions, 32 inherited `RogueMagic` seed rows, 28 batches, Candidate-only scope, Excel/openpyxl/pinned Sora 0.3.0 authority and six isolated roots. The Goal 08 local-only checkpoint is `2f7b3ccf…fc5d` (7,913 obligations: 7,199 Gold-owned and 714 shared); the required remote-backed Goal 09 checkpoint is `1f9019a2…5ae2` (2,882 source records). `pwsh -File tools/content-reference/fetch-sources.ps1 -CacheRoot .cache/content-reference` could not run because `pwsh` is absent; the isolated POSIX fetcher reproduced clean detached caches repeatedly and the focused verifier, `git diff --check` and quick repository gate pass. `node tools/repository-check/run.mjs --full --with-source-cache` reaches the immutable Goal 06 contract before repeating its known `Cargo.lock baseline differs` failure. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P0-B2 starts. |
| `G10-P0-B2` | `Complete` | This row's containing commit | Generated and rechecked `source-inventory.json` (`20fad854…baa5`, 1,036,464 bytes): all 2,646 Goal 03 source files, plus 29 focused mode/StageConfig/TextMap entries and nine bilingual StarRailRes indexes. The 2,684-file closure contains 32 `RogueMagic` tables, 16 direct ability files, 14 Scepter battle events, three service graphs, six mechanical maze graphs and 57 Rogue260 NPC graphs; 141 named other-mode files remain fail-closed exclusion evidence. Raw Git blob hashing removes checkout-EOL variance; file families grant no content ownership. Foundation/inventory verifiers, `git diff --check` and the quick repository gate pass; the full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P0-B3 starts. |
| `G10-P0-B3` | `Complete` | This row's containing commit | Generated and rechecked `content-manifest.json` (`7416da58…a758`, 1,975,471 bytes): 5,377 exact obligations in 43 categories, split into 5,243 `UnknowableDomain` and 134 `Shared` records. Only explicit `MagicRogue`, type-260 selectors, direct references or inherited stable-ID closure grant reachability; 141 named other-mode files and 27 presentation/account sources remain fail-closed evidence. Curios and Occurrences use explicit type 260. The released snapshot exposes no MagicRogue/type-260 Blessing selector or RogueMagic-to-Blessing reference, so the Blessing denominator freezes at zero rather than inheriting a generic pool. Ultra-category Components freeze the 25 Decision Component candidates; StageConfig waves/enemy slots remain P2-B5 child obligations under 1,524 room/boss parents. Manifest/foundation/inventory verifiers, `git diff --check` and the quick repository gate pass; the full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P0-B4 starts. |
| `G10-P0-B4` | `Complete` | This row's containing commit | Froze 65 normalized file families (`ee8e0d37…8d18`), the three-workbook Excel/openpyxl/Sora contract (`65329ecf…c323`) and 24 non-shrinking semantic fixture families (`ff5e508b…38d6`). Every common row requires bilingual independent mechanical summaries, explicit ownership/coverage/evidence, ordered row sources and sorted tags; source refs include revision, game version, locator, digest, evidence and mechanism quality. Canonical decimals are strings and bytes are UTF-8/NFC/LF/two-space JSON. Each normalized family belongs to exactly one of `UnknowableDomain.xlsx`, `UnknowableDomainBindings.xlsx` or `UnknowableDomainReview.xlsx`; Sora project/generated-reader paths are isolated. Reconciliation joins source path, row locator and evidence digest against the optional local Goal 08 and required remote-ancestor Goal 09 checkpoints; conflicts block instead of mutating another Goal. Contract/manifest/foundation/inventory verifiers, `git diff --check` and the quick repository gate pass; the phase-boundary full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P1-B1 starts. |
| `G10-P1-B1` | `Complete` | This row's containing commit | Generated and rechecked eight normalized files with 2,028 rows (2,874,756 bytes; digest `82f212ab…e20a`): one Candidate profile, two exact `MagicRogue` entry rows, 135 finish conditions, 13 areas, 97 difficulty source rows, 32 layers, 176 ordered positions, 1,518 typed rooms and 54 flow/carry/reset rules. Every row is bilingual, `DataReady`, ownership-scoped and carries ordered row-level provenance with mechanism quality. Area layer order and the single area-601 extra-layer reference are exact; transition timing, initial resources, optional extra-layer eligibility and field-level carry/reset use replaceable `ordered-area-layer-flow-v1` `ProjectPolicy`. Source difficulty locators, layer-position room pools and room graph/encounter membership remain `Unspecified`; no ID-shape inference grants membership. All manifest parents close exactly once. Flow and all Phase 0 verifiers, `git diff --check` and the quick repository gate pass; the deferred full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P1-B2 starts. |
| `G10-P1-B2` | `Complete` | This row's containing commit | Generated and rechecked four Extrapolation Alignment rows (11,090 bytes; SHA-256 `b8628c54…2ecf`): Break, DoT, Follow-up and Ultimate close the four manifest obligations exactly once. Explicit style membership binds six Scepters to each Alignment, covering all 24 definitions exactly once; explicit area defaults cover all 13 areas, and only Ultimate is source-available without an unlock. The released selector tables do not publish a Component candidate pool, selection cardinality or direct battle-rule contribution, so those fields remain `Unspecified` or deferred to the source-backed Scepter/Component batches instead of being inferred from labels or ID shape. Every row is bilingual, `DataReady`, ownership-scoped and carries ordered row-level provenance with mechanism quality. Alignment and all prerequisite verifiers, `git diff --check` and the quick repository gate pass; the deferred full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; commit is this row's containing commit; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verification command `git rev-parse HEAD && git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, with identical full commit IDs required before P1-B3 starts. |
| `G10-P1-B3` | `Complete` | This row's containing commit | Generated and rechecked four normalized files with 384 rows (1,433,379 bytes; digest `a323efcc…ef89`): all 24 Scepters, 72 levels, 72 exact locked-Component bindings, 72 activation rules and 216 lifecycle boundaries. The released tables split definitions into 12 Charge and 12 Speed Scepters, four Alignments, four effect ranges and three slot layouts. Joined maze-buff rows and the exact ability binding prove per-level power, binding program, Charge gain/120 threshold or 100 Speed, nine trigger kinds, event creation, attack dispatch and damage-finished boundaries. Hidden target/simultaneous ordering, post-attack Charge reset, post-action value, next-cycle transition and teardown remain explicitly `Unspecified`; no runtime lowering is claimed. Every row is bilingual, `DataReady`, ownership-scoped and carries ordered row-level table/display/maze-buff/ability provenance with direct mechanism quality. Scepter and all prerequisite verifiers, `git diff --check` and the quick repository gate pass; the deferred full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; commit is this row's containing commit; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verification command `git rev-parse HEAD && git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, with identical full commit IDs required before P1-B4 starts. |
| `G10-P1-B4` | `Pending` | — | Import Components, levels, categories/types, shapes, compatible slots/ranges and effects. |
| `G10-P1-B5` | `Pending` | — | Import Decision Components, loadout validation, insertion/removal/replacement and fallback policies. |
| `G10-P1-B6` | `Pending` | — | Import synthesis, upgrades, rerolls/reforges, costs, ordering, caps and replacement. |
| `G10-P1-B7` | `Pending` | — | Import workbench/gamble services, currencies, prices, offered pools and lifecycle. |
| `G10-P1-B8` | `Pending` | — | Import Talents/unlocks, layer effects, maze buffs, difficulty/score inputs and rule contributions. |
| `G10-P2-B1` | `Pending` | — | Freeze reachable shared and mode-owned Blessing/enhanced/alignment pools. |
| `G10-P2-B2` | `Pending` | — | Import Curios, copies, states, charges, repair, replacement and pool rules. |
| `G10-P2-B3` | `Pending` | — | Import Occurrences, variants, choices, conditions, costs and outcomes. |
| `G10-P2-B4` | `Pending` | — | Import currencies, services, workbench/gamble bindings and abstract Adventure outcomes. |
| `G10-P2-B5` | `Pending` | — | Import encounter groups, StageConfig waves, enemy variants, elite/boss pools and difficulty bindings. |
| `G10-P2-B6` | `Pending` | — | Generate rules, sources, coverage, research gaps, fixtures and pack index. |
| `G10-P3-B1` | `Pending` | — | Add isolated profile/stage/difficulty/Alignment Sora tables. |
| `G10-P3-B2` | `Pending` | — | Add Scepter, activation/state, Component, slot/loadout and Decision Component tables. |
| `G10-P3-B3` | `Pending` | — | Add synthesis/upgrade/reforge, progression, workbench/gamble/service and rule-binding tables. |
| `G10-P3-B4` | `Pending` | — | Add content/encounter/evidence/coverage/reconciliation/fixture tables and isolated schema locks/templates/readers. |
| `G10-P3-B5` | `Pending` | — | Generate complete isolated `openpyxl` workbooks and structural/semantic QA. |
| `G10-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G10-P4-B1` | `Pending` | — | Audit exact-once coverage, ownership, references, provenance and bilingual fields. |
| `G10-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G10-P4-B3` | `Pending` | — | Reconcile Goal 08/09 overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G10-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but unverified batch
remains `InProgress`.

## Frozen counters

Required counts are generated from `content-manifest.json`. They are source
obligations: later normalized child rows may expand a category but may not
remove or silently merge an obligation.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/entries/finish conditions | 138 | 138 | 138 | One profile, two explicit `MagicRogue` entry rows and all 135 finish conditions; initial resources remain an explicit replaceable policy field. |
| Areas/difficulties/layers/rooms | 1,846 | 1,846 | 1,846 | Thirteen areas, six difficulty compositions, 91 drops, 32 layers, 176 layer positions, 1,518 rooms and ten room types; unavailable pool links remain `Unspecified`. |
| Extrapolation Alignments | 4 | 4 | 4 | Break, DoT, Follow-up and Ultimate close exactly once; 24 explicit style-to-Scepter bindings and 13 area defaults are preserved, while Component pools/cardinality remain fail-closed. |
| Scepters/levels/states | 168 | 168 | 168 | Twenty-four definitions, 72 levels and 72 locked-Component bindings close exactly once; normalized children add 72 activation rules and 216 lifecycle boundaries while hidden reset/order/teardown stays fail-closed. |
| Components/levels/effects | 668 | 0 | 0 | 109 definitions, 277 levels, two categories, three types and 277 referenced effect rows. |
| Decision Components/choices | 25 | 0 | 0 | The exact Ultra-category definition boundary; choice programs expand in P1-B5. |
| Loadouts/slots/insertion/replacement | 3 | 0 | 0 | Three distinct released Active/Attach/Passive slot-count layouts; transition policies expand as child rows. |
| Synthesis/upgrades/reforges | 5 | 0 | 0 | Five workbench function definitions are the source parents for compose, upgrade, reforge and shop behavior. |
| Workbench/gamble/services | 26 | 0 | 0 | Four workbenches, five functions, ten gamble groups and seven gamble units. |
| Talents/unlocks/layer/difficulty effects | 590 | 0 | 0 | Twenty-five Talent rows, 30 unlocks, one layer effect, 387 maze buffs, 14 common constants and 133 score inputs. |
| Blessings/enhanced levels | 0 | 0 | 0 | Fail-closed: no released MagicRogue/type-260 selector or RogueMagic-to-Blessing reference exists in the fixed snapshot. |
| Curios/states | 188 | 0 | 0 | Sixty explicit type-260 shared identities, 81 mode copy/state rows and 47 weighted groups. |
| Occurrences/variants/choices | 112 | 0 | 0 | Sixty-two explicit type-260 handbooks and 50 directly referenced RogueMagic NPC/progress variants. |
| Services/Adventure outcomes | 40 | 0 | 0 | Workbench/gamble parents, nine abstract Adventure outcomes and five non-Occurrence mode service/entry NPC graphs. |
| Encounter groups/waves/enemy slots | 1,524 | 0 | 0 | All 1,518 room and six displayed-boss parent obligations are frozen; P2-B5 attaches StageConfig waves and enemy slots. |
| Mechanic rules | 41 | 0 | 0 | Exact mode-named ability, battle-event, Adventure, maze, progression and service source files; no runtime executability claim. |
| Semantic fixtures | 24 | 0 | 0 | Non-shrinking minimum covering unique systems, lifecycle, ordering and no-legal-candidate fallback. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 10 as a complete reference-data package, not a runtime goal. | Unknowable Domain research can proceed independently while Standard mechanics and other mode research continue. |
| 2026-07-29 | Use `unknowable-domain` as the stable mode slug and reserve Goal 10. | Goal 09 is already committed and pushed; no local or remote Goal 10 package or branch existed during the planning audit. |
| 2026-07-29 | Inherit the pinned Version 4.4 structured snapshot and identity cross-check used by Goals 03, 08 and 09. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require the execution goal to reproduce both caches even though the planning audit found them clean at the pinned commits. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-29 | Treat the thirty-two `RogueMagic*` tables and known configuration files as an inventory seed, not the content denominator. | Shared Rogue rows, TextMaps, StageConfig, enemies, waves and transitive ability programs determine exact reachability and mechanics. |
| 2026-07-29 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent mode goals must preserve isolated ownership ledgers and surface conflicts for merge coordination. |
| 2026-07-29 | Reuse shared stable IDs only after Unknowable Domain reachability is proven. | Prefixes, matching names and adjacent IDs do not prove identical ownership, state or eligibility. |
| 2026-07-29 | Exclude story/presentation and account/score rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-29 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |
| 2026-07-29 | Treat the Goal 08 local commit as an optional informational checkpoint and the Goal 09 remote commit as a required ancestor checkpoint. | Goal 08 has no configured remote ref yet, while Goal 09 continues to advance in parallel; neither condition should fabricate membership or block an independently reproducible Goal 10 foundation. |
| 2026-07-29 | Add an isolated POSIX source fetcher and keep repository-pinned Sora 0.3.0 authoritative. | This host lacks `pwsh`, and its global `sora` is 0.2.0; neither host limitation changes the frozen sources or Phase 3 tool contract. |
| 2026-07-29 | Define the focused file closure as all Goal 03 source paths plus exact Unknowable configuration/Stage/TextMap additions and nine public bilingual indexes. | The inherited superset preserves every shared source and ability candidate while 29 explicit additions close mode-specific battle-event, service, maze, StageConfig and localization entry points without using prefixes as ownership proof. |
| 2026-07-29 | Hash source inventory records from raw Git blobs, not checkout files. | This makes generation byte-stable across checkout EOL policies while retaining exact revision/path provenance. |
| 2026-07-29 | Grant shared Curio/Occurrence reachability only through explicit type-260 membership and shared boss reachability only through direct area references. | These stable selectors/references prove mode reachability without relying on table prefixes, display names or numeric adjacency. |
| 2026-07-29 | Freeze the reachable Blessing denominator at zero until stronger released evidence supplies a selector or reference. | The fixed snapshot contains no MagicRogue/type-260 Blessing selector or RogueMagic-to-Blessing edge; Components are a distinct mode-owned upgrade pool and cannot justify importing shared Blessings. |
| 2026-07-29 | Treat the 25 Ultra-category Component definitions as Decision Component candidates. | `MagicUnitCategory=Ultra` is an explicit released discriminator; P1-B5 still owns choice eligibility, ordering, outcomes and fallback semantics. |
| 2026-07-29 | Freeze rooms and displayed bosses as encounter parent obligations and defer StageConfig wave/enemy expansion to P2-B5. | The source denominator stays stable while later normalized child rows can close exact waves and slots without inventing them during ownership classification. |
| 2026-07-29 | Freeze 65 normalized file families with a shared bilingual/evidence envelope and exact manifest-category mapping. | Separating parent source obligations from typed derived children prevents double counting while retaining complete provenance and stable-ID resolution. |
| 2026-07-29 | Require canonical decimal strings, UTF-8/NFC/LF bytes, deterministic key/array order and omission of absent optional fields. | JSON remains reproducible staging/debug data and cannot silently acquire float, locale, filesystem-order or dual-absence drift. |
| 2026-07-29 | Partition all authoring exactly once across three isolated workbooks and keep Sora 0.3.0 authoritative. | The requested workbooks separate core authoring, content bindings and review/evidence without sharing mutable sheets or generated paths with another mode. |
| 2026-07-29 | Reconcile shared rows by source path, row locator and evidence SHA-256, with conflicts becoming `Blocked`. | Goal 08/09 may continue independently; a receipt records disagreement without rewriting their manifests, workbooks or ownership decisions. |
| 2026-07-29 | Freeze 24 non-shrinking semantic review families with explicit RNG, ordering, fallback and approximation rules. | Reference fixtures make lifecycle and selection semantics auditable while explicitly making no runtime-executability claim. |
| 2026-07-29 | Preserve area difficulty-list values as source locators and leave normalized difficulty references unresolved. | The fixed snapshot defines six `DifficultyCompID` rows but does not publish a join from the area `DifficultyIDList` values; numerical resemblance cannot supply the missing edge. |
| 2026-07-29 | Keep every layer-position room pool and room graph/encounter binding `Unspecified`. | `RogueMagicLayerRoom` contains only layer and ordinal, while `RogueMagicRoom` contains only room ID and type; parsing the room ID shape would violate the explicit-reference membership rule. |
| 2026-07-29 | Use `ordered-area-layer-flow-v1` only as a replaceable `ProjectPolicy` for transition timing, optional extra-layer eligibility and carry/reset fields. | Area rows prove ordered layer lists and one extra-layer reference, but they do not publish lifecycle timing or state-field carry/reset order. |
| 2026-07-29 | Bind each Alignment only to its explicit six-Scepter style pool and its source-declared default areas. | The fixed selectors prove 24 style memberships and 13 defaults exactly once, but publish neither a Component candidate pool nor selection cardinality; those fields remain `Unspecified` pending stronger released evidence. |
| 2026-07-29 | Defer Alignment battle-rule contributions to the Scepter and Component rule records that actually define them. | Alignment names and display prose classify play style but do not provide an independent executable contribution or timing program. |
| 2026-07-29 | Join every Scepter level through `StaffMazeBuffID`, level and exact `InBattleBindingKey` before interpreting Charge or Speed. | The join proves 72 table rows, exact ability programs and localized parameters without assuming that Scepter IDs, buff IDs or ability names share a numeric shape; it also preserves the released 2007 binding exception. |
| 2026-07-29 | Freeze initialization, attack dispatch and damage-finished boundaries while leaving reset, next-cycle, simultaneous order, target order and teardown `Unspecified`. | Released tables, localized trigger text and the Staff ability program prove these boundaries but do not fully publish engine scheduling or cleanup semantics; inventing them would convert missing evidence into false parity. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G10-R01` | `Resolved` | Which shared Rogue tables, configuration programs, TextMap rows, StageConfig rows, enemy/wave records and transitive ability files complete the `RogueMagic` seed inventory? | P0-B2 | `source-inventory.json` freezes all 2,646 inherited Goal 03 source files, 29 exact focused additions and nine public index files; raw-blob regeneration is byte-identical and the inventory verifier closes every documented family. |
| `G10-R02` | `Resolved` | What exact selectors separate Unknowable-owned, shared, evidence-only and Standard/Gold/Swarm/Divergent rows? | P0-B3 | `content-manifest.json` freezes exact-once obligations with `MagicRogue`, type-260, direct-reference and stable-ID reachability; 141 named other-mode and 27 presentation/account source files fail closed as evidence only. |
| `G10-R03` | `Researching` | What is the exact area/layer/room ordering, carry/reset behavior and finish boundary for every released stage and difficulty? | P4-B2 | P1-B1 freezes exact area/layer order, room/finish rows and source locators. Replace `ordered-area-layer-flow-v1` and each `Unspecified` difficulty/room-pool binding when released engine flow/configuration or reproducible observations prove transition timing, extra-layer eligibility and field-level carry/reset. |
| `G10-R04` | `Researching` | How do Alignment selection, eligibility and candidate pools constrain Scepters, Components and battle contributions? | P4-B2 | P1-B2 freezes all four selectors, 24 exact Scepter bindings, 13 area defaults and unlock availability. Replace each `Unspecified` Component pool/cardinality field and deferred battle contribution only when a released selector/program or reproducible observation proves it, then execute one semantic fixture per Alignment and selection boundary. |
| `G10-R05` | `Researching` | What are the exact Scepter activation, charge gain/spend, speed/action ordering, simultaneous-trigger and teardown semantics? | P4-B2 | P1-B3 freezes 72 exact activation rules and 216 initialization/dispatch/finish boundaries. Replace each `Unspecified` post-attack reset, post-action value, next-cycle, target/simultaneous order and teardown field only with released engine programs or reproducible observations, then bind the stronger evidence to the affected semantic fixtures. |
| `G10-R06` | `Open` | How do Component shape, slot/range, type, style and locked-Component constraints determine legal loadouts and insertion/removal? | P1-B4–B5 | Replace with a complete legality matrix and positive/rejected semantic fixtures for every constraint class. |
| `G10-R07` | `Open` | What are the exact Decision Component offer, eligibility, ordering, repetition and no-legal-choice rules? | P1-B5 / P2-B3 | Replace with source-backed choice programs or an explicit deterministic policy and replacement trigger for each unresolved field. |
| `G10-R08` | `Open` | How do synthesis, upgrade and reforge choose inputs/outputs, consume costs, handle caps and resolve failure or multiple legal candidates? | P1-B6 | Replace with exact transition programs and boundary fixtures, or per-field project policies tied to new released evidence/observation. |
| `G10-R09` | `Open` | How are workbench and gamble offers generated, priced, refreshed, gated and carried across stages? | P1-B7 | Replace with source-backed offer/cost/lifecycle rows and RNG candidate-set fixtures for each service. |
| `G10-R10` | `Open` | Which Talent, unlock, layer-effect, maze-buff, score and difficulty rows change a run or battle rather than account rewards/presentation? | P1-B8 | Replace with a manifest classification backed by explicit downstream references and simulation-visible effect evidence. |
| `G10-R11` | `Open` | Which Blessings, Curios, Occurrences, services and mode-specific copies are reachable in Version 4.4 Unknowable Domain pools? | P2-B1–B4 | Replace with exact selector/transitive-reference/stable-ID closures and 100% pool accounting. |
| `G10-R12` | `Open` | Which encounters, enemy variants, waves and bosses bind to each stage/difficulty, and which ability programs make them mechanically distinct? | P2-B5 | Replace with resolved StageConfig/wave/enemy/ability dossiers or a documented nonblocking boundary for unavailable released evidence. |
| `G10-R13` | `Open` | Which hidden weights, target orders, timing, caps, rounding and fallbacks remain unavailable after bounded research? | P2-B6 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with concrete evidence-triggered replacement conditions. |

## Terminal checklist

- [ ] Exact category manifests and denominators are frozen.
- [ ] Both pinned caches and the focused source/config/TextMap/Stage/ability
      inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership and shared reachability are explicit and fail closed.
- [ ] Shared classifications reconcile with committed Goal 08/09 facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Stage flow, Alignments, Scepters, charge/speed, Components, slots,
      Decision Components, synthesis/upgrades and services have complete
      semantic fixtures.
- [ ] Encounter identities, StageConfig rows, waves and boss bindings resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] All three complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and current Standard/Gold/Swarm/production bundle
      identities remain unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from its recorded remote
      branch at the recorded commit ID.
- [ ] Clean-checkout acceptance passes and `G10-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Unknowable Domain reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G10-P0-B3` |
| Release evidence | — |
| Remaining required work | Unknowable Domain runtime lowering, integration, handlers, controller/API exposure and seeded full runs belong to a later goal. |
