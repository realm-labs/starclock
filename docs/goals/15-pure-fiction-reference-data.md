# Goal 15 — Pure Fiction Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Pure Fiction reference-data pack and
an isolated Excel/Sora authoring surface before any Pure Fiction runtime
implementation.

The goal covers the stable Pure Fiction family contract and the released season
active during the frozen Version 4.4 snapshot: entry and unlock locators,
stages and nodes, participant and loadout locks, attempts and retries, turn and
wave lifecycle, continuous enemy replacement, scoring and objectives,
Whimsicality, selectable Cacophony, the current Grit/Fever system, exact
encounters, enemies and every battle-visible or cross-battle rule
contribution. It also resolves the selected `ChallengeStoryMazeTierce` and
Starward-mode extensions without assuming their topology, team, clock or
settlement meaning before released selector and program evidence proves it.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated
readers, coverage and semantic review fixtures. It does not implement or
expose a playable Pure Fiction profile.

## Start condition

Goal 15 may run while other Goals or repository-integration work are active in
other worktrees. It is unblocked when:

- Goal 03 remains `Complete`;
- both pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- the executor uses the dedicated
  `codex/goal15-pure-fiction-reference` branch and a separate worktree from
  every other active Goal checkout; and
- Pure Fiction artifacts use the isolated paths declared below.

Completion of another mode Goal is not a prerequisite for source collection,
manifest freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 15 must not modify another Goal's plans, ledgers, manifests,
normalized rows, workbooks, generated output, runtime lowering or shared
combat/activity/challenge behavior.

If research discovers a missing shared runtime primitive or a mismatch between
the released Version 4.4 data and the current challenge design, record it as a
future Pure Fiction runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and source-reproduction prerequisite

- game/content snapshot: Version 4.4;
- planning audit date: 2026-07-30;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- official version boundary: HoYoLAB
  [Version 4.4 update details](https://www.hoyolab.com/article/45851903),
  accessed 2026-07-30;
- released active-season cross-check: HoYoLAB
  [Version 4.3 update details](https://www.hoyolab.com/article/45284705),
  accessed 2026-07-30;
- challenge architecture baseline:
  `docs/18-standard-and-challenge-modes.md`;
- content and evidence baseline:
  `docs/15-content-data-and-coverage.md`;
- cross-mode ownership inputs: committed Goal 07 through 14 artifacts at the
  revisions inspected by `G15-P0-B1`, when available; and
- further public cross-check access dates: recorded per page during Goal 15
  rather than silently inheriting an earlier research date.

The planning audit found both ignored caches clean and detached at the required
commits, verified their configured origins, proved required commit readability
and ran Git connectivity checks. The `turnbasedgamedata` cache is intentionally
sparse. `G15-P0-B1` must reproduce these checks without changing another
Goal's sparse checkout or treating planning-time availability as batch-owned
release evidence.

## Starting source oracle

The pinned structured source contains these dedicated Pure Fiction tables:

```text
ChallengeStoryGroupConfig.json
ChallengeStoryGroupExtra.json
ChallengeStoryMazeConfig.json
ChallengeStoryMazeExtra.json
ChallengeStoryMazeTierce.json
ChallengeStoryRewardLine.json
ChallengeStoryTargetConfig.json
ChallengeStoryTheme.json
ScheduleDataChallengeStory.json
```

The focused inventory must also audit adjacent challenge tables, including the
following families, to prove inclusion or exclusion rather than relying on
their names:

```text
ChallengeGeneralConfig.json
ChallengeActMark.json
ChallengeActivityConfig.json
ChallengeBadgeConfig.json
ChallengeSkipConfig.json
ChallengeMaze*.json
ChallengeBoss*.json
ChallengePeak*.json
ConstValueChallengeClient.json
ConstValueChallengeCommon.json
```

Reward, badge, skip and schedule rows are evidence-only unless they prove an
enabled mechanical selector, objective, shortcut or lifecycle. Account reward
payloads, calendar presentation and quick-clear history remain excluded.

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
Config/ConfigAbility/BattleEvent/FantasticStoryHard_Ability_6.json
Config/ConfigAbility/BattleEvent/FantasticStoryHard_Ability_7.json
Config/ConfigAbility/BattleEvent/FantasticStoryHard_Scoring_Ability.json
Config/ConfigAbility/BattleEvent/FantasticStory_Environment_Ability.json
Config/ConfigAbility/BattleEvent/FantasticStory_Scoring_Ability.json
Config/ConfigAbility/BattleEvent/FantasticStory_Wave_Ability.json
Config/ConfigAbility/Level/Level_MazeChallengeBuff_Ability.json
Config/ConfigAbility/StageBattleEventAbility.json
Config/Level/StageCommonTemplate.json
```

`G15-P0-B2` must enumerate every `FantasticStory*` ability program and prove
whether it is selected by the active profile, retained only as historical
evidence or excluded. Every selected enemy must add its transitive character
configuration, ability, AI, skill, status, summon, linked-actor and phase files
to the closure. Layout, camera and presentation companions are evidence-only
unless a mechanically enabled program references them.

The active-season planning hypothesis is:

- `ScheduleDataChallengeStory` row `202024`, active from 2026-06-22 04:00
  through 2026-08-03 04:00 in the pinned structured data;
- `ChallengeStoryGroupConfig` row `2024`, bilingual title
  `借虚成真` / `Falsehood to Fact`;
- `ChallengeStoryMazeConfig` rows `20241` through `20244`, each with two
  ordinary StageConfig bindings;
- `ChallengeStoryMazeTierce` candidate `20245`, explicitly selected by group
  `2024`, with StageConfig candidate `30322043`;
- ordinary StageConfig candidates `30322011`, `30322012`, `30322021`,
  `30322022`, `30322031`, `30322032`, `30322041` and `30322042`;
- group MazeBuff `3031220`, stage MazeBuff `3031225`, theme `4`,
  `StoryType = Fever`, sub-buffs `3031227`–`3031229` and selectable buffs
  `3031359`, `3031362`, `3031361`;
- target rows `2001`–`2003`, whose structured score thresholds are 40,000,
  50,000 and 60,000;
- stage-extra turn limits `5`, `5`, `4`, `4` and per-stage clear-score seed
  `30,000`; and
- direct MazeBuff binding keys resolving to base abilities `2200`, `2250`,
  `2260` and selectable abilities `2261`, `2264`, `2263` in
  `FantasticStoryHard_Ability_6.json` and
  `FantasticStoryHard_Ability_7.json`.

The official Version 4.3 update identifies `Falsehood to Fact` as released for
the same 2026-06-22 through 2026-08-03 period and describes its initial Grit
trigger. The official Version 4.4 update establishes the 2026-07-15 version
boundary, so the released season remains active in the Version 4.4 snapshot.

This chain is a planning seed, not a frozen denominator. `G15-P0-B3` must prove
the exact active-release selector, Starward/Tierce meaning, every StageConfig
join and the exclusion of group `2025`, whose structured schedule begins only
on 2026-08-03 after the fixed planning access date. Table prefixes, numeric
adjacency, matching parameter vectors, schedule presence and display-name
equality are never sufficient ownership or reachability evidence by
themselves.

## Included content

Completeness is defined by frozen manifests for:

1. stable Pure Fiction family identity, active Version 4.4 season identity,
   entry, mechanically relevant unlocks and terminal outcomes;
2. every released difficulty/stage, node or side, predecessor relation, legal
   order, retry/reset boundary and the exact role of the selected Tierce and
   Starward records;
3. participant pools, team slots, character/combat-form uniqueness, loadout
   snapshots, Light Cone and Relic-instance lock scope, substitutions and
   between-node mutation rules;
4. attempt creation, Cacophony selection, node transition, accepted/rejected
   start, failure, abandonment, retry and completion lifecycles;
5. turn or cycle budget, Action Value windows, tick boundaries, node scope,
   wave transitions, enemy refill timing, early completion and timeout;
6. enemy roster queues, continuous replacement order, simultaneous defeats,
   spawn-slot reuse, final-group behavior and no-legal-spawn policy;
7. exact initial HP, Energy, Skill Points, Technique/battle-entry effects and
   any stage-owned resource overrides selected by the active season;
8. enemy defeat and damage scoring, per-enemy and per-node caps, attribution,
   objective timing, stage/season aggregation, stars and terminal settlement;
9. active Whimsicality and Grit/Fever identities, parameters, trigger filters,
   per-target caps, transitions, effects, target policies and teardown;
10. every selectable Cacophony identity, eligibility, choice scope, parameters,
    timing, battle contribution and interaction with the base seasonal rule;
11. every active `ChallengeStoryMazeTierce` and Starward contribution,
    including any additional team, node, clock, objective, encounter or
    reward-free settlement behavior;
12. every active theme, MazeBuff, BattleEvent, stage template, config program
    and ability operation reached from the season and stage selectors;
13. exact StageConfig encounters, ordered waves, enemy slots, concrete enemy
    variants, levels/difficulty inputs, skills, AI, abilities, summons,
    linked actors and boss phases;
14. reachable shared enemies, events and challenge definitions, plus distinct
    mode-owned copies where the active season owns different state, parameters
    or bindings;
15. Blessing, Curio, Occurrence, service, currency, shop, event-choice and
    other content-pool families, using generated exact-zero selector proofs
    when the active mode exposes none;
16. every battle-visible rule contribution, cross-battle Activity slot,
    decision, result projection, RNG candidate set and lifecycle boundary;
17. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions; and
18. source-path, stable row-locator and evidence-digest reconciliation receipts
    for every shared row overlapping another Goal.

Shared content is referenced by stable Starclock identity only after the frozen
Pure Fiction manifest proves reachability. A source copy with a
season-specific effect, state, stage binding or display binding remains a
distinct Pure Fiction-owned record.

## Excluded content

- Pure Fiction runtime lowering, handlers, controllers, CLI, Agent, MCP or
  playable challenge flow;
- changes to `starclock-combat`, `starclock-activity`,
  `starclock-mode-challenge`, `starclock-build` or shared runtime semantics;
- Memory of Chaos, Apocalyptic Shadow, Anomaly Arbitration, Simulated Universe
  modes and other gameplay data except as explicit ownership or exclusion
  evidence;
- historical Pure Fiction seasons and scheduled-but-unreleased seasons except
  as bounded active-period or exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, badges, item payloads, first-clear rewards,
  account payouts and collection-completion rewards;
- reproduction of presentation timing, camera behavior, input handling or
  calendar rotation;
- leaks, beta/test-server dumps, previews, announced-but-unavailable content
  and any row not proven released at the fixed access boundary;
- a seeded runtime golden activity or production compatibility claim.

Reward and presentation tables may be retained as provenance locators when
necessary to prove a mechanical unlock, score threshold, star target or
selector. Their prose, asset paths and account-reward payloads do not become
normalized runtime content.

## Architecture and artifact isolation

Goal 15 starts in the `Experimental` content lane and may finish with a
complete `Candidate` reference bundle. It does not promote a `Released`
production mode.

Use isolated paths:

```text
content-manifests/pure-fiction-v1/
content-reference/pure-fiction-v1/
config/pure-fiction/
config/pure-fiction-generated/
tools/pure-fiction-reference/
evidence/pure-fiction-reference-v1/
```

The authoring workbooks are mode-owned:

```text
PureFiction.xlsx
PureFictionBindings.xlsx
PureFictionReview.xlsx
```

The exact workbook/table partition and output paths are frozen in
`G15-P0-B4`. They must not share mutable sheets or generated directories with
any existing mode workbook, `config/generated/`,
`config/universe-generated/`, another mode's generated root or production
configuration.

Pure Fiction authoring may reference generic Activity/challenge concepts and
stable shared content IDs. It may not redefine Activity graph execution,
participant identity, scopes, command atomicity, BattleSpec/Result, combat
formulas, RNG, hashing or replay. Reference rows may declare a future Activity
or battle rule contribution without implementing its evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released ability/config programs;
2. official publisher update notes and released in-game text;
3. reproducible live Version 4.4 observations;
4. independent public community cross-checks.

Every normalized fact records the repository/URL, exact revision or access
date, game version, relative path/page, row locator, evidence digest, quality,
mechanism quality and note. Allowed labels remain:

- `ExactStructured`;
- `ExactPublicText`;
- `Observed`;
- `ApproximateFromReleasedText`;
- `ProjectPolicy`.

Approximation is field-level. Hidden spawn ordering, score attribution,
simultaneous-defeat handling, timing, caps, rounding, target selection and
fallback behavior must never be silently inferred. A deterministic project
policy requires the known facts, selected behavior, rejected alternatives,
rationale, affected fixtures and a concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G15-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, active season, entry, unlock and terminal result;
- stage, node/side, Tierce/Starward extension and transition;
- participant, team slot, loadout lock, attempt, retry and reset;
- clock/turn budget, wave, spawn queue, replacement and completion;
- score source, score rule, cap, objective, star and aggregation;
- theme, Whimsicality, Grit/Fever state, threshold and transition;
- Cacophony choice, eligibility, effect and lifecycle;
- initial resource, battle-entry operation and cross-battle projection;
- challenge definition, MazeBuff, BattleEvent, config/ability contribution;
- content-pool reachability or exact-zero proof;
- encounter, StageConfig wave, enemy slot, variant, AI and ability closure;
- mechanic rule, source, coverage, reconciliation, research gap, semantic
  fixture and pack index.

Definitions remain separate from mutable state, levels, choices, attempts and
season-owned bindings. Exact decimals use canonical strings. Arrays that are
sets are sorted by stable ID; waves, spawn programs, ability operations and
other semantic sequences preserve declared order.

## Parallel execution rules

When Goal 15 and any other Goal execute concurrently:

- use separate Git branches and worktrees;
- Goal 15 owns only the six isolated artifact roots declared above, its three
  goal documents and its `docs/goals/README.md` index row;
- do not edit another Goal's plan, ledger, policy, evidence, manifest,
  normalized data, workbook or generated output;
- do not edit existing Standard, challenge or Simulated Universe data in place;
- do not regenerate another mode's generated directory or production config;
- do not reclassify a shared record in place; record proposed Pure Fiction
  ownership and reachability in the Goal 15 manifest;
- compare every overlapping shared record by source path, stable row locator
  and evidence digest at `G15-P4-B3`;
- record incompatible ownership or semantic classifications and wait for merge
  coordination rather than overwriting another Goal;
- preserve Goal 03 release evidence and current production bundle identities;
  and
- record missing runtime capability as later-goal work without changing
  runtime code here.

Reference collection, source hashing, manifest construction, normalized data,
evidence, fixtures, isolated schemas and isolated workbook QA may proceed in
parallel. Runtime lowering and changes to shared schemas or operations belong
to a later goal.

## Delivery phases

### Phase 0 — Scope, sources, manifest and contracts

| Batch | Deliverable |
|---|---|
| `G15-P0-B1` | Reproduce both pinned caches, verify Goal 03 and concurrent boundaries, freeze the released Version 4.4 scope/exclusions and prove path isolation. |
| `G15-P0-B2` | Generate a focused inventory covering dedicated/adjacent tables, entry mappings, `FantasticStory*` and shared config/ability programs, CHS/EN TextMaps, StageConfig, enemies and exclusions. |
| `G15-P0-B3` | Freeze active-release selectors, exact per-category manifest IDs/counts, Tierce/Starward ownership, shared reachability, exact-zero pool proofs and scheduled/unreleased exclusions. |
| `G15-P0-B4` | Freeze normalized schema, canonical encoding, evidence/quality fields, three-workbook partition, reconciliation receipts and semantic fixture contracts. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G15-P1-B1` | Import profile, active season, entry/unlocks, four released stage records, ordinary nodes, Tierce/Starward identity, legal order and terminal outcomes. |
| `G15-P1-B2` | Import participants, team/loadout uniqueness, snapshots/locks, substitutions, attempts, retries, abandonment, reset and transitions. |
| `G15-P1-B3` | Import turn/AV clocks, node scope, wave boundaries, continuous spawn queues, replacement order, timeout and early final-group completion. |
| `G15-P1-B4` | Import defeat/damage score sources, attribution, caps, simultaneous outcomes, objectives, stars and stage/season aggregation. |
| `G15-P1-B5` | Import Whimsicality and current Grit/Fever gain, thresholds, states, effects, target rules, duration, transition order and teardown. |
| `G15-P1-B6` | Import every selectable Cacophony, choice timing/scope, eligibility, parameters, base-rule interaction and battle contribution. |
| `G15-P1-B7` | Import initial resources, battle entry, cross-battle projections and every Tierce/Starward-specific lifecycle contribution not owned by earlier batches. |

### Phase 2 — Content pools, services, events and enemies

| Batch | Deliverable |
|---|---|
| `G15-P2-B1` | Freeze reachable or selector-proven exact-zero Blessing, Curio, Occurrence and event-choice pools. |
| `G15-P2-B2` | Freeze reachable or selector-proven exact-zero services, currencies, shops and other cross-battle content pools. |
| `G15-P2-B3` | Import enabled challenge definitions, themes, MazeBuffs, BattleEvents, stage templates and config/ability relationships. |
| `G15-P2-B4` | Import exact StageConfig encounters, ordered waves, spawn slots, enemy variants, levels and difficulty bindings. |
| `G15-P2-B5` | Import enemy skills, statuses, AI, abilities, summons, linked actors, boss phases and battle-visible contributions. |
| `G15-P2-B6` | Generate mechanic rules, sources, coverage, research-gap register, semantic fixtures and canonical pack index; close or policy-bind every nonblocking evidence gap. |

### Phase 3 — Independent Sora and Excel authoring

| Batch | Deliverable |
|---|---|
| `G15-P3-B1` | Add isolated profile/season/stage/node/Tierce/participant/attempt Sora tables and typed references. |
| `G15-P3-B2` | Add clock/spawn/score/objective/star/Whimsicality/Grit/Cacophony/resource tables. |
| `G15-P3-B3` | Add pool, event, MazeBuff, encounter, wave, enemy and mechanic-binding tables without duplicating shared semantics. |
| `G15-P3-B4` | Add provenance, coverage, approximation, reconciliation, review-fixture and pack-index tables; generate isolated schema lock, templates and readers. |
| `G15-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for all three complete workbooks with validation, filters, panes and semantic QA. |
| `G15-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every authored sheet and schema column. |

### Phase 4 — Ownership audit, fixtures, reconciliation and freeze

| Batch | Deliverable |
|---|---|
| `G15-P4-B1` | Audit every manifest row, active-release selector, reference, ownership, bilingual field, provenance and quality label; reject other-mode, historical and unreleased leakage. |
| `G15-P4-B2` | Execute semantic fixtures for every distinct lifecycle, spawn, score, Grit/Fever, Cacophony and Tierce/Starward policy; verify every approximation replacement condition. |
| `G15-P4-B3` | Reconcile shared rows by source path/locator/digest, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G15-P4-B4` | Freeze documentation, machine counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution and commit rules

- Select the earliest unblocked batch and keep only one Goal 15 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(pure-fiction): <batch-id> <imperative summary>`.
- Push every completed batch commit to the configured remote branch
  immediately after the commit succeeds. Do not begin the next batch while the
  current batch exists only locally.
- Record the remote, branch, full pushed commit ID, push command and
  remote-resolution verification in the status ledger. A batch is not
  `Complete` until its commit is reachable from the recorded remote branch.
- Use the pinned released source first and public pages only for release
  boundaries, meaning and unresolved observations.
- Use Python `openpyxl` for workbook creation and inspection. Sora 0.3.0
  remains the validation, code-generation and export authority.
- Bootstrap complete workbooks into clean targets; never patch a
  designer-edited `.xlsx` or edit one as a ZIP.
- JSON is research/bootstrap/debug data and never a runtime loading path.
- Record exact commands, counts, digests, decisions, research outcomes,
  reconciliation receipts and replacement conditions in the status ledger.
- Keep generated denominators machine-derived. Never reduce a denominator or
  convert an unproved absence into zero to make coverage pass.

## Acceptance

- both pinned source caches and the Pure Fiction-focused inventory regenerate
  byte-identically;
- concrete category manifests provide 100% exact-once accounting;
- the active released Version 4.4 season and the scheduled/unreleased boundary
  are explicit and fail closed;
- every enabled record has bilingual names/summaries and resolvable
  provenance;
- every numeric vector, spawn relation, score rule, choice, unlock and
  lifecycle is exact or explicitly approximate/policy-bound;
- all PureFiction-owned/shared/evidence-only/excluded classifications are
  explicit and fail closed;
- every empty Blessing, Curio, Occurrence, service, currency, shop and choice
  family has a generated selector-closure zero proof;
- all participant/loadout, attempt, clock, spawn/refill, score, objective,
  Whimsicality, Grit/Fever, Cacophony and Tierce/Starward families have
  semantic fixtures;
- all encounters resolve exact StageConfig rows, waves, enemy identities, AI
  and abilities, or carry an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Pure Fiction row enters another mode or production runtime bundle;
- Goal 03 evidence and current production configuration remain unchanged;
- every shared row reconciles against committed overlapping Goal facts or has
  a recorded merge-coordination conflict;
- coverage reports 100% `DataReady` for the frozen Goal 15 denominator with no
  unresolved blocking research case;
- every batch commit, including `G15-P4-B4`, is reachable from the recorded
  remote branch; and
- the full clean-checkout release gate passes and `G15-P4-B4` is committed and
  pushed.

Progress is recorded in
[the Goal 15 status ledger](15-pure-fiction-reference-data-status.md).
