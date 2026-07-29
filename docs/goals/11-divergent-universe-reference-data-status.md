# Goal 11 Status — Divergent Universe Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `divergent-universe-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 0 — Scope, sources and contracts |
| Active batch | None |
| Next unblocked batch | `G11-P0-B2` after `G11-P0-B1` is pushed and remotely verified |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Source-cache reproduction | `G11-P0-B1`: both fixed caches materialized twice in an isolated target from the clean connected object cache; exact detached HEAD, origin, worktree cleanliness, connectivity and the 64-table/direct-entry seed verified |
| Starting source oracle | 64 `RogueTourn*` tables, 3 direct ability programs plus layout companions, CHS/EN TextMaps, StageConfig and transitive config/shared-source closure |
| Focused inventory | Pending `G11-P0-B2` |
| Content manifest | Denominators pending `G11-P0-B3` |
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
| Phase 0 — Scope, sources and contracts | `InProgress` | Goal 03, both pinned caches, the 64-row `RogueTourn` seed, Goal 08/09/10 checkpoints, Candidate-only scope, Sora authority and isolated paths are frozen; focused inventory generation remains. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting stage flow, Arithmetic Mapping, Equations, Divergent Blessings, Curios/Grand Miracles/Hex, Golden Blood/Titan, protocols, services and progression data. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool ownership, Blessings, Curios, Occurrences, services, Adventure outcomes and encounters. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership reconciliation, semantic fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G11-P0-B1` | `Complete` | This row's containing commit | Froze foundation `9f8a3e33…144b7`, Goal 03 commit/tree and preserved bundle digests, the two Version 4.4 source revisions, 64 inherited `RogueTourn` seed rows, 29 batches, Candidate-only scope, Excel/openpyxl/pinned Sora 0.3.0 authority and six isolated roots. Ownership checkpoints: Goal 08 local committed `c283c7f1…55a` (7,913 rows: 7,199 mode-owned/714 shared), Goal 09 remote-backed `d5d261a3…c46` (6,963: 6,305/658), Goal 10 remote-backed `a2e64e1d…94d` (5,377: 5,243/134). Fresh GitLab reproduction failed once with HTTP/2 `PROTOCOL_ERROR`; a bounded HTTP/1.1 attempt resolved the commit but stalled during promisor checkout. The accepted isolated object-cache materialization ran twice; the focused verifier, `git diff --check`, batch-set check and quick repository gate pass; no fresh-network success is claimed. Publication contract: `remote=origin`; `branch=codex/goal11-divergent-universe-reference`; push command `git push origin HEAD:refs/heads/codex/goal11-divergent-universe-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal11-divergent-universe-reference`, requiring identical full commit IDs before P0-B2 starts. |
| `G11-P0-B2` | `Pending` | — | Inventory all 64 focused tables, config/ability programs, TextMaps, StageConfig, shared Rogue/enemy closure and exclusions. |
| `G11-P0-B3` | `Pending` | — | Freeze enabled modules, exact row obligations/counts, ownership, reachability and named exclusions. |
| `G11-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G11-P1-B1` | `Pending` | — | Import modules, entry modes, difficulties, areas, layers, rooms, stage flow, finish and carry/reset rules. |
| `G11-P1-B2` | `Pending` | — | Import temporary character/Trace/Light Cone/Relic Arithmetic Mapping and teardown. |
| `G11-P1-B3` | `Pending` | — | Import Equations, recipes, categories, offers, progress, expansion, effects and replacement. |
| `G11-P1-B4` | `Pending` | — | Import Divergent Blessing categories/levels, Path bindings, transformations and Equation contribution. |
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

Populate required counts only from the generated manifest in `G11-P0-B3`.
Do not estimate denominators from Wiki totals, raw table sizes, prefixes,
modules or ID ranges.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/modules/entries/finish conditions | TBD | 0 | 0 | Includes Ordinary/Cyclical selection, unlocks, initial resources and terminal boundaries. |
| Areas/difficulties/layers/rooms | TBD | 0 | 0 | Includes legal stage flow, module binding, transitions and carry/reset rules. |
| Threshold Protocol/Astronomical Division | TBD | 0 | 0 | Includes Star-Pioneer/Practice Mode and Cognoculi only where they change entry, difficulty, enemies or contributions. |
| Arithmetic Mappings | TBD | 0 | 0 | Includes eligibility, temporary builds, refresh timing and teardown without account mutation. |
| Equations/recipes/expansion states | TBD | 0 | 0 | Includes categories, Path counts, offers, progress, effects and replacement. |
| Divergent Blessings/levels/transforms | TBD | 0 | 0 | Includes Equation contribution and exact enhanced/rewrite behavior. |
| Curios/Weighted Curios/states | TBD | 0 | 0 | Includes eligibility, weighting, charges, destruction, repair and replacement. |
| Grand Miracles/Hex states | TBD | 0 | 0 | Includes character/Path/element eligibility, effects and lifecycle. |
| Golden Blood/Titan definitions | TBD | 0 | 0 | Includes Boons, Titan types/talents, choices, levels and contributions. |
| Workbench/gamble/services | TBD | 0 | 0 | Includes operations, currencies, prices, offered sets and deterministic fallback. |
| Permanent talents/unlocks/modifiers | TBD | 0 | 0 | Only simulation-visible progression, weekly and module effects are enabled. |
| Blessing/Path/shared content pools | TBD | 0 | 0 | Shared reachability and mode copies require explicit proof. |
| Occurrences/variants/choices | TBD | 0 | 0 | Presentation prose is excluded; mechanical graphs and outcomes are included. |
| Services/Adventure outcomes | TBD | 0 | 0 | Adventure input is an abstract offered result, not simulated action gameplay. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Must resolve exact released StageConfig rows, enemy identities and boss alternatives. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every distinct unique mechanic, lifecycle and selection policy. |

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

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G11-R01` | `Open` | Which direct and transitive configuration, TextMap, StageConfig, enemy/wave, shared Rogue and ability files complete the 64-table seed inventory? | P0-B2 | Replace when the generated inventory closes every enabled selector/reference and byte-identical double generation passes. |
| `G11-R02` | `Open` | Which Version 4.4 module selectors separate enabled Divergent-owned, shared, evidence-only, historical/test and other-mode rows? | P0-B3 | Replace with a frozen exact-once ownership manifest whose rows carry selector/reference evidence and fail-closed exclusions. |
| `G11-R03` | `Open` | What are the exact Ordinary/Cyclical area, difficulty, layer, room, transition, finish and carry/reset boundaries for each enabled module? | P1-B1 | Replace with structured stage-flow facts and entry/transition/reset/terminal fixtures. |
| `G11-R04` | `Open` | How are temporary character, Trace, Light Cone and Relic mappings selected, refreshed and removed, including already-stronger account builds? | P1-B2 | Replace with source-backed eligibility/substitution/teardown rows and positive/rejected fixtures for every mapping class. |
| `G11-R05` | `Open` | How are Equations offered, rerolled, progressed, expanded, replaced and evaluated when Blessing ownership changes? | P1-B3–B4 | Replace with exact source programs or field-level policies carrying alternatives, affected fixtures and stronger-evidence triggers. |
| `G11-R06` | `Open` | What are the exact Weighted Curio and Grand Miracle/Hex eligibility, weighting, activation, simultaneous-trigger and no-legal-target semantics? | P1-B5 / P2-B2 | Replace with source-backed pools/lifecycles or explicit deterministic policies for each unresolved field. |
| `G11-R07` | `Open` | How do Golden Blood's Boons, Titan types/talents and choices level, activate, stack, carry and contribute to battle? | P1-B6 | Replace with complete definitions, transition programs and one semantic fixture per distinct lifecycle/contribution. |
| `G11-R08` | `Open` | Which Threshold Protocol, Astronomical Division, Star-Pioneer/Practice Mode and Cognoculus rows change entry, enemy composition, combat numerics, unlocks or lifecycle rather than account rank/rewards? | P1-B7 | Replace with explicit simulation-visible classification and boundary fixtures; reward-only rows remain evidence-only. |
| `G11-R09` | `Open` | How do workbench/gamble operations price and choose Equation/Blessing/Curio inputs/outputs, caps, failures and replacements? | P1-B8 | Replace with source-backed operation programs and candidate-set fixtures or per-field replaceable policies. |
| `G11-R10` | `Open` | Which permanent talents, weekly/cyclical modifiers, unlocks, room marks and services alter a run or battle? | P1-B9 | Replace with manifest classification backed by explicit downstream references and simulation-visible effect evidence. |
| `G11-R11` | `Open` | Which Blessings, Curios, Occurrences, services and mode-specific copies are reachable in each enabled Version 4.4 pool? | P2-B1–B4 | Replace with exact selector/transitive-reference/stable-ID closures and 100% pool accounting. |
| `G11-R12` | `Open` | Which encounters, enemy variants, waves and bosses bind to each module/stage/difficulty, and which ability programs make them distinct? | P2-B5 | Replace with resolved StageConfig/wave/enemy/ability dossiers or a documented nonblocking boundary for unavailable released evidence. |
| `G11-R13` | `Open` | Which hidden weights, target orders, timing, caps, rounding and fallbacks remain unavailable after bounded research? | P2-B6 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete evidence-triggered replacement condition. |

## Terminal checklist

- [ ] Exact enabled-module category manifests and denominators are frozen.
- [ ] Both pinned caches and the focused
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
| Coverage | Denominators pending `G11-P0-B3` |
| Release evidence | — |
| Remaining required work | Divergent Universe runtime lowering, integration, handlers, controller/API exposure and seeded full runs belong to a later goal. |
