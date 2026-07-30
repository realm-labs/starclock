# Goal 16 Status — Galactic Baseballer Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `galactic-baseballer-reference-v1` |
| State | `Active` |
| Active phase | Phase 2 — Demon King differences and progression |
| Active batch | `G16-P2-B1` complete in this ledger's containing commit |
| Next unblocked batch | `G16-P2-B2` |
| Snapshot | Version 4.4 / structured-source access 2026-07-22 |
| Profiles | Version 2.2 Departure and Version 3.3 Demon King, modeled over one shared base |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Focused inventory | 81 files: 41 Departure/shared candidates, 23 Demon King candidates, 10 shared closure seeds and 7 identity cross-checks; canonical SHA-256 `2430f3f2…525` |
| Localization locators | 1,739 candidate hashes and 3,403 CHS/EN receipts; canonical SHA-256 `a1e08463…1ce` |
| Public sources | 5 publisher pages and 11 revision-pinned community mechanical pages; canonical SHA-256 `8560c2be…7fa` |
| Exact denominator | 2,232 obligations: 2,207 DataReady targets and 25 EvidenceOnly reward/presentation locators |
| Shared reachability | 22 stages, 22 infinite-stage groups, 74 waves, 74 monster groups, 88 enemy variants, 70 templates, 287 skills and 10 statuses |
| Authoring contract | 40 normalized file families mapped exactly once to 4 isolated workbooks; `openpyxl==3.1.5`, Sora 0.3.0 and per-sheet/per-column visual review fixed |
| Semantic contract | 20 ReferenceOnly fixture families and 8 explicit ProjectPolicy boundaries |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl==3.1.5`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal16-galactic-baseballer-reference` |
| Branch base | `0191cc71b1735d6e101e6e04817181423c599232` |
| Parallel condition | Goal 15 was already reserved by `codex/goal15-pure-fiction-reference`; Goal 16 uses a separate worktree, branch and six isolated roots |
| Publication policy | Push and remotely verify each completed batch before starting the next |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Complete` | Foundation, inventories, profiles, exact denominator, normalized schema, Excel/Sora authoring contract, semantic fixture contract and initial approximation register are frozen. |
| Phase 1 — Departure and shared base | `Complete` | Departure profile, stages, arsenal, Legendary synthesis, growth, candidates, inventory, encounters, stable enemy/skill identities, score/settlement and 17 semantic families are closed. |
| Phase 2 — Demon King and progression | `InProgress` | The independent Demon King profile, release/correction boundaries, all seven stage rows, 56 periods and complete constant difference index are closed; P2-B2 owns arsenal and advanced synthesis. |
| Phase 3 — Evidence, Excel and Sora | `Pending` | Requires complete normalized profile data and closed evidence owners. |
| Phase 4 — Audit and Candidate freeze | `Pending` | Requires all earlier phase gates. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G16-P0-B1` | `Complete` | This row's containing commit | Goal 15 was proven occupied and the work renumbered consistently to Goal 16. Created and dry-run-push-verified `codex/goal16-galactic-baseballer-reference`; proved separate worktree ownership, clean Version 4.4 caches at both pinned revisions, readable source trees and remote connectivity. Froze 20 atomic batches, two non-overwriting profiles, six isolated roots, protected historical/generated roots, official-release-only evidence, openpyxl/Sora authority, 20 semantic families and Candidate-only scope. |
| `G16-P0-B2` | `Complete` | This row's containing commit | Added a copy-on-write isolated fixed-source reproducer and froze 81 exact Git-blob receipts: 64 `EvolveBuild`/`EvoBdSC` candidate files, 10 shared stage/battle/enemy/TextMap seeds and 7 StarRailRes cross-checks. The 29 dedicated tables expose 697 original/shared plus 831 Demon King rows; candidate programs bring the discovery total to 1,653 JSON rows/objects. Reconciled 1,739 exact hash owners to 3,403 CHS/EN locators without committing prose. Pinned 5 publisher pages and 11 MediaWiki revisions. All inventories regenerated without drift. |
| `G16-P0-B3` | `Complete` | This row's containing commit | Froze 697 Departure and 831 Demon King dedicated-table rows without cross-profile name/ID inference; retained 35 reference programs, 20 semantic obligations and 25 EvidenceOnly reward/presentation locators. Exact stable-ID recursion closes 22 stages, 22 infinite groups, 74 waves, 74 monster groups, 88 enemy variants, 70 templates, 287 skills and 10 statuses. All 2,232 obligations carry source locators and digests. Three legacy stage references and nine unmatched effect IDs remain counted with explicit replacement boundaries. |
| `G16-P0-B4` | `Complete` | This row's containing commit | Froze 40 normalized file families, a canonical lossless encoding, typed row/evidence/approximation envelopes, 4 complete isolated workbooks, openpyxl 3.1.5 and Sora 0.3.0 authority, no-overwrite/double-generation requirements and per-sheet/per-column visual QA. Reconciled all 20 semantic families to explicit trigger/owner/precondition/input/operation/expected-fact fixture contracts. Registered 8 ProjectPolicy boundaries, each with two rejected alternatives, rationale, affected fixtures, confidence and replacement condition. |
| `G16-P1-B1` | `Complete` | This row's containing commit | Authored independent `galactic-baseballer.departure.v2_2` with released/baseline versions, source season, activity module and exact entry unlock. Separated permanent mechanical retention from limited account-reward locators. Mapped all six bilingual planets/stages and all 57 stage-period rows exactly once, including phase lists, initial weapons, recommendations, team bonuses, rating thresholds, waves, timers, weaknesses and scores. Kept `3097`–`3099` as explicit unresolved shared-stage boundaries. |
| `G16-P1-B2` | `Complete` | This row's containing commit | Authored 13 Standard and 13 Legendary weapons, 16 accessories, all 117 weapon and 64 accessory levels, and exact battle-program structural bindings without copying raw programs. Reconciled every GearConfig row to a distinct `(MazeBuffID, Level)` row and retained complete canonical parameter vectors. Authored 13 exact acyclic Legendary recipes with 26 ordered prerequisites; each consumes its level-8 Standard weapon and retains its level-1 accessory. The 67 non-gear MazeBuff rows remain in the frozen denominator for later batches. |
| `G16-P1-B3` | `Complete` | This row's containing commit | Froze exact `expForLevel=40`, wave/level scaling and `2/4/8/0/0` enemy experience values with structural program evidence. Authored all 11 Adventure Strategy candidates and exact card/MazeBuff/program bindings. Retained the `18,6,3,3,7,6,2,0,2,0,7` source weight vector without guessing its ordinal mapping; froze 3 refreshes, 2 exclusions, 0 card refreshes and unlock quests. Authored Standard 4/5 weapon and 4/6 accessory slots plus Origin-stage 3/4 capacities. Added five explicit ProjectPolicy inventory operations covering duplicates, maximum/full rejection, expansion and failure invariance. |
| `G16-P1-B4` | `Complete` | This row's containing commit | Closed five reachable shared StageConfig rows, five infinite groups, 17 waves, 17 monster groups and 204 ordered enemy candidates. Resolved all 27 MonsterIDs and 81 SkillIDs exactly to frozen Version 4.4 stable identities without copying core definitions; the reachable MonsterStatus set is exactly zero. Froze exact score/weight/time/cap/final-bonus vectors, program structure and six ordered rating/settlement rows. Added 17 ReferenceOnly rules and 17 concrete semantic fixtures; Twin, Supreme and Galactic Store remain explicit Phase 2 families. |
| `G16-P2-B1` | `Complete` | This row's containing commit | Authored independent `galactic-baseballer.demon-king.v3_3` with released Version 3.3 entry requirements, Version 4.4 retention, shared activity module, explicit unlock locators and a non-replacement edge to Departure. Separated permanent mechanics, limited account rewards and released 3.4 corrections. Mapped one Origin plus six challenge stages and all 56 stage periods exactly once with zero unresolved shared StageConfig references. Compared all 83 normalized constants: 38 repeated values, 25 changed, 13 added and 7 Departure-only. Froze RuinBot Lv7/Lv8 and D007 score correction boundaries without reconstructing unpublished erroneous behavior; retained the Boothill visual fix as EvidenceOnly. |
| `G16-P2-B2` | `Pending` | — | Demon King content and advanced synthesis. |
| `G16-P2-B3` | `Pending` | — | Strategies, currencies, store and progression. |
| `G16-P2-B4` | `Pending` | — | Demon King encounters, score and fixtures. |
| `G16-P3-B1` | `Pending` | — | Evidence and 100% DataReady closure. |
| `G16-P3-B2` | `Pending` | — | Complete Excel authoring and visual QA. |
| `G16-P3-B3` | `Pending` | — | Sora schema, template, lock and double generation. |
| `G16-P3-B4` | `Pending` | — | Binary/debug exports and standalone reader. |
| `G16-P4-B1` | `Pending` | — | Full semantic execution and invariance review. |
| `G16-P4-B2` | `Pending` | — | Profile/shared identity/synthesis/isolation audit. |
| `G16-P4-B3` | `Pending` | — | Full and clean-checkout acceptance. |
| `G16-P4-B4` | `Pending` | — | Candidate freeze and terminal publication. |

## Fixed coverage categories

The exact row denominators are frozen by `G16-P0-B3`; they must not be inferred
from this planning list or reduced later.

| Category | Required closure |
|---|---|
| Profiles/releases | Departure, Demon King, permanent/limited boundaries and selection/unlock edges |
| Stages | planets, difficulties, bonuses, initial weapons, recommendations and objectives |
| Encounters | phases, waves, slots, enemies, elites, bosses, escalation and time limits |
| Growth | experience, levels, thresholds, offers, refresh/skip/exclusion and failure |
| Inventory | weapon/accessory slots, expansion, duplicates, maxima and replacement |
| Weapons | every level, parameter, trigger, target, action, counter and cooldown |
| Accessories | every level, contribution and resonance |
| Synthesis | Legendary, Twin, Esteemed/Ultimate and graph validation |
| Strategy/progression | Adventure Strategies, reputation, currency and store upgrades |
| Settlement | scoring terms, ratings, clears, boss damage and final settlement |
| Evidence/review | provenance, reconciliation, approximation, rules, fixtures and coverage |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-30 | Use Goal 16 and `G16-*` IDs. | Goal 15 is already checked out on `codex/goal15-pure-fiction-reference`; duplicate numbering would collide with the parallel Goal contract. |
| 2026-07-30 | Model Departure and Demon King as two profiles over a shared base. | Released Version 4.4 text exposes both and explicitly provides access to Departure through Demon King; Demon King also changes synthesis and progression rules rather than replacing the earlier edition. |
| 2026-07-30 | Treat source prefixes, display names and ID adjacency as discovery only. | Membership and synthesis require an explicit released selector/reference or public fact; this prevents cross-profile and cross-mode leakage. |
| 2026-07-30 | Keep the result Candidate/reference-only. | Runtime semantics, Activity orchestration and combat formulas are outside this Goal. |
| 2026-07-30 | Use stable source order and labeled RNG only in explicit policy rows when released weights/order are unavailable. | Hidden random behavior cannot be labeled exact; every policy remains replacement-tracked. |
| 2026-07-30 | Freeze `EvolveBuild*` and `EvoBdSC*` as candidate source families, not automatic profile membership. | The dedicated families expose the correct mechanics and sequel deltas, but shared tables, retained copies and disabled rows still require selector/reachability proof in P0-B3. |
| 2026-07-30 | Admit shared stage/wave/enemy records only through exact recursive stable-ID fields. | This closes the combat-content dependency chain without copying shared records or treating similar names and ID ranges as membership evidence. |
| 2026-07-30 | Retain reward and presentation rows as counted `EvidenceOnly` locators. | Account rewards, story and presentation stay outside the simulation core, while exact-once accounting prevents silent denominator reduction. |
| 2026-07-30 | Map all 40 normalized files exactly once into four isolated workbooks. | This prevents orphaned staging data, cross-workbook ambiguity and accidental loading of JSON/Excel at runtime. |
| 2026-07-30 | Require labeled integer RNG and explicit failure traces in semantic fixtures. | Candidate selection, target ties and rejected operations must be replay-safe even while their hidden source behavior remains policy-bound. |
| 2026-07-30 | Treat pinned Version 4.4 rows as the authoritative post-3.4 Demon King state. | The official correction notice names RuinBot Lv7/Lv8 and D007 scoring defects but does not publish the erroneous values or trigger; reconstructing them would misstate released evidence. |

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G16-R01` | `Closed` | The focused closure is 81 files: 64 mode-family tables/programs, 10 shared seeds and 7 identity cross-checks. Row-level membership remains intentionally owned by P0-B3. | P0-B2 |
| `G16-R02` | `Closed` | The manifest freezes 2,232 exact obligations: 1,528 dedicated table rows, 35 programs, 647 explicit shared rows, two profiles and 20 semantic families. | P0-B3 |
| `G16-R03` | `Open` | Which offer weights, orders, refresh/skip/exclusion rules and no-candidate behaviors are not publicly structured? | P0-B4 / P1-B3 |
| `G16-R04` | `Open` | What exact recipe edges and consumption/order rules define Legendary, Twin and Esteemed/Ultimate synthesis? | P1-B2 / P2-B2 |
| `G16-R05` | `Open` | How do stage phases, waves, elite objectives, bosses and scoring bind to StageConfig and shared enemies? | P1-B4 / P2-B4 |
| `G16-R06` | `Open` | Which reputation/store/reward rows change mechanics and which are account-only locators? | P2-B3 |
| `G16-R07` | `Closed` | Version 3.4 fixes RuinBot Lv7/Lv8 effects and abnormal D007 Adventure Score mechanically; Version 4.4 rows are the retained corrected state. The Boothill Ultimate visual fix is EvidenceOnly. No later released mechanical correction was found in the frozen official-source inventory. | P2-B1 |

## Phase 0 verification

| Check | Result |
|---|---|
| Isolated source regeneration | `fetch-sources.sh` reproduced both clean fixed revisions; inventory, public-source and manifest verification passed. |
| Contract double generation | All four generated contracts retained byte-identical SHA-256 values and passed `verify-contracts.mjs`. |
| Quick repository gate | `fnm exec --using 24.15.0 node tools/repository-check/run.mjs` passed. |
| Full repository gate | `fnm exec --using 24.15.0 node tools/repository-check/run.mjs --full` passed after installing checksum-bound Sora 0.3.0 in the ignored repository tool cache and supplying the bundled `openpyxl==3.1.5` Python through a temporary PATH alias; 138 test harnesses passed. |
| Protected outputs | No Standard, other-mode or production generated root changed. |

## Phase 1 verification

| Check | Result |
|---|---|
| Fixed-source regeneration | Both pinned caches reproduced; inventory, public-source and 2,232-record manifest checks passed. |
| Departure double generation | Profile, arsenal, growth, encounter and fixture outputs were regenerated and remained byte-identical. |
| Focused verification | All five Departure verifiers passed: profiles/stages, arsenal/synthesis, growth/inventory, encounter/score and 17 semantic families. |
| Quick repository gate | `fnm exec --using 24.15.0 node tools/repository-check/run.mjs` passed. |
| Full repository gate | The same fixed Sora 0.3.0 and temporary bundled `openpyxl==3.1.5` Python PATH used at Phase 0 passed `node tools/repository-check/run.mjs --full`; generated drift, Clippy and 138 test harnesses passed. |
| Protected outputs | No Standard, other-mode or production generated root changed. |

## Terminal checklist

- [ ] Exact profile/category manifests and denominators are frozen.
- [ ] Both profiles and all differences are explicit and independently reconcilable.
- [ ] Focused source inventory regenerates deterministically.
- [ ] Every weapon/accessory level, trigger and synthesis edge closes.
- [ ] Every stage/phase/wave/enemy/boss/bonus/score rule closes.
- [ ] Growth, candidates, slots, refresh, skip, replacement and failure close.
- [ ] Mechanical progression closes and account rewards remain excluded.
- [ ] Every required row has bilingual summary and row-level provenance.
- [ ] All approximations/policies include alternatives and replacement conditions.
- [ ] All 20 mechanism families have rules and executable fixtures.
- [ ] Isolated Sora schemas/templates/readers/exports regenerate without drift.
- [ ] Complete openpyxl workbooks pass structural, semantic and visual QA.
- [ ] Standalone readers load every table and every row.
- [ ] Standard, other-mode and production runtime bundle identities remain unchanged.
- [ ] Coverage is 100% DataReady with no blocking research row.
- [ ] Full source-cache and clean-checkout acceptance pass.
- [ ] Every batch commit is pushed and remote-verified.
- [ ] Candidate evidence and terminal `G16-P4-B4` commit are frozen.

## Completion record

Pending `G16-P4-B4`.
