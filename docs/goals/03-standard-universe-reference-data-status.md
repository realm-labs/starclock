# Goal 03 Status — Standard Simulated Universe Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `standard-universe-reference-v1` |
| State | `InProgress` |
| Active phase | Phase 1 — Normalized reference pack |
| Active batch | None |
| Next unblocked batch | `G03-P1-B9` |
| Snapshot | Version 4.4 / accessed 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope/evidence | `Complete` | Snapshot/scope, 2,646-file evidence inventory, corrected 1,935-row main-world manifest, stable normalized record families, provenance/quality labels, canonical JSON rules and semantic fixture contract are frozen and machine-verified. |
| Phase 1 — Reference pack | `InProgress` | All content families are normalized; final pack indexing, coverage and mechanic fixtures remain. |
| Phase 2 — Sora schema | `Pending` | — |
| Phase 3 — Excel authoring | `Pending` | — |
| Phase 4 — Review/freeze | `Pending` | — |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G03-P0-B1` | `Complete` | This row's containing commit | Frozen Version 4.4 / 2026-07-22 main-world scope, nine-World/nine-Path public boundary, pinned source revisions, required/excluded categories, evidence/quality policy, normalized pack and Excel/Sora promotion contracts. Added the 28-batch plan, persistent ledger and launch prompt; both prior release contracts and source-policy checks pass. |
| `G03-P0-B2` | `Complete` | This row's containing commit | Expanded the pinned sparse cache with every Rogue Excel table and Rogue battle/level ability file. Deterministic inventory `4c45418e…8972` hashes 372 files: 35 Standard candidates, 17 shared/reachability-review tables, 165 other-mode tables, 17 presentation/account tables and 138 mechanic-evidence files. The generator's `--check` mode rejects drift; classification is table-level only and does not pre-approve shared rows. |
| `G03-P0-B3` | `Complete` | `b08a632` | Expanded mechanic evidence to 2,646 pinned files (`1d5c7b03…99e7d`) and introduced the reproducible content manifest. P1 reviews corrected the initial denominator by excluding Curio DLC copies and `811/812/813` DLC room families. The current manifest is 1,935 rows (`1dac0f81…ce216`). Membership resolves 33 World difficulties, nine Paths, 36 Resonance/Formation rows, 162 Blessings with 324 levels, 61 Curios/base effects, 59 Occurrences with 55 base variants, nine domains, 42 Ability Tree nodes, 88 service/bonus source rows and the reachable 74-group/171-distinct-member encounter set. |
| `G03-P0-B4` | `Complete` | This row's containing commit | Frozen the machine-readable normalized schema for 25 pack files, stable common/provenance envelopes, explicit definition/level/state/variant separation, five evidence-quality labels, canonical decimal/JSON rules and semantic review-fixture shape. Added a deterministic contract verifier covering category counts, ID uniqueness, row evidence/locators and the 9/9/162/324/36 denominators; documentation now records validation order and the JSON-to-Excel/Sora-only promotion boundary. |
| `G03-P1-B1` | `Complete` | This row's containing commit | Added deterministic normalized-pack bootstrap modules and generated 9 Worlds, 33 difficulty profiles, 9 domain kinds and 579 map nodes. P1-B6's reachability review corrected rooms from 669 shared-table rows to 163 Standard rooms while retaining Standard Adventure rooms. Records carry stable Starclock IDs, bilingual names/summaries, exact source IDs, row-level provenance and evidence digests. Difficulty rows retain recommended levels/elements, score curves and Goal 01 enemy-variant bindings; topology retains source-semantic edge order and exact room content/section maps. Regeneration and `--check` are byte-identical. |
| `G03-P1-B2` | `Complete` | This row's containing commit | Normalized all nine selectable Paths and 36 Path Resonance/Resonance Formation definitions. Paths retain exact Aeon/display identity, source buff type/groups, unlock reference, three formation-selection thresholds, energy defaults and the exact 18-Blessing membership. Resonances retain kind, threshold, energy policy, released modifier/binding keys, ordered exact parameter vectors, description digests and mechanic tags without redistributing source prose. Bilingual and provenance checks plus deterministic regeneration pass. |
| `G03-P1-B3` | `Complete` | This row's containing commit | Normalized the exact 18-Blessing pool for each of nine Paths (162 definitions) and both authored levels for every Blessing (324 level rows). Definitions retain rarity, prerequisites, pool/source tags, extra effects, rule IDs and content-specific mechanic tags. Level rows retain ordered canonical-string parameters, modifier/binding keys and bilingual source-description digests; no source prose or binary float enters the pack. Count, uniqueness, parameter transport and byte-regeneration checks pass. |
| `G03-P1-B4` | `Complete` | This row's containing commit | Corrected the source boundary so `1000/3000` RogueMiracle rows remain Swarm/Gold evidence instead of fake Standard states. Normalized all 61 Standard Curios and 67 lifecycle states: one base active/effect state each plus explicit repairing/fixed phases for six Error Code Curios. Rows retain exact parameter/display vectors, effect IDs, lifecycle charge/transition fields, polarity and mechanic tags, rule references and description digests. Deterministic checks prove every definition/state reference and the 61/67 denominator. |
| `G03-P1-B5` | `Complete` | This row's containing commit | Normalized all 59 CosmosRogue handbook Occurrences, 55 unique base NPC graphs represented by 67 occurrence-variant bindings, and 321 ordered conditional choices. The importer follows released handbook → NPC → dialogue/option graph references, hashes exact bilingual choice/result text, classifies costs/outcome kinds/targets/numeric literals/chances and retains unlock conditions. 269 choices are `ExactPublicText`; 52 random outcomes without released weights carry an explicit `ProjectPolicy` stable-selection rule and replacement condition. No dialogue prose is committed. |
| `G03-P1-B6` | `Complete` | This row's containing commit | Normalized Cosmic Fragments, Blessing-choice resets, Reviver, Downloader, Respite offers, Blessing enhancement, nine Standard shops and all 79 Trailblaze bonuses into 94 implementation-facing rows. Exact released constants and structured rows retain provenance; public service prices and limits carry dated page-level cross-checks. The same review corrected shared-table membership to 163 Standard rooms, nine shops and the reachable 74-group/171-member encounter set by excluding `811/812/813` DLC room families. Manifest and pack regeneration checks pass. |
| `G03-P1-B7` | `Complete` | This row's containing commit | Normalized all 42 Ability Tree nodes as an explicit prerequisite DAG by reversing the released successor edges. Every node retains upgrade-point cost, external unlock IDs, exact source parameter vectors and description digests. Thirty-two nodes contribute battle rules, four contribute both run and battle rules, and six contribute run rules; typed operations cover flat/ratio stat changes, Path Resonance thresholds/damage/energy, service unlocks, starting currency, reward-choice limits, full Energy and consumables. No node is reward-only or omitted. |
| `G03-P1-B8` | `Complete` | This row's containing commit | Normalized 74 reachable weighted encounter groups with 173 candidate references and exact StageConfig wave compositions, resolving 538 enemy slots to Goal 01 enemy variants without gaps. Added 92 battle-room pools, retaining source condition keys and distinguishing random group resolution from World-difficulty elite/boss bindings. Corrected Domain — Encounter to an external-command nonbattle domain; only combat, elite and boss rooms hand off battles. Group, RogueMonster, StageConfig and room provenance is closed and regeneration is byte-identical. |
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
| Difficulties/topology | 33 difficulties / 742 map-room rows | 33 / 742 | 33 / 742 | 579 map nodes and 163 Standard rooms, including Standard Adventure rooms whose outcomes remain external commands. |
| Paths | 9 | 9 | 9 | Main-world selectable Paths. |
| Resonances/Formations | 36 | 36 | 36 | Four per Path; Interplays are excluded. |
| Blessings/upgrades | 162 / 324 levels | 162 / 324 | 162 / 324 | Exactly 18 Blessings and two levels per Path. |
| Curios/states | 61 / 61 source effects / 67 normalized states | 61 / 61 / 67 | 61 / 61 / 67 | CosmosRogue type 100; six Error Codes derive repairing/fixed phases from one released effect row each. |
| Occurrences/choices | 59 / 55 source variants | 59 / 55 | 59 / 55 source variants / 67 bindings / 321 choices | Shared base NPC graphs may bind multiple handbook entries; hidden random weights use explicit policy. |
| Services/currency rules | 88 source rows / 94 normalized rows | 88 / 94 | 88 / 94 | 79 run bonuses plus nine Standard shops; six normalized currency/device/service rules use released constants or dated public cross-checks. |
| Ability Tree | 42 | 42 | 42 | 32 Battle, four RunAndBattle and six Run nodes; exact effect contributions and the prerequisite DAG are normalized. |
| Encounter pools | 74 groups / 171 distinct members | 74 / 171 | 74 groups / 173 member references / 92 room pools | Directly reachable from Standard combat/elite/boss rooms; 538 authored enemy slots resolve to Goal 01 variants. |
| Mechanic fixtures | Pending | 0 | 0 | Distinct shared families. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-22 | Freeze Version 4.4 main-world Standard SU separately from every DLC profile. | Shared Rogue tables cannot be treated as proof of main-world pool membership. |
| 2026-07-22 | Use normalized JSON only for research/bootstrap; production authoring is Excel/Sora. | Preserves the established configuration architecture. |
| 2026-07-22 | Use Python `openpyxl` for workbook authoring/inspection. | Explicit user direction; Sora retains schema/export authority. |
| 2026-07-22 | Include exact or labeled approximate mechanics, not account rewards/story/assets. | Keeps the package implementation-ready and legally bounded. |
| 2026-07-22 | Treat RogueMiracle `1000/3000` rows as DLC copies, not Standard Curio states. | Their effect-display IDs and mode prefixes bind them to Swarm Disaster and Gold and Gears; Standard lifecycle phases are derived from the base effect program instead. |
| 2026-07-22 | Treat `811/812/813` RogueRoom families and shop IDs at or above `200000` as DLC-owned. | Shared base tables mix mode families; only the `800/803/810` Standard room families and nine Standard shop rows are reachable from the frozen main-world profile. |
| 2026-07-22 | Treat Domain — Encounter as an external noncombat decision. | Its room content resolves event/NPC outcomes rather than `RogueMonsterGroup`; battle handoff is limited to combat, elite and boss domains. |

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G03-R01` | `Resolved` | Concrete membership is frozen by source schedule, CosmosRogue type, canonical ID family, base NPC prefix and room-content reachability rules in `content-manifest.json`. | P0-B3 |
| `G03-R02` | `Resolved` | All released base NPC/option graphs are imported. Fifty-two choice outcomes mention randomness without exact weights and are labeled `ProjectPolicy: StableUniformOrderedCandidates` with a replacement condition. | P1-B5 |
| `G03-R03` | `Open` | Which Curio effects require battle ability-program inspection or explicit policy? | P1-B4 |
| `G03-R04` | `Resolved` | All 42 nodes affect battle and/or run state: 32 Battle, four RunAndBattle and six Run; no reward-only node exists in the released base tree. | P1-B7 |

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
