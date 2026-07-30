# Goal 11 — Divergent Universe Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Divergent Universe reference-data
pack and an isolated Excel/Sora authoring surface before any Divergent Universe
runtime implementation.

The goal covers released Divergent Universe modules and stage flow, Ordinary
and Cyclical Extrapolation, difficulties and Threshold Protocols, Arithmetic
Mapping, Equations and expansion, Divergent Blessings, Curios and Weighted
Curios, Grand Miracles/Hex effects, Golden Blood's Boons and Titan progression
where mechanically relevant, workbench and gamble services, mechanically
relevant permanent progression, content pools, encounters and every
battle-visible or cross-battle rule contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated
readers, coverage and semantic review fixtures. It does not implement or
expose a playable Divergent Universe profile.

## Start condition

Goal 11 may run while Goal 07 is completing Standard Simulated Universe
mechanics and Goals 08 through 10 are collecting mode reference data. It is
unblocked when:

- Goal 03 remains `Complete`;
- both pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- the executor uses a separate branch and git worktree from every other active
  Goal checkout; and
- Divergent Universe artifacts use the isolated paths declared below.

Goal 07, 08, 09 or 10 completion is not a prerequisite for source collection,
manifest freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 11 must not modify their plans, ledgers, manifests, normalized
rows, workbooks, generated output, runtime lowering or shared combat/activity
behavior.

If research discovers a missing shared runtime primitive, record it as a future
Divergent Universe runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and source-reproduction prerequisite

- game/content snapshot: Version 4.4;
- planning audit date: 2026-07-29;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- existing audit baseline:
  `content-manifests/standard-universe-v1/source-inventory.json`;
- cross-mode ownership inputs: committed Goal 08, 09 and 10 manifests at the
  revisions inspected by `G11-P0-B1`, when available;
- public cross-check access dates: recorded per page during Goal 11 rather than
  silently inheriting an earlier research date.

The planning audit found clean local caches at both required commits, proved
the commit objects readable, verified each configured origin and ran Git
connectivity checks. `G11-P0-B1` must reproduce those checks in its own
worktree before the first Goal data mutation. Planning-time availability is not
batch-owned release evidence.

## Starting source oracle

The pinned structured cache contains these sixty-four focused `RogueTourn`
tables:

```text
RogueTournAdventureRoom.json
RogueTournArea.json
RogueTournAreaGroup.json
RogueTournAreaGroupByTourn.json
RogueTournAvatar.json
RogueTournBuff.json
RogueTournBuffGroup.json
RogueTournBuffType.json
RogueTournBuildRefAvatar.json
RogueTournCocoonConfig.json
RogueTournCollection.json
RogueTournCollectionConfig.json
RogueTournConstClient.json
RogueTournConstCommon.json
RogueTournContentDisplay.json
RogueTournCurseChest.json
RogueTournDifficulty.json
RogueTournDifficultyComp.json
RogueTournDivision.json
RogueTournDivisionEffect.json
RogueTournExhibition.json
RogueTournExhibitionConfig.json
RogueTournExpReward.json
RogueTournExpScore.json
RogueTournExpScore_Index_ScoreExpID.json
RogueTournFinishway.json
RogueTournFormula.json
RogueTournFormulaAeonIcon.json
RogueTournFormulaDisplay.json
RogueTournFormulaRandom.json
RogueTournGambleGroup.json
RogueTournGambleUnit.json
RogueTournHandBookEvent.json
RogueTournHandbookMiracle.json
RogueTournHex.json
RogueTournHexAvatarBaseType.json
RogueTournHexDisplay.json
RogueTournKeyword.json
RogueTournKeywordParam.json
RogueTournLayer.json
RogueTournLayerRoom.json
RogueTournMiracle.json
RogueTournMiracleDisplay.json
RogueTournMiracleGroup.json
RogueTournMiracleGroupTest.json
RogueTournMiracleTest.json
RogueTournMiscDisplay.json
RogueTournModule.json
RogueTournNPC.json
RogueTournPermanentTalent.json
RogueTournRecordShowcase.json
RogueTournRole.json
RogueTournRoom.json
RogueTournRoomGroup.json
RogueTournRoomMark.json
RogueTournTitanBless.json
RogueTournTitanTalent.json
RogueTournTitanType.json
RogueTournUnlock.json
RogueTournUseBuffType.json
RogueTournWeeklyChallenge.json
RogueTournWeeklyDisplay.json
RogueTournWorkbench.json
RogueTournWorkbenchFunc.json
```

These files are mode-source candidates, not the final content denominator.
Some are presentation, test, collection or account-reward locators; some rows
belong to earlier or later mode modules. A `RogueTourn` prefix, `TournMode`
label, ID range or matching name does not prove that a row is mechanically
enabled, reachable in Version 4.4 or owned by the selected module.

The initial direct configuration and ability-program seeds include:

```text
Config/ConfigAbility/Level/Level_RogueBuff_Ability_Tourn1.json
Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S1.json
Config/ConfigAbility/Level/Level_RogueBuff_Ability_HEX_S3.json
Config/Level/Rogue/RogueModifier/RogueTournGodMode/
Config/Level/Rogue/RogueDialogue/RogueEventTourn1/
Config/Level/Rogue/RogueNPC/RogueNPC_230/
Config/Level/Rogue/RogueNPC/RogueNPC_310/
Config/Level/Rogue/RogueNPC/RogueNPC_330/
Config/Level/Rogue/RogueNPC/RogueNPC_380/
Config/Level/Rogue/RogueNPC/RogueNPC_410/
Config/Level/Rogue/RogueTitanWeight/
```

The `.layout.json` companions are evidence inputs, not independent mechanic
rows. `G11-P0-B2` must discover every configuration program transitively
referenced by enabled areas, rooms, NPCs, Equations, Blessings, Curios,
Weighted Curios, Hex effects, Golden Blood's Boons, workbench functions,
enemies and encounters. The inventory must also include:

- `TextMap/TextMapCHS.json` and `TextMap/TextMapEN.json`;
- applicable bilingual `StarRailRes` simulated Blessing, Curio and event
  indexes;
- concrete `ExcelOutput/StageConfig.json` rows selected from enabled
  area/layer/room/group and NPC relationships;
- reachable `MonsterConfig`, monster group, wave, skill/status and enemy
  ability records; and
- every transitively invoked level, battle-event, modifier, maze, group
  template and ability program.

Candidate shared source families include `RogueBuff`, `RogueBuffGroup`,
`RogueBuffType`, `RogueMazeBuff`, `RogueMiracle`,
`RogueMiracleDisplay`, `RogueMiracleEffect`, `RogueMiracleGroup`,
`RogueEventSpecialOption`, `RogueDialogueOption`, `RogueNPC`, `RogueRoom`,
`RogueRoomType`, `RogueAdventureRoom`, `RogueShop`, `RogueMonster`,
`RogueMonsterGroup` and their referenced base definitions. They become Goal 11
obligations only through an explicit enabled-module selector, a transitive
source reference or an inherited stable-ID closure.

Every shared-source reconciliation record uses source path, stable row locator
and evidence digest. The manifest must classify each discovered row as
`DivergentUniverse`, `Shared`, `EvidenceOnly` or an explicitly named excluded
mode/module. File presence, prefix, numerical adjacency and name equality are
never sufficient ownership or reachability evidence.

## Included content

Completeness is defined by frozen manifests for:

1. mode entry, unlocks, initial resources, module/version selection and
   terminal outcomes;
2. Ordinary and Cyclical Extrapolation, every released Version 4.4-enabled
   area, difficulty, layer, room, legal transition, finish condition and
   carry/reset rule;
3. Threshold Protocol and every Astronomical Division, Star-Pioneer Mode,
   Practice Mode and Cognoculus boundary that changes legal entry, difficulty
   composition, enemies, available mechanics or battle contributions;
4. Arithmetic Mapping for levels, Traces, Light Cones and Relics, including
   eligibility, temporary substitutions, refresh timing and teardown without
   mutating account inventory;
5. every Equation definition, category, Path recipe, required Blessing counts,
   offered pool, expansion state, effect parameters, replacement and discard
   behavior;
6. Divergent Blessing definitions, enhanced states, categories, Path
   relationships, conversion/rewrite behavior and Equation contribution;
7. all Curios, Weighted Curios, state copies, charges, destruction, repair,
   replacement, weighting/eligibility rules and deterministic fallbacks;
8. every Grand Miracle/Hex definition, character/Path/element eligibility,
   effect, activation, duration, interaction and teardown;
9. every Golden Blood's Boon, Titan type, mechanically relevant Titan talent,
   offered choice, level/state, activation and run/battle contribution;
10. workbench and gamble functions, Equation/Blessing/Curio rewrite,
    enhancement, synthesis and replacement operations, currencies, prices,
    candidate sets and no-legal-result policies;
11. mechanically relevant permanent talent, Inspiration Circuit, unlock,
    weekly/cyclical modifier and module row that changes run choices, content
    availability, services, mapped builds or battle state;
12. rooms, domains/marks, NPCs, services, chests and other offered decisions,
    including their state scope and lifecycle;
13. mode-owned and reachable shared Blessings, Curios, Occurrences, services,
    currencies and candidate pools;
14. Adventure outcome tiers as abstract external outcomes, without reproducing
    movement, aiming, physics or timing input;
15. encounter pools, exact enemy variants, StageConfig rows, waves,
    elite/boss choices, final bosses and difficulty/module bindings;
16. every battle-visible rule contribution, cross-battle state slot, RNG
    candidate set and lifecycle boundary;
17. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions.

Shared content is referenced by stable Starclock identity only after the frozen
Divergent Universe manifest proves reachability. A source copy with a
mode-specific effect, state, pool rule, mapped build or display binding remains
a distinct Divergent Universe-owned record.

## Excluded content

- Divergent Universe runtime lowering, handlers, controllers, CLI, Agent, MCP
  or full playable runs;
- changes to `starclock-activity`, `starclock-combat` or shared runtime
  semantics;
- Standard Simulated Universe, Swarm Disaster, Gold and Gears, Unknowable
  Domain, Currency Wars and historical temporary modes except as explicit
  ownership/exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, collection rewards, fitting-level rewards,
  first-clear/account payouts, weekly score rewards and planar-ornament
  extraction rewards;
- reproduction of Adventure/action-minigame movement, aiming, physics or
  timing input;
- announced, beta, preview, leaked or otherwise unreleased content;
- a seeded runtime golden activity or production compatibility claim.

Story, collection, display, showcase, score, reward and planar-ornament
extraction tables may be retained as provenance locators when necessary to
prove a mechanical unlock, module boundary, stage relationship or offered
choice. Their prose, presentation and account-reward payloads do not become
normalized runtime content.

## Architecture and artifact isolation

Goal 11 starts in the `Experimental` content lane and may finish with a
complete `Candidate` reference bundle. It does not promote a `Released`
production mode.

Use isolated paths:

```text
content-manifests/divergent-universe-v1/
content-reference/divergent-universe-v1/
config/divergent-universe/
config/divergent-universe-generated/
tools/divergent-universe-reference/
evidence/divergent-universe-reference-v1/
```

The authoring workbooks are mode-owned:

```text
DivergentUniverse.xlsx
DivergentUniverseBindings.xlsx
DivergentUniverseReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G11-P0-B4`. They must not share mutable sheets or generated directories with
Standard Universe, Gold and Gears, Swarm Disaster or Unknowable Domain
workbooks, `config/universe-generated/`,
`config/gold-and-gears-generated/`, `config/swarm-disaster-generated/`,
`config/unknowable-domain-generated/` or `config/generated/`.

Divergent Universe authoring may reference generic Activity concepts and
stable shared content IDs. It may not redefine Activity graph execution,
scopes, command atomicity, BattleSpec/Result, build resolution, combat
formulas, RNG, hashing or replay. Reference rows may declare a future activity,
build or battle rule contribution without implementing its evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released ability/configuration programs;
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

Approximation is field-level. Hidden weights, candidate ordering, activation
timing, simultaneous triggers, caps, rounding, fallback behavior, build
substitution details or stage/pool probabilities must never be silently
inferred. A deterministic project policy requires the known facts, selected
behavior, rejected alternatives, rationale, affected fixtures and a concrete
replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G11-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, module, entry, difficulty, protocol, area, layer, room and finish
  condition;
- Cyclical Extrapolation challenge and modifier binding;
- Arithmetic Mapping eligibility, temporary build and teardown;
- Equation definition, recipe, category, offer, progress, expansion, effect and
  replacement;
- Blessing definition/level/category, Path relationship, rewrite and Equation
  contribution;
- Curio/Weighted Curio definition, state, weight eligibility and lifecycle;
- Grand Miracle/Hex definition, eligibility, activation and state;
- Golden Blood's Boon, Titan type/talent, choice, level and contribution;
- workbench/gamble function, operation, currency, price and candidate policy;
- permanent talent/Inspiration Circuit, unlock and mechanically relevant
  progression;
- NPC, room mark, service, chest, occurrence and choice;
- encounter pool, group, StageConfig wave, enemy slot and boss;
- mechanic rules, sources, reconciliation receipts, coverage, review fixtures
  and pack index.

Definitions remain separate from levels, mutable states, variants, offers,
temporary builds and unlock conditions. Exact decimals use canonical strings.
Arrays that are sets are sorted by stable ID; stage paths, choice/effect
programs and other semantic sequences preserve declared order.

## Parallel execution rules

When Goals 07 through 11 execute concurrently:

- use separate git worktrees and branches;
- Goal 11 owns only the isolated paths declared above plus its three Goal
  documents and index row;
- do not edit Goal 07, 08, 09 or 10 plans, ledgers, manifests, policies,
  evidence, workbooks or content partitions;
- do not edit existing Standard, Gold and Gears, Swarm Disaster or Unknowable
  Domain normalized rows or generated output;
- do not regenerate any other mode's generated directory or
  `config/generated/`;
- do not reclassify a shared source record in place; record Goal 11 ownership
  and reachability in the Divergent Universe manifest;
- compare overlapping source rows by source path, stable row locator and
  evidence digest at the named reconciliation checkpoint;
- record incompatible ownership or semantic classifications and wait for merge
  coordination rather than overwriting another goal; and
- preserve Goal 03 and current production bundle digests.

Reference collection, source hashing, manifest construction, normalized data,
evidence, fixtures, isolated schemas and isolated workbook QA may proceed fully
in parallel. Runtime lowering and changes to shared schemas or operations
belong to a later goal after the relevant reference packs are frozen.

## Delivery phases

### Phase 0 — Scope, source files, manifests and contracts

| Batch | Deliverable |
|---|---|
| `G11-P0-B1` | Reproduce both pinned caches, verify Goal 03, freeze Goal 11 scope/exclusions, inspect active Goal 08/09/10 ownership boundaries and prove branch/worktree/path isolation. |
| `G11-P0-B2` | Generate a focused source inventory covering all 64 `RogueTourn` tables, direct and transitive configuration/ability programs, CHS/EN TextMaps, StageConfig rows, shared Rogue/enemy data and named exclusions. |
| `G11-P0-B3` | Freeze exact per-category manifest IDs/counts, enabled Version 4.4 modules, ownership, shared reachability and the Divergent versus Standard/Gold/Swarm/Unknowable/other-module exclusion boundary. |
| `G11-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, workbook/table families, reconciliation receipts and semantic review-fixture contracts. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G11-P1-B1` | Import profile/modules, Ordinary and Cyclical entry, difficulties, areas, layers, rooms, legal stage flow, finish and carry/reset boundaries. |
| `G11-P1-B2` | Import Arithmetic Mapping eligibility, temporary level/Trace/Light Cone/Relic substitutions, refresh timing and teardown boundaries. |
| `G11-P1-B3` | Import every Equation definition, category, Path recipe, required counts, offer pool, progress, expansion, effect and replacement lifecycle. |
| `G11-P1-B4` | Import Divergent Blessing categories/levels, Path bindings, conversion/rewrite/enhancement rules and Equation contribution semantics. |
| `G11-P1-B5` | Import Curio and Weighted Curio eligibility, weighting, states, charges, destruction/repair/replacement and Grand Miracle/Hex behavior. |
| `G11-P1-B6` | Import Golden Blood's Boons, Titan types, mechanically relevant Titan talents, choices, levels/states, activation and rule contributions. |
| `G11-P1-B7` | Import Threshold Protocol, Astronomical Division, Star-Pioneer/Practice Mode, Cognoculi, difficulty composition, enemy changes, unlock boundaries and battle-visible modifiers. |
| `G11-P1-B8` | Import workbench/gamble operations, currencies, prices, offers, Equation/Blessing/Curio transformations and no-legal-result policies. |
| `G11-P1-B9` | Import mechanically relevant permanent talents/Inspiration Circuit, unlocks, weekly/cyclical modifiers, room marks, services and cross-battle contributions. |

### Phase 2 — Content pools, services, events and enemies

| Batch | Deliverable |
|---|---|
| `G11-P2-B1` | Freeze reachable shared versus mode-owned Blessing, enhanced-level, Path, Equation and related buff pool membership. |
| `G11-P2-B2` | Import every obtainable Curio/Weighted Curio identity and mode copy, complete state lifecycle and offer-pool binding. |
| `G11-P2-B3` | Import Occurrences, variants, conditional choice graphs, chests, costs and mechanical outcomes without presentation prose. |
| `G11-P2-B4` | Import currencies, shops/services, workbench/gamble bindings and abstract Adventure outcome tiers, prices, rewards and eligibility. |
| `G11-P2-B5` | Import encounter pools, exact enemy variants, StageConfig waves, elite/boss alternatives, final bosses and difficulty/module bindings. |
| `G11-P2-B6` | Generate mechanic rules, sources, coverage, research-gap register, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking evidence gap. |

### Phase 3 — Independent Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G11-P3-B1` | Add isolated profile/module/stage/difficulty/protocol/Arithmetic Mapping Sora tables and typed references. |
| `G11-P3-B2` | Add Equation, Blessing, Curio/Weighted Curio, Hex, Golden Blood/Titan, state and lifecycle tables. |
| `G11-P3-B3` | Add progression, workbench/gamble, service, occurrence, Adventure, encounter and mechanic binding tables. |
| `G11-P3-B4` | Add provenance, coverage, approximation, reconciliation, review-fixture and pack-index tables; generate isolated schema locks/templates/readers. |
| `G11-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for all three complete isolated workbooks with validation, filters, panes and semantic QA. |
| `G11-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every authored sheet. |

### Phase 4 — Ownership audit, fixtures, reconciliation and freeze

| Batch | Deliverable |
|---|---|
| `G11-P4-B1` | Audit every manifest row, reference, enabled-module/ownership classification, bilingual field, provenance and quality label; reject cross-mode and excluded-module leaks. |
| `G11-P4-B2` | Execute semantic fixtures for every distinct stage, Equation, Arithmetic Mapping, Blessing, Curio/Grand Miracle/Hex, Titan, protocol, service, lifecycle and pool policy; verify every approximation replacement condition. |
| `G11-P4-B3` | Reconcile overlapping source rows with committed Goal 08/09/10 facts, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G11-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution, commit and publication rules

- Select the earliest unblocked batch and keep only one Goal 11 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(divergent-universe): <batch-id> <imperative summary>`.
- Push every completed batch commit to the configured remote branch
  immediately after the commit succeeds. Do not begin the next batch while the
  current batch exists only locally.
- Verify the remote branch resolves to the same full commit ID. Record the
  remote, branch, pushed commit, exact push/verification commands and result in
  the status ledger. A batch is not `Complete` until it is reachable from the
  recorded remote branch.
- Use the pinned source first and public pages only for boundary checks,
  meaning and unresolved observations.
- Use Python `openpyxl` for workbook creation and inspection. Sora 0.3.0
  remains the validation, code-generation and export authority.
- Bootstrap complete workbooks into clean targets; never patch a
  designer-edited `.xlsx` or edit one as a ZIP.
- JSON is research/bootstrap/debug data and never a runtime loading path.
- Record exact commands, counts, digests, research decisions, reconciliation
  receipts and replacement conditions in the status ledger.
- Keep generated denominators machine-derived. Never reduce a denominator to
  make coverage pass.

## Acceptance

- both pinned source caches and the focused
  table/config/TextMap/Stage/ability inventory regenerate byte-identically;
- concrete category manifests provide 100% exact-once accounting for enabled
  Version 4.4 modules;
- every enabled record has bilingual names/summaries and resolvable
  provenance;
- every numeric vector, pool membership, unlock and relationship is exact or
  explicitly approximate/policy-bound;
- all mode-owned/shared/evidence-only/excluded classifications are explicit
  and fail closed;
- every shared source row agrees with committed Goal 08/09/10 ownership facts
  or has a recorded reconciliation decision;
- all stage flow, Arithmetic Mapping, Equation, Blessing, Curio/Weighted Curio,
  Grand Miracle/Hex, Golden Blood/Titan, protocol, workbench and service
  families have semantic fixtures;
- all encounters resolve concrete released enemy identities and waves, or
  carry an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Divergent Universe row enters another mode or the production runtime
  bundle;
- Goal 03 release evidence and the current production configuration remain
  unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 11 denominator with no
  unresolved blocking research case;
- every batch commit, including `G11-P4-B4`, is reachable from the recorded
  remote branch; and
- the clean-checkout release gate passes and `G11-P4-B4` is committed and
  pushed.

Progress is recorded in
[the Goal 11 status ledger](11-divergent-universe-reference-data-status.md).
