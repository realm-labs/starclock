# Goal 13 Status — Anomaly Arbitration Reference Data

## Goal state

| Field | Value |
|---|---|
| Goal ID | `anomaly-arbitration-reference-v1` |
| State | `Ready` |
| Active phase | Not started |
| Active batch | — |
| Next unblocked batch | `G13-P0-B1` |
| Snapshot | Version 4.4 / inherited structured-source access 2026-07-22 |
| Structured source | `turnbasedgamedata@fd978d6ef09f941fba644c731ab54abd6f7c3568` |
| Identity cross-check | `StarRailRes@7b349e39ee0f6f3bf814567995829b99c95e7a93` |
| Planning cache audit | 2026-07-29: both caches clean/detached at pinned commits; origins, commit/blob readability and connectivity verified; execution must reproduce in `G13-P0-B1` |
| Starting source oracle | 6 `ChallengePeak*` tables, direct ChallengePeak battle-event ability, indirect gameplay/config closure, CHS/EN TextMaps, StageConfig and transitive shared target/MazeBuff/enemy sources |
| Active-period hypothesis | Group `8`, aliases `801`–`804` and stages `30508011`, `30508012`, `30508013`, `30508021`, `30508022`; not a denominator until `G13-P0-B3` proves the selector chain |
| Focused inventory | Pending `G13-P0-B2` |
| Content manifest | Denominators pending `G13-P0-B3` |
| Content lane | `Experimental`; target reference bundle `Candidate` |
| Workbook adapter | Python `openpyxl`; Sora 0.3.0 remains authoritative |
| Remote | `origin` |
| Branch | `codex/goal13-anomaly-arbitration-reference` |
| Branch base | `b0cd3cb912c9f2ec887c3ae29f79353c4a861643` (`G11-SETUP`; excludes every concurrent worktree's later or uncommitted data) |
| Parallel condition | Separate branch/worktree and six isolated Goal 13 roots while Goals 07 through 12 are active |
| Publication policy | Push and remotely verify each completed batch before starting the next batch |
| Blocking condition | None |

## Phase ledger

| Phase | State | Evidence |
|---|---|---|
| Phase 0 — Scope, sources and contracts | `Pending` | Awaiting execution-owned cache reproduction, focused inventory, active-period manifest and authoring contracts. |
| Phase 1 — Unique mode systems | `Pending` | Awaiting Knight records/uniqueness, King protection/Plight, clocks, Arbitral Quadrant, stars and settlement. |
| Phase 2 — Content and encounters | `Pending` | Awaiting pool zero/nonzero proofs, targets, traits, battle events, stages, waves, enemies and bosses. |
| Phase 3 — Sora and Excel | `Pending` | Awaiting isolated schemas/readers, three complete workbooks, deterministic exports and visual QA. |
| Phase 4 — Review and freeze | `Pending` | Awaiting ownership audit, semantic fixtures, reconciliation, regeneration and clean-checkout evidence. |

## Batch ledger

| Batch | State | Commit | Result/evidence |
|---|---|---|---|
| `G13-P0-B1` | `Pending` | — | Reproduce caches, verify Goal 03 and concurrent Goal boundaries, freeze scope and prove isolation. |
| `G13-P0-B2` | `Pending` | — | Inventory dedicated/shared tables, indirect config/ability programs, TextMaps, StageConfig, enemies and exclusions. |
| `G13-P0-B3` | `Pending` | — | Freeze active-period selectors, exact obligations/counts, ownership, reachability, proven-empty pools and exclusions. |
| `G13-P0-B4` | `Pending` | — | Freeze normalized schema, evidence, canonical encoding, workbook, reconciliation and fixture contracts. |
| `G13-P1-B1` | `Pending` | — | Import profile, active period, entry/eligibility, stages, legal order and terminal outcomes. |
| `G13-P1-B2` | `Pending` | — | Import Knight teams, uniqueness, records, replacement/reset and current-versus-best progress. |
| `G13-P1-B3` | `Pending` | — | Import King protection, normal/Plight states, shortcut and transition order. |
| `G13-P1-B4` | `Pending` | — | Import clocks, first-cycle window, wave carry, warnings, low-cycle effects and failure/retry. |
| `G13-P1-B5` | `Pending` | — | Import active Arbitral Quadrant offers, selection, parameters and contributions. |
| `G13-P1-B6` | `Pending` | — | Import targets, stars, evaluation, aggregation, settlement and cross-battle projection. |
| `G13-P2-B1` | `Pending` | — | Audit/freeze reachable or exact-zero Blessing, Curio, Occurrence, service, currency and related pools. |
| `G13-P2-B2` | `Pending` | — | Import stage definitions, shared battle targets, objectives and challenge events. |
| `G13-P2-B3` | `Pending` | — | Import enemy traits, King/Plight modifiers, Quadrant MazeBuffs and transitive ability contributions. |
| `G13-P2-B4` | `Pending` | — | Import StageConfig encounters, waves, enemy slots, variants, skills/AI/abilities and bosses. |
| `G13-P2-B5` | `Pending` | — | Generate mechanics, sources, coverage, research gaps, fixtures and pack index. |
| `G13-P3-B1` | `Pending` | — | Add profile/period/stage/participant/record/progress Sora tables. |
| `G13-P3-B2` | `Pending` | — | Add King/Plight, clock, target, objective, aggregation and Quadrant tables. |
| `G13-P3-B3` | `Pending` | — | Add pool, trait, event, encounter, enemy and contribution binding tables. |
| `G13-P3-B4` | `Pending` | — | Add evidence/coverage/reconciliation/fixture tables and isolated locks/templates/readers. |
| `G13-P3-B5` | `Pending` | — | Generate and structurally/semantically verify all three complete `openpyxl` workbooks. |
| `G13-P3-B6` | `Pending` | — | Prove deterministic Sora export/load and visual review of every sheet. |
| `G13-P4-B1` | `Pending` | — | Audit exact-once coverage, period selection, ownership, references, provenance and bilingual fields. |
| `G13-P4-B2` | `Pending` | — | Execute all semantic fixtures and approximation replacement checks. |
| `G13-P4-B3` | `Pending` | — | Reconcile overlap and run full regeneration, drift, reader, dependency and clean-checkout acceptance. |
| `G13-P4-B4` | `Pending` | — | Freeze final documentation, evidence and Candidate reference-bundle identity. |

For a completed batch, the result/evidence cell must record `remote`,
`branch`, full pushed commit ID, exact push command, remote-resolution
verification command and result. A locally committed but remotely unverified
batch remains `InProgress`.

The Goal package setup commit is identified as “this document's containing
commit” to avoid a recursive self-hash. Its remote, branch, exact push command
and remote-resolution result are reported in the setup handoff.
`G13-P0-B1` records the full setup commit and remote verification as immutable
foundation evidence before any data mutation.

## Frozen counters

Populate required counts only from the generated manifest in `G13-P0-B3`.
Do not estimate denominators from raw table sizes, prefixes, ID ranges,
historical periods or display names. A zero denominator also requires a
generated selector-closure proof.

| Category | Required | Accounted | DataReady | Notes |
|---|---:|---:|---:|---|
| Profile/period/entry/terminal outcomes | TBD | 0 | 0 | Stable family plus active Version 4.4 period; historical periods are evidence-only unless explicitly selected. |
| Stages/attempts/current-best progress | TBD | 0 | 0 | Three Knight stages, one King stage, legal order and progress lifecycles. |
| Participant/team/loadout records | TBD | 0 | 0 | Includes character, Light Cone and Relic-instance uniqueness, snapshots and reset/replacement. |
| King protection/Plight transitions | TBD | 0 | 0 | Includes Knight-clear contributions, normal/Plight state and direct-clear shortcut. |
| Clocks/wave carry/warnings | TBD | 0 | 0 | Includes first-cycle policy, stage-local limit, no wave reset, low-cycle effect and expiry. |
| Arbitral Quadrant offers/options | TBD | 0 | 0 | Includes active offer membership, selection, parameters and teardown. |
| Targets/stars/aggregation | TBD | 0 | 0 | Includes battle targets, per-stage result, current progress and simultaneous best record. |
| Enemy traits/MazeBuffs/battle events | TBD | 0 | 0 | Includes normal, King, Plight and Quadrant contributions with transitive abilities. |
| Blessings/Curios/Occurrences | TBD | 0 | 0 | Freeze exact reachable rows or an exact-zero selector proof for each family. |
| Services/currencies/other pools | TBD | 0 | 0 | Reward/shop presentation does not prove a mechanically reachable service. |
| Encounter groups/waves/enemy slots | TBD | 0 | 0 | Resolve exact StageConfig rows, variants, abilities and boss phases. |
| Mechanic rules | TBD | 0 | 0 | Reference contributions only; no runtime executability claim. |
| Semantic fixtures | TBD | 0 | 0 | Cover every unique mechanic, lifecycle, objective and selection policy. |

## Decisions

| Date | Decision | Rationale |
|---|---|---|
| 2026-07-29 | Create Goal 13 as a complete reference-data package, not a runtime goal. | Anomaly Arbitration research can proceed independently without changing shared challenge, Activity or combat runtime. |
| 2026-07-29 | Use `anomaly-arbitration` as the stable mode slug and reserve Goal 13. | Goal 12 already has an active dedicated branch/worktree; no local Goal 13 package or branch existed during the planning audit. |
| 2026-07-29 | Base the branch on `b0cd3cb9…`, the committed Goal 11 setup package. | The base includes Goal 01–11 indexing while excluding all concurrent worktree changes; Goal 12 remains independently owned. |
| 2026-07-29 | Inherit the pinned Version 4.4 source and identity revisions used by Goals 03 and 08–12. | Shared identity, row ownership and membership comparisons require one reproducible historical boundary. |
| 2026-07-29 | Require `G13-P0-B1` to reproduce both caches even though the planning audit found them clean and readable. | Planning-time availability is not a substitute for batch-owned reproducibility evidence. |
| 2026-07-29 | Treat the six `ChallengePeak*` tables and candidate group/stage IDs only as an inventory seed. | Historical periods, rewards and mechanics share the tables; prefix and ID range do not prove active Version 4.4 membership. |
| 2026-07-29 | Require indirect gameplay/config discovery rather than assume no config exists because `Config/Gameplays` has no direct ChallengePeak path. | StageConfig, BattleEventConfig, MazeBuff and the shared ChallengePeak ability program carry transitive mechanics. |
| 2026-07-29 | Audit Blessing, Curio, Occurrence, service and currency families even when the expected result is empty. | Completeness requires a generated zero proof; absence cannot be inferred from this being a challenge mode. |
| 2026-07-29 | Reconcile shared rows by source path, stable row locator and evidence digest without editing another Goal's artifacts. | Concurrent goals must preserve isolated ledgers and surface conflicts for merge coordination. |
| 2026-07-29 | Exclude presentation and account rewards while retaining mechanical locators. | Keeps the pack implementation-ready and within the project content boundary. |
| 2026-07-29 | Finish at Candidate-quality reference data without a Released runtime claim. | Runtime lowering, shared primitive changes and seeded full runs require a later goal. |
| 2026-07-29 | Require every completed batch commit to be pushed and remotely verified before the next batch begins. | Prevents unpublished local progress from becoming the effective resumable source of truth. |

## Research cases

| ID | State | Question | Owner | Replacement condition |
|---|---|---|---|---|
| `G13-R01` | `Open` | Which direct and transitive table, gameplay/config, TextMap, StageConfig, enemy/wave and ability files complete the focused inventory? | P0-B2 | Replace when generated inventory closes every enabled selector/reference and byte-identical double generation passes. |
| `G13-R02` | `Open` | Which released selector proves the active Version 4.4 period and separates it from seven historical groups and other challenge families? | P0-B3 | Replace with an exact-once manifest whose rows carry active-period selector/reference evidence and fail-closed exclusions. |
| `G13-R03` | `Open` | What are the exact participant, character, Light Cone and Relic identity scopes and the atomic invalidation order when a recorded Knight loadout changes? | P1-B2 | Replace with source-backed slot/uniqueness/reset programs and accepted/rejected/replacement fixtures. |
| `G13-R04` | `Open` | How do three Knight-clear states compose King protection, when is protection removed, and what does direct Plight victory project into current and best records? | P1-B3 | Replace with exact transition programs or field-level policies carrying alternatives and lifecycle fixtures. |
| `G13-R05` | `Open` | What are the first-cycle Action Value, stage-local cycle tick, wave carry, low-cycle trigger and expiry ordering semantics? | P1-B4 | Replace with configuration/ability evidence or explicit deterministic policies for each unresolved boundary. |
| `G13-R06` | `Open` | Which Arbitral Quadrant rows are offered in the active period, how are they selected and when do their battle contributions start and end? | P1-B5 / P2-B3 | Replace with selector-backed offers, exact parameters, contribution programs and selection/teardown fixtures. |
| `G13-R07` | `Open` | How are stars, current progress, simultaneous three-Knight best progress and King results evaluated and retained across reset/retry? | P1-B6 | Replace with typed objective/aggregation rows and fixtures for improvement, regression, reset and Plight shortcut. |
| `G13-R08` | `Open` | Are any Blessings, Curios, Occurrences, services, currencies or analogous pools mechanically reachable in the active period? | P2-B1 | Replace each family with a selector-backed nonzero closure or generated exact-zero proof. |
| `G13-R09` | `Open` | Which active enemy traits, King/Plight modifiers and shared battle events bind to each stage and difficulty? | P2-B2–B3 | Replace with exact stage/target/MazeBuff/event/ability joins and one fixture per distinct contribution. |
| `G13-R10` | `Open` | Which StageConfig waves, concrete enemy variants, skills, AI, phases and difficulty inputs define every active encounter? | P2-B4 | Replace with complete encounter/enemy dossiers or an explicit nonblocking boundary for unavailable released evidence. |
| `G13-R11` | `Open` | Which hidden ordering, timing, caps, rounding and fallback fields remain unavailable after bounded research? | P2-B5 / P4-B2 | Replace each field with exact/observed evidence or a reviewed approximation/project-policy row with a concrete stronger-evidence trigger. |

## Terminal checklist

- [ ] Exact active-period category manifests and denominators are frozen.
- [ ] Both pinned caches and the focused
      table/config/TextMap/Stage/ability inventory regenerate deterministically.
- [ ] Complete normalized pack and canonical pack index regenerate without
      drift.
- [ ] All required rows have bilingual summaries and row-level provenance.
- [ ] Ownership, active-period enablement and shared reachability are explicit
      and fail closed.
- [ ] Empty content-pool families have generated selector-closure proofs.
- [ ] Shared classifications reconcile with committed Goal 07–12 facts.
- [ ] All required mechanics are exact or explicitly
      approximate/policy-bound.
- [ ] Knight records/uniqueness, King/Plight, clocks, Arbitral Quadrant,
      objectives and aggregation have complete semantic fixtures.
- [ ] Encounter identities, StageConfig rows, waves, traits and boss bindings
      resolve.
- [ ] Isolated Sora schemas, templates and generated readers validate.
- [ ] All three complete `openpyxl` workbooks pass structural and visual QA.
- [ ] Sora production/debug exports regenerate without drift and load through
      isolated readers.
- [ ] Goal 03 evidence and all other mode/production bundle identities remain
      unchanged.
- [ ] Coverage reports 100% `DataReady` and no blocking research row.
- [ ] Every completed batch commit is reachable from its recorded remote
      branch at the recorded commit ID.
- [ ] Clean-checkout acceptance passes and `G13-P4-B4` is committed and pushed.

## Completion record

| Field | Value |
|---|---|
| Final state | Pending |
| Completion commit | — |
| Remote/branch verification | — |
| Anomaly Arbitration reference bundle | — |
| Workbook semantic digest | — |
| Coverage | Denominators pending `G13-P0-B3` |
| Release evidence | — |
| Remaining required work | Anomaly Arbitration runtime lowering, integration, handlers, controller/API exposure and seeded full challenge runs belong to a later goal. |
