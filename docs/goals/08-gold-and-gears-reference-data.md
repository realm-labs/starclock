# Goal 08 — Gold and Gears Reference Data

## Objective

Prepare a complete, auditable Version 4.4 Simulated Universe: Gold and Gears
reference-data pack and an isolated Excel/Sora authoring surface before any
Gold and Gears runtime implementation.

The goal covers the mode-specific plane map, Cognition and Intra-Cognition,
customizable dice and all six face slots, dice categories and passives,
Knowledge, mechanically relevant Secrets and Neural Network input, Conundrum,
Path boosts, Resonance Extrapolation, Adventure outcomes, mode content pools,
encounters and every battle-visible or cross-battle rule contribution.

This is a reference-data goal. It ends with frozen manifests, normalized
mechanics, provenance, Candidate-quality authoring workbooks, generated readers,
coverage and semantic review fixtures. It does not implement or expose a
playable Gold and Gears profile.

## Start condition

Goal 08 may run while Goal 07 is still completing Standard Simulated Universe
mechanics. It is unblocked when:

- Goal 03 remains `Complete`;
- the Version 4.4 structured source revision and existing source inventory can
  be reproduced;
- the executor uses a separate branch and git worktree when Goal 07 is active
  in another checkout; and
- Gold and Gears artifacts use isolated paths and generated outputs.

Goal 07 completion is not a prerequisite for source collection, manifest
freezing, normalization, evidence work, isolated schemas or workbook
authoring. Goal 08 must not modify Goal 07 ledgers, Standard profile manifests,
Standard workbook rows, current production generated output, runtime lowering
or shared combat/activity behavior.

If research discovers a missing shared runtime primitive, record it as a
future Gold and Gears runtime prerequisite. Do not implement it in this goal.

## Frozen snapshot and starting oracle

- game/content snapshot: Version 4.4;
- inherited structured-source access date: 2026-07-22;
- structured released-data baseline: `Dimbreath/turnbasedgamedata` commit
  `fd978d6ef09f941fba644c731ab54abd6f7c3568`;
- identity/translation cross-check: `Mar-7th/StarRailRes` commit
  `7b349e39ee0f6f3bf814567995829b99c95e7a93` where applicable;
- existing audit baseline:
  `content-manifests/standard-universe-v1/source-inventory.json`;
- public cross-check access dates: recorded per page during Goal 08 rather than
  silently inheriting the Goal 03 research date.

The existing inventory already hashes these twenty-one focused `RogueNous`
tables:

```text
RogueNousAeon.json
RogueNousAeonCross.json
RogueNousConstValueClient.json
RogueNousConstValueCommon.json
RogueNousDiceBranch.json
RogueNousDiceBranchTag.json
RogueNousDiceBranchValue.json
RogueNousDiceSlot.json
RogueNousDiceSurface.json
RogueNousDifficultyLevel.json
RogueNousEndGameReward.json
RogueNousMainStory.json
RogueNousMiscDisplay.json
RogueNousMissionReward.json
RogueNousRoom.json
RogueNousStoryDisplay.json
RogueNousStoryReward.json
RogueNousSubStory.json
RogueNousSurfaceTag.json
RogueNousTalent.json
RogueNousValueAreaLimit.json
```

These tables are a starting inventory, not the final content denominator.
Goal 08 must also resolve reachable shared Rogue definitions, battle/level
ability programs, TextMap records, enemy stages and mode-specific copies in
base tables. Table presence alone is never proof of Gold and Gears pool
membership.

## Included content

Completeness is defined by frozen manifests for:

1. mode entry, unlocks, initial resources, Path choice and Gold and Gears
   Trailblaze Bonuses;
2. all five difficulties, their Cognition ranges, enemy/difficulty inputs and
   Conundrum unlock boundary;
3. three planes, generated columns, nodes, edges, legal movement, entry and
   terminal rules;
4. all reachable room definitions, domain kinds, beacons, node replacements,
   copies, blanking and boss-choice consequences;
5. Cognition and Intra-Cognition initial values, bounds, adjustments, timing,
   plane-end evaluation and state carry/reset rules;
6. mechanically relevant Trailblaze/Aeon Secret prerequisites and
   Intra-Cognition ranges;
7. every Custom Dice definition, category, initial effect, passive, unlock and
   selected-Path boost rule;
8. all six dice-face slots, slot colors/rarities, equip constraints and complete
   loadout validation;
9. every dice face, tag, target selector, effect, duration, unlock, reroll,
   cheat and no-legal-target policy;
10. Knowledge placement, propagation, consumption, movement interaction,
    countdown interaction and scope;
11. every Neural Network node that changes run decisions, content availability,
    services, dice behavior or battle state, including its prerequisite graph
    and exact effect;
12. every Stats and Auxiliary Conundrum level, composition rule, exact modifier,
    Berserk change and cap;
13. Path boosts, Gold and Gears Path/Resonance additions and all Resonance
    Extrapolation definitions, choices and boss behavior;
14. Gold and Gears-owned and reachable shared Blessings, enhanced levels,
    Curios/states, Occurrences/choices, currencies/services and candidate pools;
15. Adventure outcome tiers and rewards as abstract external outcomes, without
    reproducing movement, aiming, physics or timing input;
16. mode encounters, exact enemy variants, waves, elite/boss choices, final
    boss pools and difficulty bindings;
17. every battle-visible rule contribution, cross-battle state slot, RNG
    candidate set and lifecycle boundary;
18. bilingual names, independently written summaries, row-level provenance,
    field-level confidence and approximation replacement conditions.

Shared Standard content is referenced by stable Starclock identity only after
the frozen Gold and Gears manifest proves reachability. A source copy with a
mode-specific effect, state, pool rule or display binding remains a distinct
Gold and Gears-owned record.

## Excluded content

- Gold and Gears runtime lowering, controllers, CLI, Agent, MCP or full playable
  runs;
- changes to `starclock-activity`, `starclock-combat` or existing Standard
  Universe runtime behavior;
- Swarm Disaster, Unknowable Domain, Divergent Universe, Currency Wars and
  historical temporary modes except as explicit exclusion evidence;
- story dialogue, cutscenes, presentation sequences, assets, audio and UI;
- Stellar Jade, achievements, index rewards, first-clear/account payouts and
  collection-completion rewards;
- reproduction of Adventure/action-minigame movement, aiming, physics or timing
  input;
- announced, beta, preview, leaked or otherwise unreleased content;
- a seeded runtime golden activity or production compatibility claim.

Story and reward tables may be retained as provenance locators when necessary
to prove a mechanical unlock, prerequisite or choice. Their prose and
account-reward payloads do not become normalized runtime content.

## Architecture and artifact isolation

Goal 08 starts in the `Experimental` content lane and may finish with a
complete `Candidate` reference bundle. It does not promote a `Released`
production mode.

Use isolated paths:

```text
content-manifests/gold-and-gears-v1/
content-reference/gold-and-gears-v1/
config/gold-and-gears/
config/gold-and-gears-generated/
tools/gold-and-gears-reference/
evidence/gold-and-gears-reference-v1/
```

The authoring workbooks should be mode-owned, for example:

```text
GoldAndGears.xlsx
GoldAndGearsBindings.xlsx
GoldAndGearsReview.xlsx
```

The exact workbook names, table families and output paths are frozen in
`G08-P0-B4`. They must not share mutable sheets or generated directories with
`Universe.xlsx`, `UniverseBindings.xlsx`, `UniverseReview.xlsx`,
`config/universe-generated/` or `config/generated/`.

Gold and Gears authoring may reference generic Activity concepts and stable
Standard/shared content IDs. It may not redefine Activity graph execution,
scopes, command atomicity, BattleSpec/Result, combat formulas, RNG, hashing or
replay. Reference rows may declare a future activity or battle rule
contribution without implementing its evaluator.

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

Approximation is field-level. Hidden weights, target ordering, clamp timing,
rounding, fallback behavior or map-generation probabilities must never be
silently inferred. A deterministic project policy requires the known facts,
selected behavior, rejected alternatives, rationale, affected fixtures and a
concrete replacement condition.

Long descriptions, source programs, story text and assets are not committed.
Exact bilingual names and factual numeric relationships may be retained;
summaries are short and independently written.

## Normalized reference families

`G08-P0-B4` freezes the exact machine schema. At minimum, the normalized pack
must account for these families:

- profile, entry, difficulty and Trailblaze Bonus;
- plane, map generation, node, edge, room, domain and beacon;
- Cognition, Intra-Cognition adjustment and Secret condition;
- Custom Dice, dice category/passive, slot, face, tag, loadout and unlock;
- Knowledge and graph-mutation rules;
- Neural Network nodes, prerequisites, costs and effects;
- Stats/Auxiliary Conundrum definitions and levels;
- Path boosts, Resonance additions and Resonance Extrapolations;
- Blessings/levels, Curios/states, Occurrences/variants/choices and services;
- Adventure outcomes, encounter pools, groups, waves, enemy slots and bosses;
- mechanic rules, sources, coverage, review fixtures and pack index.

Definitions remain separate from levels, mutable states, variants, loadouts
and unlock conditions. Exact decimals use canonical strings. Arrays that are
sets are sorted by stable ID; map paths, effect programs and other semantic
sequences preserve declared order.

## Parallel execution rules

When Goal 07 and Goal 08 execute concurrently:

- use separate git worktrees and branches;
- Goal 08 owns only the isolated paths declared above plus its three goal
  documents and index row;
- do not edit Goal 07 plans, ledgers, policies, evidence or content partitions;
- do not edit existing Standard Universe reference/manifests/workbooks;
- do not regenerate `config/universe-generated/` or `config/generated/`;
- do not reclassify a shared record in place; record the proposed ownership in
  the Gold and Gears manifest and reconcile after Goal 07 at a named checkpoint;
- preserve Goal 03 and current production bundle digests;
- treat merge conflicts in shared documentation or tool policy as a stop-and-
  reconcile condition, not permission to overwrite the other goal.

Reference collection, source hashing, manifest construction, normalized data,
evidence, fixtures, isolated schemas and isolated workbook QA may proceed
fully in parallel. Runtime lowering and changes to shared schemas or operations
belong to a later goal after Goal 07 is complete.

## Delivery phases

### Phase 0 — Freeze scope, sources and contracts

| Batch | Deliverable |
|---|---|
| `G08-P0-B1` | Verify the Goal 03 snapshot, freeze Goal 08 scope and exclusions, prove parallel path isolation and finalize the execution package. |
| `G08-P0-B2` | Regenerate a Gold and Gears-focused source inventory from the pinned cache, including all `RogueNous`, reachable shared Rogue, TextMap, StageConfig and mechanic-evidence files. |
| `G08-P0-B3` | Freeze exact per-category manifest IDs/counts, ownership, shared reachability and the Gold and Gears versus other-mode exclusion boundary. |
| `G08-P0-B4` | Freeze normalized schemas, canonical encoding, evidence/quality fields, workbook/table families and semantic review-fixture contract. |

### Phase 1 — Unique mode systems

| Batch | Deliverable |
|---|---|
| `G08-P1-B1` | Import profile entry, five difficulties, planes, columns, nodes, edges, rooms, domains, beacons and boss-choice topology. |
| `G08-P1-B2` | Import Cognition/Intra-Cognition values, legal ranges, adjustments, plane-end evaluation, Secret thresholds and carry/reset rules. |
| `G08-P1-B3` | Import Custom Dice definitions, categories, initial/passive effects, selected-Path boost rules and unlocks. |
| `G08-P1-B4` | Import six-slot layouts, slot constraints, every dice face/tag/loadout rule, reroll/cheat behavior and exact effect parameters. |
| `G08-P1-B5` | Import Knowledge placement/propagation/consumption, graph mutations, movement/countdown interaction and deterministic target policies. |
| `G08-P1-B6` | Import the mechanically relevant Neural Network prerequisite graph, costs, run/service/dice effects and battle contributions. |
| `G08-P1-B7` | Import Stats and Auxiliary Conundrum levels, composition, caps, enemy modifiers and Berserk timing. |
| `G08-P1-B8` | Import Gold and Gears Trailblaze Bonuses `201`–`205`, Path boosts, Path/Resonance additions and Resonance Extrapolation behavior. |

### Phase 2 — Content pools, outcomes and encounters

| Batch | Deliverable |
|---|---|
| `G08-P2-B1` | Freeze reachable shared versus mode-owned Blessing, enhanced-level, Path and Resonance pool membership. |
| `G08-P2-B2` | Import all obtainable Curios, mode-specific copies, lifecycle states, charges, repair, replacement and pool rules. |
| `G08-P2-B3` | Import Occurrences, variants, conditional choice graphs, costs and mechanical outcomes without presentation prose. |
| `G08-P2-B4` | Import currencies, shops/services, beacons and abstract Adventure outcome tiers, prices, rewards and eligibility. |
| `G08-P2-B5` | Import encounter pools, exact enemy variants, StageConfig waves, elite/boss alternatives, final bosses and difficulty bindings. |
| `G08-P2-B6` | Generate mechanic rules, sources, coverage, research-gap register, semantic fixtures and canonical pack index; close or policy-resolve every nonblocking evidence gap. |

### Phase 3 — Isolated Sora schema and Excel authoring

| Batch | Deliverable |
|---|---|
| `G08-P3-B1` | Add isolated profile/topology/Cognition/dice/slot/face/Knowledge Sora tables and typed references. |
| `G08-P3-B2` | Add Secret/Neural Network/Conundrum/Path/Resonance Extrapolation and progression tables. |
| `G08-P3-B3` | Add content pool, Curio, Occurrence, service, Adventure, encounter and mechanic-rule binding tables without duplicating generic semantics. |
| `G08-P3-B4` | Add provenance, coverage, approximation, review-fixture and pack-index tables; generate isolated schema locks/templates/readers. |
| `G08-P3-B5` | Add deterministic no-overwrite `openpyxl` authoring for complete isolated workbooks with validation, filters, panes and semantic QA. |
| `G08-P3-B6` | Prove byte-identical double generation, Sora check/build/export/load and rendered visual inspection for every authored sheet. |

### Phase 4 — Review and freeze

| Batch | Deliverable |
|---|---|
| `G08-P4-B1` | Audit every manifest row, reference, ownership, bilingual field, provenance and quality label; reject Standard/Swarm/Unknowable/Divergent leaks. |
| `G08-P4-B2` | Execute semantic fixtures for every distinct mode mechanic, lifecycle and pool policy; verify every approximation replacement condition. |
| `G08-P4-B3` | Run source-cache regeneration, pack/workbook double regeneration, Sora drift, isolated-reader, dependency and clean-checkout acceptance. |
| `G08-P4-B4` | Freeze documentation, counters, coverage and release evidence; mark the reference goal complete while keeping the runtime profile unreleased. |

## Execution and commit rules

- Select the earliest unblocked batch and keep only one Goal 08 batch
  `InProgress` per worktree.
- Each batch owns its source facts, normalized rows, evidence, tests and ledger
  update as one responsibility-bounded commit.
- Commit subjects use
  `<type>(gold-gears): <batch-id> <imperative summary>`.
- Use the pinned source first and public pages only for boundary checks,
  meaning and unresolved observations.
- Use Python `openpyxl` for workbook creation and inspection. Sora 0.3.0 remains
  the validation, code-generation and export authority.
- Bootstrap complete workbooks into clean targets; never patch a designer-edited
  `.xlsx` or edit one as a ZIP.
- JSON is research/bootstrap/debug data and never a runtime loading path.
- Record exact commands, counts, digests, research decisions and replacement
  conditions in the status ledger.
- Keep generated denominators machine-derived. Never reduce a denominator to
  make coverage pass.

## Acceptance

- the pinned Gold and Gears source inventory regenerates byte-identically;
- concrete category manifests provide 100% exact-once accounting;
- every enabled record has bilingual names/summaries and resolvable provenance;
- every numeric vector, pool membership, unlock and relationship is exact or
  explicitly approximate/policy-bound;
- all mode-owned/shared/excluded classifications are explicit and fail closed;
- all Custom Dice, six-slot, dice-face, Cognition, Knowledge, Neural Network,
  Conundrum and Resonance Extrapolation families have semantic fixtures;
- all encounters resolve concrete released enemy identities and waves, or carry
  an explicit nonblocking reference boundary;
- isolated workbooks validate, regenerate, render and export through pinned
  Sora without drift;
- isolated generated readers load every exported row;
- no Gold and Gears row enters the Standard profile or production runtime
  bundle;
- Goal 03 release evidence and the current production configuration remain
  unchanged;
- coverage reports 100% `DataReady` for the frozen Goal 08 denominator with no
  unresolved blocking research case;
- the clean-checkout release gate passes and `G08-P4-B4` is committed.

Progress is recorded in
[the Goal 08 status ledger](08-gold-and-gears-reference-data-status.md).
