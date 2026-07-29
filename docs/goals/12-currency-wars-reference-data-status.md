# Goal 12 Status — Currency Wars Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `currency-wars-reference-v1` |
| State | `Ready` |
| Current phase | Phase 0 — Scope, sources and contracts (`Pending`) |
| Current batch | None; Goal execution has not started |
| Next unblocked batch | `G12-P0-B1` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-29: both caches clean at pinned commits; origins, commit readability and `git fsck --connectivity-only --no-dangling` verified; execution must reproduce in `G12-P0-B1` |
| Starting source oracle | `Tourn3` / MainTourn `3` / ActivityModule `6002201`; 11 `RoguePersona*` tables; all 64 shared `RogueTourn*` tables; 4 direct `_S3` ability programs; CHS/EN TextMaps, StageConfig and transitive config/shared-source closure |
| Focused inventory | Pending `G12-P0-B2` |
| Content manifest | Denominators pending `G12-P0-B3` |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal12-currency-wars-reference` |
| Branch base | `b0cd3cb912c9f2ec887c3ae29f79353c4a861643` (`G11-SETUP`; excludes later Goal 11 batches and every concurrent uncommitted change) |
| Parallel condition | Separate branch/worktree and isolated Goal 12 artifacts while Goals 07 through 11 are active |
| Publication policy | Push each completed batch commit and verify the remote branch commit before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Pending` | Planning audit only; `G12-P0-B1` must reproduce caches, concurrent Goal boundaries and artifact isolation before Goal data mutation. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting flow, Squad HP/action value, economy, roster/shop/star, positioning/Empowerment, Bonds, build/equipment, Persona and rank data. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool ownership, Blessings, Curios, events, services, enemies, waves and bosses. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership reconciliation, semantic fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G12-P0-B1` | `Pending` | — | Reproduce caches, verify Goal 03 and concurrent Goal boundaries, freeze scope and prove branch/worktree/path isolation. |
| `G12-P0-B2` | `Pending` | — | Inventory 11 Persona tables, all 64 Tourn tables through `Tourn3`, config/ability programs, TextMaps, StageConfig, shared build/Rogue/enemy closure and exclusions. |
| `G12-P0-B3` | `Pending` | — | Freeze enabled selectors, exact row obligations/counts, ownership, reachability and named exclusions. |
| `G12-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G12-P1-B1` | `Pending` | — | Import Gambit modes, difficulties/ranks, Planes, Nodes, rooms, Domain composition, flow, finish and carry/reset. |
| `G12-P1-B2` | `Pending` | — | Import Squad HP, action-value limits, timeout projection, loss/recovery, run failure and boundary ordering. |
| `G12-P1-B3` | `Pending` | — | Import roster, costs, shop offers, Gold Coins, refreshes, Experience, team size and roster lifecycle. |
| `G12-P1-B4` | `Pending` | — | Import positioning, Character Empowerment, automatic Techniques, energy scaling, lethal rescue and battle overrides. |
| `G12-P1-B5` | `Pending` | — | Import Bonds, members, thresholds, levels, recomputation and rule contributions. |
| `G12-P1-B6` | `Pending` | — | Import star states, three-copy combination, scaling, overflow, replacement and teardown. |
| `G12-P1-B7` | `Pending` | — | Import owned/trial mapping, build substitution, off-field conversions and equipment lifecycle. |
| `G12-P1-B8` | `Pending` | — | Import Investment Environments/Strategies and reachable Persona styles, gifts, talents and room composition. |
| `G12-P1-B9` | `Pending` | — | Import rank/Gambit boundaries, enemy affixes and simulation-visible permanent progression. |
| `G12-P2-B1` | `Pending` | — | Freeze reachable shared and mode-owned Blessing/level/buff/formula pools, including proven-empty categories. |
| `G12-P2-B2` | `Pending` | — | Import Curios/Miracles/Hexes, mode copies, equipment-like states and lifecycle. |
| `G12-P2-B3` | `Pending` | — | Import events/Occurrences, variants, choices, chests, conditions, costs and outcomes. |
| `G12-P2-B4` | `Pending` | — | Import currencies, recruitment/refresh/upgrade shops, services, prices, inventories and candidate sets. |
| `G12-P2-B5` | `Pending` | — | Import encounter groups, StageConfig waves, enemy variants, elite/boss pools and Plane/difficulty bindings. |
| `G12-P2-B6` | `Pending` | — | Generate rules, sources, coverage, research gaps, fixtures and pack index. |
| `G12-P3-B1` | `Pending` | — | Add isolated profile/stage/difficulty/Squad-HP/action-value/economy Sora tables. |
| `G12-P3-B2` | `Pending` | — | Add roster/position/star/Bond/Empowerment/build/equipment/Persona Sora tables. |
| `G12-P3-B3` | `Pending` | — | Add content, service, event, encounter and rule-binding tables. |
| `G12-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated schema locks/templates/readers. |
| `G12-P3-B5` | `Pending` | — | Generate all three complete `openpyxl` workbooks and structural/semantic QA. |
| `G12-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G12-P4-B1` | `Pending` | — | Audit exact-once coverage, enabled selectors, ownership, references, provenance and bilingual fields. |
| `G12-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G12-P4-B3` | `Pending` | — | Reconcile Goal 08/09/10/11 overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G12-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but unverified batch
remains `InProgress`.

The Goal package setup commit is identified as “this document's containing
commit” to avoid a recursive self-hash. Its exact push command, result and
remote resolution are reported in the setup handoff. `G12-P0-B1` must record
the full setup commit and remote verification as immutable foundation evidence
before any Goal data mutation.

## Frozen counters

Populate required counts only from the generated manifest in `G12-P0-B3`.
Do not estimate denominators from Wiki totals, raw table sizes, prefixes,
modules, names or ID ranges.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/Gambit modes/entries/finish conditions | TBD | 0 | 0 | Includes Standard/Overclock selection, unlocks, initial resources and terminal boundaries. |
| Planes/difficulties/ranks/Nodes/rooms | TBD | 0 | 0 | Includes legal flow, Domain composition, transitions and carry/reset rules. |
| Squad HP/action-value projections | TBD | 0 | 0 | Includes initialization, timeout, victory/defeat projection, loss/recovery and run failure. |
| Roster/cost/shop/team-size economy | TBD | 0 | 0 | Includes offers, purchases/sales, bench/field caps, Gold Coins, Experience and refresh resources. |
| Positions/Character Empowerments | TBD | 0 | 0 | Includes deployment validation, automatic Techniques, battle overrides and teardown. |
| Bonds/members/levels | TBD | 0 | 0 | Includes thresholds, recomputation and Activity/Battle contributions. |
| Star states/copy combinations | TBD | 0 | 0 | Includes one-/two-/three-star scaling, three-copy merge, overflow and replacement. |
| Build mappings/equipment/conversions | TBD | 0 | 0 | Includes owned/trial substitution, off-field conversions and three equipment slots. |
| Investment Environment/Strategy/Persona | TBD | 0 | 0 | Includes only selector-reachable styles, gifts, talents and room compositions. |
| Blessings/levels/formulas | TBD | 0 | 0 | Reachability or a zero denominator requires generated selector/reference closure. |
| Curios/Miracles/Hex states | TBD | 0 | 0 | Includes ownership, charges, destruction, repair, replacement and offers. |
| Events/variants/choices | TBD | 0 | 0 | Presentation prose is excluded; mechanical graphs and outcomes are included. |
| Currencies/shops/services | TBD | 0 | 0 | Includes prices, inventories, candidate sets, eligibility and fallback. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Must resolve exact released StageConfig rows, enemy identities and boss alternatives. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every distinct unique mechanic, lifecycle and selection policy. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 12 as a complete reference-data package, not a runtime goal. | Currency Wars research can proceed independently while Standard mechanics and other mode research continue. |
| 2026-07-29 | Use `currency-wars` as the stable mode slug and reserve Goal 12. | Goals 08 through 11 already have committed packages and active isolated branches; no local or remote Goal 12 package/branch existed during the planning audit. |
| 2026-07-29 | Base the branch on `b0cd3cb9…`, the committed Goal 11 setup package. | The base includes Goal 01–11 indexing but excludes later Goal 11 batches and every concurrent worktree's uncommitted changes. |
| 2026-07-29 | Inherit the pinned Version 4.4 structured snapshot and identity cross-check used by Goals 03 and 08–11. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require `G12-P0-B1` to reproduce both caches even though the planning audit found them clean at the pinned commits. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-29 | Treat `Tourn3` / MainTourn `3` / ActivityModule `6002201` as the initial Currency Wars selector. | The three committed structured routing rows agree, while table prefixes and ID adjacency do not establish row-level ownership. |
| 2026-07-29 | Treat all 11 `RoguePersona*`, all 64 `RogueTourn*` and the four `_S3` abilities as inventory seeds, not denominators. | Shared, display, reward, historical and other-module rows coexist with enabled mechanics; transitive reachability determines the manifest. |
| 2026-07-29 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent mode goals must preserve isolated ownership ledgers and surface conflicts for merge coordination. |
| 2026-07-29 | Allow a content-family count of zero only through a generated closed reachability proof. | Currency Wars shares tables with Divergent Universe; neither table presence nor an unresolved join proves membership or absence. |
| 2026-07-29 | Exclude story/presentation and account/collection/rank/weekly rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-29 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G12-R01` | `Open` | Which direct and transitive configuration, TextMap, StageConfig, enemy/wave, shared build/Rogue and ability files complete the 11 Persona/64 Tourn table seed inventory? | P0-B2 | Replace when the generated inventory closes every enabled selector/reference and byte-identical double generation passes. |
| `G12-R02` | `Open` | Which `Tourn3` selectors separate Currency Wars-owned, shared, evidence-only, Divergent Universe and other-module rows? | P0-B3 | Replace with a frozen exact-once ownership manifest whose rows carry selector/reference evidence and fail-closed exclusions. |
| `G12-R03` | `Open` | What are the exact Standard/Overclock entry, difficulty/rank, three-Plane, Node/room, transition, finish and carry/reset boundaries? | P1-B1 / P1-B9 | Replace with structured flow facts and entry/transition/reset/terminal fixtures. |
| `G12-R04` | `Open` | How are Squad HP, action-value timeout, battle victory/defeat and run termination ordered and projected? | P1-B2 | Replace with source-backed state/projection rows and fixtures for victory, timeout, HP loss and zero-HP termination. |
| `G12-R05` | `Open` | How are roster offers, costs, refreshes, purchases/sales, bench/field caps, Experience and team-size changes selected and ordered? | P1-B3 | Replace with exact economy/candidate programs or field-level policies carrying alternatives and fixtures. |
| `G12-R06` | `Open` | What are the exact position, Character Empowerment, automatic Technique, defeat-energy and lethal-rescue/countdown semantics? | P1-B4 | Replace with source-backed activation/teardown and battle contribution rows plus positive/rejected fixtures. |
| `G12-R07` | `Open` | Which characters belong to each Bond, and how do thresholds, levels, simultaneous roster changes and contributions recompute? | P1-B5 | Replace with complete membership/threshold graphs and one fixture per distinct contribution and boundary. |
| `G12-R08` | `Open` | How do three-copy star upgrades, duplicate overflow, stat scaling, sales and replacements interact? | P1-B6 | Replace with exact transition programs and fixtures for every legal/rejected one-/two-/three-star path. |
| `G12-R09` | `Open` | How are owned/trial builds mapped and removed, and how are off-field Eidolons, signature Light Cones and equipment converted? | P1-B7 | Replace with source-backed eligibility/substitution/conversion/teardown rows and fixtures for every mapping class. |
| `G12-R10` | `Open` | Which Persona styles, gifts, talents and room compositions implement Investment Environments/Strategies, and what are their offer/activation rules? | P1-B8 | Replace with explicit `Tourn3` reachability, typed lifecycle rows and fixtures for every distinct effect family. |
| `G12-R11` | `Open` | Which Blessings, formulas, Curios/Miracles/Hexes, events and services are reachable in released Version 4.4 Currency Wars pools? | P2-B1–B4 | Replace with exact selector/transitive-reference/stable-ID closures and 100% pool accounting, including proven-empty categories. |
| `G12-R12` | `Open` | Which encounters, enemy variants, waves and bosses bind to each Plane/difficulty/rank, and which ability programs make them distinct? | P2-B5 | Replace with resolved StageConfig/wave/enemy/ability dossiers or a documented nonblocking boundary for unavailable released evidence. |
| `G12-R13` | `Open` | Which hidden weights, target orders, simultaneous transitions, timing, caps, rounding and fallbacks remain unavailable after bounded research? | P2-B6 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete evidence-triggered replacement condition. |

## Terminal checklist

- [ ] Exact enabled-selector category manifests and denominators are frozen.
- [ ] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership, module enablement and shared reachability are explicit and
      fail closed.
- [ ] Shared classifications reconcile with committed Goal 08/09/10/11 facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Flow, Squad HP/action value, economy, roster/star, position/Empowerment,
      Bonds, mapping/equipment, Persona and content pools have complete semantic
      fixtures.
- [ ] Encounter identities, StageConfig rows, waves and boss bindings resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] All three complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and all other mode/production bundle identities remain
      unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from its recorded remote
      branch at the recorded commit ID.
- [ ] Clean-checkout acceptance passes and `G12-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Currency Wars reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G12-P0-B3` |
| Release evidence | — |
| Remaining required work | Currency Wars runtime lowering, integration, handlers, controller/API exposure and seeded full runs belong to a later goal. |
