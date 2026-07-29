# Goal 10 Status — Unknowable Domain Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `unknowable-domain-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 0 — Scope, sources and contracts |
| Active batch | None |
| Next unblocked batch | `G10-P0-B3` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-29: both caches clean at pinned commits; commit readability and connectivity checked; execution must reproduce in `G10-P0-B1` |
| Focused inventory | 2,684 pinned files: 2,675 `turnbasedgamedata` and 9 `StarRailRes`; content denominator not yet frozen |
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
| Phase 0 — Scope, sources and contracts | `InProgress` | Goal 03, both pinned caches, the 2,684-file focused inventory, Goal 08/09 checkpoints, Candidate-only scope, Sora authority and isolated paths are frozen; row denominator and normalized authoring contracts remain. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting stage flow, Alignments, Scepters, Components, Decision Components, synthesis/upgrades, services and progression data. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool ownership, Blessings, Curios, Occurrences, services, Adventure outcomes and encounters. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership reconciliation, semantic fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G10-P0-B1` | `Complete` | This row's containing commit | Froze foundation `270d016b…9407`, Goal 03 commit/tree and bundle digests, the Version 4.4 revisions, 32 inherited `RogueMagic` seed rows, 28 batches, Candidate-only scope, Excel/openpyxl/pinned Sora 0.3.0 authority and six isolated roots. The Goal 08 local-only checkpoint is `2f7b3ccf…fc5d` (7,913 obligations: 7,199 Gold-owned and 714 shared); the required remote-backed Goal 09 checkpoint is `1f9019a2…5ae2` (2,882 source records). `pwsh -File tools/content-reference/fetch-sources.ps1 -CacheRoot .cache/content-reference` could not run because `pwsh` is absent; the isolated POSIX fetcher reproduced clean detached caches repeatedly and the focused verifier, `git diff --check` and quick repository gate pass. `node tools/repository-check/run.mjs --full --with-source-cache` reaches the immutable Goal 06 contract before repeating its known `Cargo.lock baseline differs` failure. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P0-B2 starts. |
| `G10-P0-B2` | `Complete` | This row's containing commit | Generated and rechecked `source-inventory.json` (`20fad854…baa5`, 1,036,464 bytes): all 2,646 Goal 03 source files, plus 29 focused mode/StageConfig/TextMap entries and nine bilingual StarRailRes indexes. The 2,684-file closure contains 32 `RogueMagic` tables, 16 direct ability files, 14 Scepter battle events, three service graphs, six mechanical maze graphs and 57 Rogue260 NPC graphs; 141 named other-mode files remain fail-closed exclusion evidence. Raw Git blob hashing removes checkout-EOL variance; file families grant no content ownership. Foundation/inventory verifiers, `git diff --check` and the quick repository gate pass; the full source-cache gate repeats the immutable Goal 06 `Cargo.lock baseline differs` boundary. Publication contract: `remote=origin`; `branch=codex/goal10-unknowable-domain-reference`; push command `git push origin HEAD:refs/heads/codex/goal10-unknowable-domain-reference`; verify with `git rev-parse HEAD` and `git ls-remote --exit-code origin refs/heads/codex/goal10-unknowable-domain-reference`, requiring identical full commit IDs before P0-B3 starts. |
| `G10-P0-B3` | `Pending` | — | Freeze concrete manifests, counts, ownership, shared reachability and named-mode exclusions. |
| `G10-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G10-P1-B1` | `Pending` | — | Import entry, areas, difficulties, layers, rooms, stage flow, finish and carry/reset rules. |
| `G10-P1-B2` | `Pending` | — | Import four Extrapolation Alignments, selection, eligibility, pools and rule contributions. |
| `G10-P1-B3` | `Pending` | — | Import Scepter definitions/levels, activation, charge/speed, ranges and lifecycle. |
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

Populate required counts only from the generated manifest in `G10-P0-B3`.
Do not estimate denominators from Wiki totals, raw table sizes, prefixes or ID
ranges.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/entries/finish conditions | TBD | 0 | 0 | Includes unlock, initial resources, terminal states and simulation-visible score/finish rules. |
| Areas/difficulties/layers/rooms | TBD | 0 | 0 | Includes legal stage flow, transitions, carry/reset and concrete room membership. |
| Extrapolation Alignments | TBD | 0 | 0 | Four is the released public boundary; exact source obligations remain manifest-derived. |
| Scepters/levels/states | TBD | 0 | 0 | Includes functions/styles, power, activation, charge/speed, range and lifecycle. |
| Components/levels/effects | TBD | 0 | 0 | Includes category/type, shape, slot/range compatibility and effect parameters. |
| Decision Components/choices | TBD | 0 | 0 | Includes eligibility, ordering, scope, outcomes and fallback. |
| Loadouts/slots/insertion/replacement | TBD | 0 | 0 | Complete legality and no-legal-option policies are required. |
| Synthesis/upgrades/reforges | TBD | 0 | 0 | Includes costs, inputs, output pools, ordering, caps and failure/replacement behavior. |
| Workbench/gamble/services | TBD | 0 | 0 | Includes currencies, prices, offered sets, eligibility and lifecycle. |
| Talents/unlocks/layer/difficulty effects | TBD | 0 | 0 | Only simulation-visible progression and mechanics are enabled. |
| Blessings/enhanced levels | TBD | 0 | 0 | Shared reachability and mode-specific copies require explicit proof. |
| Curios/states | TBD | 0 | 0 | Includes mode copies and complete lifecycle behavior. |
| Occurrences/variants/choices | TBD | 0 | 0 | Presentation prose is excluded; mechanical graphs and outcomes are included. |
| Services/Adventure outcomes | TBD | 0 | 0 | Adventure input is an abstract offered result, not simulated action gameplay. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Must resolve exact released StageConfig rows, enemy identities and boss alternatives. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every distinct unique mechanic, lifecycle and selection policy. |

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

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G10-R01` | `Resolved` | Which shared Rogue tables, configuration programs, TextMap rows, StageConfig rows, enemy/wave records and transitive ability files complete the `RogueMagic` seed inventory? | P0-B2 | `source-inventory.json` freezes all 2,646 inherited Goal 03 source files, 29 exact focused additions and nine public index files; raw-blob regeneration is byte-identical and the inventory verifier closes every documented family. |
| `G10-R02` | `Open` | What exact selectors separate Unknowable-owned, shared, evidence-only and Standard/Gold/Swarm/Divergent rows? | P0-B3 | Replace with a frozen exact-once ownership manifest whose rows carry selector/reference evidence and fail-closed exclusions. |
| `G10-R03` | `Open` | What is the exact area/layer/room ordering, carry/reset behavior and finish boundary for every released stage and difficulty? | P1-B1 | Replace with structured stage-flow facts plus fixtures for entry, transition, reset and terminal boundaries. |
| `G10-R04` | `Open` | How do Alignment selection, eligibility and candidate pools constrain Scepters, Components and battle contributions? | P1-B2 | Replace with source-backed bindings and one semantic fixture per Alignment and selection boundary. |
| `G10-R05` | `Open` | What are the exact Scepter activation, charge gain/spend, speed/action ordering, simultaneous-trigger and teardown semantics? | P1-B3 | Replace with released program/observation evidence, or a field-level policy carrying alternatives, affected fixtures and a stronger-evidence trigger. |
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
