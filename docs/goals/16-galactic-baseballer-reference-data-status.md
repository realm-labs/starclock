# Goal 16 Status — Galactic Baseballer Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `galactic-baseballer-reference-v1` |
| State | `Active` |
| Active phase | Phase 0 — Scope, versions, sources, denominator and contracts |
| Active batch | `G16-P0-B1` complete in this ledger's containing commit |
| Next unblocked batch | `G16-P0-B2` |
| Snapshot | Version 4.4 / structured-source access 2026-07-22 |
| Profiles | Version 2.2 Departure and Version 3.3 Demon King, modeled over one shared base |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
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
| Phase 0 — Scope, sources and contracts | `InProgress` | `G16-P0-B1` freezes the Goal contract and startup audit; focused source inventory begins in P0-B2. |
| Phase 1 — Departure and shared base | `Pending` | Requires frozen P0 denominator and authoring contract. |
| Phase 2 — Demon King and progression | `Pending` | Requires shared-base identity and explicit edition-difference contract. |
| Phase 3 — Evidence, Excel and Sora | `Pending` | Requires complete normalized profile data and closed evidence owners. |
| Phase 4 — Audit and Candidate freeze | `Pending` | Requires all earlier phase gates. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G16-P0-B1` | `Complete` | This row's containing commit | Goal 15 was proven occupied and the work renumbered consistently to Goal 16. Created and dry-run-push-verified `codex/goal16-galactic-baseballer-reference`; proved separate worktree ownership, clean Version 4.4 caches at both pinned revisions, readable source trees and remote connectivity. Froze 20 atomic batches, two non-overwriting profiles, six isolated roots, protected historical/generated roots, official-release-only evidence, openpyxl/Sora authority, 20 semantic families and Candidate-only scope. |
| `G16-P0-B2` | `Pending` | — | Focused dual-edition source inventory. |
| `G16-P0-B3` | `Pending` | — | Profile membership and exact denominators. |
| `G16-P0-B4` | `Pending` | — | Normalized/authoring/fixture contracts. |
| `G16-P1-B1` | `Pending` | — | Departure profile and stages. |
| `G16-P1-B2` | `Pending` | — | Departure weapons, accessories and synthesis. |
| `G16-P1-B3` | `Pending` | — | Departure growth loop and inventory boundaries. |
| `G16-P1-B4` | `Pending` | — | Departure encounters, score and fixtures. |
| `G16-P2-B1` | `Pending` | — | Demon King profile and version differences. |
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

## Research cases

| ID | State | Question | Owner |
|---|---|---|---|
| `G16-R01` | `Open` | Which Version 4.4 tables/configuration programs form the exact two-profile selector and content closure? | P0-B2 |
| `G16-R02` | `Open` | What are the exact per-category denominators and shared-versus-profile-owned identities? | P0-B3 |
| `G16-R03` | `Open` | Which offer weights, orders, refresh/skip/exclusion rules and no-candidate behaviors are not publicly structured? | P0-B4 / P1-B3 |
| `G16-R04` | `Open` | What exact recipe edges and consumption/order rules define Legendary, Twin and Esteemed/Ultimate synthesis? | P1-B2 / P2-B2 |
| `G16-R05` | `Open` | How do stage phases, waves, elite objectives, bosses and scoring bind to StageConfig and shared enemies? | P1-B4 / P2-B4 |
| `G16-R06` | `Open` | Which reputation/store/reward rows change mechanics and which are account-only locators? | P2-B3 |
| `G16-R07` | `Open` | Which Version 3.4+ released corrections are part of retained Version 4.4 behavior? | P2-B1 / P3-B1 |

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
