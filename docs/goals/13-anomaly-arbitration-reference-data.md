# Goal 13 — Anomaly Arbitration Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Anomaly Arbitration reference-data
pack and an isolated Excel/Sora authoring surface before any Anomaly Arbitration
runtime implementation.

The goal covers the stable challenge-family contract and the active Version 4.4
period: entry and eligibility, three Knight stages and their disjoint recorded
teams, King protection and Plight state, record replacement and reset rules,
Arbitral Quadrant selection, independent stage cycle limits, wave carry,
low-cycle effects, stars and best-record aggregation, enemy traits, encounters
and every battle-visible or cross-battle rule contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated readers,
coverage and semantic review fixtures. It does not implement or expose a
playable Anomaly Arbitration profile.

## Start condition

Goal 13 may run while Goals 07 through 12 are active in other worktrees. It is
unblocked when:

- Goal 03 remains `Complete`;
- both pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- the executor uses a separate branch and git worktree from every other active
  Goal checkout; and
- Anomaly Arbitration artifacts use the isolated paths declared below.

Completion of Goals 07 through 12 is not a prerequisite for source collection,
manifest freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 13 must not modify their plans, ledgers, manifests, normalized
rows, workbooks, generated output, runtime lowering or shared
combat/activity/build behavior.

If research discovers a missing shared runtime primitive, record it as a future
Anomaly Arbitration runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and source-reproduction prerequisite

- game/content snapshot: Version 4.4;
- planning audit date: 2026-07-29;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- existing challenge architecture baseline:
  `docs/18-standard-and-challenge-modes.md`;
- existing audit baseline:
  `content-manifests/standard-universe-v1/source-inventory.json`;
- cross-mode ownership inputs: committed Goal 08 through 12 manifests at the
  revisions inspected by `G13-P0-B1`, when available;
- public cross-check access dates: recorded per page during Goal 13 rather than
  silently inheriting an earlier research date.

The planning audit found both ignored caches clean and detached at the required
commits, proved the commit objects and required source blobs readable, verified
their configured origins and ran Git connectivity checks. The
turnbasedgamedata cache is intentionally sparse, but the required
`ChallengePeak` blobs are present in the pinned Git object database.
`G13-P0-B1` must reproduce these checks in its own worktree before the first
Goal data mutation. Planning-time availability is not batch-owned release
evidence.

## Starting source oracle

The pinned structured source contains these six dedicated `ChallengePeak`
tables:

```text
ChallengePeakBossConfig.json
ChallengePeakCommonConst.json
ChallengePeakConfig.json
ChallengePeakGroupConfig.json
ChallengePeakReward.json
ChallengePeakRewardOR.json
```

They are an inventory seed, not the final content denominator. Historical
periods, display fields, account rewards and an empty table can coexist with
active mechanics. A `ChallengePeak` prefix, ID range, matching display name or
table presence does not prove Version 4.4 enablement, ownership or
reachability.

The planning audit identified the following direct configuration and
ability-program seeds:

```text
Config/ConfigAbility/BattleEventAbility_ChallengePeakBattle.json
Config/ConfigAbility/BattleEventAbility_ChallengePeakBattle.layout.json
Config/ConfigAbility/BattleEvent/Camera/BattleEventAbility_ChallengePeakBattle_Camera.json
Config/ConfigAbility/BattleEvent/Camera/BattleEventAbility_ChallengePeakBattle_Camera.layout.json
Config/ConfigCharacter/BattleEvent/BattleEvent_ChallengePeakBattle_Elation_01_Config.json
Config/ConfigCharacter/BattleEvent/BattleEvent_ChallengePeakBattle_Elation_01_ElationConfig.layout.json
Config/Level/StageCommonTemplate.json
```

The two camera files, character-event configuration and layout companions are
evidence inputs only unless a mechanically enabled row references them. The
planning audit found no direct `ChallengePeak` path under
`Config/Gameplays/`; `G13-P0-B2` must prove whether the active flow is fully
selected through tables, StageConfig data and battle-event programs or whether
another indirectly named gameplay program exists.

Candidate shared source families include:

```text
BattleEventConfig.json
BattleTargetConfig.json
MazeBuff.json
StageConfig.json
MonsterConfig.json
MonsterTemplateConfig.json
MonsterSkillConfig.json
MonsterStatusConfig.json
```

The active-period planning hypothesis is `ChallengePeakGroupConfig` row `8`,
its stage aliases `801` through `804`, StageConfig rows `30508011`,
`30508012`, `30508013`, `30508021` and `30508022`, shared battle targets
`3000` through `3005` and `3007`, battle events `30502` through `30504`,
the reachable `30330xx` MazeBuff rows and their transitive ability programs.
This is a source-discovery seed, not a frozen denominator or permission to
admit an ID range. `G13-P0-B3` must either prove each candidate from an active
Version 4.4 selector/reference chain or classify it as evidence-only or
excluded.

The focused inventory must also include:

- `TextMap/TextMapCHS.json` and `TextMap/TextMapEN.json`;
- applicable bilingual `StarRailRes` identity indexes;
- exact StageConfig waves and ordered enemy slots selected by enabled stages;
- every reachable enemy template, variant, skill, status, AI and ability
  record;
- active enemy-trait and Arbitral Quadrant MazeBuff rows and their parameter
  vectors;
- battle target, battle event, clock/countdown, wave and stage-template
  programs invoked by the enabled period; and
- historical Anomaly Arbitration periods and adjacent challenge families only
  as bounded ownership, active-period or exclusion evidence.

Every shared-source reconciliation record uses source path, stable row locator
and evidence digest. The manifest must classify each discovered row as
`AnomalyArbitration`, `Shared`, `EvidenceOnly` or an explicitly named excluded
period/mode. Prefixes, ID adjacency and name equality are never sufficient
ownership or reachability evidence.

## Included content

Completeness is defined by frozen manifests for:

1. stable mode identity, Version 4.4 active-period identity, entry,
   mechanically relevant eligibility, period binding and terminal outcomes;
2. the three Knight stages, one King stage, legal challenge order and the
   distinction between normal King and Plight challenge state;
3. three disjoint Knight teams, participant and combat-form uniqueness,
   Light Cone and Relic-instance uniqueness, loadout snapshots and lock scope;
4. successful Knight records, re-challenge eligibility, same-team loadout
   changes, record replacement choice, progress reset and record erasure;
5. simultaneous best-record calculation, per-stage stars, retained historical
   best score and the distinction between current progress and best progress;
6. King protection contributed by uncleared Knight stages, removal after
   clears, direct Plight-clear shortcut and exact downstream state changes;
7. every active Arbitral Quadrant option, offer set, selection timing,
   eligibility, parameters, battle contribution and no-selection behavior;
8. every stage cycle/turn limit, first-cycle Action Value policy, tick timing,
   wave-transition carry, expiry/failure boundary and low-cycle combat effect;
9. star targets, defeat/survival and cycle conditions, evaluation timing,
   per-stage aggregation and King medal/rating boundary without account
   rewards;
10. every active enemy trait, King enhancement, Plight modifier, parameter,
    trigger, scope, stacking rule and teardown boundary;
11. every enabled Version 4.4 stage, StageConfig row, wave, enemy slot,
    concrete enemy variant, level/difficulty input and boss phase;
12. mode-owned and reachable shared battle targets, MazeBuffs, battle events,
    stage templates, enemies, abilities and mechanically relevant challenge
    definitions;
13. Blessing, Curio, Occurrence, service, currency and other content-pool
    categories, including an exact-zero manifest with selector-closure evidence
    when the active mode exposes none;
14. every battle-visible rule contribution, cross-battle Activity slot,
    decision, projection and lifecycle boundary;
15. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions.

Shared records are referenced by stable Starclock identity only after the
frozen Anomaly Arbitration manifest proves reachability. A source copy with a
mode-specific effect, state, stage binding or display binding remains a
distinct Anomaly Arbitration-owned record.

## Excluded content

- Anomaly Arbitration runtime lowering, handlers, controllers, CLI, Agent, MCP
  or playable challenge flow;
- changes to `starclock-combat`, `starclock-activity`,
  `starclock-mode-challenge`, `starclock-build` or shared runtime semantics;
- Memory of Chaos, Pure Fiction, Apocalyptic Shadow, Simulated Universe modes,
  Currency Wars and historical Anomaly Arbitration periods except as explicit
  ownership, selector or exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, avatar frames, medals, item payloads,
  first-clear rewards, star rewards, account history and collection displays;
- calendar services and wall-clock rotation behavior beyond immutable period
  identity metadata;
- announced, beta, preview, leaked or otherwise unreleased content;
- a seeded runtime golden activity or production compatibility claim.

Quest, reward, schedule, item and historical-period rows may be retained as
provenance locators when necessary to prove an unlock, active-period boundary,
stage relationship or mechanical shortcut. Their story prose, account state,
calendar behavior and reward payloads do not become normalized runtime
content.

## Architecture and artifact isolation

Goal 13 starts in the `Experimental` content lane and may finish with a
complete `Candidate` reference bundle. It does not promote a `Released`
production mode.

Use isolated paths:

```text
content-manifests/anomaly-arbitration-v1/
content-reference/anomaly-arbitration-v1/
config/anomaly-arbitration/
config/anomaly-arbitration-generated/
tools/anomaly-arbitration-reference/
evidence/anomaly-arbitration-reference-v1/
```

The authoring workbooks are mode-owned:

```text
AnomalyArbitration.xlsx
AnomalyArbitrationBindings.xlsx
AnomalyArbitrationReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G13-P0-B4`. They must not share mutable sheets or generated directories with
another Goal or `config/generated/`.

Anomaly Arbitration authoring may reference generic Activity, challenge,
participant, clock, objective, BattleSpec/Result and stable shared-content
concepts. It may not redefine graph execution, command atomicity, build
resolution, combat formulas, RNG, hashing or replay. Reference rows may declare
a future activity, build or battle rule contribution without implementing its
evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released configuration/ability
   programs;
2. official publisher announcements and released in-game text;
3. reproducible live observations;
4. independent public community cross-checks.

Every normalized fact records the repository/URL, exact revision or access
date, game version, relative path/page, row locator, evidence digest, quality,
mechanism quality and note. Allowed labels remain:

- `ExactStructured`;
- `ExactPublicText`;
- `Observed`;
- `ApproximateFromReleasedText`;
- `ProjectPolicy`.

Approximation is field-level. Active-period selection, team-record invalidation
order, equipment identity, King protection composition, clock boundaries,
first-cycle Action Value, wave carry, low-cycle timing, target ordering, caps,
rounding and fallbacks must never be silently inferred. A deterministic
project policy requires known facts, selected behavior, rejected alternatives,
rationale, affected fixtures and a concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G13-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, period, entry, eligibility and terminal outcome;
- stage, stage kind, legal order, attempt and current/best progress;
- participant policy, team slot, roster/loadout record, uniqueness claim,
  replacement and reset;
- King protection, Knight-clear contribution, Plight state and shortcut;
- Arbitral Quadrant option, offer set, selection and contribution;
- clock, cycle window, wave carry, warning threshold, expiry and failure;
- star target, evaluation, stage result and aggregation;
- enemy trait, MazeBuff binding, battle event and battle contribution;
- encounter, StageConfig wave, enemy slot, variant, ability and boss phase;
- audited Blessing, Curio, Occurrence, service and currency pool membership,
  including proven-empty categories;
- mechanic rule, source, reconciliation receipt, coverage, review fixture and
  pack index.

Definitions remain separate from mutable attempts, records, progress,
selections, waves, variants and terminal results. Exact decimals use canonical
strings. Arrays that are sets are sorted by stable ID; stage order, waves,
effect programs and other semantic sequences preserve declared order.

## Parallel execution rules

When Goals 07 through 13 execute concurrently:

- use separate git worktrees and branches;
- Goal 13 owns only the six isolated roots declared above, its three Goal
  documents and its Goal index row;
- do not edit another Goal's plan, ledger, manifest, policy, evidence,
  workbook, normalized content or generated output;
- do not regenerate another mode's generated directory or
  `config/generated/`;
- do not reclassify a shared source record in place; record Goal 13 ownership
  and reachability in the Anomaly Arbitration manifest;
- reconcile overlaps by source path, stable row locator and evidence digest at
  the named Phase 4 checkpoint;
- record incompatible ownership or semantic classifications and wait for merge
  coordination rather than overwriting another Goal; and
- preserve Goal 03 and current production bundle digests.

Reference collection, source hashing, manifests, normalized data, evidence,
fixtures, isolated schemas and isolated workbook QA may proceed in parallel.
Runtime lowering and changes to shared schemas or operations belong to a later
goal after the reference pack is frozen.

## Delivery phases

### Phase 0 — Scope, source files, manifests and contracts

| Batch | Deliverable |
|---|---|
| `G13-P0-B1` | Reproduce both pinned caches, verify Goal 03, freeze scope/exclusions, inspect active Goal 07–12 boundaries and prove branch/worktree/path isolation. |
| `G13-P0-B2` | Generate the focused inventory for all six `ChallengePeak` tables, indirect gameplay/config programs, CHS/EN TextMaps, StageConfig, shared targets/MazeBuffs/battle events, enemies, abilities and named exclusions. |
| `G13-P0-B3` | Freeze the active Version 4.4 period, exact per-category IDs/counts, ownership, shared reachability, proven-empty pools and historical/other-mode exclusions. |
| `G13-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, workbook/table families, reconciliation receipts and semantic fixture contracts. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G13-P1-B1` | Import profile, active period, entry/eligibility, four stage aliases, legal challenge order and terminal outcomes. |
| `G13-P1-B2` | Import three Knight team slots, participant/loadout uniqueness, successful records, re-challenge, replacement, reset and current-versus-best progress. |
| `G13-P1-B3` | Import King protection, Knight-clear contributions, normal/Plight states, direct Plight-clear shortcut and transition ordering. |
| `G13-P1-B4` | Import stage clocks, first-cycle window, wave carry, warnings, low-cycle effects, expiry/failure and retry boundaries. |
| `G13-P1-B5` | Import every active Arbitral Quadrant offer/option, selection policy, parameters, battle contribution and teardown. |
| `G13-P1-B6` | Import star targets, evaluation timing, per-stage results, aggregation, settlement and every cross-battle projection. |

### Phase 2 — Content pools, services, events and enemies

| Batch | Deliverable |
|---|---|
| `G13-P2-B1` | Audit Blessing, Curio, Occurrence, service, currency and other pool families; freeze each reachable set or exact-zero selector proof. |
| `G13-P2-B2` | Import active stage definitions, shared battle targets, normal/Plight objective bindings and challenge event relationships. |
| `G13-P2-B3` | Import active enemy traits, King/Plight modifiers, Arbitral Quadrant MazeBuffs and transitive battle-event/ability contributions. |
| `G13-P2-B4` | Import exact StageConfig encounters, ordered waves, enemy slots, concrete variants, skills/AI/abilities and boss phases. |
| `G13-P2-B5` | Generate mechanics, sources, coverage, research gaps, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking gap. |

### Phase 3 — Isolated Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G13-P3-B1` | Add isolated profile/period/stage/participant/record/progress Sora tables and typed references. |
| `G13-P3-B2` | Add King protection/Plight, clock, target, objective, aggregation and Arbitral Quadrant tables. |
| `G13-P3-B3` | Add audited pool, trait, event, encounter, enemy and mechanic-contribution binding tables. |
| `G13-P3-B4` | Add provenance, coverage, approximation, reconciliation, fixture and pack-index tables; generate isolated locks/templates/readers. |
| `G13-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for all three complete workbooks with validation and semantic QA. |
| `G13-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every sheet. |

### Phase 4 — Ownership audit, fixtures, reconciliation and freeze

| Batch | Deliverable |
|---|---|
| `G13-P4-B1` | Audit every manifest row, active-period selector, reference, ownership, bilingual field, provenance and quality label; reject historical/other-mode leaks. |
| `G13-P4-B2` | Execute semantic fixtures for every unique system, lifecycle, objective, contribution and proven-empty pool; verify every replacement condition. |
| `G13-P4-B3` | Reconcile shared rows with committed Goal 07–12 facts, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G13-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution and commit rules

- Select the earliest unblocked batch and keep only one Goal 13 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(anomaly-arbitration): <batch-id> <imperative summary>`.
- Push every completed batch commit to the configured remote branch
  immediately after commit, verify the remote branch resolves to the same full
  commit ID and record the remote, branch, commands and result in the ledger.
- Do not begin the next batch while the current batch exists only locally.
- Use the pinned source first and public pages only for boundary checks,
  meaning and unresolved observations.
- Use Python `openpyxl` for workbook creation and inspection. Sora 0.3.0
  remains the validation, code-generation and export authority.
- Bootstrap complete workbooks into clean targets; never patch a
  designer-edited `.xlsx` or edit one as a ZIP.
- JSON is research/bootstrap/debug data and never a runtime loading path.
- Record exact commands, counts, digests, research decisions, reconciliation
  receipts and replacement conditions in the status ledger.
- Keep generated denominators machine-derived. Never reduce a denominator or
  assume an empty pool to make coverage pass.

## Acceptance

- both pinned caches and the focused
  table/config/TextMap/Stage/ability inventory regenerate byte-identically;
- concrete active-period category manifests provide 100% exact-once
  accounting;
- every enabled record has bilingual names/summaries and resolvable
  provenance;
- every numeric vector, selector, relationship and lifecycle is exact or
  explicitly approximate/policy-bound;
- all mode-owned/shared/evidence-only/historical/other-mode classifications
  are explicit and fail closed;
- any empty Blessing, Curio, Occurrence, service or currency family is proved
  by selector closure rather than assumed from the mode name;
- Knight team records, uniqueness/reset, King protection/Plight, clocks,
  Arbitral Quadrant, star aggregation and settlement have semantic fixtures;
- all encounters resolve concrete released enemy identities, waves, traits and
  ability programs, or carry an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Goal 13 row enters another mode or production runtime bundle;
- Goal 03 evidence and current production configuration remain unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 13 denominator with no
  blocking research case;
- every completed batch commit is reachable from its recorded remote branch at
  the recorded full commit ID; and
- clean-checkout acceptance passes and `G13-P4-B4` is committed and pushed.

Progress is recorded in
[the Goal 13 status ledger](13-anomaly-arbitration-reference-data-status.md).
