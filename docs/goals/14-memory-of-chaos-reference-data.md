# Goal 14 — Memory of Chaos Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Memory of Chaos reference-data pack
and an isolated Excel/Sora authoring surface before any Memory of Chaos runtime
implementation.

The goal covers the stable Forgotten Hall/Memory of Chaos family contract and
the active released Version 4.4 season: entry and unlock locators, stage and
node flow, participant and loadout locks, attempts and retries, shared cycle
budget, wave boundaries, objectives and stars, Memory Turbulence, initial
resources, exact encounters, enemies and every battle-visible or cross-battle
rule contribution. It also resolves the active `TierceID` extension without
assuming whether it is a third node, a separate stage or another lifecycle
until released selector and configuration evidence proves that meaning.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated readers,
coverage and semantic review fixtures. It does not implement or expose a
playable Memory of Chaos profile.

## Start condition

Goal 14 may run while other Goals or repository-integration work are active in
other worktrees. It is unblocked when:

- Goal 03 remains `Complete`;
- both pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- the executor uses the dedicated
  `codex/goal14-memory-of-chaos-reference` branch and a separate worktree from
  every other active Goal checkout; and
- Memory of Chaos artifacts use the isolated paths declared below.

Completion of another mode Goal is not a prerequisite for source collection,
manifest freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 14 must not modify another Goal's plans, ledgers, manifests,
normalized rows, workbooks, generated output, runtime lowering or shared
combat/activity/challenge behavior.

If research discovers a missing shared runtime primitive or a mismatch between
the Version 4.4 data and the current two-node challenge design, record it as a
future Memory of Chaos runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and source-reproduction prerequisite

- game/content snapshot: Version 4.4;
- planning audit date: 2026-07-30;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- challenge architecture baseline:
  `docs/18-standard-and-challenge-modes.md`;
- content and evidence baseline:
  `docs/15-content-data-and-coverage.md`;
- cross-mode ownership inputs: committed Goal 08 through 13 manifests at the
  revisions inspected by `G14-P0-B1`, when available;
- public cross-check access dates: recorded per page during Goal 14 rather than
  silently inheriting an earlier research date.

The planning audit found both ignored caches clean and detached at the required
commits, verified their configured origins, proved required commit/blob
readability and ran Git connectivity checks. The turnbasedgamedata cache is
intentionally sparse. `G14-P0-B1` must reproduce these checks without changing
another Goal's sparse checkout or treating planning-time availability as
batch-owned release evidence.

## Starting source oracle

The pinned structured source contains these dedicated or directly relevant
Memory/Forgotten Hall tables:

```text
ChallengeGeneralConfig.json
ChallengeGroupConfig.json
ChallengeMazeConfig.json
ChallengeMazeGroupExtra.json
ChallengeMazeRewardLine.json
ChallengeMazeTierce.json
ChallengeTargetConfig.json
ConstValueChallengeClient.json
ConstValueChallengeCommon.json
ScheduleDataChallengeMaze.json
```

The focused inventory must also audit these adjacent challenge tables to prove
inclusion or exclusion rather than relying on their names:

```text
ChallengeActMark.json
ChallengeActivityConfig.json
ChallengeBadgeConfig.json
ChallengeSkipConfig.json
ChallengeBoss*.json
ChallengePeak*.json
ChallengeStory*.json
```

Reward, badge, skip and schedule rows are evidence-only unless they prove an
enabled mechanical selector, objective, shortcut or lifecycle. Account reward
payloads and quick-clear history remain excluded.

The planning audit identified these shared table and program seeds:

```text
BattleEventConfig.json
MapEntrance.json
MapEntranceGroup.json
MapEntranceUnlock.json
MappingInfo.json
MazeBuff.json
StageConfig.json
MonsterConfig.json
MonsterTemplateConfig.json
MonsterSkillConfig.json
MonsterStatusConfig.json
TextMap/TextMapCHS.json
TextMap/TextMapEN.json
Config/ConfigAbility/BattleEventAbility_2.json
Config/ConfigAbility/Level/Level_MazeChallengeBuff_Ability.json
Config/ConfigAbility/StageBattleEventAbility.json
Config/Level/StageCommonTemplate.json
```

Every selected enemy must add its transitive character configuration, ability,
AI, skill, status, summon, linked-actor and phase files to the closure. Layout,
camera and presentation companions are evidence-only unless a mechanically
enabled program references them.

The active-season planning hypothesis is:

- `ScheduleDataChallengeMaze` row `201033`, active from 2026-07-06 through
  2026-08-17 in the pinned released data;
- `ChallengeGroupConfig` row `1033`, bilingual title
  `学院怪谈` / `Academy Ghost Story`;
- `ChallengeMazeConfig` rows `5201` through `5212`, each with two ordinary
  StageConfig bindings;
- `ChallengeMazeTierce` candidate `5213`, explicitly selected by group
  `1033`, with StageConfig candidate `30123123`;
- active MazeBuff `3030146`, objectives `251` through `253`, and Battle Event
  `30146`;
- twenty-four ordinary StageConfig rows `30123011` through `30123122`, plus
  candidate `30123123`, each explicitly carrying
  `_CreateBattleEvent = 30146`; and
- `BattleEventAbility_Challenge_Month_46` in
  `Config/ConfigAbility/BattleEventAbility_2.json`.

This chain is a planning seed, not a frozen denominator. `G14-P0-B3` must prove
active Version 4.4 release membership, the exact role of `5213`, every
StageConfig join and the exclusion of scheduled-but-not-yet-released group
`1034`. Table prefixes, numeric adjacency, matching parameter vectors, schedule
presence and display-name equality are never sufficient ownership or
reachability evidence by themselves.

## Included content

Completeness is defined by frozen manifests for:

1. stable Forgotten Hall/Memory of Chaos family identity, active Version 4.4
   season identity, entry, mechanically relevant unlocks and terminal outcomes;
2. every active floor/stage, node or side, predecessor relation, legal order,
   retry/reset boundary and the exact role of the selected Tierce record;
3. participant pools, team slots, character/combat-form uniqueness, loadout
   snapshots, Light Cone and Relic-instance lock scope, substitutions and
   between-node mutation rules;
4. attempt creation, node transition, accepted/rejected start, failure,
   abandonment, retry and completion lifecycles;
5. initial cycle budget, first-cycle Action Value window, tick boundaries,
   shared or independent scope, node carry, wave carry/reset, expiry and
   failure timing;
6. exact initial HP, Energy, Skill Points, Technique/battle-entry effects and
   any stage-owned resource overrides selected by the active season;
7. completion, survival/downed-character and remaining-cycle objectives,
   per-objective evaluation timing, stars and stage aggregation;
8. active Memory Turbulence identity, parameters, trigger phases, hit
   accumulation, cap, cycle-start execution, random target candidate set,
   True-DMG contribution and teardown;
9. every active `ChallengeMazeTierce` contribution, including any additional
   team, clock, objective, encounter, reward-free settlement or transition;
10. every active MazeBuff, BattleEvent, stage template, config program and
    ability operation reached from the season and stage selectors;
11. exact StageConfig encounters, ordered waves, enemy slots, concrete enemy
    variants, levels/difficulty inputs, skills, AI, abilities, summons, linked
    actors and boss phases;
12. reachable shared enemies and challenge definitions, plus distinct
    mode-owned copies where the active season owns different state, parameters
    or bindings;
13. Blessing, Curio, Occurrence, service, currency, shop, event-choice and
    other content-pool families, using generated exact-zero selector proofs
    when the active mode exposes none;
14. every battle-visible rule contribution, cross-battle Activity slot,
    decision, result projection, RNG candidate set and lifecycle boundary;
15. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions; and
16. source-path, stable row-locator and evidence-digest reconciliation receipts
    for every shared row overlapping another Goal.

Shared content is referenced by stable Starclock identity only after the frozen
Memory of Chaos manifest proves reachability. A source copy with a
season-specific effect, state, stage binding or display binding remains a
distinct Memory of Chaos-owned record.

## Excluded content

- Memory of Chaos runtime lowering, handlers, controllers, CLI, Agent, MCP or
  playable challenge flow;
- changes to `starclock-combat`, `starclock-activity`,
  `starclock-mode-challenge`, `starclock-build` or shared runtime semantics;
- static Forgotten Hall content beyond the minimum evidence needed to prove the
  shared family boundary;
- Pure Fiction, Apocalyptic Shadow, Anomaly Arbitration, Simulated Universe
  modes and other gameplay data except as explicit ownership or exclusion
  evidence;
- historical Memory of Chaos seasons and scheduled-but-unreleased seasons
  except as bounded active-period or exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, badges, item payloads, first-clear rewards, star
  rewards, account history, quick-clear state and collection displays;
- calendar services and wall-clock rotation behavior beyond immutable season
  identity metadata;
- leaks, beta/test-server data, previews and announced-but-unavailable content;
- a seeded runtime golden activity or production compatibility claim.

Quest, reward, schedule, mapping and historical-period rows may be retained as
provenance locators when necessary to prove entry, unlocks, active-season
selection, stage relationships or a mechanical shortcut. Their story prose,
account state, calendar behavior and reward payloads do not become normalized
runtime content.

## Architecture and artifact isolation

Goal 14 starts in the `Experimental` content lane and may finish with a complete
`Candidate` reference bundle. It does not promote a `Released` production mode.

Use isolated paths:

```text
content-manifests/memory-of-chaos-v1/
content-reference/memory-of-chaos-v1/
config/memory-of-chaos/
config/memory-of-chaos-generated/
tools/memory-of-chaos-reference/
evidence/memory-of-chaos-reference-v1/
```

The authoring workbooks are mode-owned:

```text
MemoryOfChaos.xlsx
MemoryOfChaosBindings.xlsx
MemoryOfChaosReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G14-P0-B4`. They must not share mutable sheets or generated directories with
another Goal or `config/generated/`.

Memory of Chaos authoring may reference generic Activity, challenge,
participant, clock, objective, BattleSpec/Result and stable shared-content
concepts. It may not redefine graph execution, command atomicity, build
resolution, combat formulas, RNG, hashing or replay. Reference rows may declare
a future activity, build or battle rule contribution without implementing its
evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released configuration/ability programs;
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

Approximation is field-level. Active-season selection, Tierce semantics,
participant uniqueness, loadout invalidation, first-cycle Action Value, cycle
and wave carry, objective timing, Turbulence hit attribution, random target
ordering, True-DMG parameters, caps, rounding and fallbacks must never be
silently inferred. A deterministic project policy requires known facts,
selected behavior, rejected alternatives, rationale, affected fixtures and a
concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G14-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, season, schedule selector, entry, unlock and terminal outcome;
- floor/stage, node/side, Tierce record, predecessor, attempt and transition;
- participant policy, team slot, roster/loadout snapshot, uniqueness and lock;
- cycle clock, AV window, tick, node/wave carry, expiry and failure;
- initial resource and battle-entry contribution;
- target, objective, evaluation, star and aggregation;
- Memory Turbulence, MazeBuff, BattleEvent, ability and rule contribution;
- encounter, StageConfig wave, enemy slot, variant, skill, status, AI, summon,
  linked actor and boss phase;
- audited Blessing, Curio, Occurrence, service, currency, shop and event-choice
  membership, including proven-empty categories;
- mechanic rule, source, reconciliation receipt, research gap, coverage,
  review fixture and pack index.

Definitions remain separate from mutable attempts, selections, clocks, waves,
records and terminal results. Exact decimals use canonical strings. Arrays that
are sets are sorted by stable ID; node order, waves, target priority and effect
programs preserve declared order.

## Parallel execution rules

When Goal 14 runs beside other Goals or main-workspace integration:

- use the dedicated branch and separate worktree;
- Goal 14 owns only the six isolated roots declared above, its three Goal
  documents and its Goal index row;
- do not edit another Goal's plan, ledger, manifest, policy, evidence,
  workbook, normalized content or generated output;
- do not regenerate another mode's generated directory or
  `config/generated/`;
- do not copy uncommitted main-workspace integration changes into this branch;
- do not reclassify a shared source record in place; record Goal 14 ownership
  and reachability in the Memory of Chaos manifest;
- reconcile overlaps by source path, stable row locator and evidence digest at
  the named Phase 4 checkpoint;
- record incompatible ownership or semantic classifications and wait for merge
  coordination rather than overwriting another task; and
- preserve Goal 03 and current production/other-mode bundle digests.

Reference collection, source hashing, manifests, normalized data, evidence,
fixtures, isolated schemas and isolated workbook QA may proceed in parallel.
Runtime lowering and changes to shared schemas or operations belong to a later
goal after the reference pack is frozen.

## Delivery phases

### Phase 0 — Scope, source files, manifests and contracts

| Batch | Deliverable |
|---|---|
| `G14-P0-B1` | Reproduce both pinned caches, verify Goal 03, freeze scope/exclusions, inspect all active worktree/Goal boundaries and prove branch/path isolation. |
| `G14-P0-B2` | Generate the focused inventory for dedicated/adjacent Challenge tables, entry mappings, CHS/EN TextMaps, StageConfig, MazeBuff/BattleEvent, config/ability programs, enemies and named exclusions. |
| `G14-P0-B3` | Freeze active Version 4.4 season selectors, exact per-category IDs/counts, Tierce reachability, ownership, shared closure, proven-empty pools and historical/future/other-mode exclusions. |
| `G14-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, three-workbook/table families, reconciliation receipts and semantic fixture contracts. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G14-P1-B1` | Import profile, active season, entry/unlock locators, floors/stages, ordinary nodes, Tierce identity, legal order and terminal outcomes. |
| `G14-P1-B2` | Import participant/team slots, roster and loadout uniqueness, snapshots, locks, substitutions, attempt/retry/reset and node-transition rules. |
| `G14-P1-B3` | Import cycle budget, first-cycle AV, tick boundaries, node/wave carry or reset, expiry, failure and retry timing. |
| `G14-P1-B4` | Import completion/survival/remaining-cycle targets, objective evaluation, stars and aggregation. |
| `G14-P1-B5` | Import Memory Turbulence, MazeBuff/BattleEvent selection, hit accumulation, cap, cycle-start execution, target policy, True DMG and teardown. |
| `G14-P1-B6` | Import initial resources, battle-entry operations, Tierce-specific contributions and every cross-battle projection. |

### Phase 2 — Content pools, services, events and enemies

| Batch | Deliverable |
|---|---|
| `G14-P2-B1` | Audit Blessing, Curio, Occurrence, service, currency, shop, choice and related pools; freeze each reachable set or generated exact-zero proof. |
| `G14-P2-B2` | Import enabled challenge definitions, shared stage templates, MazeBuffs, BattleEvents and config/ability relationships. |
| `G14-P2-B3` | Import exact StageConfig encounters, ordered waves, enemy slots, variants, levels and difficulty bindings for every ordinary and Tierce node. |
| `G14-P2-B4` | Import transitive enemy skills, statuses, AI, abilities, summons, linked actors, boss phases and battle-visible rule contributions. |
| `G14-P2-B5` | Generate mechanics, sources, coverage, research gaps, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking gap. |

### Phase 3 — Isolated Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G14-P3-B1` | Add isolated profile/season/stage/node/Tierce/participant/attempt Sora tables and typed references. |
| `G14-P3-B2` | Add clock/resource/objective/star/Turbulence/event/contribution tables. |
| `G14-P3-B3` | Add audited pool, encounter, wave, enemy and transitive mechanic binding tables. |
| `G14-P3-B4` | Add provenance, coverage, approximation, reconciliation, fixture and pack-index tables; generate isolated locks/templates/readers. |
| `G14-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for all three complete workbooks with validation and semantic QA. |
| `G14-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every sheet. |

### Phase 4 — Ownership audit, fixtures, reconciliation and freeze

| Batch | Deliverable |
|---|---|
| `G14-P4-B1` | Audit every manifest row, season selector, Tierce binding, reference, ownership, bilingual field, provenance and quality label; reject historical/future/other-mode leaks. |
| `G14-P4-B2` | Execute semantic fixtures for every mode system, lifecycle, objective, Turbulence contribution, encounter policy and proven-empty pool; verify every replacement condition. |
| `G14-P4-B3` | Reconcile shared rows with committed Goal 07–13 facts, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G14-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution and commit rules

- Select the earliest unblocked batch and keep only one Goal 14 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(memory-of-chaos): <batch-id> <imperative summary>`.
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
- concrete active-season category manifests provide 100% exact-once
  accounting;
- every enabled record has bilingual names/summaries and resolvable
  provenance;
- every numeric vector, selector, relationship and lifecycle is exact or
  explicitly approximate/policy-bound;
- all mode-owned/shared/evidence-only/historical/future/other-mode
  classifications are explicit and fail closed;
- Tierce identity, topology, participant scope, clock and encounter semantics
  are proved rather than inferred from its name or obfuscated fields;
- any empty Blessing, Curio, Occurrence, service, currency, shop or choice
  family is proved by selector closure rather than assumed from the mode;
- participant/loadout locks, attempts, clocks, objectives, stars, initial
  resources and Memory Turbulence have semantic fixtures;
- all encounters resolve concrete released enemy identities, waves, AI and
  ability programs, or carry an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Goal 14 row enters another mode or production runtime bundle;
- Goal 03 evidence and current production/other-mode configurations remain
  unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 14 denominator with no
  blocking research case;
- every completed batch commit is reachable from its recorded remote branch at
  the recorded full commit ID; and
- clean-checkout acceptance passes and `G14-P4-B4` is committed and pushed.

Progress is recorded in
[the Goal 14 status ledger](14-memory-of-chaos-reference-data-status.md).
