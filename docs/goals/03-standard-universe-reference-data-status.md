# Goal 03 Status — Standard Simulated Universe Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `standard-universe-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Normalized reference pack |
| Active batch | None |
| Next unblocked batch | `G03-P1-B6` |
| Snapshot | Version 4.4 / accessed 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope/evidence | `Complete` | Snapshot/scope, 2,646-file evidence inventory, corrected 2,717-row main-world manifest, stable normalized record families, provenance/quality labels, canonical JSON rules and semantic fixture contract are frozen and machine-verified. |
| Phase 1 — Reference pack | `InProgress` | Ready for deterministic normalized import by table family. |
| Phase 2 — Sora schema | `Pending` | — |
| Phase 3 — Excel authoring | `Pending` | — |
| Phase 4 — Review/freeze | `Pending` | — |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G03-P0-B1` | `Complete` | This row's containing commit | Frozen Version 4.4 / 2026-07-22 main-world scope, nine-World/nine-Path public boundary, pinned source revisions, required/excluded categories, evidence/quality policy, normalized pack and Excel/Sora promotion contracts. Added the 28-batch plan, persistent ledger and launch prompt; both prior release contracts and source-policy checks pass. |
| `G03-P0-B2` | `Complete` | This row's containing commit | Expanded the pinned sparse cache with every Rogue Excel table and Rogue battle/level ability file. Deterministic inventory `4c45418e…8972` hashes 372 files: 35 Standard candidates, 17 shared/reachability-review tables, 165 other-mode tables, 17 presentation/account tables and 138 mechanic-evidence files. The generator's `--check` mode rejects drift; classification is table-level only and does not pre-approve shared rows. |
| `G03-P0-B3` | `Complete` | `b08a632` | Expanded mechanic evidence to 2,646 pinned files (`1d5c7b03…99e7d`) and introduced the reproducible content manifest. P1-B4's effect-table review corrected the initial Curio-state denominator by excluding 121 `1000/3000` DLC copies; the current manifest is 2,717 rows (`f2e611a1…2a416`). Membership resolves 33 World difficulties, nine Paths, 36 Resonance/Formation rows, 162 Blessings with 324 levels, 61 Curios/base effects, 59 Occurrences with 55 base variants, nine domains, 42 Ability Tree nodes, 108 service/bonus rows and the reachable 154-group/347-member encounter set. |
| `G03-P0-B4` | `Complete` | This row's containing commit | Frozen the machine-readable normalized schema for 25 pack files, stable common/provenance envelopes, explicit definition/level/state/variant separation, five evidence-quality labels, canonical decimal/JSON rules and semantic review-fixture shape. Added a deterministic contract verifier covering category counts, ID uniqueness, row evidence/locators and the 9/9/162/324/36 denominators; documentation now records validation order and the JSON-to-Excel/Sora-only promotion boundary. |
| `G03-P1-B1` | `Complete` | This row's containing commit | Added deterministic normalized-pack bootstrap modules and generated 9 Worlds, 33 difficulty profiles, 9 domain kinds, 579 map nodes and 669 rooms. Records carry stable Starclock IDs, bilingual names/summaries, exact source IDs, row-level provenance and evidence digests. Difficulty rows retain recommended levels/elements, score curves and Goal 01 enemy-variant bindings; topology retains source-semantic edge order and exact room content/section maps. Regeneration and `--check` are byte-identical. |
| `G03-P1-B2` | `Complete` | This row's containing commit | Normalized all nine selectable Paths and 36 Path Resonance/Resonance Formation definitions. Paths retain exact Aeon/display identity, source buff type/groups, unlock reference, three formation-selection thresholds, energy defaults and the exact 18-Blessing membership. Resonances retain kind, threshold, energy policy, released modifier/binding keys, ordered exact parameter vectors, description digests and mechanic tags without redistributing source prose. Bilingual and provenance checks plus deterministic regeneration pass. |
| `G03-P1-B3` | `Complete` | This row's containing commit | Normalized the exact 18-Blessing pool for each of nine Paths (162 definitions) and both authored levels for every Blessing (324 level rows). Definitions retain rarity, prerequisites, pool/source tags, extra effects, rule IDs and content-specific mechanic tags. Level rows retain ordered canonical-string parameters, modifier/binding keys and bilingual source-description digests; no source prose or binary float enters the pack. Count, uniqueness, parameter transport and byte-regeneration checks pass. |
| `G03-P1-B4` | `Complete` | This row's containing commit | Corrected the source boundary so `1000/3000` RogueMiracle rows remain Swarm/Gold evidence instead of fake Standard states. Normalized all 61 Standard Curios and 67 lifecycle states: one base active/effect state each plus explicit repairing/fixed phases for six Error Code Curios. Rows retain exact parameter/display vectors, effect IDs, lifecycle charge/transition fields, polarity and mechanic tags, rule references and description digests. Deterministic checks prove every definition/state reference and the 61/67 denominator. |
| `G03-P1-B5` | `Complete` | This row's containing commit | Normalized all 59 CosmosRogue handbook Occurrences, 55 unique base NPC graphs represented by 67 occurrence-variant bindings, and 321 ordered conditional choices. The importer follows released handbook → NPC → dialogue/option graph references, hashes exact bilingual choice/result text, classifies costs/outcome kinds/targets/numeric literals/chances and retains unlock conditions. 269 choices are `ExactPublicText`; 52 random outcomes without released weights carry an explicit `ProjectPolicy` stable-selection rule and replacement condition. No dialogue prose is committed. |
| `G03-P1-B6` | `Pending` | — | — |
| `G03-P1-B7` | `Pending` | — | — |
| `G03-P1-B8` | `Pending` | — | — |
| `G03-P1-B9` | `Pending` | — | — |
| `G03-P2-B1` | `Pending` | — | — |
| `G03-P2-B2` | `Pending` | — | — |
| `G03-P2-B3` | `Pending` | — | — |
| `G03-P2-B4` | `Pending` | — | — |
| `G03-P2-B5` | `Pending` | — | — |
| `G03-P3-B1` | `Pending` | — | — |
| `G03-P3-B2` | `Pending` | — | — |
| `G03-P3-B3` | `Pending` | — | — |
| `G03-P3-B4` | `Pending` | — | — |
| `G03-P3-B5` | `Pending` | — | — |
| `G03-P3-B6` | `Pending` | — | — |
| `G03-P4-B1` | `Pending` | — | — |
| `G03-P4-B2` | `Pending` | — | — |
| `G03-P4-B3` | `Pending` | — | — |
| `G03-P4-B4` | `Pending` | — | — |

## Frozen counters

Populate only from generated manifests in `G03-P0-B3`.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Worlds | 9 | 9 | 9 | Latest permanent manager; Worlds 1-3 use legacy area-ID ordinal derivation. |
| Difficulties/topology | 33 difficulties / 1,248 map-room rows | 33 / 1,248 | 33 / 1,248 | 579 map nodes and 669 non-Adventure rooms. |
| Paths | 9 | 9 | 9 | Main-world selectable Paths. |
| Resonances/Formations | 36 | 36 | 36 | Four per Path; Interplays are excluded. |
| Blessings/upgrades | 162 / 324 levels | 162 / 324 | 162 / 324 | Exactly 18 Blessings and two levels per Path. |
| Curios/states | 61 / 61 source effects / 67 normalized states | 61 / 61 / 67 | 61 / 61 / 67 | CosmosRogue type 100; six Error Codes derive repairing/fixed phases from one released effect row each. |
| Occurrences/choices | 59 / 55 source variants | 59 / 55 | 59 / 55 source variants / 67 bindings / 321 choices | Shared base NPC graphs may bind multiple handbook entries; hidden random weights use explicit policy. |
| Services/currency rules | 108 | 108 | 0 | 79 run bonuses plus 29 base shops; currency constants are normalized in P1. |
| Ability Tree | 42 | 42 | 0 | Battle/run/reward classification occurs in P1-B7. |
| Encounter pools | 154 groups / 347 members | 154 / 347 | 0 | Directly reachable from base combat/encounter/elite/boss room content maps. |
| Mechanic fixtures | Pending | 0 | 0 | Distinct shared families. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-22 | Freeze Version 4.4 main-world Standard SU separately from every DLC profile. | Shared Rogue tables cannot be treated as proof of main-world pool membership. |
| 2026-07-22 | Use normalized JSON only for research/bootstrap; production authoring is Excel/Sora. | Preserves the established configuration architecture. |
| 2026-07-22 | Use Python `openpyxl` for workbook authoring/inspection. | Explicit user direction; Sora retains schema/export authority. |
| 2026-07-22 | Include exact or labeled approximate mechanics, not account rewards/story/assets. | Keeps the package implementation-ready and legally bounded. |
| 2026-07-22 | Treat RogueMiracle `1000/3000` rows as DLC copies, not Standard Curio states. | Their effect-display IDs and mode prefixes bind them to Swarm Disaster and Gold and Gears; Standard lifecycle phases are derived from the base effect program instead. |

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G03-R01` | `Resolved` | Concrete membership is frozen by source schedule, CosmosRogue type, canonical ID family, base NPC prefix and room-content reachability rules in `content-manifest.json`. | P0-B3 |
| `G03-R02` | `Resolved` | All released base NPC/option graphs are imported. Fifty-two choice outcomes mention randomness without exact weights and are labeled `ProjectPolicy: StableUniformOrderedCandidates` with a replacement condition. | P1-B5 |
| `G03-R03` | `Open` | Which Curio effects require battle ability-program inspection or explicit policy? | P1-B4 |
| `G03-R04` | `Open` | Which Ability Tree nodes affect battle/run state versus account rewards only? | P1-B7 |

## Terminal checklist

- [x] Concrete category manifests and counts are frozen.
- [ ] Complete normalized pack and evidence index regenerate deterministically.
- [ ] All required rows have bilingual summaries and provenance.
- [ ] All required mechanics are exact or explicitly approximate/policy-bound.
- [ ] Universe/Activity/Rule Sora schemas and generated readers validate.
- [ ] `openpyxl` workbooks are complete, styled, checked and visually reviewed.
- [ ] Sora production/debug exports regenerate without drift.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Semantic review fixtures cover every distinct mechanic family.
- [ ] Clean checkout and prior Goal 01/02 release contracts pass.
- [ ] `G03-P4-B4` completion commit is recorded and worktree is clean.
