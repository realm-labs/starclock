# Goal 10 — Unknowable Domain Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Simulated Universe: Unknowable
Domain reference-data pack and an isolated Excel/Sora authoring surface before
any Unknowable Domain runtime implementation.

The goal covers the mode-specific stage flow, difficulties, Extrapolation
Alignments, Scepters, activation/charge/speed behavior, Component categories,
shapes and slots, Decision Components, insertion, removal, synthesis and
upgrades, workbench and gamble services, mechanically relevant progression,
mode content pools, encounters and every battle-visible or cross-battle rule
contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated
readers, coverage and semantic review fixtures. It does not implement or
expose a playable Unknowable Domain profile.

## Start condition

Goal 10 may run while Goal 07 is completing Standard Simulated Universe
mechanics and Goals 08 and 09 are collecting mode reference data. It is
unblocked when:

- Goal 03 remains `Complete`;
- both pinned Version 4.4 research caches can be reproduced at their exact
  revisions;
- the executor uses a separate branch and git worktree from every active Goal
  07, 08 or 09 checkout; and
- Unknowable Domain artifacts use isolated paths and generated outputs.

Goal 07, 08 or 09 completion is not a prerequisite for source collection,
manifest freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 10 must not modify their plans, ledgers, manifests, normalized
rows, workbooks, generated output, runtime lowering or shared combat/activity
behavior.

If research discovers a missing shared runtime primitive, record it as a
future Unknowable Domain runtime prerequisite. Do not implement it in this
goal.

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
- cross-mode ownership inputs: the committed Goal 08 and Goal 09 manifests at
  the revisions inspected by `G10-P0-B1`, when available;
- public cross-check access dates: recorded per page during Goal 10 rather than
  silently inheriting an earlier research date.

The planning audit found clean local caches at both required commits, proved
the commit objects readable and ran Git connectivity checks. `G10-P0-B1` must
repeat the documented cache bootstrap/revision/hash checks in its own
worktree before the first Goal data mutation. A cache that merely contains
similarly named files is not reproducible evidence.

## Starting source oracle

The existing Standard inventory hashes these thirty-two focused
`RogueMagic` tables:

```text
RogueMagicAdventureRoom.json
RogueMagicArea.json
RogueMagicConstClient.json
RogueMagicConstCommon.json
RogueMagicContentDisplay.json
RogueMagicDifficultyComp.json
RogueMagicDifficultyDrop.json
RogueMagicFinishway.json
RogueMagicGambleGroup.json
RogueMagicGambleUnit.json
RogueMagicLayer.json
RogueMagicLayerEffect.json
RogueMagicLayerRoom.json
RogueMagicMazeBuff.json
RogueMagicMiracle.json
RogueMagicMiracleDisplay.json
RogueMagicMiracleGroup.json
RogueMagicMiscDisplay.json
RogueMagicNPC.json
RogueMagicRoom.json
RogueMagicRoomMark.json
RogueMagicScepter.json
RogueMagicScepterDisplay.json
RogueMagicScore.json
RogueMagicStory.json
RogueMagicStyleTypeSelect.json
RogueMagicTalent.json
RogueMagicUnit.json
RogueMagicUnitDisplay.json
RogueMagicUnlock.json
RogueMagicWorkbench.json
RogueMagicWorkbenchFunc.json
```

These files are mode-owned source candidates, not the final content
denominator. A `RogueMagic` prefix, nearby ID range or matching display name
does not prove that every row is mechanically enabled, reachable or unique to
Unknowable Domain.

The initial released configuration-program inventory includes:

```text
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Magic.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Magic_DarkTeam.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Magic_LightTeam.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Module.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_NewMagic_DarkTeam.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Rune.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Staff.json
Config/ConfigAbility/Level/Level_RogueMagic_Ability_Stage.json
Config/Level/Rogue/RogueMagicPower/RogueMagicPower.json
```

Their `.layout.json` companions are evidence inputs, not independent mechanic
rows. The focused inventory must also close over referenced files under:

```text
Config/ConfigAdventureModifier/AdventureModifier_Rogue_RogueMagic.json
Config/ConfigCharacter/BattleEvent/Avatar_RogueMagic_*.json
Config/Level/GroupTemplateGraph/03_Rogue/RogueMagic260/
Config/Level/Maze/MazeRogue/Rogue260/
Config/Level/Rogue/RogueNPC/RogueNPC_260/
```

`G10-P0-B2` must discover and hash every reachable shared `Rogue*` definition,
`TextMap/TextMapCHS.json`, `TextMap/TextMapEN.json`, concrete
`ExcelOutput/StageConfig.json` rows, monster/enemy definitions, wave bindings
and transitively invoked ability/configuration programs. Candidate shared
families include base Rogue Blessing/buff, Curio/miracle, occurrence/dialogue,
room, NPC, shop and Adventure tables. They become Goal 10 obligations only
through an explicit Unknowable Domain selector, a transitive source reference
or an inherited stable-ID closure.

Every shared-source reconciliation record uses source path, stable row locator
and evidence digest. The manifest must classify each discovered row as
`UnknowableDomain`, `Shared`, `EvidenceOnly` or an explicitly named excluded
mode. File presence, table prefix, numerical adjacency and name equality are
never sufficient ownership or reachability evidence.

## Included content

Completeness is defined by frozen manifests for:

1. mode entry, unlocks, initial resources, roster/loadout boundary and terminal
   outcomes;
2. every released stage/area, difficulty composition, layer, room, legal
   transition, finish condition and carry/reset rule;
3. all four Extrapolation Alignments, their selection boundary, eligibility,
   offered pools and run/battle contributions;
4. every Scepter definition and level, function/style, base power, activation
   trigger, charge or speed behavior, effect range, slot layout and locked
   Component relationship;
5. every Component definition and level, category, type, rarity, shape,
   compatible slot/range, effect, parameter, style/alignment relationship and
   enabled pool;
6. every Decision Component choice, eligibility, ordering, one-time/repeating
   scope, mechanical outcome and fallback;
7. insertion, removal, replacement, slot validation, synthesis, upgrade,
   reroll/reforge and no-legal-result policies;
8. Scepter and Component activation order, energy/charge changes, speed/action
   ordering, simultaneous triggers, target selection and teardown;
9. workbench functions, gamble groups/units, currencies, prices, eligibility,
   offered candidate sets and deterministic failure/replacement behavior;
10. every mechanically relevant Talent/unlock/progression row that changes run
    choices, content availability, services, Scepters, Components or battle
    state;
11. layer effects, mode maze buffs, difficulty modifiers, score/finish rules
    and their battle-visible or cross-battle lifecycle;
12. Unknowable-owned and reachable shared Blessings, enhanced levels,
    Curios/states, Occurrences/choices, currencies/services and candidate
    pools;
13. Adventure outcome tiers and rewards as abstract external outcomes, without
    reproducing movement, aiming, physics or timing input;
14. mode encounters, exact enemy variants, StageConfig rows, waves,
    elite/boss choices, final bosses and difficulty bindings;
15. every battle-visible rule contribution, cross-battle state slot, RNG
    candidate set and lifecycle boundary;
16. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions.

Shared Standard content is referenced by stable Starclock identity only after
the frozen Unknowable Domain manifest proves reachability. A source copy with a
mode-specific effect, state, pool rule or display binding remains a distinct
Unknowable Domain-owned record.

## Excluded content

- Unknowable Domain runtime lowering, handlers, controllers, CLI, Agent, MCP or
  full playable runs;
- changes to `starclock-activity`, `starclock-combat` or shared runtime
  semantics;
- Standard Simulated Universe, Swarm Disaster, Gold and Gears, Divergent
  Universe, Currency Wars and historical temporary modes except as explicit
  ownership/exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, index rewards, first-clear/account payouts,
  score rewards and collection-completion rewards;
- reproduction of Adventure/action-minigame movement, aiming, physics or
  timing input;
- announced, beta, preview, leaked or otherwise unreleased content;
- a seeded runtime golden activity or production compatibility claim.

Story, display, score and reward tables may be retained as provenance locators
when necessary to prove a mechanical unlock, prerequisite, stage boundary or
choice. Their prose, presentation and account-reward payloads do not become
normalized runtime content.

## Architecture and artifact isolation

Goal 10 starts in the `Experimental` content lane and may finish with a
complete `Candidate` reference bundle. It does not promote a `Released`
production mode.

Use isolated paths:

```text
content-manifests/unknowable-domain-v1/
content-reference/unknowable-domain-v1/
config/unknowable-domain/
config/unknowable-domain-generated/
tools/unknowable-domain-reference/
evidence/unknowable-domain-reference-v1/
```

The authoring workbooks are mode-owned:

```text
UnknowableDomain.xlsx
UnknowableDomainBindings.xlsx
UnknowableDomainReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G10-P0-B4`. They must not share mutable sheets or generated directories with
Standard Universe, Gold and Gears or Swarm Disaster workbooks,
`config/universe-generated/`, `config/gold-and-gears-generated/`,
`config/swarm-disaster-generated/` or `config/generated/`.

Unknowable Domain authoring may reference generic Activity concepts and stable
shared content IDs. It may not redefine Activity graph execution, scopes,
command atomicity, BattleSpec/Result, combat formulas, RNG, hashing or replay.
Reference rows may declare a future activity or battle rule contribution
without implementing its evaluator.

## Evidence and quality policy

Evidence follows `docs/sources.md` and
`docs/content-reference/authoring-contract.md`.

Priority is:

1. pinned released structured rows and released ability programs;
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

Approximation is field-level. Hidden weights, target ordering, activation
timing, charge/speed ordering, caps, rounding, fallback behavior or stage/pool
probabilities must never be silently inferred. A deterministic project policy
requires the known facts, selected behavior, rejected alternatives, rationale,
affected fixtures and a concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G10-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, entry, difficulty, area, layer, room and finish condition;
- Extrapolation Alignment, selection, eligibility and pool binding;
- Scepter definition/level, function/style, power, activation and state;
- Component definition/level, category/type, shape, slot/range and effect;
- Decision Component choice, condition, outcome and scope;
- loadout, insertion/removal, replacement, synthesis, upgrade and reforge;
- workbench function, gamble group/unit, currency, price and offer policy;
- Talent/unlock/progression and layer/difficulty/score mechanical input;
- Blessings/levels, Curios/states, Occurrences/variants/choices and services;
- Adventure outcomes, encounter pools, groups, waves, enemy slots and bosses;
- mechanic rules, sources, reconciliation receipts, coverage, review fixtures
  and pack index.

Definitions remain separate from levels, mutable states, variants, loadouts,
offers and unlock conditions. Exact decimals use canonical strings. Arrays
that are sets are sorted by stable ID; stage paths, effect programs and other
semantic sequences preserve declared order.

## Parallel execution rules

When Goals 07 through 10 execute concurrently:

- use separate git worktrees and branches;
- Goal 10 owns only the isolated paths declared above plus its three Goal
  documents and index row;
- do not edit Goal 07, 08 or 09 plans, ledgers, policies, evidence or content
  partitions;
- do not edit existing Standard, Gold and Gears or Swarm Disaster manifests,
  normalized rows, workbooks or generated output;
- do not regenerate `config/universe-generated/`,
  `config/gold-and-gears-generated/`, `config/swarm-disaster-generated/` or
  `config/generated/`;
- do not reclassify a shared source record in place; record Goal 10 ownership
  and reachability in the Unknowable Domain manifest;
- compare overlapping source rows by source path, stable row locator and
  evidence digest at the named reconciliation checkpoint;
- record incompatible ownership or semantic classifications and wait for merge
  coordination rather than overwriting another goal;
- preserve Goal 03 and current production bundle digests.

Reference collection, source hashing, manifest construction, normalized data,
evidence, fixtures, isolated schemas and isolated workbook QA may proceed fully
in parallel. Runtime lowering and changes to shared schemas or operations
belong to a later goal after the relevant reference packs are frozen.

## Delivery phases

### Phase 0 — Scope, source files, manifests and contracts

| Batch | Deliverable |
|---|---|
| `G10-P0-B1` | Reproduce both pinned caches, verify Goal 03, freeze Goal 10 scope/exclusions, inspect active Goal 08/09 ownership boundaries and prove parallel path isolation. |
| `G10-P0-B2` | Generate a focused source inventory covering all `RogueMagic` tables, configuration programs, TextMaps, StageConfig rows, reachable shared Rogue/enemy data and transitive ability files. |
| `G10-P0-B3` | Freeze exact per-category manifest IDs/counts, ownership, shared reachability and the Unknowable versus Standard/Gold/Swarm/Divergent exclusion boundary. |
| `G10-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, workbook/table families, reconciliation receipts and semantic review-fixture contracts. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G10-P1-B1` | Import profile entry, areas, difficulties, layers, rooms, legal stage flow, finish conditions and carry/reset boundaries. |
| `G10-P1-B2` | Import all four Extrapolation Alignments, selection/eligibility, offered pools and run/battle contributions. |
| `G10-P1-B3` | Import Scepter definitions/levels, functions/styles, power, activation, charge/speed behavior, effect ranges and state lifecycle. |
| `G10-P1-B4` | Import every Component definition/level, category/type, shape, compatible slot/range, effect parameters and style/alignment binding. |
| `G10-P1-B5` | Import Decision Component choices, loadout validation, insertion/removal/replacement and deterministic no-legal-option policies. |
| `G10-P1-B6` | Import Component synthesis, upgrades, rerolls/reforges, costs, candidate ordering, caps and replacement behavior. |
| `G10-P1-B7` | Import workbench functions, gamble groups/units, currencies, prices, offers, eligibility and service lifecycle. |
| `G10-P1-B8` | Import mechanically relevant Talents/unlocks, layer effects, maze buffs, difficulty/score inputs and every cross-battle or battle rule contribution. |

### Phase 2 — Content pools, services, events and enemies

| Batch | Deliverable |
|---|---|
| `G10-P2-B1` | Freeze reachable shared versus mode-owned Blessing, enhanced-level and alignment-related pool membership. |
| `G10-P2-B2` | Import all obtainable Curios, mode-specific copies, lifecycle states, charges, repair, replacement and pool rules. |
| `G10-P2-B3` | Import Occurrences, variants, conditional choice graphs, Decision Component outcomes, costs and mechanical results without presentation prose. |
| `G10-P2-B4` | Import currencies, shops/services, workbench/gamble pool bindings and abstract Adventure outcome tiers, prices, rewards and eligibility. |
| `G10-P2-B5` | Import encounter pools, exact enemy variants, StageConfig waves, elite/boss alternatives, final bosses and difficulty bindings. |
| `G10-P2-B6` | Generate mechanic rules, sources, coverage, research-gap register, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking evidence gap. |

### Phase 3 — Independent Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G10-P3-B1` | Add isolated profile/area/layer/room/difficulty/Alignment Sora tables and typed references. |
| `G10-P3-B2` | Add Scepter, activation/state, Component, shape/slot, loadout and Decision Component tables. |
| `G10-P3-B3` | Add synthesis/upgrade/reforge, Talent/unlock, workbench/gamble/service and mechanic binding tables. |
| `G10-P3-B4` | Add content pool, Curio, Occurrence, Adventure, encounter, provenance, coverage, reconciliation, fixture and pack-index tables; generate isolated schema locks/templates/readers. |
| `G10-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for all three complete isolated workbooks with validation, filters, panes and semantic QA. |
| `G10-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every authored sheet. |

### Phase 4 — Ownership audit, fixtures, reconciliation and freeze

| Batch | Deliverable |
|---|---|
| `G10-P4-B1` | Audit every manifest row, reference, ownership, bilingual field, provenance and quality label; reject Standard/Gold/Swarm/Divergent leaks. |
| `G10-P4-B2` | Execute semantic fixtures for every distinct stage, Alignment, Scepter, Component, service, lifecycle and pool policy; verify every approximation replacement condition. |
| `G10-P4-B3` | Reconcile overlapping source rows with committed Goal 08/09 facts, then run source-cache, pack/workbook, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G10-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution, commit and publication rules

- Select the earliest unblocked batch and keep only one Goal 10 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(unknowable-domain): <batch-id> <imperative summary>`.
- Push every completed batch commit to the configured remote branch
  immediately after the commit succeeds. Do not begin the next batch while the
  current batch exists only locally.
- Verify the remote branch resolves to the same commit ID. Record the remote,
  branch, pushed commit ID, exact push/verification commands and result in the
  status ledger. A batch is not `Complete` until it is reachable from the
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

- both pinned source caches and the focused Unknowable Domain inventory
  regenerate byte-identically;
- concrete category manifests provide 100% exact-once accounting;
- every enabled record has bilingual names/summaries and resolvable
  provenance;
- every numeric vector, pool membership, unlock and relationship is exact or
  explicitly approximate/policy-bound;
- all mode-owned/shared/excluded classifications are explicit and fail closed;
- every shared source row agrees with committed Goal 08/09 ownership facts or
  has a recorded reconciliation decision;
- all stage, Alignment, Scepter, activation/charge/speed, Component,
  slot/loadout, Decision Component, synthesis/upgrade and service families
  have semantic fixtures;
- all encounters resolve concrete released enemy identities and waves, or
  carry an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Unknowable Domain row enters another mode or the production runtime
  bundle;
- Goal 03 release evidence and the current production configuration remain
  unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 10 denominator with no
  unresolved blocking research case;
- every batch commit, including `G10-P4-B4`, is reachable from the recorded
  remote branch;
- the clean-checkout release gate passes and `G10-P4-B4` is committed.

Progress is recorded in
[the Goal 10 status ledger](10-unknowable-domain-reference-data-status.md).
