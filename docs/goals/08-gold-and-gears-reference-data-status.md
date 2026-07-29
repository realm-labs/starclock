# Goal 08 Status — Gold and Gears Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `gold-and-gears-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 0 — Scope, sources and contracts |
| Active batch | None |
| Next unblocked batch | `G08-P0-B2` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Existing focused inventory | 21 hashed `RogueNous*` tables; denominator not yet frozen |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Parallel condition | Separate branch/worktree and isolated Goal 08 artifacts while Goal 07 is active |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `InProgress` | Goal 03 snapshot and source revisions verified; Goal 08 scope, exclusions, authoring boundary and isolated artifact roots are machine-frozen. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting topology, Cognition, Custom Dice, Knowledge, Neural Network, Conundrum and Resonance Extrapolation data. |
| Phase 2 — Content and encounters | `Pending` | Awaiting mode-pool ownership, Blessings, Curios, Occurrences, services, Adventure outcomes and encounters. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership audit, fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G08-P0-B1` | `Complete` | This row's containing commit | `verify-foundation.mjs`, immutable-snapshot verification and the quick repository gate pass. Froze Goal 03 commit/tree, Version 4.4 source revisions, 21 `RogueNous` seed rows, 28 batches, Candidate-only scope, Excel/openpyxl/Sora authority and six isolated artifact roots. The full source-cache gate reaches the historical Goal 06 contract before failing `Cargo.lock baseline differs`; Goal 03's current-tree verifier likewise reports evolved Universe row counts, so neither immutable historical evidence was rewritten. |
| `G08-P0-B2` | `Pending` | — | Generate the focused released-source inventory and mechanic-evidence closure. |
| `G08-P0-B3` | `Pending` | — | Freeze concrete manifests, counts, ownership and shared reachability. |
| `G08-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook and fixture contracts. |
| `G08-P1-B1` | `Pending` | — | Import entry, difficulties, planes, map topology, rooms, domains, beacons and boss choices. |
| `G08-P1-B2` | `Pending` | — | Import Cognition/Intra-Cognition, Secret thresholds and lifecycle rules. |
| `G08-P1-B3` | `Pending` | — | Import Custom Dice definitions, categories, passives, Path boosts and unlocks. |
| `G08-P1-B4` | `Pending` | — | Import slots, faces, tags, loadouts, rerolls, cheats and face effects. |
| `G08-P1-B5` | `Pending` | — | Import Knowledge and deterministic graph/movement/countdown interactions. |
| `G08-P1-B6` | `Pending` | — | Import the mechanically relevant Neural Network graph, costs and effects. |
| `G08-P1-B7` | `Pending` | — | Import Stats/Auxiliary Conundrum levels, composition and combat changes. |
| `G08-P1-B8` | `Pending` | — | Import bonuses `201`–`205`, Path/Resonance additions and Resonance Extrapolations. |
| `G08-P2-B1` | `Pending` | — | Freeze reachable shared and mode-owned Blessing/Path/Resonance pools. |
| `G08-P2-B2` | `Pending` | — | Import Curios, copies, states, charges, repair and replacement. |
| `G08-P2-B3` | `Pending` | — | Import Occurrences, variants, choices, conditions, costs and outcomes. |
| `G08-P2-B4` | `Pending` | — | Import currencies, services, beacons and abstract Adventure outcomes. |
| `G08-P2-B5` | `Pending` | — | Import encounter groups, waves, enemy variants, elite/boss pools and difficulty bindings. |
| `G08-P2-B6` | `Pending` | — | Generate rules, sources, coverage, research gaps, fixtures and pack index. |
| `G08-P3-B1` | `Pending` | — | Add isolated topology/Cognition/dice/Knowledge Sora tables. |
| `G08-P3-B2` | `Pending` | — | Add Secret/Neural Network/Conundrum/Path/Resonance tables. |
| `G08-P3-B3` | `Pending` | — | Add content, service, Adventure, encounter and mechanic binding tables. |
| `G08-P3-B4` | `Pending` | — | Add evidence/coverage/fixture tables and isolated schemas/templates/readers. |
| `G08-P3-B5` | `Pending` | — | Generate complete isolated `openpyxl` workbooks and structural QA. |
| `G08-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G08-P4-B1` | `Pending` | — | Audit exact-once coverage, ownership, references, provenance and bilingual fields. |
| `G08-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G08-P4-B3` | `Pending` | — | Run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G08-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

## Frozen counters

Populate required counts only from the generated manifest in `G08-P0-B3`.
Do not estimate denominators from Wiki page counts.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/entries/bonuses | TBD | 0 | 0 | Must include Gold and Gears Trailblaze Bonus IDs `201`–`205`. |
| Difficulties/Conundrum unlock | TBD | 0 | 0 | Five base difficulties are the public boundary; exact source rows remain to be frozen. |
| Planes/map nodes/edges/rooms/domains | TBD | 0 | 0 | Includes generated topology rules, beacons, blank/replacement behavior and boss choices. |
| Cognition/Intra-Cognition/Secret conditions | TBD | 0 | 0 | Story prose and collection rewards remain excluded. |
| Custom Dice/categories/passives | TBD | 0 | 0 | Includes initial effects, selected-Path boosts and unlocks. |
| Dice slots/faces/tags/loadouts | TBD | 0 | 0 | Six equipped faces per loadout; exact slot/color constraints are manifest facts. |
| Knowledge rules | TBD | 0 | 0 | Includes placement, propagation, consumption, movement and countdown interaction. |
| Neural Network nodes/effects | TBD | 0 | 0 | Only mechanically relevant account-progression input is enabled. |
| Conundrum definitions/levels | TBD | 0 | 0 | Stats and Auxiliary composition, modifiers, caps and Berserk changes. |
| Paths/boosts/Resonance Extrapolations | TBD | 0 | 0 | Distinguish shared Path content from mode-owned boss behavior. |
| Blessings/enhanced levels | TBD | 0 | 0 | Shared reachability and mode-specific copies require explicit proof. |
| Curios/states | TBD | 0 | 0 | Include mode copies and complete lifecycle behavior. |
| Occurrences/variants/choices | TBD | 0 | 0 | Presentation prose is excluded; mechanical graph and outcomes are included. |
| Services/beacons/Adventure outcomes | TBD | 0 | 0 | Adventure input is an abstract offered result, not simulated action gameplay. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Must resolve exact released enemy identities and boss alternatives. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every distinct unique mechanic, lifecycle and selection policy. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 08 as a complete reference-data package, not a runtime goal. | Gold and Gears research can proceed independently while Standard Universe mechanics are still being implemented. |
| 2026-07-29 | Inherit the Version 4.4 structured snapshot and pinned source revision used by Goal 03. | Shared identity and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require separate worktree/branch and isolated artifacts during concurrent Goal 07 work. | Prevents workbook/generated-output collisions and Standard profile contamination. |
| 2026-07-29 | Treat the twenty-one existing `RogueNous*` rows as an inventory seed, not a completeness oracle. | Shared Rogue tables, ability programs, stages and TextMap records still determine exact mechanics and pool reachability. |
| 2026-07-29 | Reuse shared stable IDs only after Gold and Gears reachability is proven. | Similar names or source-table reuse do not prove identical ownership, effect state or eligibility. |
| 2026-07-29 | Exclude story prose and account/collection rewards while retaining mechanical unlock locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal after Standard completion. |
| 2026-07-29 | Treat an LF checkout as equivalent to Goal 03 source hashes only when LF-to-CRLF conversion reproduces the recorded byte count and SHA-256. | Goal 03 captured CRLF checkout bytes; the pinned macOS cache contains the same Git blobs with LF endings. |

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G08-R01` | `Open` | Which reachable shared Rogue, TextMap, StageConfig and ability files complete the twenty-one-table `RogueNous` starting inventory? | P0-B2 |
| `G08-R02` | `Open` | What exact IDs and ownership rules separate Gold and Gears-owned, shared, evidence-only and other-mode rows? | P0-B3 |
| `G08-R03` | `Open` | What are the exact dice-face target ordering, duration, reroll/cheat and no-legal-target semantics? | P1-B4 |
| `G08-R04` | `Open` | At which operation boundaries are Cognition values adjusted, clamped, carried and evaluated for Secrets? | P1-B2 |
| `G08-R05` | `Open` | How are Knowledge placement, propagation, countdown recovery and graph mutation ordered when multiple effects coincide? | P1-B5 |
| `G08-R06` | `Open` | What are the exact Conundrum level modifiers, stacking/composition rules, caps and Berserk timing changes? | P1-B7 |
| `G08-R07` | `Open` | How do Path boosts and Resonance Extrapolation select, scale, charge and act in the final boss battle? | P1-B8 |
| `G08-R08` | `Open` | Which Blessings, Curios, Occurrences, services and mode-specific copies are actually reachable in the Version 4.4 Gold and Gears pools? | P2-B1–B4 |
| `G08-R09` | `Open` | Which map-generation, room-selection, occurrence and reward weights are released, observed or require explicit project policy? | P1-B1 / P2-B3–B4 |
| `G08-R10` | `Open` | Which Neural Network nodes affect a run or battle, and which are collection/account rewards outside the simulation boundary? | P1-B6 |

## Terminal checklist

- [ ] Exact category manifests and denominators are frozen.
- [ ] Focused source inventory and evidence closure regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership and shared reachability are explicit and fail closed.
- [ ] All required mechanics are exact or explicitly approximate/policy-bound.
- [ ] Cognition, dice, Knowledge, Neural Network, Conundrum and Resonance
      Extrapolation have complete semantic fixtures.
- [ ] Encounter identities, waves and boss bindings resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] Complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and current Standard/production bundle identities remain
      unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Clean-checkout acceptance passes and `G08-P4-B4` is committed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Gold and Gears reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G08-P0-B3` |
| Release evidence | — |
| Remaining required work | Gold and Gears runtime lowering, integration, controller/API exposure and seeded full runs belong to a later goal. |
