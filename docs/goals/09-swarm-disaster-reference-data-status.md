# Goal 09 Status — Swarm Disaster Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `swarm-disaster-reference-v1` |
| State | `Ready` |
| Active phase | None |
| Active batch | None |
| Next unblocked batch | `G09-P0-B1` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Existing focused inventory | 32 hashed `RogueDLC*` tables; shared framework, denominator not yet frozen |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Parallel condition | Separate branch/worktree and isolated Goal 09 artifacts while Goals 07 or 08 are active |
| Publication policy | Push each completed batch commit to its configured remote branch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Pending` | Awaiting snapshot verification, shared-DLC ownership audit, focused inventory, frozen denominator and normalized/authoring contracts. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting topology, domains/beacons, countdown/Planar Disarray, Audience Dice, Communing Device/Trail, Pathstrider and Resonance Interplay data. |
| Phase 2 — Content and encounters | `Pending` | Awaiting mode-pool ownership, Blessings, Curios, Occurrences, services, Adventure outcomes and encounters. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership reconciliation, fixtures, regeneration, release evidence and clean-checkout acceptance. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G09-P0-B1` | `Pending` | — | Verify Goal 03 snapshot, freeze scope/exclusions, inspect Goal 08 ownership and prove parallel artifact isolation. |
| `G09-P0-B2` | `Pending` | — | Generate the focused released-source inventory and mechanic-evidence closure. |
| `G09-P0-B3` | `Pending` | — | Freeze concrete manifests, counts, ownership, shared reachability and Gold-mode exclusions. |
| `G09-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G09-P1-B1` | `Pending` | — | Import entry, difficulties, planes, map topology and terminal rules. |
| `G09-P1-B2` | `Pending` | — | Import rooms, domains, beacons, replacements and boss-choice consequences. |
| `G09-P1-B3` | `Pending` | — | Import countdown, Planar Disarray, boss-decay levels, caps, timing and combat changes. |
| `G09-P1-B4` | `Pending` | — | Import Paths, Audience Dice definitions, initial/passive effects and unlocks. |
| `G09-P1-B5` | `Pending` | — | Import dice faces, rarities, targets, effects, rolls, rerolls, cheats and fallback policies. |
| `G09-P1-B6` | `Pending` | — | Import Communing Device choices, Aeon cabinets/dimensions, point changes and carry/order rules. |
| `G09-P1-B7` | `Pending` | — | Import mechanically relevant Communing Trail nodes, prerequisites, thresholds and effects. |
| `G09-P1-B8` | `Pending` | — | Import Pathstrider objectives, finish/progress conditions, unlocks and mechanical chapter locators. |
| `G09-P1-B9` | `Pending` | — | Import bonuses `101`–`106`, Path/Resonance additions, Propagation and Resonance Interplays. |
| `G09-P2-B1` | `Pending` | — | Freeze reachable shared and Swarm-owned Blessing/Path/Resonance pools. |
| `G09-P2-B2` | `Pending` | — | Import Curios, copies, states, charges, repair and replacement. |
| `G09-P2-B3` | `Pending` | — | Import Occurrences, variants, choices, conditions, costs and outcomes. |
| `G09-P2-B4` | `Pending` | — | Import currencies, services, beacons and abstract Adventure outcomes. |
| `G09-P2-B5` | `Pending` | — | Import encounter groups, waves, enemy variants, elite/boss pools and difficulty bindings. |
| `G09-P2-B6` | `Pending` | — | Generate rules, sources, coverage, research gaps, fixtures and pack index. |
| `G09-P3-B1` | `Pending` | — | Add isolated topology/domain/countdown/Disarray Sora tables. |
| `G09-P3-B2` | `Pending` | — | Add dice, Communing Device/Trail and Pathstrider Sora tables. |
| `G09-P3-B3` | `Pending` | — | Add Interplay, content, service, Adventure, encounter and mechanic binding tables. |
| `G09-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated schemas/templates/readers. |
| `G09-P3-B5` | `Pending` | — | Generate complete isolated `openpyxl` workbooks and structural QA. |
| `G09-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G09-P4-B1` | `Pending` | — | Audit exact-once coverage, ownership, references, provenance and bilingual fields. |
| `G09-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G09-P4-B3` | `Pending` | — | Reconcile Goal 08 overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G09-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

## Frozen counters

Populate required counts only from the generated manifest in `G09-P0-B3`.
Do not estimate denominators from Wiki page counts or raw table sizes.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profiles/entries/bonuses | TBD | 0 | 0 | Must include Swarm Disaster Trailblaze Bonus IDs `101`–`106`. |
| Difficulties/unlocks | TBD | 0 | 0 | Five base difficulties are the public boundary; exact source obligations remain to be frozen. |
| Planes/map nodes/edges/rooms/domains | TBD | 0 | 0 | Includes three-plane topology, legal movement, beacons, replacements and boss choices. |
| Countdown/Planar Disarray/decay | TBD | 0 | 0 | Includes entry boundary, levels, caps, timing, modifiers and battle contributions. |
| Paths/Audience Dice | TBD | 0 | 0 | Includes definitions, initial/passive effects, unlocks and Path-specific graph rules. |
| Dice faces/rarities/roll controls | TBD | 0 | 0 | Includes targets, durations, reroll/cheat and no-legal-target behavior. |
| Communing Device/cabinets/dimensions | TBD | 0 | 0 | Includes choices, point changes, eligibility, ordering and carry/reset. |
| Communing Trail nodes/effects | TBD | 0 | 0 | Only mechanically relevant progression input enters the simulation boundary. |
| Pathstrider objectives/unlocks | TBD | 0 | 0 | Story prose and account rewards remain excluded. |
| Paths/Resonances/Interplays | TBD | 0 | 0 | Includes Propagation additions and final-boss contributions. |
| Blessings/enhanced levels | TBD | 0 | 0 | Shared reachability and mode-specific copies require explicit proof. |
| Curios/states | TBD | 0 | 0 | Include Swarm copies and complete lifecycle behavior. |
| Occurrences/variants/choices | TBD | 0 | 0 | Presentation prose is excluded; mechanical graphs and outcomes are included. |
| Services/beacons/Adventure outcomes | TBD | 0 | 0 | Adventure input is an abstract offered result, not simulated action gameplay. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Must resolve exact released enemy identities and boss alternatives. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every distinct unique mechanic, lifecycle and selection policy. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 09 as a complete reference-data package, not a runtime goal. | Swarm Disaster research can proceed independently while Standard mechanics and Gold and Gears research continue. |
| 2026-07-29 | Inherit the Version 4.4 structured snapshot and pinned source revision used by Goal 03 and Goal 08. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require a separate worktree/branch and isolated artifacts during concurrent Goal 07/08 work. | Prevents workbook/generated-output collisions and cross-mode contamination. |
| 2026-07-29 | Treat the thirty-two existing `RogueDLC*` tables as a shared-framework inventory seed, not a Swarm denominator. | Gold and Gears also uses the DLC framework; only selectors, transitive references and stable-ID closure prove Swarm reachability. |
| 2026-07-29 | Reconcile shared `RogueDLC` rows by source path, row locator and evidence digest without editing Goal 08 artifacts. | Concurrent mode goals must agree on shared facts while retaining isolated ownership ledgers. |
| 2026-07-29 | Reuse shared stable IDs only after Swarm reachability is proven. | Matching names, adjacent IDs or base-table reuse do not prove identical ownership, state or eligibility. |
| 2026-07-29 | Exclude story prose and account/collection rewards while retaining mechanical unlock locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-29 | Require every completed batch commit to be pushed before the next batch begins. | Keeps the resumable ledger and artifacts visible from other machines and prevents unpublished local progress from becoming the effective source of truth. |

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G09-R01` | `Open` | Which `ChessRogue` topology, shared Rogue, TextMap, StageConfig and ability files complete the thirty-two-table `RogueDLC` starting inventory? | P0-B2 |
| `G09-R02` | `Open` | What exact selectors and ownership rules separate Swarm-owned, shared, Gold-owned, evidence-only and other-mode rows? | P0-B3 |
| `G09-R03` | `Open` | Which map edges, generation weights, replacement rules and legal movement relationships are released versus inferred? | P1-B1–B2 |
| `G09-R04` | `Open` | At which operation boundaries is countdown adjusted, clamped/carried and converted into each Planar Disarray/decay level? | P1-B3 |
| `G09-R05` | `Open` | What are the exact Audience Die face target ordering, duration, roll/reroll/cheat and no-legal-target semantics? | P1-B4–B5 |
| `G09-R06` | `Open` | How are Communing Device choices, cabinet/dimension points, eligibility and simultaneous unlocks ordered and carried? | P1-B6 |
| `G09-R07` | `Open` | Which Communing Trail and Pathstrider rows change a run or battle, and what are their exact prerequisites/progress semantics? | P1-B7–B8 |
| `G09-R08` | `Open` | How do Resonance Interplays unlock, select, scale, charge and act, including Propagation and final-boss contributions? | P1-B9 |
| `G09-R09` | `Open` | Which Blessings, Curios, Occurrences, services and mode-specific copies are actually reachable in Version 4.4 Swarm pools? | P2-B1–B4 |
| `G09-R10` | `Open` | Which room, occurrence, reward, dice and boss-choice weights are released, observed or require explicit project policy? | P1-B1–B6 / P2-B3–B5 |
| `G09-R11` | `Open` | Which boss choices alter later boss identity, weaknesses, modifiers or Disarray behavior, and at what lifecycle boundary? | P1-B2–B3 / P2-B5 |

## Terminal checklist

- [ ] Exact category manifests and denominators are frozen.
- [ ] Focused source inventory and evidence closure regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership and shared reachability are explicit and fail closed.
- [ ] Shared `RogueDLC` classifications reconcile with committed Goal 08 facts.
- [ ] All required mechanics are exact or explicitly approximate/policy-bound.
- [ ] Topology, countdown/Disarray, dice, Communing Device/Trail, Pathstrider
      and Resonance Interplay have complete semantic fixtures.
- [ ] Encounter identities, waves and boss bindings resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] Complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and current Standard/Gold/production bundle identities
      remain unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from the recorded remote
      branch.
- [ ] Clean-checkout acceptance passes and `G09-P4-B4` is committed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Swarm Disaster reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G09-P0-B3` |
| Release evidence | — |
| Remaining required work | Swarm Disaster runtime lowering, integration, controller/API exposure and seeded full runs belong to a later goal. |
